use syscalls::{Errno, Sysno, syscall};

use enumflags2::{BitFlag, BitFlags};
pub use rseq_utils::rseq_types::{Rseq, RseqCpuIdState, RseqCs, RseqCsFlags, RseqFlags};

use rseq_utils::RSEQ_SIG;

pub trait RseqCsExt {
    fn new(start_ip: u64, post_commit_offset: u64, abort_ip: u64) -> Self;
    fn get_post_commit_mut(&mut self) -> &mut u64;
}

impl RseqCsExt for RseqCs {
    fn new(start_ip: u64, post_commit_offset: u64, abort_ip: u64) -> Self {
        let sig_ptr = unsafe { (abort_ip as *const u32).offset(-1) };
        let found_sig = unsafe { *sig_ptr };

        if found_sig != RSEQ_SIG {
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

    fn get_post_commit_mut(&mut self) -> &mut u64 {
        &mut self.post_commit_offset
    }
}

pub fn sys_rseq(rseq_ptr: usize, flags: u32, rseq_sig: u32) -> Result<usize, Errno> {
    unsafe {
        syscall!(
            Sysno::rseq,
            rseq_ptr,
            std::mem::size_of::<Rseq>() as u32,
            flags,
            rseq_sig
        )
    }
}

pub fn rseq_thread_registor(flags: BitFlags<RseqFlags>) {
    let rseq = get_thread_rseq();
    let rseq_addr = rseq as usize;

    let cpu_id = unsafe { (*rseq).cpu_id };
    assert_eq!(cpu_id, RseqCpuIdState::Uninitialized as u32);

    match { sys_rseq(rseq_addr, flags.bits(), RSEQ_SIG) } {
        Ok(0) => {}
        Err(errno) => {
            panic!("rseq registration failed {}", errno);
        }
        _ => {
            panic!("this shouldn't happen");
        }
    }
}

pub fn rseq_thread_unregistor() {
    let rseq = get_thread_rseq();
    let rseq_addr = rseq as usize;

    let cpu_id = unsafe { (*rseq).cpu_id };
    assert_ne!(cpu_id, RseqCpuIdState::Uninitialized as u32);

    match { sys_rseq(rseq_addr, RseqFlags::Unregister as u32, RSEQ_SIG) } {
        Ok(0) => {}
        Err(errno) => {
            panic!("rseq registration failed {}", errno);
        }
        _ => {
            panic!("this shouldn't happen");
        }
    }
}

pub fn get_thread_rseq() -> *mut Rseq {
    #[thread_local]
    static mut RSEQ: Rseq = Rseq {
        cpu_id_start: 0,
        cpu_id: u32::MAX,
        rseq_cs: 0,
        flags: 0,
        pad: [0; 3],
    };

    #[thread_local]
    static mut IS_RSEQ_INIT: bool = false;

    unsafe {
        if !IS_RSEQ_INIT {
            IS_RSEQ_INIT = true;
            rseq_thread_registor(RseqFlags::empty());
        }

        std::ptr::addr_of_mut!(RSEQ)
    }
}

pub unsafe fn get_thread_rseq_cs_ref() -> *mut u64 {
    let rseq = get_thread_rseq();
    let cpu_id = unsafe { (*rseq).cpu_id };
    assert_ne!(cpu_id, RseqCpuIdState::Uninitialized as u32);

    unsafe { &mut (*rseq).rseq_cs }
}
