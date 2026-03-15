#![feature(thread_local)]

use dlopen_rs::{ElfLibrary, OpenFlags, Result};

use std::ptr;

use syscalls::{syscall, Sysno};

// הטמעת ה-Payload שנבנה על ידי ה-build.rs כבייטים בתוך ה-Binary
const PAYLOAD_SO: &[u8] =
    include_bytes!("../../target/release/librseq_payload.so");

// קבועים עבור ה-Kernel
const RSEQ_SIG: u32 = 0x12345678;
const SYS_RSEQ: usize = 334;

// מבנה ה-RSEQ ABI (חייב להיות Aligned ל-32 בתים)
#[repr(C, align(32))]
struct Rseq {
    cpu_id_start: u32,
    cpu_id: u32,
    rseq_cs: u64,
    flags: u32,
}

// מבנה ה-Critical Section Descriptor
#[repr(C, align(32))]
struct RseqCs {
    version: u32,
    flags: u32,
    start_ip: u64,
    post_commit_offset: u64,
    abort_ip: u64,
}

// יצירת מופע Rseq לכל Thread
#[thread_local]
static mut THREAD_RSEQ: Rseq = Rseq {
    cpu_id_start: 0,
    cpu_id: u32::MAX, // -1 (לא מאותחל)
    rseq_cs: 0,
    flags: 0,
};

fn main() -> Result<()> {
    // let lib = ElfLibrary::dlopen_from_binary(PAYLOAD_SO, "librseq_payload.so", OpenFlags::RTLD_NOW)?;

    let lib =
        ElfLibrary::dlopen("target/release/librseq_payload.so", OpenFlags::RTLD_LAZY)?;

    // 2. שליפת כתובות ה-RSEQ מה-Payload
    // אלו כתובות זיכרון אמיתיות שה-dlopen-rs חישב עבורנו
    let start_addr = unsafe{*lib.get::<u64>("rseq_critical_store")?};
    let post_commit_addr = unsafe{*lib.get::<u64>("rseq_abort_handler")?};
    let abort_addr = unsafe{*lib.get::<u64>("rseq_abort_handler")?};

    // 3. הכנת ה-Descriptor (חייב להיות Static כדי שה-Kernel יוכל לקרוא אותו תמיד)
    static mut CS_DESC: RseqCs = RseqCs {
        version: 0,
        flags: 0,
        start_ip: 0,
        post_commit_offset: 0,
        abort_ip: 0,
    };

    unsafe {
        CS_DESC.start_ip = start_addr;
        CS_DESC.post_commit_offset = post_commit_addr - start_addr;
        CS_DESC.abort_ip = abort_addr;
    }

    // 4. רישום ה-Thread מול ה-Kernel באמצעות syscall
    unsafe {
        let rseq_ptr = &THREAD_RSEQ as *const _ as usize;
        let rseq_len = std::mem::size_of::<Rseq>();

        unsafe {
            match {
                syscall!(
                    Sysno::rseq,
                    rseq_ptr,
                    std::mem::size_of::<Rseq>(),
                    0u64,
                    RSEQ_SIG as u64
                )
            } {
                Ok(0) => {}
                Err(errno) => {
                    panic!("rseq registration failed {}", errno);
                }
                _ => {
                    panic!("haaaa");
                }
            }
        }
    }

    println!("--- RSEQ Gold Standard Initialized ---");
    println!("Start IP: 0x{:x}", start_addr);
    println!("Abort IP: 0x{:x}", abort_addr);

    // 5. הרצת ה-Critical Section
    unsafe {
        // טעינת ה-Descriptor לתוך ה-Rseq ABI של ה-Thread
        THREAD_RSEQ.rseq_cs = &CS_DESC as *const _ as u64;

        // המרה של כתובת ה-Start לפונקציה והרצתה
        let rseq_func: unsafe extern "C" fn(*mut u64, u64) = std::mem::transmute(start_addr);
        let mut counter: u64 = 0;

        println!("Executing RSEQ logic...");
        rseq_func(&mut counter as *mut u64, 101);

        println!("Counter result: {}", counter);

        // ניקוי בסיום (חשוב למקרה שה-Thread ממשיך לרוץ)
        THREAD_RSEQ.rseq_cs = 0;
    }

    Ok(())
}
