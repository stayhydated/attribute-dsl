# attribute-dsl

[![CI][ci-badge]][ci]
[![Codecov][codecov-badge]][codecov]
[![Book][book-badge]][book]
[![crates.io: attribute-dsl][crate-badge]][crate]

`attribute-dsl` provides `syn` parsers for Rust proc-macro attributes built from
a path root, ordered dot calls, optional labels, comma-separated entries, and
named groups. Authors of derive and attribute macros can preserve Rust syntax
while supporting rust-analyzer completion probes and `_` placeholders for
application-owned subject types.

## Overview

- Parse one chain, a labeled entry, a list, or a named group.
- Retain `syn::Path`, `syn::Ident`, and `syn::Expr` nodes for syntax- and
  span-preserving expansion.
- Recover a trailing dot as a typed rust-analyzer completion probe.
- Inspect or replace `_` type placeholders in paths, types, and expressions.
- Return `syn::Error` values for spanned proc-macro diagnostics.

Consumers keep ownership of domain validation, constructors, and generated
Rust.

## Example

Parse an attribute chain through `syn`:

```rust
use attribute_dsl::{AttributeChain, ChainCompletion};

fn main() -> syn::Result<()> {
    let chain: AttributeChain =
        syn::parse_str("RootType::<_>.first(1).second::<String>(\"value\")")?;

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

    Ok(())
}
```

[ci-badge]: https://github.com/stayhydated/attribute-dsl/actions/workflows/ci.yml/badge.svg?branch=master
[ci]: https://github.com/stayhydated/attribute-dsl/actions/workflows/ci.yml
[codecov-badge]: https://codecov.io/github/stayhydated/attribute-dsl/graph/badge.svg
[codecov]: https://codecov.io/github/stayhydated/attribute-dsl
[book-badge]: https://img.shields.io/badge/Book-mdBook-blue
[book]: https://stayhydated.github.io/attribute-dsl/book/
[crate-badge]: https://img.shields.io/crates/v/attribute-dsl.svg?label=attribute-dsl
[crate]: https://crates.io/crates/attribute-dsl
