use std::path::Path;
use std::process::Command;

use dlopen_rs::{ElfLibrary, OpenFlags};

#[test]
fn so_is_built() {
    const PAYLOAD_SO: &str = env!("PAYLOAD_SO");
    let so_path = Path::new(PAYLOAD_SO);

    // Check if file exists
    assert!(
        so_path.exists(),
        "Shared library not found at {:?}",
        so_path
    );

    // Optional: Verify it's a valid ELF shared object
    let output = Command::new("file")
        .arg(&so_path)
        .output()
        .expect("failed to run 'file' command");

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        output_str.contains("ELF"),
        "File is not an ELF shared object: {}",
        output_str
    );
}

const RSEQ_START: &str = "rseq_start";
const RSEQ_COMMIT_END: &str = "rseq_commit_end";
const RSEQ_ABORT_IP: &str = "rseq_abort_ip";

const RSEQ_LIB_FLAGS: OpenFlags = OpenFlags::RTLD_NOW;

#[test]
fn so_has_required_symbols() {
    let lib = ElfLibrary::dlopen(env!("PAYLOAD_SO"), RSEQ_LIB_FLAGS).unwrap();

    let start: u64 = unsafe { *lib.get(RSEQ_START).unwrap() };
    assert_ne!(start, 0);

    let commit: u64 = unsafe { *lib.get(RSEQ_COMMIT_END).unwrap() };
    assert_ne!(commit, 0);

    let abort: u64 = unsafe { *lib.get(RSEQ_ABORT_IP).unwrap() };
    assert_ne!(abort, 0);
}

macro_rules! parse_u32 {
    ($s:expr) => {{
        let bytes = $s.as_bytes();
        let mut n = 0;
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i];
            assert!(b'0' <= c && c <= b'9');
            n = n * 10 + (c - b'0') as u32;
            i += 1;
        }
        n
    }};
}

const RSEQ_SIG: u32 = parse_u32!(env!("RSEQ_SIG"));

#[test]
fn abort_first_4_bytes_match_rseq_sig() {
    let lib =  ElfLibrary::dlopen(env!("PAYLOAD_SO"), RSEQ_LIB_FLAGS).unwrap();
    let abort_trampoline_addr: *const u64 = unsafe { *lib.get(RSEQ_ABORT_IP).unwrap() };
    let value: u32 =
        unsafe { std::ptr::read((abort_trampoline_addr as *const u8).sub(4) as *const u32) };
    assert_eq!(value, RSEQ_SIG);
}
