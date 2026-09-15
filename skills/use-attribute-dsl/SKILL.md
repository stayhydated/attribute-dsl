---
name: use-attribute-dsl
description: Integrate or review attribute-dsl in Rust proc macros that parse path-rooted dot-call chains, labeled entries, lists, or named groups. Covers subject-type substitution and rust-analyzer completion probes; use syn directly for arbitrary expression grammars.
---

# Use attribute-dsl

Apply this workflow to consumer parsing and expansion. For a review, report
findings without editing the consumer.

## Establish the consumer contract

1. Read the consumer's attribute parsing, expansion, tests, and dependency
   versions. `attribute-dsl` 0.2 exposes `syn` 3 nodes and requires Rust 1.98.
2. Identify the complete accepted argument shape.
3. Determine whether `_` represents a subject type and whether trailing-dot input
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
disabled. Custom options apply to direct chain parsing; the composite `Parse`
implementations use the defaults.

## Check the contract

Check the consumer tests for the changed contract: accepted outer shapes,
rejected roots, preserved call order and turbofish, subject-type handling, or
completion behavior as applicable. Run the focused checks available in that
repository. Keep semantic validation in the consumer.

## Load concrete patterns

Read [references/patterns.md](references/patterns.md) for parser selection and
copyable substitution, completion, and quoting templates. Consult the crate's
current API documentation when a consumer pins a different release.
