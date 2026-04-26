#[cfg(test)]
use crate::source::SourceInput;
use crate::{
    compiler::StageOutput,
    diagnostic::Diagnostic,
    parsed::{Flow, Object, Story},
    source::{SourceFile, SourceLine},
};

use super::rule::RuleParser;
use super::weave::group_weave_content;
use super::{
    author_warning_statement, choice_statement, declaration, divert_statement, gather,
    is_choice_continuation_boundary, knot, leading_whitespace_count, logic, parse_choice_from_line,
    structure, text_statement, variable,
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

            if line.text.trim_start().starts_with("STRUCT ") {
                if let Some(parsed) = self.parse_struct_declaration(&lines, &mut index) {
                    objects.push(Object::StructDeclaration(parsed));
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

            if let Some(parsed) = self.parse_compound_statement(&lines, &mut index) {
                objects.extend(parsed);
                continue;
            }

            objects.extend(self.parse_statement(line));
            index += 1;
        }

        Story::new(group_weave_content(objects), flows)
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

        let diagnostics = line_parser.finish();
        self.diagnostics.extend(diagnostics);

        if let Some(diagnostic) = self.try_unsupported_statement(line) {
            self.diagnostics.push(diagnostic);
        }
        Vec::new()
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

    fn try_unsupported_statement(&self, line: &SourceLine) -> Option<Diagnostic> {
        let trimmed = line.text.trim_start();

        if trimmed.starts_with("LIST ") {
            return Some(Diagnostic::removed_feature(
                line.span.clone(),
                "LIST declarations",
                "Use variables, functions, or host data instead.",
            ));
        }

        let feature = if trimmed.starts_with("INCLUDE ") {
            Some("include")
        } else if trimmed.starts_with("VAR ") {
            Some("global variable declaration")
        } else if trimmed.starts_with("CONST ") {
            Some("constant declaration")
        } else if trimmed.starts_with("EXTERNAL ") {
            Some("external declaration")
        } else if trimmed.starts_with("STRUCT ") {
            Some("struct declaration")
        } else if knot::is_knot_declaration_line(trimmed) {
            Some("knot declaration")
        } else if trimmed.starts_with('*') || trimmed.starts_with('+') {
            Some("choice")
        } else if trimmed.starts_with('~') {
            Some("logic line")
        } else if trimmed.contains('{') || trimmed.contains('}') {
            Some("inline logic")
        } else if trimmed.contains("<>") {
            Some("glue")
        } else if trimmed.contains('#') {
            Some("tag")
        } else {
            None
        };

        feature.map(|feature| Diagnostic::unsupported(line.span.clone(), feature))
    }

    fn parse_flow(&mut self, lines: &[SourceLine], index: &mut usize) -> Option<Flow> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(knot::parse_knot_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let Some(declaration) = declaration else {
            return None;
        };

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

        Some(Flow::new(
            declaration.level,
            declaration.name,
            group_weave_content(content),
            child_flows,
            declaration.arguments,
            declaration.return_type,
            declaration.is_function,
        ))
    }

    fn parse_stitch(&mut self, lines: &[SourceLine], index: &mut usize) -> Option<Flow> {
        let line = &lines[*index];
        let mut line_parser = RuleParser::new(line);
        let declaration = line_parser.parse_rule(knot::parse_stitch_declaration);
        let had_error = line_parser.had_error();
        self.diagnostics.extend(line_parser.finish());

        let Some(declaration) = declaration else {
            return None;
        };

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

        Some(Flow::new(
            declaration.level,
            declaration.name,
            group_weave_content(content),
            Vec::new(),
            declaration.arguments,
            declaration.return_type,
            declaration.is_function,
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

        let Some(mut header) = header else {
            return None;
        };

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
            parser.parse_multiline_sequence(lines, index)
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
        if !line
            .text
            .trim_start()
            .starts_with(|ch| matches!(ch, '*' | '+'))
        {
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
}

pub(super) fn is_global_var_declaration_line(trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix("VAR") else {
        return false;
    };
    rest.chars().next().is_some_and(|ch| ch.is_whitespace())
}

pub(super) fn nested_global_var_declaration_diagnostic(line: &SourceLine) -> Diagnostic {
    Diagnostic::removed_feature(
        line.span.clone(),
        "nested VAR declarations",
        "Move global VAR declarations to the story top level, outside knots, stitches, functions, choices, conditionals, and sequences.",
    )
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
    fn unsupported_syntax_diagnostics_stay_parser_owned() {
        let output = parse(SourceInput::new("INCLUDE file.ink"));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].severity,
            crate::diagnostic::DiagnosticSeverity::Error
        );
        assert_eq!(
            output.diagnostics[0].code,
            Some(crate::diagnostic::DiagnosticCode::UnsupportedSyntax)
        );
        assert_eq!(output.diagnostics[0].message, "unsupported syntax: include");
    }

    #[test]
    fn removed_feature_diagnostics_stay_parser_owned() {
        let output = parse(SourceInput::new("LIST items = ()"));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].severity,
            crate::diagnostic::DiagnosticSeverity::Error
        );
        assert_eq!(
            output.diagnostics[0].code,
            Some(crate::diagnostic::DiagnosticCode::RemovedFeature)
        );
        assert_eq!(output.diagnostics[0].line, 1);
        assert_eq!(output.diagnostics[0].column, 1);
        assert_eq!(
            output.diagnostics[0].message,
            "removed feature: LIST declarations. Use variables, functions, or host data instead."
        );
    }

    #[test]
    fn global_var_declarations_inside_flows_report_removed_feature() {
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
                output.diagnostics[0].code,
                Some(crate::diagnostic::DiagnosticCode::RemovedFeature)
            );
            assert_eq!(
                output.diagnostics[0].message,
                "removed feature: nested VAR declarations. Move global VAR declarations to the story top level, outside knots, stitches, functions, choices, conditionals, and sequences."
            );
        }
    }

    #[test]
    fn invalid_logic_expression_reports_specific_error_and_recovers_next_line() {
        let output = parse(SourceInput::new("~ x +\nLIST items = ()"));

        assert_eq!(output.diagnostics.len(), 2, "{:#?}", output.diagnostics);
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
        assert_eq!(
            output.diagnostics[1].code,
            Some(crate::diagnostic::DiagnosticCode::RemovedFeature)
        );
        assert_eq!(output.diagnostics[1].line, 2);
        assert_eq!(
            output.diagnostics[1].message,
            "removed feature: LIST declarations. Use variables, functions, or host data instead."
        );
    }

    #[test]
    fn invalid_choice_bracket_reports_specific_error_span() {
        let output = parse(SourceInput::new("* Hello [choice text"));

        assert_eq!(output.diagnostics.len(), 1, "{:#?}", output.diagnostics);
        assert_eq!(
            output.diagnostics[0].code,
            Some(crate::diagnostic::DiagnosticCode::InvalidChoiceSyntax)
        );
        assert_eq!(output.diagnostics[0].line, 1);
        assert_eq!(output.diagnostics[0].column, 9);
        assert_eq!(
            output.diagnostics[0].message,
            "expected closing `]` for choice-only text before end of line"
        );
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
