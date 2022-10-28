use crate::fixtures::{Target, TestCase};

mod fixtures;

#[test]
#[ignore]
fn js() {
    TestCase::new("js", Target::JavaScript, fixtures::wit_pack())
        .execute("yarn")
        .callback(|ctx| format!("yarn add {}", ctx.tarball()))
        .execute("tsc --noEmit")
        .execute("tsc test")
        .run();
}

#[test]
#[ignore]
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
        .execute("poetry install")
        .callback(|ctx| format!("poetry add {}", ctx.unpacked().display()))
        .execute("poetry run pytest")
        .callback(|ctx| {
            format!(
                "poetry run mypy {}",
                ctx.unpacked().join(ctx.name()).display()
            )
        })
        .execute("poetry run mypy .")
        .run();
}

#[test]
fn python_wasi() {
    TestCase::new("python-wasi", Target::Python, fixtures::wabt())
        .execute("poetry install")
        .callback(|ctx| format!("poetry add {}", ctx.unpacked().display()))
        .execute("poetry run pytest")
        .callback(|ctx| {
            format!(
                "poetry run mypy {}",
                ctx.unpacked().join(ctx.name()).display()
            )
        })
        .execute("poetry run mypy .")
        .run();
}
