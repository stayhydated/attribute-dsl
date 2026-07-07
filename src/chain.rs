use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens as _, quote};
use syn::parse::{Parse, ParseStream, discouraged::Speculative as _};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{AngleBracketedGenericArguments, Error, Expr, Ident, Path, Result, Token, parenthesized};

/// The method name rust-analyzer should complete against after a trailing dot.
///
/// Attribute parsers can accept incomplete syntax such as `TypeName.` by
/// appending this marker, then generated code can emit a method access on the
/// real builder value so rust-analyzer offers builder methods at the original
/// cursor.
pub const DEFAULT_COMPLETION_MARKER: &str = "raCompletionMarker";

/// Parser options for [`AttributeChain`].
#[derive(Clone, Debug)]
pub struct ChainParseOptions {
    completion_marker: String,
    allow_completion_probe: CompletionProbeParsing,
}

impl Default for ChainParseOptions {
    fn default() -> Self {
        Self {
            completion_marker: DEFAULT_COMPLETION_MARKER.to_owned(),
            allow_completion_probe: CompletionProbeParsing::Enabled,
        }
    }
}

/// Whether completion-probe syntax is accepted while parsing a chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionProbeParsing {
    /// Accept trailing-dot recovery and explicit completion-marker syntax.
    Enabled,
    /// Reject trailing-dot recovery and explicit completion-marker syntax.
    Disabled,
}

impl CompletionProbeParsing {
    fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

impl ChainParseOptions {
    /// Create parser options with completion probes enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the field/method marker inserted when the input ends in a dot.
    ///
    /// The marker must be a valid Rust identifier. Invalid markers cause
    /// trailing-dot recovery to return a parse error.
    pub fn completion_marker(mut self, marker: impl Into<String>) -> Self {
        self.completion_marker = marker.into();
        self
    }

    /// Set whether completion-probe parsing is enabled.
    ///
    /// Completion probes are useful for consumers that emit typed
    /// rust-analyzer probe code. Consumers that only accept complete chains can
    /// disable probes to reject both trailing-dot recovery and explicit marker
    /// syntax.
    pub fn allow_completion_probe(mut self, allow: CompletionProbeParsing) -> Self {
        self.allow_completion_probe = allow;
        self
    }

    fn completion_marker_ident(&self) -> Result<Ident> {
        syn::parse_str(&self.completion_marker).map_err(|_| {
            Error::new(
                Span::call_site(),
                "completion marker must be a valid Rust identifier",
            )
        })
    }
}

/// A parsed Rust attribute expression made from a path root and zero or more dot calls.
///
/// Accepted roots are:
///
/// - a path, such as `RangeValidation::<_>`.
#[derive(Clone, Debug)]
pub struct AttributeChain {
    root: Path,
    calls: Vec<ChainCall>,
    completion: ChainCompletion,
    span: Span,
}

impl AttributeChain {
    /// Parse an [`AttributeChain`] from a [`ParseStream`] with custom options.
    ///
    /// # Errors
    ///
    /// Returns [`syn::Error`] when the token stream is not a supported path or
    /// dot-call chain, completion probes are disabled for probe syntax, or the
    /// configured completion marker cannot be emitted as a Rust identifier.
    pub fn parse_with_options(input: ParseStream<'_>, options: &ChainParseOptions) -> Result<Self> {
        parse_chain_with_options(input, options)
    }

    /// Parse an [`AttributeChain`] from tokens with custom options.
    ///
    /// # Errors
    ///
    /// Returns [`syn::Error`] for the same syntax and option failures as
    /// [`Self::parse_with_options`].
    pub fn parse_tokens_with_options(
        tokens: TokenStream,
        options: &ChainParseOptions,
    ) -> Result<Self> {
        syn::parse::Parser::parse2(
            |input: ParseStream<'_>| Self::parse_with_options(input, options),
            tokens,
        )
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }

    pub fn calls(&self) -> &[ChainCall] {
        &self.calls
    }

    pub fn completion(&self) -> &ChainCompletion {
        &self.completion
    }

    pub fn has_completion_probe(&self) -> bool {
        matches!(self.completion, ChainCompletion::DotProbe { .. })
    }

    pub fn completion_marker(&self) -> Option<&Ident> {
        match &self.completion {
            ChainCompletion::None => None,
            ChainCompletion::DotProbe { marker } => Some(marker),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl Parse for AttributeChain {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Self::parse_with_options(input, &ChainParseOptions::default())
    }
}

impl quote::ToTokens for AttributeChain {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.root.to_tokens(tokens);
        for call in &self.calls {
            call.to_tokens(tokens);
        }
        if let ChainCompletion::DotProbe { marker } = &self.completion {
            quote! { .#marker }.to_tokens(tokens);
        }
    }
}

/// One dot-call in an [`AttributeChain`].
#[derive(Clone, Debug)]
pub struct ChainCall {
    method: Ident,
    turbofish: Option<AngleBracketedGenericArguments>,
    args: Vec<Expr>,
}

impl ChainCall {
    pub fn method(&self) -> &Ident {
        &self.method
    }

    pub fn turbofish(&self) -> Option<&AngleBracketedGenericArguments> {
        self.turbofish.as_ref()
    }

    pub fn args(&self) -> &[Expr] {
        &self.args
    }
}

impl quote::ToTokens for ChainCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let method = &self.method;
        let turbofish = &self.turbofish;
        let args = &self.args;
        quote! { .#method #turbofish (#(#args),*) }.to_tokens(tokens);
    }
}

/// Completion state for a parsed chain.
#[derive(Clone, Debug)]
pub enum ChainCompletion {
    /// The chain ended without a completion probe.
    None,
    /// The chain ended at a completion marker after a dot.
    DotProbe {
        /// Marker identifier emitted by completion-probe recovery.
        marker: Ident,
    },
}

/// A single chain entry, optionally labeled as `label = Chain`.
#[derive(Clone, Debug)]
pub struct ChainEntry {
    label: Option<Ident>,
    chain: AttributeChain,
}

impl ChainEntry {
    pub fn label(&self) -> Option<&Ident> {
        self.label.as_ref()
    }

    pub fn chain(&self) -> &AttributeChain {
        &self.chain
    }

    pub fn into_chain(self) -> AttributeChain {
        self.chain
    }
}

impl Parse for ChainEntry {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let label = parse_optional_label(input)?;
        let chain = input.parse()?;
        Ok(Self { label, chain })
    }
}

/// A comma-separated list of [`ChainEntry`] values.
#[derive(Clone, Debug, Default)]
pub struct ChainList {
    entries: Vec<ChainEntry>,
}

impl ChainList {
    pub fn entries(&self) -> &[ChainEntry] {
        &self.entries
    }

    pub fn into_entries(self) -> Vec<ChainEntry> {
        self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Parse for ChainList {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self::default());
        }

        let entries = Punctuated::<ChainEntry, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();
        Ok(Self { entries })
    }
}

/// A named parenthesized chain group such as `each(Thing.a(1), other = Thing.b(2))`.
#[derive(Clone, Debug)]
pub struct NamedChainGroup {
    name: Ident,
    entries: Vec<ChainEntry>,
}

impl NamedChainGroup {
    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn entries(&self) -> &[ChainEntry] {
        &self.entries
    }
}

impl Parse for NamedChainGroup {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        let content;
        parenthesized!(content in input);
        let entries = content.parse::<ChainList>()?.into_entries();
        Ok(Self { name, entries })
    }
}

fn parse_chain_with_options(
    input: ParseStream<'_>,
    options: &ChainParseOptions,
) -> Result<AttributeChain> {
    let fork = input.fork();
    let (expr, advanced_fork) = match fork.parse::<Expr>() {
        Ok(expr) => {
            if !fork.is_empty() && fork.peek(Token![.]) {
                match parse_trailing_dot_probe_expr(&fork, expr.to_token_stream(), options)? {
                    Some(expr) => (expr, fork),
                    None => return Err(invalid_chain_syntax_error(input)),
                }
            } else {
                (expr, fork)
            }
        },
        Err(_) => {
            let fallback = input.fork();
            match parse_trailing_dot_probe_expr(&fallback, TokenStream::new(), options)? {
                Some(expr) => (expr, fallback),
                None => return Err(invalid_chain_syntax_error(input)),
            }
        },
    };
    input.advance_to(&advanced_fork);

    let span = expr.span();
    let Some((root, calls, completion)) = analyze_chain_expr(&expr, true, options)? else {
        return Err(Error::new(
            span,
            "expected attribute path or dot-call chain",
        ));
    };
    Ok(AttributeChain {
        root,
        calls,
        completion,
        span,
    })
}

fn parse_trailing_dot_probe_expr(
    input: ParseStream<'_>,
    mut probe_expr: TokenStream,
    options: &ChainParseOptions,
) -> Result<Option<Expr>> {
    if !options.allow_completion_probe.is_enabled() {
        return Ok(None);
    }

    let mut saw_token = false;
    let mut ends_with_dot = false;

    while !input.is_empty() && !input.peek(Token![,]) {
        let token: TokenTree = input.parse()?;
        ends_with_dot = matches!(&token, TokenTree::Punct(punct) if punct.as_char() == '.');
        probe_expr.extend(token.to_token_stream());
        saw_token = true;
    }

    if !saw_token || !ends_with_dot {
        return Ok(None);
    }

    probe_expr.extend(options.completion_marker_ident()?.to_token_stream());
    Ok(syn::parse2::<Expr>(probe_expr).ok())
}

fn analyze_chain_expr(
    expr: &Expr,
    is_terminal: bool,
    options: &ChainParseOptions,
) -> Result<Option<(Path, Vec<ChainCall>, ChainCompletion)>> {
    match expr {
        Expr::Group(group) => analyze_chain_expr(&group.expr, is_terminal, options),
        Expr::Paren(paren) => analyze_chain_expr(&paren.expr, is_terminal, options),
        Expr::Field(field) => {
            let Some((root, calls, _completion)) = analyze_chain_expr(&field.base, false, options)?
            else {
                return Ok(None);
            };

            let syn::Member::Named(marker) = &field.member else {
                return Ok(None);
            };

            if !is_terminal || marker != options.completion_marker.as_str() {
                return Ok(None);
            }
            if !options.allow_completion_probe.is_enabled() {
                return Ok(None);
            }

            Ok(Some((
                root,
                calls,
                ChainCompletion::DotProbe {
                    marker: marker.clone(),
                },
            )))
        },
        Expr::MethodCall(method_call) => {
            let Some((root, mut calls, _completion)) =
                analyze_chain_expr(&method_call.receiver, false, options)?
            else {
                return Ok(None);
            };

            calls.push(ChainCall {
                method: method_call.method.clone(),
                turbofish: method_call.turbofish.clone(),
                args: method_call.args.iter().cloned().collect(),
            });
            Ok(Some((root, calls, ChainCompletion::None)))
        },
        Expr::Path(path) => Ok(Some((path.path.clone(), Vec::new(), ChainCompletion::None))),
        _ => Ok(None),
    }
}

fn parse_optional_label(input: ParseStream<'_>) -> Result<Option<Ident>> {
    if !input.peek(Ident) {
        return Ok(None);
    }

    let fork = input.fork();
    let _: Ident = fork.parse()?;
    if !fork.peek(Token![=]) {
        return Ok(None);
    }

    let label = input.parse::<Ident>()?;
    input.parse::<Token![=]>()?;
    Ok(Some(label))
}

fn invalid_chain_syntax_error(input: ParseStream<'_>) -> Error {
    Error::new(
        input.span(),
        "attribute chain syntax expects a path such as `Thing::<_>` or a dot-call chain such as `Thing::<_>.option(value)`",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, parse_str};

    fn compact(tokens: impl quote::ToTokens) -> String {
        tokens
            .to_token_stream()
            .to_string()
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect()
    }

    #[test]
    fn parses_path_root_dot_chain() {
        let chain: AttributeChain = parse_str("validators::RangeValidation::<_>.min(0).max(100)")
            .expect("chain should parse");

        assert_eq!(compact(chain.root()), "validators::RangeValidation::<_>");
        assert_eq!(chain.calls().len(), 2);
        assert_eq!(chain.calls()[0].method().to_string(), "min");
        assert_eq!(chain.calls()[0].args().len(), 1);
        assert_eq!(chain.calls()[1].method().to_string(), "max");
        assert_eq!(
            compact(&chain),
            "validators::RangeValidation::<_>.min(0).max(100)"
        );
    }

    #[test]
    fn parses_root_only_chain_and_accessors() {
        let chain: AttributeChain =
            parse_str("::validators::RangeValidation").expect("root-only chain should parse");

        assert_eq!(compact(chain.root_path()), "::validators::RangeValidation");
        assert!(matches!(chain.completion(), ChainCompletion::None));
        assert!(chain.completion_marker().is_none());
        assert!(!chain.has_completion_probe());
        let _span = chain.span();
    }

    #[test]
    fn parses_method_turbofish_and_entry_accessors() {
        let entry: ChainEntry =
            parse_str("field = Validator.map::<String>(value)").expect("labeled entry");

        assert_eq!(entry.label().map(ToString::to_string), Some("field".into()));
        let chain = entry.chain();
        assert_eq!(chain.calls().len(), 1);
        assert_eq!(chain.calls()[0].method().to_string(), "map");
        assert!(chain.calls()[0].turbofish().is_some());

        let chain = entry.into_chain();
        assert_eq!(compact(&chain), "Validator.map::<String>(value)");
    }

    #[test]
    fn rejects_associated_call_root_dot_chain() {
        let result = parse_str::<AttributeChain>("StringFaker::builder().with_min_length(5)");

        assert!(result.is_err());
    }

    #[test]
    fn rejects_non_chain_expressions() {
        assert!(parse_str::<AttributeChain>("1u8..=3u8").is_err());
        assert!(parse_str::<AttributeChain>("\"legacy shorthand\"").is_err());
        assert!(parse_str::<AttributeChain>("make_faker() + other").is_err());
        assert!(parse_str::<AttributeChain>("left + right.").is_err());
        assert!(parse_str::<AttributeChain>("RangeValidation::<_>..").is_err());
    }

    #[test]
    fn rejects_empty_chain() {
        let result = parse_str::<AttributeChain>("");

        assert!(result.is_err());
    }

    #[test]
    fn rejects_field_expressions_that_are_not_completion_markers() {
        assert!(parse_str::<AttributeChain>("true.raCompletionMarker").is_err());
        assert!(parse_str::<AttributeChain>("RangeValidation.0").is_err());
        assert!(parse_str::<AttributeChain>("RangeValidation.field").is_err());
    }

    #[test]
    fn analyzes_parenthesized_and_grouped_chain_expressions() {
        let chain: AttributeChain =
            parse_str("(RangeValidation::<_>.min(0))").expect("parenthesized chain");
        assert_eq!(compact(&chain), "RangeValidation::<_>.min(0)");

        let expr = Expr::Group(syn::ExprGroup {
            attrs: Vec::new(),
            group_token: Default::default(),
            expr: Box::new(parse_quote!(RangeValidation::<_>)),
        });
        let (root, calls, completion) =
            analyze_chain_expr(&expr, true, &ChainParseOptions::default())
                .expect("group analysis should parse")
                .expect("grouped path should be a chain");
        assert_eq!(compact(root), "RangeValidation::<_>");
        assert!(calls.is_empty());
        assert!(matches!(completion, ChainCompletion::None));
    }

    #[test]
    fn parses_trailing_dot_completion_probe() {
        let chain: AttributeChain =
            parse_str("RangeValidation::<_>.min(0).").expect("trailing dot should recover");

        assert!(chain.has_completion_probe());
        assert_eq!(
            chain.completion_marker().map(ToString::to_string),
            Some(DEFAULT_COMPLETION_MARKER.to_owned())
        );
        assert_eq!(chain.calls().len(), 1);
        assert_eq!(
            compact(&chain),
            "RangeValidation::<_>.min(0).raCompletionMarker"
        );
    }

    #[test]
    fn parses_root_trailing_dot_completion_probe() {
        let chain: AttributeChain =
            parse_str("RangeValidation::<_>.").expect("root trailing dot should recover");

        assert!(chain.has_completion_probe());
        assert_eq!(chain.calls().len(), 0);
        assert_eq!(compact(&chain), "RangeValidation::<_>.raCompletionMarker");
    }

    #[test]
    fn parses_root_trailing_dot_completion_probe_before_comma() {
        let list: ChainList =
            parse_str("RangeValidation::<_>., Other").expect("trailing dot before comma");

        assert_eq!(list.entries().len(), 2);
        assert!(list.entries()[0].chain().has_completion_probe());
        assert_eq!(
            compact(list.entries()[0].chain()),
            "RangeValidation::<_>.raCompletionMarker"
        );
    }

    #[test]
    fn parses_named_completion_probe_with_infer_placeholder() {
        let chain: AttributeChain = parse_str("RangeValidation::<_>.min(0).raCompletionMarker")
            .expect("named completion marker should parse");

        assert!(chain.has_completion_probe());
        assert_eq!(chain.calls().len(), 1);
        assert_eq!(
            compact(&chain),
            "RangeValidation::<_>.min(0).raCompletionMarker"
        );
    }

    #[test]
    fn rejects_trailing_dot_completion_probe_when_disabled() {
        let options =
            ChainParseOptions::new().allow_completion_probe(CompletionProbeParsing::Disabled);
        let result = AttributeChain::parse_tokens_with_options(
            quote!(RangeValidation::<i32>.min(0).),
            &options,
        );

        assert!(result.is_err());
    }

    #[test]
    fn rejects_root_trailing_dot_completion_probe_when_disabled() {
        let options =
            ChainParseOptions::new().allow_completion_probe(CompletionProbeParsing::Disabled);
        let result =
            AttributeChain::parse_tokens_with_options(quote!(RangeValidation::<i32>.), &options);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_named_completion_probe_when_disabled() {
        let options =
            ChainParseOptions::new().allow_completion_probe(CompletionProbeParsing::Disabled);
        let result = AttributeChain::parse_tokens_with_options(
            quote!(RangeValidation::<i32>.min(0).raCompletionMarker),
            &options,
        );

        assert!(result.is_err());
    }

    #[test]
    fn parses_trailing_dot_completion_probe_with_custom_marker() {
        let options = ChainParseOptions::new().completion_marker("completeHere");
        let chain = AttributeChain::parse_tokens_with_options(
            quote!(RangeValidation::<_>.min(0).),
            &options,
        )
        .expect("trailing dot should recover with custom marker");

        assert!(chain.has_completion_probe());
        assert_eq!(
            chain.completion_marker().map(ToString::to_string),
            Some("completeHere".to_owned())
        );
        assert_eq!(compact(&chain), "RangeValidation::<_>.min(0).completeHere");
    }

    #[test]
    fn rejects_invalid_custom_completion_marker() {
        let options = ChainParseOptions::new().completion_marker("not a marker");
        let err =
            AttributeChain::parse_tokens_with_options(quote!(RangeValidation::<_>.), &options)
                .expect_err("invalid completion marker should error");

        assert!(
            err.to_string()
                .contains("completion marker must be a valid Rust identifier"),
            "{err}"
        );
    }

    #[test]
    fn parses_empty_chain_list() {
        let list: ChainList = parse_str("").expect("empty list");

        assert!(list.is_empty());
        assert!(list.entries().is_empty());
    }

    #[test]
    fn parses_unlabeled_chain_entry_starting_with_colon() {
        let entry: ChainEntry = parse_str("::Validator").expect("unlabeled absolute path");

        assert!(entry.label().is_none());
        assert_eq!(compact(entry.chain()), "::Validator");
    }

    #[test]
    fn parses_labeled_chain_lists() {
        let list: ChainList = syn::parse_quote! {
            first = Validator::<_>.min(1),
            Validator::<String>,
            built = Faker.weight(2)
        };

        assert_eq!(list.entries().len(), 3);
        assert_eq!(
            list.entries()[0].label().map(ToString::to_string),
            Some("first".to_owned())
        );
        assert!(list.entries()[1].label().is_none());
        assert_eq!(
            list.entries()[2].label().map(ToString::to_string),
            Some("built".to_owned())
        );
    }

    #[test]
    fn parses_named_chain_group() {
        let group: NamedChainGroup = syn::parse_quote! {
            each(tag = TagFaker.length(8), OtherFaker)
        };

        assert_eq!(group.name().to_string(), "each");
        assert_eq!(group.entries().len(), 2);
        assert_eq!(
            group.entries()[0].label().map(ToString::to_string),
            Some("tag".to_owned())
        );
    }
}
