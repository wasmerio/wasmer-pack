use anyhow::{Context as _, Error};
use std::{
    ffi::OsStr,
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use flate2::Compression;
use tar::Header;
use wit_pack::{Abi, Files, Interface, Library, Metadata, Module, Package};

enum Execute {
    HardCodedCommand(String),
    Callback(Box<dyn FnOnce(&Ctx) -> String>),
}

pub struct TestCase {
    name: String,
    target: Target,
    package: Package,
    checks: Vec<Execute>,
}

impl TestCase {
    pub fn new(name: &str, target: Target, package: Package) -> Self {
        TestCase {
            name: name.to_string(),
            target,
            package,
            checks: Vec::new(),
        }
    }

    pub fn execute(mut self, cmd: impl Into<String>) -> Self {
        self.checks.push(Execute::HardCodedCommand(cmd.into()));
        self
    }

    pub fn callback(mut self, cmd: impl FnOnce(&Ctx) -> String + 'static) -> Self {
        self.checks.push(Execute::Callback(Box::new(cmd)));
        self
    }

    pub fn out_dir(&self) -> PathBuf {
        Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(env!("CARGO_PKG_NAME"))
            .join(&self.name)
    }

    pub fn tarball_filename(&self) -> String {
        let Metadata {
            package_name,
            version,
            ..
        } = self.package.metadata();

        format!("{package_name}-{version}.tar.gz")
    }

    pub fn run(self) {
        let bindings = self.target.generate(&self.package).unwrap();

        let tarball = self.save_tarball(bindings).unwrap();

        let test_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join(&self.name);

        assert!(
            test_dir.exists(),
            "The \"{}\" directory should exist",
            test_dir.display()
        );

        let ctx = Ctx {
            tarball: pathdiff::diff_paths(&tarball, &test_dir)
                .expect("Unable to get the tarball path relative to the test directory"),
            name: self.target.package_name(&self.package),
        };

        for exec in self.checks {
            let command = match exec {
                Execute::HardCodedCommand(cmd) => cmd,
                Execute::Callback(cb) => cb(&ctx),
            };

            execute(command, &test_dir);
        }
    }

    fn save_tarball(&self, bindings: Files) -> Result<PathBuf, Error> {
        let out_dir = self.out_dir();

        let dest = out_dir.join(self.tarball_filename());

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Unable to create \"{}\"", parent.display()))?;
        }

        let f = File::create(&dest)
            .with_context(|| format!("Unable to open \"{}\" for writing", dest.display()))?;

        let encoder = flate2::write::GzEncoder::new(f, Compression::fast());
        let mut builder = tar::Builder::new(encoder);
        for (path, file) in bindings.iter() {
            let mut header = Header::new_gnu();
            header.set_size(file.contents().len() as u64);
            builder
                .append_data(&mut header, path, file.contents())
                .with_context(|| format!("Unable to add \"{}\" to the tarball", path.display()))?;
        }

        builder
            .into_inner()
            .context("Unable to finalize the tar archive")?
            .finish()
            .context("Unable to finish gzip encoding")?
            .sync_all()
            .context("Unable to flush to disk")?;

        Ok(dest)
    }
}

pub fn wit_pack() -> Package {
    let project_root = project_root();

    let exports = project_root
        .join("crates")
        .join("wasm")
        .join("wit-pack.exports.wit");
    assert!(exports.exists());

    execute(
        "cargo build --target=wasm32-unknown-unknown --package=wit-pack-wasm",
        &project_root,
    );

    let wasm = project_root
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("debug")
        .join("wit_pack_wasm.wasm");

    let metadata = Metadata::new("wasmer/wit-pack".parse().unwrap(), "0.0.0");
    let libraries = vec![Library {
        module: Module::from_path(&wasm, Abi::None).unwrap(),
        interface: Interface::from_path(exports).unwrap(),
    }];
    let commands = Vec::new();

    Package::new(metadata, libraries, commands)
}

pub fn wabt() -> Package {
    let project_root = project_root();

    let wabt_dir = project_root
        .join("crates")
        .join("wit-pack")
        .join("tests")
        .join("wabt");

    let metadata = Metadata::new("wasmer/wabt".parse().unwrap(), "0.0.0");
    let libraries = vec![
        Library {
            module: Module::from_path(wabt_dir.join("libwabt.wasm"), Abi::Wasi).unwrap(),
            interface: Interface::from_path(wabt_dir.join("wabt.exports.wit")).unwrap(),
        },
        // Note: we have a duplicate copy of libwabt to check support for
        // multiple libraries
        Library {
            module: Module::from_path(wabt_dir.join("libwabt.wasm"), Abi::Wasi).unwrap(),
            interface: Interface::from_path(wabt_dir.join("wabt2.exports.wit")).unwrap(),
        },
    ];
    let mut commands = Vec::new();

    for entry in wabt_dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().is_none() {
            commands.push(wit_pack::Command {
                name: path.file_stem().unwrap().to_str().unwrap().to_string(),
                wasm: std::fs::read(&path).unwrap(),
            });
        }
    }

    Package::new(metadata, libraries, commands)
}

fn project_root() -> PathBuf {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = crate_dir.ancestors().nth(2).unwrap();
    assert!(root.join(".git").exists());
    root.to_path_buf()
}

fn execute(command: impl AsRef<OsStr>, current_dir: impl AsRef<Path>) {
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new("cmd.exe");
        cmd.arg("/c");
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd
    };

    cmd.arg(command.as_ref()).current_dir(current_dir);

    assert_runs_successfully(&mut cmd);
}

fn assert_runs_successfully(cmd: &mut Command) {
    let Output {
        status,
        stdout,
        stderr,
    } = cmd
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .expect("Unable to start the process");
    if !status.success() {
        let stdout = String::from_utf8_lossy(&stdout);
        if !stdout.is_empty() {
            println!("----- Stdout -----");
            println!("{stdout}");
        }
        let stderr = String::from_utf8_lossy(&stderr);
        if !stderr.is_empty() {
            println!("----- Stderr -----");
            println!("{stderr}");
        }
        panic!("Command failed: {cmd:?}");
    }
}

pub enum Target {
    Python,
    JavaScript,
}

impl Target {
    pub fn generate(&self, package: &Package) -> Result<Files, Error> {
        match self {
            Target::Python => wit_pack::generate_python(package),
            Target::JavaScript => wit_pack::generate_javascript(package),
        }
    }

    pub fn package_name(&self, package: &Package) -> String {
        let Metadata { package_name, .. } = package.metadata();

        match self {
            Target::Python => package_name.python_name(),
            Target::JavaScript => package_name.javascript_package(),
        }
    }
}

pub struct Ctx {
    tarball: PathBuf,
    name: String,
}

impl Ctx {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tarball(&self) -> impl Display + '_ {
        self.tarball.display()
    }
}
