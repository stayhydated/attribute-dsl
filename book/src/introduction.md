# Introduction

`attribute-dsl` parses Rust proc-macro attributes made from a path root,
dot-call chains, comma-separated entries, and named groups. It preserves
`syn` syntax nodes so a macro can inspect the input, report spanned errors, and
quote the parsed pieces into generated Rust.

This guide is for authors of derive and attribute macros. It assumes familiarity
with `syn`, `quote`, and Rust token streams. The crate supports Rust 1.96 and
edition 2024.

Use the crate when an attribute accepts syntax such as:

```text
RootType::<_>.first(1).second::<String>("value")
```

The parsed model separates that input into a `syn::Path` root, ordered
`ChainCall` values, and an optional completion-probe marker. Additional parsers
cover labeled entries, comma-separated lists, and named parenthesized groups.

The infer helpers replace `_` placeholders with an application-owned subject
type without converting syntax trees to strings. This is useful when an
attribute describes a builder or validator whose type depends on the annotated
field.

Start with [Getting started](getting_started.md). Continue to
[Parse chain syntax](parse_chain_syntax.md) when choosing a parser,
[Emit completion probes](completion_probes.md) for rust-analyzer completion, and
[Substitute infer placeholders](infer_placeholders.md) for subject-type
substitution. [Build a proc-macro expansion](proc_macro_expansion.md) combines
those operations into one derive-macro workflow.
