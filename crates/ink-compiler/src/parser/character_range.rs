use std::sync::OnceLock;

use super::character_set::CharacterSet;

#[derive(Debug)]
pub struct CharacterRange {
    start: char,
    end: char,
    excludes: CharacterSet,
    corresponding_char_set: OnceLock<CharacterSet>,
}

impl CharacterRange {
    pub fn define(start: char, end: char, excludes: Option<CharacterSet>) -> Self {
        Self {
            start,
            end,
            excludes: excludes.unwrap_or_default(),
            corresponding_char_set: OnceLock::new(),
        }
    }

    pub fn start(&self) -> char {
        self.start
    }

    pub fn end(&self) -> char {
        self.end
    }

    pub fn to_character_set(&self) -> &CharacterSet {
        self.corresponding_char_set.get_or_init(|| {
            let mut character_set = CharacterSet::new();

            for character in (self.start as u32)..=(self.end as u32) {
                if let Some(character) = char::from_u32(character) {
                    if !self.excludes.contains(&character) {
                        character_set.add(character);
                    }
                }
            }

            character_set
        })
    }
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use super::{CharacterRange, CharacterSet};

    #[test]
    fn string_parser_character_range_excludes_chars_and_caches_the_set() {
        let range = CharacterRange::define('a', 'd', Some(CharacterSet::from("bc")));

        let first = range.to_character_set();
        let second = range.to_character_set();

        assert!(first.contains(&'a'));
        assert!(first.contains(&'d'));
        assert!(!first.contains(&'b'));
        assert!(!first.contains(&'c'));
        assert!(ptr::eq(first, second));
    }

    #[test]
    fn string_parser_character_range_exposes_endpoints() {
        let range = CharacterRange::define('x', 'z', None);

        assert_eq!(range.start(), 'x');
        assert_eq!(range.end(), 'z');
    }
}
