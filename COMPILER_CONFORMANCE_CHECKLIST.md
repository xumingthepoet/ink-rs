# compiler_conformance_legacy Checklist

Progress: **111/126** completed.

This is the working queue for the imported legacy compiler-conformance suite.
The order now follows the broad progression in `ink-csharp/Documentation/
WritingWithInk.md`: content, choices, knots, diverts, flow branching,
weaves, variables, advanced flow control, and finally the hardest bug
fixtures. Dependency-light cases still come first within each chapter.

Legend:

- `[x]` done
- `[ ]` pending

## 1. Minimal text and simple flow/export

- [x] `basictext/oneline`
- [x] `basictext/twolines`
- [x] `test1`

## 2. Basic choices

- [x] `choices/no-choice-text`
- [x] `choices/one`
- [x] `choices/multi-choice`
- [x] `choices/single-choice`
- [x] `choices/suppress-choice`
- [x] `choices/mixed-choice`
- [x] `choices/varying-choice`

## 3. Choice edge cases

- [x] `choices/fallback-choice`
- [x] `choices/fallback-choice2`
- [x] `choices/conditional-choice`
- [x] `choices/label-flow`
- [x] `choices/label-flow2`
- [x] `choices/label-scope`
- [ ] `choices/label-scope-error`
- [x] `choices/divert-choice`
- [x] `choices/sticky-choice`

## 4. Knots and stitches

- [x] `knot/single-line`
- [x] `knot/multi-line`
- [x] `knot/strip-empty-lines`
- [x] `knot/param-ints`
- [x] `knot/param-floats`
- [x] `knot/param-strings`
- [x] `knot/param-vars`
- [x] `knot/param-multi`
- [x] `knot/param-recurse`
- [x] `stitch/auto-stitch`
- [x] `stitch/auto-stitch2`
- [x] `stitch/manual-stitch`
- [x] `stitch/manual-stitch2`

## 5. Diverts and glue

- [x] `divert/simple-divert`
- [ ] `divert/invisible-divert`
- [ ] `divert/divert-on-choice`
- [ ] `divert/complex-branching`
- [x] `glue/simple-glue`
- [x] `glue/glue-with-divert`
- [x] `glue/left-right-glue-matching`
- [x] `glue/testbugfix1`
- [x] `glue/testbugfix2`

## 6. Gather and nested flow

- [x] `gather/gather-basic`
- [ ] `gather/gather-chain`
- [ ] `gather/nested-gather`
- [ ] `gather/nested-flow`
- [ ] `gather/deep-nesting`
- [x] `gather/complex-flow`

## 7. Conditionals and sequences

- [x] `conditional/iftrue`
- [x] `conditional/iffalse`
- [x] `conditional/ifelse`
- [x] `conditional/ifelse-ext`
- [x] `conditional/ifelse-ext-text1`
- [x] `conditional/ifelse-ext-text2`
- [x] `conditional/ifelse-ext-text3`
- [x] `conditional/condtext`
- [x] `conditional/condopt`
- [x] `conditional/cycle`
- [x] `conditional/once`
- [x] `conditional/shuffle`
- [x] `conditional/shuffle_once`
- [x] `conditional/shuffle_stopping`
- [x] `conditional/stopping`
- [x] `conditional/multiline`
- [x] `conditional/multiline-divert`
- [x] `conditional/multiline-choice`

## 8. Functions

- [x] `function/func-none`
- [x] `function/func-basic`
- [x] `function/func-inline`
- [x] `function/setvar-func`
- [x] `function/rnd-func`
- [x] `function/complex-func1`
- [ ] `function/complex-func2`
- [x] `function/complex-func3`
- [x] `function/evaluating-function-variablestate-bug`
- [ ] `function/test-error`

## 9. Variables, variable text, and lists

- [x] `variable/variable-declaration`
- [x] `variable/varcalc`
- [x] `variable/varstringinc`
- [x] `variable/var-divert`
- [x] `variabletext/sequence`
- [x] `variabletext/once`
- [x] `variabletext/cycle`
- [x] `variabletext/list-in-choice`
- [x] `variabletext/empty-elements`
- [x] `lists/basic-operations`
- [x] `lists/more-list-operations`
- [x] `lists/more-list-operations2`
- [x] `lists/list-mixed-items`
- [x] `lists/list-comparison`
- [x] `lists/list-range`
- [x] `lists/list-save-load`
- [x] `lists/list-all`
- [x] `lists/empty-list-origin`
- [x] `lists/empty-list-origin-after-assignment`
- [x] `lists/bug-adding-element`

## 10. Runtime, misc, tags, threads, tunnels

- [x] `runtime/external-function-0-arg`
- [x] `runtime/external-function-1-arg`
- [x] `runtime/external-function-2-arg`
- [x] `runtime/external-function-3-arg`
- [x] `runtime/jump-knot`
- [x] `runtime/jump-stitch`
- [x] `runtime/load-save`
- [x] `runtime/multiflow-basics`
- [x] `runtime/multiflow-saveloadthreads`
- [x] `runtime/read-visit-counts`
- [x] `runtime/saving-loading`
- [x] `runtime/set-get-variables`
- [x] `runtime/variable-observers`
- [x] `misc/operations`
- [x] `misc/read-counts`
- [x] `misc/turns-since`
- [x] `misc/issue15`
- [x] `misc/newlines_with_string_eval`
- [x] `misc/i18n`
- [x] `tags/tags`
- [x] `tags/tagsDynamicContent`
- [x] `tags/tagsInChoice`
- [x] `tags/tagsInChoiceDynamic`
- [x] `tags/tagsInSeq`
- [x] `threads/thread-bug`
- [x] `tunnel/tunnel-onwards-divert-override`

## 11. Current blocker watchlist

- [ ] `gather/gather-chain`
- [ ] `gather/nested-flow`
- [ ] `function/complex-func2`
- [ ] `function/evaluating-function-variablestate-bug`

## 12. Last-pass fixtures

- [ ] `TheIntercept`

## Notes

- Keep the work order simple: finish the current section before jumping to a
  harder one unless a blocker is clearly dependency-free.
- After a small batch turns green, add a commit and check off the finished
  entries here.
- When a fixture is hard, look for an easier neighboring fixture in the same
  section first so the change stays small.
- If the WritingWithInk order and a local fixture directory order disagree,
  follow WritingWithInk first.
