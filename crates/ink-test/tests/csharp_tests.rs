#![cfg(feature = "csharp-tests")]

#[test]
fn csharp_compatibility_suite_is_retired() {
    // The old ported C# suite was intentionally removed from the integration
    // gate because it exercised legacy root-story syntax through inline source
    // strings. Current integration coverage lives in module-syntax .ink
    // fixtures instead.
}
