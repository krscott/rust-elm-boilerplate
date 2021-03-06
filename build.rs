use rust_elm_types::ApiSpec;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const API_TYPES_FILE: &str = "api_types.yml";
const RUST_OUTPUT: &str = "src/api_types.rs";
const ELM_OUTPUT: &str = "gui/src/ApiTypes.elm";
const ELM_DIR: &str = "gui";
const ELM_DIST_OUTPUT: &str = "elm-dist.js";

fn main() {
    let input_path = Path::new(API_TYPES_FILE);

    let file = fs::File::open(&input_path).unwrap();
    let spec: ApiSpec = serde_yaml::from_reader(file).unwrap();

    let rust_path = Path::new(RUST_OUTPUT);
    let rust_code = format!(
        "// Auto-generated by rust_elm_types\n\n{}\n",
        spec.to_rust()
    );
    fs::write(&rust_path, rust_code).unwrap();

    let elm_path = Path::new(ELM_OUTPUT);
    let elm_code = format!("-- Auto-generated by rust_elm_types\n\n{}\n", spec.to_elm());
    fs::write(&elm_path, elm_code).unwrap();

    env::set_current_dir(ELM_DIR).unwrap();
    let elm_build_status = Command::new("elm")
        .args(&[
            "make",
            "--optimize",
            "--output",
            ELM_DIST_OUTPUT,
            "src/Main.elm",
        ])
        .status()
        .unwrap();

    assert!(elm_build_status.success());
}
