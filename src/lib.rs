#![feature(thread_local)]
pub mod rseq_lib_wrapper;
pub mod sys_rseq;

include!(concat!(env!("OUT_DIR"), "/post_commit_offsets.rs"));

pub use rseq_utils::{RSEQ_SIG, RseqCsInput};
pub use sys_rseq::{RseqCs, RseqCsExt, RseqFlags, get_thread_rseq, rseq_thread_registor};

pub use enumflags2::BitFlag;
pub use rseq_lib_wrapper::RseqSo;

pub fn find_offset(name: &str) -> Option<u64> {
    RSEQ_CS_POST_COMMIT_OFFSETS
        .binary_search_by_key(&name, |&(n, _)| n)
        .ok()
        .map(|index| RSEQ_CS_POST_COMMIT_OFFSETS[index].1)
}
