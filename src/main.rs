// #![feature(thread_local)]

#![feature(thread_local)]
pub mod rseq_lib_wrapper;
pub mod sys_rseq;

use std::ffi::c_void;

use enumflags2::BitFlag;
use rseq_lib_wrapper::RseqSo;
use rseq_utils::{RSEQ_SIG, RseqCsInput};
use sys_rseq::{RseqCs, RseqCsExt, RseqFlags, rseq_thread_registor, get_thread_rseq};

fn main() {
    println!("searching for symbols");

    let rseq_lib: &RseqSo = RseqSo::get();

    rseq_thread_registor(RSEQ_SIG, RseqFlags::empty());

    let rseq_cs_wrapper_function: fn(&mut RseqCsInput) =
        rseq_lib.get_function_ptr("rseq_cs_wrapper");

    let post_commit_offset: u64 = rseq_lib.commit_section_end - rseq_lib.start_section_addr;
    let this_rseq_cs = RseqCs::new(
        rseq_lib.start_section_addr,
        post_commit_offset,
        rseq_lib.abort_trampoline_addr,
    );

    let this_rseq_cs_ref: u64 = &this_rseq_cs as *const _ as u64;

    let counter: u64 = 0;
    
    let mut cs_input = RseqCsInput::new(
        get_thread_rseq(), this_rseq_cs_ref,  rseq_lib.get_function_ptr("rseq_cs_func"), Option::None, &counter as *const u64 as  *mut c_void);


    println!("Executing RSEQ logic...");
    for _ in 1..1000 {
        // unsafe { rseq_func(&mut counter as *mut u64, rseq_cs_ref, this_rseq_cs_ref) };
        rseq_cs_wrapper_function(&mut cs_input);
        println!("Counter result: {}", counter);
    }
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
        let addr = rseq.get_symbol_addr("rseq_start");
        assert_ne!(addr, 0);
    }

    #[test]
    #[should_panic(expected = "Failed to load symbol")]
    fn test_invalid_symbol_panics() {
        let rseq = RseqSo::get();
        rseq.get_symbol_addr("invalid_symbol");
    }
}
