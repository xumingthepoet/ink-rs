//! Shared scanner primitives for syntax modules.
//!
//! Top-level syntax scanning belongs here. Parser modules should use these
//! helpers instead of maintaining local string/escape/nesting state machines;
//! add a `ScanOptions` mode when a syntax surface needs different rules.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ScanOptions {
    pub(super) track_strings: bool,
    pub(super) track_parentheses: bool,
    pub(super) track_braces: bool,
    pub(super) escape_outside_strings: bool,
}

impl ScanOptions {
    pub(super) const fn new() -> Self {
        Self {
            track_strings: true,
            track_parentheses: true,
            track_braces: true,
            escape_outside_strings: false,
        }
    }

    pub(super) const fn expression() -> Self {
        Self {
            track_strings: true,
            track_parentheses: true,
            track_braces: false,
            escape_outside_strings: false,
        }
    }

    pub(super) const fn inline_text() -> Self {
        Self {
            track_strings: true,
            track_parentheses: true,
            track_braces: true,
            escape_outside_strings: true,
        }
    }

    pub(super) const fn inline_tokens() -> Self {
        Self {
            track_strings: false,
            track_parentheses: false,
            track_braces: false,
            escape_outside_strings: true,
        }
    }
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct ScanState {
    in_string: bool,
    escaped: bool,
    paren_depth: usize,
    brace_depth: usize,
}

impl ScanState {
    fn advance(&mut self, ch: char, options: ScanOptions) {
        if self.escaped {
            self.escaped = false;
            return;
        }

        match ch {
            '\\' if self.in_string || options.escape_outside_strings => self.escaped = true,
            '"' if options.track_strings => self.in_string = !self.in_string,
            '(' if !self.in_string && options.track_parentheses => self.paren_depth += 1,
            ')' if !self.in_string && options.track_parentheses => {
                self.paren_depth = self.paren_depth.saturating_sub(1);
            }
            '{' if !self.in_string && options.track_braces => self.brace_depth += 1,
            '}' if !self.in_string && options.track_braces => {
                self.brace_depth = self.brace_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn is_top_level(&self) -> bool {
        !self.in_string && self.paren_depth == 0 && self.brace_depth == 0
    }

    fn can_match_top_level(&self) -> bool {
        !self.escaped && self.is_top_level()
    }
}

pub(super) fn find_top_level_char_with_options(
    source: &str,
    needle: char,
    options: ScanOptions,
) -> Option<usize> {
    let mut state = ScanState::default();

    for (index, ch) in source.char_indices() {
        if state.can_match_top_level() && ch == needle {
            return Some(index);
        }

        state.advance(ch, options);
    }

    None
}

pub(super) fn find_top_level_token<'token>(
    source: &str,
    tokens: &'token [&'token str],
) -> Option<(usize, &'token str)> {
    find_top_level_token_with_options(source, tokens, ScanOptions::default())
}

pub(super) fn find_top_level_token_with_options<'token>(
    source: &str,
    tokens: &'token [&'token str],
    options: ScanOptions,
) -> Option<(usize, &'token str)> {
    top_level_token_matches_with_options(source, tokens, options)
        .into_iter()
        .next()
}

pub(super) fn top_level_token_matches_with_options<'token>(
    source: &str,
    tokens: &'token [&'token str],
    options: ScanOptions,
) -> Vec<(usize, &'token str)> {
    let mut state = ScanState::default();
    let mut matches = Vec::new();

    for (index, ch) in source.char_indices() {
        if state.can_match_top_level() {
            if let Some(token) = tokens
                .iter()
                .copied()
                .find(|token| !token.is_empty() && source[index..].starts_with(token))
            {
                matches.push((index, token));
            }
        }

        state.advance(ch, options);
    }

    matches
}

pub(super) fn split_top_level_once_with_options(
    source: &str,
    separator: char,
    options: ScanOptions,
) -> Option<(&str, &str)> {
    let index = find_top_level_char_with_options(source, separator, options)?;
    Some((&source[..index], &source[index + separator.len_utf8()..]))
}

pub(super) fn split_top_level_with_options(
    source: &str,
    separator: char,
    options: ScanOptions,
) -> Vec<&str> {
    split_top_level_preserving_whitespace_with_options(source, separator, options)
        .into_iter()
        .map(str::trim)
        .collect()
}

pub(super) fn split_top_level_preserving_whitespace_with_options(
    source: &str,
    separator: char,
    options: ScanOptions,
) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = source;

    while let Some((left, right)) = split_top_level_once_with_options(rest, separator, options) {
        parts.push(left);
        rest = right;
    }

    parts.push(rest);
    parts
}

pub(super) fn find_matching_delimiter(
    source_after_open: &str,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in source_after_open.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            _ if ch == open && !in_string => depth += 1,
            _ if ch == close && !in_string => {
                if depth == 0 {
                    return Some(index);
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ignores_separators_inside_strings_and_nested_delimiters() {
        let parts = split_top_level_with_options(
            r#"one, "two, still two", call(3, 4), {x, y}"#,
            ',',
            ScanOptions::default(),
        );

        assert_eq!(
            parts,
            vec!["one", r#""two, still two""#, "call(3, 4)", "{x, y}"]
        );
    }

    #[test]
    fn split_respects_escaped_quotes_inside_strings() {
        let parts = split_top_level_with_options(
            r#"say("a \"quoted, value\""), done"#,
            ',',
            ScanOptions::default(),
        );

        assert_eq!(parts, vec![r#"say("a \"quoted, value\"")"#, "done"]);
    }

    #[test]
    fn char_and_split_searches_use_configurable_top_level_rules() {
        assert_eq!(
            find_top_level_char_with_options("left, call(a, b)", ',', ScanOptions::default()),
            Some(4)
        );
        assert_eq!(
            split_top_level_once_with_options(
                "condition: {nested: value}",
                ':',
                ScanOptions::default()
            ),
            Some(("condition", " {nested: value}"))
        );
    }

    #[test]
    fn expression_options_ignore_braces_but_track_strings_and_parentheses() {
        assert_eq!(
            find_top_level_char_with_options(
                r#""+" + call(1 + 2)"#,
                '+',
                ScanOptions::expression()
            ),
            Some(4)
        );
    }

    #[test]
    fn finds_top_level_tokens_after_nested_content() {
        let source = r#"choice { condition("->") } -> target"#;

        assert_eq!(find_top_level_token(source, &["->"]), Some((27, "->")));
    }

    #[test]
    fn token_matches_include_all_top_level_tokens() {
        let source = "a - call(b - c) - d";

        assert_eq!(
            top_level_token_matches_with_options(source, &["-"], ScanOptions::expression()),
            vec![(2, "-"), (16, "-")]
        );
    }

    #[test]
    fn finds_matching_delimiter_with_nested_pairs_and_strings() {
        let source_after_open = r#"a { "}" } } suffix"#;

        assert_eq!(
            find_matching_delimiter(source_after_open, '{', '}'),
            Some(10)
        );
    }

    #[test]
    fn inline_text_options_treat_escaped_separator_as_text() {
        let parts = split_top_level_with_options(
            r#"escaped \| separator | split"#,
            '|',
            ScanOptions::inline_text(),
        );

        assert_eq!(parts, vec![r#"escaped \| separator"#, "split"]);
    }
}
