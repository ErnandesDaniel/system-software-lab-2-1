//! Abstract Syntax Tree — the parsed representation of a source program.
//!
//! The AST is produced by the parser and consumed by the semantic analyser
//! and IR generator. All node types are defined in the [`types`] submodule.

pub use types::*;

pub mod types;
