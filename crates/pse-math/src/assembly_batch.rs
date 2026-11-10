// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure observation populations through admitted physical gathers and row transport.
use super::*;
use crate::batch::{
    BatchBindings, BatchRequest, BatchWorker, PointOutcome, PointResult, PreparedBatch,
};

/// One original observation point; equal numerical values still retain separate identities.
#[derive(Clone, Copy, Debug)]
pub struct CaseBatchPoint<'a> {
    /// Caller-issued scientific point identity.
    pub id: SemanticId,
    /// Values in the existing original physical case universe.
    pub values: &'a CaseValues,
}
/// Ordered pure constraint observations in the original case's selected row coordinates.
#[derive(Debug)]
pub struct CaseBatchOutcome {
    rows: Vec<SemanticId>,
    points: Vec<PointResult>,
    values: Vec<f64>,
    canceled: bool,
}
impl CaseBatchOutcome {
    pub(crate) fn from_parts(
        rows: Vec<SemanticId>,
        points: Vec<PointResult>,
        values: Vec<f64>,
        canceled: bool,
    ) -> Result<Self, MathError> {
        if rows.len().checked_mul(points.len()) != Some(values.len()) {
            return Err(MathError::Contract("case batch result extent".into()));
        }
        let mut outcome = Self {
            rows,
            points,
            values,
            canceled,
        };
        for (index, point) in outcome.points.iter_mut().enumerate() {
            if !matches!(point.outcome, PointOutcome::Succeeded) {
                outcome.values[index * outcome.rows.len()..(index + 1) * outcome.rows.len()]
                    .fill(f64::NAN);
                point.applicability.clear();
            }
        }
        Ok(outcome)
    }
    /// Actual owned result storage, including original failure and scientific evidence payloads.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.rows.capacity() * size_of::<SemanticId>()
            + self.values.capacity() * size_of::<f64>()
            + self.points.capacity() * size_of::<PointResult>()
            + self
                .points
                .iter()
                .map(|point| {
                    point.applicability.capacity()
                        * size_of::<pse_model::applicability::Observation>()
                        + point
                            .applicability
                            .iter()
                            .map(pse_model::applicability::Observation::retained_bytes)
                            .sum::<usize>()
                        + match &point.outcome {
                            PointOutcome::Failed(cause) => cause.retained_bytes(),
                            _ => 0,
                        }
                })
                .sum::<usize>()
    }
    /// Original selected row identities in result-cell order.
    pub fn rows(&self) -> &[SemanticId] {
        &self.rows
    }
    /// Every original scientific point, including individual failures and cancellation.
    pub fn points(&self) -> &[PointResult] {
        &self.points
    }
    /// Only successful points expose canonical physical output values.
    pub fn row(&self, point: usize) -> Option<&[f64]> {
        if !matches!(self.points.get(point)?.outcome, PointOutcome::Succeeded) {
            return None;
        }
        self.values
            .get(point * self.rows.len()..(point + 1) * self.rows.len())
    }
    /// True when cancellation prevented a complete observation operation.
    pub fn canceled(&self) -> bool {
        self.canceled
    }
}
struct LocalBatch {
    prepared: PreparedBatch,
    worker: BatchWorker,
}
/// Private reusable local workers, packing and scatter state over one admitted assembly.
pub struct CaseBatchWorker {
    assembly: Arc<CaseAssembly>,
    locals: Vec<Option<LocalBatch>>,
    inputs: Vec<f64>,
    row: Vec<f64>,
    ids: Vec<SemanticId>,
    indices: Vec<usize>,
    scope: pse_kernels::ExecutionScope,
}
impl std::fmt::Debug for CaseBatchWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaseBatchWorker")
            .field("templates", &self.locals.len())
            .field("rows", &self.assembly.rows.len())
            .finish_non_exhaustive()
    }
}
impl CaseAssembly {
    /// Conservative live storage for a pure Value population, before worker creation.
    fn batch_owned_backing(&self, points: usize) -> Result<usize, MathError> {
        let max_inputs = self
            .bindings
            .iter()
            .filter_map(|bindings| bindings[EvaluationDemand::Constraints].as_ref())
            .map(EvalBindings::len)
            .max()
            .unwrap_or(0);
        let max_outputs = self
            .instances
            .iter()
            .filter_map(|i| i.groups[EvaluationDemand::Constraints].as_ref())
            .map(|g| g.outputs.len())
            .max()
            .unwrap_or(0);
        let mut witness_bytes = 0usize;
        let mut source_bytes = 0usize;
        for (instance, binding) in self.instances.iter().zip(self.structure.instances()) {
            if let Some(group) = &instance.groups[EvaluationDemand::Constraints] {
                witness_bytes = witness_bytes
                    .checked_add(self.programs[group.request].evaluation_cache_bytes())
                    .ok_or(MathError::Limit("case batch witnesses"))?;
                let locations = binding
                    .occurrences
                    .values()
                    .map(pse_model::diagnostic::SourceLocation::heap_bytes)
                    .sum::<usize>();
                let entries = binding
                    .occurrences
                    .len()
                    .checked_mul(
                        size_of::<(SemanticId, pse_model::diagnostic::SourceLocation)>() + 96,
                    )
                    .and_then(|n| {
                        n.checked_add(
                            (binding.checked_members.len() + binding.checked_sources.len())
                                * (size_of::<(SemanticId, SemanticId)>() + 96),
                        )
                    })
                    .and_then(|n| n.checked_add(locations))
                    .ok_or(MathError::Limit("case batch source context"))?;
                source_bytes = source_bytes.max(entries);
            }
        }
        let per_point = max_inputs
            .checked_add(max_outputs)
            .and_then(|n| n.checked_add(self.rows.len()))
            .and_then(|n| n.checked_mul(2 * size_of::<f64>()))
            .and_then(|n| {
                n.checked_add(
                    2 * size_of::<PointResult>() + 2 * size_of::<SemanticId>() + size_of::<usize>(),
                )
            })
            .and_then(|n| n.checked_add(witness_bytes))
            .and_then(|n| n.checked_add(source_bytes))
            .ok_or(MathError::Limit("case batch population storage"))?;
        points
            .checked_mul(per_point)
            .and_then(|n| n.checked_add(size_of::<CaseBatchWorker>()))
            .ok_or(MathError::Limit("case batch worker storage"))
    }
    /// Complete source-issued constructor allowance, including the existing
    /// monotonic admission of sequential local result allocations. This is a
    /// conservative backing bound, rather than a measured peak-heap claim.
    pub fn batch_worker_bytes(&self, points: usize) -> Result<usize, MathError> {
        let mut bytes = self
            .numeric_worker_bytes
            .checked_add(self.batch_owned_backing(points)?)
            .ok_or(MathError::Limit("case batch worker storage"))?;
        for instance in self.instances.iter() {
            if let Some(group) = &instance.groups[EvaluationDemand::Constraints] {
                let prepared = PreparedBatch::new(
                    self.programs[group.request].clone(),
                    DerivativeOrder::Value,
                )?;
                bytes = bytes
                    .checked_add(prepared.layout().backing_bytes(points)?)
                    .ok_or(MathError::Limit("case batch local results"))?;
            }
        }
        Ok(bytes)
    }
    /// Prepare pure observation batches through this compiler-issued assembly. Current
    /// instance/source context stays authoritative while numerical programs are shared.
    pub fn batch_worker(
        self: &Arc<Self>,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<CaseBatchWorker, MathError> {
        crate::construction::additional(self.numeric_worker_bytes)?;
        let mut locals = (0..self.programs.len())
            .map(|_| None)
            .collect::<Vec<Option<LocalBatch>>>();
        let mut max_inputs = 0;
        for (instance, bindings) in self.instances.iter().zip(&self.bindings) {
            let Some(group) = &instance.groups[EvaluationDemand::Constraints] else {
                continue;
            };
            max_inputs = max_inputs.max(
                bindings[EvaluationDemand::Constraints]
                    .as_ref()
                    .map_or(0, EvalBindings::len),
            );
            if locals[group.request].is_none() {
                let prepared = PreparedBatch::new(
                    self.programs[group.request].clone(),
                    DerivativeOrder::Value,
                )?;
                let worker = prepared.worker_scoped(scope.clone());
                locals[group.request] = Some(LocalBatch { prepared, worker });
            }
        }
        Ok(CaseBatchWorker {
            assembly: self.clone(),
            locals,
            inputs: Vec::new(),
            row: vec![f64::NAN; max_inputs],
            ids: Vec::new(),
            indices: Vec::new(),
            scope,
        })
    }
}
impl CaseBatchWorker {
    /// Evaluate independent points using each original occurrence's admitted compact
    /// input gather and output contribution map. A failed point publishes no partial row.
    pub fn constraints(
        &mut self,
        points: &[CaseBatchPoint<'_>],
    ) -> Result<CaseBatchOutcome, MathError> {
        let count = points.len();
        let backing = self.assembly.batch_worker_bytes(count)?;
        if backing > self.assembly.limits.worker_bytes {
            return Err(MathError::Limit("case batch backing bytes"));
        }
        crate::construction::additional(self.assembly.batch_owned_backing(count)?)?;
        let rows = self.assembly.rows.len();
        let extent = count
            .checked_mul(rows)
            .ok_or(MathError::Limit("case batch result extent"))?;
        let mut outcome = CaseBatchOutcome {
            rows: vec![SemanticId::NIL; rows],
            points: Vec::new(),
            values: Vec::new(),
            canceled: false,
        };
        for (&id, &row) in self.assembly.rows.iter() {
            outcome.rows[row.get()] = id;
        }
        outcome
            .points
            .try_reserve_exact(count)
            .map_err(|_| MathError::Limit("case batch outcomes"))?;
        outcome
            .values
            .try_reserve_exact(extent)
            .map_err(|_| MathError::Limit("case batch cells"))?;
        outcome.values.resize(extent, 0.0);
        for point in points {
            let status = match self.assembly.structure.validate_values(point.values) {
                Ok(()) => PointOutcome::Succeeded,
                Err(cause) => PointOutcome::Failed(cause),
            };
            outcome.points.push(PointResult {
                id: point.id,
                outcome: status,
                applicability: Vec::new(),
            });
        }
        self.ids
            .try_reserve_exact(count)
            .map_err(|_| MathError::Limit("case batch identities"))?;
        self.indices
            .try_reserve_exact(count)
            .map_err(|_| MathError::Limit("case batch indices"))?;
        for (instance, (binding, physical)) in self
            .assembly
            .instances
            .iter()
            .zip(self.assembly.structure.instances())
            .enumerate()
        {
            let Some(group) = &binding.groups[EvaluationDemand::Constraints] else {
                continue;
            };
            let local = self.locals[group.request]
                .as_mut()
                .ok_or_else(|| MathError::Contract("case batch local worker missing".into()))?;
            let gather = self.assembly.bindings[instance][EvaluationDemand::Constraints]
                .as_ref()
                .ok_or_else(|| MathError::Contract("case batch input gather missing".into()))?;
            let context = |cause| MathError::Instance {
                instance: physical.instance,
                occurrences: physical.occurrences.as_ref().clone(),
                checked_sources: physical.checked_sources.as_ref().clone(),
                checked_members: physical.checked_members.as_ref().clone(),
                cause: Box::new(cause),
            };
            self.inputs.clear();
            self.ids.clear();
            self.indices.clear();
            let packed = count
                .checked_mul(gather.len())
                .ok_or(MathError::Limit("case batch packed extent"))?;
            self.inputs
                .try_reserve_exact(packed)
                .map_err(|_| MathError::Limit("case batch packed inputs"))?;
            for (index, point) in points.iter().enumerate() {
                if !matches!(outcome.points[index].outcome, PointOutcome::Succeeded) {
                    continue;
                }
                if self.scope.cancellation().load(Ordering::Acquire) {
                    outcome.canceled = true;
                    outcome.points[index].outcome = PointOutcome::NotExecuted;
                    continue;
                }
                let row = &mut self.row[..gather.len()];
                match EvalInputs::from(point.values).fill(gather, row) {
                    Ok(()) => {
                        self.inputs.extend_from_slice(row);
                        self.ids.push(point.id);
                        self.indices.push(index);
                    }
                    Err(cause) => {
                        outcome.points[index].outcome = PointOutcome::Failed(context(cause))
                    }
                }
            }
            let batch = local.worker.evaluate(
                BatchRequest {
                    layout: local.prepared.layout(),
                    points: &self.ids,
                    inputs: &self.inputs,
                    bindings: BatchBindings::SelectedPartials,
                },
                self.scope.cancellation(),
            )?;
            let (results, cells, canceled) = batch.into_parts();
            outcome.canceled |= canceled;
            let width = local.prepared.layout().cells();
            for (local_row, result) in results.into_iter().enumerate() {
                let index = self.indices[local_row];
                match result.outcome {
                    PointOutcome::Succeeded => {
                        for term in &binding.constraint_terms {
                            let destination = index * rows + term.destination;
                            let value = outcome.values[destination]
                                + term.scale * cells[local_row * width + term.result_row];
                            if let Err(cause) = finite(value) {
                                outcome.points[index].outcome =
                                    PointOutcome::Failed(context(cause));
                                break;
                            }
                            outcome.values[destination] = value;
                        }
                        for mut evidence in result.applicability {
                            evidence.instance = Some(physical.instance);
                            if !outcome.points[index].applicability.contains(&evidence) {
                                outcome.points[index].applicability.push(evidence);
                            }
                        }
                    }
                    PointOutcome::Failed(cause) => {
                        outcome.points[index].outcome = PointOutcome::Failed(context(cause))
                    }
                    PointOutcome::Canceled => {
                        outcome.points[index].outcome = PointOutcome::Canceled
                    }
                    PointOutcome::NotExecuted => {
                        outcome.points[index].outcome = PointOutcome::NotExecuted
                    }
                }
            }
        }
        outcome.canceled |= self.scope.cancellation().load(Ordering::Acquire);
        self.inputs.fill(f64::NAN);
        self.row.fill(f64::NAN);
        CaseBatchOutcome::from_parts(
            outcome.rows,
            outcome.points,
            outcome.values,
            outcome.canceled,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding::*,
        construction::ConstructionAdmission,
        typed::{Binary, BodyBuilder, BodyLimits},
    };
    use pse_kernels::Port;
    use pse_model::generated::enums::ModelingVariableDomain;
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, ids, standard_registry},
    };
    use std::sync::atomic::AtomicUsize;
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    #[derive(Debug)]
    struct Prepaid {
        used: AtomicUsize,
        limit: usize,
    }
    impl ConstructionAdmission for Prepaid {
        fn try_grow(&self, bytes: usize) -> Result<(), MathError> {
            let previous = self.used.fetch_add(bytes, Ordering::AcqRel);
            if previous.checked_add(bytes).is_none_or(|n| n > self.limit) {
                return Err(MathError::Limit("test batch backing"));
            }
            Ok(())
        }
    }
    #[test]
    fn case_batch_preserves_multiinstance_transport_full_value_admission_and_backing() {
        let registry = standard_registry().unwrap();
        let q = ids::quantity("neutral");
        let port = |n| Port {
            id: id(n),
            quantity: q,
            unit: registry.quantity_type(q).unwrap().canonical_unit,
        };
        let mut b = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap();
        let x = b.input(0, q, IndexSet::new(), id(20)).unwrap();
        let square = b.binary(Binary::Mul, x.clone(), x, None, id(21)).unwrap();
        let body = Arc::new(b.prepare(&[square]).unwrap());
        let key = ContentHash::from_bytes([19; 32]);
        let instance = |n, source, scale| InstanceBinding {
            instance: id(n),
            body: key,
            checked_members: Default::default(),
            checked_sources: Default::default(),
            occurrences: Default::default(),
            slots: vec![SlotBinding::new(&port(source), &port(1), &registry).unwrap()].into(),
            contributions: vec![Contribution {
                output: 0,
                target: Target::Row(id(10)),
                scale,
            }],
        };
        let structure = Arc::new(
            CaseStructure::new(
                vec![
                    Variable {
                        port: port(1),
                        fixed: false,
                        domain: ModelingVariableDomain::Continuous,
                        lower: Some(0.),
                        upper: Some(5.),
                    },
                    Variable {
                        port: port(2),
                        fixed: true,
                        domain: ModelingVariableDomain::Integer,
                        lower: Some(0.),
                        upper: Some(10.),
                    },
                ],
                vec![],
                vec![instance(30, 1, 1.), instance(31, 2, 2.)],
                vec![Row {
                    id: id(10),
                    quantity: q,
                    lower: 0.,
                    upper: 100.,
                }],
                None,
                CaseLimits::default(),
            )
            .unwrap(),
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let scope =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None);
        let plan = Arc::new(
            CasePlan::prepare(
                structure,
                BTreeMap::from([(key, body)]),
                &registry,
                DerivativeOrder::Value,
                AssemblyLimits::default(),
                &cancel,
            )
            .unwrap(),
        );
        let assembly = Arc::new(
            plan.compile(Optimization::default(), EvaluationLimits::default(), &scope)
                .unwrap(),
        );
        let values = |a, b| CaseValues {
            scalars: BTreeMap::from([(id(1), a), (id(2), b)]),
        };
        let valid = values(2., 3.);
        let outside = values(-1., 3.);
        let fractional = values(2., 3.5);
        let mut unknown = valid.clone();
        unknown.scalars.insert(id(99), 1.);
        let mut missing = valid.clone();
        missing.scalars.remove(&id(2));
        let population = [&valid, &outside, &fractional, &unknown, &missing, &valid];
        let points = population
            .iter()
            .map(|values| CaseBatchPoint { id: id(1), values })
            .collect::<Vec<_>>();
        let bound = assembly.batch_worker_bytes(points.len()).unwrap();
        let owner = Arc::new(Prepaid {
            used: AtomicUsize::new(0),
            limit: bound,
        });
        let result = crate::construction::scoped(owner.clone(), || {
            let mut worker = assembly.batch_worker(scope).unwrap();
            worker.constraints(&points).unwrap()
        });
        assert!(owner.used.load(Ordering::Acquire) <= bound);
        assert_eq!(result.row(0), Some([22.].as_slice()));
        assert_eq!(result.row(5), Some([22.].as_slice()));
        for row in 1..5 {
            assert!(matches!(
                result.points()[row].outcome,
                PointOutcome::Failed(MathError::Contract(_))
            ));
            assert!(result.row(row).is_none());
        }
    }
}
