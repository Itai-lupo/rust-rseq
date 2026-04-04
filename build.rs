use build_print::{error, info};
use std::env;
use std::path::PathBuf;
use std::process::Command;

use build_utils::generate_post_commit_offsets::process_functions_in_so;
use build_utils::handle_rseq_macros::genrate_rseq_code;


fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.join("inner_target");

    println!("cargo:rerun-if-changed={}", out_dir.display());

    let generate_payload_code_path =  genrate_rseq_code();

    // Build the inner lib in a separate target dir to avoid cargo lock contention
    let status = Command::new("cargo")
        .arg("build")
        .arg("--color=always")
        .arg("--release")
        .arg("--manifest-path=rseq_payload/Cargo.toml")
        .arg("--target-dir")
        .arg(&target_dir)
        .env("USER_TASKS_PATH", &generate_payload_code_path)
        .status()
        .expect("Failed to start cargo");

    if !status.success() {
        error!(
            "error: failed to build librseq payload with status code {:?}",
            status.code().unwrap_or(-1)
        );
        info!("See above for cargo output");
        std::process::exit(1);
    }

    let src = target_dir.join("release/librseq_payload.so");
    let dest = out_dir.join("librseq_payload.so");
    std::fs::copy(&src, &dest).expect("Failed to copy .so");

    // Make path available to code
    println!("cargo:rustc-env=PAYLOAD_SO={}", dest.display());
    println!("cargo:rerun-if-changed=../rseq_payload/");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");

    match process_functions_in_so(dest.display().to_string().as_str()) {
        Ok(_) => {}
        Err(e) => {
            error!("failed to gen post commit offsets with error: {}", e);

            std::process::exit(1);
        }
    }
}
