use std::usize;

use syscalls::{Errno, Sysno, syscall};

use enumflags2::{BitFlags, bitflags};

#[repr(i32)]
pub enum RseqCpuIdState {
    Uninitialized = -1i32,
    RegistrationFailed = -2i32,
}

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RseqFlags {
    Unregister = (1u32 << 0),
    SliceExtDefaultOn = (1u32 << 1),
}

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RseqCsFlags {
    // Historical and unsupported bits
    NoRestartOnPreempt = 1 << 0,
    NoRestartOnSignal = 1 << 1,
    NoRestartOnMigrate = 1 << 2,
    // (3) Intentional gap

    // User read only feature flags
    SliceExtAvailable = 1 << 4,
    SliceExtEnabled = 1 << 5,
}

#[repr(C, align(32))]
pub struct Rseq {
    cpu_id_start: u32,
    pub cpu_id: u32,
    pub rseq_cs: u64,
    flags: u32,
}

#[repr(C, align(32))]
pub struct RseqCs {
    version: u32,
    flags: u32,
    start_ip: u64,
    post_commit_offset: u64,
    abort_ip: u64,
}

impl RseqCs {
    pub fn new(start_ip: u64, post_commit_offset: u64, abort_ip: u64, rseq_sig: u32) -> Self {
        let sig_ptr = unsafe { (abort_ip as *const u32).offset(-1) };
        let found_sig = unsafe { *sig_ptr };

        if found_sig != rseq_sig {
            panic!(
                "SIGNATURE MISMATCH! Expected 0x53514552, found 0x{:x}. Kernel will give EINVAL!",
                found_sig
            );
        }

        Self {
            version: 0,
            flags: 0,
            start_ip: start_ip,
            post_commit_offset: post_commit_offset,
            abort_ip: abort_ip,
        }
    }

    pub fn get_post_commit_mut(&mut self) -> &mut u64 {
        &mut self.post_commit_offset
    }
}

pub fn sys_rseq(rseq_ptr: usize, flags: u32, rseq_sig: u32) -> Result<usize, Errno> {
    unsafe {
        syscall!(
            Sysno::rseq,
            rseq_ptr,
            std::mem::size_of::<Rseq>(),
            flags,
            rseq_sig
        )
    }
}

// #[thread_local]
static mut THREAD_RSEQ: Rseq = Rseq {
    cpu_id_start: 0,
    cpu_id: u32::MAX,
    rseq_cs: 0,
    flags: 0,
};

pub fn rseq_thread_registor(rseq_sig: u32, flags: BitFlags<RseqFlags>) {
    let rseq_ptr = &raw const THREAD_RSEQ as *const _ as usize;

    let cpu_id = unsafe { THREAD_RSEQ.cpu_id };
    assert_eq!(cpu_id, RseqCpuIdState::Uninitialized as u32);

    match { sys_rseq(rseq_ptr, flags.bits(), rseq_sig) } {
        Ok(0) => {}
        Err(errno) => {
            panic!("rseq registration failed {}", errno);
        }
        _ => {
            panic!("this shouldn't happen");
        }
    }
}

pub unsafe fn get_thread_rseq_cs_ref() -> *mut u64 {
    let cpu_id = unsafe { THREAD_RSEQ.cpu_id };
    assert_ne!(cpu_id, RseqCpuIdState::Uninitialized as u32);

    unsafe { &raw mut THREAD_RSEQ.rseq_cs }
}
