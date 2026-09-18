// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Structural admission of rows before any caller may use their identities.
use super::vec_sink::{KernelBinding, QuantitySelection};
use super::{MathRelationSource, VecSink};
use crate::{ExprGraph, MathIrError, Node, NodeId, Payload, equation::EquationRecord};
use pse_ids::SemanticId;
use std::collections::{BTreeMap, BTreeSet};

/// Structurally admitted row data. Supplied types, selections and hashes have not been
/// physically re-established; the inference driver must do that before canonical use.
#[derive(Clone, Debug)]
pub struct LoadedMath {
    /// Acyclic graph with locally remapped ordinals.
    pub graph: ExprGraph,
    /// Every equation and all expression references, remapped together.
    pub equations: Vec<EquationRecord>,
    /// Explicit caller roots, preserving their order and multiplicity.
    pub roots: Vec<NodeId>,
    /// Declared selections retained for comparison with actual inference.
    pub selections: Vec<QuantitySelection>,
    /// Bound kernel rows with all input expressions remapped.
    pub kernel_bindings: BTreeMap<SemanticId, KernelBinding>,
    /// Original storage ordinal to admitted graph ordinal.
    pub node_mapping: BTreeMap<NodeId, NodeId>,
}
/// Load and structurally admit complete relation rows. Node IDs need not be contiguous,
/// and row order is irrelevant; child/payload/list order remains authoritative.
///
/// # Errors
/// Rejects duplicate/orphan rows, wrong payloads, nonfinite values, dangling
/// references and cycles, including kernel inputs and unreachable components. Digests are
/// deliberately not consulted to determine validity.
pub fn load_untyped(
    source: &dyn MathRelationSource,
    roots: &[NodeId],
) -> Result<LoadedMath, MathIrError> {
    let mut rows = VecSink::new();
    source.read(&mut rows)?;
    let mut nodes = load_nodes(&mut rows)?;
    let mut bindings = load_bindings(rows.bindings, &nodes)?;
    let order = binding_order(&nodes, &bindings)?;
    let mut graph = ExprGraph::new();
    let mut remap = BTreeMap::new();
    for old in order {
        let mut node = nodes
            .remove(&old)
            .ok_or_else(|| MathIrError::malformed_at(old, "missing traversed node"))?;
        let map = |id| mapped(&remap, id);
        node.children = node
            .children
            .iter()
            .copied()
            .map(map)
            .collect::<Result<_, _>>()?;
        node.payload.map_node_references(map)?;
        let new = match node.quantity_type {
            Some(quantity) => graph.insert_typed(
                node.opcode,
                node.payload,
                &node.children,
                quantity,
                node.scope,
            )?,
            None => graph.insert(node.opcode, node.payload, &node.children, node.scope)?,
        };
        remap.insert(old, new);
    }
    for binding in bindings.values_mut() {
        for (_, node) in &mut binding.inputs {
            *node = mapped(&remap, *node)?;
        }
    }
    let equations = load_equations(rows.equations, rows.indices, &remap)?;
    let selections = load_selections(rows.selections, &remap)?;
    Ok(LoadedMath {
        roots: roots
            .iter()
            .copied()
            .map(|id| mapped(&remap, id))
            .collect::<Result<_, _>>()?,
        graph,
        equations,
        selections,
        kernel_bindings: bindings,
        node_mapping: remap,
    })
}
fn mapped(remap: &BTreeMap<NodeId, NodeId>, id: NodeId) -> Result<NodeId, MathIrError> {
    remap
        .get(&id)
        .copied()
        .ok_or_else(|| MathIrError::malformed_at(id, "referenced node does not exist"))
}
fn load_nodes(rows: &mut VecSink) -> Result<BTreeMap<NodeId, Node>, MathIrError> {
    let mut nodes = BTreeMap::new();
    for (id, opcode, children, payload, quantity_type, scope, _) in rows.nodes.drain(..) {
        if id.0 < 0 {
            return Err(MathIrError::malformed_at(
                id,
                "negative expression-node ordinal",
            ));
        }
        if nodes
            .insert(
                id,
                Node {
                    opcode,
                    payload,
                    children,
                    quantity_type,
                    scope,
                },
            )
            .is_some()
        {
            return Err(MathIrError::malformed_at(
                id,
                "duplicate expression-node row",
            ));
        }
    }
    Ok(nodes)
}
fn load_bindings(
    rows: Vec<KernelBinding>,
    nodes: &BTreeMap<NodeId, Node>,
) -> Result<BTreeMap<SemanticId, KernelBinding>, MathIrError> {
    let mut bindings = BTreeMap::new();
    for row in rows {
        let mut names = BTreeSet::new();
        for (name, symbol, value, unit) in &row.parameters {
            if name.is_empty() || !names.insert(name) {
                return Err(MathIrError::malformed(
                    "duplicate or empty kernel parameter name",
                ));
            }
            if !matches!(
                (symbol, value, unit),
                (Some(_), None, None) | (None, Some(_), Some(_))
            ) || value.is_some_and(|value| !value.is_finite())
            {
                return Err(MathIrError::malformed(
                    "kernel parameter requires exactly a symbol or finite value with unit",
                ));
            }
        }
        names.clear();
        for (name, node) in &row.inputs {
            if name.is_empty() || !names.insert(name) {
                return Err(MathIrError::malformed(
                    "duplicate or empty kernel input name",
                ));
            }
            if !nodes.contains_key(node) {
                return Err(MathIrError::malformed_at(*node, "dangling kernel input"));
            }
        }
        if bindings.insert(row.binding, row).is_some() {
            return Err(MathIrError::malformed("duplicate kernel binding row"));
        }
    }
    Ok(bindings)
}
pub(crate) fn binding_order(
    nodes: &BTreeMap<NodeId, Node>,
    bindings: &BTreeMap<SemanticId, KernelBinding>,
) -> Result<Vec<NodeId>, MathIrError> {
    let mut dependencies = nodes.clone();
    for (id, node) in &mut dependencies {
        if let Payload::KernelCall { kernel_binding, .. } = node.payload {
            let binding = bindings
                .get(&kernel_binding)
                .ok_or(MathIrError::UnknownBinding {
                    node: *id,
                    binding: kernel_binding,
                })?;
            // This copy is solely for cycle traversal; external inputs never become args.
            node.children
                .extend(binding.inputs.iter().map(|(_, child)| *child));
        }
    }
    crate::topo::validate_nodes(&dependencies)
}
fn load_equations(
    rows: Vec<EquationRecord>,
    indices: Vec<(SemanticId, crate::equation::FreeIndex)>,
    remap: &BTreeMap<NodeId, NodeId>,
) -> Result<Vec<EquationRecord>, MathIrError> {
    let mut equations = BTreeMap::new();
    for mut equation in rows {
        crate::canonicalize::validate_equation_bounds(&equation)?;
        equation.map_node_references(|id| mapped(remap, id))?;
        if equations
            .insert(equation.indexed_equation_id, equation)
            .is_some()
        {
            return Err(MathIrError::malformed("duplicate indexed equation row"));
        }
    }
    for (id, index) in indices {
        let equation = equations
            .get_mut(&id)
            .ok_or_else(|| MathIrError::malformed("orphan equation free-index row"))?;
        if equation.free_indices.iter().any(|existing| {
            existing.bound_index == index.bound_index || existing.position == index.position
        }) {
            return Err(MathIrError::malformed(
                "duplicate equation binder or axis position",
            ));
        }
        equation.free_indices.push(index);
    }
    for equation in equations.values_mut() {
        equation
            .free_indices
            .sort_unstable_by_key(|index| index.position);
        if equation
            .free_indices
            .iter()
            .enumerate()
            .any(|(position, index)| position != usize::from(index.position))
        {
            return Err(MathIrError::malformed("noncontiguous equation index axes"));
        }
    }
    Ok(equations.into_values().collect())
}
fn load_selections(
    rows: Vec<QuantitySelection>,
    remap: &BTreeMap<NodeId, NodeId>,
) -> Result<Vec<QuantitySelection>, MathIrError> {
    let mut seen = BTreeSet::new();
    let mut selections = BTreeMap::new();
    for mut row in rows {
        if !seen.insert(row.node) {
            return Err(MathIrError::malformed_at(
                row.node,
                "duplicate quantity-selection row",
            ));
        }
        if row.operation.is_some() == row.builtin.is_some() {
            return Err(MathIrError::malformed_at(
                row.node,
                "selection requires exactly one operation or built-in rule",
            ));
        }
        if row.builtin.is_some() && !row.permutation.is_empty() {
            return Err(MathIrError::malformed_at(
                row.node,
                "built-in selection has no matched operand permutation",
            ));
        }
        let mut permutation = row.permutation.clone();
        permutation.sort_unstable();
        if permutation
            .iter()
            .enumerate()
            .any(|(ordinal, slot)| ordinal != usize::from(*slot))
        {
            return Err(MathIrError::malformed_at(
                row.node,
                "invalid operand permutation",
            ));
        }
        let mut conversions = BTreeSet::new();
        if row
            .conversions
            .iter()
            .any(|(operand, _)| !conversions.insert(*operand))
        {
            return Err(MathIrError::malformed_at(
                row.node,
                "duplicate operand conversion",
            ));
        }
        row.node = mapped(remap, row.node)?;
        if let Some(existing) = selections.get(&row.node) {
            if existing != &row {
                return Err(MathIrError::malformed_at(
                    row.node,
                    "shared structure has contradictory selection rows",
                ));
            }
        } else {
            selections.insert(row.node, row);
        }
    }
    Ok(selections.into_values().collect())
}

/// Reload a canonical relation bundle by repeating physical inference from actual source
/// facts, then comparing every emitted row and nested value. Stored digests are checked
/// only after structural and physical admission has succeeded.
///
/// # Errors
/// Rejects untyped rows, failed inference, stale selections, modified nested values,
/// noncanonical storage numbering, or a digest inconsistent with the admitted contents.
pub fn load_canonical(
    source: &dyn MathRelationSource,
    roots: &[NodeId],
    symbols: &dyn crate::infer::SymbolTypeSource,
    registry: &pse_quantity::QuantityRegistry,
) -> Result<crate::CanonicalGraph, MathIrError> {
    let mut stored = VecSink::new();
    source.read(&mut stored)?;
    if stored.nodes.iter().any(|row| row.4.is_none()) {
        return Err(MathIrError::malformed(
            "canonical node lacks a complete quantity type",
        ));
    }
    let loaded = load_untyped(&stored, roots)?;
    let input = crate::canonicalize::CanonicalizeInput {
        graph: &loaded.graph,
        equations: &loaded.equations,
        roots: &loaded.roots,
        symbols,
        registry,
        kernel_bindings: &loaded.kernel_bindings,
        selections: &loaded.selections,
    };
    let graph = crate::canonicalize::canonicalize(input, crate::canonicalize::Policy::Strict)?;
    let mut expected = VecSink::new();
    super::emit(&graph, &mut expected)?;
    if !stored.same_rows(&expected) {
        return Err(MathIrError::malformed(
            "stored canonical rows differ from uncached physical reconstruction",
        ));
    }
    Ok(graph)
}
