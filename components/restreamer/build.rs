use static_files::{resource_dir, NpmBuild};
use std::{env, path::Path};

fn build_client(out_dir: &str) -> std::io::Result<()> {
    let root_files = Path::new(out_dir).join("generated.rs");

    let mut res_dir = NpmBuild::new("./client")
        .executable("yarn")
        .install()?
        .run(if cfg!(debug_assertions) {
            "build:dev"
        } else {
            "build:prod"
        })?
        .target("./client/public")
        .to_resource_dir();

    res_dir.with_generated_filename(root_files);
    res_dir.with_filter(|p| !p.ends_with("mix"));

    res_dir.build()
}

fn build_mix(out_dir: &str) -> std::io::Result<()> {
    let mix_files = Path::new(&out_dir).join("generated_mix.rs");
    let mut res_dir = resource_dir("./client/public/mix");
    res_dir.with_generated_filename(mix_files);

    res_dir.build()
}

fn build_dashboard(out_dir: &str) -> std::io::Result<()> {
    let dashboard_files = Path::new(&out_dir).join("generated_dashboard.rs");
    let mut res_dir = resource_dir("./client/public/dashboard");
    res_dir.with_generated_filename(dashboard_files);

    res_dir.build()
}

fn main() -> std::io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();

    build_client(&out_dir)?;
    build_mix(&out_dir)?;
    build_dashboard(&out_dir)
}
