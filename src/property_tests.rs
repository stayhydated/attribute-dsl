use super::*;
use proc_macro2::TokenStream;
use proptest::prelude::*;
use quote::ToTokens as _;
use syn::{Expr, Path, Type};

fn property_config() -> ProptestConfig {
    let mut config = ProptestConfig::default();
    if std::env::var_os("PROPTEST_CASES").is_none() {
        config.cases = 64;
    }
    config
}

#[derive(Clone, Debug)]
enum Argument {
    Number(i16),
    Text(String),
    Qualified(u8),
}

impl Argument {
    fn source(&self) -> String {
        match self {
            Self::Number(value) => value.to_string(),
            Self::Text(value) => format!("{value:?}"),
            Self::Qualified(id) => format!("<T{id} as Trait>::VALUE"),
        }
    }
}

#[derive(Clone, Debug)]
struct Call {
    id: u8,
    generic: bool,
    args: Vec<Argument>,
}

#[derive(Clone, Debug)]
struct Chain {
    roots: Vec<u8>,
    absolute: bool,
    calls: Vec<Call>,
    probe: u8,
}

impl Chain {
    fn root_source(&self) -> String {
        let prefix = if self.absolute { "::" } else { "" };
        let segments = self
            .roots
            .iter()
            .map(|id| format!("Root{id}"))
            .collect::<Vec<_>>();
        format!("{prefix}{}::<_>", segments.join("::"))
    }

    fn source(&self, marker: &str) -> String {
        let mut source = self.root_source();
        for call in &self.calls {
            let generic = if call.generic { "::<String>" } else { "" };
            let args = call.args.iter().map(Argument::source).collect::<Vec<_>>();
            source.push_str(&format!(".method{}{generic}({})", call.id, args.join(",")));
        }
        match self.probe {
            1 => source.push('.'),
            2 => source.push_str(&format!(".{marker}")),
            _ => {},
        }
        source
    }

    fn check(&self, chain: &AttributeChain, marker: &str) -> Result<(), TestCaseError> {
        let expected_root: Path = syn::parse_str(&self.root_source()).unwrap();
        prop_assert_eq!(chain.root(), &expected_root);
        prop_assert_eq!(chain.calls().len(), self.calls.len());
        for (actual, expected) in chain.calls().iter().zip(&self.calls) {
            prop_assert_eq!(
                actual.method().to_string(),
                format!("method{}", expected.id)
            );
            prop_assert_eq!(actual.turbofish().is_some(), expected.generic);
            if let Some(generic) = actual.turbofish() {
                prop_assert_eq!(generic.args.len(), 1);
                prop_assert_eq!(generic.args.first().unwrap(), &syn::parse_quote!(String));
            }
            let expected_args = expected
                .args
                .iter()
                .map(|arg| syn::parse_str::<Expr>(&arg.source()).unwrap())
                .collect::<Vec<_>>();
            prop_assert_eq!(actual.args(), expected_args.as_slice());
        }
        prop_assert_eq!(chain.has_completion_probe(), self.probe != 0);
        prop_assert_eq!(
            chain.completion_marker().map(ToString::to_string),
            (self.probe != 0).then(|| marker.to_owned())
        );
        Ok(())
    }
}

fn chains() -> impl Strategy<Value = Chain> {
    let argument = prop_oneof![
        any::<i16>().prop_map(Argument::Number),
        "[a-z ,]{0,12}".prop_map(Argument::Text),
        (0u8..4).prop_map(Argument::Qualified),
    ];
    let call = (0u8..4, any::<bool>(), prop::collection::vec(argument, 0..4))
        .prop_map(|(id, generic, args)| Call { id, generic, args });
    (
        prop::collection::vec(0u8..4, 1..5),
        any::<bool>(),
        prop::collection::vec(call, 0..9),
        0u8..3,
    )
        .prop_map(|(roots, absolute, calls, probe)| Chain {
            roots,
            absolute,
            calls,
            probe,
        })
}

// This syntax model renders the expected result directly; it does not use syn's
// visitor or inspect the production output to decide where replacement occurs.
#[derive(Clone, Debug)]
enum TypeSyntax {
    Infer,
    Atom(u8),
    Opaque(u8),
    Option(Box<Self>),
    Reference(Box<Self>),
    Tuple(Vec<Self>),
    Qualified(Box<Self>),
    ConstExpression(Box<Self>),
}

impl TypeSyntax {
    fn source(&self, infer: &str) -> String {
        match self {
            Self::Infer => infer.to_owned(),
            Self::Atom(id) => format!("T{id}"),
            Self::Opaque(id) => format!("Opaque!(_, token{id})"),
            Self::Option(inner) => format!("Option<{}>", inner.source(infer)),
            Self::Reference(inner) => format!("&'static {}", inner.source(infer)),
            Self::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|item| format!("{},", item.source(infer)))
                    .collect::<String>();
                format!("({items})")
            },
            Self::Qualified(inner) => format!("<{} as Trait>::Item", inner.source(infer)),
            Self::ConstExpression(inner) => format!("[u8; size_of::<{}>()]", inner.source(infer)),
        }
    }
}

fn types() -> impl Strategy<Value = TypeSyntax> {
    prop_oneof![
        Just(TypeSyntax::Infer),
        (0u8..4).prop_map(TypeSyntax::Atom),
        (0u8..4).prop_map(TypeSyntax::Opaque)
    ]
    .prop_recursive(4, 32, 4, |inner| {
        prop_oneof![
            inner
                .clone()
                .prop_map(|ty| TypeSyntax::Option(Box::new(ty))),
            inner
                .clone()
                .prop_map(|ty| TypeSyntax::Reference(Box::new(ty))),
            prop::collection::vec(inner.clone(), 0..4).prop_map(TypeSyntax::Tuple),
            inner
                .clone()
                .prop_map(|ty| TypeSyntax::Qualified(Box::new(ty))),
            inner.prop_map(|ty| TypeSyntax::ConstExpression(Box::new(ty))),
        ]
    })
}

proptest! {
    #![proptest_config(property_config())]

    #[test]
    fn chains_preserve_model_and_completion(chain in chains(), marker_id in 0u8..4) {
        let marker = format!("complete{marker_id}");
        let options = ChainParseOptions::new().completion_marker(&marker);
        let tokens: TokenStream = chain.source(&marker).parse().unwrap();
        let parsed = AttributeChain::parse_tokens_with_options(tokens.clone(), &options).unwrap();
        chain.check(&parsed, &marker)?;
        let reparsed = AttributeChain::parse_tokens_with_options(parsed.to_token_stream(), &options).unwrap();
        chain.check(&reparsed, &marker)?;
        let disabled = options.allow_completion_probe(CompletionProbeParsing::Disabled);
        let result = AttributeChain::parse_tokens_with_options(tokens, &disabled);
        prop_assert_eq!(result.is_ok(), chain.probe == 0);
    }

    #[test]
    fn lists_and_groups_preserve_entry_order(entries in prop::collection::vec((any::<bool>(), chains()), 0..9)) {
        let sources = entries.iter().enumerate().map(|(index, (labeled, chain))| {
            let label = if *labeled { format!("label{index} = ") } else { String::new() };
            format!("{label}{}", chain.source(DEFAULT_COMPLETION_MARKER))
        }).collect::<Vec<_>>();
        let source = sources.join(",");
        let list: ChainList = syn::parse_str(&source).unwrap();
        let group: NamedChainGroup = syn::parse_str(&format!("each({source})")).unwrap();
        prop_assert_eq!(group.name(), "each");
        prop_assert_eq!(list.entries().len(), entries.len());
        prop_assert_eq!(group.entries().len(), entries.len());
        for (index, (labeled, expected)) in entries.iter().enumerate() {
            for actual in [&list.entries()[index], &group.entries()[index]] {
                prop_assert_eq!(actual.label().map(ToString::to_string), labeled.then(|| format!("label{index}")));
                expected.check(actual.chain(), DEFAULT_COMPLETION_MARKER)?;
            }
        }
    }

    #[test]
    fn qualified_roots_are_rejected_even_during_completion(id in 0u8..4, probe in any::<bool>()) {
        let suffix = if probe { "." } else { ".method(1)" };
        let source = format!("<T{id} as Trait>::Builder{suffix}");
        let result = syn::parse_str::<AttributeChain>(&source);
        prop_assert!(result.is_err());
        prop_assert!(result.unwrap_err().to_string().contains("qualified self"));
    }

    #[test]
    fn substitution_matches_constructed_syntax(
        ty in types(),
        replacement in prop_oneof![Just("R"), Just("Option<_>"), Just("<Vec<_> as Trait>::Item")],
    ) {
        let source = ty.source("_");
        let expected = ty.source(replacement);
        let replacement: Type = syn::parse_str(replacement).unwrap();
        let input: Type = syn::parse_str(&source).unwrap();
        let input_before = input.clone();
        let replacement_before = replacement.clone();
        let output = substitute_infer_in_type(&input, &replacement);
        prop_assert_eq!(&output, &syn::parse_str::<Type>(&expected).unwrap());
        let path: Path = syn::parse_str(&format!("Wrapper<{source}>")).unwrap();
        let expr: Expr = syn::parse_str(&format!("consume::<{source}>(<T as Trait>::VALUE)")).unwrap();
        prop_assert_eq!(
            substitute_infer_in_path(&path, &replacement),
            syn::parse_str::<Path>(&format!("Wrapper<{expected}>")).unwrap()
        );
        prop_assert_eq!(
            substitute_infer_in_expr(&expr, &replacement),
            syn::parse_str::<Expr>(&format!("consume::<{expected}>(<T as Trait>::VALUE)")).unwrap()
        );
        prop_assert_eq!(input, input_before);
        prop_assert_eq!(&replacement, &replacement_before);
        if replacement_before == syn::parse_quote!(R) {
            prop_assert_eq!(substitute_infer_in_type(&output, &replacement_before), output);
        }
    }

    #[test]
    fn terminal_split_preserves_earlier_arguments(ty in types(), kind in 0u8..3) {
        let prefix = format!("module::Prefix<{}>::Target", ty.source("_"));
        let suffix = match kind {
            0 => String::new(),
            1 => "::<_>".to_owned(),
            _ => format!("::<Option<{}>>", ty.source("_")),
        };
        let path: Path = syn::parse_str(&format!("{prefix}{suffix}")).unwrap();
        let (stripped, arg) = split_terminal_single_type_arg(path, "subject").unwrap();
        prop_assert_eq!(stripped, syn::parse_str::<Path>(&prefix).unwrap());
        match (kind, arg) {
            (0, SingleTypeArg::None) | (1, SingleTypeArg::Infer) => {},
            (2, SingleTypeArg::Explicit(actual)) => prop_assert_eq!(
                *actual,
                syn::parse_str::<Type>(&format!("Option<{}>", ty.source("_"))).unwrap()
            ),
            (_, actual) => prop_assert!(false, "wrong terminal classification: {actual:?}"),
        }
    }
}

#[test]
fn replacement_placeholders_and_macro_tokens_survive_together() {
    let input: Type =
        syn::parse_quote!((Opaque!(_), <Vec<_> as Trait>::Item, [u8; size_of::<_>()]));
    let replacement: Type = syn::parse_quote!(Option<_>);
    let expected: Type = syn::parse_quote!((
        Opaque!(_),
        <Vec<Option<_>> as Trait>::Item,
        [u8; size_of::<Option<_>>()]
    ));
    assert_eq!(substitute_infer_in_type(&input, &replacement), expected);
}
