use bindgen;
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let linker_script_path = PathBuf::from(manifest_dir).join("linker.ld");

    if !linker_script_path.exists() {
        panic!("Error: Linker script not found at {:?}", linker_script_path);
    }
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .use_core()
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()?;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("libc_bindings.rs"))?;

    println!("cargo:rustc-link-arg=-T{}", linker_script_path.display());
    println!("cargo:rustc-link-arg=-z");
    println!("cargo:rustc-link-arg=nodelete");
    println!("cargo:rustc-target-feature=+crt-static");

    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
