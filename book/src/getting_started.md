# Getting started

This path parses one path-rooted dot-call chain and inspects its root and calls.
A successful `cargo check` confirms the parser and `syn` types resolve in the
macro implementation crate.

## Prerequisites

- Rust 1.96 or newer.
- A derive-macro or attribute-macro implementation crate.
- A clear grammar for the attribute accepted by that macro.

## Add dependencies

Add `attribute-dsl` alongside the syntax dependencies used by the macro:

```toml
[dependencies]
attribute-dsl = "0.1"
quote = "1.0"
syn = { features = [ "full" ], version = "2.0" }
```

## Parse a chain

`AttributeChain` implements `syn::parse::Parse`, so it works with attribute
argument parsing and `syn::parse_str`:

```rust,ignore
use attribute_dsl::AttributeChain;

let chain: AttributeChain =
    syn::parse_str("RootType::<_>.first(1).second::<String>(\"value\")")?;

assert_eq!(
    chain
        .root_path()
        .segments
        .last()
        .expect("a parsed path has a segment")
        .ident,
    "RootType"
);
assert_eq!(chain.calls().len(), 2);

# Ok::<(), syn::Error>(())
```

In a proc macro, use `attr.parse_args::<AttributeChain>()?` to retain the
attribute's source spans in diagnostics.

## Verify the integration

Run `cargo check` in the macro workspace. A successful check confirms that the
crate versions resolve and the selected parser API is available.

If parsing fails, return the `syn::Error` from the macro expansion path or
combine it with other spanned diagnostics. Do not replace it with a string-only
error, because callers need the source span to locate invalid syntax.
