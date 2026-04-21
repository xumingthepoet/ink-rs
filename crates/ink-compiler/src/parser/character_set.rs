use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CharacterSet {
    characters: BTreeSet<char>,
}

impl CharacterSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_range(start: char, end: char) -> Self {
        let mut set = Self::new();
        set.add_range(start, end);
        set
    }

    pub fn from_characters(chars: impl IntoIterator<Item = char>) -> Self {
        let mut set = Self::new();
        set.add_characters(chars);
        set
    }

    pub fn contains(&self, character: &char) -> bool {
        self.characters.contains(character)
    }

    pub fn len(&self) -> usize {
        self.characters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.characters.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &char> {
        self.characters.iter()
    }

    pub fn add(&mut self, character: char) -> &mut Self {
        self.characters.insert(character);
        self
    }

    pub fn add_range(&mut self, start: char, end: char) -> &mut Self {
        for character in (start as u32)..=(end as u32) {
            if let Some(character) = char::from_u32(character) {
                self.add(character);
            }
        }
        self
    }

    pub fn add_characters(&mut self, chars: impl IntoIterator<Item = char>) -> &mut Self {
        for character in chars {
            self.add(character);
        }
        self
    }

    pub fn union_with(&mut self, other: &CharacterSet) -> &mut Self {
        for character in other.iter().copied() {
            self.add(character);
        }
        self
    }
}

impl From<&str> for CharacterSet {
    fn from(value: &str) -> Self {
        Self::from_characters(value.chars())
    }
}

impl From<String> for CharacterSet {
    fn from(value: String) -> Self {
        Self::from_characters(value.chars())
    }
}

impl From<&CharacterSet> for CharacterSet {
    fn from(value: &CharacterSet) -> Self {
        value.clone()
    }
}

impl FromIterator<char> for CharacterSet {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self::from_characters(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::CharacterSet;

    #[test]
    fn string_parser_character_set_adds_ranges_and_characters() {
        let mut set = CharacterSet::new();
        set.add_range('a', 'c')
            .add('z')
            .add_characters("12".chars());

        assert!(set.contains(&'a'));
        assert!(set.contains(&'b'));
        assert!(set.contains(&'c'));
        assert!(set.contains(&'z'));
        assert!(set.contains(&'1'));
        assert!(set.contains(&'2'));
        assert_eq!(set.len(), 6);
    }

    #[test]
    fn string_parser_character_set_can_be_copied_from_strings_and_other_sets() {
        let original = CharacterSet::from("ink");
        let copy = CharacterSet::from(&original);

        assert_eq!(original, copy);
        assert!(copy.contains(&'i'));
    }
}
