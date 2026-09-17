# Working on Agconflo

The README is the mechanical reference: prerequisites, setup, every gate command, the commit hook,
branch protection, the requirements metamodel and where files go. Read it first, and do not restate
it here — one copy is the only copy that stays true.

This file is the working mode: how a change gets from a request to a merged commit. It applies to
any agent or contributor working in this repository.

The short version: **plan before implementing, gather facts from the files rather than from
recollection, verify by running rather than by reasoning, and review your own diff before
committing.**

## The cycle

1. A change is requested.
2. **Gather facts from the repository first.** Read the rule as it stands in `schemas.json`, run
   the query, check what `ubproject.toml` actually declares. Never plan from a remembered account
   of how something works.
3. **Present a plan** in the format below, and wait for approval.
4. **Implement as a series of commits**, one at a time, each independently verifiable, with the
   gates green between each.
5. **Review your own diff before each commit.** A green gate is the input to that review, not a
   substitute for it.
6. **Report with evidence** — real command output, not claims.

Do one commit at a time: read its brief, do exactly that, verify it, report back. Do not run ahead
into the next commit because it looks small.

## Planning

Analyse the options from the files and the data, then present **one recommendation per decision**
with the alternatives and concretely why each lost. A plan that ends in a list of open questions
hands back the work it existed to do. Where a decision genuinely cannot be made from what is in the
repository, that is a stop-and-ask, not a bullet list.

### The plan format

- **Lead with anything that changes scope**, or that might be cut. A fact discovered while
  gathering that invalidates part of the request belongs in the first paragraph, not an appendix.
- **A decision table**: `# | Decision | Recommended | Alternatives considered`. One recommendation
  per row, never a question. The alternatives column says *why* each was rejected, concretely — "an
  enum change needs its own fixture", not "that seemed worse".
- **Reasoning from this repository**, with numbers read out of it rather than recalled.
- **Pitfalls**, numbered, each naming the concrete failure it prevents.
- **Build order as N commits**, each independently verifiable.
- **Tests**: positive briefly, error-path at length, with the reason each case matters.
- **A self-evaluation**: what is most likely to be wrong, what cannot be verified, what is most
  likely to grow in scope.
- End by inviting approval or a cut.

Keep it tight. Tables and short paragraphs; padding costs the reader time and gains nothing.

**Experiments are part of planning, not a failure of it.** When a detail is uncertain, run
something and roll it back — a throwaway fixture, a query, a scratch clone. Replanning because an
experiment contradicted the plan is the normal outcome, not a mistake to hide.

## Changing behaviour: two stages

When a defect is reported or a behaviour has to change, the thinking comes first and the code
second, as two separate turns.

**Stage 1 — analyse. Change nothing.**

- Gather the facts from the files first. This is what reveals that a fix has a different shape than
  the report implied.
- Several real options, each with why it would or would not work *here*.
- **One recommendation**, with the alternatives and concretely why they lost.
- **Pitfalls**, numbered.
- **Which tests to add** — positive, defect-catching and error-path, named correctly (below).
- Self-evaluation: what is expected to be wrong, and what cannot be verified.

**Stage 2 — implement, only after approval.**

**Stage 3 — review the diff before committing.** Standing, and not something to wait to be asked
for.

**Bundling is expected.** Three defects means stage 1 for all three in one message, then one
approval covering the batch. Do not interleave — no implementing the first while the second is
still unapproved.

## Before every commit: review your own diff

Run this with the gates already green. Green is what makes the review necessary, not what makes it
unnecessary.

1. **Read the diff as if auditing a stranger's.** `git diff` and `git status --porcelain`, every
   file, fixtures and goldens included.
2. **Write 8-12 falsifiable hypotheses about what was just introduced**, each stated as the
   *defect* rather than the topic. "The new rule might be wrong" is useless; "the new rule's
   composite keyword sits directly under `validate.local`, so it is ignored and the project goes
   green" is testable. List them all before verifying any.
3. **Verify each by running.** Reason only to choose the experiment, never to reach the verdict.
4. **Fix what is confirmed — then review the fixes too.** A fix regresses.
5. **Report the split honestly**, and correct any severity claim from the first pass that the
   evidence does not support.

Categories that produce hits here:

- **A claim nothing verifies.** A comment, a commit body or a need's body asserting behaviour that
  no rule, fixture or gate checks. The richest seam by far.
- **A gate that cannot fail.** A rule of the wrong shape, a Cypher query without `--strict`, a
  `check` given a path: each is green and empty for its own reason, and they look identical from
  outside.
- **A golden that moved for a reason nobody predicted.** A blessed diff nobody read is how a broken
  rule becomes an expectation.
- **What the change quietly stopped doing.** A correct fix can remove a behaviour something else
  depended on by accident, and no test of the fix can show that — its tests assert the new
  behaviour.
- **Blast radius, not diff.** Which documents, rules, fixtures and gates reach the thing that
  changed. A rule's array index appears in every message it produces, so editing `schemas.json`
  reaches every golden below it.
- **Mechanical.** Line endings, a bare key landing inside the previous table of a TOML file, a
  document added but never listed in the toctree.

## Tests

Three kinds, all expected, none an alternative to the others. **`test_kind` declares the first and
the third. The second is a proof technique, not a need type — do not add an enum value for it.**

| Kind | What it asserts | When it must fail |
| --- | --- | --- |
| **Positive** | valid input produces the right result | never |
| **Defect-catching** | the specific defect is gone | **against the pre-change code** — that is the proof below |
| **Error-path** (negative) | invalid, hostile or degenerate input fails **in the documented way** | never — it must pass against the final code |

A test that fails against the old code is valuable and required, but it is *not* the error-path
half and does not discharge that requirement. Calling it one is how the real kind goes missing.

Error-path means asserting **which** failure occurred and how things behave afterwards, not merely
that something failed. A failure mode nobody asserts on is a failure mode nobody designed, which is
why failure modes are enumerated when the `comp_req` is written rather than at test time.

Property-based tests are preferred wherever an invariant can be stated instead of a table of cases.
`proptest-regressions/` is committed, never ignored: it holds the seeds of found bugs.

### Proving a test catches its defect

A green test can guard nothing, and that is the failure mode to fear most. Prove it:

1. Revert **only the wired-in part** of the change, keeping any new helper or fixture.
2. Re-run. It must fail, with the symptom described.
3. Restore, re-run, and it must pass.

Deleting the helper as well gives a collection or parse error, which proves only that the test file
loads. For a rule in `schemas.json`, **neuter its `select`** rather than deleting the entry — the
array index is load-bearing.

**A proof that does not bite may be a broken revert rather than a safe property.** Check the edit
landed before concluding anything, and remove the *mechanism* rather than its name: renaming a
block's keyword can leave everything inside it in force and change nothing observable.

**Report the split.** Most tests around a narrowing change pass against the broken code on purpose,
because they exist to catch a change that is too wide. "4 of 15 fail; the other 11 are controls
proving it did not over-block" is the sentence that matters.

### Re-run every proof against the final state

A proof taken at commit 2 says nothing about the tree at commit 6: a later commit can change a
shared fixture and turn a defect-catching test into one that passes for a new reason. Re-run them
all at the branch head before calling it finished. That is a measurement rather than a deduction,
and it costs minutes.

## Verification techniques

- **Probe before asserting.** Run the thing over a spread of inputs, read what actually happens,
  then write assertions against that. Reasoning alone produces confident wrong answers.
- **Suspect a surprisingly clean result.** Zero rows, zero diagnostics and exit 0 are what both
  "nothing is wrong" and "nothing ran" look like. Count the rows.
- **A positive control makes a negative result credible.** "No diagnostic appeared" means something
  only once a case that *should* produce one has been seen to produce it.
- **Verify the stimulus fired**, not just that the outcome looks right. "Nothing was sent" and "it
  was sent and was harmless" produce identical readings. Better still, replace a stimulus you do
  not control with one you do.
- **A correction to a method invalidates every conclusion that method produced**, not only the one
  that looked odd. On finding a timing, control or fixture problem, list what else that run
  concluded and re-run those too.
- **Isolate which layer failed.** When something reports an error, establish *who* reported it
  before theorising about why.
- **Re-measure a recorded measurement before building on it**, and widen the shapes rather than
  re-running the same ones. A recorded measurement says what somebody looked at, not what is true;
  one lucky sample is not a property.
- **Check that a faster version gives the same answers before measuring how much faster it is.**
- **Distinguish "the test was wrong" from "the code is wrong", out loud and immediately.**
- **Dates come from `git log` or the environment, never from a note.** A fabricated date is well
  formed, so no rule catches it. One propagated through thirteen places here and reached a
  queryable field, claiming a measurement taken in the future.

## Commits and branches

- One branch per unit of work: `infra/<topic>` for tooling and process, `step/<feature>/<slice>`
  for work through the V. One pull request, **squash-merged**, and the branch is kept afterwards so
  the commit-by-commit reasoning survives the squash.
- `main` is protected with no admin bypass: both CI jobs must pass, commits are signed, history
  stays linear, and a direct push is rejected.
- **Never override the configured git identity.** No `-c user.email=...`, ever. The repository
  carries a signing setup; commit plainly and let it apply. If an identity looks wrong, ask.
- **Subject: a capitalised imperative saying what changed in plain words** — "Stand up the
  documentation toolchain". No prefix, no conventional-commits tag.
- **The body explains why, not what**: what was rejected and for what concrete reason, what was
  measured, what consequence follows. State what was verified and how, with numbers. Own defects
  found in your own work in the commit that fixes them. Squashing compresses the history, not the
  reasoning.
- Two commits are right when the plan said two. "One commit per change" means "not the five it took
  to build it", not "never two deliberate pieces".
- **Stage explicit paths** rather than everything, so scratch files cannot ride along.
- **Never use `git commit --no-verify` to get past a gate.**
- **Nothing is pushed without explicit approval.** An instruction to implement is not an
  instruction to push, and approval does not carry from one commit to the next. When a commit is
  ready and unpushed, say so and stop.

## Reporting

- **Lead with anything that went wrong**, anything destructive, or anything that needs a decision.
  Do not bury it under the good news.
- Tables for results; fenced blocks holding **real output**, not paraphrase.
- Report failures with their output rather than summarising them away.
- An explicit section for what was got wrong, in plain language. Hedging reads worse than the
  mistake does.
- End with the current state — branch, commit, which gates were run — and what comes next.

## Stop and ask when

- a wording rule rejects a statement believed to be correct, since changing a rule is a decision
  rather than a workaround;
- a golden file moves that was not predicted;
- a requirement cannot be written without inventing a new `stkh_req` parent;
- the work needs a decision the plan has not pinned;
- anything is about to be pushed, opened or merged.

## Never

- Never write a metadata `feat_req`. It has no honest parent, and pointing it at an existing
  stakeholder requirement would be a link true enough to pass and not true. Nothing in the
  toolchain can catch that.
- Never bless golden files without reading the diff. Blessing blindly turns a broken rule into an
  expectation, which is the one way this harness fails while staying green.
- Never weaken a rule to make your own material pass.
- Never add a coverage rule. It fires on everything not yet built and blocks the first step of
  authoring; coverage is a query here.
- Never commit a secret. This repository is public from its first commit, so a leak is permanent.
- Never commit generated output except where a plan explicitly says to.
