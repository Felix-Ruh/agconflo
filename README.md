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

**Pre-alpha. The engine is not implemented yet.** What exists today is the development process around
it: a requirements project under `docs/` with a validated metamodel behind it, a commit gate, and
continuous integration. The design is well developed; the code is not.

Expect the public API to change without warning. Breaking changes, yes; force-pushes to `main`, no —
those are blocked outright, along with direct pushes to it.

## Planned shape

- **Rust**, as a Cargo workspace. `agconflo-core` is a normal Rust crate with a deliberately strict
  public API; frontends (CLI, viewer, MCP server) are separate crates that depend on it.
- **Provider-agnostic LLM access** via [`genai`](https://github.com/jeremychone/rust-genai), and MCP
  via [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk). The agent loop is ours — that is the
  layer this project exists to own.
- **Node behaviour in Lua** (`mlua`), behind a host-function interface that is not Lua-shaped, so a
  WASM backend stays possible later.
- **Workflow topology as declarative data**, not script — so it can be statically validated,
  round-tripped through a visual editor, and edited by a machine.
- **An MCP server whose tools actually edit workflows and node types**, so agents can help build
  agents.

## Development

### Prerequisites

- **Git.** On Windows, [Git for Windows](https://git-scm.com/download/win) — it supplies the POSIX
  `sh`, `curl` and `sha256sum` that the setup script and the commit hook need, so nothing else has to
  be installed for them.
- **Rust**, via [rustup](https://rustup.rs). `rust-toolchain.toml` pins the channel and components,
  so the right ones are installed on first use.

The documentation toolchain is [ubCode](https://ubcode.useblocks.com/) (`ubc`) and nothing else — no
Python, no Sphinx, no Java. Setup fetches it.

### Setup

Two steps, once per clone, in either order:

```
sh scripts/get-ubc.sh
git config core.hooksPath .githooks
```

`get-ubc.sh` downloads one pinned version of `ubc` (~66 MB) into `tools/`, verifies its SHA-256, and
refuses to install anything that does not match. `tools/` is gitignored, and is deliberately *not*
added to `PATH`: `ubc` is always invoked by path. Re-running the script is free — an already-correct
binary is left alone — and `--force` reinstalls anyway.

`core.hooksPath` is local configuration and so cannot be committed, which is why every clone sets it
for itself. Until it is set, the hook does not run at all.

The order really does not matter: with the hook enabled but `ubc` not yet installed, the
documentation gate skips itself and tells you the command to fix that.

### Running the checks by hand

```
sh .githooks/pre-commit                                        # everything the commit gate does
( cd docs && ../tools/ubc check --deny warning )               # lint the requirements project
sh scripts/docs-selftest.sh                                    # prove the metamodel's rules still fire
tools/ubc query cypher --project docs 'MATCH (n) RETURN n.id'  # query the needs graph
```

`ubc check` is run from `docs/` because that is the project root; from the repository root it stops
with "No configuration file found". Under Git Bash on Windows, `tools/ubc` resolves to
`tools/ubc.exe` on its own.

### The commit hook

`.githooks/pre-commit` is a POSIX shell script, so running it by hand needs `sh` — PowerShell cannot
execute it directly.

It scans staged changes for credentials — this repository is public, so a leak is permanent — then
checks and format-checks the requirements project, then runs the metamodel self-test, then
`cargo fmt --check`, `clippy -D warnings` and the tests. Both toolchains degrade rather than block: a
missing `ubc` or a missing `cargo` skips its own gate with a message rather than failing the commit, and
the Rust steps are also skipped while the workspace has no crates in it.

The self-test is the one step restricted to commits that can affect it — the metamodel, the fixtures,
the driver, or the pinned `ubc` version. It costs around a quarter of a second per fixture, and it is
safe to filter precisely because each fixture is checked as a one-file project, so editing prose in
`docs/` cannot change its result. CI runs it unconditionally regardless.

Both `ubc` steps are licensed through ubCode's free open-source grant, which is determined from the
repository's remote and needs network access — the answer is then cached for a few days. `ubc format`
needs the grant at any size; `ubc check` has a five-file free tier and needs it once the project holds
more than that. Either way an unavailable grant is reported with the same exit code as a real defect,
so the hook tells the two apart by the message: when the grant cannot be confirmed it says so and
carries on rather than sending you to fix documentation that is fine. CI runs both and fails hard.

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

Testing policy: property-based tests wherever a property can be stated, alongside ordinary positive
tests and error-path tests that assert *which* failure occurs and how the system behaves afterwards.

### The requirements metamodel

Six need types, descending from what someone wants to the code that does it:

```
stkh_req -> feat_req -> feat_arch -> comp_req -> Rust
                            |            |
                          comp <---------+          test_case verifies either requirement level
```

Each level exists because it carries a decision the level above cannot: `stkh_req` says whose goal it
is, `feat_req` says how the behaviour will be verified, `feat_arch` names the components a feature
decomposes into, `comp_req` allocates a behaviour to exactly one of them. A level that cannot name such
a field is not a level — which is why there is no detailed-design level below `comp_req`. In Rust the
type system *is* the detailed design, and a need restating a trait signature restates it by
construction.

Links always point **up** the V, from the concrete to the abstract, so a link can never dangle at
authoring time.

**The obligation lives in a `statement` field, not in the need's body.** This is the one thing worth
knowing before writing a requirement here. A need's body is invisible to schema validation, so a rule
about it silently matches nothing; the body therefore carries rationale and derivation, and the
sentence that can be held to a grammar lives in `statement`. Requirement statements follow
[EARS](https://alistairmavin.com/ears/), declared per requirement in `ears_pattern` and checked against
that pattern's grammar.

**Where the reference actually is:** `docs/ubproject.toml` for the types, links and fields,
`docs/schemas.json` for the 32 rules, and `docs-selftest/fixtures/` for what each rule catches in
practice. All three are commented; this section is an orientation, not a specification, so that there
is only one copy to keep true.

### Changing a rule

`docs-selftest/` holds deliberately invalid needs. Each fixture breaks one rule on purpose, and
`docs-selftest/expected/` records the exact diagnostics that break must produce.

It exists because a wrongly shaped rule in `ubc` is **not rejected — it is silently ignored**, and the
project goes green. Four ways to do that have been measured: a composite keyword in the wrong place, a
misspelled keyword, a keyword of the wrong kind for the field, and any rule about a need's body. So a
rule that has never been seen to fail cannot be assumed to work.

After deliberately changing a rule, or bumping the pinned `ubc`:

```
sh scripts/docs-selftest.sh --bless    # rewrite every golden file
git diff                               # this diff is the review
```

Two rules of thumb. **Every rule needs a fixture that fails without it** — the driver refuses to pass a
rule in `schemas.json` that no golden file mentions. And **append new rules to the end of
`schemas.json`, never insert**: a rule's array index appears in every message it produces, so inserting
one re-blesses every golden below it for nothing.

## Licence

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this
work by you, as defined in the Apache-2.0 licence, shall be dual-licensed as above, without any
additional terms or conditions.
