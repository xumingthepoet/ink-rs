use super::super::{is_identifier, scan};

pub(in crate::syntax) fn split_top_level_args(source: &str) -> Vec<&str> {
    scan::split_top_level_with_options(source, ',', scan::ScanOptions::expression())
}

pub(super) fn is_path_identifier(source: &str) -> bool {
    source.split('.').all(is_identifier)
}
