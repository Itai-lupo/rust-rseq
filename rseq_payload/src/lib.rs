#![no_std]
#![feature(thread_local)]
use core::arch::asm;
use core::panic::PanicInfo;

pub mod abort_handler;
pub mod critical_section_wrapper;

pub use critical_section_wrapper::{rseq_cs_wrapper, rseq_end_handler_call_marker};

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(improper_ctypes)]
mod musl_binding;

pub use core::alloc::{GlobalAlloc, Layout};

struct NoAlloc;
unsafe impl GlobalAlloc for NoAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        panic!("heap allocation disabled")
    }
    unsafe fn alloc_zeroed(&self, _layout: Layout) -> *mut u8 {
        panic!("heap allocation disabled")
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
    unsafe fn realloc(&self, _ptr: *mut u8, _layout: Layout, _new_size: usize) -> *mut u8 {
        panic!("heap allocation disabled")
    }
}

// make sure there arn't any non stack allocation within the rseq
#[global_allocator]
static ALLOC: NoAlloc = NoAlloc;

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

#[cfg(has_user_tasks)]
include!(concat!(env!("OUT_DIR"), "/all_user_tasks.rs"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
