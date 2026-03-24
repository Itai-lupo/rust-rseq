use enumflags2::{bitflags};

#[repr(i32)]
pub enum RseqCpuIdState {
    Uninitialized = -1i32,
    RegistrationFailed = -2i32,
}

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RseqFlags {
    Unregister = (1u32 << 0),
    SliceExtDefaultOn = (1u32 << 1),
}

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RseqCsFlags {
    // Historical and unsupported bits
    NoRestartOnPreempt = 1 << 0,
    NoRestartOnSignal = 1 << 1,
    NoRestartOnMigrate = 1 << 2,
    // (3) Intentional gap

    // User read only feature flags
    SliceExtAvailable = 1 << 4,
    SliceExtEnabled = 1 << 5,
}

#[repr(C, align(32))]
pub struct Rseq {
    pub cpu_id_start: u32,
    pub cpu_id: u32,
    pub rseq_cs: u64,
    pub flags: u32,
    pub pad: [u32; 3],
}

#[repr(C, align(32))]
pub struct RseqCs {
    pub version: u32,
    pub flags: u32,
    pub start_ip: u64,
    pub post_commit_offset: u64,
    pub abort_ip: u64,
}

