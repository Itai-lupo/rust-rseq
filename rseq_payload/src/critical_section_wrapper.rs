use crate::musl_binding::{jmp_buf, longjmp, setjmp, write};
use rseq_utils::RseqCsInput;

use core::ptr;

// Thread-local storage to hold the context safely
// #[thread_local]
pub static mut RSEQ_CONTEXT: jmp_buf = jmp_buf([0; 8]);

#[unsafe(link_section = ".rseq_abort")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rseq_cs_wrapper(rseq_data: &mut RseqCsInput) {
    // if there is an abort or the rseq finshed we return here,
    // this has to be out side of the cs to prevent abort after finish bugs
    // this
    match unsafe { setjmp(&raw mut RSEQ_CONTEXT as *mut _) } {
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
    let msg = b"[RSEQ SO] rseq ended! Jumping to longjmp...\n";
    unsafe {
        // write(2, msg.as_ptr(), msg.len());
    }
    jmp_to_rseq_start(2);
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rseq_end_handler_call_marker() {
    core::arch::naked_asm!("call rseq_end_handler")
}

#[unsafe(link_section = ".rseq_abort")]
pub fn jmp_to_rseq_start(jmp_val: i32) {
    unsafe {
        longjmp(&raw mut RSEQ_CONTEXT as *mut _, jmp_val);
    }
}
