# attribute-dsl integration patterns

## Choose the outer parser

| Attribute arguments | Parser |
|---|---|
| `Root::<_>.first(1)` | `AttributeChain` |
| `name = Root::<_>` | `ChainEntry` |
| `Root::<_>, name = Other::<i32>` | `ChainList` |
| `fields(Root::<_>, name = Other::<i32>)` | `NamedChainGroup` |

Parse through the attribute to preserve its source spans:

```rust
let chain = attr.parse_args::<attribute_dsl::AttributeChain>()?;
let entry = attr.parse_args::<attribute_dsl::ChainEntry>()?;
let list = attr.parse_args::<attribute_dsl::ChainList>()?;
let group = attr.parse_args::<attribute_dsl::NamedChainGroup>()?;
```

Use only the parser matching the consumer's complete outer grammar.

## Substitute the subject type

Replace `_` in a parsed root with the annotated field type:

```rust
let root = attribute_dsl::substitute_infer_in_path(
    chain.root_path(),
    &field.ty,
);
```

Use `substitute_infer_in_type` for a standalone `syn::Type` and
`substitute_infer_in_expr` for paths and types nested in a `syn::Expr`. Use
`split_terminal_single_type_arg(path, "validator")?` when absent, inferred, and
explicit final type arguments select different expansion behavior.

## Quote calls and completion

Preserve every parsed call component:

```rust
let calls = chain.calls().iter().map(|call| {
    let method = call.method();
    let turbofish = call.turbofish();
    let args = call.args();
    quote::quote! { .#method #turbofish (#(#args),*) }
});

let completion = chain
    .completion_marker()
    .map(|marker| quote::quote! { .#marker })
    .unwrap_or_default();

let expanded = quote::quote! {
    #root::builder_for("field") #(#calls)* #completion
};
```

The generated constructor must return the real receiver type before the marker
access.

## Configure direct chain parsing

Use custom options when parsing `AttributeChain` directly:

```rust
let options = attribute_dsl::ChainParseOptions::new()
    .completion_marker("completeHere");
let chain = attr.parse_args_with(|input| {
    attribute_dsl::AttributeChain::parse_with_options(input, &options)
})?;
```

Disable both trailing-dot recovery and explicit marker syntax when the consumer
does not emit typed completion probes:

```rust
let options = attribute_dsl::ChainParseOptions::new().allow_completion_probe(
    attribute_dsl::CompletionProbeParsing::Disabled,
);
```

The composite `Parse` implementations for entries, lists, and groups use the
default chain options.

## Focus consumer tests

- Accept root-only chains and ordered calls with arguments and turbofish.
- Accept labels, empty lists or groups, and trailing commas when the grammar
  exposes those forms.
- Reject associated-constructor roots, arbitrary expressions, and ordinary
  field access.
- Cover absent, `_`, and explicit terminal type arguments when they select
  different behavior.
- Cover trailing-dot input and disabled probes when completion behavior is part
  of the contract.
