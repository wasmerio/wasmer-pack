use anyhow::{Context, Error};
use graphql_client::GraphQLQuery;
use serde::de::DeserializeOwned;
use ureq::Request;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "graphql/change_generator.graphql",
    schema_path = "graphql/schema.graphql"
)]
struct ChangeGenerator;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "graphql/get_package_version.graphql",
    schema_path = "graphql/schema.graphql"
)]
struct GetPackageVersionQuery;

/// Change the bindings generator used by the WAPM backend.
#[derive(Debug, clap::Parser)]
pub struct SetGenerator {
    /// The GraphQL endpoint to send requests to.
    #[clap(long, env, default_value = "https://registry.wasmer.io/graphql")]
    registry: String,
    /// Look up the package and command, but don't send the final request to
    /// update the generator.
    #[clap(long)]
    dry_run: bool,
    /// The token used to authenticate with the WAPM backend.
    #[clap(long, env)]
    token: String,
    /// The package version to use.
    #[clap(long, default_value = "latest", env)]
    version: String,
    /// The WASI command from the generator package to use (inferred if there is
    /// only one)
    #[clap(long, short, env)]
    command: Option<String>,
    /// The name of the package
    package_name: String,
}

impl SetGenerator {
    pub fn execute(self) -> Result<(), Error> {
        let SetGenerator {
            registry,
            dry_run,
            token,
            version,
            command,
            package_name,
        } = self;

        let pkg_info = query_package_info(&registry, &package_name, &version)
            .context("Unable to look up information about the command")?;

        let command = infer_command(command, &pkg_info.commands)?;

        set_bindings_generator(&registry, &token, &pkg_info, &command, dry_run)
            .context("Unable to set the bindings generator")?;

        Ok(())
    }
}

fn set_bindings_generator(
    registry: &str,
    token: &str,
    pkg: &PackageInfo,
    command: &str,
    dry_run: bool,
) -> Result<(), Error> {
    use self::change_generator::{ResponseData, Variables};

    let id = &pkg.id;
    log::info!("Setting the bindings generator id={id} command={command}");

    let query = ChangeGenerator::build_query(Variables {
        command: command.to_string(),
        id: id.clone(),
    });

    if dry_run {
        log::warn!("This is only a dry-run. Aborting...");
        return Ok(());
    }

    let bearer = format!("Bearer {token}");
    let response: ResponseData = send_query(registry, query, |r| r.set("Authorization", &bearer))?;

    if let Some(response) = response.generate_bindings_for_all_packages {
        log::info!("{}", response.message);
    }

    Ok(())
}

fn query_package_info(
    registry: &str,
    package_name: &str,
    version: &str,
) -> Result<PackageInfo, Error> {
    use self::get_package_version_query::{
        GetPackageVersionQueryPackageVersion, ResponseData, Variables,
    };

    log::debug!("Looking up {package_name}@{version} on {registry}");

    let query = GetPackageVersionQuery::build_query(Variables {
        name: package_name.to_string(),
        version: Some(version.to_string()),
    });
    let response_data: ResponseData = send_query(registry, query, std::convert::identity)?;

    let GetPackageVersionQueryPackageVersion {
        id,
        version,
        commands,
    } = response_data.package_version.context("Package not found")?;

    let pkg_info = PackageInfo {
        id,
        version,
        commands: commands.into_iter().map(|c| c.command).collect(),
    };
    log::debug!(
        "package info id={}, version={}, commands={:?}",
        pkg_info.id,
        pkg_info.version,
        pkg_info.commands
    );
    Ok(pkg_info)
}

#[derive(Debug, Clone)]
struct PackageInfo {
    id: String,
    version: String,
    commands: Vec<String>,
}

fn send_query<R>(
    registry: &str,
    query: impl serde::Serialize,
    update_request: impl FnOnce(Request) -> Request,
) -> Result<R, Error>
where
    R: DeserializeOwned,
{
    let query =
        serde_json::to_string(&query).context("Unable to serialize the request body as JSON")?;

    let request = ureq::post(registry).set("Content-Type", "application/json");

    let response = update_request(request)
        .send_string(&query)
        .map_err(translate_graphql_error)
        .context("Request failed")?;

    let status = response.status();
    if status != 200 {
        let status_text = response.status_text();
        let url = response.get_url();
        log::warn!("{url} {status} {status_text}");
    }

    let response: graphql_client::Response<R> = serde_json::from_reader(response.into_reader())
        .context("Unable to deserialize the response")?;

    if let Some(e) = response.errors.as_deref().and_then(coalesce_errors) {
        return Err(e);
    }

    response.data.context("The response was empty")
}

fn translate_graphql_error(e: ureq::Error) -> Error {
    let msg = e.to_string();

    e.into_response()
        .and_then(|response| {
            serde_json::from_reader(response.into_reader())
                .ok()
                .and_then(|r: graphql_client::Response<serde_json::Value>| r.errors)
                .and_then(|r| coalesce_errors(&r))
                .map(Error::from)
        })
        .unwrap_or_else(|| Error::msg(msg))
}

/// Turn 1 or more GraphQL errors into a single [`Error`] we can return.
fn coalesce_errors(errors: &[graphql_client::Error]) -> Option<Error> {
    match errors {
        [] => None,
        [e] => Some(Error::msg(e.clone())),
        errors => {
            let errors = errors
                .iter()
                .map(|e| format!("\t{e}"))
                .collect::<Vec<_>>()
                .join("\n");
            let count = errors.len();
            Some(anyhow::anyhow!(
                "The server responded with {count} errors:\n{errors}"
            ))
        }
    }
}

/// Use some fuzzy logic to let users
fn infer_command(
    requested: Option<String>,
    available_commands: &[String],
) -> Result<String, Error> {
    match available_commands {
        [] => anyhow::bail!("The package doesn't contain any commands"),
        [cmd] if requested.is_none() => Ok(cmd.clone()),
        commands => {
            if let Some(requested) = requested {
                anyhow::ensure!(
                    commands.contains(&requested),
                    "The package doesn't contain a \"{requested}\" command (available: {})",
                    commands.join(", "),
                );
                return Ok(requested);
            }

            anyhow::bail!(
                "The package contains multiple commands. Please choose from one of the following: {}",
                commands.join(","),
            );
        }
    }
}
