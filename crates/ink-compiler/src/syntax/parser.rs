#[cfg(test)]
use crate::source::SourceInput;
use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{
        Flow, FlowParts, ImportDeclaration, InterfaceDeclaration, InterfaceMemberKind,
        InterfaceMemberSignature, Module, Object, Story,
    },
    source::{SourceFile, SourceLine},
};

use super::rule::RuleParser;
use super::scan;
use super::weave::group_weave_content;
use super::{
    author_warning_statement, choice_statement, declaration, divert_statement, gather, import,
    interface, is_choice_continuation_boundary, knot, leading_whitespace_count, logic, module,
    parse_choice_from_line, structure, text, text_statement, variable,
};

type StatementRuleFn = for<'source> fn(&mut RuleParser<'source>) -> Option<Vec<Object>>;

#[derive(Clone, Copy)]
struct StatementRule {
    name: &'static str,
    parse: StatementRuleFn,
}

// Trial order is semantic. More specific declaration and logic forms must run
// before generic logic, divert, and text parsing so failed trials rewind without
// turning structured syntax into plain content.
const STATEMENT_RULES: &[StatementRule] = &[
    StatementRule {
        name: "global variable declaration",
        parse: variable::declaration_statement,
    },
    StatementRule {
        name: "constant declaration",
        parse: declaration::constant_statement,
    },
    StatementRule {
        name: "external declaration",
        parse: declaration::external_statement,
    },
    StatementRule {
        name: "return statement",
        parse: logic::return_statement,
    },
    StatementRule {
        name: "temporary declaration",
        parse: variable::temp_declaration_statement,
    },
    StatementRule {
        name: "variable assignment",
        parse: variable::assignment_statement,
    },
    StatementRule {
        name: "logic line",
        parse: logic::line_statement,
    },
    StatementRule {
        name: "choice",
        parse: choice_statement,
    },
    StatementRule {
        name: "author warning",
        parse: author_warning_statement,
    },
    StatementRule {
        name: "divert",
        parse: divert_statement,
    },
    StatementRule {
        name: "text",
        parse: text_statement,
    },
];

#[cfg(test)]
pub(crate) fn parse(input: SourceInput) -> StageOutput<Story> {
    let source = SourceFile::from_input(input);
    parse_source(source)
}

pub(crate) fn parse_source(source: SourceFile) -> StageOutput<Story> {
    let mut parser = Parser::new(source);
    let story = parser.parse_story();

    StageOutput {
        artifact: Some(story),
        diagnostics: parser.diagnostics,
    }
}

pub(super) struct Parser {
    source: SourceFile,
    diagnostics: Vec<Diagnostic>,
    allow_global_var_declarations: bool,
}

impl Parser {
    fn new(source: SourceFile) -> Self {
        Self {
            source,
            diagnostics: Vec::new(),
            allow_global_var_declarations: true,
        }
    }

    fn parse_story(&mut self) -> Story {
        let lines = self.source.lines.clone();
        let mut objects = Vec::new();
        let mut flows = Vec::new();
        let mut interfaces = Vec::new();
        let mut modules = Vec::new();
        let mut active_module_index = None;
        let explicit_module_source = lines
            .iter()
            .any(|line| module::is_module_like_declaration_line(&line.text));
        let mut index = 0;

        while index < lines.len() {
            let line = &lines[index];

            if line.text.trim().is_empty() {
                index += 1;
                continue;
            }

            if line.text.trim() == "}" {
                index += 1;
                continue;
            }

            if interface::is_interface_like_declaration_line(&line.text) {
                active_module_index = None;
                if let Some(interface) = self.parse_interface_declaration(&lines, &mut index) {
                    interfaces.push(interface);
                }
                continue;
            }

            if module::is_module_like_declaration_line(&line.text) {
                if let Some(module) = self.parse_module_header(line) {
                    modules.push(module);
                    active_module_index = Some(modules.len() - 1);
                }
                index += 1;
                continue;
            }

            if explicit_module_source && active_module_index.is_none() {
                self.diagnostics.push(Diagnostic::error(
                    line.span.clone(),
                    "Content and module-scoped declarations must appear after an explicit module declaration",
                ));
                index += 1;
                continue;
            }

            if let Some(module_index) = active_module_index {
                if import::is_import_like_declaration_line(&line.text) {
                    if let Some(import) = self.parse_import_declaration(&lines, &mut index) {
                        modules[module_index].push_import(import);
                    }
                    continue;
                }

                if line.text.trim_start().starts_with("STRUCT ") {
                    if let Some(parsed) = self.parse_struct_declaration(&lines, &mut index) {
                        modules[module_index].push_objects(vec![Object::StructDeclaration(parsed)]);
                        continue;
                    }
                }

                if line.text.trim_start().starts_with("ENUM ") {
                    if let Some(parsed) = self.parse_enum_declaration(&lines, &mut index) {
                        modules[module_index].push_objects(vec![Object::EnumDeclaration(parsed)]);
                        continue;
                    }
                }

                if knot::is_knot_declaration_line(&line.text) {
                    if let Some(flow) = self.parse_flow(&lines, &mut index) {
                        modules[module_index].push_flow(flow);
                    } else {
                        index += 1;
                    }
                    continue;
                }

                if knot::is_stitch_declaration_line(&line.text) {
                    self.diagnostics.push(Diagnostic::error(
                        line.span.clone(),
                        "Stitch declarations must appear inside a knot",
                    ));
                    index += 1;
                    continue;
                }

                if is_module_scoped_statement_line(&line.text) {
                    let parsed = if let Some(parsed) =
                        self.parse_multiline_module_scoped_statement(&lines, &mut index)
                    {
                        parsed
                    } else {
                        let parsed = self.parse_statement(line);
                        index += 1;
                        parsed
                    };
                    if parsed.iter().all(is_module_scoped_object) {
                        modules[module_index].push_objects(parsed);
                    }
                    continue;
                }

                self.diagnostics.push(module_level_content_diagnostic(line));
                index += 1;
                continue;
            }

            if line.text.trim_start().starts_with("STRUCT ") {
                if let Some(parsed) = self.parse_struct_declaration(&lines, &mut index) {
                    objects.push(Object::StructDeclaration(parsed));
                    continue;
                }
            }

            if line.text.trim_start().starts_with("ENUM ") {
                if let Some(parsed) = self.parse_enum_declaration(&lines, &mut index) {
                    objects.push(Object::EnumDeclaration(parsed));
                    continue;
                }
            }

            if knot::is_knot_declaration_line(&line.text) {
                if let Some(flow) = self.parse_flow(&lines, &mut index) {
                    flows.push(flow);
                } else {
                    index += 1;
                }
                continue;
            }

            if knot::is_stitch_declaration_line(&line.text) {
                if let Some(flow) = self.parse_stitch(&lines, &mut index) {
                    flows.push(flow);
                    continue;
                }
            }

            if let Some(parsed) = self.parse_multiline_module_scoped_statement(&lines, &mut index) {
                objects.extend(parsed);
                continue;
            }

            if let Some(parsed) = self.parse_compound_statement(&lines, &mut index) {
                objects.extend(parsed);
                continue;
            }

            objects.extend(self.parse_statement(line));
            index += 1;
        }

        Story::new_with_modules_and_interfaces(
            group_weave_content(objects),
            flows,
            modules,
            interfaces,
        )
    }

    pub(super) fn parse_statement(&mut self, line: &SourceLine) -> Vec<Object> {
        if line.text.trim().is_empty() {
            return Vec::new();
        }

        if !self.allow_global_var_declarations
            && is_global_var_declaration_line(line.text.trim_start())
        {
            self.diagnostics
                .push(nested_global_var_declaration_diagnostic(line));
            return Vec::new();
        }

        if let Some(objects) = self.parse_gather_line(line) {
            return objects;
        }

        let mut line_parser = RuleParser::new(line);

        for rule in STATEMENT_RULES {
            if let Some(rule_match) = line_parser.parse_rule_with_metadata(rule.parse) {
                debug_assert!(!rule.name.is_empty());
                debug_assert!(rule_match.metadata.is_forward());
                self.diagnostics.extend(line_parser.finish());
                return rule_match.value;
            }

            if line_parser.had_error() {
                self.diagnostics.extend(line_parser.finish());
                return Vec::new();
            }
        }

        self.diagnostics.extend(line_parser.finish());
        Vec::new()
    }

    fn parse_multiline_module_scoped_statement(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        let line = &lines[*index];
        let initializer = multiline_declaration_initializer(&line.text)?;
        if !scan::has_unclosed_expression_delimiters(initializer) {
            return None;
        }

        let mut combined = line.text.clone();
        let mut combined_initializer = initializer.to_string();
        let mut next_index = *index + 1;
        while next_index < lines.len()
            && scan::has_unclosed_expression_delimiters(&combined_initializer)
        {
            let next_line = &lines[next_index];
            combined.push('\n');
            combined.push_str(&next_line.text);
            combined_initializer.push('\n');
            combined_initializer.push_str(&next_line.text);
            next_index += 1;
        }

        let combined_line = SourceLine {
            text: combined,
            span: line.span.clone(),
        };
        let parsed = self.parse_statement(&combined_line);
        *index = next_index;
        Some(parsed)
    }

    fn parse_gather_line(&mut self, line: &SourceLine) -> Option<Vec<Object>> {
        let mut line_parser = RuleParser::new(line);
        let mut objects = line_parser.parse_rule(gather::parse_statement)?;
        let had_error = line_parser.had_error();
        line_parser.skip_horizontal_whitespace();

        let remaining = line_parser.line_remainder().to_string();
        let remaining_span = line_parser.current_span();
        self.diagnostics.extend(line_parser.finish());

        if had_error {
            return Some(Vec::new());
        }

        if !remaining.trim().is_empty() {
            let remaining_line = SourceLine {
                text: remaining,
                span: remaining_span,
            };
            objects.extend(self.parse_statement(&remaining_line));
        }

        Some(objects)
    }

    fn parse_module_header(&mut self, line: &SourceLine) -> Option<Module> {
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(module::parse_module_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        declaration.filter(|_| !had_error).map(|declaration| {
            Module::new(
                declaration.name,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                declaration.name_span,
                declaration.span,
            )
        })
    }

    fn parse_interface_declaration(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<InterfaceDeclaration> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(interface::parse_interface_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());
        *index += 1;

        let declaration = declaration?;

        let mut members = Vec::new();
        if had_error {
            return Some(InterfaceDeclaration::new_with_members(
                declaration.name,
                members,
                declaration.name_span,
                declaration.span,
            ));
        }

        while *index < lines.len() {
            let next_line = &lines[*index];

            if next_line.text.trim().is_empty() {
                *index += 1;
                continue;
            }

            if module::is_module_like_declaration_line(&next_line.text)
                || interface::is_interface_like_declaration_line(&next_line.text)
            {
                break;
            }

            if knot::is_stitch_declaration_line(&next_line.text) {
                self.diagnostics.push(Diagnostic::error(
                    next_line.span.clone(),
                    "Interface bodies do not support stitch declarations; declare interface members with `== name ==` or `== function name(...) => type ==`",
                ));
                *index += 1;
                continue;
            }

            if knot::is_knot_declaration_line(&next_line.text) {
                if let Some(member) = self.parse_interface_member_signature(next_line) {
                    members.push(member);
                }
                *index += 1;
                continue;
            }

            self.diagnostics
                .push(interface_body_content_diagnostic(next_line));
            *index += 1;
        }

        Some(InterfaceDeclaration::new_with_members(
            declaration.name,
            members,
            declaration.name_span,
            declaration.span,
        ))
    }

    fn parse_interface_member_signature(
        &mut self,
        line: &SourceLine,
    ) -> Option<InterfaceMemberSignature> {
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(knot::parse_knot_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let declaration = declaration?;
        if had_error {
            return None;
        }

        if declaration.is_internal {
            self.diagnostics.push(Diagnostic::error(
                declaration.span,
                "Interface function signatures must use `function`, not `INTERNAL`",
            ));
            return None;
        }

        let kind = if declaration.is_function {
            InterfaceMemberKind::Function
        } else {
            InterfaceMemberKind::Knot
        };
        let has_typed_signature = declaration.is_function
            || declaration
                .arguments
                .iter()
                .any(|argument| argument.declared_type().is_some());
        let return_type = declaration
            .is_function
            .then_some(declaration.return_type.clone());

        Some(InterfaceMemberSignature::new(
            kind,
            declaration.name,
            declaration.arguments,
            return_type,
            has_typed_signature,
            declaration.name_span,
            declaration.span,
        ))
    }

    fn parse_import_declaration(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<ImportDeclaration> {
        let (declaration, diagnostics, next_index) =
            import::parse_import_declaration_lines(lines, *index);
        self.diagnostics.extend(diagnostics);
        *index = next_index;
        declaration
    }

    fn parse_flow(&mut self, lines: &[SourceLine], index: &mut usize) -> Option<Flow> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(knot::parse_knot_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let declaration = declaration?;

        if had_error {
            return None;
        }

        *index += 1;
        let mut content = Vec::new();
        let mut child_flows = Vec::new();
        let previous_global_var_setting = self.allow_global_var_declarations;
        self.allow_global_var_declarations = false;

        while *index < lines.len() {
            let next_line = &lines[*index];
            if next_line.text.trim().is_empty() {
                *index += 1;
                continue;
            }

            if next_line.text.trim() == "}" {
                *index += 1;
                continue;
            }

            if module::is_module_like_declaration_line(&next_line.text)
                || interface::is_interface_like_declaration_line(&next_line.text)
            {
                break;
            }

            if knot::is_knot_declaration_line(&next_line.text) {
                break;
            }

            if knot::is_stitch_declaration_line(&next_line.text) {
                if let Some(stitch) = self.parse_stitch(lines, index) {
                    child_flows.push(stitch);
                } else {
                    *index += 1;
                }
                continue;
            }

            if let Some(parsed) = self.parse_compound_statement(lines, index) {
                content.extend(parsed);
                continue;
            }

            content.extend(self.parse_statement(next_line));
            *index += 1;
        }
        self.allow_global_var_declarations = previous_global_var_setting;

        Some(Flow::from_parts(
            FlowParts::new(
                declaration.level,
                declaration.name,
                group_weave_content(content),
            )
            .child_flows(child_flows)
            .arguments(declaration.arguments)
            .return_type(declaration.return_type)
            .function(declaration.is_function)
            .internal(declaration.is_internal)
            .span(line.span.clone()),
        ))
    }

    fn parse_stitch(&mut self, lines: &[SourceLine], index: &mut usize) -> Option<Flow> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(knot::parse_stitch_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let declaration = declaration?;

        if had_error {
            return None;
        }

        *index += 1;
        let mut content = Vec::new();
        let previous_global_var_setting = self.allow_global_var_declarations;
        self.allow_global_var_declarations = false;

        while *index < lines.len() {
            let next_line = &lines[*index];
            if next_line.text.trim().is_empty() {
                *index += 1;
                continue;
            }

            if next_line.text.trim() == "}" {
                *index += 1;
                continue;
            }

            if module::is_module_like_declaration_line(&next_line.text)
                || interface::is_interface_like_declaration_line(&next_line.text)
            {
                break;
            }

            if knot::is_knot_declaration_line(&next_line.text)
                || knot::is_stitch_declaration_line(&next_line.text)
            {
                break;
            }

            if let Some(parsed) = self.parse_compound_statement(lines, index) {
                content.extend(parsed);
                continue;
            }

            content.extend(self.parse_statement(next_line));
            *index += 1;
        }
        self.allow_global_var_declarations = previous_global_var_setting;

        Some(Flow::from_parts(
            FlowParts::new(
                declaration.level,
                declaration.name,
                group_weave_content(content),
            )
            .arguments(declaration.arguments)
            .return_type(declaration.return_type)
            .function(declaration.is_function)
            .internal(declaration.is_internal)
            .span(line.span.clone()),
        ))
    }

    fn parse_struct_declaration(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<crate::parsed::StructDeclaration> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let header = line_parser.parse_rule(structure::parse_struct_header);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let mut header = header?;

        *index += 1;
        if had_error {
            return Some(crate::parsed::StructDeclaration::new(
                header.name,
                header.fields,
                line.span.clone(),
            ));
        }

        while !header.closed && *index < lines.len() {
            let next_line = &lines[*index];
            let trimmed = next_line.text.trim();

            if trimmed.is_empty() {
                *index += 1;
                continue;
            }

            if trimmed == "}" {
                header.closed = true;
                *index += 1;
                break;
            }

            let mut field_parser = RuleParser::new(next_line);
            let field = field_parser.parse_rule(structure::parse_struct_field);
            let had_error = field_parser.had_error();
            self.diagnostics.extend(field_parser.finish());
            if let Some(field) = field {
                header.fields.push(field);
            }
            *index += 1;

            if had_error {
                continue;
            }
        }

        if !header.closed {
            self.diagnostics.push(Diagnostic::error(
                line.span.clone(),
                "Expected closing '}' for struct declaration",
            ));
        }

        Some(crate::parsed::StructDeclaration::new(
            header.name,
            header.fields,
            line.span.clone(),
        ))
    }

    fn parse_enum_declaration(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<crate::parsed::EnumDeclaration> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let header = line_parser.parse_rule(structure::parse_enum_header);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let mut header = header?;

        *index += 1;
        if had_error {
            return Some(crate::parsed::EnumDeclaration::new(
                header.name,
                header.members,
                line.span.clone(),
            ));
        }

        while !header.closed && *index < lines.len() {
            let next_line = &lines[*index];
            let trimmed = next_line.text.trim();

            if trimmed.is_empty() {
                *index += 1;
                continue;
            }

            if trimmed == "}" {
                header.closed = true;
                *index += 1;
                break;
            }

            let mut member_parser = RuleParser::new(next_line);
            let member = member_parser.parse_rule(structure::parse_enum_member);
            self.diagnostics.extend(member_parser.finish());
            if let Some(member) = member {
                header.members.push(member);
            }
            *index += 1;
        }

        if !header.closed {
            self.diagnostics.push(Diagnostic::error(
                line.span.clone(),
                "Expected closing '}' for enum declaration",
            ));
        }

        Some(crate::parsed::EnumDeclaration::new(
            header.name,
            header.members,
            line.span.clone(),
        ))
    }

    pub(super) fn parse_compound_statement(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        if let Some(parsed) = self.parse_multiline_rule(index, |parser, index| {
            parser.parse_choice_with_continuation(lines, index)
        }) {
            return Some(parsed);
        }

        if let Some(parsed) = self.parse_multiline_rule(index, |parser, index| {
            parser.reject_removed_multiline_sequence(lines, index)
        }) {
            return Some(parsed);
        }

        self.parse_multiline_rule(index, |parser, index| {
            parser.parse_multiline_conditional(lines, index)
        })
    }

    pub(super) fn parse_multiline_rule<T>(
        &mut self,
        index: &mut usize,
        rule: impl FnOnce(&mut Self, &mut usize) -> Option<T>,
    ) -> Option<T> {
        let start_index = *index;
        let diagnostic_count = self.diagnostics.len();

        let result = rule(self, index);
        if result.is_none() {
            *index = start_index;
            self.diagnostics.truncate(diagnostic_count);
        }

        result
    }

    fn parse_choice_with_continuation(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        let line = &lines[*index];
        if !line.text.trim_start().starts_with('*') {
            return None;
        }

        let initial_choice = parse_choice_from_line(line)?;
        if !initial_choice.is_invisible_default() || initial_choice.condition().is_none() {
            return None;
        }

        let current_indent = leading_whitespace_count(&line.text);
        let mut next_index = *index + 1;
        let mut continuation_parts = Vec::new();
        while next_index < lines.len() {
            let next_line = &lines[next_index];
            let trimmed = next_line.text.trim();
            if trimmed.is_empty()
                || trimmed == "}"
                || leading_whitespace_count(&next_line.text) <= current_indent
                || is_choice_continuation_boundary(trimmed)
            {
                break;
            }

            continuation_parts.push(trimmed.to_string());
            next_index += 1;
        }

        if continuation_parts.is_empty() {
            return None;
        }

        let combined = format!("{} {}", line.text.trim_end(), continuation_parts.join(" "));
        let combined_line = SourceLine {
            text: combined,
            span: line.span.clone(),
        };
        let combined_choice = parse_choice_from_line(&combined_line)?;
        if combined_choice.is_invisible_default() {
            return None;
        }

        *index = next_index;
        Some(vec![Object::Choice(combined_choice)])
    }

    fn reject_removed_multiline_sequence(
        &mut self,
        lines: &[SourceLine],
        index: &mut usize,
    ) -> Option<Vec<Object>> {
        let line = &lines[*index];
        let trimmed = line.text.trim();
        let after_open = trimmed.strip_prefix('{')?.trim();
        let rest = text::removed_sequence_type_annotation_rest(after_open)?;
        if !rest.trim().is_empty() {
            return None;
        }

        self.diagnostics.push(
            Diagnostic::error(line.span.clone(), text::REMOVED_SEQUENCE_MESSAGE)
                .with_code(crate::diagnostic::DiagnosticCode::InvalidInlineSyntax),
        );

        *index += 1;
        while *index < lines.len() {
            let current_trimmed = lines[*index].text.trim();
            *index += 1;
            if current_trimmed.starts_with('}') {
                break;
            }
        }

        Some(Vec::new())
    }
}

pub(super) fn is_global_var_declaration_line(trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix("VAR") else {
        return false;
    };
    rest.chars().next().is_some_and(|ch| ch.is_whitespace())
}

pub(super) fn nested_global_var_declaration_diagnostic(line: &SourceLine) -> Diagnostic {
    Diagnostic::error(
        line.span.clone(),
        "Global VAR declarations must appear at the story top level, outside knots, stitches, functions, choices, and conditionals.",
    )
}

fn is_module_scoped_statement_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("CONST ")
        || is_global_var_declaration_line(trimmed)
        || trimmed.starts_with("EXTERNAL ")
}

fn multiline_declaration_initializer(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("CONST ") && !is_global_var_declaration_line(trimmed) {
        return None;
    }

    let initializer_index = line.find('=')?;
    Some(&line[initializer_index + '='.len_utf8()..])
}

fn is_module_scoped_object(object: &Object) -> bool {
    match object {
        Object::ConstantDeclaration(_) | Object::ExternalDeclaration(_) => true,
        Object::VariableAssignment(assignment) => assignment.is_global(),
        _ => false,
    }
}

fn module_level_content_diagnostic(line: &SourceLine) -> Diagnostic {
    if line.text.trim_start().starts_with('#') {
        return Diagnostic::error(
            line.span.clone(),
            "Module-level tags are not allowed; tags must be inside knots or stitches",
        );
    }

    Diagnostic::error(
        line.span.clone(),
        "Module-level story content is not allowed; put story content inside a knot or stitch",
    )
}

fn interface_body_content_diagnostic(line: &SourceLine) -> Diagnostic {
    let trimmed = line.text.trim_start();
    let message = if trimmed.starts_with('#') {
        "Interface bodies do not support tags; declare only knot and function signatures"
    } else if trimmed.starts_with('*') || trimmed.starts_with('+') {
        "Interface bodies do not support choices; declare only knot and function signatures"
    } else if trimmed.starts_with("->") {
        "Interface bodies do not support diverts; declare only knot and function signatures"
    } else if trimmed.starts_with('-') {
        "Interface bodies do not support gathers; declare only knot and function signatures"
    } else if trimmed.starts_with("VAR ") {
        "Interface bodies do not support variable declarations; declare only knot and function signatures"
    } else if trimmed.starts_with("CONST ") {
        "Interface bodies do not support constants; declare only knot and function signatures"
    } else if trimmed.starts_with("STRUCT ") {
        "Interface bodies do not support structs; declare only knot and function signatures"
    } else if trimmed.starts_with("ENUM ") {
        "Interface bodies do not support enums; declare only knot and function signatures"
    } else if trimmed.starts_with("EXTERNAL ") {
        "Interface bodies do not support external declarations; declare only knot and function signatures"
    } else {
        "Interface bodies only support knot and function signatures"
    };

    Diagnostic::error(line.span.clone(), message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_multiline_compound_rule_rewinds_index() {
        let source = SourceFile::from_input(SourceInput::new("{ once\n- A"));
        let lines = source.lines.clone();
        let mut parser = Parser::new(source);
        let mut index = 0;

        assert!(parser
            .parse_compound_statement(&lines, &mut index)
            .is_none());

        assert_eq!(index, 0);
        assert!(parser.diagnostics.is_empty(), "{:#?}", parser.diagnostics);
    }

    #[test]
    fn statement_rule_order_keeps_specific_rules_before_fallbacks() {
        let names = STATEMENT_RULES
            .iter()
            .map(|rule| rule.name)
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                "global variable declaration",
                "constant declaration",
                "external declaration",
                "return statement",
                "temporary declaration",
                "variable assignment",
                "logic line",
                "choice",
                "author warning",
                "divert",
                "text",
            ]
        );
    }

    #[test]
    fn parses_struct_declaration_and_snapshot() {
        let output = parse(SourceInput::new(
            "STRUCT Player {\n\
             hp: int\n\
             name: string\n\
             inventory: Item[]\n\
             }",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        let Object::StructDeclaration(declaration) = &story.root_weave().content()[0] else {
            panic!("expected struct declaration");
        };
        assert_eq!(declaration.name(), "Player");
        assert_eq!(declaration.fields().len(), 3);
        assert_eq!(declaration.fields()[0].name(), "hp");
        assert_eq!(declaration.fields()[0].type_name().snapshot_name(), "int");
        assert_eq!(
            declaration.fields()[2].type_name().snapshot_name(),
            "Item[]"
        );
        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n    StructDeclaration(name=\"Player\")\n      Field(name=\"hp\", type=int)\n      Field(name=\"name\", type=string)\n      Field(name=\"inventory\", type=Item[])\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)"
        );
    }

    #[test]
    fn parses_inline_enum_declaration_and_snapshot() {
        let output = parse(SourceInput::new("ENUM State { Idle Busy Done }"));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        let Object::EnumDeclaration(declaration) = &story.root_weave().content()[0] else {
            panic!("expected enum declaration");
        };
        assert_eq!(declaration.name(), "State");
        assert_eq!(
            declaration
                .members()
                .iter()
                .map(|member| member.name())
                .collect::<Vec<_>>(),
            vec!["Idle", "Busy", "Done"]
        );
        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n    EnumDeclaration(name=\"State\")\n      Member(name=\"Idle\")\n      Member(name=\"Busy\")\n      Member(name=\"Done\")\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)"
        );
    }

    #[test]
    fn parses_multiline_module_enum_declaration() {
        let output = parse(SourceInput::new(
            "=== module items ===\n\
             ENUM State {\n\
             Idle\n\
             Busy\n\
             }\n\
             == main ==\n\
             -> END",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        let Object::EnumDeclaration(declaration) = &story.modules()[0].weave().content()[0] else {
            panic!("expected enum declaration");
        };
        assert_eq!(declaration.name(), "State");
        assert_eq!(declaration.members().len(), 2);
        assert_eq!(declaration.members()[0].span().line, 3);
        assert_eq!(story.modules()[0].flows()[0].name(), "main");
    }

    #[test]
    fn rejects_enum_member_values_commas_and_semicolons() {
        let cases = [
            (
                "ENUM State { Idle = \"idle\" }",
                "Enum members do not support explicit values",
            ),
            (
                "ENUM State { Idle, Busy }",
                "Enum members must be declared without comma or semicolon separators",
            ),
            (
                "ENUM State {\nIdle;\n}",
                "Enum members must be declared without comma or semicolon separators",
            ),
        ];

        for (source, expected_message) in cases {
            let output = parse(SourceInput::new(source));

            assert_eq!(output.diagnostics.len(), 1, "{:#?}", output.diagnostics);
            assert_eq!(output.diagnostics[0].message, expected_message);
        }
    }

    #[test]
    fn parses_module_headers() {
        let output = parse(SourceInput::new("=== module game ==="));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert_eq!(story.modules().len(), 1);
        assert_eq!(story.modules()[0].name(), "game");
        assert_eq!(story.modules()[0].name_span().line, 1);
        assert_eq!(story.modules()[0].name_span().column, 12);
        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)\n  Module(name=\"game\")"
        );
    }

    #[test]
    fn parses_interface_headers() {
        let output = parse(SourceInput::new("=== interface IItem ==="));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert_eq!(story.interfaces().len(), 1);
        assert_eq!(story.interfaces()[0].name(), "IItem");
        assert_eq!(story.interfaces()[0].name_span().line, 1);
        assert_eq!(story.interfaces()[0].name_span().column, 15);
        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)\n  Interface(name=\"IItem\")"
        );
    }

    #[test]
    fn parses_interface_member_signatures() {
        let output = parse(SourceInput::new(
            "=== interface IItem ===\n\
             == target(amount: int) ==\n\
             == function score(amount: int) => int ==\n\
             === module game ===",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        let members = story.interfaces()[0].members();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].kind(), &crate::parsed::InterfaceMemberKind::Knot);
        assert_eq!(members[0].name(), "target");
        assert_eq!(
            members[0].arguments()[0].declared_type(),
            Some(&crate::parsed::TypeName::int())
        );
        assert_eq!(
            members[1].kind(),
            &crate::parsed::InterfaceMemberKind::Function
        );
        assert_eq!(members[1].name(), "score");
        assert_eq!(
            members[1].return_type(),
            Some(&crate::parsed::TypeName::int())
        );
        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)\n  Interface(name=\"IItem\")\n    InterfaceMember(kind=Knot, name=\"target\", typed=true)\n      Argument(name=\"amount\", type=int)\n    InterfaceMember(kind=Function, name=\"score\", typed=true, return=int)\n      Argument(name=\"amount\", type=int)\n  Module(name=\"game\")"
        );
    }

    #[test]
    fn rejects_executable_interface_body_content() {
        let output = parse(SourceInput::new(
            "=== interface IItem ===\n\
             VAR bad: int = 1\n\
             CONST BAD: int = 1\n\
             EXTERNAL ext(x: int) => int\n\
             STRUCT Bad { value: int }\n\
             ENUM State { Idle }\n\
             # tag\n\
             * choice\n\
             - gather\n\
             -> DONE\n\
             = stitch\n\
             Text.\n\
             === module game ===",
        ));

        let messages = output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>();
        for expected in [
            "Interface bodies do not support variable declarations",
            "Interface bodies do not support constants",
            "Interface bodies do not support external declarations",
            "Interface bodies do not support structs",
            "Interface bodies do not support enums",
            "Interface bodies do not support tags",
            "Interface bodies do not support choices",
            "Interface bodies do not support gathers",
            "Interface bodies do not support diverts",
            "Interface bodies do not support stitch declarations",
            "Interface bodies only support knot and function signatures",
        ] {
            assert!(
                messages.iter().any(|message| message.contains(expected)),
                "missing diagnostic containing {expected:?}: {:#?}",
                output.diagnostics
            );
        }
    }

    #[test]
    fn parses_interfaces_as_top_level_peers_of_modules() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             == main ==\n\
             -> DONE\n\
             === interface IItem ===\n\
             === module items ===",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert_eq!(story.interfaces().len(), 1);
        assert_eq!(story.interfaces()[0].name(), "IItem");
        assert_eq!(story.modules().len(), 2);
        assert_eq!(story.modules()[0].flows()[0].name(), "main");
        assert_eq!(story.modules()[1].name(), "items");
    }

    #[test]
    fn parses_multiple_module_headers() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             === module items ===",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert_eq!(story.modules().len(), 2);
        assert_eq!(story.modules()[0].name(), "game");
        assert_eq!(story.modules()[1].name(), "items");
    }

    #[test]
    fn parses_module_import_declarations() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             IMPORT sword, heal FROM items",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert_eq!(story.modules().len(), 1);
        let imports = story.modules()[0].imports();
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source_module(), "items");
        assert_eq!(imports[0].imported_names()[0].name(), "sword");
        assert_eq!(imports[0].imported_names()[1].name(), "heal");
        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)\n  Module(name=\"game\")\n    Import(from=\"items\", names=[\"sword\", \"heal\"])"
        );
    }

    #[test]
    fn parses_multiline_module_import_declarations() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             IMPORT {\n\
                 sword, heal, shield,\n\
                 mend,\n\
             } FROM items",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert_eq!(story.modules().len(), 1);
        let imports = story.modules()[0].imports();
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source_module(), "items");
        assert_eq!(
            imports[0]
                .imported_names()
                .iter()
                .map(|name| name.name())
                .collect::<Vec<_>>(),
            vec!["sword", "heal", "shield", "mend"]
        );
        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)\n  Module(name=\"game\")\n    Import(from=\"items\", names=[\"sword\", \"heal\", \"shield\", \"mend\"])"
        );
    }

    #[test]
    fn attaches_imports_to_active_module() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             IMPORT start FROM flow\n\
             === module items ===\n\
             IMPORT sword FROM gear",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert_eq!(story.modules().len(), 2);
        assert_eq!(story.modules()[0].imports()[0].source_module(), "flow");
        assert_eq!(story.modules()[1].imports()[0].source_module(), "gear");
    }

    #[test]
    fn rejects_invalid_module_headers_without_falling_through_to_text() {
        let cases = [
            (
                "== module game ==",
                "Module declarations must use `=== module name ===`",
            ),
            (
                "= module game",
                "Module declarations must use `=== module name ===`",
            ),
            (
                "=== Module game ===",
                "Module declarations must use lowercase `module`",
            ),
            (
                "=== module game(seed) ===",
                "Module declarations do not accept parameters",
            ),
            (
                "=== module game.seed ===",
                "Module names must be single identifiers; hierarchical module names are not supported",
            ),
            (
                "=== module 123 ===",
                "Module name must be a single identifier",
            ),
        ];

        for (source, expected_message) in cases {
            let output = parse(SourceInput::new(source));

            assert_eq!(
                output.diagnostics.len(),
                1,
                "{source}: {:#?}",
                output.diagnostics
            );
            assert_eq!(output.diagnostics[0].message, expected_message);
            let story = output.artifact.expect("story should still be returned");
            assert!(story.modules().is_empty(), "{source}");
            assert!(
                story.root_weave().content().is_empty(),
                "invalid module header should not be parsed as text: {source}"
            );
            assert!(story.flows().is_empty(), "{source}");
        }
    }

    #[test]
    fn rejects_invalid_interface_headers_without_falling_through_to_text() {
        let cases = [
            (
                "== interface IItem ==",
                "Interface declarations must use `=== interface name ===`",
            ),
            (
                "= interface IItem",
                "Interface declarations must use `=== interface name ===`",
            ),
            (
                "=== Interface IItem ===",
                "Interface declarations must use lowercase `interface`",
            ),
            (
                "=== interface IItem(seed) ===",
                "Interface declarations do not accept parameters",
            ),
            (
                "=== interface game.IItem ===",
                "Interface names must be single identifiers; hierarchical interface names are not supported",
            ),
            (
                "=== interface 123 ===",
                "Interface name must be a single identifier",
            ),
        ];

        for (source, expected_message) in cases {
            let output = parse(SourceInput::new(source));

            assert_eq!(
                output.diagnostics.len(),
                1,
                "{source}: {:#?}",
                output.diagnostics
            );
            assert_eq!(output.diagnostics[0].message, expected_message);
            let story = output.artifact.expect("story should still be returned");
            assert!(story.interfaces().is_empty(), "{source}");
            assert!(
                story.root_weave().content().is_empty(),
                "invalid interface header should not be parsed as text: {source}"
            );
        }
    }

    #[test]
    fn rejects_invalid_module_import_declarations() {
        let cases = [
            (
                "=== module game ===\nimport sword FROM items",
                "Import declarations must use uppercase `IMPORT`",
            ),
            (
                "=== module game ===\nIMPORT sword from items",
                "Import declarations must use uppercase `FROM`",
            ),
            (
                "=== module game ===\nIMPORT FROM items",
                "IMPORT declarations must name at least one symbol before FROM",
            ),
            (
                "=== module game ===\nIMPORT sword AS blade FROM items",
                "Import aliases are not supported in the first module-support phase",
            ),
            (
                "=== module game ===\nIMPORT function play FROM audio",
                "IMPORT names do not include kind annotations",
            ),
            (
                "=== module game ===\nIMPORT sword,",
                "IMPORT declarations must include an imported symbol name after ','",
            ),
            (
                "=== module game ===\nIMPORT {\n}",
                "IMPORT declarations must include FROM moduleName",
            ),
            (
                "=== module game ===\nIMPORT {\n  sword\n} from items",
                "Import declarations must use uppercase `FROM`",
            ),
            (
                "=== module game ===\nIMPORT {\n  sword blade\n} FROM items",
                "Expected ',' or end of line after imported name but saw 'blade'",
            ),
        ];

        for (source, expected_message) in cases {
            let output = parse(SourceInput::new(source));

            assert_eq!(
                output.diagnostics.len(),
                1,
                "{source}: {:#?}",
                output.diagnostics
            );
            assert_eq!(output.diagnostics[0].message, expected_message);
            let story = output.artifact.expect("story should still be returned");
            assert_eq!(story.modules().len(), 1);
            assert!(story.modules()[0].imports().is_empty());
        }
    }

    #[test]
    fn explicit_modules_own_top_level_declarations_and_flows() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             IMPORT sword FROM items\n\
             CONST START: int = 1\n\
             VAR score: int = 0\n\
             STRUCT Player { hp: int }\n\
             EXTERNAL play(name: string) => void\n\
             == function setup() => void ==\n\
             ~ return\n\
             == main ==\n\
             = intro\n\
             -> END",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert!(story.root_weave().content().is_empty());
        assert!(story.flows().is_empty());
        let module = &story.modules()[0];
        assert_eq!(module.imports().len(), 1);
        assert_eq!(module.weave().content().len(), 4);
        assert!(matches!(
            &module.weave().content()[0],
            Object::ConstantDeclaration(_)
        ));
        assert!(matches!(
            &module.weave().content()[1],
            Object::VariableAssignment(assignment) if assignment.is_global()
        ));
        assert!(matches!(
            &module.weave().content()[2],
            Object::StructDeclaration(_)
        ));
        assert!(matches!(
            &module.weave().content()[3],
            Object::ExternalDeclaration(_)
        ));
        assert_eq!(module.flows().len(), 2);
        assert_eq!(module.flows()[0].name(), "setup");
        assert!(module.flows()[0].is_function());
        assert_eq!(module.flows()[1].name(), "main");
        assert_eq!(module.flows()[1].child_flows().len(), 1);
        assert_eq!(module.flows()[1].child_flows()[0].name(), "intro");
    }

    #[test]
    fn explicit_module_parse_snapshot_shows_owned_declarations() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             CONST START: int = 1\n\
             == main ==\n\
             -> END",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert_eq!(
            story.to_parse_snapshot(),
            "Story\n  Weave(baseIndent=0)\n    Gather(name=null, depth=1)\n    Divert(target=\"-> DONE\", empty=false, tunnel=false, thread=false)\n  Module(name=\"game\")\n    Weave(baseIndent=0)\n      ConstantDeclaration(name=\"START\", type=int)\n        Number(1)\n    Flow(level=Knot, name=\"main\", function=false)\n      Weave(baseIndent=0)\n        Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)"
        );
    }

    #[test]
    fn explicit_module_headers_stop_previous_flow() {
        let output = parse(SourceInput::new(
            "=== module first ===\n\
             == main ==\n\
             First.\n\
             === module second ===\n\
             == main ==\n\
             Second.",
        ));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        assert_eq!(story.modules().len(), 2);
        assert_eq!(story.modules()[0].flows().len(), 1);
        assert_eq!(story.modules()[0].flows()[0].name(), "main");
        assert_eq!(story.modules()[1].flows().len(), 1);
        assert_eq!(story.modules()[1].flows()[0].name(), "main");
    }

    #[test]
    fn explicit_module_sources_reject_content_before_first_module() {
        let output = parse(SourceInput::new("Line.\n=== module game ==="));

        assert_eq!(output.diagnostics.len(), 1, "{:#?}", output.diagnostics);
        assert_eq!(
            output.diagnostics[0].message,
            "Content and module-scoped declarations must appear after an explicit module declaration"
        );
        let story = output.artifact.expect("story should still be returned");
        assert!(story.root_weave().content().is_empty());
        assert_eq!(story.modules().len(), 1);
    }

    #[test]
    fn explicit_modules_reject_direct_content_tags_and_top_level_stitches() {
        let output = parse(SourceInput::new(
            "=== module game ===\n\
             Line.\n\
             # module tag\n\
             = stitch",
        ));

        assert_eq!(output.diagnostics.len(), 3, "{:#?}", output.diagnostics);
        assert_eq!(
            output.diagnostics[0].message,
            "Module-level story content is not allowed; put story content inside a knot or stitch"
        );
        assert_eq!(
            output.diagnostics[1].message,
            "Module-level tags are not allowed; tags must be inside knots or stitches"
        );
        assert_eq!(
            output.diagnostics[2].message,
            "Stitch declarations must appear inside a knot"
        );
        let story = output.artifact.expect("story should still be returned");
        assert!(story.modules()[0].weave().content().is_empty());
        assert!(story.modules()[0].flows().is_empty());
    }

    #[test]
    fn parses_single_line_struct_declaration() {
        let output = parse(SourceInput::new("STRUCT Point { x: int }"));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("story should parse");
        let Object::StructDeclaration(declaration) = &story.root_weave().content()[0] else {
            panic!("expected struct declaration");
        };
        assert_eq!(declaration.name(), "Point");
        assert_eq!(declaration.fields().len(), 1);
        assert_eq!(declaration.fields()[0].name(), "x");
    }

    #[test]
    fn rejects_struct_comma_and_semicolon_field_separators() {
        let cases = [
            "STRUCT Player {\n  hp: int, name: string\n}",
            "STRUCT Player {\n  hp: int;\n}",
        ];

        for source in cases {
            let output = parse(SourceInput::new(source));

            assert_eq!(output.diagnostics.len(), 1, "{:#?}", output.diagnostics);
            assert_eq!(
                output.diagnostics[0].message,
                "Struct fields must be declared one per line without comma or semicolon separators"
            );
        }
    }

    #[test]
    fn uppercase_prose_with_punctuation_parses_as_text() {
        let output = parse(SourceInput::new("INVENTORY items = ()"));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("expected story");
        assert!(matches!(
            &story.root_weave().content()[0],
            Object::Text(text) if text.text() == "INVENTORY items = ()"
        ));
    }

    #[test]
    fn global_var_declarations_inside_flows_report_current_syntax_error() {
        let cases = [
            "== knot ==\nVAR score: int = 0\n-> DONE",
            "== knot ==\n= stitch\nVAR score: int = 0\n-> DONE",
            "== function setup() => void ==\nVAR score: int = 0",
        ];

        for source in cases {
            let output = parse(SourceInput::new(source));

            assert_eq!(output.diagnostics.len(), 1, "{:#?}", output.diagnostics);
            assert_eq!(
                output.diagnostics[0].severity,
                crate::diagnostic::DiagnosticSeverity::Error
            );
            assert_eq!(
                output.diagnostics[0].message,
                "Global VAR declarations must appear at the story top level, outside knots, stitches, functions, choices, and conditionals."
            );
        }
    }

    #[test]
    fn invalid_logic_expression_reports_specific_error_and_recovers_next_line() {
        let output = parse(SourceInput::new("~ x +\nRecovered line."));

        assert_eq!(output.diagnostics.len(), 1, "{:#?}", output.diagnostics);
        assert_eq!(
            output.diagnostics[0].code,
            Some(crate::diagnostic::DiagnosticCode::InvalidExpression)
        );
        assert_eq!(output.diagnostics[0].line, 1);
        assert_eq!(output.diagnostics[0].column, 6);
        assert_eq!(
            output.diagnostics[0].message,
            "expected expression after operator `+` before end of input"
        );
    }

    #[test]
    fn square_brackets_in_choices_parse_as_literal_text() {
        let output = parse(SourceInput::new("* Hello [choice text"));

        assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
        let story = output.artifact.expect("expected story");
        let Object::Choice(choice) = &story.root_weave().content()[0] else {
            panic!("expected choice");
        };
        let start_content = choice.start_content().expect("expected choice text");
        assert!(matches!(
            &start_content.objects()[0],
            Object::Text(text) if text.text() == "Hello [choice text"
        ));
    }

    #[test]
    fn invalid_inline_brace_reports_specific_error_span() {
        let output = parse(SourceInput::new("Line {x + 1"));

        assert_eq!(output.diagnostics.len(), 1, "{:#?}", output.diagnostics);
        assert_eq!(
            output.diagnostics[0].code,
            Some(crate::diagnostic::DiagnosticCode::InvalidInlineSyntax)
        );
        assert_eq!(output.diagnostics[0].line, 1);
        assert_eq!(output.diagnostics[0].column, 6);
        assert_eq!(
            output.diagnostics[0].message,
            "expected closing `}` for inline expression before end of line"
        );
    }
}
