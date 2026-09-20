# Repository guide

`attribute-dsl` provides parsers and infer helpers for proc-macro attribute
chains. Start with `src/lib.rs` for public exports and `just --list` for local
commands. `README.md` is included as the crate documentation.

## Where to work

| Surface | Ownership |
| --- | --- |
| `src/chain.rs` | Public chain, entry, list, and group parsers; completion options and markers; inline parser tests. |
| `src/infer.rs` | Public terminal type-argument splitting and infer substitution; inline syntax tests. |
| `examples/derive_field_attrs.rs` | Executable derive-style expansion example. |
| `book/src/` | User guidance for parsing, completion, substitution, and expansion. |
| `skills/use-attribute-dsl/` | Consumer integration skill and its concrete patterns. |
| `web/src/lib.rs` | Project identity, destinations, and the Pages route manifest. Shared components own the portal's styling. |
| `xtask/src/commands/` | Book and llms.txt generation, Pages assembly, and static preview. |

## Keep related surfaces aligned

- Parser syntax, completion, or diagnostic changes belong with the inline
  tests in `src/chain.rs`. Update affected README examples, book guidance, and
  skill patterns in the same change.
- Infer changes belong with the supported syntax cases in `src/infer.rs`.
  Keep `book/src/infer_placeholders.md` and affected expansion examples and
  skill patterns aligned.
- Export public API changes through `src/lib.rs`. Keep
  `examples/derive_field_attrs.rs` and `book/src/proc_macro_expansion.md`
  aligned when the expansion workflow changes.
- Change book and site sources through their owning files; use the xtask
  commands to regenerate their outputs. For project destinations or Pages
  routing, check `web/src/lib.rs`, `web/Dioxus.toml`, and the xtask URL and
  base-path inputs together.
- Keep the three `stayhydated/shared` dependencies pinned to the same full
  revision; the site components and build helpers form one integration.

## Validate the changed surface

- Parser or infer behavior: `cargo test -p attribute-dsl --lib --locked`.
- Derive-style example: `cargo run -p attribute-dsl --example derive_field_attrs --locked`.
- README or crate examples: `cargo test -p attribute-dsl --doc --locked`.
  `just test-docs` builds and opens rustdoc; it does not execute doctests.
- Book content: `MDBOOK_BUILD__CREATE_MISSING=false mdbook build book`.
  Run the Rust snippets against a fresh crate build as shown below.
- Markdown: `rumdl check README.md AGENTS.md book/src skills/use-attribute-dsl`.
- Skill changes: run the skill creator's `quick_validate.py` helper on
  `skills/use-attribute-dsl` when available, then check its patterns against
  the public API and consumer example.
- Pages or output-generation changes: `just web-build`; use
  `just web-preview` to inspect the assembled site.

Use the relevant `just check`, `just clippy`, and `just test` recipes for
workspace Rust changes. `.github/workflows/ci.yml` defines the merge checks;
`just ci` also runs formatting commands that modify files.

For book snippets, use a fresh target directory so rustdoc can resolve one
build of each dependency:

```bash
book_target=$(mktemp -d)
cargo build -p attribute-dsl --lib --locked --target-dir "$book_target"
MDBOOK_BUILD__CREATE_MISSING=false mdbook test book -L "$book_target/debug/deps"
```
