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
    /// Inputs and writable outputs follow the exact declared port order. Every
    /// output must be set on success; an unwritten slot remains an explicit failure.
    fn evaluate(
        &mut self,
        inputs: &[f64],
        outputs: &mut [f64],
        execution: &Execution,
    ) -> Result<(), ProblemError>;
    /// Complete solved original local state after a successful unit evaluation,
    /// including non-port unknowns. Opaque explicit maps may have no local state.
    fn original_values(&self) -> pse_math::binding::CaseValues {
        pse_math::binding::CaseValues {
            scalars: BTreeMap::new(),
        }
    }
}
type OriginalObserver =
    Box<dyn FnMut(Option<&pse_math::binding::CaseValues>, &Execution) -> Result<(), ProblemError>>;
/// Complete declared causal map in selected tear destination coordinates.
#[derive(Debug)]
pub struct CausalMap {
    contract: OracleContract,
    original: faer::sparse::SymbolicSparseColMat<usize>,
    units: Vec<Box<dyn CausalUnit>>,
    exchange: Vec<UnitExchange>,
    tears: Vec<(usize, usize, pse_quantity::UnitConvertSpec)>,
    fixed: Vec<(usize, f64)>,
    values: Vec<f64>,
    predicted: Vec<f64>,
    execution: Execution,
    _graph: Arc<FlowGraph>,
    original_observer: Option<OriginalObserverWrapper>,
    inner_context: crate::solve::RawMapInnerContext,
}
/// Prepared ordinals preserve each unit's declared input/output order.
#[derive(Debug)]
struct UnitExchange {
    inputs: Vec<usize>,
    outputs: Vec<usize>,
    propagate: Vec<(usize, usize, pse_quantity::UnitConvertSpec)>,
    input_values: Vec<f64>,
    output_values: Vec<f64>,
}
impl std::fmt::Debug for OriginalObserverWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OriginalObserver")
    }
}
struct OriginalObserverWrapper(OriginalObserver);
impl CausalMap {
    /// Known propagation/tear containers and conservative full source support before
    /// constructing a causal map. Every original port may depend on every tear column;
    /// symbolic CSC construction retains input pairs, sorting maps and final indices.
    /// Unit evaluator storage is charged separately by the caller's WorkerBudget.
    pub fn construction_allocation_bound(
        graph: &FlowGraph,
        n: usize,
        fixed: usize,
    ) -> Result<usize, ProblemError> {
        let overflow = || ProblemError::Contract("causal construction extent overflow".into());
        let d = graph.declaration();
        let ports = d.nodes.iter().try_fold(0usize, |total, node| {
            total.checked_add(node.ports.len()).ok_or_else(overflow)
        })?;
        let bindings = d.connections.iter().try_fold(0usize, |total, edge| {
            total.checked_add(edge.bindings.len()).ok_or_else(overflow)
        })?;
        let coordinates = ports
            .checked_add(bindings)
            .and_then(|v| v.checked_add(fixed))
            .ok_or_else(overflow)?;
        let support = coordinates
            .checked_add(1)
            .and_then(|v| v.checked_mul(n.checked_add(1)?))
            .ok_or_else(overflow)?;
        let matrix = n.checked_mul(n).ok_or_else(overflow)?;
        let mut bytes = graph
            .tear_allocation_bound()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        for (count, width) in [
            (
                d.nodes.len(),
                11 * size_of::<(SemanticId, Vec<Binding>)>()
                    + 16 * size_of::<usize>()
                    + size_of::<Box<dyn CausalUnit>>(),
            ),
            (
                bindings,
                11 * size_of::<(SemanticId, Binding)>()
                    + 16 * size_of::<usize>()
                    + 2 * size_of::<Binding>(),
            ),
            (
                coordinates,
                11 * size_of::<(SemanticId, BTreeSet<usize>)>()
                    + 19 * size_of::<usize>()
                    + 3 * size_of::<f64>(),
            ),
            // Concurrent support sets plus candidate union; one full BTree node
            // per source/column pair covers sparsely occupied leaves and splits.
            (support, 11 * size_of::<usize>() + 16 * size_of::<usize>()),
            (
                matrix,
                size_of::<faer::sparse::Pair<usize, usize>>() + size_of::<[usize; 8]>(),
            ),
            (
                n.checked_add(1).ok_or_else(overflow)?,
                4 * size_of::<usize>(),
            ),
            (n, size_of::<f64>()),
        ] {
            bytes = count
                .checked_mul(2)
                .and_then(|v| v.checked_add(4))
                .and_then(|v| v.checked_mul(width))
                .and_then(|v| bytes.checked_add(v))
                .ok_or_else(overflow)?;
        }
        // Each Binding is cloned into the propagation/tear partition and again
        // into the final ordered tear Vec while the partition map still exists.
        graph
            .binding_payload_bytes()
            .map_err(|e| ProblemError::Contract(e.to_string()))?
            .checked_mul(2)
            .and_then(|v| bytes.checked_add(v))
            .and_then(|v| v.checked_add(size_of::<Self>()))
            .ok_or_else(overflow)
    }
    /// Assess complete original reconstruction only at independent final tear
    /// validation. An unsuccessful sweep never publishes a partial local state.
    /// Bind the context actually used by the scientific unit owners.
    pub fn with_inner_context(mut self, context: crate::solve::RawMapInnerContext) -> Self {
        self.inner_context = context;
        self
    }
    /// Attach independent complete-model validation. None begins a new assessment;
    /// Some carries its complete reconstruction after a successful sweep.
    pub fn with_original_observer(mut self, observer: OriginalObserver) -> Self {
        self.original_observer = Some(OriginalObserverWrapper(observer));
        self
    }
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
                .map_err(|e| ProblemError::Internal(e.to_string()))?;
        // Semantic port lookup ends at construction. Every sweep uses these exact
        // ordered gather/scatter and conversion slots, with no map rebuilt per unit.
        let slots: BTreeMap<_, _> = available
            .iter()
            .enumerate()
            .map(|(slot, &id)| (id, slot))
            .collect();
        let exchange = ordered
            .iter()
            .map(|unit| UnitExchange {
                inputs: unit.inputs().iter().map(|id| slots[id]).collect(),
                outputs: unit.outputs().iter().map(|id| slots[id]).collect(),
                propagate: propagate
                    .get(&unit.id())
                    .into_iter()
                    .flatten()
                    .map(|b| (slots[&b.source], slots[&b.target], b.conversion))
                    .collect(),
                input_values: vec![f64::NAN; unit.inputs().len()],
                output_values: vec![f64::NAN; unit.outputs().len()],
            })
            .collect();
        let tears = tears
            .iter()
            .map(|b| (slots[&b.source], slots[&b.target], b.conversion))
            .collect();
        let fixed = fixed
            .into_iter()
            .map(|(id, value)| (slots[&id], value))
            .collect();
        Ok(Self {
            contract,
            original,
            units: ordered,
            exchange,
            tears,
            fixed,
            values: vec![f64::NAN; slots.len()],
            predicted: vec![f64::NAN; n],
            execution,
            _graph: graph,
            original_observer: None,
            inner_context: crate::solve::RawMapInnerContext::default(),
        })
    }
    fn sweep(&mut self, x: &[f64]) -> Result<pse_math::binding::CaseValues, ProblemError> {
        if x.len() != self.tears.len() || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("invalid causal trial"));
        }
        self.values.fill(f64::NAN);
        for &(slot, value) in &self.fixed {
            self.values[slot] = value;
        }
        let values = &mut self.values;
        let mut original = pse_math::binding::CaseValues {
            scalars: BTreeMap::new(),
        };
        for (&(_, target, _), &value) in self.tears.iter().zip(x) {
            values[target] = value;
        }
        for (unit, exchange) in self.units.iter_mut().zip(&mut self.exchange) {
            if self.execution.stopped().is_some() {
                return Err(pse_math::MathError::Cancelled.into());
            }
            for (input, &slot) in exchange.input_values.iter_mut().zip(&exchange.inputs) {
                *input = values[slot];
            }
            exchange.output_values.fill(f64::NAN);
            unit.evaluate(
                &exchange.input_values,
                &mut exchange.output_values,
                &self.execution,
            )?;
            if exchange.output_values.iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical(format!(
                    "causal unit {} returned incomplete or nonfinite outputs",
                    unit.id()
                )));
            }
            for (&output, &slot) in exchange.output_values.iter().zip(&exchange.outputs) {
                values[slot] = output;
            }
            for (id, value) in unit.original_values().scalars {
                if !value.is_finite()
                    || original
                        .scalars
                        .insert(id, value)
                        .is_some_and(|previous| previous != value)
                {
                    return Err(ProblemError::Contract(
                        "conflicting or nonfinite original unit reconstruction".into(),
                    ));
                }
            }
            for &(source, target, conversion) in &exchange.propagate {
                let v = values[source] * conversion.scale + conversion.offset;
                if !v.is_finite() {
                    return Err(ProblemError::numerical("nonfinite flow conversion"));
                }
                values[target] = v;
            }
        }
        for (predicted, &(source, _, conversion)) in self.predicted.iter_mut().zip(&self.tears) {
            let v = values[source] * conversion.scale + conversion.offset;
            if !v.is_finite() {
                return Err(ProblemError::numerical("nonfinite tear conversion"));
            }
            *predicted = v;
        }
        Ok(original)
    }
}
impl FixedPointOracle for CausalMap {
    fn inner_context(&self) -> crate::solve::RawMapInnerContext {
        self.inner_context
    }
    fn physical_tear_coordinates(&self) -> bool {
        true
    }
    fn original_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.original.as_ref()
    }
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn map(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if out.len() != x.len() {
            return Err(ProblemError::Internal("causal output dimensions".into()));
        }
        self.sweep(x)?;
        out.copy_from_slice(&self.predicted);
        Ok(())
    }
    fn original_residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if out.len() != x.len() {
            return Err(ProblemError::Internal(
                "connection residual dimensions".into(),
            ));
        }
        if let Some(observer) = &mut self.original_observer {
            (observer.0)(None, &self.execution)?;
        }
        let original = self.sweep(x)?;
        if let Some(observer) = &mut self.original_observer {
            (observer.0)(Some(&original), &self.execution)?;
        }
        for ((out, p), x) in out.iter_mut().zip(&self.predicted).zip(x) {
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
        hidden: f64,
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
            v: &[f64],
            outputs: &mut [f64],
            _: &Execution,
        ) -> Result<(), ProblemError> {
            self.hidden = v[0] * 3.0;
            if v[0] >= 0.0 {
                outputs[0] = 0.5 * v[0] + 1.0;
            }
            Ok(())
        }
        fn original_values(&self) -> pse_math::binding::CaseValues {
            pse_math::binding::CaseValues {
                scalars: BTreeMap::from([(id(99), self.hidden)]),
            }
        }
    }
    #[derive(Debug)]
    struct PermutedUnit {
        inputs: [SemanticId; 2],
        outputs: [SemanticId; 2],
    }
    impl CausalUnit for PermutedUnit {
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
            inputs: &[f64],
            outputs: &mut [f64],
            _: &Execution,
        ) -> Result<(), ProblemError> {
            outputs[0] = 10. * inputs[0] + inputs[1];
            outputs[1] = inputs[0] - inputs[1];
            Ok(())
        }
    }
    #[test]
    fn indexed_exchange_preserves_declared_port_order_fixed_inputs_and_tear_conversion() {
        use pse_structural::flowsheet::*;
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = pse_quantity::standard::ids::quantity("neutral");
        let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
        let graph = Arc::new(
            FlowGraph::admit(
                Declaration {
                    nodes: vec![Node {
                        id: id(10),
                        ports: [1, 2, 3, 4]
                            .map(|n| pse_kernels::Port {
                                id: id(n),
                                quantity,
                                unit,
                            })
                            .to_vec(),
                    }],
                    connections: vec![Connection {
                        id: id(20),
                        from: id(10),
                        to: id(10),
                        decision: id(30),
                        bindings: vec![(id(4), id(1))],
                    }],
                    decisions: vec![Decision {
                        id: id(30),
                        cost: 1.,
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
        let unit: Box<dyn CausalUnit> = Box::new(PermutedUnit {
            inputs: [id(3), id(1)],
            outputs: [id(4), id(2)],
        });
        let mut map = CausalMap::new(
            graph,
            BTreeSet::from([id(30)]),
            contract,
            BTreeMap::from([(id(10), unit)]),
            BTreeMap::from([(id(3), 2.)]),
            execution(),
        )
        .unwrap();
        let mut out = [f64::NAN];
        map.map(&[5.], &mut out).unwrap();
        assert_eq!(out, [25.]);
        assert_eq!(map.values, [5., -3., 2., 25.]);
        map.tears[0].2.scale = 0.5;
        map.tears[0].2.offset = 3.;
        map.map(&[1.], &mut out).unwrap();
        assert_eq!(out, [13.5]);
        assert_eq!(map.values, [1., 1., 2., 21.]);
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
            hidden: 0.0,
        });
        let units = BTreeMap::from([(id(10), unit)]);
        let observed = std::rc::Rc::new(std::cell::RefCell::new(None));
        let final_state = observed.clone();
        let mut map = CausalMap::new(
            graph,
            BTreeSet::from([id(30)]),
            contract,
            units,
            BTreeMap::new(),
            execution(),
        )
        .unwrap()
        .with_original_observer(Box::new(move |state, _| {
            *final_state.borrow_mut() = state.cloned();
            Ok(())
        }));
        let mut out = [0.0];
        map.map(&[0.0], &mut out).unwrap();
        assert!(map.physical_tear_coordinates());
        assert_eq!(out, [1.0]);
        assert_eq!(map.exchange[0].inputs, [0]);
        assert_eq!(map.exchange[0].outputs, [1]);
        let capacity = (
            map.values.capacity(),
            map.exchange[0].input_values.capacity(),
            map.exchange[0].output_values.capacity(),
            map.predicted.capacity(),
        );
        let predicted_storage = map.predicted.as_ptr();
        assert!(observed.borrow().is_none());
        assert!(
            map.map(&[-1.0], &mut out).is_err(),
            "unwritten outputs cannot reuse preceding scratch"
        );
        assert_eq!(out, [1.0], "failed sweep cannot publish a partial map");
        map.map(&[0.0], &mut out).unwrap();
        assert_eq!(out, [1.0]);
        assert_eq!(
            capacity,
            (
                map.values.capacity(),
                map.exchange[0].input_values.capacity(),
                map.exchange[0].output_values.capacity(),
                map.predicted.capacity()
            )
        );
        assert_eq!(predicted_storage, map.predicted.as_ptr());
        map.original_residual(&[2.0], &mut out).unwrap();
        assert_eq!(out, [0.0]);
        assert_eq!(observed.borrow().as_ref().unwrap().scalars[&id(99)], 6.0);
        map.map(&[0.0], &mut out).unwrap();
        assert_eq!(out, [1.0]);
        assert_eq!(observed.borrow().as_ref().unwrap().scalars[&id(99)], 6.0);
        assert!(map.original_residual(&[f64::NAN], &mut out).is_err());
        assert!(
            observed.borrow().is_none(),
            "failed latest reconstruction cannot publish the preceding original candidate"
        );
    }
}
