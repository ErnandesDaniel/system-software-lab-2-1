#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::too_many_arguments,
    clippy::unused_self,
    clippy::needless_lifetimes,
    clippy::collapsible_match,
    clippy::vec_init_then_push
)]

mod build;
mod bytecode;
mod collect;
mod context;
mod coro_build;
mod generator;
mod instr;
mod loaders;
mod runtime;
mod stacks;
mod types;

pub use generator::JvmGenerator;
pub(crate) use generator::{JumpPlaceholder, JvmInst};
