//! Tests for the serverless.rs macros
//!
//! This file contains the test harness for the trybuild tests
//! that compile and run small programs that use the macros.

#[test]
fn pass_tests() {
    let t = trybuild::TestCases::new();

    t.pass("tests/01-basic-handler.rs");
    t.pass("tests/02-with-route.rs");
    t.pass("tests/03-with-requirements.rs");
}
