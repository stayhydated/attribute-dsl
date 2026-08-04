# Parse chain syntax

Choose the parser that matches the complete attribute arguments. Each parser
retains paths, identifiers, and call arguments as `syn` nodes.

| Input shape | Parser |
|---|---|
| `Root::<_>.first(1)` | `AttributeChain` |
| `label = Root::<_>.first(1)` | `ChainEntry` |
| `Root::<_>, other = Root::<i32>` | `ChainList` |
| `fields(Root::<_>, other = Root::<i32>)` | `NamedChainGroup` |

## Supported grammar

An `AttributeChain` starts with a Rust path. Each following call requires a dot,
a method name, and parentheses. Calls can include turbofish arguments and
comma-separated `syn::Expr` arguments.

```text
AttributeChain  := Path ("." Ident Turbofish? "(" Expr,* ")")* CompletionProbe?
CompletionProbe := "." CompletionMarker
ChainEntry      := (Ident "=")? AttributeChain
ChainList       := (ChainEntry ("," ChainEntry)* ","?)?
NamedChainGroup := Ident "(" ChainList ")"
```

`Path` includes module-qualified and absolute paths with normal Rust generic
arguments. Parentheses around a complete chain are accepted and normalized to
the same parsed model. A `ChainList` and the contents of a `NamedChainGroup` may
be empty, and lists may end with a comma.

## Inspect parsed values

Use the accessors instead of reparsing tokens:

```rust,ignore
use attribute_dsl::{AttributeChain, ChainList, NamedChainGroup};

let chain: AttributeChain = syn::parse_str("Root::<i32>.first(1)")?;
let first_call = &chain.calls()[0];
assert_eq!(first_call.method().to_string(), "first");
assert_eq!(first_call.args().len(), 1);

let list: ChainList =
    syn::parse_str("value = Root::<_>.first(1), Root::<String>")?;
assert_eq!(list.entries().len(), 2);
assert_eq!(
    list.entries()[0]
        .label()
        .expect("a labeled entry")
        .to_string(),
    "value"
);

let group: NamedChainGroup =
    syn::parse_str("fields(value = Root::<_>, Root::<String>)")?;
assert_eq!(group.name().to_string(), "fields");
assert_eq!(group.entries().len(), 2);

# Ok::<(), syn::Error>(())
```

`AttributeChain` and `ChainCall` implement `quote::ToTokens`. An expansion that
changes the root or completion behavior should quote the root, calls, and probe
separately instead.

## Keep constructors in the expansion

The chain root must remain a `syn::Path`; arbitrary Rust expressions are not
accepted. This boundary keeps parsing predictable and leaves construction in
the consumer macro.

| Unsupported input | Action |
|---|---|
| `Root::builder().first(1)` | Parse `Root.first(1)`, then emit `Root::builder()` before the calls. |
| `Root.field` | Use a method call, or reserve the configured terminal marker for completion probes. |
| `left + right` | Parse the expression with `syn` directly instead of `AttributeChain`. |

Perform domain checks after parsing and attach each `syn::Error` to the
narrowest relevant path, method, or argument node.
