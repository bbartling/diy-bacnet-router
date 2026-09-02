use std::{env, fs, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let version_path = Path::new(&manifest_dir).join("../../VERSION");
    println!("cargo:rerun-if-changed={}", version_path.display());

    let version = fs::read_to_string(&version_path)
        .unwrap_or_else(|error| {
            panic!(
                "VERSION file missing at {}: {error}",
                version_path.display()
            )
        })
        .trim()
        .to_owned();

    if version.is_empty() {
        panic!("VERSION file must not be empty");
    }

    println!("cargo:rustc-env=DBR_VERSION={version}");
}
