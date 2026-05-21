pub fn tag_value(tags: &[String], key: &str) -> Option<String> {
    tags.iter().rev().find_map(|tag| tag_value_in_tag(tag, key))
}

pub fn tag_marker(tags: &[String], key: &str) -> bool {
    tags.iter().any(|tag| tag_marker_in_tag(tag, key))
}

pub fn tag_bool(tags: &[String], key: &str) -> Option<bool> {
    let value = tag_value(tags, key)?.to_ascii_lowercase();
    match value.as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn tag_value_in_tag(tag: &str, key: &str) -> Option<String> {
    let tag = tag.trim().trim_start_matches('#').trim();
    let key_start = last_key_assignment_start(tag, key)?;
    let value_start = key_start + key.len() + 1;
    let value = parse_tag_value(&tag[value_start..]);

    Some(normalize_tag_value(value))
}

fn tag_marker_in_tag(tag: &str, key: &str) -> bool {
    let tag = tag.trim().trim_start_matches('#').trim();
    tag.split_whitespace()
        .any(|entry| entry.eq_ignore_ascii_case(key))
}

fn last_key_assignment_start(tag: &str, key: &str) -> Option<usize> {
    let mut found = None;
    for (index, _) in tag.char_indices() {
        if !is_entry_start(tag, index) {
            continue;
        }

        let rest = &tag[index..];
        let Some(candidate_key) = rest.get(..key.len()) else {
            continue;
        };
        if !candidate_key.eq_ignore_ascii_case(key) {
            continue;
        }

        let separator = rest[key.len()..].chars().next();
        if matches!(separator, Some(':') | Some('=')) {
            found = Some(index);
        }
    }

    found
}

fn is_entry_start(tag: &str, index: usize) -> bool {
    index == 0
        || tag[..index]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
}

fn parse_tag_value(value: &str) -> &str {
    let value = value.trim_start();
    let Some(first) = value.chars().next() else {
        return value;
    };

    if first == '"' || first == '\'' {
        let value_start = first.len_utf8();
        let value_body = &value[value_start..];
        let end = value_body
            .char_indices()
            .find_map(|(index, character)| (character == first).then_some(index))
            .unwrap_or(value_body.len());
        return &value_body[..end];
    }

    let end = next_key_assignment_offset(value).unwrap_or(value.len());
    value[..end].trim_end()
}

fn next_key_assignment_offset(value: &str) -> Option<usize> {
    for (index, character) in value.char_indices() {
        if !character.is_whitespace() {
            continue;
        }

        let next_index = index + character.len_utf8();
        if starts_with_key_assignment(&value[next_index..]) {
            return Some(index);
        }
    }

    None
}

fn starts_with_key_assignment(value: &str) -> bool {
    let Some(first) = value.chars().next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    for (index, character) in value.char_indices() {
        if matches!(character, ':' | '=') {
            return index > 0;
        }
        if character.is_whitespace() {
            return false;
        }
        if !character.is_ascii_alphanumeric() && character != '_' && character != '-' {
            return false;
        }
    }

    false
}

fn normalize_tag_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('{')
        .trim_matches('}')
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_value_reads_key_value_pairs_from_single_or_multi_tag_strings() {
        assert_eq!(
            tag_value(&["color:red otherTag:otherTagValue".to_string()], "color").as_deref(),
            Some("red")
        );
        assert_eq!(
            tag_value(&["menu".to_string(), "# color:{Blue}".to_string()], "color").as_deref(),
            Some("Blue")
        );
    }

    #[test]
    fn tag_value_accepts_spaced_or_quoted_values() {
        assert_eq!(
            tag_value(
                &["prompt:Choose the next action color:blue".to_string()],
                "prompt"
            )
            .as_deref(),
            Some("Choose the next action")
        );
        assert_eq!(
            tag_value(
                &["title=\"Location: Border Village - Square\" color:blue".to_string()],
                "title"
            )
            .as_deref(),
            Some("Location: Border Village - Square")
        );
    }

    #[test]
    fn tag_marker_reads_bare_tags() {
        assert!(tag_marker(&["prompt".to_string()], "prompt"));
        assert!(tag_marker(&["# Title".to_string()], "title"));
        assert!(!tag_marker(&["prompt:Choose".to_string()], "prompt"));
    }

    #[test]
    fn tag_bool_accepts_common_boolean_values() {
        assert_eq!(
            tag_bool(&["enabled:false".to_string()], "enabled"),
            Some(false)
        );
        assert_eq!(
            tag_bool(&["enabled:{1}".to_string()], "enabled"),
            Some(true)
        );
        assert_eq!(tag_bool(&["enabled:maybe".to_string()], "enabled"), None);
    }
}
