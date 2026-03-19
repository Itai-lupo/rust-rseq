// #![feature(thread_local)]

pub mod sys_rseq;
use enumflags2::BitFlag;
use sys_rseq::{RseqCs, RseqFlags, get_thread_rseq_cs_ref, rseq_thread_registor};

use dlopen_rs::{ElfLibrary, OpenFlags, Result};

const PAYLOAD_SO: &[u8] =
    include_bytes!("../../target/x86_64-unknown-linux-musl/release/librseq_payload.so");

#[cfg(target_arch = "x86_64")]
const RSEQ_SIG: u32 = 0x53514552u32;

fn main() -> Result<()> {
    println!("start");

    let lib =
        ElfLibrary::dlopen_from_binary(PAYLOAD_SO, "librseq_payload.so", OpenFlags::RTLD_NOW)?;

    println!("searching for symbols");

    rseq_thread_registor(RSEQ_SIG, RseqFlags::empty());

    let start_addr = unsafe { *lib.get::<u64>("rseq_start")? };
    let rseq_cs_function = unsafe { *lib.get::<u64>("rseq_critical_store")? };
    let post_commit_addr = unsafe { *lib.get::<u64>("rseq_commit_end")? };
    let abort_addr = unsafe { *lib.get::<u64>("rseq_abort_ip")? };

    println!(
        "loaded rseq so with {} {} {}",
        start_addr, post_commit_addr, abort_addr
    );

    println!("--- RSEQ Initialized ---");
    println!("Start IP: 0x{:x}", start_addr);
    println!("Abort IP: 0x{:x}", abort_addr);

    let post_commit_offset = post_commit_addr - start_addr;
    let this_rseq_cs = RseqCs::new(start_addr, post_commit_offset, abort_addr, RSEQ_SIG);

    let this_rseq_cs_ref = &this_rseq_cs as *const _ as u64;

    let rseq_func: unsafe extern "C" fn(ptr: *mut u64, rseq_cs: &mut u64, this_rseq_cs: u64) =
        unsafe { std::mem::transmute(rseq_cs_function) };

    let mut counter: u64 = 0;

    let rseq_cs_ref = unsafe { &mut *get_thread_rseq_cs_ref() };

    println!("Executing RSEQ logic...");
    unsafe { rseq_func(&mut counter as *mut u64, rseq_cs_ref, this_rseq_cs_ref) };

    println!("Counter result: {}", counter);

    *rseq_cs_ref = 0;
    Ok(())
}
