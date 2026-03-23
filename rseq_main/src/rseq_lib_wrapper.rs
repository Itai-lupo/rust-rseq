use dlopen_rs::{ElfLibrary, OpenFlags};
use std::sync::OnceLock;

const PAYLOAD_SO: &[u8] = include_bytes!(env!("PAYLOAD_SO"));

pub static RSEQ_LIB: OnceLock<RseqSo> = OnceLock::new();

pub struct RseqSo {
    pub lib: ElfLibrary,

    pub start_section_addr: u64,
    pub commit_section_end: u64,
    pub abort_trampoline_addr: u64,
}

const RSEQ_START: &str = "rseq_start";
const RSEQ_COMMIT_END: &str = "rseq_commit_end";
const RSEQ_ABORT_IP: &str = "rseq_abort_ip";

const RSEQ_LIB_FLAGS: OpenFlags = OpenFlags::RTLD_NOW;

impl RseqSo {
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<RseqSo> = OnceLock::new();
        INSTANCE.get_or_init(|| Self::init())
    }

    fn init() -> Self {
        let lib = match {
            ElfLibrary::dlopen_from_binary(PAYLOAD_SO, "librseq_payload.so", RSEQ_LIB_FLAGS)
        } {
            Ok(lib) => lib,
            Err(e) => {
                panic!("got error {} while loading librseq_payload.so", e)
            }
        };

        let mut res = Self {
            lib,
            start_section_addr: 0,
            commit_section_end: 0,
            abort_trampoline_addr: 0,
        };

        res.start_section_addr = res.get_symbol_wrapper::<u64>(RSEQ_START);
        res.commit_section_end = res.get_symbol_wrapper::<u64>(RSEQ_COMMIT_END);
        res.abort_trampoline_addr = res.get_symbol_wrapper::<u64>(RSEQ_ABORT_IP);

        res
    }

    fn get_symbol_wrapper<T>(&self, symbol_name: &str) -> T {
        match unsafe { self.lib.get::<T>(symbol_name) } {
            Ok(symbol) => unsafe { std::ptr::read(&*symbol) },
            Err(e) => {
                panic!("Failed to load symbol '{}': {}", symbol_name, e)
            }
        }
    }

    pub fn get_function_addr(&self, fun_name: &str) -> u64 {
        self.get_symbol_wrapper::<u64>(fun_name)
    }
}
