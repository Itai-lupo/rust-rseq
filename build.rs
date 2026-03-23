use build_print::{error, info};
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.join("inner_target");

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let rseq_sig = match target_arch.as_str() {
        // for know I use the same rseqsig for all archs so this is just for futre refrance
        "x86_64" => 0x53514552,
        _ => 0x53514552,
    };

    unsafe {
        std::env::set_var("RSEQ_SIG", rseq_sig.to_string());
    }

    println!("cargo:rustc-env=RSEQ_SIG={}", rseq_sig);
    // Build the inner lib in a separate target dir to avoid cargo lock contention
    let status = Command::new("cargo")
        .arg("build")
        .arg("--color=always")
        .arg("--release")
        .arg("--manifest-path=rseq_payload/Cargo.toml")
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--locked")
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

    // println!("cargo:rustc-env=SHARED_VALUE={}", shared_value);
}
