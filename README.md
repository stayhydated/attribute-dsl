# attribute-dsl

[![CI](https://github.com/stayhydated/attribute-dsl/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/attribute-dsl/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/github/stayhydated/attribute-dsl/graph/badge.svg)](https://codecov.io/github/stayhydated/attribute-dsl)
[![Book](https://img.shields.io/badge/book-online-blue)](https://stayhydated.github.io/attribute-dsl/book/)
[![Crates.io](https://img.shields.io/crates/v/attribute-dsl.svg)](https://crates.io/crates/attribute-dsl)

`attribute-dsl` provides `syn` parsers for Rust proc-macro attributes built from
a path root, ordered dot calls, optional labels, comma-separated entries, and
named groups. It also supports rust-analyzer completion probes and `_`
placeholders for application-owned subject types.

Use it in derive-macro and attribute-macro implementation crates. The consumer
keeps ownership of domain validation, constructors, and generated Rust.

## Parse an attribute chain

Parse an attribute chain through `syn`:

```rust
use attribute_dsl::{AttributeChain, ChainCompletion};

let chain: AttributeChain =
    syn::parse_str("RootType::<_>.first(1).second::<String>(\"value\")")
        .expect("valid attribute chain");

assert_eq!(
    chain
        .root_path()
        .segments
        .last()
        .expect("a parsed path has a segment")
        .ident
        .to_string(),
    "RootType"
);
assert_eq!(chain.calls().len(), 2);
assert!(matches!(chain.completion(), ChainCompletion::None));
```

The result retains `syn::Path`, `syn::Ident`, and `syn::Expr` nodes so the
consumer can preserve syntax and spans while quoting its expansion.

## Capabilities

- Parse one chain, a labeled entry, a list, or a named group.
- Recover a trailing dot as a typed rust-analyzer completion probe.
- Inspect or replace `_` type placeholders in paths, types, and expressions.
- Return `syn::Error` values for spanned proc-macro diagnostics.
