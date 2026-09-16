// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The expression DAG and the one API that builds it (blueprint §7.1, §7.4).
//!
//! [`ExprGraph::insert`] is the only way to add a node, and it accepts a reference only to
//! a node that already exists. That single rule is what makes a cycle *unrepresentable*
//! through this API rather than merely detected afterwards: a node can only point
//! backwards, so [`MathIrError::Cycle`] belongs to the loader that reads a graph out of
//! relations, not to the builder.
//!
//! # Structural sharing
//!
//! An insert that repeats an existing `(opcode, payload, children)` returns the existing
//! [`NodeId`] — the hash-consing of §7.4 step 1, done here by value comparison. Three
//! consequences are deliberate:
//!
//! - **Floats compare through `canonical_f64_bits`.** `-0.0` and `0.0` are *different*
//!   literals and get different nodes; every NaN would compare equal, which is why a NaN
//!   literal is rejected outright instead (ADR-0030).
//! - **`scope` is not part of the key.** The same expression in two instances is one node,
//!   exactly as §7.4 step 1 excludes scope from the structural hash. Conflicting scopes merge
//!   to `None`, which remains absorbing on subsequent inserts.
//! - **Sharing is not permission to hoist.** A node shared with a `Conditional` branch is
//!   still evaluated inside that branch (§7.4 step 1); the guard lives in the payload so
//!   that the child list cannot suggest otherwise.
//!
//! Sharing compares the complete framed structure, not only a digest. Hashes are
//! identity/lookup aids; payload, reference and value checks remain mandatory.

use std::collections::BTreeMap;

use pse_ids::{SemanticId, canonical_f64_bits};
use pse_quantity::{Opcode, QuantityTypeId, UnitConvertSpec, UnitId};

use crate::error::MathIrError;
use crate::node::{Node, NodeId, arity};
use crate::payload::Payload;

/// An expression DAG under construction (blueprint §6.9, §7.1).
///
/// ```
/// use pse_ids::SemanticId;
/// use pse_mathir::{ExprGraph, UnitId};
///
/// let mut graph = ExprGraph::new();
/// let kelvin = UnitId::from_id(SemanticId::from_bytes([1; 16]));
/// let t = graph.symbol(SemanticId::from_bytes([2; 16]))?;
/// let offset = graph.float_const(273.15, kelvin)?;
/// let sum = graph.add(t, offset)?;
///
/// // The same structure inserted twice is one node.
/// assert_eq!(graph.add(t, offset)?, sum);
/// assert_eq!(graph.len(), 3);
/// # Ok::<(), pse_mathir::MathIrError>(())
/// ```
#[derive(Clone, Debug, Default)]
pub struct ExprGraph {
    /// The nodes, in insertion order; the index is the `NodeId`.
    nodes: Vec<Node>,
    /// Structural key to node, so a repeated structure returns its first identity.
    by_structure: BTreeMap<Vec<u8>, NodeId>,
}

impl ExprGraph {
    /// An empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node, returning the existing identity when the structure repeats.
    ///
    /// # Errors
    ///
    /// [`MathIrError::NonFiniteLiteral`] for a NaN or infinite literal, and
    /// [`MathIrError::Malformed`] when the node does not satisfy the node model: a payload
    /// that does not belong to the opcode, a child count the arity refuses, a reference to
    /// a node that does not exist, `Affine` terms that do not match the child list, an
    /// empty `WeightedMean`, a `CertifiedUnitSum` without its invariant, or a
    /// non-positive smoothing parameter.
    pub fn insert(
        &mut self,
        opcode: Opcode,
        payload: Payload,
        children: &[NodeId],
        scope: Option<SemanticId>,
    ) -> Result<NodeId, MathIrError> {
        self.insert_with_type(opcode, payload, children, None, scope)
    }

    /// Insert one physically inferred occurrence without merging distinct quantity contracts.
    /// The caller establishes the quantity contract; this method checks structural admission.
    ///
    /// # Errors
    /// Rejects malformed payloads, values, arities, and references just like [`Self::insert`].
    pub fn insert_typed(
        &mut self,
        opcode: Opcode,
        payload: Payload,
        children: &[NodeId],
        quantity_type: QuantityTypeId,
        scope: Option<SemanticId>,
    ) -> Result<NodeId, MathIrError> {
        self.insert_with_type(opcode, payload, children, Some(quantity_type), scope)
    }

    fn insert_with_type(
        &mut self,
        opcode: Opcode,
        payload: Payload,
        children: &[NodeId],
        quantity_type: Option<QuantityTypeId>,
        scope: Option<SemanticId>,
    ) -> Result<NodeId, MathIrError> {
        check_payload_belongs_to(opcode, &payload)?;
        check_arity(opcode, children.len())?;
        self.check_references(opcode, &payload, children)?;
        check_payload_values(opcode, &payload, children)?;

        let key = typed_key(opcode, &payload, children, quantity_type);
        if let Some(existing) = self.by_structure.get(&key) {
            let id = *existing;
            if let Some(node) = usize::try_from(id.0)
                .ok()
                .and_then(|index| self.nodes.get_mut(index))
                && node.scope != scope
            {
                node.scope = None;
            }
            return Ok(id);
        }

        let id = NodeId(self.nodes.len() as u64);
        self.nodes.push(Node {
            opcode,
            payload,
            children: children.to_vec(),
            quantity_type,
            scope,
        });
        self.by_structure.insert(key, id);
        Ok(id)
    }

    /// Replace a node after a pass has checked its operation and physical contract.
    pub(crate) fn replace_literal(
        &mut self,
        id: NodeId,
        payload: Payload,
    ) -> Result<(), MathIrError> {
        check_payload_belongs_to(Opcode::Const, &payload)?;
        check_payload_values(Opcode::Const, &payload, &[])?;
        let node = usize::try_from(id.0)
            .ok()
            .and_then(|index| self.nodes.get_mut(index))
            .ok_or_else(|| MathIrError::malformed_at(id, "no such node"))?;
        node.opcode = Opcode::Const;
        node.payload = payload;
        node.children.clear();
        Ok(())
    }

    pub(crate) fn rebuild_structural_index(&mut self) {
        self.by_structure.clear();
        for (position, node) in self.nodes.iter().enumerate() {
            self.by_structure
                .entry(typed_key(
                    node.opcode,
                    &node.payload,
                    &node.children,
                    node.quantity_type,
                ))
                .or_insert(NodeId(position as u64));
        }
    }

    /// The node with this identity.
    ///
    /// # Errors
    ///
    /// [`MathIrError::Malformed`] when the identity does not belong to this graph.
    pub fn node(&self, id: NodeId) -> Result<&Node, MathIrError> {
        usize::try_from(id.0)
            .ok()
            .and_then(|index| self.nodes.get(index))
            .ok_or_else(|| MathIrError::malformed_at(id, "no such node in this graph"))
    }

    /// How many nodes the graph holds.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Is the graph empty?
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The nodes with their identities, in insertion order.
    ///
    /// Insertion order is not storage order: §7.4 step 5 renumbers in deterministic
    /// post-order when the graph is canonicalized.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (NodeId(index as u64), node))
    }

    /// Records the quantity type P10 inferred for a node.
    ///
    /// Unit inference is a pass, not a runtime check (§7.1 constraint 3): this is the P10
    /// write path and nothing else should call it.
    ///
    /// # Errors
    ///
    /// [`MathIrError::Malformed`] when the identity does not belong to this graph.
    pub fn set_quantity_type(
        &mut self,
        id: NodeId,
        quantity_type: QuantityTypeId,
    ) -> Result<(), MathIrError> {
        let old = self.node(id)?;
        let prior_key = typed_key(old.opcode, &old.payload, &old.children, old.quantity_type);
        let node = usize::try_from(id.0)
            .ok()
            .and_then(|index| self.nodes.get_mut(index))
            .ok_or_else(|| MathIrError::malformed_at(id, "no such node in this graph"))?;
        node.quantity_type = Some(quantity_type);
        let new_key = typed_key(
            node.opcode,
            &node.payload,
            &node.children,
            node.quantity_type,
        );
        if self.by_structure.get(&prior_key) == Some(&id) {
            self.by_structure.remove(&prior_key);
        }
        self.by_structure.entry(new_key).or_insert(id);
        Ok(())
    }

    /// A floating-point literal with the unit it was written in.
    ///
    /// # Errors
    ///
    /// [`MathIrError::NonFiniteLiteral`] when the value is NaN or infinite.
    pub fn float_const(&mut self, value: f64, unit: UnitId) -> Result<NodeId, MathIrError> {
        self.insert(
            Opcode::Const,
            Payload::FloatConst { value, unit },
            &[],
            None,
        )
    }

    /// An integer literal.
    ///
    /// # Errors
    ///
    /// Never in practice; the signature matches the other constructors.
    pub fn int_const(&mut self, value: i64) -> Result<NodeId, MathIrError> {
        self.insert(Opcode::Const, Payload::IntConst { value }, &[], None)
    }

    /// A reference to a symbol.
    ///
    /// # Errors
    ///
    /// Never in practice; the signature matches the other constructors.
    pub fn symbol(&mut self, symbol: SemanticId) -> Result<NodeId, MathIrError> {
        self.insert(
            Opcode::SymbolRef,
            Payload::SymbolRef {
                symbol: symbol.into(),
            },
            &[],
            None,
        )
    }

    /// An ordered sum of two nodes.
    ///
    /// # Errors
    ///
    /// [`MathIrError::Malformed`] when either child does not exist.
    pub fn add(&mut self, left: NodeId, right: NodeId) -> Result<NodeId, MathIrError> {
        self.insert(Opcode::Add, Payload::None, &[left, right], None)
    }

    /// An ordered difference of two nodes.
    ///
    /// # Errors
    ///
    /// [`MathIrError::Malformed`] when either child does not exist.
    pub fn sub(&mut self, left: NodeId, right: NodeId) -> Result<NodeId, MathIrError> {
        self.insert(Opcode::Sub, Payload::None, &[left, right], None)
    }

    /// An ordered product of two nodes.
    ///
    /// # Errors
    ///
    /// [`MathIrError::Malformed`] when either child does not exist.
    pub fn mul(&mut self, left: NodeId, right: NodeId) -> Result<NodeId, MathIrError> {
        self.insert(Opcode::Mul, Payload::None, &[left, right], None)
    }

    /// An ordered quotient of two nodes.
    ///
    /// # Errors
    ///
    /// [`MathIrError::Malformed`] when either child does not exist.
    pub fn div(&mut self, left: NodeId, right: NodeId) -> Result<NodeId, MathIrError> {
        self.insert(Opcode::Div, Payload::None, &[left, right], None)
    }

    /// A base raised to an exponent.
    ///
    /// # Errors
    ///
    /// [`MathIrError::Malformed`] when either child does not exist.
    pub fn pow(&mut self, base: NodeId, exponent: NodeId) -> Result<NodeId, MathIrError> {
        self.insert(Opcode::Pow, Payload::None, &[base, exponent], None)
    }

    /// The negation of a node.
    ///
    /// # Errors
    ///
    /// [`MathIrError::Malformed`] when the child does not exist.
    pub fn neg(&mut self, child: NodeId) -> Result<NodeId, MathIrError> {
        self.insert(Opcode::Neg, Payload::None, &[child], None)
    }

    /// A declared unit-conversion edge over a node.
    ///
    /// # Errors
    ///
    /// [`MathIrError::Malformed`] when the child does not exist.
    pub fn unit_convert(
        &mut self,
        child: NodeId,
        spec: UnitConvertSpec,
    ) -> Result<NodeId, MathIrError> {
        self.insert(
            Opcode::UnitConvert,
            Payload::UnitConvert(spec),
            &[child],
            None,
        )
    }

    /// Does this identity belong to this graph?
    fn holds(&self, id: NodeId) -> bool {
        usize::try_from(id.0).is_ok_and(|index| index < self.nodes.len())
    }

    /// Every node the new node points at must already exist — the rule that makes a cycle
    /// unrepresentable.
    fn check_references(
        &self,
        opcode: Opcode,
        payload: &Payload,
        children: &[NodeId],
    ) -> Result<(), MathIrError> {
        for child in children.iter().chain(payload.referenced_nodes().iter()) {
            if !self.holds(*child) {
                return Err(MathIrError::malformed(format!(
                    "`{opcode}` references {child}, which this graph does not hold"
                )));
            }
        }
        Ok(())
    }
}

/// Is this payload the one §6.9 stores for this opcode?
fn check_payload_belongs_to(opcode: Opcode, payload: &Payload) -> Result<(), MathIrError> {
    let ok = match opcode {
        Opcode::Const => matches!(
            payload,
            Payload::FloatConst { .. } | Payload::IntConst { .. }
        ),
        Opcode::SymbolRef => matches!(
            payload,
            Payload::SymbolRef { .. } | Payload::PendingPath { .. }
        ),
        Opcode::Affine => matches!(payload, Payload::Affine { .. }),
        Opcode::WeightedMean => matches!(payload, Payload::WeightedMean { .. }),
        Opcode::SmoothMax
        | Opcode::SmoothMin
        | Opcode::SmoothAbs
        | Opcode::SafeSqrt
        | Opcode::SafeLog => matches!(
            payload,
            Payload::SmoothOp { .. } | Payload::PendingSmoothOp { .. }
        ),
        Opcode::Conditional => matches!(payload, Payload::Conditional { .. }),
        Opcode::SumOver => matches_reduction(payload, pse_quantity::ReductionKind::Sum),
        Opcode::ProdOver => matches_reduction(payload, pse_quantity::ReductionKind::Prod),
        Opcode::MinOver => matches_reduction(payload, pse_quantity::ReductionKind::Min),
        Opcode::MaxOver => matches_reduction(payload, pse_quantity::ReductionKind::Max),
        Opcode::Gather => matches!(
            payload,
            Payload::Gather { .. } | Payload::PendingGather { .. } | Payload::PendingPath { .. }
        ),
        Opcode::Broadcast => matches!(payload, Payload::Broadcast { .. }),
        Opcode::Derivative => matches!(payload, Payload::Derivative { .. }),
        Opcode::Integral => matches!(payload, Payload::Integral { .. }),
        Opcode::KernelCall => matches!(payload, Payload::KernelCall { .. }),
        Opcode::ImplicitRef => matches!(payload, Payload::ImplicitRef { .. }),
        Opcode::UnitConvert => matches!(
            payload,
            Payload::UnitConvert(_) | Payload::PendingUnitConvert { .. }
        ),
        Opcode::PiecewiseLinear => matches!(payload, Payload::PiecewiseLinear { .. }),
        Opcode::Add
        | Opcode::Sub
        | Opcode::Mul
        | Opcode::Div
        | Opcode::Pow
        | Opcode::Neg
        | Opcode::Abs
        | Opcode::Exp
        | Opcode::Log
        | Opcode::Log10
        | Opcode::Sqrt
        | Opcode::Sin
        | Opcode::Cos
        | Opcode::Tan
        | Opcode::Asin
        | Opcode::Acos
        | Opcode::Atan
        | Opcode::Sinh
        | Opcode::Cosh
        | Opcode::Tanh
        | Opcode::Erf => matches!(payload, Payload::None),
    };
    if ok {
        return Ok(());
    }
    let found = payload.relation_name().unwrap_or("no payload");
    Err(MathIrError::malformed(format!(
        "`{opcode}` does not carry the payload of `{found}`"
    )))
}

/// Is this a reduction payload of the kind the opcode names?
fn matches_reduction(payload: &Payload, expected: pse_quantity::ReductionKind) -> bool {
    matches!(payload, Payload::Reduction { kind, .. } if *kind == expected)
}

/// Does the child count satisfy the operator's arity (§7.2)?
fn check_arity(opcode: Opcode, children: usize) -> Result<(), MathIrError> {
    let declared = arity(opcode);
    if declared.admits(children) {
        return Ok(());
    }
    Err(MathIrError::malformed(format!(
        "`{opcode}` takes {declared} children, but {children} were offered"
    )))
}

/// The payload-specific value checks (§7.2 domain restrictions, §7.6 finiteness).
fn check_payload_values(
    opcode: Opcode,
    payload: &Payload,
    children: &[NodeId],
) -> Result<(), MathIrError> {
    if let Payload::SymbolRef { symbol } = payload {
        symbol.validate()?;
    }
    if let Some(domain) = payload.domain() {
        domain.validate()?;
    }

    match payload {
        Payload::FloatConst { value, .. } => require_finite(*value)?,
        Payload::Affine {
            constant,
            constant_quantity_type,
            constant_unit,
            terms,
        } => {
            check_affine_values(
                *constant,
                *constant_quantity_type,
                *constant_unit,
                terms,
                children,
            )?;
        }
        Payload::WeightedMean {
            pairs,
            normalization,
            unit_sum_invariant,
        } => {
            if pairs.is_empty() {
                return Err(MathIrError::malformed(
                    "`WeightedMean` requires a nonempty weight/value list",
                ));
            }
            if *normalization == pse_quantity::WeightNormalization::CertifiedUnitSum
                && unit_sum_invariant.is_none()
            {
                return Err(MathIrError::malformed(
                    "`certified_unit_sum` requires the invariant that certifies it",
                ));
            }
        }
        Payload::SmoothOp { eps } | Payload::PendingSmoothOp { eps, .. } => {
            require_finite(*eps)?;
            if *eps <= 0.0 {
                return Err(MathIrError::malformed(format!(
                    "`{opcode}` requires eps > 0, but {eps} was offered"
                )));
            }
        }
        Payload::PiecewiseLinear { breakpoints, .. } => {
            if breakpoints.is_empty() {
                return Err(MathIrError::malformed(
                    "`PiecewiseLinear` requires at least one breakpoint",
                ));
            }
            for (x, y) in breakpoints {
                require_finite(*x)?;
                require_finite(*y)?;
            }
            if breakpoints.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
                return Err(MathIrError::malformed(
                    "PiecewiseLinear coordinates must be strictly increasing",
                ));
            }
        }
        Payload::Derivative { order, .. } => {
            if *order == 0 {
                return Err(MathIrError::malformed(
                    "`Derivative` requires a positive order",
                ));
            }
        }
        Payload::UnitConvert(spec) => {
            require_finite(spec.scale)?;
            require_finite(spec.offset)?;
            if spec.scale <= 0.0 {
                return Err(MathIrError::malformed("UnitConvert scale must be positive"));
            }
        }
        Payload::None
        | Payload::SymbolRef { .. }
        | Payload::IntConst { .. }
        | Payload::Reduction { .. }
        | Payload::Gather { .. }
        | Payload::PendingGather { .. }
        | Payload::PendingPath { .. }
        | Payload::Broadcast { .. }
        | Payload::Integral { .. }
        | Payload::PendingUnitConvert { .. }
        | Payload::Conditional { .. }
        | Payload::KernelCall { .. }
        | Payload::ImplicitRef { .. } => {}
    }
    Ok(())
}

/// §7.6 leaves no representation for a non-finite literal; reject it at the door.
fn check_affine_values(
    constant: f64,
    constant_quantity_type: Option<QuantityTypeId>,
    constant_unit: Option<UnitId>,
    terms: &[crate::AffineTerm],
    children: &[NodeId],
) -> Result<(), MathIrError> {
    require_finite(constant)?;
    if constant_quantity_type.is_some() != constant_unit.is_some()
        || (constant != 0.0 && constant_quantity_type.is_none())
    {
        return Err(MathIrError::malformed(
            "Affine constant requires paired physical type and unit; a nonzero constant cannot be untyped",
        ));
    }
    if terms.len() != children.len() {
        return Err(MathIrError::malformed(format!(
            "`Affine` has {} terms but {} children",
            terms.len(),
            children.len()
        )));
    }
    for (position, term) in terms.iter().enumerate() {
        require_finite(term.coefficient)?;
        if children.get(position) != Some(&term.child) {
            return Err(MathIrError::malformed(format!(
                "`Affine` term {position} names {} but child {position} is {}",
                term.child,
                children
                    .get(position)
                    .map_or_else(|| "absent".to_owned(), NodeId::to_string)
            )));
        }
    }
    Ok(())
}

fn require_finite(value: f64) -> Result<(), MathIrError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(MathIrError::NonFiniteLiteral {
            value_bits: value.to_bits(),
        })
    }
}

fn typed_key(
    opcode: Opcode,
    payload: &Payload,
    children: &[NodeId],
    quantity_type: Option<QuantityTypeId>,
) -> Vec<u8> {
    let mut key = structural_key(opcode, payload, children);
    put_optional_id(&mut key, quantity_type.map(QuantityTypeId::as_id));
    key
}

/// The in-memory structural comparison key of a node.
///
/// Not a hash and not storage: a byte encoding used only to find an identical structure
/// already in this graph. Floats enter through `canonical_f64_bits`, so `-0.0` and `0.0`
/// stay distinct (ADR-0030); `scope` and `quantity_type` are excluded, matching what
/// §7.4 step 1 excludes from the untyped structural hash. This exact key remains the
/// equality check even when a digest is used to locate candidates.
pub(crate) fn structural_key(opcode: Opcode, payload: &Payload, children: &[NodeId]) -> Vec<u8> {
    let mut out = Vec::new();
    put_str(&mut out, opcode.as_str());
    encode_payload(&mut out, payload);
    put_len(&mut out, children.len());
    for child in children {
        put_u64(&mut out, child.0);
    }
    out
}

/// Appends a length-prefixed string.
fn put_str(out: &mut Vec<u8>, text: &str) {
    put_len(out, text.len());
    out.extend_from_slice(text.as_bytes());
}

/// Appends a little-endian `u32`.
fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Appends a length as a little-endian `u64`.
///
/// A `usize` fits a `u64` on every target this workspace builds for; the saturation keeps
/// the function total without a panic policy escape.
fn put_len(out: &mut Vec<u8>, len: usize) {
    put_u64(out, u64::try_from(len).unwrap_or(u64::MAX));
}

/// Appends a little-endian `u64`.
fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Appends a float through the ADR-0030 canonical bits.
fn put_f64(out: &mut Vec<u8>, value: f64) {
    put_u64(out, canonical_f64_bits(value));
}

/// Appends an identity.
fn put_id(out: &mut Vec<u8>, id: SemanticId) {
    out.extend_from_slice(id.as_bytes());
}

/// Appends an optional identity with an explicit presence byte.
fn put_domain(out: &mut Vec<u8>, domain: &crate::DomainRef) {
    match domain {
        crate::DomainRef::Actual(domain) => {
            out.push(0);
            put_id(out, domain.as_id());
        }
        crate::DomainRef::Template {
            template_id,
            domain_name,
        } => {
            out.push(1);
            put_id(out, *template_id);
            put_str(out, domain_name);
        }
    }
}

fn put_value(out: &mut Vec<u8>, value: &crate::ValueRef) {
    put_str(out, value.kind());
    match value {
        crate::ValueRef::ActualSymbol(symbol) => put_id(out, *symbol),
        crate::ValueRef::Template {
            template_id, name, ..
        } => {
            put_id(out, *template_id);
            put_str(out, name);
        }
        crate::ValueRef::Domain(domain) => put_domain(out, domain),
        crate::ValueRef::Index(index) => put_id(out, index.as_id()),
    }
}
fn put_guard(out: &mut Vec<u8>, guard: crate::GuardRef) {
    match guard {
        crate::GuardRef::Math(node) => {
            out.push(0);
            put_u64(out, node.0);
        }
        crate::GuardRef::Predicate {
            source_id,
            predicate_id,
        } => {
            out.push(1);
            put_id(out, source_id);
            put_u64(out, predicate_id);
        }
    }
}
fn put_optional_guard(out: &mut Vec<u8>, guard: Option<crate::GuardRef>) {
    match guard {
        Some(guard) => {
            out.push(1);
            put_guard(out, guard);
        }
        None => out.push(0),
    }
}

fn put_optional_id(out: &mut Vec<u8>, id: Option<SemanticId>) {
    match id {
        Some(value) => {
            out.push(1);
            put_id(out, value);
        }
        None => out.push(0),
    }
}

/// Appends the payload's discriminator and fields.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per §6.9 payload relation; splitting it would hide the one-to-one correspondence"
)]
pub(crate) fn encode_payload(out: &mut Vec<u8>, payload: &Payload) {
    put_str(out, payload.relation_name().unwrap_or(""));
    match payload {
        Payload::None => {}
        Payload::SymbolRef { symbol } => put_value(out, symbol),
        Payload::PendingPath {
            source_id,
            path_id,
            indices,
        } => {
            put_str(out, Payload::PENDING_PATH_KIND);
            put_id(out, *source_id);
            put_u64(out, *path_id);
            put_len(out, indices.len());
            for index in indices {
                put_u64(out, index.0);
            }
        }
        Payload::FloatConst { value, unit } => {
            put_f64(out, *value);
            put_id(out, unit.as_id());
        }
        Payload::IntConst { value } => out.extend_from_slice(&value.to_le_bytes()),
        Payload::Affine {
            constant,
            constant_quantity_type,
            constant_unit,
            terms,
        } => {
            put_f64(out, *constant);
            put_optional_id(out, constant_quantity_type.map(QuantityTypeId::as_id));
            put_optional_id(out, constant_unit.map(UnitId::as_id));
            put_len(out, terms.len());
            for term in terms {
                put_f64(out, term.coefficient);
                put_u64(out, term.child.0);
            }
        }
        Payload::WeightedMean {
            pairs,
            normalization,
            unit_sum_invariant,
        } => {
            put_len(out, pairs.len());
            for pair in pairs {
                put_u64(out, pair.weight.0);
                put_u64(out, pair.value.0);
            }
            put_str(out, normalization.as_str());
            put_optional_id(
                out,
                unit_sum_invariant.map(pse_quantity::InvariantId::as_id),
            );
        }
        Payload::Reduction {
            kind,
            domain,
            bound_index,
            filter,
        } => {
            put_str(out, kind.as_str());
            put_domain(out, domain);
            put_id(out, bound_index.as_id());
            put_optional_guard(out, *filter);
        }
        Payload::Gather {
            group,
            coordinate_map,
        } => {
            out.push(1);
            put_id(out, *group);
            put_len(out, coordinate_map.len());
            for (bound_index, position) in coordinate_map {
                put_id(out, bound_index.as_id());
                put_u32(out, u32::from(*position));
            }
        }
        Payload::PendingGather { group, indices } => {
            out.push(0);
            put_id(out, *group);
            put_len(out, indices.len());
            for index in indices {
                put_u64(out, index.0);
            }
        }
        Payload::Broadcast {
            domain,
            bound_index,
        } => {
            put_domain(out, domain);
            put_id(out, bound_index.as_id());
        }
        Payload::Derivative { wrt_domain, order } => {
            put_domain(out, wrt_domain);
            out.push(*order);
        }
        Payload::Integral {
            domain,
            bound_index,
            quadrature_policy,
            filter,
        } => {
            put_domain(out, domain);
            put_id(out, bound_index.as_id());
            put_optional_id(out, *quadrature_policy);
            match filter {
                None => out.push(0),
                Some(guard) => {
                    out.push(1);
                    put_guard(out, *guard);
                }
            }
        }
        Payload::SmoothOp { eps } => {
            out.push(0);
            put_f64(out, *eps);
        }
        Payload::PendingSmoothOp { eps, unit } => {
            out.push(1);
            put_f64(out, *eps);
            put_id(out, unit.as_id());
        }
        Payload::Conditional { guard } => put_guard(out, *guard),
        Payload::KernelCall {
            kernel_binding,
            output_ordinal,
        } => {
            put_id(out, *kernel_binding);
            put_u32(out, u32::from(*output_ordinal));
        }
        Payload::ImplicitRef {
            implicit_system,
            unknown_ordinal,
        } => {
            put_id(out, *implicit_system);
            put_u32(out, u32::from(*unknown_ordinal));
        }
        Payload::PendingUnitConvert { to } => {
            out.push(0);
            put_id(out, to.as_id());
        }
        Payload::UnitConvert(spec) => {
            out.push(1);
            put_id(out, spec.from.as_id());
            put_id(out, spec.to.as_id());
            put_f64(out, spec.scale);
            put_f64(out, spec.offset);
        }
        Payload::PiecewiseLinear {
            breakpoints,
            input,
            output,
        } => {
            put_len(out, breakpoints.len());
            for (x, y) in breakpoints {
                put_f64(out, *x);
                put_f64(out, *y);
            }
            put_id(out, input.as_id());
            put_id(out, output.as_id());
        }
    }
}

#[cfg(test)]
mod tests {
    use pse_ids::SemanticId;
    use pse_quantity::{
        BoundIndexId, DomainId, InvariantId, Opcode, QuantityTypeId, ReductionKind,
        UnitConvertSpec, UnitId, WeightNormalization,
    };

    use super::ExprGraph;
    use crate::error::MathIrError;
    use crate::node::NodeId;
    use crate::payload::{AffineTerm, Payload, WeightedPair};

    fn unit(byte: u8) -> UnitId {
        UnitId::from_id(SemanticId::from_bytes([byte; 16]))
    }

    #[test]
    fn a_repeated_structure_returns_the_existing_identity() {
        let mut graph = ExprGraph::new();
        let left = graph.float_const(1.0, unit(1)).expect("a finite literal");
        let right = graph.float_const(2.0, unit(1)).expect("a finite literal");
        let first = graph.add(left, right).expect("two existing children");
        let second = graph.add(left, right).expect("two existing children");
        assert_eq!(first, second);
        assert_eq!(graph.len(), 3);
    }

    /// Argument order is significant: §7.4 step 1 forbids sorting commutative children.
    #[test]
    fn argument_order_is_part_of_the_structure() {
        let mut graph = ExprGraph::new();
        let left = graph.float_const(1.0, unit(1)).expect("a finite literal");
        let right = graph.float_const(2.0, unit(1)).expect("a finite literal");
        let forwards = graph.add(left, right).expect("two existing children");
        let backwards = graph.add(right, left).expect("two existing children");
        assert_ne!(forwards, backwards);
    }

    /// ADR-0030: `-0.0` is preserved on the hashing path, so it is a different literal.
    #[test]
    fn negative_zero_and_positive_zero_are_distinct_literals() {
        let mut graph = ExprGraph::new();
        let positive = graph.float_const(0.0, unit(1)).expect("a finite literal");
        let negative = graph.float_const(-0.0, unit(1)).expect("a finite literal");
        assert_ne!(positive, negative);
        assert_eq!(graph.len(), 2);
    }

    /// The same literal in a different unit is a different node.
    #[test]
    fn the_unit_is_part_of_a_literal() {
        let mut graph = ExprGraph::new();
        let celsius = graph.float_const(20.0, unit(1)).expect("a finite literal");
        let kelvin = graph.float_const(20.0, unit(2)).expect("a finite literal");
        assert_ne!(celsius, kelvin);
    }

    /// §7.6 leaves no representation for a non-finite literal.
    #[test]
    fn a_non_finite_literal_is_refused() {
        let mut graph = ExprGraph::new();
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let refused = graph.float_const(value, unit(1));
            assert!(matches!(refused, Err(MathIrError::NonFiniteLiteral { .. })));
        }
        assert!(graph.is_empty());
    }

    #[test]
    fn an_arity_mismatch_is_malformed() {
        let mut graph = ExprGraph::new();
        let only = graph.int_const(1).expect("an integer literal");
        let refused = graph.insert(Opcode::Add, Payload::None, &[only], None);
        assert!(matches!(refused, Err(MathIrError::Malformed { .. })));

        let refused = graph.insert(Opcode::Neg, Payload::None, &[only, only], None);
        assert!(matches!(refused, Err(MathIrError::Malformed { .. })));
    }

    /// A child that does not exist cannot be referenced, which is what makes a cycle
    /// unrepresentable through this API.
    #[test]
    fn an_unknown_child_is_malformed() {
        let mut graph = ExprGraph::new();
        let real = graph.int_const(1).expect("an integer literal");
        let refused = graph.insert(Opcode::Add, Payload::None, &[real, NodeId(99)], None);
        let Err(MathIrError::Malformed { detail, .. }) = refused else {
            panic!("a dangling child must be refused");
        };
        assert!(detail.contains("#99"), "{detail}");
    }

    #[test]
    fn a_payload_reference_must_exist_too() {
        let mut graph = ExprGraph::new();
        let body = graph.int_const(1).expect("an integer literal");
        let refused = graph.insert(
            Opcode::SumOver,
            Payload::Reduction {
                kind: ReductionKind::Sum,
                domain: DomainId::from_id(SemanticId::NIL).into(),
                bound_index: BoundIndexId::from_id(SemanticId::NIL),
                filter: Some(NodeId(42).into()),
            },
            &[body],
            None,
        );
        assert!(matches!(refused, Err(MathIrError::Malformed { .. })));
    }

    #[test]
    fn affine_terms_and_children_must_agree() {
        let mut graph = ExprGraph::new();
        let first = graph.int_const(1).expect("an integer literal");
        let second = graph.int_const(2).expect("an integer literal");

        let too_few = graph.insert(
            Opcode::Affine,
            Payload::Affine {
                constant: 0.0,
                constant_quantity_type: None,
                constant_unit: None,
                terms: vec![AffineTerm {
                    coefficient: 1.0,
                    child: first,
                }],
            },
            &[first, second],
            None,
        );
        assert!(matches!(too_few, Err(MathIrError::Malformed { .. })));

        let out_of_order = graph.insert(
            Opcode::Affine,
            Payload::Affine {
                constant: 0.0,
                constant_quantity_type: None,
                constant_unit: None,
                terms: vec![
                    AffineTerm {
                        coefficient: 1.0,
                        child: second,
                    },
                    AffineTerm {
                        coefficient: 1.0,
                        child: first,
                    },
                ],
            },
            &[first, second],
            None,
        );
        assert!(matches!(out_of_order, Err(MathIrError::Malformed { .. })));

        let accepted = graph.insert(
            Opcode::Affine,
            Payload::Affine {
                constant: 0.5,
                constant_quantity_type: Some(QuantityTypeId::from_id(SemanticId::NIL)),
                constant_unit: Some(UnitId::from_id(SemanticId::NIL)),
                terms: vec![
                    AffineTerm {
                        coefficient: 1.0,
                        child: first,
                    },
                    AffineTerm {
                        coefficient: -1.0,
                        child: second,
                    },
                ],
            },
            &[first, second],
            None,
        );
        assert!(accepted.is_ok());
    }

    #[test]
    fn a_weighted_mean_needs_pairs_and_its_certificate() {
        let mut graph = ExprGraph::new();
        let weight = graph.int_const(1).expect("an integer literal");
        let value = graph.int_const(2).expect("an integer literal");

        let empty = graph.insert(
            Opcode::WeightedMean,
            Payload::WeightedMean {
                pairs: Vec::new(),
                normalization: WeightNormalization::DivideBySum,
                unit_sum_invariant: None,
            },
            &[],
            None,
        );
        assert!(matches!(empty, Err(MathIrError::Malformed { .. })));

        let uncertified = graph.insert(
            Opcode::WeightedMean,
            Payload::WeightedMean {
                pairs: vec![WeightedPair { weight, value }],
                normalization: WeightNormalization::CertifiedUnitSum,
                unit_sum_invariant: None,
            },
            &[],
            None,
        );
        assert!(matches!(uncertified, Err(MathIrError::Malformed { .. })));

        let certified = graph.insert(
            Opcode::WeightedMean,
            Payload::WeightedMean {
                pairs: vec![WeightedPair { weight, value }],
                normalization: WeightNormalization::CertifiedUnitSum,
                unit_sum_invariant: Some(InvariantId::from_id(SemanticId::NIL)),
            },
            &[],
            None,
        );
        assert!(certified.is_ok());
    }

    /// §7.2: a smooth operator's smoothing parameter must be positive.
    #[test]
    fn a_smooth_operator_needs_a_positive_eps() {
        let mut graph = ExprGraph::new();
        let left = graph.int_const(1).expect("an integer literal");
        let right = graph.int_const(2).expect("an integer literal");
        for eps in [0.0, -1e-3] {
            let refused = graph.insert(
                Opcode::SmoothMax,
                Payload::SmoothOp { eps },
                &[left, right],
                None,
            );
            assert!(matches!(refused, Err(MathIrError::Malformed { .. })));
        }
        let accepted = graph.insert(
            Opcode::SmoothMax,
            Payload::SmoothOp { eps: 1e-3 },
            &[left, right],
            None,
        );
        assert!(accepted.is_ok());
    }

    /// The payload belongs to the opcode: §6.9 stores one relation per operator family.
    #[test]
    fn a_payload_must_belong_to_its_opcode() {
        let mut graph = ExprGraph::new();
        let refused = graph.insert(Opcode::Add, Payload::IntConst { value: 1 }, &[], None);
        assert!(matches!(refused, Err(MathIrError::Malformed { .. })));

        let body = graph.int_const(1).expect("an integer literal");
        let wrong_kind = graph.insert(
            Opcode::ProdOver,
            Payload::Reduction {
                kind: ReductionKind::Sum,
                domain: DomainId::from_id(SemanticId::NIL).into(),
                bound_index: BoundIndexId::from_id(SemanticId::NIL),
                filter: None,
            },
            &[body],
            None,
        );
        assert!(matches!(wrong_kind, Err(MathIrError::Malformed { .. })));
    }

    /// Scope is excluded from the structural key (§7.4 step 1), so the same expression in
    /// two instances is one node, and conflicting scopes merge to `None`.
    #[test]
    fn scope_is_not_part_of_the_structure() {
        let mut graph = ExprGraph::new();
        let first_scope = SemanticId::from_bytes([7; 16]);
        let second_scope = SemanticId::from_bytes([8; 16]);
        let first = graph
            .insert(
                Opcode::SymbolRef,
                Payload::SymbolRef {
                    symbol: SemanticId::NIL.into(),
                },
                &[],
                Some(first_scope),
            )
            .expect("a symbol reference");
        let second = graph
            .insert(
                Opcode::SymbolRef,
                Payload::SymbolRef {
                    symbol: SemanticId::NIL.into(),
                },
                &[],
                Some(second_scope),
            )
            .expect("a symbol reference");
        assert_eq!(first, second);
        assert_eq!(graph.node(first).expect("the node exists").scope, None);
        let third = graph
            .insert(
                Opcode::SymbolRef,
                Payload::SymbolRef {
                    symbol: SemanticId::NIL.into(),
                },
                &[],
                Some(first_scope),
            )
            .expect("same symbol");
        assert_eq!(third, first);
        assert_eq!(graph.node(first).expect("node").scope, None);
    }

    #[test]
    fn a_unit_convert_carries_its_spec() {
        let mut graph = ExprGraph::new();
        let child = graph.int_const(1).expect("an integer literal");
        let spec = UnitConvertSpec {
            from: unit(1),
            to: unit(2),
            scale: 1.0,
            offset: 273.15,
        };
        let converted = graph.unit_convert(child, spec).expect("one existing child");
        let node = graph.node(converted).expect("the node exists");
        assert_eq!(node.opcode, Opcode::UnitConvert);
        assert_eq!(node.payload, Payload::UnitConvert(spec));

        // A different offset is a different node.
        let other = UnitConvertSpec {
            offset: 0.0,
            ..spec
        };
        assert_ne!(
            graph
                .unit_convert(child, other)
                .expect("one existing child"),
            converted
        );
    }

    #[test]
    fn an_unknown_identity_is_reported_rather_than_indexed() {
        let graph = ExprGraph::new();
        assert!(matches!(
            graph.node(NodeId(0)),
            Err(MathIrError::Malformed { .. })
        ));
    }

    #[test]
    fn a_quantity_type_is_recorded_by_p10_only() {
        let mut graph = ExprGraph::new();
        let node = graph.int_const(1).expect("an integer literal");
        assert!(
            graph
                .node(node)
                .expect("the node exists")
                .quantity_type
                .is_none()
        );

        let quantity_type = QuantityTypeId::from_id(SemanticId::from_bytes([3; 16]));
        graph
            .set_quantity_type(node, quantity_type)
            .expect("the node exists");
        assert_eq!(
            graph.node(node).expect("the node exists").quantity_type,
            Some(quantity_type)
        );
        assert!(graph.set_quantity_type(NodeId(42), quantity_type).is_err());
    }

    #[test]
    fn iteration_is_insertion_order() {
        let mut graph = ExprGraph::new();
        let first = graph.int_const(1).expect("an integer literal");
        let second = graph.int_const(2).expect("an integer literal");
        let ids: Vec<_> = graph.iter().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![first, second]);
    }
}
