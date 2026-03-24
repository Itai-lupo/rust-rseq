// Define the jump buffer size for musl x86_64 (typically 8 words)
#[repr(C, align(16))]
pub struct jmp_buf(pub [u64; 8]);

unsafe extern "C" {
    // musl implementation of setjmp/longjmp
    pub fn setjmp(env: *mut jmp_buf) -> i32;
    pub fn longjmp(env: *const jmp_buf, val: i32) -> !;
    pub fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}
