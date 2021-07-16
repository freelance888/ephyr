use actix_web_static_files::{resource_dir, NpmBuild};
use std::{env, path::Path};

fn main() -> anyhow::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let root_files = Path::new(&out_dir).join("generated.rs");
    let restream_files = Path::new(&out_dir).join("generated_output.rs");

    NpmBuild::new("./")
        .executable("yarn")
        .install()?
        .run(if cfg!(debug_assertions) {
            "build:dev"
        } else {
            "build:prod"
        })?
        .target("./public")
        .to_resource_dir()
        .with_generated_filename(root_files)
        .with_filter(|p| !p.ends_with("restream"))
        .build()?;

    resource_dir("./public/restream")
        .with_generated_filename(restream_files)
        .build()?;

    Ok(())
}
