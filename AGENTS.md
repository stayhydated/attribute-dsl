# AGENTS.md

This is the working guide for contributors and coding agents in the
`attribute-dsl` workspace. The repository is a single published Rust crate for
parsing proc-macro attribute DSL chains and infer placeholders.

Start with:

- `src/lib.rs` for the public crate facade and exported API.
- `README.md` for user-facing docs; it is included as crate documentation with
  `#![doc = include_str!("../README.md")]`.
- `justfile` for local validation and maintenance recipes.

## Project Map

- `src/chain.rs`
  Audience: public API and validation.
  Owns `AttributeChain`, `ChainCall`, `ChainEntry`, `ChainList`,
  `NamedChainGroup`, completion-probe parsing, and their inline tests.

- `src/infer.rs`
  Audience: public API and validation.
  Owns `SingleTypeArg` plus the `split_terminal_single_type_arg` and
  `substitute_infer_*` helpers, with inline tests for supported syntax forms.

- `src/lib.rs`
  Audience: public facade.
  Re-exports the crate API and includes `README.md` as crate docs.

- `examples/derive_field_attrs.rs`
  Audience: executable public example.
  Mirrors the derive-macro workflow shown in `README.md`.

## Synchronization Rules

- When parser syntax, accepted/rejected chain forms, completion probes, or
  diagnostic behavior changes, update `src/chain.rs`, its inline tests, and the
  matching grammar/examples in `README.md`.
- When infer placeholder behavior changes, update `src/infer.rs`, its inline
  tests, and the `README.md` Infer Helpers section.
- When public exports change, update `src/lib.rs`, the owning module docs/tests,
  and any affected README examples in the same change.
- When the derive-macro workflow changes, keep `examples/derive_field_attrs.rs`
  and the README Derive Macro Example aligned.
- When local or CI validation changes, keep `justfile`, `.github/workflows/ci.yml`,
  and any named guidance here aligned.
- The crate is version `0.1.0`; durable docs and examples should describe the
  current API and repository shape.

## Validation

- Use `just --list` to inspect the repository command index.
- For Rust behavior changes, choose the narrowest applicable recipe from
  `justfile`: `just check`, `just clippy`, or `just test`.
- Use `just fmt` for formatting Rust, TOML, and Markdown files when formatting is
  part of the change.
- Use `just test-docs` when README or crate documentation examples change.
- Use `just ci` for the local recipe chain when the required external tools from
  the recipes are available.
- CI runs formatting checks, locked Rust tests, clippy, docs, package dry-run,
  cargo-machete, coverage, and Codecov publishing from `.github/workflows/ci.yml`.
