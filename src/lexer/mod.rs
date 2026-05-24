//! Lexer — tokenises `MyLang` source text into a stream of [`Token`]s.
//!
//! Uses the [`logos`](logos) crate for fast, zero-copy tokenisation.
//! Sub-modules:
//! - [`tokens`] — token definitions and the token iterator
//! - [`iter`] — peekable token stream wrapper used by the parser

pub use tokens::{LexerError, Token};

pub mod iter;
pub mod tokens;
