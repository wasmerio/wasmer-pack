use std::path::Path;

use anyhow::{Context, Error};

/// Update the `schema.graphql` file.
#[derive(Debug, clap::Parser)]
pub struct SyncSchema {
    /// The GraphQL endpoint to send requests to.
    #[clap(long, env, default_value = "https://registry.wapm.io/graphql")]
    registry: String,
}

impl SyncSchema {
    pub fn execute(self) -> Result<(), Error> {
        let SyncSchema { registry } = self;
        let endpoint = format!("{registry}/schema.graphql");

        fetch_schema(&endpoint).and_then(|s| sync(&s))
    }
}

fn fetch_schema(endpoint: &str) -> Result<String, Error> {
    let response = ureq::get(endpoint)
        .send_bytes(&[])
        .context("Unable to fetch the GraphQL schema")?;
    anyhow::ensure!(
        response.status() == 200,
        "{} {}",
        response.status(),
        response.status_text()
    );

    response
        .into_string()
        .context("Unable to read the response as a string")
}

fn sync(schema: &str) -> Result<(), Error> {
    let graphql_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("graphql");
    let schema_path = graphql_dir.join("schema.graphql");

    match std::fs::read_to_string(&schema_path) {
        Ok(s) if s == schema => {
            log::debug!("Schema is already up to date");
            return Ok(());
        }
        Ok(_) => {
            log::debug!("Schema needs updating");
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            log::debug!("The schema file hasn't been created yet");
        }
        Err(e) => {
            return Err(
                Error::new(e).context(format!("Unable to read \"{}\"", schema_path.display()))
            );
        }
    }

    std::fs::create_dir_all(&graphql_dir).with_context(|| {
        format!(
            "Unable to create the \"{}\" directory",
            graphql_dir.display()
        )
    })?;

    std::fs::write(&schema_path, schema)
        .with_context(|| format!("Unable to write to \"{}\"", schema_path.display()))?;

    log::info!("The schema was updated. Please commit the changes.");

    Ok(())
}
