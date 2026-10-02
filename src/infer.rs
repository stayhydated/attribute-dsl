use syn::spanned::Spanned as _;
use syn::visit_mut::{self, VisitMut as _};
use syn::{Error, Expr, GenericArgument, Path, PathArguments, Result, Type};

/// Single terminal type argument split from a path.
#[derive(Clone, Debug)]
pub enum SingleTypeArg {
    /// The path's final segment had no generic arguments.
    None,
    /// The path's final segment used `_` as its only type argument.
    Infer,
    /// The path's final segment used one explicit type argument.
    Explicit(Box<Type>),
}

impl SingleTypeArg {
    /// Return the explicit type argument, if one was present.
    pub fn explicit_type(&self) -> Option<&Type> {
        match self {
            Self::Explicit(ty) => Some(ty),
            Self::None | Self::Infer => None,
        }
    }

    /// Return whether the terminal type argument was `_`.
    pub fn is_infer(&self) -> bool {
        matches!(self, Self::Infer)
    }
}

/// Split a path's final generic argument into a normalized single type arg.
///
/// This is useful for DSLs where `Thing::<_>` means "infer the field type" and
/// `Thing::<T>` pins an explicit target type.
///
/// # Errors
///
/// Returns [`syn::Error`] when the path has no final segment, the final segment
/// has more than one generic argument, the argument is not a type, or the final
/// segment uses parenthesized generic arguments.
pub fn split_terminal_single_type_arg(
    mut path: Path,
    subject: &str,
) -> Result<(Path, SingleTypeArg)> {
    let path_span = path.span();
    let last_segment = path
        .segments
        .last_mut()
        .ok_or_else(|| Error::new(path_span, format!("expected {subject} path")))?;

    let args = std::mem::replace(&mut last_segment.arguments, PathArguments::None);
    let type_arg = match args {
        PathArguments::None => SingleTypeArg::None,
        PathArguments::AngleBracketed(mut angle_args) => {
            if angle_args.args.len() != 1 {
                return Err(Error::new(
                    angle_args.span(),
                    format!("{subject} type syntax expects exactly one type argument"),
                ));
            }

            let arg = angle_args.args.pop().expect("len checked");
            match arg {
                GenericArgument::Type(Type::Infer(_)) => SingleTypeArg::Infer,
                GenericArgument::Type(ty) => SingleTypeArg::Explicit(Box::new(ty)),
                _ => Err(Error::new(
                    arg.span(),
                    format!("{subject} type syntax expects a type argument"),
                ))?,
            }
        },
        PathArguments::Parenthesized(args) => {
            return Err(Error::new(
                args.span(),
                format!("{subject} path does not support parenthesized arguments"),
            ));
        },
    };

    Ok((path, type_arg))
}

/// Substitute `replacement` for every `_` type placeholder inside a type.
///
/// Traverses parsed type nodes, including qualified-self types and types inside
/// const expressions. Macro token bodies are opaque and remain unchanged.
/// Placeholders inside the replacement itself are not substituted.
pub fn substitute_infer_in_type(ty: &Type, replacement: &Type) -> Type {
    let mut ty = ty.clone();
    InferSubstitutor { replacement }.visit_type_mut(&mut ty);
    ty
}

/// Substitute `replacement` for every `_` type placeholder inside an expression.
///
/// Uses the same traversal rules as [`substitute_infer_in_type`].
pub fn substitute_infer_in_expr(expr: &Expr, replacement: &Type) -> Expr {
    let mut expr = expr.clone();
    InferSubstitutor { replacement }.visit_expr_mut(&mut expr);
    expr
}

/// Substitute `replacement` for every `_` type placeholder inside path arguments.
///
/// Uses the same traversal rules as [`substitute_infer_in_type`].
pub fn substitute_infer_in_path(path: &Path, replacement: &Type) -> Path {
    let mut path = path.clone();
    InferSubstitutor { replacement }.visit_path_mut(&mut path);
    path
}

struct InferSubstitutor<'a> {
    replacement: &'a Type,
}

impl visit_mut::VisitMut for InferSubstitutor<'_> {
    fn visit_type_mut(&mut self, node: &mut Type) {
        if matches!(node, Type::Infer(_)) {
            *node = self.replacement.clone();
        } else {
            visit_mut::visit_type_mut(self, node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Type, parse_quote};

    fn compact(tokens: impl quote::ToTokens) -> String {
        tokens
            .to_token_stream()
            .to_string()
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect()
    }

    fn parenthesized_path(output: Type) -> Path {
        let mut inputs = syn::punctuated::Punctuated::new();
        inputs.push(syn::NamedArg {
            attrs: Vec::new(),
            name: None,
            ty: parse_quote!(_),
        });

        Path::from(syn::PathSegment {
            ident: parse_quote!(FnOnce),
            arguments: PathArguments::Parenthesized(syn::ParenthesizedGenericArguments {
                paren_token: Default::default(),
                inputs,
                output: syn::ReturnType::Type(Default::default(), Box::new(output)),
            }),
        })
    }

    #[test]
    fn splits_terminal_single_type_arg() {
        let path: Path = parse_quote!(crate::RangeValidation::<_>);
        let (path, arg) = split_terminal_single_type_arg(path, "validator").expect("valid path");
        assert_eq!(compact(&path), "crate::RangeValidation");
        assert!(arg.is_infer());

        let path: Path = parse_quote!(crate::RangeValidation::<i32>);
        let (_, arg) = split_terminal_single_type_arg(path, "validator").expect("valid path");
        assert_eq!(compact(arg.explicit_type().expect("explicit type")), "i32");
    }

    #[test]
    fn splits_absent_terminal_type_arg_and_rejects_invalid_args() {
        let path: Path = parse_quote!(crate::RangeValidation);
        let (path, arg) = split_terminal_single_type_arg(path, "validator").expect("valid path");
        assert_eq!(compact(&path), "crate::RangeValidation");
        assert!(!arg.is_infer());
        assert!(arg.explicit_type().is_none());

        let path: Path = parse_quote!(crate::RangeValidation::<i32, String>);
        let err = split_terminal_single_type_arg(path, "validator").expect_err("too many args");
        assert!(
            err.to_string()
                .contains("validator type syntax expects exactly one type argument"),
            "{err}"
        );

        let path: Path = parse_quote!(crate::RangeValidation::<3>);
        let err = split_terminal_single_type_arg(path, "validator").expect_err("const arg");
        assert!(
            err.to_string()
                .contains("validator type syntax expects a type argument"),
            "{err}"
        );

        let path = parenthesized_path(parse_quote!(i32));
        let err = split_terminal_single_type_arg(path, "validator").expect_err("function args");
        assert!(
            err.to_string()
                .contains("validator path does not support parenthesized arguments"),
            "{err}"
        );
    }

    #[test]
    fn substitutes_infer_in_paths_types_and_exprs() {
        let replacement: Type = parse_quote!(String);
        let path: Path = parse_quote!(crate::Input<Option<_>>);
        assert_eq!(
            compact(substitute_infer_in_path(&path, &replacement)),
            "crate::Input<Option<String>>"
        );

        let ty: Type = parse_quote!(fn([_; 2], &[_]) -> Option<_>);
        assert_eq!(
            compact(substitute_infer_in_type(&ty, &replacement)),
            "fn([String;2],&[String])->Option<String>"
        );

        let expr: Expr = parse_quote!(crate::Select::<_>.searchable(true));
        assert_eq!(
            compact(substitute_infer_in_expr(&expr, &replacement)),
            "crate::Select::<String>.searchable(true)"
        );
    }

    #[test]
    fn substitutes_infer_in_additional_type_forms() {
        let replacement: Type = parse_quote!(String);

        let ptr: Type = parse_quote!(*const _);
        assert_eq!(
            compact(substitute_infer_in_type(&ptr, &replacement)),
            "*constString"
        );

        let trait_object: Type = parse_quote!(dyn Iterator<Item = _> + Send);
        assert_eq!(
            compact(substitute_infer_in_type(&trait_object, &replacement)),
            "dynIterator<Item=String>+Send"
        );

        let impl_trait: Type = parse_quote!(impl Into<_> + Send);
        assert_eq!(
            compact(substitute_infer_in_type(&impl_trait, &replacement)),
            "implInto<String>+Send"
        );

        let tuple: Type = parse_quote!((_, Option<_>));
        assert_eq!(
            compact(substitute_infer_in_type(&tuple, &replacement)),
            "(String,Option<String>)"
        );

        let paren: Type = parse_quote!((Option<_>));
        assert_eq!(
            compact(substitute_infer_in_type(&paren, &replacement)),
            "(Option<String>)"
        );

        let group = Type::Group(syn::TypeGroup {
            attrs: Vec::new(),
            group_token: Default::default(),
            elem: Box::new(parse_quote!(Option<_>)),
        });
        assert_eq!(
            compact(substitute_infer_in_type(&group, &replacement)),
            "Option<String>"
        );

        let never: Type = parse_quote!(!);
        assert_eq!(compact(substitute_infer_in_type(&never, &replacement)), "!");
    }

    #[test]
    fn substitutes_infer_in_path_argument_variants() {
        let replacement: Type = parse_quote!(String);

        let parenthesized = parenthesized_path(parse_quote!(_));
        assert_eq!(
            compact(substitute_infer_in_path(&parenthesized, &replacement)),
            "FnOnce(String)->String"
        );

        let assoc_type: Path = parse_quote!(Trait<Assoc<_> = Result<_, _>>);
        assert_eq!(
            compact(substitute_infer_in_path(&assoc_type, &replacement)),
            "Trait<Assoc<String>=Result<String,String>>"
        );

        let constraint: Path = parse_quote!(Trait<Assoc<_>: Into<_> + From<_>>);
        assert_eq!(
            compact(substitute_infer_in_path(&constraint, &replacement)),
            "Trait<Assoc<String>:Into<String>+From<String>>"
        );

        let lifetime_and_const: Path = parse_quote!(Trait<'static, 3, _>);
        assert_eq!(
            compact(substitute_infer_in_path(&lifetime_and_const, &replacement)),
            "Trait<'static,3,String>"
        );
    }

    #[test]
    fn substitutes_infer_inside_expression_types() {
        let replacement: Type = parse_quote!(String);
        let expr: Expr = parse_quote!(value as *const _);

        assert_eq!(
            compact(substitute_infer_in_expr(&expr, &replacement)),
            "valueas*constString"
        );
    }

    #[test]
    fn substitutes_infer_in_qualified_self_and_const_expressions() {
        let replacement: Type = parse_quote!(String);
        let qualified: Type = parse_quote!(<Vec<_> as Trait>::Item);
        assert_eq!(
            substitute_infer_in_type(&qualified, &replacement),
            parse_quote!(<Vec<String> as Trait>::Item)
        );

        let array: Type = parse_quote!([u8; size_of::<_>()]);
        assert_eq!(
            substitute_infer_in_type(&array, &replacement),
            parse_quote!([u8; size_of::<String>()])
        );

        let path: Path = parse_quote!(Buffer<{ size_of::<_>() }>);
        assert_eq!(
            substitute_infer_in_path(&path, &replacement),
            parse_quote!(Buffer<{ size_of::<String>() }>)
        );
    }

    #[test]
    fn substitution_leaves_macro_tokens_opaque() {
        let input: Type = parse_quote!(SomeMacro!(_));
        let replacement: Type = parse_quote!(String);
        assert_eq!(substitute_infer_in_type(&input, &replacement), input);
    }

    #[test]
    fn substitution_preserves_input_and_infer_inside_replacement() {
        let input: Type = parse_quote!(Result<Vec<_>, [_; 2]>);
        let replacement: Type = parse_quote!(Option<_>);

        let output = substitute_infer_in_type(&input, &replacement);

        assert_eq!(compact(output), "Result<Vec<Option<_>>,[Option<_>;2]>");
        assert_eq!(compact(input), "Result<Vec<_>,[_;2]>");
        assert_eq!(compact(replacement), "Option<_>");
    }
}
