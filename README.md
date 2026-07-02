# attribute-dsl

Shared parser helpers for Rust proc-macro attribute DSLs built from path roots,
dot-call chains, and comma-separated entries.

This crate is intended for derive and attribute macro implementation crates.

## Example

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

## What Stays Outside

Consumers still own their domain grammar:

- reserved words and other domain-specific identifiers;
- how parsed completion probes are handled;
- method arity and reserved method names;
- labels and generated API naming;
- runtime semantics for parsed roots, labels, entries, and calls.

The crate also exposes `_` substitution helpers for paths, types, and
expressions, plus a helper for splitting a path's terminal single type argument.
