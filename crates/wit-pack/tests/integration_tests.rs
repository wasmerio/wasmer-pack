use crate::fixtures::{Target, TestCase};

mod fixtures;

#[test]
fn js() {
    TestCase::new("js", Target::JavaScript, fixtures::wit_pack())
        .execute("yarn")
        .callback(|ctx| format!("yarn add {}", ctx.tarball()))
        .execute("tsc --noEmit")
        .execute("tsc test")
        .run();
}

#[test]
fn js_wasi() {
    TestCase::new("js-wasi", Target::JavaScript, fixtures::wabt())
        .execute("yarn")
        .callback(|ctx| format!("yarn add {}", ctx.tarball()))
        .execute("tsc --noEmit")
        .execute("tsc test")
        .run();
}

#[test]
fn python() {
    TestCase::new("python", Target::Python, fixtures::wit_pack())
        .execute("pipenv install")
        .callback(|ctx| format!("pipenv install {}", ctx.tarball()))
        .execute("pipenv run pytest")
        .callback(|ctx| format!("pipenv run mypy -m {}", ctx.name()))
        .run();
}

#[test]
fn python_wasi() {
    TestCase::new("python-wasi", Target::Python, fixtures::wabt())
        .execute("pipenv install")
        .callback(|ctx| format!("pipenv install {}", ctx.tarball()))
        .execute("pipenv run pytest")
        .callback(|ctx| format!("pipenv run mypy -m {}", ctx.name()))
        .run();
}
