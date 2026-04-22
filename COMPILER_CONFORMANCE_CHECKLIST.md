# compiler_conformance_legacy Checklist

This is the working queue for the imported legacy compiler-conformance suite.
The order is intentionally dependency-light first:

1. minimal text and simple flow/export
2. simple diverts and glue
3. knots and stitches
4. basic choices
5. choice edge cases
6. gather and nested flow
7. conditionals and sequences
8. functions
9. variables, variable text, and lists
10. runtime / misc / tags / threads / tunnels
11. special bug fixtures

Legend:

- `[x]` done
- `[ ]` pending

## 1. Minimal text and simple flow/export

- [x] `basictext/oneline`
- [x] `basictext/twolines`
- [ ] `TheIntercept`
- [ ] `test1`

## 2. Simple diverts and glue

- [x] `divert/simple-divert`
- [ ] `divert/invisible-divert`
- [ ] `divert/divert-on-choice`
- [ ] `divert/complex-branching`
- [x] `glue/simple-glue`
- [ ] `glue/glue-with-divert`
- [ ] `glue/left-right-glue-matching`
- [ ] `glue/testbugfix1`
- [ ] `glue/testbugfix2`

## 3. Knots and stitches

- [ ] `knot/single-line`
- [ ] `knot/multi-line`
- [ ] `knot/strip-empty-lines`
- [ ] `knot/param-ints`
- [ ] `knot/param-floats`
- [ ] `knot/param-strings`
- [ ] `knot/param-vars`
- [ ] `knot/param-multi`
- [ ] `knot/param-recurse`
- [ ] `stitch/auto-stitch`
- [ ] `stitch/auto-stitch2`
- [ ] `stitch/manual-stitch`
- [ ] `stitch/manual-stitch2`

## 4. Basic choices

- [ ] `choices/no-choice-text`
- [ ] `choices/one`
- [ ] `choices/multi-choice`
- [ ] `choices/single-choice`
- [ ] `choices/suppress-choice`
- [ ] `choices/mixed-choice`
- [ ] `choices/varying-choice`

## 5. Choice edge cases

- [ ] `choices/fallback-choice`
- [ ] `choices/fallback-choice2`
- [ ] `choices/conditional-choice`
- [ ] `choices/label-flow`
- [ ] `choices/label-flow2`
- [x] `choices/label-scope`
- [ ] `choices/label-scope-error`
- [ ] `choices/divert-choice`
- [x] `choices/sticky-choice`

## 6. Gather and nested flow

- [ ] `gather/gather-basic`
- [ ] `gather/gather-chain`
- [ ] `gather/nested-gather`
- [ ] `gather/nested-flow`
- [ ] `gather/deep-nesting`
- [ ] `gather/complex-flow`

## 7. Conditionals and sequences

- [ ] `conditional/iftrue`
- [ ] `conditional/iffalse`
- [ ] `conditional/ifelse`
- [ ] `conditional/ifelse-ext`
- [ ] `conditional/ifelse-ext-text1`
- [ ] `conditional/ifelse-ext-text2`
- [ ] `conditional/ifelse-ext-text3`
- [ ] `conditional/condtext`
- [ ] `conditional/condopt`
- [ ] `conditional/cycle`
- [ ] `conditional/once`
- [ ] `conditional/shuffle`
- [ ] `conditional/shuffle_once`
- [ ] `conditional/shuffle_stopping`
- [ ] `conditional/stopping`
- [ ] `conditional/multiline`
- [ ] `conditional/multiline-divert`
- [ ] `conditional/multiline-choice`

## 8. Functions

- [ ] `function/func-none`
- [ ] `function/func-basic`
- [ ] `function/func-inline`
- [ ] `function/setvar-func`
- [ ] `function/rnd-func`
- [ ] `function/complex-func1`
- [ ] `function/complex-func2`
- [ ] `function/complex-func3`
- [ ] `function/evaluating-function-variablestate-bug`
- [ ] `function/test-error`

## 9. Variables, variable text, and lists

- [ ] `variable/variable-declaration`
- [ ] `variable/varcalc`
- [ ] `variable/varstringinc`
- [ ] `variable/var-divert`
- [ ] `variabletext/sequence`
- [ ] `variabletext/once`
- [ ] `variabletext/cycle`
- [ ] `variabletext/list-in-choice`
- [ ] `variabletext/empty-elements`
- [ ] `lists/basic-operations`
- [ ] `lists/more-list-operations`
- [ ] `lists/more-list-operations2`
- [ ] `lists/list-mixed-items`
- [ ] `lists/list-comparison`
- [ ] `lists/list-range`
- [ ] `lists/list-save-load`
- [ ] `lists/list-all`
- [ ] `lists/empty-list-origin`
- [ ] `lists/empty-list-origin-after-assignment`
- [ ] `lists/bug-adding-element`

## 10. Runtime, misc, tags, threads, tunnels

- [ ] `runtime/external-function-0-arg`
- [ ] `runtime/external-function-1-arg`
- [ ] `runtime/external-function-2-arg`
- [ ] `runtime/external-function-3-arg`
- [ ] `runtime/jump-knot`
- [ ] `runtime/jump-stitch`
- [ ] `runtime/load-save`
- [ ] `runtime/multiflow-basics`
- [ ] `runtime/multiflow-saveloadthreads`
- [ ] `runtime/read-visit-counts`
- [ ] `runtime/saving-loading`
- [ ] `runtime/set-get-variables`
- [ ] `runtime/variable-observers`
- [ ] `misc/operations`
- [ ] `misc/read-counts`
- [ ] `misc/turns-since`
- [ ] `misc/issue15`
- [ ] `misc/newlines_with_string_eval`
- [ ] `misc/i18n`
- [ ] `tags/tags`
- [ ] `tags/tagsDynamicContent`
- [ ] `tags/tagsInChoice`
- [ ] `tags/tagsInChoiceDynamic`
- [ ] `tags/tagsInSeq`
- [ ] `threads/thread-bug`
- [ ] `tunnel/tunnel-onwards-divert-override`

## 11. Current blocker watchlist

- [ ] `gather/gather-chain`
- [ ] `gather/nested-flow`
- [ ] `function/complex-func2`
- [ ] `function/evaluating-function-variablestate-bug`

## Notes

- Keep the work order simple: finish the current section before jumping to a
  harder one unless a blocker is clearly dependency-free.
- After a small batch turns green, add a commit and check off the finished
  entries here.
- When a fixture is hard, look for an easier neighboring fixture in the same
  section first so the change stays small.
