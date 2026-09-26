# Disclaimer: This project is 100% vibe-coded. If you don't like this, please don't waste your time and don't use it.

# Agconflo

***Ag**ent **con**text **flo**w* — a workflow engine for AI pipelines whose single organising concern
is **context management**.

Nodes are wired into a graph. Each node consumes zero or more immutable `Context` values and produces
exactly one output. Context is a first-class, immutable, addressable value, and **a node sees exactly
what was wired to it — nothing else**.

That is the deliberate opposite of frameworks where state is one mutable shared blob and what actually
lands in an LLM's context window is emergent. Here, *"what was in this call's context window, and
where did every byte come from?"* has an exact answer.

## Status

**Pre-alpha. Nodes run as scripts, call models or wait for a person, a model can call another node
type, a run survives an interruption, a person runs a workflow from its documents with the
`agconflo` command, and a node type can be a tool — reading a file, writing one, running a
command — kept in a container to what that person granted** — no loops, and no MCP yet. What exists is the
development process around it and the first slices of code through it: a requirements project
under `docs/` with a validated metamodel behind it, a commit gate, continuous integration, three
library crates and a command.

`agconflo-core` implements the `Context` value itself — identity, composition by reference, and
lineage — the static side of a workflow, and a run over one. The static side is checking its wiring
before anything runs, and reading it from TOML documents and writing it back without losing what the
reader did not understand. The run decides which node may activate and what it is given, refuses an
output that contradicts its declaration, holds it to a budget, and ends in exactly one of four ways;
**its caller performs the activation**, so the core reaches no provider and owns no runtime. A run can
be written down as text at any point and resumed from that text in another process, holding the
contexts it held under the identifiers they had; the core stores nothing, and where the text is kept
is its caller's.

`agconflo-lua` is such a caller. It performs each activation by running a Lua 5.5 script supplied
for the node type, in a state made for that one activation, with nothing to reach but its inputs and
the context API, and under a limit on instructions, memory and model calls. A script calls a model
by a role - `host.complete('drafting', prompt)` - and the caller maps each role to a provider's model
through [`genai`](https://github.com/jeremychone/rust-genai), so the same workflow runs against
another provider by changing that mapping. What a model is shown is contexts and nothing else, and
its answer comes back as a context of its own. A model may call the node types its instance
declares, offered to it as tools: each call runs as a step of the run - counted against the budget,
performed by that node type's own script or by a person - and the model goes on with its output. A
scripted run hands its caller a record when it starts, after every output and after every answer,
so a run interrupted mid-call is resumed without asking again for any answer its record holds. A
node type can instead be performed by a person: the run stops at that step and hands it to
its caller, who answers later - from the run's record, in another process if need be - with text that
becomes the step's output.

`agconflo-runner` is the caller a person does not have to write. It reads a manifest naming a
workflow's documents and scripts, and a model mapping naming the model each role is played by,
and starts, resumes, answers or checks a run from them, keeping its record in a file that one run
at a time holds. A node type the manifest names as a tool it performs itself, in a locked-down
container holding only the folders a grants file allows. `agconflo-cli` is its command line, `agconflo`, described below. Each crate is
traced from its requirements to the code and back from the tests that check it.

Expect the public API to change without warning. Breaking changes, yes; force-pushes to `main`, no —
those are blocked outright, along with direct pushes to it.

## Running a workflow

A workflow is its documents: a workflow document, its node type documents and a Lua script for
each node type a script performs. A **manifest** names them, each by a path relative to the
manifest itself:

```toml
workflow = "flow.toml"
types = ["types.toml"]
budget = 10                 # the most activations a run may make
persons = ["review"]        # node types a person performs

[scripts]
draft = "draft.lua"

[limits]                    # optional, each of the three
model_calls = 2
```

Which model plays each role is the machine's, so it is in a **model mapping** of its own. A model
is always named with its provider, and a key is only ever taken from the variable the mapping
names; nothing reads `ANTHROPIC_API_KEY` or its like by default:

```toml
[roles]
drafting = "anthropic::claude-sonnet-5"
asking = { model = "openai::qwen3.8-27b-ridge", endpoint = "http://localhost:1234/v1/", key_env = "LM_API_TOKEN" }
```

```
agconflo check  manifest.toml --models models.toml
agconflo run    manifest.toml --record run.toml --models models.toml --arg subject brief "a lighthouse"
agconflo answer manifest.toml --record run.toml --models models.toml --instance reviewed --text "Approved."
agconflo resume manifest.toml --record run.toml --models models.toml
```

`run` refuses a record file that already exists, and `run`, `resume` and `answer` each hold the
record file for as long as they run, so two people cannot answer one step at once. A completed run's result, or the
step a run awaits, is all that goes to standard output. The exit status says how the run stopped:
0 completed, 2 a command line it cannot read, 3 awaiting a person, 4 refused before anything ran,
5 a node failed, 6 the budget ran out, 7 no node can make further progress, and 8 the record could
not be kept, whatever else happened.

### Tools

A node type can be a **tool**: it reads a file, writes one, or runs a command, in a Docker container
holding only what the person running the workflow granted. The manifest says which node types are
tools and what each does, beside its scripts and persons. Each action takes parameters of fixed
names, which the node type declares: `read` takes `path`, `write` takes `path` and `text`, and
`run` takes `command`. A model calls a tool as it calls any node type its instance declares.

```toml
[tools]
read_file = "read"
write_file = "write"
run_tests = { action = "run", image = "python@sha256:cea0e6040540fb2b965b6e7fb5ffa00871e632eef63719f0ea54bca189ce14a6", container = "py" }
```

A tool may name the **image** it runs in, by digest, and the **container** it runs in, by name. Tools
naming one container share it, and what a step leaves outside the granted folders - in `/tmp`, in
its home - the next step there sees, within one `run`, `resume` or `answer`; a tool naming another
container sees none of it. A tool naming neither runs in the grants' image, in a container every such
tool shares, `tools`. A container has one image, so a tool naming its own image needs a container of
its own too.

What the tools may do is the person's to say, in a **grants file** of its own, given with `--grants`
to every command, `check` included:

```toml
image = "alpine@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6"
images = ["python@sha256:cea0e6040540fb2b965b6e7fb5ffa00871e632eef63719f0ea54bca189ce14a6"]
actions = ["read", "write", "run"]   # none unless named
network = false                      # the default
trust = "ca.pem"                     # certificates a step trusts, if any

[folders.project]                    # mounted at /work/project
path = "../work"                     # from this file's directory
writable = true                      # read-only unless true

[limits]                             # each optional
seconds = 60                         # the time a command may run
output = 16384                       # the bytes a step gives back
tmp = 268435456                      # the bytes /tmp may hold
```

The grants' `image` is where a tool naming none runs, and `images` lists the others a tool may
name; a tool naming an image outside both is refused. Every image is named by its digest or an image
id and **never pulled**: pull it yourself, and a run whose image is absent is refused. A path a tool is given names a granted folder and a place in it,
`project/src/main.c`, and a command runs in `/work`, where each folder sits under its name; an
absolute path, or one naming a parent folder, is refused. What goes wrong in a tool — a file not
there, a command exiting 1 — is the step's output, for the model to act on. Each step runs with no
capabilities, as your user on Linux, root included, and as 1000 elsewhere, with `/tmp` as its home,
under the time limit, with everything it started killed after it, and output past the limit keeps
its start and its end. `/tmp` holds no more than `tmp`, and a step that fills it is told its output
may have been lost. Each container is named `agconflo-<run>-<name>`, where `docker ps` shows it, and
removed when the `run`, `resume` or `answer` that made it stops. The container keeps a mistaken
command from what was not granted; it is no defence against code built to escape one.

A network where outgoing TLS is intercepted, as a proxy with its own certificate authority does,
refuses a container's connections unless it trusts that authority. `trust` names a file of
certificates, read from the grants file's directory, and a file that cannot be read refuses the
grants. Its content is copied into each container's `/tmp` before the container's first step, and
every step is told of the copy through `SSL_CERT_FILE`, which `ubc` was measured reading; nothing is
mounted for it.

**Grant a git worktree, not your checkout.** Inside a writable folder a tool may change anything,
mistakes included. `git worktree add ../work -b tool-run` gives it a copy whose every change
`git diff` shows, and `git worktree remove` discards. Run as root, what a tool writes is root's, a
setuid bit it sets included, which `git diff` does not show and `find ../work -perm -4000` does.
A tool that runs `ubc` needs a clone instead, `git clone . ../work`: ubc refused a worktree in a
container even with the git directory it names mounted.

A run whose manifest names a tool is refused before anything runs when its grants are missing,
cannot be read or do not cover a tool, or an image it needs is absent. If Docker cannot be reached,
or fails while a step is performed, the run stops awaiting the tool step it reached, exit status 3,
with the reason on standard error: `agconflo resume` performs it again once Docker is back, and
`agconflo answer` supplies its output at once. `agconflo check` reports Docker out of reach.

```
agconflo run manifest.toml --record run.toml --models models.toml --grants grants.toml --arg-file given brief task.txt
```

## Planned shape

- **Rust**, as a Cargo workspace. `agconflo-core` is a normal Rust crate with a deliberately strict
  public API; frontends are separate crates that depend on it. The command line is the first; a
  viewer and an MCP server are planned.
- **Provider-agnostic LLM access** via [`genai`](https://github.com/jeremychone/rust-genai), now
  running, and MCP via [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk). The agent loop is
  ours — that is the layer this project exists to own.
- **Node behaviour in Lua** (`mlua`), now running: a script is given the context API and nothing
  else, which is what leaves a second backend, such as WASM, possible later.
- **Workflow topology as declarative data**, not script — so it can be statically validated,
  round-tripped through a visual editor, and edited by a machine.
- **An MCP server whose tools actually edit workflows and node types**, so agents can help build
  agents.

## Development

### Prerequisites

- **Git.** On Windows, [Git for Windows](https://git-scm.com/download/win) — it supplies the POSIX
  `sh`, `curl`, `unzip` and `sha256sum` that the setup scripts and the commit hook need, so nothing
  else has to be installed for them.
- **Rust**, via [rustup](https://rustup.rs). `rust-toolchain.toml` pins the channel and components,
  so the right ones are installed on first use. `agconflo-lua` builds Lua from its C source, so it
  needs the C compiler Rust's own toolchain already relies on — the MSVC build tools on Windows —
  and no Lua installed anywhere.
- **Docker**, running, for the tests of a tool's container, with the two images they pin present:
  `docker pull alpine@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6` and
  `docker pull python@sha256:cea0e6040540fb2b965b6e7fb5ffa00871e632eef63719f0ea54bca189ce14a6`.
  Those tests fail, naming what is missing, rather than skip.

The documentation toolchain is [ubCode](https://ubcode.useblocks.com/) (`ubc`) and nothing else — no
Python, no Sphinx, no Java. The tests run under [cargo-nextest](https://nexte.st/), because it writes
the JUnit report that stable `cargo test` cannot. Setup fetches both.

### Setup

Three steps, once per clone, in any order:

```
sh scripts/get-ubc.sh
sh scripts/get-nextest.sh
git config core.hooksPath .githooks
```

Each script downloads one pinned version into `tools/` — `ubc` (~90 MB) and `cargo-nextest` — verifies
its SHA-256, and refuses to install anything that does not match. `tools/` is gitignored, and is
deliberately *not* added to `PATH`: both are always invoked by path. Re-running a script is free — an
already-correct binary is left alone — and `--force` reinstalls anyway.

`core.hooksPath` is local configuration and so cannot be committed, which is why every clone sets it
for itself. Until it is set, the hook does not run at all.

The order really does not matter: with the hook enabled but `ubc` not yet installed, the
documentation gate skips itself and tells you the command to fix that, and without nextest the tests
run under `cargo test` instead, with a message saying so.

### Running the checks by hand

```
sh .githooks/pre-commit                                        # everything the commit gate does
( cd docs && ../tools/ubc check --deny warning )               # lint the requirements project
sh scripts/docs-selftest.sh                                    # prove the metamodel's rules still fire
sh scripts/cypher-gates.sh                                     # run each Cypher gate, proved first
sh scripts/cypher-gates.sh --report                            # print the review reports
tools/ubc query cypher --project docs --strict 'MATCH (n) RETURN n.id'  # query the graph (AGENTS.md)
tools/cargo-nextest nextest run --workspace --all-targets      # the tests, as the gates run them
sh scripts/import-test-runs.sh --check                         # are the committed test results current?
sh scripts/import-test-runs.sh                                 # rewrite them, then read the diff
sh scripts/impact.sh <NEED_ID>                                 # what a change to one need reaches
sh scripts/change-records.sh --staged                          # does every changed requirement have a record?
sh scripts/comment-rules.sh                                    # no need id in a comment, every code marker well formed
sh scripts/comment-rules.sh --report                           # long comments and docstrings, for review
```

The tests write a JUnit report to `target/nextest/default/junit.xml`, and a test still running after
20 s is killed and fails; both are set in `.config/nextest.toml`.

`ubc check` is run from `docs/` because that is the project root; from the repository root it stops
with "No configuration file found". Under Git Bash on Windows, `tools/ubc` resolves to
`tools/ubc.exe` on its own.

### The commit hook

`.githooks/pre-commit` is a POSIX shell script, so running it by hand needs `sh` — PowerShell cannot
execute it directly.

It scans staged changes for credentials — this repository is public, so a leak is permanent — then
checks and format-checks the requirements project, runs the Cypher gates, runs the metamodel
self-test, refuses a change to an existing requirement that no change record names, refuses a need id
in a Rust comment and a malformed code marker, then
`cargo fmt --check`, `clippy -D warnings` and the tests under nextest. Both toolchains
degrade rather than block: a missing `ubc` or a missing `cargo` skips its own gate with a message
rather than failing the commit, a missing nextest falls back to `cargo test`, and the Rust steps are
skipped while the workspace has no crates in it.

The self-test is the one step restricted to commits that can affect it — the metamodel, the fixtures,
the driver, or the pinned `ubc` version. It costs around a quarter of a second per fixture, and it is
safe to filter precisely because each fixture is checked as a one-file project, so editing prose in
`docs/` cannot change its result. CI runs it unconditionally regardless.

Every `ubc` step except the self-test is licensed through ubCode's free open-source grant, which is
determined from the repository's remote and needs network access — the answer is then cached for a
few days. `ubc format` and the Cypher queries need the grant at any size; `ubc check` has a five-file
free tier, and **`docs/` is past it**, so all of them now depend on it. In practice that means a
machine which has been offline longer than the cached answer lives gets **no local documentation
checking at all** — it is told so plainly, and the commit proceeds.

An unavailable grant is reported with the same exit code as a real defect, so the hook tells the two
apart by the message rather than sending you to fix documentation that is fine. CI has the grant, runs
all of them, and fails hard; it is the authority.

`git commit --no-verify` bypasses the hook deliberately.

### Branch protection on `main`

The hook is a courtesy and can be bypassed; `main` cannot. It is protected, and **the rules apply to
the repository owner too — there is no admin bypass**, on the grounds that a gate the only contributor
can step around is not a gate. So `git push origin main` is rejected and every change lands through a
pull request.

- **Both CI jobs must pass** — `rust` and `docs` — and the branch must be up to date with `main` first.
- **Signed commits are required**, history must stay **linear** (so merge by squash or rebase, never a
  merge commit), review conversations must be resolved, and force-pushes and deletion are blocked.
- **Zero approving reviews are required.** Not laxity: GitHub does not allow approving your own pull
  request, so on a single-contributor repository any non-zero count would make `main` permanently
  unmergeable. It becomes one the day there is a second contributor.

One coupling is worth knowing, because it is a hard block rather than an inconvenience: `ci.yml` has no
`pull_request` trigger, and the required checks are satisfied only because the push-triggered run
attaches to the same head commit as the pull request. That holds for a branch of this repository and
**not for a fork** — the first pull request from a fork will never report its required checks, and
cannot be merged until `pull_request` is added to the workflow.

The live configuration is the source of truth, and reading it costs nothing:

```
gh api repos/Felix-Ruh/agconflo/branches/main/protection
```

Required signatures is the one setting that lives at its own endpoint
(`.../protection/required_signatures`) rather than in that payload, so a `PUT` of the protection object
cannot switch it on — though an already-enabled setting does survive one.

### Process and testing

This project is developed against its own requirements, in a V-Model shape. Requirements,
architecture, decisions and test cases live in `docs/` as linked, schema-validated objects rather
than prose, written in [Sphinx-Needs](https://sphinx-needs.readthedocs.io/) format — the format is
what is shared with that project, not the build, which is `ubc`.

The testing policy, and how work is run here — planning, the review of a diff before committing, the
commit conventions and what to stop and ask about — are in [AGENTS.md](AGENTS.md).

### The requirements metamodel

A descending chain from what someone wants to the code that does it, and back up from the tests that
check it, plus the types that sit beside that chain rather than on it:

```
stkh_req -> feat_req -> feat_arch -> comp_req <- impl    where code meets a requirement
                ^           |            ^
                |         comp <---------+
                |                        |
                +------- test_case ------+              a case verifies either requirement level
                              ^
                          test_run                      the latest run of the case it ran

dec -> evd                                              a decision, and the measurement it rests on
 |
 +-- supersedes -> dec                                  a decision this one replaced
```

`impl` and `test_run` are the two nobody writes: an implementation comes from a one-line marker in
the Rust source, and a run is imported from the test runner's report. Both are described in their own
sections below.

Each level exists because it carries a decision the level above cannot: `stkh_req` says whose goal it
is, `feat_req` says how the behaviour will be verified, `feat_arch` names the components a feature
decomposes into, `comp_req` allocates a behaviour to exactly one of them. A level that cannot name such
a field is not a level — which is why there is no detailed-design level below `comp_req`. In Rust the
type system *is* the detailed design, and a need restating a trait signature restates it by
construction.

**A `dec` records a choice the project made and must not re-litigate; an `evd` records a measurement one
rests on.** Neither is a level — nothing is derived from them. The design point worth knowing is that
evidence carries its finding in an `observation` field rather than in `statement`: every wording rule
selects on `statement`, and a measurement legitimately says "faster" and "approximately 650 ms", which
are exactly the words those rules exist to ban from a requirement. A decision, by contrast, does use
`statement`, so it is held to them — one hiding behind "the simpler option" is refused.

**No link may dangle at authoring time:** whatever a need refers to already exists when that need is
written. Most links achieve that by pointing **up** the V, from the concrete to the abstract, because
the abstract was written first. The two decision links achieve it by pointing the other way — evidence
is recorded *before* the decision resting on it, and a superseded decision exists before its
replacement — so the link belongs on whichever need was written second. The guarantee is the absence of
dangling references, not the direction.

**The obligation lives in a `statement` field, not in the need's body.** This is the one thing worth
knowing before writing a requirement here. A need's body is invisible to schema validation, so a rule
about it silently matches nothing; the body therefore carries rationale and derivation, and the
sentence that can be held to a grammar lives in `statement`. Requirement statements follow
[EARS](https://alistairmavin.com/ears/), declared per requirement in `ears_pattern` and checked against
that pattern's grammar.

**Where the reference actually is:** `docs/ubproject.toml` for the types, links and fields,
`docs/schemas.json` for the rules, and `docs-selftest/fixtures/` for what each rule catches in practice.
All three are commented; this section is an orientation, not a specification, so that there is only one
copy to keep true. Counts are deliberately not quoted here — prose restating a number goes stale the
first time the number changes, and nothing checks prose.

### Where things go

```
docs/index.rst            the table of contents, and nothing else
docs/stakeholder/         what people want: context, authoring, execution, and the project's own goals
docs/features/            per feature: its feature requirements, then its architecture
docs/components/          per feature: its components, then the requirements allocated to them
docs/tests/               per feature: how each of those requirements is checked
docs/code/                per crate: where each component requirement is met, traced from its source
docs/decisions/           choices made, grouped by what they are about
docs/decisions/changes    every change to an existing requirement, with its impact analysis
docs/evidence/            measurements the decisions rest on
docs/test-runs.json       the latest run of each test case, written by the importer
```

Requirements are grouped by subject, and **the grouping is enforced**. Once any toctree exists, `ubc`
reports every document that no toctree reaches — so adding a file under `docs/` without listing it in
`docs/index.rst` fails the build rather than leaving it to be read by nobody. A mistyped entry is caught
from the other side, as a reference to a document that does not exist.

### Changing a rule

`docs-selftest/` holds deliberately invalid needs. Each fixture breaks one rule on purpose, and
`docs-selftest/expected/` records the exact diagnostics that break must produce.

It exists because a wrongly shaped rule in `ubc` can be **silently ignored**, leaving the project green.
Of the four shapes measured, two still do that on the pinned version — a misspelled keyword, and a
keyword of the wrong kind for the field — while a composite keyword in the wrong place is now rejected,
and a rule about a need's body is reported as a configuration warning. A rule of the right shape can
also simply match the wrong thing, which nothing reports at all. So a rule that has never been seen to
fail cannot be assumed to work.

A rule's `message` is printed under each finding as a note, so it appears in the golden files too:
editing a message moves its fixture's golden, and that diff is read like any other.

After deliberately changing a rule, or bumping the pinned `ubc`:

```
sh scripts/docs-selftest.sh --bless    # rewrite every golden file
git diff                               # this diff is the review
```

Two rules of thumb. **Every rule needs a fixture that fails without it** — the driver refuses to pass a
rule in `schemas.json` that no golden file mentions. And **append new rules to the end of
`schemas.json`, never insert**: a rule's array index appears in every message it produces, so inserting
one re-blesses every golden below it for nothing.

### Adding a Cypher gate

Some checks compare two needs — a requirement's statement against the title of the component it is
allocated to — and no schema rule can express that. Those are Cypher queries in
`scripts/gates/<name>.cypher`, run by `sh scripts/cypher-gates.sh`, and a gate passes when its query
returns no rows. A query must return a column named `offender`; rows without one are refused rather
than counted as none.

Every gate needs `docs-selftest/fixtures/gate_<name>.rst`, and the runner proves the gate on it before
trusting it on `docs/`: every need there whose id contains `_BAD_` must be reported, and no other may
be. The controls matter as much as the offenders — the first version of the subject-agreement gate
reported five valid requirements and missed one wrong one.

A question whose honest answer is sometimes "yes" is a **report** instead:
`scripts/reports/<name>.cypher` with a fixture `report_<name>.rst`, proved the same way, whose rows in
`docs/` are printed for a person to read rather than failing anything.
`sh scripts/cypher-gates.sh --report` runs them. The hook does not, since output nobody asked for on
every commit is output nobody reads; CI does, so a report whose query has stopped working fails there.

### Code in the graph

Where a component requirement is met is recorded beside the code that meets it, as a one-line comment
on the item doing the work:

```rust
// @Each ancestor walked once,IMPL_LINEAGE_WALK,impl,[CREQ_WALKER_EACH_ONCE]
```

`ubc` reads those markers into the graph as `impl` needs — `docs/code/agconflo-core.rst` is what asks
for them — and fills each one's `code_url` with a permalink to its line at the commit being checked.
Nothing is written by hand, so the trace cannot drift from the code the way prose would. One
requirement may be met in several places, and each place carries its own marker.

A marker's second list names the decisions the code follows, so analysing a decision finds the code
it shaped; code that follows one and meets no requirement gets a `trace` marker instead of an `impl`.
Why code is as it is lives there, in the graph, and a comment says what the code does - AGENTS.md,
"Comments and docstrings", has the guidelines, and `scripts/comment-rules.sh` refuses a need id in
a comment's prose.

A marker naming a requirement that does not exist is a dead link and fails the build. A requirement
that no marker names is **not** an error — code is written after its requirement — and appears in the
`unimplemented` review report instead. `untested` and `unrun` ask the same question of test cases and
of runs; all three are printed by `sh scripts/cypher-gates.sh --report` and gate nothing.

### Test results in the graph

`docs/test-runs.json` holds one `test_run` need per test — the test case it ran and whether it
passed — so that a requirement's status can be asked of the graph rather than of a log. It is
written by `crates/junit-to-needs` from the report nextest leaves at
`target/nextest/default/junit.xml`, and a test is matched to its case by name alone: the case's id is
`TEST_` followed by the test's path uppercased, so a renamed test breaks the build as a dead link
rather than going quietly unrecorded.

The file is **committed**, because an external needs file that is wired but absent is reported as a
warning — which fails at the `--deny warning` every gate here uses — so producing it on demand would
leave a fresh clone unable to check its documentation until it had built and run the tests. Both gates then check that it is current, and **neither ever rewrites it**,
just as `cargo fmt --check` and the golden files never fix their own subject. Rewriting is a command
you run, and the diff is one you read:

```
sh scripts/import-test-runs.sh
```

It records only what changes when an outcome changes. A report also carries a run identifier, a
timestamp per test and a duration per test, all different between two runs of the same tests; keeping
them would dirty the file on every run and say nothing. The history is in git.

## Licence

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this
work by you, as defined in the Apache-2.0 licence, shall be dual-licensed as above, without any
additional terms or conditions.
