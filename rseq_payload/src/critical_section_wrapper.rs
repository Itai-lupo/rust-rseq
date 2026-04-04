use crate::musl_binding::{jmp_buf, longjmp, setjmp, write};
use rseq_utils::RseqCsInput;

use core::ptr;

// Use the generated jmp_buf type
// let mut buf: bindings::jmp_buf = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };

#[thread_local]
pub static mut RSEQ_CONTEXT: jmp_buf = [unsafe { core::mem::MaybeUninit::zeroed().assume_init() }];

#[unsafe(link_section = ".rseq_abort")]
#[inline(never)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rseq_cs_wrapper(rseq_data: &mut RseqCsInput) {
    // if there is an abort or the rseq finshed we return here,
    // this has to be out side of the cs to prevent abort after finish bugs
    // this
    match unsafe { setjmp(&raw mut RSEQ_CONTEXT[0]) } {
        0 => {}
        2 => {
            unsafe { ptr::write_volatile(&mut (*rseq_data.rseq).rseq_cs, 0) };
            return;
        }
        _ => {}
    }

    rseq_cs_wrapper_inner(rseq_data)
}

#[unsafe(link_section = ".rseq_critical")]
pub fn rseq_cs_wrapper_inner(rseq_data: &mut RseqCsInput) {
    unsafe {
        ptr::write_volatile(
            &mut (*rseq_data.rseq).rseq_cs,
            rseq_data.critical_section_to_use,
        );
    }

    unsafe { (rseq_data.cs_callback)(rseq_data.user_data) };

    unsafe { ptr::write_volatile(&mut (*rseq_data.rseq).rseq_cs, 0) };
    panic!("when rseq cs finish it should use longjmp to get back it should never get here.");
}

#[unsafe(no_mangle)]
#[inline(never)]
#[unsafe(link_section = ".rseq_abort")]
pub fn rseq_end_handler() {
    /*  let msg = b"[RSEQ SO] rseq ended! Jumping to longjmp...\n";
    unsafe {
        write(2, msg.as_ptr(), msg.len());
    } */
    jmp_to_rseq_start(2);
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rseq_end_handler_call_marker() {
    core::arch::naked_asm!(
        "jmp 91f",
        ".long 0xDEADC0DE",
        "91:",
        ".long 0xABCDEFFF",
        // the call might be diffrent bytecode so it needs to be after the magic itself
        // we can put whatever code we want here and it will be outside the rseq cs
        "call rseq_end_handler",
    )
}

#[macro_export]
macro_rules! rseq_cs_end {
    () => {
        unsafe {
            asm!(
                "jmp 91f",
                ".long 0xDEADC0DE",
                "91:",
                options(nostack, preserves_flags)
            );
            $crate::critical_section_wrapper::rseq_end_handler();
        }
    };
}

#[unsafe(link_section = ".rseq_abort")]
pub fn jmp_to_rseq_start(jmp_val: i32) {
    unsafe {
        longjmp(&raw mut RSEQ_CONTEXT as *mut _, jmp_val);
    }
}
