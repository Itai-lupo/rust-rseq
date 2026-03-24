#![no_std]
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
    // This is the very last instruction of the critical section
    // todo need to find a way to get the exect addr of this call, might need to do naked_asm here?
    unsafe {
        *ptr = *ptr + 1;
        asm!("call rseq_end_handler");
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
