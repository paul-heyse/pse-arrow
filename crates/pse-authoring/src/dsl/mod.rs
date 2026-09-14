// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The expression DSL: lexer, parser, AST and renderer (blueprint §7.7).
//!
//! The text in a `pse.expr_dsl` column is the authored fact and the only form a change set
//! may edit; its parsed graph is a derived `normalized.*_expr_*` relation produced by P3.
//! `parse(render(a))` is the identity on the graph, which is what keeps the two from
//! becoming two authorities.
//!
//! Packet C-dsl fills this module.
