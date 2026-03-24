use build_print::{error, info};
use object::{Object, ObjectSection, ObjectSymbol};
use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;

#[unsafe(naked)]
extern "C" fn ret_marker() {
    core::arch::naked_asm!("ret");
}

fn extract_ret_bytes() -> &'static [u8] {
    unsafe { std::slice::from_raw_parts(ret_marker as *const u8, 1) }
}

fn find_functions_in_section(so_path: &str, section_name: &str) -> Vec<(String, u64, Vec<u8>)> {
    let data = fs::read(so_path).expect("Failed to read .so");
    let file = object::File::parse(&*data).expect("Failed to parse ELF");

    let mut functions = Vec::new();
    for sym in file.symbols() {
        if sym.kind() != object::SymbolKind::Text || sym.size() == 0 {
            continue;
        }
        let section = sym.section();
        if let Ok(section) = file.section_by_index(section.index().unwrap()) {
            if section.name() == Ok(section_name) {
                let addr = sym.address();
                let size = sym.size();
                if let Ok(Some(code)) = section.data_range(addr, size) {
                    let name = sym.name().unwrap_or("<unknown>").to_string();
                    functions.push((name, addr, code.to_vec()));
                }
            }
        }
    }
    functions
}

fn find_single_ret_offset(code: &[u8], ret_bytes: &[u8]) -> Option<u64> {
    let matches: Vec<usize> = code
        .windows(ret_bytes.len())
        .enumerate()
        .filter_map(|(i, w)| if w == ret_bytes { Some(i) } else { None })
        .collect();
    if matches.len() == 1 {
        Some(matches[0] as u64)
    } else {
        None
    }
}

fn generate_output(results: Vec<(String, u64)>) {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("ret_offsets.rs");
    let out_file = fs::File::create(&dest_path).unwrap();
    let mut writer = BufWriter::new(out_file);
    write!(writer, "pub const RET_OFFSETS: &[(&str, u64)] = &[").unwrap();
    for (name, offset) in results {
        write!(writer, "    (\"{}\", {}),", name, offset).unwrap();
    }
    write!(writer, "];").unwrap();
    // writer.flush();
}

fn process_functions_in_so(so_path: &str, section_name: &str) {
    let ret_bytes = extract_ret_bytes();
    let functions = find_functions_in_section(so_path, section_name);
    let results: Vec<_> = functions
        .into_iter()
        .filter_map(|(name, _addr, code)| {
            find_single_ret_offset(&code, ret_bytes).map(|offset| (name, offset))
        })
        .collect();
    generate_output(results);
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.join("inner_target");

    // Build the inner lib in a separate target dir to avoid cargo lock contention
    let status = Command::new("cargo")
        .arg("build")
        .arg("--color=always")
        .arg("--release")
        .arg("--manifest-path=rseq_payload/Cargo.toml")
        .arg("--target-dir")
        .arg(&target_dir)
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

    process_functions_in_so(dest.display().to_string().as_str(), ".text");

    // println!("cargo:rustc-env=SHARED_VALUE={}", shared_value);
}
