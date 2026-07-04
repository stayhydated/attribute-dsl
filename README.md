# attribute-dsl

[![Build Status](https://github.com/stayhydated/attribute-dsl/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/attribute-dsl/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/github/stayhydated/attribute-dsl/graph/badge.svg)](https://codecov.io/github/stayhydated/attribute-dsl)
[![Docs](https://docs.rs/attribute-dsl/badge.svg)](https://docs.rs/attribute-dsl/)
[![Crates.io](https://img.shields.io/crates/v/attribute-dsl.svg)](https://crates.io/crates/attribute-dsl)

Shared parser helpers for Rust proc-macro attribute DSLs built from path roots,
dot-call chains, and comma-separated entries.

This crate is intended for derive and attribute macro implementation crates.

## Parsed Model

The core parser accepts a Rust path root followed by zero or more method calls:

```text
AttributeChain := Path ("." Ident Turbofish? "(" Expr,* ")")* CompletionProbe?
CompletionProbe := "." CompletionMarker

ChainEntry     := Ident "=" AttributeChain | AttributeChain
ChainList      := ChainEntry ("," ChainEntry)* ","?
NamedChainGroup := Ident "(" ChainList ")"
```

Examples of accepted chain shapes:

```text
RootType::<_>
RootType::<_>.first(1)
RootType::<i32>.first(1)
RootType::<_>.
RootType::<_>.first(1).raCompletionMarker
```

The root is kept as a `syn::Path`. Each dot-call is represented as a
`ChainCall` containing the method `Ident`, optional turbofish, and call
arguments as `syn::Expr` values. Parser errors are reported as `syn::Error`.

Associated constructors and other expression forms belong in the generated code
around the parsed root. For example, parse `RootType::<_>` as the root, then
emit `RootType::<i32>::builder_for(stringify!(value))` from your macro
expansion.

## Quick Parsing

```rust
use attribute_dsl::{AttributeChain, ChainCompletion, ChainList, NamedChainGroup};
use syn::parse_str;

let chain: AttributeChain =
    parse_str("RootType::<_>.first(1)")?;

assert_eq!(
    chain.root_path().segments.last().unwrap().ident.to_string(),
    "RootType"
);
assert_eq!(chain.calls().len(), 1);
assert_eq!(chain.calls()[0].method().to_string(), "first");
assert_eq!(chain.calls()[0].args().len(), 1);
assert!(matches!(chain.completion(), ChainCompletion::None));

let list: ChainList = parse_str(
    "value = RootType::<_>.first(1), RootType::<i32>.first(2)",
)?;
assert_eq!(list.entries().len(), 2);
assert_eq!(list.entries()[0].label().unwrap().to_string(), "value");
assert!(list.entries()[1].label().is_none());

let group: NamedChainGroup =
    parse_str("fields(value = RootType::<_>.first(1), RootType::<i32>)")?;
assert_eq!(group.name().to_string(), "fields");
assert_eq!(group.entries().len(), 2);

# Ok::<(), syn::Error>(())
```

## Derive Macro Example

```rust
use attribute_dsl::{AttributeChain, substitute_infer_in_path};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, parse_quote};

fn main() -> syn::Result<()> {
    let input: DeriveInput = parse_quote! {
        struct Input {
            #[attribute_dsl(RootType::<_>.first(1).)]
            value: i32,
        }
    };

    let expanded = expand_derive(&input)?;
    assert_eq!(
        compact(&expanded),
        "implInput{fn__attribute_dsl_probe(){let_=RootType::<i32>::builder_for(stringify!(value)).first(1).raCompletionMarker;}}"
    );

    Ok(())
}

fn expand_derive(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = &input.ident;
    let mut field_expansions = Vec::new();

    for field in named_fields(input)? {
        field_expansions.extend(expand_field_attrs(field)?);
    }

    Ok(quote! {
        impl #struct_ident {
            fn __attribute_dsl_probe() {
                #(#field_expansions)*
            }
        }
    })
}

fn named_fields(
    input: &DeriveInput,
) -> syn::Result<&syn::punctuated::Punctuated<Field, syn::token::Comma>> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => Ok(&fields.named),
            _ => Err(syn::Error::new_spanned(
                input,
                "example expects a struct with named fields",
            )),
        },
        _ => Err(syn::Error::new_spanned(input, "example expects a struct")),
    }
}

fn expand_field_attrs(field: &Field) -> syn::Result<Vec<TokenStream>> {
    let field_ident = field
        .ident
        .as_ref()
        .expect("named_fields only returns named struct fields");
    let mut expansions = Vec::new();

    for attr in &field.attrs {
        if !attr.path().is_ident("attribute_dsl") {
            continue;
        }

        let chain = attr.parse_args::<AttributeChain>()?;
        let root = substitute_infer_in_path(chain.root_path(), &field.ty);

        let calls = chain.calls().iter().map(|call| {
            let method = call.method();
            let turbofish = call.turbofish();
            let args = call.args();
            quote! { .#method #turbofish (#(#args),*) }
        });

        let completion = chain
            .completion_marker()
            .map(|marker| quote! { .#marker })
            .unwrap_or_default();

        expansions.push(quote! {
            let _ = #root::builder_for(stringify!(#field_ident)) #(#calls)* #completion;
        });
    }

    Ok(expansions)
}

fn compact(tokens: &TokenStream) -> String {
    tokens
        .to_string()
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect()
}
```

The example parses `#[attribute_dsl(RootType::<_>.first(1).)]`, substitutes the
field type for `_` in the root path, preserves the parsed method calls, and emits
a typed completion probe expression for rust-analyzer.

## Completion Probe

Incomplete input such as `RootType::<_>.` is parsed as if it ended with
`RootType::<_>.raCompletionMarker`.

Macro expanders can detect `AttributeChain::has_completion_probe()` and emit a
method access on the real generated builder. That gives rust-analyzer a typed
receiver inside an attribute token tree, so it can offer normal method
completion at the original dot.

Consumers that accept complete chains but do not emit typed probe code can parse
with `ChainParseOptions::new().allow_completion_probe(CompletionProbeParsing::Disabled)`
to reject both trailing-dot recovery and explicit marker syntax.

```rust
use attribute_dsl::{AttributeChain, ChainParseOptions, CompletionProbeParsing};
use quote::quote;

let options = ChainParseOptions::new().completion_marker("completeHere");
let chain = AttributeChain::parse_tokens_with_options(
    quote!(RootType::<_>.first(1).),
    &options,
)?;

assert!(chain.has_completion_probe());
assert_eq!(
    chain.completion_marker().unwrap().to_string(),
    "completeHere"
);

let strict = ChainParseOptions::new()
    .allow_completion_probe(CompletionProbeParsing::Disabled);
assert!(
    AttributeChain::parse_tokens_with_options(
        quote!(RootType::<_>.first(1).),
        &strict,
    )
    .is_err()
);

# Ok::<(), syn::Error>(())
```

Completion probes are also recognized before a comma in a `ChainList`, which
lets an attribute parser recover a partially typed entry while preserving the
remaining entries.

## Infer Helpers

Many attribute DSLs use `_` as a placeholder for a subject type, such as the
field type in a derive macro. This crate provides helpers that operate directly
on `syn` syntax trees:

- `split_terminal_single_type_arg(path, subject)` removes the final path
  segment's single type argument and returns `SingleTypeArg::None`,
  `SingleTypeArg::Infer`, or `SingleTypeArg::Explicit`.
- `substitute_infer_in_path(path, replacement)` substitutes `_` inside path
  arguments.
- `substitute_infer_in_type(ty, replacement)` substitutes `_` inside supported
  `syn::Type` forms, including arrays, slices, pointers, function types, trait
  objects, `impl Trait`, tuples, references, parenthesized types, and grouped
  types.
- `substitute_infer_in_expr(expr, replacement)` visits expression syntax and
  substitutes `_` in nested paths and types.

```rust
use attribute_dsl::{
    split_terminal_single_type_arg, substitute_infer_in_expr, substitute_infer_in_path,
    substitute_infer_in_type,
};
use quote::ToTokens as _;
use syn::{Expr, Path, Type, parse_quote};

let path: Path = parse_quote!(RootType::<_>);
let (base_path, type_arg) = split_terminal_single_type_arg(path, "root")?;

assert_eq!(
    base_path.segments.last().unwrap().ident.to_string(),
    "RootType"
);
assert!(type_arg.is_infer());

let replacement: Type = parse_quote!(i32);

let path: Path = parse_quote!(RootType::<Option<_>>);
let substituted_path = substitute_infer_in_path(&path, &replacement);
assert!(
    substituted_path
        .to_token_stream()
        .to_string()
        .contains("i32")
);

let ty: Type = parse_quote!(fn([_; 2], &[_]) -> Option<_>);
let substituted_type = substitute_infer_in_type(&ty, &replacement);
assert!(
    substituted_type
        .to_token_stream()
        .to_string()
        .contains("i32")
);

let expr: Expr = parse_quote!(RootType::<_>.first(1));
let substituted_expr = substitute_infer_in_expr(&expr, &replacement);
assert!(
    substituted_expr
        .to_token_stream()
        .to_string()
        .contains("i32")
);

# Ok::<(), syn::Error>(())
```
