// #![feature(core_intrinsics)]
//
use std::arch::{asm, global_asm};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use syscalls::{Sysno, syscall};

const RSEQ_SIG: u32 = 0x53053053;

#[repr(C, align(32))]
struct RseqCs {
    version: u32,
    flags: u32,
    start_ip: u64,
    post_commit_offset: u64,
    abort_ip: u64,
}

#[repr(C, align(32))]
struct Rseq {
    cpu_id_start: u32,
    cpu_id: u32,
    rseq_cs: *const RseqCs,
    flags: u32,
}

// #[thread_local]
static mut RSEQ: Rseq = Rseq {
    cpu_id_start: 0,
    cpu_id: !0,
    rseq_cs: ptr::null(),
    flags: 0,
};

extern "C" fn abort_rust() {
    // Custom abort logic in Rust
    println!("rseq aborted, running cleanup");
    unsafe {
        RSEQ.rseq_cs = ptr::null();
    }
}

unsafe fn register_rseq() {
    match {
        syscall!(
            Sysno::rseq,
            &raw const RSEQ as *const Rseq,
            std::mem::size_of::<Rseq>(),
            0u64,
            RSEQ_SIG as u64
        )
    } {
        Ok(0) => return,
        Err(errno) => {
            panic!("rseq registration failed {}", errno);
        }
        _ => {
            panic!("haaaa");
        }
    }
}

fn increment_counter(counter: &AtomicUsize) {
    let mut start: *const u8 = ptr::null();
    let mut abort: *const u8 = ptr::null();
    let mut end: *const u8 = ptr::null();

    let mut c = 0u64;

    let mut RSEQ_CS: RseqCs = RseqCs {
        version: 0,
        flags: 0,
        start_ip: 0,
        post_commit_offset: 0,
        abort_ip: 0,
    };
        println!("c {}", c);
    c = 1;
        println!("c {}", c);
    unsafe {
        asm!(
            "lea rax, [rip + 90f]",
            "mov {start_out}, rax",
            options(nostack, nomem),
            start_out = out(reg) start,
        );

        asm!(
            "lea rax, [91f + rip]",
            "mov {end_out}, rax",
            options(nostack, nomem),
            end_out = out(reg) end,
        );

        asm!(
            ".align 8",
                    "lea rax, [rip + 92f]",
                    "mov {about_ip}, rax",
                    "jmp 92f",
                    ".long 0x53053053",
                    "92:",
            options(nostack, nomem),
            about_ip = out(reg) abort
        );

        c += 1;
        // println!("c {}", c);

        if RSEQ_CS.start_ip == 0 {
            RSEQ_CS.start_ip = start as u64;
            RSEQ_CS.post_commit_offset = end as u64 - start as u64;
            RSEQ_CS.abort_ip = abort as u64;
        }

        RSEQ.rseq_cs = &RSEQ_CS;
        asm!("90:");
        // counter.fetch_add(1, Ordering::SeqCst);
  
        counter.fetch_add(1, Ordering::SeqCst);

        asm!("91:");

        if RSEQ.rseq_cs == 0 as *const RseqCs {
            panic!("aaa");
        }

        RSEQ.rseq_cs = ptr::null();
        println!("{}", c);
    }
}

fn main() {
    unsafe {
        register_rseq();
    }
    let counter = AtomicUsize::new(0);
    for _ in 0..1_000_000_000u64 {
        increment_counter(&counter);
        println!("Counter: {}", counter.load(Ordering::Relaxed));
    }
    println!("Counter: {}", counter.load(Ordering::Relaxed));
}
