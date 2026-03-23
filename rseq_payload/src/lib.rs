#![no_std]
use core::arch::asm;
use core::panic::PanicInfo;

// Define the jump buffer size for musl x86_64 (typically 8 words)
#[repr(C, align(16))]
pub struct jmp_buf([u64; 8]);

unsafe extern "C" {
    // musl implementation of setjmp/longjmp
    pub fn setjmp(env: *mut jmp_buf) -> i32;
    pub fn longjmp(env: *const jmp_buf, val: i32) -> !;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}

// Thread-local storage to hold the context safely
// #[thread_local]
static mut RSEQ_CONTEXT: jmp_buf = jmp_buf([0; 8]);

#[unsafe(link_section = ".rseq_critical")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rseq_critical_store(ptr: *mut u64, rseq_cs: &mut u64, this_cs: u64) {
    unsafe {
        *ptr = 0;
    };
    match unsafe { setjmp(&raw mut RSEQ_CONTEXT as *mut _) } {
        0 => {}
        _ => {}
    }
    //start the cs
    unsafe {
        *rseq_cs = this_cs;

        *ptr = *ptr + 1;
    }
    for _i in 1..100000000u64 {
        unsafe { asm!("") };
    }

    commit_action();
}

#[unsafe(no_mangle)]
#[inline(never)]
#[unsafe(link_section = ".rseq_commit")]
pub fn commit_action() {
    // This is the very last instruction of the critical section
    // todo need to find a way to get the exect addr of this call, might need to do naked_asm here?
    unsafe {
        asm!("");
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

const RSEQ_SIG: u32 = parse_u32!(env!("RSEQ_SIG"));

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
    let msg = b"[RSEQ SO] Abort detected! Jumping to longjmp...\n";
    unsafe {
        write(2, msg.as_ptr(), msg.len());
        longjmp(&raw mut RSEQ_CONTEXT as *mut _, 1);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
