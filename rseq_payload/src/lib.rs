#![no_std]
#![feature(thread_local)]
use core::arch::asm;
use core::panic::PanicInfo;

mod abort_handler;
mod critical_section_wrapper;
mod musl_binding;

#[unsafe(link_section = ".rseq_critical")]
#[unsafe(no_mangle)]
fn rseq_cs_func(ptr: *mut u64) {
    for _i in 1..100000000u64 {
        unsafe { asm!("") };
    }

    commit_action(ptr);
    for _i in 1..100000000u64 {
        unsafe { asm!("") };
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
#[unsafe(link_section = ".rseq_commit")]
pub fn commit_action(ptr: *mut u64) {
    unsafe {
        *ptr = *ptr + 1;
        rseq_cs_end!();
    }
}
#[unsafe(no_mangle)]
#[inline(never)]
#[unsafe(link_section = ".rseq_commit")]
pub fn commit_action2(ptr: *mut u64) {
    unsafe {
        *ptr = *ptr + 5;
        rseq_cs_end!();
    }
}

#[unsafe(no_mangle)]
#[inline(never)]
#[unsafe(link_section = ".rseq_commit")]
pub fn commit_action3(ptr: *mut u64) {
    unsafe {
        *ptr = 7 * *ptr;
        rseq_cs_end!();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
