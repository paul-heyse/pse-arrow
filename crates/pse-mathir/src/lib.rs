// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Expression graph, operator catalog and contracts, canonicalization and unit inference
//! (blueprint §3.2, §6.9, §7).
//!
//! The math IR is relational: expression graphs are rows in `compiled.math_expr_nodes` and
//! `compiled.math_expr_args`, never an in-memory AST that escapes the registry (§7.1
//! constraint 1). The types in this crate are the in-memory shape of exactly those rows,
//! so that a pass can build and check a graph without an engine, and packet M-5's
//! `MathRelationSink` can write the same facts out.
//!
//! # Why this is not a `datafusion::Expr`
//!
//! D6 in one paragraph: indexed operators — `SumOver`, `Gather`, `Broadcast`, `Derivative`,
//! `Integral`, `ImplicitRef`, `KernelCall` — survive until a backend requires
//! scalarization. A species balance over 1 000 cells stays one node with a free cell index
//! until P12 expands it. DataFusion `Expr` is used to *compute over* this IR (the rule
//! compiler), never to *be* it.
//!
//! # What the keel guarantees
//!
//! [`ExprGraph::insert`] accepts a reference only to a node that already exists, so a
//! cycle cannot be built through the API; an identical `(opcode, payload, children)`
//! returns the identity already assigned, with floats compared through
//! `pse_ids::canonical_f64_bits` and `scope` excluded (§7.4 step 1, ADR-0030). Ordered
//! structure is preserved exactly as authored: nothing here sorts a commutative child list
//! or reassociates an `Affine` node (§7.4 step 2).
//!
//! # Layout
//!
//! Landed by the keel (packet K-4):
//!
//! - [`node`] — [`NodeId`], [`Node`], [`Arity`] and [`arity`].
//! - [`payload`] — [`Payload`], one variant per §6.9 payload relation.
//! - [`graph`] — [`ExprGraph`], the only builder.
//! - [`error`] — [`MathIrError`] with its §23.2 classes.
//!
//! Declared here and filled by later packets: [`opspec`], [`catalog`], [`hash`],
//! [`canonical`], [`topo`] (packet M-1); [`fold`] (packet M-3); [`index`], [`infer`],
//! [`equation`], [`canonicalize`], [`walk`] (packet M-4); [`relations`] (packet M-5).

pub mod canonical;
pub mod canonicalize;
pub mod catalog;
pub mod equation;
pub mod error;
pub mod fold;
pub mod graph;
pub mod hash;
pub mod index;
pub mod infer;
pub mod node;
pub mod opspec;
pub mod payload;
pub mod relations;
pub mod topo;
pub mod walk;

pub use crate::error::MathIrError;
pub use crate::graph::ExprGraph;
pub use crate::node::{Arity, Node, NodeId, arity};
pub use crate::payload::{AffineTerm, Payload, WeightedPair};

/// The 43 operators of §7.2, re-exported from where they are declared.
///
/// The opcode is declared in `pse-quantity` because the registry types
/// `reference.quantity_operations.opcode` as `enum(Opcode)` and `pse-quantity` sits below
/// this crate. Re-exporting rather than redeclaring is what keeps one member list
/// (prime directive 3).
pub use pse_quantity::Opcode;

/// The identity, index and enumeration types a math-IR payload carries.
///
/// Re-exported so that a caller building a graph needs one `use` rather than two; each of
/// these is declared in `pse-quantity`, and this crate adds nothing to them.
pub use pse_quantity::{
    BoundIndexId, DomainId, InvariantId, QuantityTypeId, ReductionKind, UnitConvertSpec, UnitId,
    WeightNormalization,
};
