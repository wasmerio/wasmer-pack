use anyhow::Error;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    wasmer_pack_testing::autodiscover(env!("CARGO_MANIFEST_DIR"))?;
    Ok(())
}
