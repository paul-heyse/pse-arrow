// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordered independent points over one immutable product and private reusable scratch.
use super::*;
#[path = "batch/checked.rs"]
mod checked;
#[cfg(feature = "compiled-evaluator")]
#[path = "batch/packed.rs"]
mod packed;
pub use checked::CheckedBatchExecution;
pub(crate) use checked::numerical_batch_backing;

/// Exact compiled program and output/coordinate/product qualification.
#[derive(Clone)]
pub struct BatchLayout {
    body: Arc<CompiledBody>,
    order: DerivativeOrder,
    cells: usize,
}
impl std::fmt::Debug for BatchLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchLayout")
            .field("product", &self.product())
            .field("order", &self.order)
            .field("outputs", &self.outputs())
            .field("coordinates", &self.coordinates())
            .field("pairs", &self.pairs())
            .field("cells", &self.cells)
            .finish()
    }
}
impl PartialEq for BatchLayout {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.body.template, &other.body.template)
            && self.order == other.order
            && self.cells == other.cells
    }
}
impl Eq for BatchLayout {}
impl BatchLayout {
    /// Product selected before execution; runtime bindings cannot change it.
    pub fn product(&self) -> ProductKind {
        self.body.product_kind()
    }
    /// Requested partial order, or the selected action's required order.
    pub fn order(&self) -> DerivativeOrder {
        self.order
    }
    /// Original selected output identities, in numerical row order.
    pub fn outputs(&self) -> &[usize] {
        self.body.support.outputs()
    }
    /// Original derivative coordinates, in explicitly selected axis order.
    pub fn coordinates(&self) -> &[usize] {
        self.body.coordinates()
    }
    /// Symmetric pairs of selected-axis ordinals, in numerical component order.
    pub fn pairs(&self) -> &[(usize, usize)] {
        if self.product() == ProductKind::SelectedPartials {
            self.body.template.layouts[self.order]
                .as_ref()
                .map_or(&[], |layout| layout.pairs.as_slice())
        } else {
            self.body.product_pairs()
        }
    }
    /// Checked owned result/witness extent, admitted before allocation by the host.
    pub fn backing_bytes(&self, points: usize) -> Result<usize, MathError> {
        let diagnostics = self.body.template.programs[self.order]
            .as_ref()
            .map_or(Ok(0), |stages| checked::diagnostic_backing(stages))?;
        let per_point = self
            .cells
            .checked_mul(size_of::<f64>())
            .and_then(|n| n.checked_add(size_of::<PointResult>()))
            .and_then(|n| n.checked_add(self.body.template.evaluation_cache_bytes))
            .and_then(|n| n.checked_add(diagnostics))
            .ok_or(MathError::Limit("batch backing extent"))?;
        points
            .checked_mul(per_point)
            .and_then(|n| n.checked_add(size_of::<BatchOutcome>()))
            .ok_or(MathError::Limit("batch backing extent"))
    }
    /// Number of admitted inputs in each point-major row.
    pub fn inputs(&self) -> usize {
        self.body.template.inputs
    }
    /// Numerical cells in each successful row. Selected partials are output-major:
    /// value, then ordered first axes, then symmetric second pairs for each output.
    pub fn cells(&self) -> usize {
        self.cells
    }
}

/// Prepared scalar realization of exactly one qualified immutable numerical product.
#[derive(Clone, Debug)]
pub struct PreparedBatch {
    layout: BatchLayout,
}
impl PreparedBatch {
    /// Bind a completed product without rebuilding its mathematics. The program's
    /// existing interpreted/scalar-native selection remains authoritative.
    pub fn new(body: Arc<CompiledBody>, order: DerivativeOrder) -> Result<Self, MathError> {
        if !body.required_provider_keys().is_empty() {
            return Err(MathError::Contract(
                "pure batch cannot execute provider effects".into(),
            ));
        }
        let cells = if body.product_kind() == ProductKind::SelectedPartials {
            let layout = body.template.layouts[order].as_ref().ok_or_else(|| {
                MathError::Contract("batch partial order was not compiled".into())
            })?;
            let width = 1usize
                .checked_add(if order >= DerivativeOrder::First {
                    body.coordinates().len()
                } else {
                    0
                })
                .and_then(|n| {
                    n.checked_add(if order >= DerivativeOrder::Second {
                        layout.pairs.len()
                    } else {
                        0
                    })
                })
                .ok_or(MathError::Limit("batch cell layout"))?;
            body.template
                .outputs
                .len()
                .checked_mul(width)
                .ok_or(MathError::Limit("batch cell layout"))?
        } else {
            if order != body.product_kind().required_order(body.support.order()) {
                return Err(MathError::Contract(
                    "batch action order differs from prepared product".into(),
                ));
            }
            body.product_len()
        };
        Ok(Self {
            layout: BatchLayout { body, order, cells },
        })
    }
    /// Sealed output and runtime binding qualification for subsequent calls.
    pub fn layout(&self) -> &BatchLayout {
        &self.layout
    }
    /// Each worker owns independent evaluator stacks, trial witnesses and row scratch.
    pub fn worker(&self) -> BatchWorker {
        BatchWorker {
            worker: self.layout.body.worker(),
            layout: self.layout.clone(),
            trial: TrialEvaluation::default(),
            row: vec![f64::NAN; self.layout.cells],
            #[cfg(feature = "compiled-evaluator")]
            packed: packed::Scratch::default(),
        }
    }
    /// Preserve the original enclosing deadline/cancellation owner in each private evaluator.
    pub fn worker_scoped(&self, scope: pse_kernels::ExecutionScope) -> BatchWorker {
        BatchWorker {
            worker: self.layout.body.worker_scoped(scope),
            layout: self.layout.clone(),
            trial: TrialEvaluation::default(),
            row: vec![f64::NAN; self.layout.cells],
            #[cfg(feature = "compiled-evaluator")]
            packed: packed::Scratch::default(),
        }
    }
    /// Complete backing of a reusable selected-partials population worker.
    /// Admission precedes materialization; the returned rows remain callback borrows.
    pub fn reusable_backing_bytes(&self, max_rows: usize) -> Result<usize, MathError> {
        if self.layout.product() != ProductKind::SelectedPartials {
            return Err(MathError::Contract(
                "reusable batch requires selected partials".into(),
            ));
        }
        numerical_batch_backing(&self.layout.body.template, max_rows, self.layout.order)
    }
    /// Reuse this exact prepared mathematical product with current scoped scratch.
    /// The enclosing owner retains the complete allowance for the worker's lifetime.
    pub fn reusable_worker_scoped(
        &self,
        scope: pse_kernels::ExecutionScope,
        max_rows: usize,
        backing_bytes: usize,
    ) -> Result<CheckedBatchExecution, MathError> {
        if self.reusable_backing_bytes(max_rows)? > backing_bytes {
            return Err(MathError::Limit("prepaid numerical batch backing"));
        }
        CheckedBatchExecution::from_frame(
            self.layout.body.numeric_frame_scoped(scope),
            max_rows,
            self.layout.order,
            backing_bytes,
        )
    }
}

/// Point-major runtime coefficients, each with the layout's exact stride.
#[derive(Clone, Copy, Debug)]
pub enum BatchBindings<'a> {
    /// Selected values/partials consume no numerical seeds.
    SelectedPartials,
    /// One coordinate direction per point.
    JacobianAction {
        /// Seeds in point-major selected-coordinate order.
        directions: &'a [f64],
    },
    /// One selected-output cotangent per point.
    TransposeJacobianAction {
        /// Cotangents in point-major selected-output order.
        weights: &'a [f64],
    },
    /// One selected-output weight per point.
    WeightedHessian {
        /// Weights in point-major selected-output order.
        weights: &'a [f64],
    },
    /// Independent selected-output weights and coordinate directions per point.
    WeightedHessianAction {
        /// Weights in point-major selected-output order.
        weights: &'a [f64],
        /// Seeds in point-major selected-coordinate order.
        directions: &'a [f64],
    },
}
impl BatchBindings<'_> {
    fn kind(self) -> ProductKind {
        match self {
            Self::SelectedPartials => ProductKind::SelectedPartials,
            Self::JacobianAction { .. } => ProductKind::JacobianAction,
            Self::TransposeJacobianAction { .. } => ProductKind::TransposeJacobianAction,
            Self::WeightedHessian { .. } => ProductKind::WeightedHessian,
            Self::WeightedHessianAction { .. } => ProductKind::WeightedHessianAction,
        }
    }
}
/// Borrowed independent-point request. Point identity determines population even
/// when a constant program has no inputs; repeated identities remain separate rows.
#[derive(Clone, Copy, Debug)]
pub struct BatchRequest<'a> {
    /// Exact qualification issued by the prepared batch.
    pub layout: &'a BatchLayout,
    /// Original point identities in result order.
    pub points: &'a [SemanticId],
    /// `inputs[row * layout.inputs() + input]`.
    pub inputs: &'a [f64],
    /// Point-major seeds/weights with unchanged semantic shape.
    pub bindings: BatchBindings<'a>,
}
/// One point's truthful terminal state; original typed/source failures are retained.
#[derive(Debug)]
pub enum PointOutcome {
    /// All requested cells completed with finite numerical values.
    Succeeded,
    /// Original physical/domain/derivative/numerical refusal.
    Failed(MathError),
    /// Cancellation interrupted this point without publishing its row.
    Canceled,
    /// The operation canceled before starting this point.
    NotExecuted,
}
/// Original identity and its outcome, preserving repeated identities independently.
#[derive(Debug)]
pub struct PointResult {
    /// Caller-issued point identity.
    pub id: SemanticId,
    /// Actual outcome of this original row.
    pub outcome: PointOutcome,
    /// Original active-path scientific applicability for a successful point.
    pub applicability: Vec<pse_model::applicability::Observation>,
}
/// Ordered operation result. Failed-row storage is private and cannot be read as cells.
#[derive(Debug)]
pub struct BatchOutcome {
    layout: BatchLayout,
    points: Vec<PointResult>,
    cells: Vec<f64>,
    canceled: bool,
}
impl BatchOutcome {
    pub(crate) fn into_parts(self) -> (Vec<PointResult>, Vec<f64>, bool) {
        (self.points, self.cells, self.canceled)
    }
    /// Original ordered point population and truthful individual outcomes.
    pub fn points(&self) -> &[PointResult] {
        &self.points
    }
    /// Whether cancellation prevented a complete operation; point failures remain individual.
    pub fn canceled(&self) -> bool {
        self.canceled
    }
    /// Only a successful row exposes numerical cells; failed rows never expose stale scratch.
    pub fn row(&self, index: usize) -> Option<&[f64]> {
        if !matches!(self.points.get(index)?.outcome, PointOutcome::Succeeded) {
            return None;
        }
        self.cells
            .get(index * self.layout.cells..(index + 1) * self.layout.cells)
    }
    /// The complete point-major numerical population is available only if every row succeeded.
    pub fn point_major(&self) -> Option<&[f64]> {
        (!self.canceled
            && self
                .points
                .iter()
                .all(|p| matches!(p.outcome, PointOutcome::Succeeded)))
        .then_some(self.cells.as_slice())
    }
    /// Exact row/cell interpretation retained with the results.
    pub fn layout(&self) -> &BatchLayout {
        &self.layout
    }
}
/// Private reusable mutable evaluator/trial/row state. Workers never share this storage.
#[derive(Debug)]
pub struct BatchWorker {
    worker: Worker,
    layout: BatchLayout,
    trial: TrialEvaluation,
    row: Vec<f64>,
    #[cfg(feature = "compiled-evaluator")]
    packed: packed::Scratch,
}
impl BatchWorker {
    /// Evaluate each independent row in order. Request shape errors fail before any
    /// point executes; point-specific failures coexist with independent successes.
    pub fn evaluate(
        &mut self,
        request: BatchRequest<'_>,
        cancel: &Arc<AtomicBool>,
    ) -> Result<BatchOutcome, MathError> {
        self.evaluate_observing(request, cancel, |_| {})
    }
    fn evaluate_observing(
        &mut self,
        request: BatchRequest<'_>,
        cancel: &Arc<AtomicBool>,
        mut completed: impl FnMut(usize),
    ) -> Result<BatchOutcome, MathError> {
        let count = request.points.len();
        if request.layout != &self.layout || request.bindings.kind() != self.layout.product() {
            return Err(MathError::Contract(
                "batch program/layout/product qualification differs".into(),
            ));
        }
        let extent = |stride: usize| {
            count
                .checked_mul(stride)
                .ok_or(MathError::Limit("batch population extent"))
        };
        if request.inputs.len() != extent(self.layout.inputs())? {
            return Err(MathError::Contract("batch point-major input extent".into()));
        }
        let (weights, directions) = match request.bindings {
            BatchBindings::SelectedPartials => (None, None),
            BatchBindings::JacobianAction { directions } => (None, Some(directions)),
            BatchBindings::TransposeJacobianAction { weights }
            | BatchBindings::WeightedHessian { weights } => (Some(weights), None),
            BatchBindings::WeightedHessianAction {
                weights,
                directions,
            } => (Some(weights), Some(directions)),
        };
        let weight_extent = extent(self.layout.outputs().len())?;
        let direction_extent = extent(self.layout.coordinates().len())?;
        if weights.is_some_and(|v| v.len() != weight_extent)
            || directions.is_some_and(|v| v.len() != direction_extent)
        {
            return Err(MathError::Contract(
                "batch point-major seed/weight extent".into(),
            ));
        }
        let backing = self.layout.backing_bytes(count)?;
        if backing > self.layout.body.template.limits.scratch_bytes {
            return Err(MathError::Limit("batch backing bytes"));
        }
        crate::construction::additional(backing)?;
        // Checked length products precede allocation, including zero-input populations.
        let cells = extent(self.layout.cells)?;
        let mut output = BatchOutcome {
            layout: self.layout.clone(),
            points: Vec::new(),
            cells: Vec::new(),
            canceled: false,
        };
        output
            .points
            .try_reserve_exact(count)
            .map_err(|_| MathError::Limit("batch point outcomes"))?;
        output
            .cells
            .try_reserve_exact(cells)
            .map_err(|_| MathError::Limit("batch result cells"))?;
        output.cells.resize(cells, f64::NAN);
        #[cfg(feature = "compiled-evaluator")]
        if self.uses_packed() {
            return self.evaluate_packed(request, cancel, output, completed);
        }
        let mut providers = ProviderBindings::new();
        for (row, &id) in request.points.iter().enumerate() {
            self.row.fill(f64::NAN);
            self.trial.clear();
            if cancel.load(Ordering::Acquire) || output.canceled {
                output.canceled = true;
                output.points.push(PointResult {
                    id,
                    outcome: PointOutcome::NotExecuted,
                    applicability: Vec::new(),
                });
                continue;
            }
            let inputs =
                &request.inputs[row * self.layout.inputs()..(row + 1) * self.layout.inputs()];
            let w = weights.map(|v| {
                &v[row * self.layout.outputs().len()..(row + 1) * self.layout.outputs().len()]
            });
            let d = directions.map(|v| {
                &v[row * self.layout.coordinates().len()
                    ..(row + 1) * self.layout.coordinates().len()]
            });
            let bindings = match request.bindings {
                BatchBindings::SelectedPartials => ProductValues::SelectedPartials,
                BatchBindings::JacobianAction { .. } => ProductValues::JacobianAction {
                    direction: d.unwrap_or(&[]),
                },
                BatchBindings::TransposeJacobianAction { .. } => {
                    ProductValues::TransposeJacobianAction {
                        weights: w.unwrap_or(&[]),
                    }
                }
                BatchBindings::WeightedHessian { .. } => ProductValues::WeightedHessian {
                    weights: w.unwrap_or(&[]),
                },
                BatchBindings::WeightedHessianAction { .. } => {
                    ProductValues::WeightedHessianAction {
                        weights: w.unwrap_or(&[]),
                        direction: d.unwrap_or(&[]),
                    }
                }
            };
            let result = if self.layout.product() == ProductKind::SelectedPartials {
                self.worker
                    .evaluate_trial_into(
                        inputs,
                        self.layout.order,
                        &mut providers,
                        cancel,
                        &mut self.trial,
                    )
                    .and_then(|()| {
                        copy_trial(&self.trial, &self.layout, &mut self.row)?;
                        Ok(())
                    })
            } else {
                self.worker.evaluate_product_trial_into(
                    inputs,
                    bindings,
                    &mut providers,
                    cancel,
                    &mut self.row,
                    &mut self.trial,
                )
            };
            let (outcome, applicability) = match result {
                Ok(()) if cancel.load(Ordering::Acquire) => {
                    output.canceled = true;
                    (PointOutcome::Canceled, Vec::new())
                }
                Ok(()) => {
                    output.cells[row * self.layout.cells..(row + 1) * self.layout.cells]
                        .copy_from_slice(&self.row);
                    (
                        PointOutcome::Succeeded,
                        self.trial
                            .applicability
                            .iter()
                            .flat_map(ApplicabilityWitness::hydrate)
                            .collect(),
                    )
                }
                Err(MathError::Cancelled) => {
                    output.canceled = true;
                    (PointOutcome::Canceled, Vec::new())
                }
                Err(error) => (PointOutcome::Failed(error), Vec::new()),
            };
            output.points.push(PointResult {
                id,
                outcome,
                applicability,
            });
            completed(row);
        }
        output.canceled |= cancel.load(Ordering::Acquire);
        self.row.fill(f64::NAN);
        self.trial.clear();
        Ok(output)
    }
}

fn copy_trial(
    trial: &TrialEvaluation,
    layout: &BatchLayout,
    destination: &mut [f64],
) -> Result<(), MathError> {
    let mut cell = 0;
    for (row, &value) in trial.values.iter().enumerate() {
        destination[cell] = value;
        cell += 1;
        if layout.order >= DerivativeOrder::First {
            for axis in 0..layout.coordinates().len() {
                destination[cell] = trial.first(row, axis);
                cell += 1;
            }
        }
        if layout.order >= DerivativeOrder::Second {
            for &(i, j) in layout.pairs() {
                destination[cell] = trial.second(row, i, j);
                cell += 1;
            }
        }
    }
    if cell != layout.cells {
        return Err(MathError::Contract("batch numerical cell layout".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> SemanticId {
        SemanticId::from_bytes([17; 16])
    }
    fn scope() -> pse_kernels::ExecutionScope {
        pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(false), None)
    }
    fn body(guard: bool) -> PreparedBody {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let mut stages = vec![];
        if guard {
            stages.push(Stage::Require {
                argument: 0,
                condition: Condition::Positive,
                order: DerivativeOrder::Value,
                source: source(),
                lineage: None,
            });
        }
        stages.push(Stage::Block {
            expressions: vec![x.pow(2) + &y * Atom::num(3), &x * &y],
            outputs: vec![2, 3],
            source: source(),
        });
        PreparedBody::new(2, 4, vec![2, 3], stages, DerivativeOrder::Second).unwrap()
    }
    #[test]
    fn ordered_reusable_source_workers_admit_once_and_reuse_numeric_rows() {
        #[derive(Debug, Default)]
        struct Recorder(std::sync::atomic::AtomicUsize);
        impl crate::construction::ConstructionAdmission for Recorder {
            fn try_grow(&self, bytes: usize) -> Result<(), MathError> {
                self.0.fetch_add(bytes, Ordering::Relaxed);
                Ok(())
            }
        }
        let profiles = [
            library::Evaluator::Interpreted,
            #[cfg(feature = "compiled-evaluator")]
            library::Evaluator::CompiledX4 {
                module_bytes: 16 << 20,
            },
        ];
        for profile in profiles {
            let compiled = Arc::new(
                body(true)
                    .compile(
                        &[1, 0],
                        &[1, 0],
                        DerivativeOrder::Second,
                        Optimization {
                            evaluator: profile,
                            ..Default::default()
                        },
                        EvaluationLimits::default(),
                        &scope(),
                    )
                    .unwrap(),
            );
            let prepared = PreparedBatch::new(compiled, DerivativeOrder::Second).unwrap();
            let backing = prepared.reusable_backing_bytes(6).unwrap();
            assert!(matches!(
                prepared.reusable_worker_scoped(scope(), 6, backing - 1),
                Err(MathError::Limit("prepaid numerical batch backing"))
            ));
            let mut worker = prepared
                .reusable_worker_scoped(scope(), 6, backing)
                .unwrap();
            let recorder = Arc::new(Recorder::default());
            let mut pointers = None;
            for rows in [6, 3, 0, 6] {
                let inputs: Vec<_> = (0..rows)
                    .flat_map(|row| [if row == 2 { -1. } else { 2. + row as f64 }, 5.])
                    .collect();
                let mut observed = Vec::new();
                crate::construction::scoped(recorder.clone(), || {
                    worker.evaluate_observing(rows, &inputs, |row, result| {
                        observed.push(row);
                        if row == 2 {
                            assert!(matches!(result, Err(MathError::Domain { .. })));
                            return;
                        }
                        let value = result.unwrap();
                        let x = 2. + row as f64;
                        assert_eq!(value.values, [5. * x, x * x + 15.]);
                        assert_eq!(value.jacobian, [x, 5., 3., 2. * x]);
                        assert_eq!(value.hessians, [0., 1., 1., 0., 0., 0., 0., 2.]);
                        let current = (
                            value.values.as_ptr(),
                            value.jacobian.as_ptr(),
                            value.hessians.as_ptr(),
                        );
                        if let Some(previous) = pointers {
                            assert_eq!(previous, current);
                        } else {
                            pointers = Some(current);
                        }
                    })
                })
                .unwrap();
                assert_eq!(observed, (0..rows).collect::<Vec<_>>());
                assert_eq!(
                    recorder.0.load(Ordering::Relaxed),
                    0,
                    "complete worker backing is admitted once before trials"
                );
            }
        }
    }
    #[cfg(feature = "compiled-evaluator")]
    #[test]
    fn ordered_x4_batch_preserves_failed_rows_second_axes_and_segment_cancellation() {
        let body = body(true);
        let compiled = Arc::new(
            body.compile(
                &[1, 0],
                &[1, 0],
                DerivativeOrder::Second,
                Optimization {
                    evaluator: library::Evaluator::CompiledX4 {
                        module_bytes: 16 << 20,
                    },
                    ..Default::default()
                },
                EvaluationLimits::default(),
                &scope(),
            )
            .unwrap(),
        );
        let prepared = PreparedBatch::new(compiled, DerivativeOrder::Second).unwrap();
        let mut worker = prepared.worker();
        for count in [0, 1, 3, 4, 5, 260] {
            let ids = vec![source(); count];
            let inputs: Vec<_> = (0..count)
                .flat_map(|row| [if row == 2 { -1. } else { 2. + row as f64 }, 5.])
                .collect();
            let result = worker
                .evaluate(
                    BatchRequest {
                        layout: prepared.layout(),
                        points: &ids,
                        inputs: &inputs,
                        bindings: BatchBindings::SelectedPartials,
                    },
                    &Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
            for row in 0..count {
                if row == 2 {
                    assert!(matches!(
                        result.points()[row].outcome,
                        PointOutcome::Failed(MathError::Domain { .. })
                    ));
                    assert!(result.row(row).is_none());
                    continue;
                }
                let x = 2. + row as f64;
                let mut expected = Vec::new();
                for &output in prepared.layout().outputs() {
                    expected.push(if output == 1 { x * 5. } else { x * x + 15. });
                    for &axis in prepared.layout().coordinates() {
                        expected.push(if output == 1 {
                            if axis == 0 { 5. } else { x }
                        } else if axis == 0 {
                            2. * x
                        } else {
                            3.
                        });
                    }
                    for &(i, j) in prepared.layout().pairs() {
                        let (i, j) = (
                            prepared.layout().coordinates()[i],
                            prepared.layout().coordinates()[j],
                        );
                        expected.push(if output == 1 && i != j {
                            1.
                        } else if output == 0 && i == 0 && j == 0 {
                            2.
                        } else {
                            0.
                        });
                    }
                }
                assert_eq!(
                    result.row(row).unwrap(),
                    expected,
                    "original row {row} of {count}"
                );
            }
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let ids = vec![source(); 260];
        let inputs = vec![2.; 520];
        let result = worker
            .evaluate_observing(
                BatchRequest {
                    layout: prepared.layout(),
                    points: &ids,
                    inputs: &inputs,
                    bindings: BatchBindings::SelectedPartials,
                },
                &cancel,
                |row| {
                    if row == 255 {
                        cancel.store(true, Ordering::Release);
                    }
                },
            )
            .unwrap();
        assert!(result.canceled());
        assert!(
            result.points()[..256]
                .iter()
                .all(|row| matches!(row.outcome, PointOutcome::Succeeded))
        );
        assert!(
            result.points()[256..]
                .iter()
                .all(|row| matches!(row.outcome, PointOutcome::NotExecuted))
        );
    }
    #[test]
    fn ordered_batch_preserves_reordered_partial_cells_and_independent_workers() {
        let body = body(false);
        let compiled = Arc::new(
            body.compile(
                &[1, 0],
                &[1, 0],
                DerivativeOrder::Second,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope(),
            )
            .unwrap(),
        );
        let batch = PreparedBatch::new(compiled, DerivativeOrder::Second).unwrap();
        let ids = [source(), source(), SemanticId::NIL];
        let inputs = [2., 5., 2., 5., 3., 7.];
        let cancel = Arc::new(AtomicBool::new(false));
        let request = BatchRequest {
            layout: batch.layout(),
            points: &ids,
            inputs: &inputs,
            bindings: BatchBindings::SelectedPartials,
        };
        let mut first = batch.worker();
        let mut second = batch.worker();
        let result = first.evaluate(request, &cancel).unwrap();
        let other = second.evaluate(request, &cancel).unwrap();
        assert_eq!(result.point_major(), other.point_major());
        assert_eq!(result.points.iter().map(|p| p.id).collect::<Vec<_>>(), ids);
        for (row, [x, y]) in [[2., 5.], [2., 5.], [3., 7.]].into_iter().enumerate() {
            let mut expected = vec![];
            for &output in batch.layout().outputs() {
                expected.push(if output == 1 { x * y } else { x * x + 3. * y });
                for &axis in batch.layout().coordinates() {
                    expected.push(if output == 1 {
                        if axis == 0 { y } else { x }
                    } else if axis == 0 {
                        2. * x
                    } else {
                        3.
                    });
                }
                for &(i, j) in batch.layout().pairs() {
                    let (i, j) = (
                        batch.layout().coordinates()[i],
                        batch.layout().coordinates()[j],
                    );
                    expected.push(if output == 1 && i != j {
                        1.
                    } else if output == 0 && i == 0 && j == 0 {
                        2.
                    } else {
                        0.
                    });
                }
            }
            assert_eq!(result.row(row).unwrap(), expected);
        }
        let incompatible = PreparedBatch::new(
            Arc::new(
                body.compile(
                    &[1, 0],
                    &[0, 1],
                    DerivativeOrder::Second,
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &scope(),
                )
                .unwrap(),
            ),
            DerivativeOrder::Second,
        )
        .unwrap();
        assert!(
            first
                .evaluate(
                    BatchRequest {
                        layout: incompatible.layout(),
                        ..request
                    },
                    &cancel
                )
                .is_err()
        );
    }
    #[test]
    fn ordered_batch_keeps_failed_rows_private_and_empty_constant_populations() {
        let body = body(true);
        let compiled = Arc::new(
            body.compile(
                &[0],
                &[],
                DerivativeOrder::Value,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope(),
            )
            .unwrap(),
        );
        let batch = PreparedBatch::new(compiled, DerivativeOrder::Value).unwrap();
        let mut worker = batch.worker();
        let cancel = Arc::new(AtomicBool::new(false));
        let ids = [source(); 3];
        let result = worker
            .evaluate(
                BatchRequest {
                    layout: batch.layout(),
                    points: &ids,
                    inputs: &[2., 1., -1., 1., 3., 1.],
                    bindings: BatchBindings::SelectedPartials,
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(result.row(0), Some([7.].as_slice()));
        assert!(
            matches!(result.points()[1].outcome,PointOutcome::Failed(MathError::Domain { source_id, .. }) if source_id == source())
        );
        assert_eq!(result.row(1), None);
        assert_eq!(result.row(2), Some([12.].as_slice()));
        assert!(result.point_major().is_none());
        let constant = PreparedBody::new(
            0,
            1,
            vec![0],
            vec![Stage::Block {
                expressions: vec![Atom::num(7)],
                outputs: vec![0],
                source: source(),
            }],
            DerivativeOrder::Value,
        )
        .unwrap();
        let constant = Arc::new(
            constant
                .compile(
                    &[0],
                    &[],
                    DerivativeOrder::Value,
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &scope(),
                )
                .unwrap(),
        );
        let constant = PreparedBatch::new(constant, DerivativeOrder::Value).unwrap();
        let mut worker = constant.worker();
        let request = BatchRequest {
            layout: constant.layout(),
            points: &ids,
            inputs: &[],
            bindings: BatchBindings::SelectedPartials,
        };
        assert_eq!(
            worker.evaluate(request, &cancel).unwrap().point_major(),
            Some([7., 7., 7.].as_slice())
        );
        assert_eq!(
            worker
                .evaluate(
                    BatchRequest {
                        points: &[],
                        ..request
                    },
                    &cancel
                )
                .unwrap()
                .point_major(),
            Some([].as_slice())
        );
        cancel.store(true, Ordering::Release);
        let result = worker.evaluate(request, &cancel).unwrap();
        assert!(result.canceled());
        assert!(
            result
                .points()
                .iter()
                .all(|p| matches!(p.outcome, PointOutcome::NotExecuted))
        );
        assert!(result.point_major().is_none());
    }
    #[test]
    fn ordered_batch_cancellation_between_points_keeps_original_success_and_stops_remainder() {
        let body = body(false);
        let compiled = Arc::new(
            body.compile(
                &[0],
                &[],
                DerivativeOrder::Value,
                Optimization::default(),
                EvaluationLimits::default(),
                &scope(),
            )
            .unwrap(),
        );
        let batch = PreparedBatch::new(compiled, DerivativeOrder::Value).unwrap();
        let mut worker = batch.worker();
        let ids = [source(); 3];
        let cancel = Arc::new(AtomicBool::new(false));
        let result = worker
            .evaluate_observing(
                BatchRequest {
                    layout: batch.layout(),
                    points: &ids,
                    inputs: &[2., 1., 3., 1., 4., 1.],
                    bindings: BatchBindings::SelectedPartials,
                },
                &cancel,
                |row| {
                    assert_eq!(row, 0);
                    cancel.store(true, Ordering::Release);
                },
            )
            .unwrap();
        assert_eq!(result.row(0), Some([7.].as_slice()));
        assert!(matches!(
            result.points()[1].outcome,
            PointOutcome::NotExecuted
        ));
        assert!(matches!(
            result.points()[2].outcome,
            PointOutcome::NotExecuted
        ));
        assert!(result.canceled());
        assert!(result.point_major().is_none());
    }
    #[test]
    fn ordered_batch_direct_weighted_action_avoids_unrequested_second_overflow() {
        use crate::{
            Function,
            typed::{Binary, BodyBuilder, BodyLimits},
        };
        use pse_quantity::{
            IndexSet,
            standard::{StandardInvariantChecker, ids, standard_registry},
        };
        let registry = standard_registry().unwrap();
        let mut b = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            1,
            BodyLimits::default(),
        )
        .unwrap();
        let x = b
            .input(0, ids::quantity("neutral"), IndexSet::new(), source())
            .unwrap();
        let u = b.unary(Function::Sqrt, x.clone(), source()).unwrap();
        let u = b.bind(u).unwrap();
        let square = b.binary(Binary::Mul, u.clone(), u, None, source()).unwrap();
        let quadratic = b.binary(Binary::Mul, x.clone(), x, None, source()).unwrap();
        let output = b
            .binary(Binary::Add, square, quadratic, None, source())
            .unwrap();
        let body = b.prepare(&[output]).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let support = body
            .prepare_support(&[0], &[0], DerivativeOrder::Second, &cancel)
            .unwrap();
        let compiled = Arc::new(
            support
                .compile_product(
                    ProductKind::WeightedHessianAction,
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &scope(),
                )
                .unwrap(),
        );
        let batch = PreparedBatch::new(compiled, DerivativeOrder::Second).unwrap();
        let mut worker = batch.worker();
        // The local second derivative of sqrt overflows, but the demanded
        // curvature of u²+x² is exactly 2. No full intermediate Second is requested.
        let result = worker
            .evaluate(
                BatchRequest {
                    layout: batch.layout(),
                    points: &[source(); 2],
                    inputs: &[1e-300, 1e-300],
                    bindings: BatchBindings::WeightedHessianAction {
                        weights: &[1., 2.],
                        directions: &[1., -3.],
                    },
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(
            result.point_major(),
            Some([2., -12.].as_slice()),
            "{:?}",
            result.points()
        );
    }
    #[test]
    fn ordered_batch_binds_runtime_action_seeds_without_shape_preparation() {
        let body = body(false);
        let support = body
            .prepare_support(
                &[1, 0],
                &[1, 0],
                DerivativeOrder::First,
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let compiled = Arc::new(
            support
                .compile_product(
                    ProductKind::JacobianAction,
                    Optimization::default(),
                    EvaluationLimits::default(),
                    &scope(),
                )
                .unwrap(),
        );
        let batch = PreparedBatch::new(compiled, DerivativeOrder::First).unwrap();
        let mut worker = batch.worker();
        let ids = [source(); 2];
        let cancel = Arc::new(AtomicBool::new(false));
        let first = worker
            .evaluate(
                BatchRequest {
                    layout: batch.layout(),
                    points: &ids,
                    inputs: &[2., 5., 3., 7.],
                    bindings: BatchBindings::JacobianAction {
                        directions: &[1., 2., 2., 1.],
                    },
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(first.point_major(), Some([12., 11., 13., 12.].as_slice()));
        let second = worker
            .evaluate(
                BatchRequest {
                    layout: batch.layout(),
                    points: &ids,
                    inputs: &[2., 5., 3., 7.],
                    bindings: BatchBindings::JacobianAction {
                        directions: &[0., 1., 0., 2.],
                    },
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(second.point_major(), Some([5., 4., 14., 12.].as_slice()));
    }
}
