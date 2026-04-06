use rseq_macros::{
    rseq_commit_action, rseq_critical_section, rseq_critical_section_start, rseq_shared_struct,
};
use rseq_utils::{RseqCommitActionName, RseqStart};

#[rseq_critical_section_start]
pub fn my_api_handler() {
    // a
}

#[rseq_critical_section]
pub fn helper_function() -> Test {
    Test { a: 1 }
}

#[rseq_commit_action]
pub fn commit(a: &mut Test) {
    a.a += 1;

    rseq_cs_end!();
}

#[rseq_critical_section_start]
pub fn secondary_func() {}

#[rseq_shared_struct]
#[warn(dead_code)]
struct Test {
    a: u64,
}

#[test]
fn test_type_safety() {
    let name_struct: RseqStart = my_api_handler;

    assert_eq!(name_struct.0, "my_api_handler");
}

#[test]
fn test_deref_behavior() {
    let name: &str = &*my_api_handler;
    assert_eq!(name, "my_api_handler");

    let formatted = format!("Name: {}", my_api_handler);
    assert_eq!(formatted, "Name: my_api_handler");
}

#[test]
fn test_function_accepts_only_function_name() {
    fn registry(name: RseqStart) -> String {
        name.0.to_string()
    }

    assert_eq!(registry(my_api_handler), "my_api_handler");
}

#[test]
fn test_visibility_in_integration() {
    assert_eq!((*secondary_func).len(), 14);
}
