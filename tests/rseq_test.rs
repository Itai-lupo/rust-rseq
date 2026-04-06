use std::ffi::c_void;

use rseq_main::rseq_context;
use rseq_main::{RseqCs, RseqCsExt, RseqCsInput, RseqSo, RseqTask, find_offset, get_thread_rseq};

use rseq_macros::{
    rseq_commit_action, rseq_critical_section, rseq_critical_section_start, rseq_shared_struct,
};

use rseq_utils::{RseqCommitActionName, RseqStart};

#[rseq_critical_section_start]
pub fn MY_COUNTER_a(ctx: *mut c_void) {
    update_log(ctx as *mut u64);
}

#[rseq_critical_section]
pub fn helper_function1() -> Test {
    Test { a: 1 }
}

#[rseq_commit_action]
pub fn update_log(res: *mut u64) {
    unsafe {
        *res = *res + 1;
        rseq_cs_end!();
    }
}

rseq_context! {
    name = MY_COUNTER,

    helpers = {
        use core::ffi::c_void;
        fn internal_add(a: u32, b: u32) -> u32 { a + b }
    },

    commit = fn update_log(res: *mut u64) {
        unsafe{
            *res = *res + 1;
        }
        rseq_cs_end!();
    },

    cs = |ctx: *mut c_void| {
        update_log(ctx as *mut u64);
       ctx
    }
}

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
fn tests_rseq_counter_is_correct2() {
    let mut data: u32 = 0;

    for _ in 1..101 {
        unsafe {
            MY_COUNTER.run(&mut data as *mut u32 as *mut c_void);
        }
    }

    assert_eq!(data, 100);
    println!("Final data: {}", data);
}
