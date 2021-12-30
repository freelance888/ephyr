use actix_web_static_files::{resource_dir, NpmBuild};
use std::{env, path::Path};

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let root_files = Path::new(&out_dir).join("generated.rs");
    let mix_files = Path::new(&out_dir).join("generated_mix.rs");
    let dashboard_files = Path::new(&out_dir).join("generated_dashboard.rs");

    NpmBuild::new("./client")
        .executable("yarn")
        .install()?
        .run(if cfg!(debug_assertions) {
            "build:dev"
        } else {
            "build:prod"
        })?
        .target("./client/public")
        .to_resource_dir()
        .with_generated_filename(root_files)
        .with_filter(|p| !p.ends_with("mix"))
        .build()?;

    resource_dir("./client/public/mix")
        .with_generated_filename(mix_files)
        .build()?;

    resource_dir("./client/public/dashboard")
        .with_generated_filename(dashboard_files)
        .build()?;

    Ok(())
}
