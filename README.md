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

**Pre-alpha. Nothing is implemented yet.** This repository currently contains only a skeleton: a
licence, a workspace manifest, and this file. The design is well developed; the code is not.

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

## Development process

This project is developed against its own requirements, in a V-Model shape, using
[Sphinx-Needs](https://sphinx-needs.readthedocs.io/). Requirements, architecture, decisions and test
cases live in `docs/` as linked, schema-validated objects rather than prose. That tooling is not yet
in the repository.

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
