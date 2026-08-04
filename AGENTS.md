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

- `book/`
  Audience: public user documentation.
  Owns task-focused guidance for parsing chains, completion probes, infer
  substitution, and proc-macro expansion.

- `skills/use-attribute-dsl/`
  Audience: public agent integration.
  Owns the reusable workflow and patterns for applying `attribute-dsl` in a
  consumer proc-macro crate.

- `web/`
  Audience: public project navigation.
  Owns the demo-less single-page GitHub Pages portal, project identity, and
  route manifest. Shared owns the portal styling and component theme.

- `xtask/`
  Audience: internal workflow.
  Owns book, llms.txt, GitHub Pages assembly, and static preview commands.

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
- When public parser or infer behavior changes, keep the matching `book/src/`
  chapters and `skills/use-attribute-dsl/` guidance aligned with the owning
  Rust modules and README examples.
- When project identity, destinations, or Pages routing changes, keep
  `web/src/lib.rs`, `web/Dioxus.toml`, the xtask build inputs, and Pages
  workflows aligned.
- Keep `stayhydated-dioxus`, `stayhydated-site`, and `stayhydated-xtask` pinned
  to one full `stayhydated/shared` revision.
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
- Use `mdbook build book` for book-only changes and validate repository skills
  with the skill creator's `quick_validate.py` helper.
- Use `just web-build` plus the stayhydated Pages consumer audit when web,
  book-output, llms-output, sitemap, or preview behavior changes.
- Use `just ci` for the local recipe chain when the required external tools from
  the recipes are available.
- CI runs formatting checks, locked Rust tests, clippy, docs, package dry-run,
  cargo-machete, coverage, and Codecov publishing from `.github/workflows/ci.yml`.
