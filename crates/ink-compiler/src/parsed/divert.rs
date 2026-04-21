use std::fmt;

use super::{Object, ObjectKind, ObjectRef, Path};

#[derive(Debug, Clone)]
pub struct Divert {
    object: ObjectRef,
}

impl Divert {
    pub fn new(target: Option<Path>) -> Self {
        let object = Object::new_ref();
        object
            .borrow_mut()
            .set_divert_kind(target, false, false, false);

        Self { object }
    }

    pub(crate) fn from_object(object: ObjectRef) -> Self {
        Self { object }
    }

    pub fn empty() -> Self {
        let object = Object::new_ref();
        object
            .borrow_mut()
            .set_divert_kind(None, true, false, false);

        Self { object }
    }

    pub fn tunnel(target: Option<Path>) -> Self {
        let object = Object::new_ref();
        object
            .borrow_mut()
            .set_divert_kind(target, false, true, false);

        Self { object }
    }

    pub fn object(&self) -> ObjectRef {
        self.object.clone()
    }

    pub fn target(&self) -> Option<Path> {
        match self.object.borrow().kind() {
            ObjectKind::Divert { target, .. } => target.clone(),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::Divert { is_empty: true, .. }
        )
    }

    pub fn is_tunnel(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::Divert {
                is_tunnel: true,
                ..
            }
        )
    }

    pub fn is_thread(&self) -> bool {
        matches!(
            self.object.borrow().kind(),
            ObjectKind::Divert {
                is_thread: true,
                ..
            }
        )
    }

    pub fn is_end(&self) -> bool {
        self.target()
            .and_then(|path| path.dot_separated_components())
            .as_deref()
            == Some("END")
    }

    pub fn is_done(&self) -> bool {
        self.target()
            .and_then(|path| path.dot_separated_components())
            .as_deref()
            == Some("DONE")
    }
}

impl fmt::Display for Divert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.object.borrow().kind() {
            ObjectKind::Divert {
                target,
                is_empty,
                is_tunnel,
                ..
            } => {
                if *is_empty || target.is_none() {
                    if *is_tunnel {
                        f.write_str("->->")
                    } else {
                        f.write_str("->")
                    }
                } else if *is_tunnel {
                    match target
                        .as_ref()
                        .and_then(|path| path.dot_separated_components())
                    {
                        Some(components) => write!(f, "->-> {components}"),
                        None => f.write_str("->->"),
                    }
                } else {
                    write!(
                        f,
                        "{}",
                        target
                            .as_ref()
                            .expect("target present for non-empty divert")
                    )
                }
            }
            _ => f.write_str("->"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Divert;
    use crate::parsed::{Identifier, Path};

    #[test]
    fn divert_wraps_target_paths_and_flags() {
        let divert = Divert::new(Some(Path::new(vec![Identifier::new("knot")])));

        assert_eq!(divert.target().unwrap().first_component(), Some("knot"));
        assert!(!divert.is_empty());
        assert!(!divert.is_tunnel());
        assert!(!divert.is_thread());
        assert!(!divert.is_end());
        assert!(!divert.is_done());
        assert_eq!(divert.to_string(), "-> knot");
    }

    #[test]
    fn empty_divert_marks_empty_flag() {
        let divert = Divert::empty();

        assert!(divert.is_empty());
        assert_eq!(divert.to_string(), "->");
    }

    #[test]
    fn tunnel_divert_marks_tunnel_flag() {
        let divert = Divert::tunnel(Some(Path::new(vec![Identifier::new("ending")])));

        assert!(divert.is_tunnel());
        assert!(!divert.is_empty());
        assert_eq!(divert.to_string(), "->-> ending");
    }
}
