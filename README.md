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

Requires Rust 1.99 or newer. Its public syntax nodes use `syn` 3.

## Overview

- Parse one chain, a labeled entry, a list, or a named group.
- Retain `syn::Path`, `syn::Ident`, and `syn::Expr` nodes for syntax- and
  span-preserving expansion.
- Recover a trailing dot as a typed rust-analyzer completion probe.
- Inspect or replace `_` type placeholders in paths, types, and expressions.
- Return `syn::Error` values for spanned proc-macro diagnostics.

Consumers keep ownership of domain validation, constructors, and generated
Rust.

See the [API reference][api] for accepted syntax, completion options, and
substitution boundaries.

## Example

Parse a `syn::Attribute` in a proc-macro expansion function, then substitute
the annotated subject type into its root. This complete example preserves the
`label` call and resolves `TextFilter::<_>` to `TextFilter::<String>`:

```rust
use attribute_dsl::{AttributeChain, substitute_infer_in_path};
use syn::parse_quote;

fn main() -> syn::Result<()> {
    let attr: syn::Attribute = parse_quote! {
        #[column(TextFilter::<_>.label("Name"))]
    };
    let chain = attr.parse_args::<AttributeChain>()?;
    let subject: syn::Type = parse_quote!(String);
    let root = substitute_infer_in_path(chain.root_path(), &subject);

    assert_eq!(root, parse_quote!(TextFilter::<String>));
    assert_eq!(chain.calls().len(), 1);
    assert_eq!(chain.calls()[0].method(), "label");
    assert!(!chain.has_completion_probe());
    Ok(())
}
```

The [derive-style expansion example][derive-example] shows how a consumer
quotes the root, calls, and a typed completion probe into generated Rust.

[ci-badge]: https://github.com/stayhydated/attribute-dsl/actions/workflows/ci.yml/badge.svg?branch=master
[ci]: https://github.com/stayhydated/attribute-dsl/actions/workflows/ci.yml
[codecov-badge]: https://codecov.io/gh/stayhydated/attribute-dsl/branch/master/graph/badge.svg
[codecov]: https://codecov.io/gh/stayhydated/attribute-dsl
[book-badge]: https://img.shields.io/badge/Book-mdBook-blue
[book]: https://stayhydated.github.io/attribute-dsl/book/
[crate-badge]: https://img.shields.io/crates/v/attribute-dsl.svg?label=attribute-dsl
[crate]: https://crates.io/crates/attribute-dsl
[api]: https://docs.rs/attribute-dsl/latest/attribute_dsl/
[derive-example]: https://github.com/stayhydated/attribute-dsl/blob/master/examples/derive_field_attrs.rs
