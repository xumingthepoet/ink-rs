use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use ink_story_json_format as format;

use crate::object::{Object, RTObject};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DynamicInterfaceMemberKind {
    Knot,
    Function,
}

impl DynamicInterfaceMemberKind {
    fn from_format(kind: format::InterfaceMemberKind) -> Self {
        match kind {
            format::InterfaceMemberKind::Knot => Self::Knot,
            format::InterfaceMemberKind::Function => Self::Function,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DynamicInterfaceDefinition {
    members: BTreeMap<String, DynamicInterfaceMemberKind>,
    implementations: BTreeSet<String>,
}

impl DynamicInterfaceDefinition {
    fn from_format(definition: format::InterfaceDefinition) -> Self {
        Self {
            members: definition
                .members
                .into_iter()
                .map(|(name, kind)| (name, DynamicInterfaceMemberKind::from_format(kind)))
                .collect(),
            implementations: definition.implementations.into_iter().collect(),
        }
    }

    pub(crate) fn member_kind(&self, member: &str) -> Option<DynamicInterfaceMemberKind> {
        self.members.get(member).copied()
    }

    pub(crate) fn implementations(&self) -> &BTreeSet<String> {
        &self.implementations
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DynamicInterfaceRegistry {
    interfaces: BTreeMap<String, DynamicInterfaceDefinition>,
}

impl DynamicInterfaceRegistry {
    pub(crate) fn from_format(interfaces: BTreeMap<String, format::InterfaceDefinition>) -> Self {
        Self {
            interfaces: interfaces
                .into_iter()
                .map(|(name, definition)| {
                    (name, DynamicInterfaceDefinition::from_format(definition))
                })
                .collect(),
        }
    }

    pub(crate) fn interface(&self, name: &str) -> Option<&DynamicInterfaceDefinition> {
        self.interfaces.get(name)
    }
}

pub(crate) struct DynamicInterfaceTarget {
    obj: Object,
    interface: String,
    member: String,
}

impl DynamicInterfaceTarget {
    pub(crate) fn new(interface: impl Into<String>, member: impl Into<String>) -> Self {
        Self {
            obj: Object::new(),
            interface: interface.into(),
            member: member.into(),
        }
    }

    pub(crate) fn interface(&self) -> &str {
        &self.interface
    }

    pub(crate) fn member(&self) -> &str {
        &self.member
    }
}

impl RTObject for DynamicInterfaceTarget {
    fn get_object(&self) -> &Object {
        &self.obj
    }
}

impl fmt::Display for DynamicInterfaceTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}::{}", self.interface, self.member)
    }
}

pub(crate) struct DynamicInterfaceFunctionCall {
    obj: Object,
    interface: String,
    member: String,
    args: usize,
}

impl DynamicInterfaceFunctionCall {
    pub(crate) fn new(
        interface: impl Into<String>,
        member: impl Into<String>,
        args: usize,
    ) -> Self {
        Self {
            obj: Object::new(),
            interface: interface.into(),
            member: member.into(),
            args,
        }
    }

    pub(crate) fn interface(&self) -> &str {
        &self.interface
    }

    pub(crate) fn member(&self) -> &str {
        &self.member
    }

    pub(crate) fn args(&self) -> usize {
        self.args
    }
}

impl RTObject for DynamicInterfaceFunctionCall {
    fn get_object(&self) -> &Object {
        &self.obj
    }
}

impl fmt::Display for DynamicInterfaceFunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}::{}()", self.interface, self.member)
    }
}
