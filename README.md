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

**Pre-alpha. The engine is not implemented yet.** What exists today is the development toolchain
around it: a requirements project under `docs/`, a commit gate, and continuous integration. The
design is well developed; the code is not.

Expect the public API to change without warning, and expect force-pushes to be avoided but breaking
changes not to be.

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
tools/ubc query cypher --project docs 'MATCH (n) RETURN n.id'  # query the needs graph
```

`ubc check` is run from `docs/` because that is the project root; from the repository root it stops
with "No configuration file found". Under Git Bash on Windows, `tools/ubc` resolves to
`tools/ubc.exe` on its own.

### The commit hook

`.githooks/pre-commit` is a POSIX shell script, so running it by hand needs `sh` — PowerShell cannot
execute it directly.

It scans staged changes for credentials — this repository is public, so a leak is permanent — then
checks and format-checks the requirements project, then runs `cargo fmt --check`,
`clippy -D warnings` and the tests. Both toolchains degrade rather than block: a missing `ubc` or a
missing `cargo` skips its own gate with a message rather than failing the commit, and the Rust steps
are also skipped while the workspace has no crates in it.

Both `ubc` steps are licensed through ubCode's free open-source grant, which is determined from the
repository's remote and needs network access — the answer is then cached for a few days. `ubc format`
needs the grant at any size; `ubc check` has a five-file free tier and needs it once the project holds
more than that. Either way an unavailable grant is reported with the same exit code as a real defect,
so the hook tells the two apart by the message: when the grant cannot be confirmed it says so and
carries on rather than sending you to fix documentation that is fine. CI runs both and fails hard.

`git commit --no-verify` bypasses the hook deliberately.

### Process and testing

This project is developed against its own requirements, in a V-Model shape. Requirements,
architecture, decisions and test cases live in `docs/` as linked, schema-validated objects rather
than prose, written in [Sphinx-Needs](https://sphinx-needs.readthedocs.io/) format — the format is
what is shared with that project, not the build, which is `ubc`.

Testing policy: property-based tests wherever a property can be stated, alongside ordinary positive
tests and error-path tests that assert *which* failure occurs and how the system behaves afterwards.

## Licence

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this
work by you, as defined in the Apache-2.0 licence, shall be dual-licensed as above, without any
additional terms or conditions.
