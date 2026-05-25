//! Compile-fail tests that verify the macro produces clear diagnostics for
//! invalid usage.  Run with `TRYBUILD=overwrite cargo test ui` on first setup
//! to generate / update the `.stderr` snapshot files.

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
