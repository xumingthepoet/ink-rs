#![cfg(feature = "csharp-tests")]
#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]

#[path = "conformance/mod.rs"]
mod conformance;
#[path = "csharp_tests/mod.rs"]
mod csharp_tests;
