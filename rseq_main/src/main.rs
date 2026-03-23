// #![feature(thread_local)]

pub mod rseq_lib_wrapper;
pub mod sys_rseq;

use enumflags2::BitFlag;
use rseq_lib_wrapper::RseqSo;
use sys_rseq::{RseqCs, RseqFlags, get_thread_rseq_cs_ref, rseq_thread_registor};

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

fn main() {
    println!("searching for symbols");

    let rseq_lib: &RseqSo = RseqSo::get();

    rseq_thread_registor(RSEQ_SIG, RseqFlags::empty());

    let rseq_cs_function: u64 = rseq_lib.get_function_addr("rseq_critical_store");

    println!(
        "loaded rseq so with {} {} {}",
        rseq_lib.start_section_addr, rseq_lib.commit_section_end, rseq_lib.abort_trampoline_addr
    );

    println!("--- RSEQ Initialized ---");
    println!("Start IP: 0x{:x}", rseq_lib.start_section_addr);
    println!("abort ip: 0x{:x}", rseq_lib.abort_trampoline_addr);

    let post_commit_offset: u64 = rseq_lib.commit_section_end - rseq_lib.start_section_addr;
    let this_rseq_cs = RseqCs::new(
        rseq_lib.start_section_addr,
        post_commit_offset,
        rseq_lib.abort_trampoline_addr,
        RSEQ_SIG,
    );

    let this_rseq_cs_ref: u64 = &this_rseq_cs as *const _ as u64;

    let rseq_func: unsafe extern "C" fn(ptr: *mut u64, rseq_cs: &mut u64, this_rseq_cs: u64) =
        unsafe { std::mem::transmute(rseq_cs_function) };

    let mut counter: u64 = 0;

    let rseq_cs_ref = unsafe { &mut *get_thread_rseq_cs_ref() };

    println!("Executing RSEQ logic...");
    unsafe { rseq_func(&mut counter as *mut u64, rseq_cs_ref, this_rseq_cs_ref) };

    println!("Counter result: {}", counter);

    *rseq_cs_ref = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rseq_so_loads() {
        let rseq = RseqSo::get();
        assert_ne!(rseq.start_section_addr, 0);
        assert_ne!(rseq.commit_section_end, 0);
        assert_ne!(rseq.abort_trampoline_addr, 0);
    }

    #[test]
    fn test_get_function_addr() {
        let rseq = RseqSo::get();
        let addr = rseq.get_function_addr("rseq_start");
        assert_ne!(addr, 0);
    }

    #[test]
    #[should_panic(expected = "Failed to load symbol")]
    fn test_invalid_symbol_panics() {
        let rseq = RseqSo::get();
        rseq.get_function_addr("invalid_symbol");
    }
}
