# Active Plan Workflow

Use an active plan when a change is large enough that implementation order,
review checkpoints, and durable task state matter. Ordinary small changes can
use normal implementation commits without creating an active plan.

Active plans are goal-driven, not commit-driven. When a project goal is active,
use that goal as the top-level marker for completing the active plan. Use the
active plan task list as the durable progress ledger below the goal.

The task list is the progress unit below commit granularity. Do not create
commits merely because a task is complete. Create checkpoint commits only when
they help manage implementation risk, local review, or recovery from a large
temporary diff.

## Location

- Store active implementation plans under `docs/active_plan/`.
- Keep active plan details in that directory; do not add one-off plan file
  names to `AGENTS.md`.
- When an active plan is complete, move its plan directory from
  `docs/active_plan/` to `docs/finished_plans/`.

## Continuation

If the user sends a continuation prompt such as `continue`, `go on`, `keep
going`, `next`, or a localized equivalent without replacing the task, continue
the active implementation plan in `docs/active_plan/` if one exists. Otherwise,
continue the current project goal from repository state and durable notes.

Active plan directories may include a progress file. Keep that progress file
synchronized while development proceeds, following the plan's own convention.
If no separate progress file exists, keep progress inside the task list itself.

## Task List Requirements

- Task lists under `docs/active_plan/` must be implementation plans, not
  research logs.
- A task is not valid if it is only read-only inventory, planning, or context
  gathering. Put necessary inventory inside the implementation method of the
  first task that uses it.
- Do not write task lists as one-line task tables. Each task must have its own
  section with enough detail for another implementer to execute it without
  guessing: goal, implementation method, acceptance criteria, forbidden
  shortcuts, modification boundaries, validation commands, progress ledger
  fields, and optional checkpoint commit notes.
- The first non-blank line of every task list must be a progress indicator in
  `Progress: X/N` form.
- Every task must be represented by a status marker in its task heading, such
  as `### [ ] Task 01: ...`. Do not rely only on table status columns.
- Use these task heading markers: `[ ]` pending, `[~]` in progress, `[>]`
  implemented and validated but waiting owner review, `[x]` complete, and `[!]`
  blocked.
- `[>]` does not require a commit. It means the implementation is present in the
  working tree or in optional checkpoint commits, focused validation has passed,
  and the next required work is owner review plus any fixes.
- Group related tasks under milestone sections so parser, analysis, lowering,
  runtime, fixture, and documentation work are easy to navigate.
- Prefer task lists with more than 20-25 tasks and fewer than 100 tasks. Split
  large milestones into reviewable implementation tasks, but do not split out
  read-only inventory or planning-only tasks.

## Valid Task Scope

A valid task must produce a reviewable repository change that includes
production code, test code, fixtures, editor assets, or another code-adjacent
artifact. Maintained documentation should be updated in the same task when
behavior changes, but pure prose-only tasks are not valid implementation tasks
except for final plan closeout.

Task scope should be large enough to make meaningful progress and small enough
to review safely. Prefer a cohesive vertical slice that adds tests,
implementation, diagnostics, docs, and migration updates together when those
pieces are required for the behavior to be correct.

## Progress Ledger

The task list is the progress tool below commit granularity. Keep it useful for
LLM continuation after context loss.

Each task should record, inside that task section:

- changed files or directories
- focused validation commands and results
- owner review state when review is needed
- remaining risks or follow-up notes
- optional checkpoint commit hashes, when checkpoint commits exist

The ledger must be specific enough that another LLM can resume without relying
on commit history. A checkpoint commit hash can support the ledger, but it must
not replace the changed-files, validation, risk, and status fields.

Do not append separate running commentary after the task list. Put durable
execution state into the relevant task section or into the plan's explicit
progress file.

## Validation And Completion Flow

- Every active-plan task must pass focused validation relevant to the changed
  surface before it can be marked `[>]` or `[x]`.
- `make gate` is required before finishing the active plan. It may also be run
  at milestone boundaries or whenever the implementer judges that accumulated
  changes need wider coverage.
- Do not add documentation-only or read-only exceptions to task completion,
  except for the final plan closeout task after implementation is already
  validated.
- Keep the progress counter, status key, acceptance conditions, forbidden
  shortcuts, modification boundaries, and ledger fields in the task list itself.
- Update task status and progress only after the required focused validation has
  passed or the blocking condition is recorded.
- A task can move from `[~]` directly to `[x]` when implementation, focused
  validation, task-ledger updates, and any required owner review are complete.
- A task can move to `[>]` when implementation and focused validation are done
  but owner review is still needed.
- Do not start the next implementation task until the prior task is `[x]`,
  unless the task list explicitly records why limited overlap is safe.

## Commit Flow

Active plans do not require one commit per task.

- Let the LLM or implementer decide when to create optional checkpoint commits
  to avoid an oversized temporary diff, preserve a recovery point, or separate
  risky work for local review.
- Checkpoint commits are development aids, not required task boundaries.
- A checkpoint commit may include several completed tasks, part of a task, or a
  milestone slice, as long as the task ledger accurately records what is
  implemented and validated.
- Before the active plan is pushed or considered landed, combine all commits
  belonging to the active plan into one final active-plan commit unless the
  project owner explicitly asks for a different history shape.
- The final active-plan commit is the durable review unit. It should be created
  only after final required validation passes.
- The final commit should include the implementation, tests, documentation, task
  ledger, and the move from `docs/active_plan/` to `docs/finished_plans/` when
  the plan is complete.
- If no checkpoint commits were created, create one final commit after the final
  validation passes.
