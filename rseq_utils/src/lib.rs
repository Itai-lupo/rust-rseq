#![cfg_attr(not(feature = "std"), no_std)]

pub mod rseq_types;

use core::ffi::c_void;
use rseq_types::Rseq;

pub struct RseqCsInput {
    pub rseq: *mut Rseq,
    pub critical_section_to_use: u64,

    // this will be called at the start of the rseq cs
    pub cs_callback: unsafe extern "C" fn(*mut c_void),

    // this should have only one function call at the end to rseq_end_handler and the 1 instraction commit right before it.
    // it's best to verfy only this function using objdump -d -j .rseq_commit <path_to_librseq_so>
    // the call might not be at the end but you should check that the commit instaction is right
    // before the call
    // pub commit_action_callback: unsafe extern "C" fn(*mut c_void),

    // this will be called if the critical section was aborted.
    pub abort_callback: Option<unsafe extern "C" fn()>,

    pub user_data: *mut c_void,
}

impl RseqCsInput {
    pub fn new(
        rseq: *mut Rseq,
        critical_section_to_use: u64,
        cs_callback: unsafe extern "C" fn(*mut c_void),
        // commit_action_callback: unsafe extern "C" fn(*mut c_void),
        abort_callback: Option<unsafe extern "C" fn()>,
        user_data: *mut c_void,
    ) -> Self {
        Self {
            rseq,
            critical_section_to_use,
            cs_callback,
            // commit_action_callback,
            abort_callback,
            user_data,
        }
    }
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

pub const RSEQ_SIG: u32 = parse_u32!(env!("RSEQ_SIG"));

#[cfg(test)]
mod tests {
    use super::*;
}
