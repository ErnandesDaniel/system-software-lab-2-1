//! Intermediate Representation — a target-independent IR used by all backends.
//!
//! The IR is a control-flow graph (CFG) of basic blocks. Each block contains
//! a linear sequence of [`IrInstruction`]s. Sub-modules:
//! - [`types`] — IR opcodes, operands, and program structure
//! - [`cfg`] — CFG analysis (dominators, loop detection)
//! - [`diagram`] — Mermaid diagram generation for CFG visualisation
//! - [`validator`] — IR validation passes

pub use types::*;

pub mod cfg;
pub mod diagram;
pub mod types;
pub mod validator;
