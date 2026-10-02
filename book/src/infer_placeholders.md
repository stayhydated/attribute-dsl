# Substitute infer placeholders

Use the infer helpers when `_` in an attribute stands for a subject type known
by the macro, such as an annotated field's type. The substitution helpers
return a new syntax tree and leave the input node available to the caller.

| Helper | Use it for |
|---|---|
| `split_terminal_single_type_arg` | Distinguish an absent, inferred, or explicit final type argument. |
| `substitute_infer_in_path` | Replace `_` inside path arguments. |
| `substitute_infer_in_type` | Replace `_` in parsed nested type nodes. |
| `substitute_infer_in_expr` | Replace `_` in paths and types nested in an expression. |

## Inspect a terminal type argument

`split_terminal_single_type_arg` consumes a path, removes its final segment's
generic arguments, and returns `SingleTypeArg::None`, `SingleTypeArg::Infer`, or
`SingleTypeArg::Explicit`:

```rust
# extern crate attribute_dsl;
# extern crate syn;
use attribute_dsl::{SingleTypeArg, split_terminal_single_type_arg};
use syn::{Path, parse_quote};

let path: Path = parse_quote!(RootType::<_>);
let (root, argument) = split_terminal_single_type_arg(path, "validator")?;

assert_eq!(
    root.segments
        .last()
        .expect("a parsed path has a segment")
        .ident
        .to_string(),
    "RootType"
);
assert!(matches!(argument, SingleTypeArg::Infer));

# Ok::<(), syn::Error>(())
```

The subject string appears in diagnostics. Use the consumer's domain term,
such as `"validator"` or `"component"`, so errors identify the invalid path.
Omitting generic arguments produces `SingleTypeArg::None`. When angle brackets
are present, they must contain exactly one type argument. Empty brackets,
multiple arguments, non-type arguments, and parenthesized arguments on the
final segment produce a `syn::Error`.

## Substitute nested placeholders

All three helpers use `syn`'s syntax-tree traversal to replace `_` type nodes.
This includes generic arguments, associated type values and constraints,
function arguments and results, qualified-self types, and types nested in const
expressions such as array lengths.

Macro token bodies and other unparsed tokens remain unchanged. Expression
placeholders are not type placeholders and are also left unchanged. Traversal
stops at each replacement, so `_` inside the replacement type stays available
for the consumer to resolve.

```rust
# extern crate attribute_dsl;
# extern crate syn;
use attribute_dsl::{
    substitute_infer_in_expr, substitute_infer_in_path,
    substitute_infer_in_type,
};
use syn::{Expr, Path, Type, parse_quote};

let replacement: Type = parse_quote!(i32);

let path: Path = parse_quote!(RootType::<Option<_>>);
let path = substitute_infer_in_path(&path, &replacement);
assert_eq!(path, parse_quote!(RootType::<Option<i32>>));

let ty: Type = parse_quote!(fn([_; 2], &[_]) -> Option<_>);
let ty = substitute_infer_in_type(&ty, &replacement);
assert_eq!(ty, parse_quote!(fn([i32; 2], &[i32]) -> Option<i32>));

let qualified: Type = parse_quote!(<Vec<_> as Trait>::Item);
assert_eq!(
    substitute_infer_in_type(&qualified, &replacement),
    parse_quote!(<Vec<i32> as Trait>::Item)
);

let array: Type = parse_quote!([u8; size_of::<_>()]);
assert_eq!(
    substitute_infer_in_type(&array, &replacement),
    parse_quote!([u8; size_of::<i32>()])
);

let expr: Expr = parse_quote!(RootType::<_>.first(Vec::<_>::new()));
let expr = substitute_infer_in_expr(&expr, &replacement);
assert_eq!(expr, parse_quote!(RootType::<i32>.first(Vec::<i32>::new())));
```

Keep the replacement as a `syn::Type` and quote the returned tree directly.
Converting through source strings discards syntax and span information needed
for precise diagnostics.
