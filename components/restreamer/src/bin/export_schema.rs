//! Binary exporting server's GraphQL schemas into JSON files.
//!
//! # Usage
//!
//! ```bash
//! cargo run --bin export_schema -- --api=client --out-dir=./
//! ```

use std::{fs, path::PathBuf, str::FromStr};

use anyhow::anyhow;
use derive_more::Display;
use ephyr_restreamer::api;
use structopt::StructOpt;

/// Introspects GraphQL schema and exports it into `*.graphql.schema.json` file.
fn main() -> anyhow::Result<()> {
    let opts = CliOpts::from_args_safe()?;

    let err_fn = |e| anyhow!("Failed to execute introspection query: {}", e);

    let (res, _) = match opts.api {
        Api::Client => juniper::introspect(
            &api::graphql::client::schema(),
            &api::graphql::Context::fake(),
            juniper::IntrospectionFormat::default(),
        )
        .map_err(err_fn)?,
        Api::Mix => juniper::introspect(
            &api::graphql::mix::schema(),
            &api::graphql::Context::fake(),
            juniper::IntrospectionFormat::default(),
        )
        .map_err(err_fn)?,
    };

    let json = serde_json::to_string_pretty(&res)
        .map_err(|e| anyhow!("Failed to encode schema as JSON: {}", e))?;

    let filename = format!(
        "{}/{}.graphql.schema.json",
        opts.out_dir.components().as_path().display(),
        opts.api,
    );
    fs::write(
        &filename,
        // "data" wrapping is required by GraphDoc.
        // See: https://github.com/2fd/graphdoc/issues/54
        format!(r#"{{"data":{}}}"#, json),
    )
    .map_err(|e| {
        anyhow!("Failed to write schema to the `{}` file: {}", filename, e)
    })?;

    Ok(())
}

/// CLI (command line interface) of this binary.
#[derive(Clone, Debug, StructOpt)]
#[structopt(
    about = "Export GraphQL schema to a JSON file",
    rename_all = "kebab-case"
)]
struct CliOpts {
    /// [`api::graphql`] to export schema of.
    #[structopt(
        long,
        default_value = "client",
        help = "Backend API to export schema of: client"
    )]
    api: Api,

    /// Output directory to create JSON file in.
    ///
    /// [`vod::meta::State`]: crate::vod::meta::State
    #[structopt(
        long,
        default_value = "./components/restreamer/",
        help = "Output directory to create JSON file in"
    )]
    pub out_dir: PathBuf,
}

/// Possible backend APIs for exporting their GraphQL schema.
#[derive(Clone, Copy, Debug, Display)]
enum Api {
    /// [`api::graphql::client`].
    #[display(fmt = "client")]
    Client,
    /// [`api::graphql::mix`].
    #[display(fmt = "mix")]
    Mix,
}

impl FromStr for Api {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "client" => Ok(Self::Client),
            "mix" => Ok(Self::Mix),
            _ => Err(anyhow!("Unknown backend API '{}'", s)),
        }
    }
}
