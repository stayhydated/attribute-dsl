# Emit completion probes

Completion probes let an incomplete trailing dot produce rust-analyzer method
completion inside an attribute token tree. Use them only when the macro emits
the marker after a real typed receiver.

## Handle a trailing dot

With the default options, `Root::<_>.first(1).` parses as a chain whose terminal
marker is `raCompletionMarker`:

```rust,ignore
use attribute_dsl::AttributeChain;
use quote::quote;

let chain: AttributeChain = syn::parse_str("Root::<_>.first(1).")?;
assert!(chain.has_completion_probe());

let completion = chain
    .completion_marker()
    .map(|marker| quote! { .#marker })
    .unwrap_or_default();

# Ok::<(), syn::Error>(())
```

Append `completion` after the application-owned constructor and parsed calls.
At that position, the missing marker method gives rust-analyzer a typed
receiver and allows it to offer the receiver's methods at the original dot.
The explicit terminal syntax `Root::<_>.raCompletionMarker` represents the same
completion state.

## Configure the marker

Use one stable Rust identifier when the consumer needs a custom marker:

```rust,ignore
use attribute_dsl::{AttributeChain, ChainParseOptions};
use quote::quote;

let options = ChainParseOptions::new().completion_marker("completeHere");
let chain = AttributeChain::parse_tokens_with_options(
    quote!(Root::<_>.first(1).),
    &options,
)?;

assert_eq!(
    chain
        .completion_marker()
        .expect("the input ends with a probe")
        .to_string(),
    "completeHere"
);

# Ok::<(), syn::Error>(())
```

An invalid identifier produces a `syn::Error` when trailing-dot recovery needs
to emit it. Custom `ChainParseOptions` apply to direct `AttributeChain` parsing;
the `Parse` implementations for entries, lists, and groups use the defaults.

## Reject probe syntax

Disable completion probes when the macro accepts only complete chains or cannot
emit a typed receiver:

```rust,ignore
use attribute_dsl::{AttributeChain, ChainParseOptions, CompletionProbeParsing};
use quote::quote;

let options = ChainParseOptions::new()
    .allow_completion_probe(CompletionProbeParsing::Disabled);
let result = AttributeChain::parse_tokens_with_options(
    quote!(Root::<_>.first(1).),
    &options,
);

assert!(result.is_err());
```

Disabling probes rejects both trailing-dot recovery and an explicit terminal
completion marker.

## Preserve later list entries

Default completion recovery stops before a comma in `ChainList`, so later
entries remain available while one entry is incomplete:

```rust,ignore
use attribute_dsl::ChainList;

let list: ChainList = syn::parse_str(
    "first = Root::<_>., second = Other::<String>",
)?;

assert!(list.entries()[0].chain().has_completion_probe());
assert_eq!(list.entries().len(), 2);

# Ok::<(), syn::Error>(())
```

If parsing succeeds but rust-analyzer offers no methods, inspect the generated
expression. The constructor and all preceding calls must resolve to the desired
receiver type before the marker access.
