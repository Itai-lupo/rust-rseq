use std::ffi::c_void;

use rseq_main::{RseqCs, RseqCsExt, RseqCsInput, RseqSo, find_offset, get_thread_rseq};

#[test]
fn tests_rseq_counter_is_correct() {
    println!("searching for symbols");

    let rseq_lib: &RseqSo = RseqSo::get();

    let rseq_cs_wrapper_function: fn(&mut RseqCsInput) =
        rseq_lib.get_function_ptr("rseq_cs_wrapper");

    let commit_function = rseq_lib.get_symbol_addr("commit_action") as u64;

    let post_commit_offset: u64 =
        commit_function - rseq_lib.start_section_addr + find_offset("commit_action").unwrap();

    let this_rseq_cs = RseqCs::new(
        rseq_lib.start_section_addr,
        post_commit_offset,
        rseq_lib.abort_trampoline_addr,
    );

    let this_rseq_cs_ref: u64 = &this_rseq_cs as *const _ as u64;

    let counter: u64 = 0;

    let mut cs_input = RseqCsInput::new(
        get_thread_rseq(),
        this_rseq_cs_ref,
        rseq_lib.get_function_ptr("rseq_cs_func"),
        Option::None,
        &counter as *const u64 as *mut c_void,
    );

    println!("Executing RSEQ logic...");

    for _ in 1..100 {
        rseq_cs_wrapper_function(&mut cs_input);
        println!("Counter result: {}", counter);
    }
    assert_eq!(counter, 99);
}

#[test]
fn tests_rseq_counter_is_correct2() {}
