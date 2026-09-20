# Getting started

Parse a path-rooted dot-call chain and inspect its root and calls in a macro
implementation crate.

## Prerequisites

- Rust 1.98 or newer.
- A derive-macro or attribute-macro implementation crate.
- A clear grammar for the attribute accepted by that macro.

## Add dependencies

Add `attribute-dsl` alongside the syntax dependencies used by the macro:

```toml
[dependencies]
attribute-dsl = "0.2"
proc-macro2 = "1.0"
quote = "1.0"
syn = { features = [ "full" ], version = "3.0" }
```

Use the same major version of `syn` as `attribute-dsl`: the parser's public
API accepts and returns `syn` nodes. `quote` and `proc-macro2` are used by the
expansion examples later in this guide.

## Parse a chain

`AttributeChain` implements `syn::parse::Parse`, so it works with attribute
argument parsing and `syn::parse_str`:

```rust
# extern crate attribute_dsl;
# extern crate syn;
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
