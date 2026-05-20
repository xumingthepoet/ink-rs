Progress: 17/27

# Logical Large-File Split Active Plan

This plan covers one refactoring round for production Rust files whose size is
caused by multiple independently growing responsibilities. It intentionally
does not target every file over 500 lines. A file qualifies only when the split
can name stable ownership boundaries, reduce local cognitive load, and leave
future feature work with an obvious home.

The target order is:

1. `crates/ink-runtime/src/story_state.rs`
2. `crates/ink-compiler/src/syntax/expression.rs`
3. `crates/ink-compiler/src/analysis/targets.rs`
4. `crates/ink-compiler/src/lower/expression.rs`
5. `crates/ink-story-json-format/src/json.rs`

Every implementation task must preserve behavior. Do not combine tasks unless
the previous task is already `[x]` and the combined write set is still easy to
review. Every task must pass focused validation plus `make gate` before its
implementation commit can be recorded as `[>]`.

## Milestone 1: Runtime Story State Boundaries

### [x] Task 01: Split StoryState Error And Warning Handling

Goal: Move current error and warning state helpers out of the large story state
file into a dedicated runtime state concern.

Implementation method: Convert `crates/ink-runtime/src/story_state.rs` into a
`story_state/` module if needed, then move `has_error`, `has_warning`,
`get_current_errors`, `get_current_warnings`, `add_error`, and `reset_errors`
into `crates/ink-runtime/src/story_state/errors.rs`. Keep `StoryState` fields
and method names unchanged for existing callers.

Acceptance criteria: Runtime callers compile without import changes outside the
module move. Error/warning behavior and messages are unchanged. The old file no
longer owns error/warning implementation bodies.

Forbidden shortcuts: Do not rename public or `pub(crate)` methods. Do not move
unrelated output, save, or function-evaluation code in this task.

Modification boundaries: `crates/ink-runtime/src/story_state*` and the module
declaration in `crates/ink-runtime/src/lib.rs` only if the file-to-directory
conversion requires it.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p ink-runtime
story_state::tests::malformed_evaluation_stack_underflow_returns_error`; `make
gate`.

Commit record: implementation commit `c760ed43 Split StoryState error
handling`; validation passed with `cargo fmt --all --check`, `cargo test -q -p
ink-runtime story_state::tests::malformed_evaluation_stack_underflow_returns_error`,
and `make gate`. Review found no follow-up changes; completion validation
passed with the same focused command and `make gate`.

### [x] Task 02: Split StoryState Output Text And Tag Readers

Goal: Give read-only output presentation logic a home separate from state
mutation and save/load.

Implementation method: Move `in_string_evaluation`, `get_current_text`,
`get_current_tags`, `clean_output_whitespace`, and
`output_stream_ends_in_newline` into `story_state/output.rs`. Keep helper
visibility as narrow as possible while preserving current callers.

Acceptance criteria: Current text and tag extraction produce identical results.
Whitespace cleaning remains covered by existing tests or newly relocated tests.

Forbidden shortcuts: Do not rewrite whitespace rules. Do not change tag command
interpretation or output stream storage.

Modification boundaries: `crates/ink-runtime/src/story_state/`.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p ink-runtime
story_state::tests`; `cargo test -p ink-test --test integration tags`; `make
gate`.

Commit record: implementation commit `446fc5e5 Split StoryState output
readers`; validation passed with `cargo fmt --all --check`, `cargo test -q -p
ink-runtime story_state::tests`, `cargo test -p ink-test --test integration
tags`, and `make gate`. Review found no follow-up changes; completion
validation passed with the same focused commands and `make gate`.

### [x] Task 03: Split StoryState Output Stream Mutation

Goal: Isolate output stream mutation, glue handling, and newline trimming from
core state bookkeeping.

Implementation method: Move `reset_output`, `get_output_stream`,
`get_output_stream_mut`, `output_stream_dirty`, `push_to_output_stream`,
`try_splitting_head_tail_whitespace`, `push_to_output_stream_individual`,
`trim_newlines_from_output_stream`, `remove_existing_glue`,
`output_stream_contains_content`, and `pop_from_output_stream` into
`story_state/output.rs` or `story_state/output_mutation.rs`, depending on the
shape left by Task 02.

Acceptance criteria: Continue/output tests remain unchanged. Output stream
dirtying and glue removal still happen in the same situations.

Forbidden shortcuts: Do not merge runtime output behavior with story progress
logic. Do not change newline or glue semantics to make tests pass.

Modification boundaries: `crates/ink-runtime/src/story_state/` plus imports in
runtime modules that directly use moved helpers.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p ink-runtime
story_state::tests`; `cargo test -p ink-test --test integration text`; `cargo
test -p ink-test --test integration glue`; `make gate`.

Commit record: implementation commit `2d2aa5cb Split StoryState output
mutation`; validation passed with `cargo fmt --all --check`, `cargo test -q -p
ink-runtime story_state::tests`, `cargo test -p ink-test --test integration
text`, `cargo test -p ink-test --test integration glue`, and `make gate`.
Review found no follow-up changes; completion validation passed with the same
focused commands and `make gate`.

### [x] Task 04: Split StoryState Evaluation Stack Operations

Goal: Move evaluation stack primitives into their own implementation unit.

Implementation method: Move `get_in_expression_evaluation`,
`set_in_expression_evaluation`, `push_evaluation_stack`,
`pop_evaluation_stack`, `pop_evaluation_stack_multiple`, and
`peek_evaluation_stack` into `story_state/evaluation_stack.rs`.

Acceptance criteria: Underflow and malformed evaluation errors are unchanged.
All existing callers continue using `StoryState` methods.

Forbidden shortcuts: Do not expose `evaluation_stack` more broadly. Do not
change error wording or stack pop order.

Modification boundaries: `crates/ink-runtime/src/story_state/`.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p ink-runtime
story_state::tests::malformed_evaluation_stack_underflow_returns_error`; `cargo
test -q -p ink-runtime story_state::tests::malformed_native_call_underflow_returns_error`;
`make gate`.

Commit record: implementation commit `47798a93 Split StoryState evaluation
stack`; validation passed with `cargo fmt --all --check`, `cargo test -q -p
ink-runtime story_state::tests::malformed_evaluation_stack_underflow_returns_error`,
`cargo test -q -p ink-runtime
story_state::tests::malformed_native_call_underflow_returns_error`, and `make
gate`. Review found no follow-up changes; completion validation passed with
the same focused commands and `make gate`.

### [x] Task 05: Split StoryState Function Evaluation API

Goal: Isolate host-driven function evaluation state transitions from general
story state.

Implementation method: Move `try_exit_function_evaluation_from_game`,
`start_function_evaluation_from_game`, `pass_arguments_to_evaluation_stack`,
`complete_function_evaluation_from_game`, and
`trim_whitespace_from_function_end` into `story_state/function_eval.rs`.

Acceptance criteria: Runtime API function calls, argument passing, return value
handling, and whitespace trimming remain unchanged.

Forbidden shortcuts: Do not change callstack push/pop semantics. Do not alter
host function API behavior.

Modification boundaries: `crates/ink-runtime/src/story_state/` and direct
imports affected by helper visibility.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration runtime_api`; `cargo test -p ink-test --test integration
functions`; `make gate`.

Commit record: implementation commit `2a70b7de Split StoryState function
evaluation`; validation passed with `cargo fmt --all --check`, `cargo test -p
ink-test --test integration runtime_api`, `cargo test -p ink-test --test
integration functions`, and `make gate`. Review found no follow-up changes;
completion validation passed with the same focused commands and `make gate`.

### [x] Task 06: Split StoryState Pointer And Choice State Helpers

Goal: Separate navigation-adjacent state fields from output and save logic.

Implementation method: Move `can_continue`, `current_path_string`,
`get_current_pointer`, `set_current_pointer`, `set_previous_pointer`,
`get_previous_pointer`, `get_generated_choices_mut`, `get_generated_choices`,
`get_current_choices`, `set_diverted_pointer`, `set_chosen_path`, `force_end`,
`set_did_safe_exit`, and `is_did_safe_exit` into `story_state/navigation.rs`
or `story_state/flow_state.rs`.

Acceptance criteria: Choice selection, force-end behavior, and current pointer
behavior remain unchanged.

Forbidden shortcuts: Do not move `Story` navigation behavior from
`story/navigation.rs` into story state. This task only moves `StoryState`
methods.

Modification boundaries: `crates/ink-runtime/src/story_state/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration choices`; `cargo test -p ink-test --test integration flow`; `make
gate`.

Commit record: implementation commit `2cd6a715 Split StoryState flow state
helpers`; validation passed with `cargo fmt --all --check`, `cargo test -p
ink-test --test integration choices`, `cargo test -p ink-test --test
integration flow`, and `make gate`. Review found no follow-up changes;
completion validation passed with the same focused commands and `make gate`.

### [x] Task 07: Split StoryState Patch Lifecycle

Goal: Give background-save and patch lifecycle code an owner separate from
runtime save JSON.

Implementation method: Move `copy_and_start_patching`,
`restore_after_patch`, and `apply_any_patch` into `story_state/patch.rs`.

Acceptance criteria: Existing save/load and runtime progression tests still
pass. `StatePatch` interactions remain local and readable.

Forbidden shortcuts: Do not change clone semantics, patch ownership, or
background-save behavior.

Modification boundaries: `crates/ink-runtime/src/story_state/`.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p ink-runtime
story_state::tests`; `cargo test -p ink-test --test integration runtime_api`;
`make gate`.

Commit record: implementation commit `3a582da0 Split StoryState patch
lifecycle`; validation passed with `cargo fmt --all --check`, `cargo test -q
-p ink-runtime story_state::tests`, `cargo test -p ink-test --test integration
runtime_api`, and `make gate`. Review found no follow-up changes; completion
validation passed with the same focused commands and `make gate`.

### [x] Task 08: Split Runtime Save-State JSON

Goal: Move runtime save/load versioning and JSON shape into a save-state module
while keeping it runtime-owned.

Implementation method: Move `INK_SAVE_STATE_VERSION`, `to_json`, `load_json`,
`write_json`, `ensure_minimal_save_ready`, and `load_json_obj` into
`story_state/save_json.rs`. Keep compiled-story JSON loading in
`json/json_read.rs`; this task is only for runtime save-state JSON.

Acceptance criteria: Save-state v2 JSON shape is unchanged. Malformed save
state errors still report the same messages. The architecture boundary between
compiled-story JSON and runtime save-state JSON remains visible.

Forbidden shortcuts: Do not route save-state JSON through
`ink-story-json-format`. Do not reintroduce old upstream save-state fields.

Modification boundaries: `crates/ink-runtime/src/story_state/`,
`crates/ink-runtime/src/story_state.rs` if it still exists, and architecture
docs only if the file map becomes stale.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p ink-runtime
story_state::tests::save_state_uses_minimal_v2_shape`; `cargo test -q -p
ink-runtime story_state::tests::save_state_roundtrips_array_and_object_variables`;
`cargo test -q -p ink-runtime story_state::tests::save_state_roundtrips_dict_variables_and_omits_defaults`;
`make gate`.

Commit record: implementation commit `a9348c47 Split runtime save-state JSON`;
validation passed with `cargo fmt --all --check`, `cargo test -q -p
ink-runtime story_state::tests::save_state_uses_minimal_v2_shape`, `cargo test
-q -p ink-runtime
story_state::tests::save_state_roundtrips_array_and_object_variables`, `cargo
test -q -p ink-runtime
story_state::tests::save_state_roundtrips_dict_variables_and_omits_defaults`,
and `make gate`. Review found no follow-up changes; completion validation
passed with the same focused commands and `make gate`.

## Milestone 2: Syntax Expression Boundaries

### [x] Task 09: Split Expression Token Model And Operator Tables

Goal: Separate expression token data definitions from tokenization and parsing.

Implementation method: Convert `crates/ink-compiler/src/syntax/expression.rs`
into an `expression/` module if needed, then move `ExpressionToken`,
`ExpressionTokenKind`, `BinaryOperatorRule`, `OperatorTokenKind`,
`binary_operator_rule`, and operator token lookup data into
`syntax/expression/token.rs`.

Acceptance criteria: Parser entry points keep the same paths for callers.
Operator precedence and token display behavior are unchanged.

Forbidden shortcuts: Do not change token variants, precedence values, or parsed
expression output.

Modification boundaries: `crates/ink-compiler/src/syntax/expression*`.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p
ink-compiler syntax::expression::tests::operator_rule_table_drill_covers_tokenizer_and_parser_lookup`;
`cargo test -p ink-test --test integration expressions`; `make gate`.

Commit record: implementation commit `d788f0fe Split expression token model`;
validation passed with `cargo fmt --all --check`, `cargo test -q -p
ink-compiler
syntax::expression::tests::operator_rule_table_drill_covers_tokenizer_and_parser_lookup`,
`cargo test -p ink-test --test integration expressions`, and `make gate`.
Review found no follow-up changes; completion validation passed with the same
focused commands and `make gate`.

### [x] Task 10: Split Expression Tokenizer

Goal: Give lexical scanning for expressions an independent module.

Implementation method: Move `tokenize_expression`,
`tokenize_expression_at`, `token_at`, `match_operator`,
`read_string_literal`, `is_token_word_start`, `read_token_word`,
`classify_word_token_with_decimal_suffix`, and `classify_word_token` into
`syntax/expression/tokenize.rs` or `lexer.rs`.

Acceptance criteria: Existing tokenizer tests remain meaningful and pass after
relocation. Unicode column tracking and separator handling are unchanged.

Forbidden shortcuts: Do not replace structured tokenization with ad hoc string
splitting. Do not change diagnostic spans.

Modification boundaries: `crates/ink-compiler/src/syntax/expression/`.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p
ink-compiler syntax::expression::tests::tokenizer_covers_expression_token_categories`;
`cargo test -q -p ink-compiler syntax::expression::tests::tokenizer_tracks_character_columns_for_unicode_prefixes`;
`make gate`.

Commit record: implementation commit `6b500f7e Split expression tokenizer`;
validation passed with `cargo fmt --all --check`, `cargo test -q -p
ink-compiler
syntax::expression::tests::tokenizer_covers_expression_token_categories`,
`cargo test -q -p ink-compiler
syntax::expression::tests::tokenizer_tracks_character_columns_for_unicode_prefixes`,
and `make gate`. Review found no follow-up changes; completion validation
passed with the same focused commands and `make gate`.

### [x] Task 11: Split Expression Parse Errors

Goal: Move structured parse error construction and diagnostic formatting away
from parser mechanics.

Implementation method: Move `ExpressionParseError`,
`ExpressionParseErrorKind`, `ExpressionParseError` impls, `found_clause`, and
`describe_token_kind` into `syntax/expression/error.rs`.

Acceptance criteria: Syntax diagnostics and spans are unchanged for expression
parse failures.

Forbidden shortcuts: Do not simplify diagnostics to generic parse failures. Do
not change error text unless an existing test explicitly requires a maintained
diagnostic update.

Modification boundaries: `crates/ink-compiler/src/syntax/expression/`.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p
ink-compiler syntax::expression::tests::token_parser_reports_structured_errors_with_spans`;
`cargo test -p ink-test --test integration diagnostics`; `make gate`.

Commit record: implementation commit `416d4012 Split expression parse errors`;
validation passed with `cargo fmt --all --check`, `cargo test -q -p
ink-compiler
syntax::expression::tests::token_parser_reports_structured_errors_with_spans`,
`cargo test -p ink-test --test integration diagnostics`, and `make gate`.
Review found no follow-up changes; completion validation passed with the same
focused commands and `make gate`.

### [x] Task 12: Split Expression Pratt Parser Core

Goal: Isolate the expression parser state machine from literal-specific parse
helpers.

Implementation method: Move `TokenExpressionParser`, `parse_token_expression`,
`parse_token_expression_at`, `end_span`, `unary_expression`, and parser cursor
helpers into `syntax/expression/parser.rs`. Leave literal parsing delegated to
the modules created by later tasks only when each literal boundary is moved.

Acceptance criteria: Parsed expression snapshots remain unchanged. Public
syntax entry points still live behind `syntax::expression`.

Forbidden shortcuts: Do not change parse trial order, binding power, or
associativity.

Modification boundaries: `crates/ink-compiler/src/syntax/expression/`.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p
ink-compiler syntax::expression::tests::token_parser_reproduces_current_expression_baseline`;
`cargo test -p ink-test --test integration expressions`; `make gate`.

Commit record: implementation commit `69b68ac9 Split expression Pratt parser
core`; validation passed with `cargo fmt --all --check`, `cargo test -q -p
ink-compiler
syntax::expression::tests::token_parser_reproduces_current_expression_baseline`,
`cargo test -p ink-test --test integration expressions`, and `make gate`.
Review found no follow-up changes; completion validation passed with the same
focused commands and `make gate`.

### [x] Task 13: Split Composite Expression Literal Parsing

Goal: Give array, struct, and dict expression literals a focused parser owner.

Implementation method: Move `parse_array_literal`, `parse_braced_literal`,
`parse_percent_literal`, `parse_struct_literal`, `parse_dict_literal`, and
`parse_dict_literal_key` into `syntax/expression/literals.rs`. Keep the parser
core as the caller.

Acceptance criteria: Typed-value syntax fixtures and parser tests keep the same
parsed structures.

Forbidden shortcuts: Do not narrow accepted literal syntax to current fixture
forms. Do not move semantic type checks into syntax.

Modification boundaries: `crates/ink-compiler/src/syntax/expression/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration typed_values`; `cargo test -q -p ink-compiler
syntax::expression::tests::tokenizer_covers_struct_literal_tokens`; `make
gate`.

Commit record: implementation commit `446322e8 Split composite expression
literals`; validation passed with `cargo fmt --all --check`, `cargo test -p
ink-test --test integration typed_values`, `cargo test -q -p ink-compiler
syntax::expression::tests::tokenizer_covers_struct_literal_tokens`, and `make
gate`. Review found no follow-up changes; completion validation passed with
the same focused commands and `make gate`.

### [x] Task 14: Split Dynamic Interface Expression Parsing

Goal: Give dynamic interface target/member expression syntax its own growth
point.

Implementation method: Move `current_brace_pair_is_dynamic_interface_target`,
`parse_dynamic_interface_target`, `parse_dynamic_interface_member_after_target`,
and related separator handling that is specific to dynamic interface
expressions into `syntax/expression/dynamic_interface.rs`.

Acceptance criteria: Dynamic interface parsing behavior and diagnostics are
unchanged.

Forbidden shortcuts: Do not merge dynamic interface parsing with analysis or
lowering. Do not hardcode module/interface names from tests.

Modification boundaries: `crates/ink-compiler/src/syntax/expression/`.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p
ink-compiler syntax::expression::tests::tokenizer_covers_dynamic_interface_separators`;
`cargo test -p ink-test --test integration typed_values`; `make gate`.

Commit record: implementation commit `94b85279 Split dynamic interface
expression parser`; validation passed with `cargo fmt --all --check`, `cargo
test -q -p ink-compiler
syntax::expression::tests::tokenizer_covers_dynamic_interface_separators`,
`cargo test -p ink-test --test integration typed_values`, and `make gate`.
Review found no follow-up changes; completion validation passed with the same
focused commands and `make gate`.

### [x] Task 15: Split Expression String Parsing

Goal: Separate string-expression content parsing from general token expression
parsing.

Implementation method: Move `parse_string_expression` and
`flatten_string_expression_content` into `syntax/expression/string.rs`. Keep
dependencies on text parsing explicit and local.

Acceptance criteria: String expressions lower and execute as before. Existing
expression and text tests pass.

Forbidden shortcuts: Do not change string interpolation, nested logic, or text
flattening behavior.

Modification boundaries: `crates/ink-compiler/src/syntax/expression/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration text`; `cargo test -p ink-test --test integration expressions`;
`make gate`.

Commit record: implementation commit `8d761283 Split expression string
parsing`; validation passed with `cargo fmt --all --check`, `cargo test -p
ink-test --test integration text`, `cargo test -p ink-test --test integration
expressions`, and `make gate`. Review found no follow-up changes; completion
validation passed with the same focused commands and `make gate`.

### [x] Task 16: Split Expression Argument And Path Utilities

Goal: Keep reusable top-level expression helpers separate from parser internals.

Implementation method: Move `split_top_level_args` and `is_path_identifier`
into `syntax/expression/args.rs` or `syntax/expression/path.rs`, depending on
actual coupling after earlier tasks. Keep function visibility restricted to the
syntax modules that need these helpers.

Acceptance criteria: Function calls, external declarations, and diverts still
parse the same argument boundaries.

Forbidden shortcuts: Do not replace balanced scanning with comma splitting.
Do not introduce duplicate argument splitting helpers elsewhere.

Modification boundaries: `crates/ink-compiler/src/syntax/expression/` and
callers in `crates/ink-compiler/src/syntax/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration functions`; `cargo test -p ink-test --test integration diverts`;
`make gate`.

Commit record: implementation commit `24345619 Split expression argument
utilities`; validation passed with `cargo fmt --all --check`, `cargo test -p
ink-test --test integration functions`, `cargo test -p ink-test --test
integration diverts`, and `make gate`. Review found no follow-up changes;
completion validation passed with the same focused commands and `make gate`.

## Milestone 3: Analysis Target Boundaries

### [x] Task 17: Split Target Checker Module Shell

Goal: Create a module boundary for target diagnostics without changing any
checks yet.

Implementation method: Convert `analysis/targets.rs` into
`analysis/targets/mod.rs`, keep public entry points
`call_target_diagnostics` and `call_target_diagnostics_with_indexes`, and move
`CallTargetChecker` into `analysis/targets/checker.rs`.

Acceptance criteria: All analysis callers still use `analysis::targets` as
before. This task should be a pure module-boundary extraction with no semantic
changes.

Forbidden shortcuts: Do not change pass ordering in `analysis/mod.rs`. Do not
split individual checks in this task.

Modification boundaries: `crates/ink-compiler/src/analysis/targets*` and module
declarations needed by Rust.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration diagnostics`; `make gate`.

Commit record: implementation commit `6e898e83 Split target checker module
shell`; validation passed with `cargo fmt --all --check`, `cargo test -p
ink-test --test integration diagnostics`, and `make gate`. Review found no
follow-up changes; completion validation passed with the same focused command
and `make gate`.

### [~] Task 18: Split Divert Target Checks

Goal: Move static, dynamic, conditional, tunnel, and cross-module divert target
checking into a focused module.

Implementation method: Move `check_plain_divert_target`,
`check_plain_divert_target_expression`, `check_dynamic_divert_target`,
`check_divert_target_value`, `check_cross_module_stitch_target`, and
`static_divert_target_name` into `analysis/targets/diverts.rs`.

Acceptance criteria: Divert diagnostics and accepted target shapes are
unchanged.

Forbidden shortcuts: Do not change module target visibility rules. Do not add
fixture-name or path-name special cases.

Modification boundaries: `crates/ink-compiler/src/analysis/targets/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration diverts`; `cargo test -p ink-test --test integration diagnostics`;
`make gate`.

Commit record: pending.

### [ ] Task 19: Split Function And External Call Checks

Goal: Separate ordinary call target and signature validation from built-in and
interface-specific checks.

Implementation method: Move `check_function_call`,
`check_function_call_signature`, `infer_call_argument_type`, and callable
target helpers into `analysis/targets/calls.rs`.

Acceptance criteria: Function and external call argument count/type diagnostics
are unchanged.

Forbidden shortcuts: Do not mix lowering target names into analysis. Do not
change external declaration interpretation.

Modification boundaries: `crates/ink-compiler/src/analysis/targets/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration functions`; `cargo test -p ink-test --test integration diagnostics`;
`make gate`.

Commit record: pending.

### [ ] Task 20: Split Typed Builtin Target Checks

Goal: Give runtime built-in call type checking its own module.

Implementation method: Move `check_typed_builtin_call`,
`check_array_remove_call`, `check_array_push_call`,
`check_array_insert_call`, `check_array_argument`, `check_int_argument`,
`check_array_value_argument`, `check_len_call`, dict builtin checks, and
`dict_key_type_name` into `analysis/targets/builtins.rs`.

Acceptance criteria: Array, dict, and `LEN` diagnostics and accepted call forms
are unchanged.

Forbidden shortcuts: Do not duplicate built-in name tables outside the new
module. Do not change runtime built-in semantics.

Modification boundaries: `crates/ink-compiler/src/analysis/targets/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration typed_values`; `cargo test -p ink-test --test integration
diagnostics`; `make gate`.

Commit record: pending.

### [ ] Task 21: Split Dynamic Interface Target Checks

Goal: Give dynamic interface target and function call validation a module that
can grow with interface semantics.

Implementation method: Move `check_dynamic_interface_divert_target`,
`dynamic_interface_member_signature`,
`check_dynamic_interface_function_call`,
`dynamic_interface_function_signature`,
`check_dynamic_interface_member_arguments`, and
`is_interface_module_literal_argument` into `analysis/targets/interfaces.rs`.

Acceptance criteria: Interface module literal, dynamic interface target, and
dynamic interface function diagnostics are unchanged.

Forbidden shortcuts: Do not change import/reachability semantics. Do not
hardcode interface names or implementations.

Modification boundaries: `crates/ink-compiler/src/analysis/targets/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration modules`; `cargo test -p ink-test --test integration typed_values`;
`make gate`.

Commit record: pending.

### [ ] Task 22: Split Target Context Helpers

Goal: Move target-analysis helper functions that are shared across checks into
a stable utility module.

Implementation method: Move `current_flow_path`, `current_module`,
`current_flow_context`, `dynamic_interface_signature_inputs`,
`current_flow_arguments`, `current_flow_is_function`, `is_mutable_lvalue`,
`is_composite_literal`, `is_runtime_builtin_function`,
`expression_root_variable_name`, `scoped_context_key`, and
`resolve_current_flow_argument` into `analysis/targets/context.rs` or
`helpers.rs`.

Acceptance criteria: Target checker modules depend on shared helpers instead
of reaching into each other. Behavior and diagnostics remain unchanged.

Forbidden shortcuts: Do not make helpers public outside `analysis::targets`
unless an existing external caller requires it.

Modification boundaries: `crates/ink-compiler/src/analysis/targets/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration diagnostics`; `cargo test -p ink-test --test integration modules`;
`make gate`.

Commit record: pending.

## Milestone 4: Lowering Expression Boundaries

### [ ] Task 23: Split Lower Expression Operators And Name Resolution

Goal: Separate operator token mapping and runtime name resolution from the main
expression lowering traversal.

Implementation method: Convert `lower/expression.rs` into a module if needed.
Move `native_function_for_binary_operator`,
`native_function_for_unary_operator`, `builtin_native_function`,
`resolve_runtime_variable_name`, `resolve_constant_name`, and
`resolve_callable_name` into `lower/expression/operators.rs` and
`lower/expression/name_resolution.rs`.

Acceptance criteria: Lowered JSON for expression operators and variable/call
names is unchanged.

Forbidden shortcuts: Do not change variable visibility order. Do not duplicate
native-function mappings already owned by the format/runtime metadata.

Modification boundaries: `crates/ink-compiler/src/lower/expression*`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration expressions`; `cargo test -p ink-test --test integration
variables`; `make gate`.

Commit record: pending.

### [ ] Task 24: Split Lower Expression Type Inference Helpers

Goal: Give lowering-time type inference and type qualification helpers their
own module.

Implementation method: Move `infer_lowered_expression_type`,
`visible_variable_or_constant_type`, `callable_return_type`,
`builtin_return_type_for_call`, `qualify_field_type_for_base`,
`qualify_type_for_qualified_name`, `qualify_type_name_for_module`, and
`qualified_type_name` into `lower/expression/types.rs`.

Acceptance criteria: Lowering still uses analysis-checked types consistently.
No new diagnostics are introduced by lowering.

Forbidden shortcuts: Do not move semantic validation from analysis into
lowering. Do not alter type names to satisfy snapshots.

Modification boundaries: `crates/ink-compiler/src/lower/expression/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration typed_values`; `cargo test -p ink-test --test integration
compiler_snapshots`; `make gate`.

Commit record: pending.

### [ ] Task 25: Split Lower Expression Calls And Collection Builtins

Goal: Move function call and collection mutation lowering into dedicated
modules.

Implementation method: Move `lower_function_call_into`,
`runtime_function_target`, `lower_array_remove_call_into`,
`lower_array_push_call_into`, `lower_array_insert_call_into`,
`infer_array_element_type`, `lower_dict_remove_call_into`,
`lower_function_arg_into`, and `lower_function_arg_into_parts` into
`lower/expression/calls.rs` and `lower/expression/builtins.rs`.

Acceptance criteria: Function call JSON, external call JSON, and collection
mutation JSON are unchanged.

Forbidden shortcuts: Do not hardcode snapshot fragments. Do not change lvalue
or assignment-lowering behavior in this task.

Modification boundaries: `crates/ink-compiler/src/lower/expression/` plus
imports from sibling lowering modules.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration functions`; `cargo test -p ink-test --test integration
typed_values`; `make gate`.

Commit record: pending.

### [ ] Task 26: Split Lower Dynamic Interface Expressions

Goal: Give dynamic interface target/function lowering a focused owner.

Implementation method: Move `dynamic_interface_knot_signature`,
`dynamic_interface_function_signature`,
`lower_dynamic_interface_target_into`, and
`lower_dynamic_interface_function_call_into` into
`lower/expression/dynamic_interface.rs`.

Acceptance criteria: Dynamic interface lowered JSON and runtime behavior are
unchanged.

Forbidden shortcuts: Do not change interface metadata collection in
`lower.rs`. Do not move analysis checks into lowering.

Modification boundaries: `crates/ink-compiler/src/lower/expression/`.

Validation commands: `cargo fmt --all --check`; `cargo test -p ink-test --test
integration modules`; `cargo test -p ink-test --test integration typed_values`;
`make gate`.

Commit record: pending.

## Milestone 5: Format JSON Codec Boundary And Closeout

### [ ] Task 27: Split Format JSON Codec And Close The Plan

Goal: Give the compiled-story JSON codec independent modules for program,
metadata, containers, objects, and dynamic values, then close the active plan.

Implementation method: Convert `crates/ink-story-json-format/src/json.rs` into
`json/` modules with a narrow public surface equivalent to the current
`json.rs` module. Extract program/metadata handling, container handling, object
handling, dict/dynamic values, and scalar helper functions only where the
boundaries are clear after the earlier runtime/compiler refactors. Update
`docs/Architecture.md` if the format crate file map changes. After validation,
move this plan directory to `docs/finished_plans/` and record the final commit
metadata before the move.

Acceptance criteria: `ink-story-json-format` remains the only compiled-story
JSON wire-format owner. Compiler emission and runtime loading still go through
the format crate. The active plan is moved to finished plans only after all
prior tasks are `[x]`.

Forbidden shortcuts: Do not add duplicate compiled-story JSON schemas in
compiler or runtime. Do not split helpers into tiny modules without a stable
ownership label.

Modification boundaries: `crates/ink-story-json-format/src/json*`,
`crates/ink-story-json-format/src/lib.rs`, `docs/Architecture.md`, and this
plan directory during closeout.

Validation commands: `cargo fmt --all --check`; `cargo test -q -p
ink-story-json-format`; `cargo test -p ink-test --test integration
compiler_snapshots`; `make gate`.

Commit record: pending.
