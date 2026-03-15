#![no_std]
use core::panic::PanicInfo;
use core::arch::asm;
#[link_section = ".rseq_sig"]
#[no_mangle]
pub static RSEQ_SIG: u32 = 0x12345678;

#[link_section = ".rseq_logic"]
#[no_mangle]
pub unsafe extern "C" fn rseq_critical_store(ptr: *mut u64, val: u64) {
    // Strings work fine in .so!
    // In real RSEQ, don't do slow IO here, but for demo:
    *ptr = val;
    for i in 1..1_000_000_000u64{ asm!("")}
}

#[link_section = ".rseq_abort"]
#[no_mangle]
pub unsafe extern "C" fn rseq_abort_handler() {
    println!("a");
    // Kernel jumps here on preemption
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
