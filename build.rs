use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/shader.metal");

    if env::var("CARGO_FEATURE_METAL").is_ok() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let metallib_path = PathBuf::from(out_dir).join("shader.metallib");

        let status = Command::new("xcrun")
            .args([
                "-sdk",
                "macosx",
                "metal",
                "src/shader.metal",
                "-o",
                metallib_path.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to execute xcrun.");

        if !status.success() {
            panic!("Failed to compile Metal shader.");
        }
    }
}
