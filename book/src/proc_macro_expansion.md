# Build a proc-macro expansion

Build an expansion by parsing each matching attribute, substituting its subject
type, preserving every call in order, and appending a completion marker only
when the parsed chain contains one.

## Expand one attribute

For each attribute:

1. Parse the arguments with `attr.parse_args::<AttributeChain>()?`.
2. Substitute the field type into `chain.root_path()`.
3. Quote each call's method, optional turbofish, and arguments.
4. Quote `chain.completion_marker()` after the calls when it exists.
5. Place the result after an application-owned constructor that returns the
   desired receiver type.

```rust,ignore
use attribute_dsl::{AttributeChain, substitute_infer_in_path};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Type};

fn expand_attribute(
    attr: &Attribute,
    field_ty: &Type,
    field_name: &str,
) -> syn::Result<TokenStream> {
    let chain = attr.parse_args::<AttributeChain>()?;
    let root = substitute_infer_in_path(chain.root_path(), field_ty);

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

    Ok(quote! {
        #root::builder_for(#field_name) #(#calls)* #completion
    })
}
```

The consumer owns `builder_for`, its arguments, and the final generated item.
Keeping construction outside the parser lets multiple macro crates share the
chain grammar while producing different domain-specific code.

## Preserve completion typing

A complete chain has no marker, so its normal expansion ends after the final
call. For trailing-dot input, the marker must follow the same constructor and
calls so rust-analyzer sees the real receiver type at the cursor. Disable probe
parsing when the expansion cannot maintain that invariant.

## Report and test failures

Return parsing and semantic failures as `syn::Error` values. Use
`syn::Error::new_spanned` for consumer-owned checks so the diagnostic points to
the relevant root, method, or argument.

Test at least:

- a root-only chain;
- ordered calls with arguments and turbofish syntax;
- inferred and explicit subject types;
- rejected non-path roots; and
- trailing-dot input when completion probes are enabled.

The repository's `examples/derive_field_attrs.rs` shows the same workflow in a
complete executable derive-style example.
