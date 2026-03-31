#![feature(thread_local)]
pub mod rseq_lib_wrapper;
pub mod sys_rseq;

pub use rseq_macros::rseq_context;

include!(concat!(env!("OUT_DIR"), "/post_commit_offsets.rs"));

use core::ffi::c_void;

pub use rseq_utils::{RSEQ_SIG, RseqCsInput};
pub use sys_rseq::{RseqCs, RseqCsExt, RseqFlags, get_thread_rseq, rseq_thread_registor};

pub use enumflags2::BitFlag;
pub use rseq_lib_wrapper::RseqSo;

pub struct RseqTask {
    pub main_symbol: &'static str,
    pub commit_symbol: &'static str,
}

impl RseqTask {
    pub unsafe fn run(&self, ctx: *mut c_void) {
        let rseq_lib: &RseqSo = RseqSo::get();

        let rseq_cs_wrapper_function: fn(&mut RseqCsInput) =
            rseq_lib.get_function_ptr("rseq_cs_wrapper");

        let commit_function = rseq_lib.get_symbol_addr(self.commit_symbol) as u64;

        let post_commit_offset: u64 = commit_function - rseq_lib.start_section_addr
            + find_offset(self.commit_symbol).unwrap();

        let this_rseq_cs = RseqCs::new(
            rseq_lib.start_section_addr,
            post_commit_offset,
            rseq_lib.abort_trampoline_addr,
        );

        let this_rseq_cs_ref: u64 = &this_rseq_cs as *const _ as u64;

        let mut cs_input = RseqCsInput::new(
            get_thread_rseq(),
            this_rseq_cs_ref,
            rseq_lib.get_function_ptr(self.main_symbol),
            Option::None,
            ctx,
        );

        rseq_cs_wrapper_function(&mut cs_input)
    }
}

pub fn find_offset(name: &str) -> Option<u64> {
    RSEQ_CS_POST_COMMIT_OFFSETS
        .binary_search_by_key(&name, |&(n, _)| n)
        .ok()
        .map(|index| RSEQ_CS_POST_COMMIT_OFFSETS[index].1)
}
