use build_print::{error, info};
use object::{File, Object, ObjectSection, ObjectSymbol, SymbolKind};
use std::env;
use std::error::Error;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const POST_COMMIT_OFFSET_MARKER_SYMBOL_NAME: &str = "rseq_end_handler_call_marker";
const POST_COMMIT_OFFSET_MARKER_ENDS_WITH: u32 = 0xABCDEFFFu32;

fn find_magic_offset_exactly_once(data: &[u8], magic: &[u8]) -> Result<usize> {
    let mut matches = data
        .windows(magic.len())
        .enumerate()
        .filter(|(_, window)| *window == magic)
        .map(|(i, _)| i);

    let first = matches.next().ok_or("didn't find magic in symbol")?;

    if matches.next().is_some() {
        return Err("magic found multiple times".into());
    }

    Ok(first)
}

fn get_symbol_bytes<'a>(file: &'a File<'a>, symbol_name: &str) -> Result<&'a [u8]> {
    let symbol = file
        .symbol_by_name(symbol_name)
        .ok_or_else(|| format!("Symbol '{}' not found", symbol_name))?;

    let section_index = symbol
        .section()
        .index()
        .ok_or_else(|| format!("Symbol '{}' has no associated section", symbol_name))?;

    let section = file.section_by_index(section_index)?;

    let data = section
        .data_range(symbol.address(), symbol.size())?
        .ok_or_else(|| {
            format!(
                "Data for symbol '{}' is not present in the file",
                symbol_name
            )
        })?;

    Ok(data)
}

fn get_post_commit_offset_marker_value<'a>(obj_file: &'a File<'a>) -> Result<&'a [u8]> {
    let sym_data = get_symbol_bytes(obj_file, POST_COMMIT_OFFSET_MARKER_SYMBOL_NAME)?;

    let magic_bytes = POST_COMMIT_OFFSET_MARKER_ENDS_WITH.to_ne_bytes();

    let magic_pos = find_magic_offset_exactly_once(sym_data, &magic_bytes)?;

    Ok(&sym_data[..magic_pos])
}

fn get_symbol_offsets(
    file: &object::File,
    section_name: &str,
    magic: &[u8],
) -> Result<Vec<(String, usize)>> {
    let section_index = file
        .section_by_name(section_name)
        .ok_or_else(|| format!("Section '{}' not found", section_name))?
        .index();

    let mut results = Vec::new();
    for symbol in file.symbols() {
        if symbol.section().index() != Some(section_index) {
            continue;
        }

        match process_symbol(file, &symbol, magic) {
            Ok(Some(res)) => results.push(res),
            Ok(None) => continue,
            Err(e) => return Err(e),
        }
    }

    Ok(results)
}

fn process_symbol(
    file: &object::File,
    symbol: &object::Symbol,
    magic: &[u8],
) -> Result<Option<(String, usize)>> {
    if symbol.kind() != SymbolKind::Text || symbol.size() == 0 {
        return Ok(None);
    }

    let name = symbol.name()?;
    let symbol_data = get_symbol_bytes(file, name)?;
    info!(
        "Searching in symbol '{}' (size: {}) for magic: {:02x?}",
        name,
        symbol.size(),
        magic
    );

    let offset = find_magic_offset_exactly_once(symbol_data, magic)?;

    Ok(Some((name.to_string(), offset)))
}

fn generate_output(results: Vec<(String, usize)>) {
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

fn process_functions_in_so(so_path: &str) -> Result<()> {
    let data = fs::read(so_path).expect("Failed to read .so");
    let obj_file = object::File::parse(&*data).expect("Failed to parse ELF");

    let magic = get_post_commit_offset_marker_value(&obj_file)?;
    let result = get_symbol_offsets(&obj_file, ".text.rseq_commit", &magic)?;

    generate_output(result);
    Ok(())
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.join("inner_target");

    // Build the inner lib in a separate target dir to avoid cargo lock contention
    unsafe { env::set_var("RUSTFLAGS", "-C relocation-model=pic") };
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

    match process_functions_in_so(dest.display().to_string().as_str()) {
        Ok(_) => {}
        Err(e) => {
            error!("failed to gen post commit offsets with error: {}", e);

            std::process::exit(1);
        }
    }
}
