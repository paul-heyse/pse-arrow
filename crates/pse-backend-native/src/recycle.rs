// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! An explicit causal sweep is a KINSOL map; no recycle iteration algorithm lives here.
use crate::{OracleContract, ProblemError, kinsol::FixedPointOracle, solve::Execution};
use pse_ids::SemanticId;
use pse_structural::flowsheet::{Binding, FlowGraph};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
/// One causal process unit. Nonlinear unit implementations must use an admitted
/// native solver and return only independently validated outputs. Direct explicit
/// physical calculations may return their exact outputs without a solver attempt.
pub trait CausalUnit: std::fmt::Debug {
    /// Owning flowsheet node.
    fn id(&self) -> SemanticId;
    /// Complete declared input ports; all must be available before this call.
    fn inputs(&self) -> &[SemanticId];
    /// Complete declared output ports, disjoint from inputs.
    fn outputs(&self) -> &[SemanticId];
    /// Compute one causal unit, respecting the inherited execution budget. This is
    /// a single unit calculation, never a caller-owned fixed-point update loop.
    fn evaluate(
        &mut self,
        inputs: &BTreeMap<SemanticId, f64>,
        execution: &Execution,
    ) -> Result<BTreeMap<SemanticId, f64>, ProblemError>;
}
/// Complete declared causal map in selected tear destination coordinates.
#[derive(Debug)]
pub struct CausalMap {
    contract: OracleContract,
    original: faer::sparse::SymbolicSparseColMat<usize>,
    units: Vec<Box<dyn CausalUnit>>,
    propagate: BTreeMap<SemanticId, Vec<Binding>>,
    tears: Vec<Binding>,
    fixed: BTreeMap<SemanticId, f64>,
    execution: Execution,
    _graph: Arc<FlowGraph>,
}
impl CausalMap {
    /// Admit exact unit inventory, physical bindings, tear coordinates and the
    /// independently checked residual DAG. The declared row order is the original
    /// connection-residual order corresponding to these tear destination variables.
    pub fn new(
        graph: Arc<FlowGraph>,
        selected: BTreeSet<SemanticId>,
        contract: OracleContract,
        mut units: BTreeMap<SemanticId, Box<dyn CausalUnit>>,
        fixed: BTreeMap<SemanticId, f64>,
        execution: Execution,
    ) -> Result<Self, ProblemError> {
        contract.validate(pse_kernels::DerivativeOrder::Value)?;
        if contract.rows.len() != contract.variables.len() || fixed.values().any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "causal map dimensions/values".into(),
            ));
        }
        let order = graph
            .witness(&selected)
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        if units.len() != order.len() {
            return Err(ProblemError::Contract(
                "causal unit inventory differs from complete flow graph".into(),
            ));
        }
        let mut propagate: BTreeMap<_, Vec<_>> = BTreeMap::new();
        let mut tears = BTreeMap::new();
        let mut connected = BTreeSet::new();
        for e in &graph.declaration().connections {
            let source = units
                .get(&e.from)
                .ok_or_else(|| ProblemError::Contract("missing causal source".into()))?;
            let target = units
                .get(&e.to)
                .ok_or_else(|| ProblemError::Contract("missing causal destination".into()))?;
            for b in &graph.bindings()[&e.id] {
                if !source.outputs().contains(&b.source) || !target.inputs().contains(&b.target) {
                    return Err(ProblemError::Contract(
                        "connection must bind a causal output to a causal input".into(),
                    ));
                }
                connected.insert(b.target);
                if selected.contains(&e.decision) {
                    tears.insert(b.target, b.clone());
                } else {
                    propagate.entry(e.from).or_default().push(b.clone());
                }
            }
        }
        let external: BTreeSet<_> = units
            .values()
            .flat_map(|u| u.inputs())
            .copied()
            .filter(|id| !connected.contains(id))
            .collect();
        if fixed.keys().copied().collect::<BTreeSet<_>>() != external {
            return Err(ProblemError::Contract(
                "fixed inputs must exactly cover unconnected causal inputs".into(),
            ));
        }
        if tears.len() != contract.variables.len()
            || contract
                .variables
                .iter()
                .any(|v| !tears.contains_key(&v.id) || fixed.contains_key(&v.id))
        {
            return Err(ProblemError::Contract(
                "tear coordinate inventory differs from declared map".into(),
            ));
        }
        let mut available: BTreeSet<_> =
            fixed.keys().copied().chain(tears.keys().copied()).collect();
        let mut ordered = vec![];
        for id in order {
            let unit = units
                .remove(&id)
                .ok_or_else(|| ProblemError::Contract("missing causal unit".into()))?;
            let node = graph
                .declaration()
                .nodes
                .iter()
                .find(|n| n.id == id)
                .ok_or_else(|| ProblemError::Contract("missing flow node".into()))?;
            let ports: BTreeSet<_> = node.ports.iter().map(|p| p.id).collect();
            let declared: BTreeSet<_> = unit
                .inputs()
                .iter()
                .chain(unit.outputs())
                .copied()
                .collect();
            if unit.id() != id
                || declared != ports
                || declared.len() != unit.inputs().len() + unit.outputs().len()
                || unit.inputs().iter().any(|v| !available.contains(v))
                || unit.outputs().iter().any(|v| available.contains(v))
            {
                return Err(ProblemError::Contract(format!(
                    "causal unit {id} has incomplete/overlapping ports or missing predecessor input"
                )));
            }
            available.extend(unit.outputs().iter().copied());
            for b in propagate.get(&id).into_iter().flatten() {
                if !unit.outputs().contains(&b.source) {
                    return Err(ProblemError::Contract(
                        "flow source is not a causal output".into(),
                    ));
                }
                available.insert(b.target);
            }
            ordered.push(unit);
        }
        if tears.values().any(|b| !available.contains(&b.source)) {
            return Err(ProblemError::Contract(
                "tear source is unavailable after causal sweep".into(),
            ));
        }
        let tears: Vec<_> = contract
            .variables
            .iter()
            .map(|v| tears[&v.id].clone())
            .collect();
        // Causal units are opaque: conservatively retain all declared input support.
        // This projects the already admitted DAG, not the iteration/splitting matrix.
        let mut support: BTreeMap<SemanticId, BTreeSet<usize>> =
            fixed.keys().map(|&id| (id, BTreeSet::new())).collect();
        for (column, variable) in contract.variables.iter().enumerate() {
            support.insert(variable.id, BTreeSet::from([column]));
        }
        for unit in &ordered {
            let columns: BTreeSet<_> = unit
                .inputs()
                .iter()
                .flat_map(|id| support[id].iter().copied())
                .collect();
            for &output in unit.outputs() {
                support.insert(output, columns.clone());
            }
            for binding in propagate.get(&unit.id()).into_iter().flatten() {
                support.insert(binding.target, support[&binding.source].clone());
            }
        }
        let mut indices = Vec::new();
        for (row, binding) in tears.iter().enumerate() {
            let mut columns = support[&binding.source].clone();
            columns.insert(row); // The original connection residual is predicted - x.
            indices.extend(columns.into_iter().map(|c| faer::sparse::Pair::new(row, c)));
        }
        let n = contract.variables.len();
        let (original, _) =
            faer::sparse::SymbolicSparseColMat::try_new_from_indices(n, n, &indices)
                .map_err(|e| ProblemError::Contract(e.to_string()))?;
        Ok(Self {
            contract,
            original,
            units: ordered,
            propagate,
            tears,
            fixed,
            execution,
            _graph: graph,
        })
    }
    fn sweep(&mut self, x: &[f64]) -> Result<Vec<f64>, ProblemError> {
        if x.len() != self.tears.len() || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract("invalid causal trial".into()));
        }
        let mut values = self.fixed.clone();
        values.extend(self.tears.iter().zip(x).map(|(b, v)| (b.target, *v)));
        for unit in &mut self.units {
            if self.execution.stopped().is_some() {
                return Err(pse_math::MathError::Cancelled.into());
            }
            let inputs = unit.inputs().iter().map(|id| (*id, values[id])).collect();
            let outputs = unit.evaluate(&inputs, &self.execution)?;
            if outputs.len() != unit.outputs().len()
                || unit
                    .outputs()
                    .iter()
                    .any(|id| outputs.get(id).is_none_or(|v| !v.is_finite()))
            {
                return Err(ProblemError::Contract(format!(
                    "causal unit {} returned incomplete or nonfinite outputs",
                    unit.id()
                )));
            }
            values.extend(outputs);
            for b in self.propagate.get(&unit.id()).into_iter().flatten() {
                let v = values[&b.source] * b.conversion.scale + b.conversion.offset;
                if !v.is_finite() {
                    return Err(ProblemError::Contract("nonfinite flow conversion".into()));
                }
                values.insert(b.target, v);
            }
        }
        self.tears
            .iter()
            .map(|b| {
                let v = values[&b.source] * b.conversion.scale + b.conversion.offset;
                if v.is_finite() {
                    Ok(v)
                } else {
                    Err(ProblemError::Contract("nonfinite tear conversion".into()))
                }
            })
            .collect()
    }
}
impl FixedPointOracle for CausalMap {
    fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.original.as_ref()
    }
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if out.len() != x.len() {
            return Err(ProblemError::Contract("causal output dimensions".into()));
        }
        let values = self.sweep(x)?;
        out.copy_from_slice(&values);
        Ok(())
    }
    fn original_residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if out.len() != x.len() {
            return Err(ProblemError::Contract(
                "connection residual dimensions".into(),
            ));
        }
        let predicted = self.sweep(x)?;
        for ((out, p), x) in out.iter_mut().zip(predicted).zip(x) {
            *out = p - x;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver_tests::{execution, id};
    #[derive(Debug)]
    struct Unit {
        inputs: [SemanticId; 1],
        outputs: [SemanticId; 1],
    }
    impl CausalUnit for Unit {
        fn id(&self) -> SemanticId {
            id(10)
        }
        fn inputs(&self) -> &[SemanticId] {
            &self.inputs
        }
        fn outputs(&self) -> &[SemanticId] {
            &self.outputs
        }
        fn evaluate(
            &mut self,
            v: &BTreeMap<SemanticId, f64>,
            _: &Execution,
        ) -> Result<BTreeMap<SemanticId, f64>, ProblemError> {
            Ok(BTreeMap::from([(id(2), 0.5 * v[&id(1)] + 1.0)]))
        }
    }
    #[test]
    fn causal_map_uses_declared_tears_and_recomputes_original_connection_residual() {
        use pse_structural::flowsheet::*;
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = pse_quantity::standard::ids::quantity("neutral");
        let unit = registry.quantity_type(q).unwrap().canonical_unit;
        let graph = Arc::new(
            FlowGraph::admit(
                Declaration {
                    nodes: vec![Node {
                        id: id(10),
                        ports: vec![
                            pse_kernels::Port {
                                id: id(1),
                                quantity: q,
                                unit,
                            },
                            pse_kernels::Port {
                                id: id(2),
                                quantity: q,
                                unit,
                            },
                        ],
                    }],
                    connections: vec![Connection {
                        id: id(20),
                        from: id(10),
                        to: id(10),
                        decision: id(30),
                        bindings: vec![(id(2), id(1))],
                    }],
                    decisions: vec![Decision {
                        id: id(30),
                        cost: 1.0,
                        policy: Policy::Mandatory,
                    }],
                },
                &registry,
                pse_structural::projection::GraphLimits { nodes: 5, edges: 5 },
            )
            .unwrap(),
        );
        let mut contract = crate::solver_tests::contract();
        contract.rows = vec![id(20)];
        let unit: Box<dyn CausalUnit> = Box::new(Unit {
            inputs: [id(1)],
            outputs: [id(2)],
        });
        let units = BTreeMap::from([(id(10), unit)]);
        let mut map = CausalMap::new(
            graph,
            BTreeSet::from([id(30)]),
            contract,
            units,
            BTreeMap::new(),
            execution(),
        )
        .unwrap();
        let mut out = [0.0];
        map.map(&[0.0], &mut out).unwrap();
        assert_eq!(out, [1.0]);
        map.original_residual(&[2.0], &mut out).unwrap();
        assert_eq!(out, [0.0]);
        map.map(&[0.0], &mut out).unwrap();
        assert_eq!(out, [1.0]);
    }
}
