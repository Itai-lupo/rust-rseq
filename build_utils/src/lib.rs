mod generate_payload_so;
mod generate_post_commit_offsets;
mod generate_rseq_gen;
mod parse_rseq_macros;

pub use generate_post_commit_offsets::process_functions_in_so;
pub use generate_rseq_gen::genrate_rseq_code;

use snafu::prelude::*;
use snafu::Backtrace;
use snafu::Location;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum RseqBuildError {
    // --- שגיאות פנימיות ---
    #[snafu(display("Function '{name}' has wrong argument count: expected 2 parameters: (*c_void, u32), got {count}.\n at {location}"))]
    RseqStartWrongArgumentCount {
        name: String,
        count: usize,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(
        "Function '{name}' has invalid first argument: expected *mut c_void, got '{actual}'"
    ))]
    RseqStartFirstArgInvalid {
        name: String,
        actual: String,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(
        "Function '{name}' has invalid second argument: expected u32, got '{actual}'"
    ))]
    RseqStartSecondArgInvalid {
        name: String,
        actual: String,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Function '{name}' has invalid return type: expected Result<*mut c_void, E>, got '{actual}'"))]
    RseqStartReturnInvalid {
        name: String,
        actual: String,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("section '{section_name}' not found in rseq so"))]
    RseqCommitSectionNotFound {
        section_name: String,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Data for symbol '{symbol_name}' is not present in the so file"))]
    SymbolDataNotFound {
        symbol_name: String,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("problem with symbol '{symbol_name}': {message}"))]
    GenricSymbolError {
        symbol_name: String,
        message: String,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("rseq end magic not found for symbol in rseq commet section"))]
    MagicNotFound {
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("rseq end magic found multiple times for symbolin rseq commet section"))]
    MagicFoundMultipleTimes {
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    // --- שגיאות חיצוניות ---
    #[snafu(display("IO error: {source}"))]
    Io {
        #[snafu(source(from(std::io::Error, Box::new)))]
        source: Box<std::io::Error>,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("JSON parsing failed: {source}"))]
    JsonError {
        #[snafu(source(from(serde_json::Error, Box::new)))]
        source: Box<serde_json::Error>,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    // שינוי כאן: הסרת transparent כי הוא מתנגש עם ניהול Backtrace ידני
    #[snafu(display("Eyre error: {source}"))]
    ColorEyre {
        #[snafu(source(from(color_eyre::eyre::Error, Box::new)))]
        source: Box<color_eyre::eyre::Error>,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("Failed to fetch cargo metadata: {source}"))]
    CargoMetadata {
        #[snafu(source(from(cargo_metadata::Error, Box::new)))]
        source: Box<cargo_metadata::Error>,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    // שינוי כאן: הסרת transparent
    #[snafu(display("Object error: {source}"))]
    Object {
        #[snafu(source(from(object::Error, Box::new)))]
        source: Box<object::Error>,
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },

    // #[snafu(display("Error at {action}: {source} (at {location})"))]
    // GenericContext {
    //     source: Box<dyn std::error::Error + Send + Sync>,
    //     action: String,
    //     backtrace: Backtrace,
    //     #[snafu(implicit)]
    //     location: Location,
    // },

    // #[snafu(whatever, display("{message}"))]
    // Whatever {
    //     message: String,
    //     #[snafu(source)]
    //     source: Option<Box<dyn std::error::Error + Send + Sync>>,
    //     backtrace: Backtrace,
    // },
    #[snafu(display("{message}: {source} (at {location})"))]
    Context {
        message: String,
        source: Box<RseqBuildError>, // עוטף שגיאה קיימת מהטיפוס שלך
        backtrace: Backtrace,
        #[snafu(implicit)]
        location: Location,
    },
}

impl RseqBuildError {
    #[track_caller]
    pub fn wrap(self, msg: impl Into<String>) -> Self {
        let caller = std::panic::Location::caller();
        RseqBuildError::Context {
            message: msg.into(),
            source: Box::new(self),
            backtrace: snafu::Backtrace::capture(),
            location: snafu::Location::new(caller.file(), caller.line(), caller.column()),
        }
    }
}

pub type Result<T> = color_eyre::Result<T, RseqBuildError>;
