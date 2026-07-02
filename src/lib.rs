#![doc = include_str!("../README.md")]

mod chain;
mod infer;

pub use chain::{
    AttributeChain, ChainCall, ChainCompletion, ChainEntry, ChainList, ChainParseOptions,
    CompletionProbeParsing, DEFAULT_COMPLETION_MARKER, NamedChainGroup,
};
pub use infer::{
    SingleTypeArg, split_terminal_single_type_arg, substitute_infer_in_expr,
    substitute_infer_in_path, substitute_infer_in_type,
};
