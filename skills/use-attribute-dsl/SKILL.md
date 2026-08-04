---
name: use-attribute-dsl
description: Apply the attribute-dsl crate when implementing, reviewing, or updating Rust derive-macro or attribute-macro parsers whose arguments use a Rust path root followed by dot calls, optional labels or lists, named groups, rust-analyzer trailing-dot completion probes, or `_` subject-type placeholders. Covers parser selection, syn-preserving expansion, infer substitution, completion configuration, diagnostics, and focused tests.
---

# Use attribute-dsl

## Establish the consumer contract

1. Read the consumer's attribute parsing, expansion, and tests.
2. Write down the complete accepted argument shape.
3. Decide whether `_` represents a subject type and whether trailing-dot input
   must produce rust-analyzer completion.
4. Identify the application-owned constructor and the typed receiver it
   returns.

Use `syn` directly when the grammar is an arbitrary Rust expression rather than
a path-rooted call chain.

## Implement the integration

1. Select `AttributeChain`, `ChainEntry`, `ChainList`, or `NamedChainGroup` for
   the outer syntax.
2. Parse from the `syn::Attribute` so errors retain source spans.
3. Keep the root as a `syn::Path` and preserve call methods, turbofish, arguments,
   and order.
4. Apply the narrowest infer helper needed by the consumer grammar.
5. Quote the application-owned constructor, parsed calls, and optional probe
   marker into the expansion.
6. Return `syn::Error` from parsing and attach semantic errors to the narrowest
   relevant node.

Enable completion probes only when the expansion places the marker after a real
typed receiver. Otherwise parse `AttributeChain` with completion probes
disabled.

## Check the contract

Cover accepted outer shapes, rejected non-path roots, call order and turbofish,
absent/inferred/explicit subject types, and trailing-dot behavior when enabled.
Keep semantic validation in the consumer instead of widening the parser grammar.

## Load concrete patterns

Read [references/patterns.md](references/patterns.md) for parser selection and
copyable substitution, completion, and quoting templates. Consult the crate's
current API documentation when a consumer pins a different release.
