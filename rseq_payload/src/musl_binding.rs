include!(concat!(env!("OUT_DIR"), "/libc_bindings.rs"));

unsafe extern "C" {
    pub fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}
