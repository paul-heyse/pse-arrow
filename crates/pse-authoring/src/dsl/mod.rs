// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Bounded expression syntax, exact byte spans and structural round trips (blueprint §7.7).
//!
//! The AST is transient syntax for the authored text. P3 resolves it into declared
//! relations; neither tree hashes nor rendered bytes substitute for validation.

#[cfg(feature = "arbitrary")]
mod arbitrary;
mod ast;
mod error;
mod lexer;
mod parser;
mod render;
mod walk;

pub use ast::{
    BinaryOp, Binder, CompareOp, Equation, EquationKind, EquationSense, Expr, ExprKind, Function,
    Number, Path, PathSegment, Predicate, PredicateKind, ReduceKind, Span,
};
pub use error::DslError;
pub use parser::{parse_equation, parse_expr, parse_predicate};
pub use render::{render_equation, render_expr, render_path, render_predicate};
