use crate::critical_section_wrapper::jmp_to_rseq_start;

// use crate::musl_binding::write;

use rseq_utils::RSEQ_SIG;

#[unsafe(link_section = ".rseq_abort")]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rseq_abort_handler() {
    core::arch::naked_asm!(
        ".long {sig}",
        // Kernel jumps here on preemption
        // this stack is in a invalid state
        // we can use call as it only add to the stack
        // but we can't return as the stack register point at the last thing it pointed in the cs
        // and we don't know what it is
        "call inner_abort_handler",
        // if we return from here it will be undefined behavier so it is better to hard crush
        "ud2",
        sig = const RSEQ_SIG
    );
}

#[unsafe(link_section = ".rseq_abort")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn inner_abort_handler() {
    // let msg = b"[RSEQ SO] Abort detected! Jumping to longjmp...\n";
    // unsafe { write(2, msg.as_ptr(), msg.len()) };

    jmp_to_rseq_start(1);
}
