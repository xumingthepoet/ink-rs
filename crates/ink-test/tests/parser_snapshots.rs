use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use ink_compiler::{parsed::snapshot::render_story, parser::InkParser, FileHandler};
use ink_test::load_fixture_text;

#[derive(Debug)]
struct IncludeFileHandler {
    files: HashMap<PathBuf, String>,
}

impl IncludeFileHandler {
    fn new(files: HashMap<PathBuf, String>) -> Self {
        Self { files }
    }
}

impl FileHandler for IncludeFileHandler {
    fn resolve_ink_filename(&self, include_name: &str) -> PathBuf {
        PathBuf::from("/virtual").join(include_name)
    }

    fn load_ink_file_contents(&self, full_filename: &Path) -> io::Result<String> {
        self.files.get(full_filename).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing include: {}", full_filename.display()),
            )
        })
    }
}

fn parse_and_render(
    source: &str,
    source_filename: &str,
    file_handler: Option<Arc<dyn FileHandler>>,
) -> String {
    let mut parser = InkParser::new(source, Some(source_filename), file_handler);
    let result = parser.parse();

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:#?}",
        result.diagnostics
    );

    let story = result.parsed_story.expect("expected parsed story");
    render_story(&story)
}

#[test]
fn ink_parser_parses_trusted_basictext_twolines_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/basictext/twolines.ink");
    assert_eq!(
        parse_and_render(&source, "twolines.ink", None),
        "Story\n  ContentList\n    Text(\"Line.\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"Other line.\")\n    Text(\"\\n\")"
    );
}

#[test]
fn ink_parser_parses_trusted_basictext_oneline_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/basictext/oneline.ink");
    assert_eq!(
        parse_and_render(&source, "oneline.ink", None),
        "Story\n  ContentList\n    Text(\"Line.\")\n    Text(\"\\n\")"
    );
}

#[test]
fn ink_parser_parses_trusted_conditional_iftrue_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/conditional/iftrue.ink");
    assert_eq!(
        parse_and_render(&source, "iftrue.ink", None),
        "Story\n  ContentList\n    Text(\"\\n\")\n  VariableAssignment(name=\"x\", global=true, temp=false)\n    Number(2)\n  VariableAssignment(name=\"y\", global=true, temp=false)\n    Number(0)\n  Conditional\n    Binary(>, VariableReference(x), Number(0))\n    ConditionalBranch(true=true, else=false, inline=false)\n      VariableAssignment(name=\"y\", global=false, temp=false)\n        Binary(-, VariableReference(x), Number(1))\n  ContentList\n    Text(\"        The value is {y}. \")\n    Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)\n    Text(\"\\n\")"
    );
}

#[test]
fn ink_parser_parses_trusted_conditional_ifelse_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/conditional/ifelse.ink");
    assert_eq!(
        parse_and_render(&source, "ifelse.ink", None),
        "Story\n  ContentList\n    Text(\"\\n\")\n  VariableAssignment(name=\"x\", global=true, temp=false)\n    Number(0)\n  VariableAssignment(name=\"y\", global=true, temp=false)\n    Number(3)\n  Conditional\n    Binary(>, VariableReference(x), Number(0))\n    ConditionalBranch(true=true, else=false, inline=false)\n      VariableAssignment(name=\"y\", global=false, temp=false)\n        Binary(-, VariableReference(x), Number(1))\n    ConditionalBranch(true=false, else=true, inline=false)\n      VariableAssignment(name=\"y\", global=false, temp=false)\n        Binary(+, VariableReference(x), Number(1))\n  ContentList\n    Text(\"        The value is {y}. \")\n    Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)"
    );
}

#[test]
fn ink_parser_parses_trusted_function_none_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/function/func-none.ink");
    assert_eq!(
        parse_and_render(&source, "func-none.ink", None),
        "Story\n  VariableAssignment(name=\"x\", global=true, temp=false)\n    Number(0)\n  VariableAssignment(name=\"x\", global=false, temp=false)\n    FunctionCall(f, args=0)\n  ContentList\n    Text(\"  The value of x is {x}.\")\n    Text(\"\\n\")\n  Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"f()\", function=true)\n    Return\n      Number(3.8)"
    );
}

#[test]
fn ink_parser_parses_trusted_function_basic_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/function/func-basic.ink");
    assert_eq!(
        parse_and_render(&source, "func-basic.ink", None),
        "Story\n  VariableAssignment(name=\"x\", global=true, temp=false)\n    Number(0)\n  VariableAssignment(name=\"x\", global=false, temp=false)\n    FunctionCall(lerp, args=3)\n  ContentList\n    Text(\"  The value of x is {x}.\")\n    Text(\"\\n\")\n  Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"lerp(a,\", function=true)\n    Return\n      Binary(+, Binary(*, Binary(-, VariableReference(b), VariableReference(a)), VariableReference(k)), VariableReference(a))"
    );
}

#[test]
fn ink_parser_parses_trusted_function_inline_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/function/func-inline.ink");
    assert_eq!(
        parse_and_render(&source, "func-inline.ink", None),
        "Story\n  ContentList\n    Text(\"The value of x is {lerp(2, 8, 0.4)}.\")\n    Text(\"\\n\")\n  Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"lerp(a,\", function=true)\n    Return\n      Binary(+, Binary(*, Binary(-, VariableReference(b), VariableReference(a)), VariableReference(k)), VariableReference(a))"
    );
}

#[test]
fn ink_parser_parses_trusted_glue_left_right_matching_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/glue/left-right-glue-matching.ink");
    assert_eq!(
        parse_and_render(&source, "left-right-glue-matching.ink", None),
        "Story\n  ContentList\n    Text(\"A line.\")\n    Text(\"\\n\")\n  Conditional\n    FunctionCall(f, args=0)\n    ConditionalBranch(true=true, else=false, inline=false)\n      Text(\"Another line.\")\n      Text(\"\\n\")\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"f\", function=true)\n    Conditional\n      Number(false)\n      ConditionalBranch(true=true, else=false, inline=true)\n        ContentList\n          Text(\"nothing\")\n    Return\n      Number(true)"
    );
}

#[test]
fn ink_parser_parses_trusted_glue_testbugfix1_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/glue/testbugfix1.ink");
    assert_eq!(
        parse_and_render(&source, "testbugfix1.ink", None),
        "Story\n  ContentList\n    Text(\"A\")\n    Text(\"\\n\")\n  Conditional\n    FunctionCall(f, args=0)\n    ConditionalBranch(true=true, else=false, inline=true)\n      ContentList\n        Text(\"X\")\n  ContentList\n    Text(\"C\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"f()\", function=true)\n    Conditional\n      Number(true)\n      ConditionalBranch(true=true, else=false, inline=false)\n        Return\n          Number(false)"
    );
}

#[test]
fn ink_parser_parses_trusted_glue_testbugfix2_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/glue/testbugfix2.ink");
    assert_eq!(
        parse_and_render(&source, "testbugfix2.ink", None),
        "Story\n  ContentList\n    Text(\"A {f():B}\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"X\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"f()\", function=true)\n    Conditional\n      Number(true)\n      ConditionalBranch(true=true, else=false, inline=false)\n        Return\n          Number(false)"
    );
}

#[test]
fn ink_parser_parses_trusted_function_setvar_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/function/setvar-func.ink");
    assert_eq!(
        parse_and_render(&source, "setvar-func.ink", None),
        "Story\n  FunctionCall(herp, args=2)\n  ContentList\n    Text(\"  The value is {x}.\")\n    Text(\"\\n\")\n  Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"herp(a,\", function=true)\n    VariableAssignment(name=\"x\", global=true, temp=false)\n      Number(0)\n    VariableAssignment(name=\"x\", global=false, temp=false)\n      Binary(*, VariableReference(a), VariableReference(b))"
    );
}

#[test]
fn ink_parser_parses_trusted_function_rnd_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/function/rnd-func.ink");
    assert_eq!(
        parse_and_render(&source, "rnd-func.ink", None),
        "Story\n  FunctionCall(SEED_RANDOM, args=1)\n  ContentList\n    Text(\"\\n\")\n  ContentList\n    Text(\"Rolling dice 1: {RANDOM(1,6)}.\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"Rolling dice 2: {RANDOM(1,6)}.\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"Rolling dice 3: {RANDOM(1,6)}.\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"Rolling dice 4: {RANDOM(1,6)}.\")"
    );
}

#[test]
fn ink_parser_parses_trusted_function_complex_func1_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/function/complex-func1.ink");
    assert_eq!(
        parse_and_render(&source, "complex-func1.ink", None),
        "Story\n  FunctionCall(derp, args=3)\n  ContentList\n    Text(\"   The values are {x} and {y}.\")\n    Text(\"\\n\")\n  Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)\n  ContentList\n    Text(\"\\n\")\n  Flow(level=Knot, name=\"derp(a,\", function=true)\n    VariableAssignment(name=\"x\", global=true, temp=false)\n      Number(0)\n    VariableAssignment(name=\"x\", global=false, temp=false)\n      Binary(+, VariableReference(a), VariableReference(b))\n    VariableAssignment(name=\"y\", global=true, temp=false)\n      Number(3)\n    Conditional\n      Binary(==, VariableReference(x), Number(5))\n      ConditionalBranch(true=true, else=false, inline=false)\n        VariableAssignment(name=\"x\", global=false, temp=false)\n          Number(6)\n    VariableAssignment(name=\"y\", global=false, temp=false)\n      Binary(+, VariableReference(x), VariableReference(c))"
    );
}

#[test]
fn ink_parser_parses_trusted_function_complex_func2_fixture() {
    let source = load_fixture_text("conformance-tests/inkfiles/function/complex-func2.ink");
    let render = parse_and_render(&source, "complex-func2.ink", None);
    assert!(render.contains("FunctionCall(derp, args=2)"));
    assert!(render.contains("Text(\"    The values are {x} and {y} and {z}.\")"));
    assert!(render.contains("Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)"));
    assert!(render.contains("Flow(level=Knot, name=\"derp(a,\", function=true)"));
    assert!(render.contains("Conditional"));
    assert!(render.contains("Binary(==, VariableReference(x), Number(0))"));
    assert!(render.contains("Binary(>, VariableReference(x), Number(0))"));
    assert!(render.contains("VariableAssignment(name=\"z\", global=true, temp=false)"));
}

#[test]
fn ink_parser_parses_trusted_function_evaluating_variablestate_bug_fixture() {
    let source = load_fixture_text(
        "conformance-tests/inkfiles/function/evaluating-function-variablestate-bug.ink",
    );
    let mut files = HashMap::new();
    files.insert(
        PathBuf::from("/virtual/test_included_file.ink"),
        load_fixture_text("ink-csharp/tests/test_included_file.ink"),
    );
    let file_handler: Arc<dyn FileHandler> = Arc::new(IncludeFileHandler::new(files));

    let render = parse_and_render(
        &source,
        "evaluating-function-variablestate-bug.ink",
        Some(file_handler),
    );
    assert!(
        render.contains("Divert(target=\"-> tunnel\", empty=false, tunnel=false, thread=false)")
    );
    assert!(render.contains("Divert(target=\"->\", empty=false, tunnel=true, thread=false)"));
    assert!(render.contains("Flow(level=Knot, name=\"function_to_evaluate()\", function=true)"));
    assert!(render.contains("FunctionCall(zero_equals_, args=1)"));
    assert!(render.contains("Flow(level=Knot, name=\"zero_equals_(k)\", function=true)"));
    assert!(render.contains("FunctionCall(do_nothing, args=1)"));
    assert!(render.contains("Flow(level=Knot, name=\"do_nothing(k)\", function=true)"));
    assert!(render.contains("Conditional"));
    assert!(render.contains("Return"));
    assert!(render.contains("Binary(==, Number(0), VariableReference(k))"));
}

#[test]
fn ink_parser_parses_official_recursive_include_chain() {
    let source = load_fixture_text("ink-csharp/tests/test_included_file3.ink");
    let mut files = HashMap::new();
    files.insert(
        PathBuf::from("/virtual/test_included_file4.ink"),
        load_fixture_text("ink-csharp/tests/test_included_file4.ink"),
    );
    let file_handler: Arc<dyn FileHandler> = Arc::new(IncludeFileHandler::new(files));

    let render = parse_and_render(
        &source,
        "/virtual/test_included_file3.ink",
        Some(file_handler),
    );
    assert_eq!(
        render,
        "Story\n  VariableAssignment(name=\"t2\", global=true, temp=false)\n    Number(5)\n  ContentList\n    Text(\"\\n\")\n  ContentList\n    Text(\"The value of a variable in test file 2 is { t2 }.\")\n    Text(\"\\n\")\n  ContentList\n    Text(\"\\n\")\n  Text(\"\\n\")\n  Flow(level=Knot, name=\"knot_in_2\", function=false)\n    ContentList\n      Text(\" The value when accessed from knot_in_2 is { t2 }.\")\n      Text(\"\\n\")\n    Divert(target=\"-> END\", empty=false, tunnel=false, thread=false)"
    );
}

#[test]
fn ink_parser_parses_official_include_text_chain() {
    let source = "\
INCLUDE test_included_file.ink\n\
  INCLUDE test_included_file2.ink\n\
\n\
This is the main file.\n";
    let mut files = HashMap::new();
    files.insert(
        PathBuf::from("/virtual/test_included_file.ink"),
        load_fixture_text("ink-csharp/tests/test_included_file.ink"),
    );
    files.insert(
        PathBuf::from("/virtual/test_included_file2.ink"),
        load_fixture_text("ink-csharp/tests/test_included_file2.ink"),
    );
    let file_handler: Arc<dyn FileHandler> = Arc::new(IncludeFileHandler::new(files));

    let render = parse_and_render(source, "include-chain.ink", Some(file_handler));
    assert_eq!(
        render,
        "Story\n  ContentList\n    Text(\"This is include 1.\")\n  Text(\"\\n\")\n  ContentList\n    Text(\"This is include 2.\")\n  Text(\"\\n\")\n  ContentList\n    Text(\"\\n\")\n  ContentList\n    Text(\"This is the main file.\")\n    Text(\"\\n\")"
    );
}
