use syn::spanned::Spanned as _;
use syn::visit_mut::{self, VisitMut as _};
use syn::{
    AngleBracketedGenericArguments, Error, Expr, GenericArgument, Path, PathArguments, Result,
    Token, Type,
};

/// Single terminal type argument split from a path.
#[derive(Clone, Debug)]
pub enum SingleTypeArg {
    None,
    Infer,
    Explicit(Box<Type>),
}

impl SingleTypeArg {
    pub fn explicit_type(&self) -> Option<&Type> {
        match self {
            Self::Explicit(ty) => Some(ty),
            Self::None | Self::Infer => None,
        }
    }

    pub fn is_infer(&self) -> bool {
        matches!(self, Self::Infer)
    }
}

/// Split a path's final generic argument into a normalized single type arg.
///
/// This is useful for DSLs where `Thing::<_>` means "infer the field type" and
/// `Thing::<T>` pins an explicit target type.
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

            let arg = angle_args.args.pop().expect("len checked").into_value();
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

/// Substitute `replacement` for every `_` occurrence inside a type.
pub fn substitute_infer_in_type(ty: &Type, replacement: &Type) -> Type {
    match ty {
        Type::Infer(_) => replacement.clone(),
        Type::Path(type_path) => {
            let mut type_path = type_path.clone();
            type_path.path = substitute_infer_in_path(&type_path.path, replacement);
            Type::Path(type_path)
        },
        Type::Array(array) => {
            let mut array = array.clone();
            array.elem = Box::new(substitute_infer_in_type(&array.elem, replacement));
            Type::Array(array)
        },
        Type::Slice(slice) => {
            let mut slice = slice.clone();
            slice.elem = Box::new(substitute_infer_in_type(&slice.elem, replacement));
            Type::Slice(slice)
        },
        Type::Ptr(ptr) => {
            let mut ptr = ptr.clone();
            ptr.elem = Box::new(substitute_infer_in_type(&ptr.elem, replacement));
            Type::Ptr(ptr)
        },
        Type::BareFn(bare_fn) => {
            let mut bare_fn = bare_fn.clone();
            for input in &mut bare_fn.inputs {
                input.ty = substitute_infer_in_type(&input.ty, replacement);
            }
            substitute_infer_in_return_type(&mut bare_fn.output, replacement);
            Type::BareFn(bare_fn)
        },
        Type::TraitObject(trait_object) => {
            let mut trait_object = trait_object.clone();
            substitute_infer_in_bounds(&mut trait_object.bounds, replacement);
            Type::TraitObject(trait_object)
        },
        Type::ImplTrait(impl_trait) => {
            let mut impl_trait = impl_trait.clone();
            substitute_infer_in_bounds(&mut impl_trait.bounds, replacement);
            Type::ImplTrait(impl_trait)
        },
        Type::Tuple(tuple) => {
            let mut tuple = tuple.clone();
            tuple.elems = tuple
                .elems
                .iter()
                .map(|ty| substitute_infer_in_type(ty, replacement))
                .collect();
            Type::Tuple(tuple)
        },
        Type::Paren(paren) => {
            let mut paren = paren.clone();
            paren.elem = Box::new(substitute_infer_in_type(&paren.elem, replacement));
            Type::Paren(paren)
        },
        Type::Group(group) => {
            let mut group = group.clone();
            group.elem = Box::new(substitute_infer_in_type(&group.elem, replacement));
            Type::Group(group)
        },
        Type::Reference(reference) => {
            let mut reference = reference.clone();
            *reference.elem = substitute_infer_in_type(&reference.elem, replacement);
            Type::Reference(reference)
        },
        _ => ty.clone(),
    }
}

/// Substitute `replacement` for every `_` occurrence inside an expression.
pub fn substitute_infer_in_expr(expr: &Expr, replacement: &Type) -> Expr {
    let mut expr = expr.clone();
    InferSubstitutor { replacement }.visit_expr_mut(&mut expr);
    expr
}

/// Substitute `replacement` for every `_` occurrence inside path arguments.
pub fn substitute_infer_in_path(path: &Path, replacement: &Type) -> Path {
    let mut path = path.clone();

    for segment in &mut path.segments {
        substitute_infer_in_path_arguments(&mut segment.arguments, replacement);
    }

    path
}

struct InferSubstitutor<'a> {
    replacement: &'a Type,
}

impl visit_mut::VisitMut for InferSubstitutor<'_> {
    fn visit_type_mut(&mut self, node: &mut Type) {
        *node = substitute_infer_in_type(node, self.replacement);
    }

    fn visit_path_mut(&mut self, node: &mut Path) {
        *node = substitute_infer_in_path(node, self.replacement);
    }
}

fn substitute_infer_in_return_type(return_type: &mut syn::ReturnType, replacement: &Type) {
    if let syn::ReturnType::Type(_, ty) = return_type {
        **ty = substitute_infer_in_type(ty, replacement);
    }
}

fn substitute_infer_in_bounds(
    bounds: &mut syn::punctuated::Punctuated<syn::TypeParamBound, Token![+]>,
    replacement: &Type,
) {
    for bound in bounds {
        if let syn::TypeParamBound::Trait(trait_bound) = bound {
            trait_bound.path = substitute_infer_in_path(&trait_bound.path, replacement);
        }
    }
}

fn substitute_infer_in_path_arguments(arguments: &mut PathArguments, replacement: &Type) {
    match arguments {
        PathArguments::AngleBracketed(args) => {
            substitute_infer_in_angle_bracketed_arguments(args, replacement);
        },
        PathArguments::Parenthesized(args) => {
            args.inputs = args
                .inputs
                .iter()
                .map(|ty| substitute_infer_in_type(ty, replacement))
                .collect();
            substitute_infer_in_return_type(&mut args.output, replacement);
        },
        PathArguments::None => {},
    }
}

fn substitute_infer_in_angle_bracketed_arguments(
    args: &mut AngleBracketedGenericArguments,
    replacement: &Type,
) {
    for arg in &mut args.args {
        match arg {
            GenericArgument::Type(ty) => {
                *ty = substitute_infer_in_type(ty, replacement);
            },
            GenericArgument::AssocType(assoc_type) => {
                if let Some(generics) = &mut assoc_type.generics {
                    substitute_infer_in_angle_bracketed_arguments(generics, replacement);
                }
                assoc_type.ty = substitute_infer_in_type(&assoc_type.ty, replacement);
            },
            GenericArgument::Constraint(constraint) => {
                if let Some(generics) = &mut constraint.generics {
                    substitute_infer_in_angle_bracketed_arguments(generics, replacement);
                }
                substitute_infer_in_bounds(&mut constraint.bounds, replacement);
            },
            _ => {},
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
    fn substitutes_infer_in_paths_types_and_exprs() {
        let replacement: Type = parse_quote!(String);
        let path: Path = parse_quote!(crate::Input<Option<_>>);
        assert_eq!(
            compact(&substitute_infer_in_path(&path, &replacement)),
            "crate::Input<Option<String>>"
        );

        let ty: Type = parse_quote!(fn([_; 2], &[_]) -> Option<_>);
        assert_eq!(
            compact(&substitute_infer_in_type(&ty, &replacement)),
            "fn([String;2],&[String])->Option<String>"
        );

        let expr: Expr = parse_quote!(crate::Select::<_>.searchable(true));
        assert_eq!(
            compact(&substitute_infer_in_expr(&expr, &replacement)),
            "crate::Select::<String>.searchable(true)"
        );
    }
}
