use static_files::{resource_dir, NpmBuild};
use std::{env, path::Path};

fn build_root(out_dir: &str) -> std::io::Result<()> {
    let root_files = Path::new(out_dir).join("generated.rs");
    let mut res_dir = resource_dir("./client/public");
    res_dir
        .with_generated_filename(root_files)
        .with_filter(|p| {
            !(p.ends_with("mix")
                || p.ends_with("dashboard")
                || p.ends_with("full-stream"))
        });

    res_dir.build()
}

fn build_mix(out_dir: &str) -> std::io::Result<()> {
    let mix_files = Path::new(&out_dir).join("generated_mix.rs");
    let mut res_dir = resource_dir("./client/public/mix");
    res_dir
        .with_generated_filename(mix_files)
        .with_module_name("mix");

    res_dir.build()
}

fn build_dashboard(out_dir: &str) -> std::io::Result<()> {
    let dashboard_files = Path::new(&out_dir).join("generated_dashboard.rs");
    let mut res_dir = resource_dir("./client/public/dashboard");
    res_dir
        .with_generated_filename(dashboard_files)
        .with_module_name("dashboard");

    res_dir.build()
}

fn build_full_stream(out_dir: &str) -> std::io::Result<()> {
    let dashboard_files = Path::new(&out_dir).join("generated_full_stream.rs");
    let mut res_dir = resource_dir("./client/public/full-stream");
    res_dir
        .with_generated_filename(dashboard_files)
        .with_module_name("full_stream");

    res_dir.build()
}

fn main() -> std::io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();

    let package_json_dir = "./client";
    if !cfg!(debug_assertions) {
        NpmBuild::new(package_json_dir)
            .executable("yarn")
            .install()?;
    }
    NpmBuild::new(package_json_dir)
        .executable("yarn")
        .run(if cfg!(debug_assertions) {
            "build:dev"
        } else {
            "build:prod"
        })?
        .target("./client/public")
        .to_resource_dir()
        .build()?;

    build_root(&out_dir)?;
    build_mix(&out_dir)?;
    build_full_stream(&out_dir)?;
    build_dashboard(&out_dir)?;

    Ok(())
}
