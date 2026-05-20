# Active Plan Workflow

Use an active plan when a change is large enough that implementation order,
review checkpoints, and durable task state matter. Ordinary small changes can
use normal implementation commits without creating an active plan.

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

## Task List Requirements

- Task lists under `docs/active_plan/` must be implementation plans, not
  research logs.
- A task is not valid if it is only read-only inventory, planning, or context
  gathering. Put necessary inventory inside the implementation method of the
  first task that uses it.
- Do not write task lists as one-line task tables. Each task must have its own
  section with enough detail for another implementer to execute it without
  guessing: goal, implementation method, acceptance criteria, forbidden
  shortcuts, modification boundaries, validation commands, and commit record.
- The first non-blank line of every task list must be a progress indicator in
  `Progress: X/N` form.
- Every task must be represented by a status marker in its task heading, such
  as `### [ ] Task 01: ...`. Do not rely only on table status columns.
- Use these task heading markers: `[ ]` pending, `[~]` in progress, `[>]`
  waiting review, `[x]` complete, and `[!]` blocked.
- `[>]` means the implementation commit exists after focused validation and
  `make gate`, and the next required work is reviewing that commit plus any
  fixes.
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

## Validation And Commit Flow

- Every active-plan task must pass focused validation relevant to the changed
  surface and then `make gate` before its implementation commit can be treated
  as waiting review.
- Do not add documentation-only or read-only exceptions to task completion,
  except for the final plan closeout task after implementation is already
  validated.
- Keep the progress counter, status key, acceptance conditions, forbidden
  shortcuts, and modification boundaries in the task list itself.
- Update task status and progress only after validation has passed.
- Record validation and commit metadata inside the relevant task section. Do
  not append separate notes, logs, journals, or running commentary after the
  task list.
- Commit implementation code separately from task-list progress records.
- After implementation validation passes, commit the code and move the task to
  `[>]` waiting review.
- The next task-list action after `[>]` is to review that implementation commit,
  fix issues in follow-up code commits if needed, rerun focused validation and
  `make gate`, then mark the task `[x]`, update `Progress: X/N`, and commit the
  completion record separately.
- Do not start the next implementation task until the prior task has passed
  review, any fixes have been committed, and the task-list completion record
  has been committed.
