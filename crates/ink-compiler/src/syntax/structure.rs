use crate::parsed::{EnumMember, StructField};

use super::{error, is_identifier, is_identifier_continue, rule::RuleParser, type_name};

pub(super) struct StructHeader {
    pub(super) name: String,
    pub(super) fields: Vec<StructField>,
    pub(super) closed: bool,
}

pub(super) struct EnumHeader {
    pub(super) name: String,
    pub(super) members: Vec<EnumMember>,
    pub(super) closed: bool,
}

pub(super) fn parse_struct_header(parser: &mut RuleParser<'_>) -> Option<StructHeader> {
    parser.skip_horizontal_whitespace();
    parser.match_string("STRUCT")?;
    parser.skip_horizontal_whitespace();

    let name = parser.take_while(is_identifier_continue)?;
    if !is_identifier(&name) {
        parser.error(format!("Expected struct name but saw '{name}'"));
        return None;
    }

    parser.skip_horizontal_whitespace();
    parser.match_string("{")?;
    parser.skip_horizontal_whitespace();

    if parser.line_remainder().is_empty() {
        return Some(StructHeader {
            name,
            fields: Vec::new(),
            closed: false,
        });
    }

    if parser.match_string("}").is_some() {
        parser.skip_horizontal_whitespace();
        if !parser.line_remainder().is_empty() {
            parser.error(error::expected_message(
                "end of line",
                parser.line_remainder(),
            ));
            return None;
        }
        return Some(StructHeader {
            name,
            fields: Vec::new(),
            closed: true,
        });
    }

    let field = parse_struct_field_body(parser)?;
    parser.skip_horizontal_whitespace();
    if parser.match_string("}").is_none() {
        reject_field_separator_or_trailing_text(parser);
        return None;
    }
    parser.skip_horizontal_whitespace();
    if !parser.line_remainder().is_empty() {
        parser.error(error::expected_message(
            "end of line",
            parser.line_remainder(),
        ));
        return None;
    }

    Some(StructHeader {
        name,
        fields: vec![field],
        closed: true,
    })
}

pub(super) fn parse_struct_field(parser: &mut RuleParser<'_>) -> Option<StructField> {
    let field = parse_struct_field_body(parser)?;
    parser.skip_horizontal_whitespace();
    if !parser.line_remainder().is_empty() {
        reject_field_separator_or_trailing_text(parser);
        return None;
    }

    Some(field)
}

pub(super) fn parse_enum_header(parser: &mut RuleParser<'_>) -> Option<EnumHeader> {
    parser.skip_horizontal_whitespace();
    parser.match_string("ENUM")?;
    parser.skip_horizontal_whitespace();

    let name = parser.take_while(is_identifier_continue)?;
    if !is_identifier(&name) {
        parser.error(format!("Expected enum name but saw '{name}'"));
        return None;
    }

    parser.skip_horizontal_whitespace();
    parser.match_string("{")?;
    parser.skip_horizontal_whitespace();

    if parser.line_remainder().is_empty() {
        return Some(EnumHeader {
            name,
            members: Vec::new(),
            closed: false,
        });
    }

    let mut members = Vec::new();
    loop {
        parser.skip_horizontal_whitespace();
        if parser.match_string("}").is_some() {
            parser.skip_horizontal_whitespace();
            if !parser.line_remainder().is_empty() {
                parser.error(error::expected_message(
                    "end of line",
                    parser.line_remainder(),
                ));
                return None;
            }
            return Some(EnumHeader {
                name,
                members,
                closed: true,
            });
        }

        let Some(member) = parse_enum_member_body(parser) else {
            return None;
        };
        members.push(member);
        parser.skip_horizontal_whitespace();

        if parser.line_remainder().starts_with(',') || parser.line_remainder().starts_with(';') {
            reject_enum_member_separator_or_assignment(parser);
            return None;
        }
    }
}

pub(super) fn parse_enum_member(parser: &mut RuleParser<'_>) -> Option<EnumMember> {
    let member = parse_enum_member_body(parser)?;
    parser.skip_horizontal_whitespace();
    if !parser.line_remainder().is_empty() {
        reject_enum_member_separator_or_assignment(parser);
        return None;
    }

    Some(member)
}

fn parse_struct_field_body(parser: &mut RuleParser<'_>) -> Option<StructField> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let name = parser.take_while(is_identifier_continue)?;
    if !is_identifier(&name) {
        parser.error(format!("Expected struct field name but saw '{name}'"));
        return None;
    }

    parser.skip_horizontal_whitespace();
    parser.match_string(":")?;
    let type_name = type_name::parse_type_name(parser)?;

    Some(StructField::new(name, type_name, span))
}

fn parse_enum_member_body(parser: &mut RuleParser<'_>) -> Option<EnumMember> {
    parser.skip_horizontal_whitespace();
    let span = parser.current_span();
    let name = parser.take_while(is_identifier_continue)?;
    if !is_identifier(&name) {
        parser.error(format!("Expected enum member name but saw '{name}'"));
        return None;
    }

    parser.skip_horizontal_whitespace();
    if parser.line_remainder().starts_with('=') {
        parser.error("Enum members do not support explicit values");
        parser.skip_to_end();
        return None;
    }

    Some(EnumMember::new(name, span))
}

fn reject_field_separator_or_trailing_text(parser: &mut RuleParser<'_>) {
    if parser.line_remainder().contains(',') || parser.line_remainder().contains(';') {
        parser.error(
            "Struct fields must be declared one per line without comma or semicolon separators",
        );
    } else {
        parser.error(error::expected_message(
            "end of line",
            parser.line_remainder(),
        ));
    }
    parser.skip_to_end();
}

fn reject_enum_member_separator_or_assignment(parser: &mut RuleParser<'_>) {
    if parser.line_remainder().starts_with('=') {
        parser.error("Enum members do not support explicit values");
    } else if parser.line_remainder().contains(',') || parser.line_remainder().contains(';') {
        parser.error("Enum members must be declared without comma or semicolon separators");
    } else {
        parser.error(error::expected_message(
            "end of line",
            parser.line_remainder(),
        ));
    }
    parser.skip_to_end();
}
