#![cfg(feature = "csharp-tests")]
#![allow(
    unused_variables,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]

#[path = "conformance/api.rs"]
mod api;
#[path = "csharp_tests/mod.rs"]
mod csharp_tests;
