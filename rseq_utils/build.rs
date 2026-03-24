use std::env;

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let rseq_sig = match target_arch.as_str() {
        // for know I use the same rseqsig for all archs so this is just for futre refrance
        "x86_64" => 0x53514552,
        _ => 0x53514552,
    };

    println!("cargo:rustc-env=RSEQ_SIG={}", rseq_sig);
}
