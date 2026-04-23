pub(super) fn expected_message(expected: &str, saw: &str) -> String {
    if saw.is_empty() {
        format!("Expected {expected} but saw end of line")
    } else {
        format!("Expected {expected} but saw '{saw}'")
    }
}
