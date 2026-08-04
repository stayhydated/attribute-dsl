# Substitute infer placeholders

Use the infer helpers when `_` in an attribute stands for a subject type known
by the macro, such as an annotated field's type. Each helper returns a new
syntax tree and leaves the input node available to the caller.

| Helper | Use it for |
|---|---|
| `split_terminal_single_type_arg` | Distinguish an absent, inferred, or explicit final type argument. |
| `substitute_infer_in_path` | Replace `_` inside path arguments. |
| `substitute_infer_in_type` | Replace `_` in supported nested type forms. |
| `substitute_infer_in_expr` | Replace `_` in paths and types nested in an expression. |

## Inspect a terminal type argument

`split_terminal_single_type_arg` consumes a path, removes one terminal type
argument, and returns `SingleTypeArg::None`, `SingleTypeArg::Infer`, or
`SingleTypeArg::Explicit`:

```rust,ignore
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
The helper returns a `syn::Error` for multiple arguments, non-type arguments,
or parenthesized arguments on the final segment.

## Substitute nested placeholders

The path helper traverses generic arguments on every segment, including nested
types, associated type values and constraints, and parenthesized function
arguments and results.

The type helper supports:

- `_` and paths with type arguments;
- arrays, slices, raw pointers, and references;
- bare function inputs and results;
- trait-object and `impl Trait` bounds;
- tuples; and
- parenthesized and grouped types.

The expression helper traverses the expression and applies the path and type
rules wherever those nodes occur. A `syn::Type` variant outside the listed
forms is cloned unchanged, so choose a consumer grammar whose placeholders are
within the supported forms.

```rust,ignore
use attribute_dsl::{
    substitute_infer_in_expr, substitute_infer_in_path,
    substitute_infer_in_type,
};
use quote::ToTokens as _;
use syn::{Expr, Path, Type, parse_quote};

let replacement: Type = parse_quote!(i32);

let path: Path = parse_quote!(RootType::<Option<_>>);
let path = substitute_infer_in_path(&path, &replacement);
assert!(path.to_token_stream().to_string().contains("i32"));

let ty: Type = parse_quote!(fn([_; 2], &[_]) -> Option<_>);
let ty = substitute_infer_in_type(&ty, &replacement);
assert!(ty.to_token_stream().to_string().contains("i32"));

let expr: Expr = parse_quote!(RootType::<_>.first(Vec::<_>::new()));
let expr = substitute_infer_in_expr(&expr, &replacement);
assert!(expr.to_token_stream().to_string().contains("i32"));
```

Keep the replacement as a `syn::Type` and quote the returned tree directly.
Converting through source strings discards syntax and span information needed
for precise diagnostics.
