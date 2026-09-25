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

**Decide what can be measured, and say what you decided.** Nearly every choice here is settled by
reading a file, running a query or running an experiment, and doing that is the work. Asking about
one instead spends a round trip to arrive where the measurement already was. Present the decision
and the evidence for it; a reader who disagrees can say so, and that is cheaper than a question
nobody needed.

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

## The V-Model: what a requirement answers to, and how it changes

The README says which levels and links exist. This is what they commit a person to, when a
requirement is written and when one is changed. These rules are not negotiated per feature.

### What a requirement answers to

- **A requirement answers to its parents - the needs its `derived_from` names - and to nothing
  else.** It may have several; it then answers to each and is necessary for them. A stakeholder
  requirement answers to its stakeholder.
- **It states what its parents need of its level, and no more.** Every requirement's body answers
  two questions. *Could it be false while its parents hold?* - why it exists at all. *Could its
  parents hold while it is false?* - if so, it claims more than they need, and that surplus is
  exactly where some later, unrelated feature will collide with it.
- **It names a property, not the mechanism that delivers it today.** The mechanism is a decision,
  a `dec`, which later evidence may supersede; a requirement statement is not meant to move. Words
  that close the world - *only*, *each*, *every*, *exactly*, *no other*, a list of the mechanisms
  that happen to exist - are allowed where the parent closes it too, and nowhere else.
  (`EVD_AMENDMENTS_CAME_SIDEWAYS`: five of the first six statements ever changed here were
  changed for another stakeholder requirement's feature, each because it named the mechanisms of
  its day as the only ones.)
- **Ask it of every statement before it is committed:** would it read the same if no other
  stakeholder requirement existed?

### When a requirement may change

Changing an existing requirement's statement, EARS pattern, verification method, stakeholder, or
the links that place it - or removing it - is a **change request**. Rewording its body is not.

A change request has two possible justifications, and only two:

1. **its parent changed**, or
2. **it is wrong against its own parents** - it claims more or less than they need, or cannot be
   verified as written.

**The needs of other work are never a justification.** When new work collides with an existing
requirement, the question is not what the new work needs but whether the existing requirement is
right for its own parents:

- **If it is wrong for them**, it is corrected on their terms. The corrected statement must be the
  one its author would have written had the new work never been proposed; if the new work shows
  through in the wording, the correction is wrong.
- **If it is right for them**, the new work is designed around it.
- **If neither can give way**, two stakeholder requirements conflict. That is the stakeholder's to
  resolve, so stop and ask. The resolution is recorded at the stakeholder level first and flows down
  through the parent it affects - never sideways into another requirement's descendants.

Stakeholder requirements change only at their stakeholder's word. Decisions are different on
purpose: a `dec` is superseded by a new one naming it in `supersedes`, on new evidence, and that
is where design is allowed to move.

### The change procedure

A change request is carried out in this order, in one pull request, and the order matters:

1. **Name it in the plan**: the requirement, what raised it, and which of the two justifications
   applies.
2. **Run the impact analysis**: `sh scripts/impact.sh <ID>`. It prints four directions -
   **up**, the chain the requirement answers to; **down**, everything depending on it by a link,
   from derived requirements to code markers, test cases and their runs; **sideways**, what shares a
   component or an architecture with it without depending on it; and **text**, every line naming
   it in prose. Every row gets a verdict: unaffected, and why - or changing, and how.
3. **Analyse it against the parents alone**, in the form of stage 1 below: the parent's statement,
   what the requirement claims beyond or short of it, the options, one recommendation, the
   pitfalls. The trigger is context for the analysis, never its argument.
4. **Record it** in `docs/decisions/changes.rst`: a `dec` whose body names the requirement,
   gives the old and the new statement and carries the analysis and every verdict, `supported_by`
   an `evd` holding the impact analysis as run - its date, the command, the counts per direction.
   The changed requirement's body names the record.
5. **Change the requirement and everything the verdicts said changes**, in the same pull request -
   the requirements below it, their test cases and the code. Then re-run the test cases of every
   row, and the revert-proofs of those whose meaning moved.

`sh scripts/change-records.sh` holds the part of this a machine can: in the commit hook and in CI,
a change to a requirement's statement, pattern, method, stakeholder or placing links, or its
removal, fails unless the same range adds a line to `docs/decisions/changes.rst` naming it. It
cannot check that the analysis is any good - that is the review's - only that one was written.

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
- **A requirement changed sideways.** Every existing statement the branch changes -
  `git diff origin/main -- docs | grep '^-   :statement:'` - needs its change record, and the
  record's analysis has to stand on the requirement's own parents. The check sees that a record
  exists; only the review sees whether its argument is the new work in disguise.
- **A reason in a comment.** Every comment and docstring the diff adds, read
  against "Comments and docstrings" below: a sentence saying why the code is so,
  or what it answers to, belongs to a need and a marker. The gate sees an id;
  only the review sees a reason with no id in it.
- **Blast radius, not diff.** Which documents, rules, fixtures and gates reach the thing that
  changed. Editing `schemas.json` reaches every golden below the rule you touched, for the reason
  the README gives under "Changing a rule".
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

## Gates that go green without checking anything

Measured, not folklore. Each of these reports success while doing less than it appears to. The two
that live in the metamodel are in the README, under "Changing a rule"; these are the rest.

**`ubc check <path>` filters diagnostics; it does not scope the check.** The whole project is
resolved either way, and findings outside `<path>` are dropped. On a project carrying eight
diagnostics in `decisions/`, `check --deny warning stakeholder/context.rst` prints `No errors
found.` and exits 0. Passing a path does now print a notice that the whole project was resolved,
which is the only hint. Never pass a path when the question is "is the project clean" — use the
`( cd docs && ../tools/ubc check --deny warning )` subshell.

**A Cypher gate is vacuously green in more ways than it looks.** Without `--strict`, an unknown
label or property is a warning with an empty result and exit 0: `MATCH (n:not_a_real_type) RETURN
n.id` prints `[]` and exits 0, and exits 1 only under `--strict`. But `--strict` catches just the
unknown name — a well-formed query matching nothing still exits 0. **Pass `--strict` and count the
rows yourself.** A gate asserting "no need violates this" is otherwise indistinguishable from one
whose query could never match.

**`-c 'source.include=[…]'` resolves against the current directory, not against `--project`.** From
the repository root, `query cypher --project docs-selftest -c 'source.include=["fixtures/x.rst"]'`
matches zero needs and prints `[]` with exit 0, while the same command run from inside
`docs-selftest/` finds all of them. A scoped query is therefore run from inside its project, as
`scripts/cypher-gates.sh` does, and a scoped gate needs a planted offender to prove it matched
anything at all.

**Cypher's regex is not `schemas.json`'s regex.** `=~` matches the whole value, so
`n.statement =~ "shall"` returns nothing where `n.statement =~ ".*shall.*"` returns every
stakeholder requirement. `\b` does not work either: `n.id =~ ".*\bSELF.*"` matches zero needs
where `".*SELF.*"` matches one. A gate written with either mistake passes by matching nothing.

**A stray key in `ubproject.toml` is dropped without a word.** It produces no diagnostic and it
does not appear in `ubc config`. TOML also means a bare key appended at the end of the file belongs
to whatever table precedes it rather than to the document, so it can be both misplaced and ignored.
**Verify a config change by finding it in `ubc config` output**, never by the absence of complaints.

**Declaring an enum field is already a rule.** A value outside it is `needs.invalid_field_value` at
warning severity, and every gate here runs `--deny warning`, so it fails the build. Adding or
changing an enum value is a change that needs its own fixture, not a neutral edit.

## Comments and docstrings

A comment says what the code is; the graph says why it is so and what it
answers to (`STKH_REASONS_IN_THE_GRAPH`, `DEC_COMMENTS_SAY_WHAT`). There is no
length limit. The test is what a sentence says, and it is judged sentence by
sentence, using the table below.

| The text says | It goes |
| --- | --- |
| What an item does, what it takes, what it gives back, how it fails | Its docstring, in as many lines as that takes. |
| What a step inside a body does, or what holds at that point | A comment on it, usually one line, two or three when needed. |
| Why the code is this way: a choice between alternatives, a measurement behind it | A `dec` (with its `evd`), which the code's marker `follows`. Write the decision first if none says it yet. |
| Which requirement the code meets | The marker's `implements`. |
| An explanation this code alone needs, too long for a comment | A `code_note` in `docs/code/<crate>.rst`, which the marker `follows` (`DEC_NOTES_BESIDE_THE_CRATE`). |
| How this code relates to other code, a requirement, or a decision | The body of the need that owns the relation, not a note. |
| How an item does what it does | Nowhere in its docstring: that is what the item hides. A non-obvious step gets a comment where it happens. |

The marker is one line, on the line above the item, after its docstring:

    // @<title>,<IMPL id>,impl,[<component requirements>],[<decisions or notes>]
    // @<title>,<TRACE id>,trace,[],[<decisions or notes>]

`trace` is for code that follows a decision or note and meets no component
requirement (`DEC_TRACE_MARKERS`). No commas in the title: a comma drops the
marker without a word from ubc (`EVD_CODELINKS_COMMA_DROPS_MARKER`), which is
one of the two things `scripts/comment-rules.sh` refuses. The other is a need
id anywhere in a comment but a marker. An id in prose is a relation no query
can follow, so name it in the marker instead.

**A reason disguised as a description** is the case to watch for, because no
machine sees it. The tells: *so that*, *because*, *otherwise*, *rather than*,
*measured*, *was the alternative*, *without this*. Each usually starts a
sentence that belongs to a decision.

Before, from `crates/agconflo-lua/src/host.rs`:

    /// Model calls one activation's script may make (`DEC_MODEL_CALLS_COUNTED`).
    /// Awaiting a model costs no instructions, so without this a loop was
    /// measured making 2000 calls in one activation (`EVD_MODEL_CALLS_UNLIMITED`).
    pub model_calls: u32,

After: what the field is, and a link to why. The measurement is already in the
decision's evidence.

    /// Model calls one activation's script may make.
    // @Model calls limited per activation,IMPL_HOST_MODEL_CALL_LIMIT_FIELD,impl,[CREQ_HOST_MODEL_CALL_LIMIT],[DEC_MODEL_CALLS_COUNTED]
    pub model_calls: u32,

A long docstring is fine when it is all interface. This one keeps its length:

    /// Resume the run `record` is a record of, against `definition`.
    ///
    /// Returns where the run stopped, with the identifier source it came back
    /// with, advanced past everything the resumed run issued.
    ///
    /// Refused, before any script runs, when the record does not describe a run
    /// of `definition`, and for the scripts as a start is.

A body comment clarifies the code in front of it: `// The run's settled
contexts and each activation's in-progress ones, at once.` is a comment. `// It
takes a list of maps because the output would otherwise be refused` is a
decision, or a note if it is local to this code.

**Tests.** Why a test exists and what it catches is its `test_case`'s body. A
comment in a test says what a step sets up or asserts. A test with no
`test_case`, as in the tooling crates, keeps its reasons in a `code_note`.

**An example of an id's format is an id too.** The gate cannot tell
`TEST_LINEAGE_REACHES_EACH_ONCE` as an example from the same id as a relation,
and neither can a reader. Write the format: `TEST_` followed by the path.

**Moving an existing comment.** A crate's markers count only once it has a
codelinks project in `docs/ubproject.toml` and a `docs/code/<crate>.rst` with
its `src-trace`; without them its markers are ignored without a word. Nothing
is deleted until what it says is found in a need, or moved into one in the
same commit. A comment that restated a
decision loses the restatement and gains the marker link; a reason that no
need holds becomes a decision first. `sh scripts/comment-rules.sh --report`
lists the long blocks to start from, and `--crate <name>` shows what a crate
would be refused for before it is added to the script's `HELD`.

## Commits and branches

- One branch per unit of work: `infra/<topic>` for tooling and process, `step/<feature>/<slice>`
  for work through the V. One pull request, **squash-merged**, and the branch is kept afterwards so
  the commit-by-commit reasoning survives the squash.
- `main` is protected and a direct push is rejected, so every change lands through a pull request.
  The exact rules, and the one fork coupling worth knowing, are in the README.
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
- an existing requirement that is right for its own parents cannot hold beside new work - two
  stakeholder requirements conflict, and resolving that is the stakeholder's;
- the work needs a decision the plan has not pinned **and that no measurement can settle**;
- anything is about to be pushed, opened or merged.

## Never

- Never change an existing requirement because other work needs it changed. Its parents are its
  only grounds, and the change procedure is the only way.
- Never write a metadata `feat_req`. It has no honest parent, and pointing it at an existing
  stakeholder requirement would be a link true enough to pass and not true. Nothing in the
  toolchain can catch that.
- Never bless golden files without reading the diff. Blessing blindly turns a broken rule into an
  expectation, which is the one way this harness fails while staying green.
- Never weaken a rule to make your own material pass.
- Never add a coverage rule. It fires on everything not yet built and blocks the first step of
  authoring; coverage is a query here.
- Never commit a secret. This repository is public from its first commit, so a leak is permanent.
- Never let a test, a probe or a live run use a provider key found in the environment -
  `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` or any other - unless the maintainer allows it for that run.
  `genai` reads each adapter's own variable without being asked (`EVD_GENAI_KEY_CHECKED_AT_CALL`),
  so remove those variables from anything that could reach one. Tests answer with a loopback stub,
  and a live run uses a model already loaded in LM Studio.
- Never commit generated output except where a plan explicitly says to.
