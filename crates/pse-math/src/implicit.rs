// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Nested residual evaluation and implicit-function derivatives. Native solvers own iteration.
macro_rules! observed {
    ($phase:literal, $body:expr) => {
        $crate::implicit::observation::observe(
            $phase,
            concat!($phase, ".calls"),
            concat!($phase, ".elapsed"),
            concat!($phase, ".failures"),
            concat!($phase, ".completed"),
            || $body,
        )
    };
}
macro_rules! observed_ift {
    ($phase:literal, $cancel:expr, $body:expr) => {
        $crate::implicit::observation::stage(
            $phase,
            concat!($phase, ".calls"),
            concat!($phase, ".elapsed"),
            concat!($phase, ".failures"),
            concat!($phase, ".completed"),
            $cancel,
            || $body,
        )
    };
}
macro_rules! observed_proof {
    ($phase:literal, $body:expr) => {
        $crate::implicit::observation::observe_proof(
            $phase,
            concat!($phase, ".calls"),
            concat!($phase, ".elapsed"),
            concat!($phase, ".failures"),
            concat!($phase, ".completed"),
            || $body,
        )
    };
}
#[path = "implicit_accelerators.rs"]
pub mod accelerators;
#[path = "implicit_affine.rs"]
mod affine;
#[path = "implicit_configuration.rs"]
mod configuration;
#[path = "implicit_cubic.rs"]
mod cubic;
#[path = "implicit_isolation.rs"]
mod isolation;
#[path = "implicit_observation.rs"]
mod observation;
#[path = "implicit_products.rs"]
mod products;
#[cfg(test)]
#[path = "implicit_property_tests.rs"]
mod property_tests;
#[path = "implicit_reconstruction.rs"]
pub mod reconstruction;
#[path = "implicit_regimes.rs"]
mod regimes;
#[path = "implicit_scalar.rs"]
mod scalar;
#[path = "implicit_selection.rs"]
mod selection;
use crate::{
    MathError,
    guarded::{CompiledBody, Evaluation, Worker},
};
pub use affine::Affine;
use configuration::ConfigurationWorker;
pub use configuration::{Configuration, HintResolver};
pub use cubic::CubicRoots;
use faer::{
    Conj, Mat, Par,
    dyn_stack::{MemBuffer, MemStack},
    sparse::{
        SparseColMat, SymbolicSparseColMat,
        linalg::lu::{self, LuRef, NumericLu, SymbolicLu},
    },
};
pub use isolation::{
    ChartChainCoverage, ChartChainEvidence, ChartChainProof, ChartChainRequest, ChartChainWork,
    ProofInterval, RootActionEvidence, RootNeighborhoodEvidence, RootPointEvidence,
    SelectionAlternative, SelectionChart, SelectionEvidence, SelectionProofRefusal,
    SelectionProofRequest, SelectionScope, SelectionVerifier,
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{DerivativeOrder, ProviderValues};
pub(crate) use regimes::NumericalRefinement;
pub use regimes::{Regime, RegimeFactory, RegimeFactoryBranch, RegimeSelection, SelectedRegime};
pub use scalar::BracketedRoot;
pub use selection::{Selection, SelectionEquivalence, graph_equivalence};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

/// One original implicit unknown, with an arbitrary closed physical interval.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Unknown {
    /// Semantic identity of the unknown.
    pub id: SemanticId,
    /// Lower end of the closed physical interval.
    pub lower: f64,
    /// Upper end of the closed physical interval.
    pub upper: f64,
}
/// Explicit bounded native solve and derivative verification controls.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Options {
    /// Deterministic start; no previous trial is used implicitly.
    pub start: Vec<f64>,
    /// Characteristic unknown magnitudes in canonical units, resolved by numerical policy.
    pub variable_nominals: Vec<f64>,
    /// Original-space unknown acceptance budgets, independent of the nominal scale.
    pub variable_tolerance: Vec<f64>,
    /// Per-residual original physical acceptance budget.
    pub residual_tolerance: Vec<f64>,
    /// Positive native iteration limit.
    pub iterations: u32,
    /// Wall time charged to this child solve on the already admitted outer worker.
    pub time_limit: Duration,
    /// Dimensionless derivative linear-system backward-error tolerance.
    pub derivative_tolerance: f64,
}
/// Distinct requested derivative programs over one original residual definition.
#[derive(Clone, Debug)]
pub struct ResidualBodies {
    /// Unknown-only native/Newton product; parameters remain ordinary value inputs.
    pub unknown: Arc<CompiledBody>,
    /// Original Value source, optimizer policy and limits for demanded derivative preparation.
    pub source: Arc<CompiledBody>,
}
impl ResidualBodies {
    fn check(&self, unknowns: usize, inputs: usize) -> Result<(), MathError> {
        let a = self.unknown.prepared_support();
        let b = self.source.prepared_support();
        let axes = if self.unknown.compiled_order() >= DerivativeOrder::First {
            (0..unknowns).collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let width = unknowns
            .checked_add(inputs)
            .ok_or_else(|| MathError::Contract("implicit formal extent".into()))?;
        if !a.same_source(b)
            || a.outputs()!=b.outputs()
            || self.unknown.product_kind()!=crate::guarded::ProductKind::SelectedPartials
            || self.source.product_kind()!=crate::guarded::ProductKind::SelectedPartials
            || self.source.compiled_order()!=DerivativeOrder::Value || !self.source.coordinates().is_empty()
            || b.source_input_count()!=width
            || self.unknown.compiled_order()>DerivativeOrder::First || self.unknown.coordinates()!=axes
            || self.unknown.input_formals().iter().chain(self.source.input_formals()).any(|formal|*formal>=width)
        {
            return Err(MathError::Contract(
                "implicit residual programs have different source/output/input premises".into(),
            ));
        }
        Ok(())
    }
}

/// A library body with unknown coordinates first and independent inputs last.
#[derive(Debug)]
pub struct Problem {
    /// Semantic identity of the implicit system.
    pub id: SemanticId,
    /// Content identity of the compiled residual body.
    pub identity: ContentHash,
    /// Unknowns in coordinate order, leading the body's inputs.
    pub unknowns: Vec<Unknown>,
    /// Residual equation identities in output order.
    pub rows: Vec<SemanticId>,
    /// Number of independent inputs after the unknowns.
    pub inputs: usize,
    /// Resolved derivative facts shared with provider and native contracts.
    pub requirements: pse_kernels::DerivativeRequirements,
    residual_offsets: Vec<f64>,
    source_worker: Mutex<ResidualWorker>,
    unknown_worker: Mutex<ResidualWorker>,
    providers: Mutex<pse_kernels::ProviderBindings>,
    pattern: SymbolicSparseColMat<usize>,
    symbolic: SymbolicLu<usize>,
    factor: Mutex<FactorWorkspace>,
    parameter_source: Arc<CompiledBody>,
    product_workers: Mutex<Vec<Arc<products::ResidualProduct>>>,
    product_scratch: Mutex<products::ProductScratch>,
    max_entries: usize,
    effectful_residual: bool,
    required_provider_keys:Vec<pse_kernels::ProviderKey>,
    native_order:DerivativeOrder,
    construction_owner: Option<Arc<dyn crate::construction::ConstructionAdmission>>,
    prepared_bytes: Mutex<usize>,
    prepared_demands: Mutex<Vec<pse_kernels::ProviderRequest>>,
}
#[derive(Debug)]
struct ResidualWorker {
    worker: Worker,
    inputs: Vec<f64>,
    evaluation: crate::execution::TrialEvaluation,
}
impl ResidualWorker {
    fn new(body: &CompiledBody) -> Self {
        Self {
            worker: body.worker(),
            inputs: vec![f64::NAN; body.input_formals().len()],
            evaluation: crate::execution::TrialEvaluation::default(),
        }
    }
}
struct FactorWorkspace {
    numeric: NumericLu<usize, f64>,
    memory: MemBuffer,
    parallel: Par,
    association: Option<FactorAssociation>,
    first: Mat<f64>,
    rhs: Mat<f64>,
    original: Mat<f64>,
    sums: Vec<f64>,
    scales: Vec<f64>,
}
#[derive(Debug)]
struct FactorAssociation {
    premise: ContentHash,
    point: Vec<u64>,
    matrix: Vec<u64>,
    first_inputs: Vec<u64>,
    first_columns: Vec<usize>,
    children: Vec<crate::execution::ProviderVisit>,
}
impl std::fmt::Debug for FactorWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FactorWorkspace")
            .field("scratch_bytes", &self.memory.len())
            .finish_non_exhaustive()
    }
}
fn factor_workspace_bytes(unknowns: usize) -> Option<usize> {
    unknowns
        .checked_mul(unknowns)?
        .checked_mul(64)?
        .checked_add(unknowns.checked_mul(512)?)?
        .checked_add(4096)
}

fn selected_inputs(worker:&Worker,unknowns:&[f64],parameters:&[f64])->Result<Vec<f64>,MathError> {
    let mut values=Vec::new();fill_selected_inputs(worker,unknowns,parameters,&mut values)?;Ok(values)
}
fn fill_selected_inputs(worker:&Worker,unknowns:&[f64],parameters:&[f64],values:&mut Vec<f64>)->Result<(),MathError> {
    values.clear();
    for &formal in worker.input_formals() {
        values.push(if formal<unknowns.len() { unknowns.get(formal) } else { parameters.get(formal-unknowns.len()) }
            .copied().ok_or_else(|| MathError::Contract("implicit compiled input signature".into()))?);
    }
    Ok(())
}

// Owned public results copy only the requested representation. Internal residual
// consumers borrow the admitted worker's completed scratch instead.
fn owned_evaluation(jet: &crate::execution::TrialEvaluation, width: usize, order: DerivativeOrder) -> Evaluation {
    let mut hessians = Vec::new();
    if order >= DerivativeOrder::Second {
        hessians.resize(jet.values.len() * width * width, 0.);
        for row in 0..jet.values.len() {
            for (i, j, value) in jet.second_entries(row) {
                hessians[(row * width + i) * width + j] = value;
                hessians[(row * width + j) * width + i] = value;
            }
        }
    }
    Evaluation {
        applicability: jet.applicability.iter().flat_map(crate::execution::ApplicabilityWitness::hydrate).collect(),
        values: jet.values.clone(),
        jacobian: if order >= DerivativeOrder::First {
            (0..jet.values.len()).flat_map(|row| (0..width).map(move |axis| jet.first(row, axis))).collect()
        } else { Vec::new() },
        hessians,
    }
}

impl Problem {
    /// Bind compiled residuals with their library-owned structural Jacobian support.
    pub fn new(
        id: SemanticId,
        identity: ContentHash,
        unknowns: Vec<Unknown>,
        rows: Vec<SemanticId>,
        inputs: usize,
        bodies: ResidualBodies,
        max_entries: usize,
    ) -> Result<Self, MathError> {
        Self::new_with_parallelism(
            id,
            identity,
            unknowns,
            rows,
            inputs,
            bodies,
            max_entries,
            Par::Seq,
        )
    }
    /// Construct factor scratch under the same explicitly admitted parallelism used by
    /// numeric factors and backsolves. Direct and nested construction default to Seq.
    #[expect(
        clippy::too_many_arguments,
        reason = "the explicit admitted team accompanies the existing complete implicit problem constructor"
    )]
    pub fn new_with_parallelism(
        id: SemanticId,
        identity: ContentHash,
        unknowns: Vec<Unknown>,
        rows: Vec<SemanticId>,
        inputs: usize,
        bodies: ResidualBodies,
        max_entries: usize,
        parallel: Par,
    ) -> Result<Self, MathError> {
        let n = unknowns.len();
        bodies.check(n, inputs)?;
        let body = &bodies.source;
        // Construction admits the complete state solve. Parameter derivative
        // populations belong to the actual prepared invocation, before effects.
        let extent = if bodies.unknown.compiled_order() >= DerivativeOrder::First { n.checked_mul(n) } else { Some(n) };
        if n == 0 || n != rows.len() || extent.is_none_or(|v| v > max_entries) {
            return Err(MathError::Limit("implicit derivative extent"));
        }
        if unknowns.iter().any(|u| {
            u.lower.is_nan()
                || u.upper.is_nan()
                || u.lower > u.upper
                || u.lower == f64::INFINITY
                || u.upper == f64::NEG_INFINITY
        }) || unknowns
            .iter()
            .map(|u| u.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != n
            || rows.iter().collect::<std::collections::BTreeSet<_>>().len() != n
        {
            return Err(MathError::Contract(
                "implicit identities or intervals".into(),
            ));
        }
        let incidence = body.prepared_support().incidence(&(0..n).collect::<Vec<_>>(),&Arc::new(AtomicBool::new(false)))?;
        if incidence.support().first.len() != n {
            return Err(MathError::Contract(
                "implicit residual support extent".into(),
            ));
        }
        let indices = incidence
            .support()
            .first
            .iter()
            .enumerate()
            .flat_map(|(i, columns)| {
                columns
                    .iter()
                    .copied()
                    .filter(move |j| *j < n)
                    .map(move |j| faer::sparse::Pair::new(i, j))
            })
            .collect::<Vec<_>>();
        let (pattern, _) = SymbolicSparseColMat::try_new_from_indices(n, n, &indices)
            .map_err(|e| MathError::Library(e.to_string()))?;
        let symbolic = observed!("implicit.factor.symbolic", {
            lu::factorize_symbolic_lu(pattern.as_ref(), Default::default())
                .map_err(|e| MathError::Library(e.to_string()))
        })?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.integer(
                "implicit.factor.pattern_entries",
                "structural-entries",
                indices.len() as i64,
            );
        }
        let rhs = 1;
        let scratch = symbolic
            .factorize_numeric_lu_scratch::<f64>(parallel, Default::default())
            .or(symbolic.solve_in_place_scratch::<f64>(rhs, parallel));
        let allowance = factor_workspace_bytes(n)
            .and_then(|bytes| bytes.checked_add(max_entries.checked_mul(8)?))
            .ok_or(MathError::Limit("implicit factor workspace extent"))?;
        if scratch.size_bytes() > allowance {
            return Err(MathError::Limit("implicit factor workspace scratch"));
        }
        let factor = FactorWorkspace {
            numeric: NumericLu::new(),
            parallel,
            memory: MemBuffer::try_new(scratch)
                .map_err(|_| MathError::Limit("implicit factor scratch allocation"))?,
            association: None,
            first: Mat::zeros(n, 0), rhs: Mat::zeros(n, 0), original: Mat::zeros(n, 0),
            sums: vec![0.; n], scales: vec![0.; n],
        };
        let residual_offsets = vec![0.0; rows.len()];
        let required_provider_keys=body.prepared_support().source_providers().iter().map(pse_kernels::ProviderSpec::key).collect::<Vec<_>>();
        let available=body.prepared_support().source_available_order().max(bodies.unknown.compiled_order());
        Ok(Self {
            id,
            identity,
            unknowns,
            rows,
            inputs,
            requirements: pse_kernels::DerivativeRequirements::new(
                available,
                available,
                available,
                bodies.unknown.compiled_order(),
                available,
            )
            .map_err(|e| MathError::Contract(e.to_string()))?,
            residual_offsets,
            source_worker: Mutex::new(ResidualWorker::new(body)),
            unknown_worker: Mutex::new(ResidualWorker::new(&bodies.unknown)),
            providers: Mutex::new(pse_kernels::ProviderBindings::new()),
            pattern,
            symbolic,
            factor: Mutex::new(factor),
            parameter_source: body.clone(), product_workers: Mutex::new(Vec::new()), max_entries,
            product_scratch: Mutex::new(products::ProductScratch::new(n)),
            effectful_residual: !required_provider_keys.is_empty(),required_provider_keys,native_order:bodies.unknown.compiled_order(),
            construction_owner: crate::construction::current(), prepared_bytes: Mutex::new(0), prepared_demands: Mutex::new(Vec::new()),
        })
    }
    /// Apply an explicit authored-equality offset in named residual output order.
    /// Derivatives and guards stay those of the actual compiled body. Identity changes
    /// with every consumed offset; verifier projection must apply the same subtraction.
    pub fn with_residual_offsets(mut self, offsets: &[f64]) -> Result<Self, MathError> {
        if offsets.len() != self.rows.len() || offsets.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Contract(
                "implicit residual offset extent/value".into(),
            ));
        }
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("authored-implicit-residual-offsets")
            .hash(&self.identity);
        for (row, value) in self.rows.iter().zip(offsets) {
            h.id(row).f64(*value);
        }
        self.identity = h.finish_hash();
        self.residual_offsets = offsets.to_vec();
        Ok(self)
    }
    /// Attach admitted external workers to this residual worker; ownership is attempt-local.
    pub fn with_providers(
        mut self,
        providers: pse_kernels::ProviderBindings,
    ) -> Self {
        self.providers = Mutex::new(providers);
        self
    }
    /// Attach the compiler's resolved selected-function requirements to its residual problem.
    pub fn with_requirements(
        mut self,
        requirements: pse_kernels::DerivativeRequirements,
    ) -> Result<Self, MathError> {
        self.requirements = requirements;
        Ok(self)
    }
    /// Exact derivative capacity of the eagerly prepared native unknown product.
    pub fn native_order(&self)->DerivativeOrder { self.native_order }
    /// Structural Jacobian support of the residuals with respect to the unknowns.
    pub fn pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.as_ref()
    }
    /// Interval guards are checked before library evaluation, without clipping a trial.
    pub fn evaluate(
        &self,
        parameters: &[f64],
        unknowns: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Evaluation, MathError> {
        let program=if order==DerivativeOrder::Value { None } else { Some(self.complete_program(order)?) };
        self.with_trial_evaluation(program.as_ref().map_or(&self.source_worker,|p|&p.worker), parameters, unknowns, order, cancel, |jet| {
            Ok(owned_evaluation(jet, self.unknowns.len() + self.inputs, order))
        })
    }
    /// Native residual/unknown Jacobian product. Parameter derivative axes are never executed.
    pub fn evaluate_unknown(
        &self,
        parameters: &[f64],
        unknowns: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Evaluation, MathError> {
        self.with_trial_evaluation(&self.unknown_worker, parameters, unknowns, order, cancel, |jet| {
            Ok(owned_evaluation(jet, self.unknowns.len(), order))
        })
    }
    fn with_trial_evaluation<R>(
        &self,
        residual_worker: &Mutex<ResidualWorker>,
        parameters: &[f64],
        unknowns: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
        consume: impl FnOnce(&crate::execution::TrialEvaluation) -> Result<R, MathError>,
    ) -> Result<R, MathError> {
        let held = self.evaluate_trial_with(residual_worker, parameters, unknowns, order, cancel)?;
        consume(&held.evaluation)
    }
    fn evaluate_trial_with<'a>(
        &self,
        residual_worker: &'a Mutex<ResidualWorker>,
        parameters: &[f64],
        unknowns: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<std::sync::MutexGuard<'a, ResidualWorker>, MathError> {
        observed!("implicit.residual", {
            if let Some(context) = pse_diagnostics::observation::current() {
                context.source_ref("implicit.residual.source", self.identity.as_bytes());
                context.integer("implicit.residual.order", "derivative-order", order as i64);
            }
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            if parameters.len() != self.inputs || unknowns.len() != self.unknowns.len() {
                return Err(MathError::Contract("implicit coordinate extent".into()));
            }
            if unknowns
                .iter()
                .zip(&self.unknowns)
                .any(|(x, b)| !x.is_finite() || *x < b.lower || *x > b.upper)
            {
                return Err(MathError::Domain {
                    source_id: self.id,
                    requirement: "implicit unknown outside its declared interval",
                });
            }
            let mut held = residual_worker
                .lock()
                .map_err(|_| MathError::Library("implicit worker lock poisoned".into()))?;
            let ResidualWorker { worker, inputs, evaluation } = &mut *held;
            fill_selected_inputs(worker,unknowns,parameters,inputs)?;
            worker.evaluate_trial_into(
                inputs,
                order,
                &mut *self
                    .providers
                    .lock()
                    .map_err(|_| MathError::Library("implicit provider lock poisoned".into()))?,
                cancel,
                evaluation,
            )?;
            self.normalize_residual_values(&mut evaluation.values)?;
            Ok(held)
        })
    }
    /// Check that options match the system's extents and hold finite positive budgets.
    pub fn validate_options(&self, options: &Options) -> Result<(), MathError> {
        if options.start.len() != self.unknowns.len()
            || options.variable_nominals.len() != self.unknowns.len()
            || options.variable_tolerance.len() != self.unknowns.len()
            || options
                .variable_nominals
                .iter()
                .chain(&options.variable_tolerance)
                .any(|v| !v.is_finite() || *v <= 0. || !v.recip().is_finite())
            || options.residual_tolerance.len() != self.rows.len()
            || options.iterations == 0
            || options.time_limit.is_zero()
            || !options.derivative_tolerance.is_finite()
            || options.derivative_tolerance <= 0.0
            || options
                .residual_tolerance
                .iter()
                .any(|t| !t.is_finite() || *t <= 0.0 || !t.recip().is_finite())
            || options
                .start
                .iter()
                .zip(&self.unknowns)
                .any(|(x, b)| !x.is_finite() || *x < b.lower || *x > b.upper)
        {
            return Err(MathError::Contract(
                "implicit start, interval or budgets".into(),
            ));
        }
        Ok(())
    }
    /// Qualify the original residual and interval before publishing a root or any derivative.
    pub fn verify(
        &self,
        parameters: &[f64],
        point: &[f64],
        options: &Options,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), MathError> {
        observed!("implicit.original_assessment", {
            self.validate_options(options)?;
            self.with_trial_evaluation(&self.unknown_worker, parameters, point, DerivativeOrder::Value, cancel, |residual| {
            if residual.values.len() != self.rows.len()
                || residual
                    .values
                    .iter()
                    .zip(&options.residual_tolerance)
                    .any(|(f, t)| !f.is_finite() || f.abs() > *t)
            {
                return Err(MathError::Domain {
                    source_id: self.id,
                    requirement: "inner root did not satisfy original residual budgets",
                });
            }
            Ok(())
            })
        })
    }
    fn normalize_residual_values(&self,values:&mut [f64])->Result<(),MathError> {
        if values.len()!=self.residual_offsets.len() { return Err(MathError::Contract("implicit residual value extent".into())); }
        for (value,offset) in values.iter_mut().zip(&self.residual_offsets) { *value-=offset; }
        if values.iter().any(|v| !v.is_finite()) { return Err(MathError::Domain { source_id:self.id,requirement:"implicit offset residual is nonfinite" }); }
        Ok(())
    }
    fn verify_values(&self, values: &[f64], options: &Options) -> Result<(), MathError> {
        if values.len() != self.rows.len() || values.iter().zip(&options.residual_tolerance)
            .any(|(f,t)| !f.is_finite() || f.abs() > *t) {
            return Err(MathError::Domain { source_id:self.id, requirement:"inner root did not satisfy original residual budgets" });
        }
        Ok(())
    }
    fn frame_completed_structure(&self,h:&mut pse_ids::FramedHasher)->Option<()> {
        let factor=self.factor.lock().ok()?;let a=factor.association.as_ref()?;
        h.id(&self.id).hash(&self.identity).u64(a.point.len() as u64);
        for v in &a.point { h.u64(*v); }h.u64(a.matrix.len() as u64);for v in &a.matrix { h.u64(*v); }
        h.u64(a.children.len() as u64);
        for visit in &a.children { h.id(&visit.source).hash(&visit.provider.0);visit.linearization?.frame(h); }
        Some(())
    }
    fn completed_premise(&self, premise: ContentHash, point: &[f64]) -> Option<ContentHash> {
        let factor = self.factor.lock().ok()?;
        let association = factor.association.as_ref()?;
        if association.premise != premise || association.point.iter().copied().ne(point.iter().map(|x| x.to_bits())) { return None; }
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("implicit-current-linearization-v1").hash(&self.identity).hash(&premise).u64(association.point.len() as u64);
        for value in &association.point { h.u64(*value); }
        h.u64(association.matrix.len() as u64);
        for value in &association.matrix { h.u64(*value); }
        h.u64(association.children.len() as u64);
        for visit in &association.children {
            h.id(&visit.source).hash(&visit.provider.0);
            visit.linearization?.frame(&mut h);
        }
        Some(h.finish_hash())
    }
    fn factorize_matrix(
        &self,
        matrix: &SparseColMat<usize, f64>,
        requirement: &'static str,
        cancel: &Arc<AtomicBool>,
    ) -> Result<std::sync::MutexGuard<'_, FactorWorkspace>, MathError> {
        observed_ift!("implicit.factor.numeric", cancel, {
            let mut held = self
                .factor
                .lock()
                .map_err(|_| MathError::Contract("implicit factor lock poisoned".into()))?;
            // Affine/native workspace use never establishes an IFT completion.
            held.association = None;
            let FactorWorkspace {
                numeric,
                memory,
                parallel,
                ..
            } = &mut *held;
            self.symbolic
                .factorize_numeric_lu(
                    numeric,
                    matrix.as_ref(),
                    *parallel,
                    MemStack::new(memory),
                    Default::default(),
                )
                .map_err(|_| MathError::Domain {
                    source_id: self.id,
                    requirement,
                })?;
            Ok(held)
        })
    }
    fn invalidate_factor(&self) {
        if let Ok(mut factor) = self.factor.lock() {
            factor.association = None;
        }
    }
    fn factorize_ift(
        &self, matrix: &SparseColMat<usize, f64>, premise: ContentHash, point: &[f64], children: &[crate::execution::ProviderVisit], cancel: &Arc<AtomicBool>,
    ) -> Result<std::sync::MutexGuard<'_, FactorWorkspace>, MathError> {
        if cancel.load(Ordering::Acquire) { return Err(MathError::Cancelled); }
        let held = self.factor.lock().map_err(|_| MathError::Contract("implicit factor lock poisoned".into()))?;
        if held.association.as_ref().is_some_and(|association| {
            children.iter().all(|visit| visit.linearization.is_some())
                && association.children == children
                && association.premise == premise
                && association.point.len() == point.len()
                && association.point.iter().zip(point).all(|(a,b)| *a == b.to_bits())
                && association.matrix.len() == matrix.val().len()
                && association.matrix.iter().zip(matrix.val()).all(|(a,b)| *a == b.to_bits())
        }) {
            if let Some(context) = pse_diagnostics::observation::current() { context.count("implicit.factor.numeric_reuse", "calls", 1); }
            return Ok(held);
        }
        drop(held);
        let mut factor = self.factorize_matrix(matrix, "implicit Jacobian is singular", cancel)?;
        factor.association = Some(FactorAssociation {
            premise, point: point.iter().map(|x| x.to_bits()).collect(),
            matrix: matrix.val().iter().map(|x| x.to_bits()).collect(), first_inputs: Vec::new(), first_columns: Vec::new(), children: children.to_vec(),
        });
        Ok(factor)
    }
    fn solve_factor(
        &self,
        factor: &mut FactorWorkspace,
        rhs: faer::MatMut<'_, f64>,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), MathError> {
        observed_ift!("implicit.factor.solve", cancel, {
            self.ensure_solve_scratch(factor, rhs.ncols())?;
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("implicit.factor.backsolves", "calls", 1);
                context.count(
                    "implicit.factor.right_hand_sides",
                    "columns",
                    rhs.ncols() as u64,
                );
            }
            LuRef::new_unchecked(&self.symbolic, &factor.numeric).solve_in_place_with_conj(
                Conj::No,
                rhs,
                factor.parallel,
                MemStack::new(&mut factor.memory),
            );
            Ok(())
        })
    }
    fn ensure_solve_scratch(&self, factor: &mut FactorWorkspace, columns: usize) -> Result<(), MathError> {
        if self.unknowns.len().checked_mul(columns).is_none_or(|n| n > self.max_entries) {
            return Err(MathError::Limit("implicit factor RHS extent"));
        }
        let request = self.symbolic.solve_in_place_scratch::<f64>(columns, factor.parallel)
            .or(self.symbolic.solve_transpose_in_place_scratch::<f64>(columns, factor.parallel));
        let allowance = factor_workspace_bytes(self.unknowns.len())
            .and_then(|bytes| bytes.checked_add(self.max_entries.checked_mul(8)?))
            .ok_or(MathError::Limit("implicit factor workspace extent"))?;
        if request.size_bytes() > allowance { return Err(MathError::Limit("implicit factor workspace scratch")); }
        if factor.memory.len() < request.size_bytes() {
            factor.memory = MemBuffer::try_new(request).map_err(|_| MathError::Limit("implicit factor scratch allocation"))?;
        }
        Ok(())
    }

    /// Use one faer factorization and multiple right-hand sides for first and second IFT jets.
    pub fn derivatives(
        &self,
        parameters: &[f64],
        point: &[f64],
        order: DerivativeOrder,
        options: &Options,
        cancel: &Arc<AtomicBool>,
    ) -> Result<ProviderValues, MathError> {
        let width = self.unknowns.len().checked_add(self.inputs);
        let extent = match order {
            DerivativeOrder::Value => Some(self.unknowns.len()),
            DerivativeOrder::First => width.and_then(|w| self.unknowns.len().checked_mul(w)),
            DerivativeOrder::Second => width.and_then(|w| self.unknowns.len().checked_mul(w)?.checked_mul(w)),
        };
        if extent.is_none_or(|v| v > self.max_entries) { return Err(MathError::Limit("implicit demanded derivative extent")); }
        if order>=DerivativeOrder::First { self.complete_program(order)?; }
        let result = self.verify(parameters, point, options, cancel).and_then(|()| {
            self.derivatives_at_verified_point(parameters, point, order, options, cancel,
                numerical_product_key(self, parameters, options, solver_identity("implicit.public-ift.v1")))
        });
        if result.is_err() { self.invalidate_factor(); }
        result
    }
    fn derivatives_at_verified_point(
        &self,
        parameters: &[f64],
        point: &[f64],
        order: DerivativeOrder,
        options: &Options,
        cancel: &Arc<AtomicBool>,
        premise: ContentHash,
    ) -> Result<ProviderValues, MathError> {
        let columns = (0..self.inputs).collect::<Vec<_>>();
        if order==DerivativeOrder::Value { return Ok(ProviderValues {values:point.to_vec(),jacobian:Vec::new(),hessians:Vec::new()}); }
        let prepared=self.complete_program(order)?;
        self.derivatives_from_worker(parameters,point,order,options,cancel,premise,&prepared.worker,&columns)
    }
    fn derivatives_from_worker(
        &self, parameters: &[f64], point: &[f64], order: DerivativeOrder, options: &Options,
        cancel: &Arc<AtomicBool>, premise: ContentHash, worker: &Mutex<ResidualWorker>, columns: &[usize],
    ) -> Result<ProviderValues, MathError> {
        observed!("implicit.derivatives", {
            let n = self.unknowns.len();
            let p = columns.len();
            let mut output = ProviderValues {
                values: point.to_vec(),
                jacobian: vec![],
                hessians: vec![],
            };
            if order == DerivativeOrder::Value {
                return Ok(output);
            }
            if point
                .iter()
                .zip(&self.unknowns)
                .zip(&options.variable_tolerance)
                .any(|((value, bounds), tolerance)| {
                    *value - bounds.lower <= *tolerance || bounds.upper - *value <= *tolerance
                })
            {
                return Err(MathError::Domain {
                    source_id: self.id,
                    requirement: "implicit derivative neighborhood meets a declared unknown bound",
                });
            }
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("implicit.derivatives.parameter_columns", "columns", p as u64);
            }
            let residual = observed_ift!("implicit.derivatives.jet", cancel, {
                self.evaluate_trial_with(worker, parameters, point, order, cancel)
            })?;
            let jet = &residual.evaluation;
            self.verify_values(&jet.values, options)?;
            let matrix = observed_ift!("implicit.derivatives.extraction", cancel, {
                if jet.values.len() != n {
                    return Err(MathError::Contract("implicit residual jet extent".into()));
                }
                let values = (0..n)
                    .flat_map(|j| self.pattern.as_ref().row_idx_of_col(j).map(move |i| (i, j)))
                    .map(|(i, j)| jet.first(i, j))
                    .collect();
                Ok(SparseColMat::new(self.pattern.clone(), values))
            })?;
            let mut factor = self.factorize_ift(&matrix, premise, point, jet.provider_visits(), cancel)?;
            let computed = (|| {
            let reuse_first = factor.association.as_ref().is_some_and(|association| {
                association.children.iter().all(|visit| visit.linearization.is_some())
                    && association.first_inputs.len() == jet.first.len()
                    && association.first_columns == columns
                    && factor.first.ncols() == p
                    && association.first_inputs.iter().zip(&jet.first).all(|(a,b)| *a == b.to_bits())
            });
            if !reuse_first {
                let mut first = std::mem::replace(&mut factor.first, Mat::zeros(n, 0));
                let mut original = std::mem::replace(&mut factor.original, Mat::zeros(n, 0));
                first.resize_with(n, p, |_,_| 0.);
                original.resize_with(n, p, |_,_| 0.);
                observed_ift!("implicit.derivatives.first_rhs", cancel, {
                    for i in 0..n { for j in 0..p {
                        let value = -jet.first(i, n + j);
                        first[(i,j)] = value; original[(i,j)] = value;
                    }}
                    Ok(())
                })?;
                let checked = self.solve_factor(&mut factor, first.as_mut(), cancel).and_then(|()| {
                    observed_ift!("implicit.derivatives.verification", cancel, {
                        let FactorWorkspace { sums, scales, .. } = &mut *factor;
                        check_solve_into(&matrix, &first, &original, options.derivative_tolerance, self.id, false, sums, scales)
                    })
                });
                factor.first = first; factor.original = original;
                checked?;
                if let Some(association) = &mut factor.association {
                    association.first_inputs.clear();
                    association.first_inputs.extend(jet.first.iter().map(|x| x.to_bits()));
                    association.first_columns.clear(); association.first_columns.extend_from_slice(columns);
                }
            } else if let Some(context) = pse_diagnostics::observation::current() {
                context.count("implicit.derivatives.first_reuse", "calls", 1);
            }
            output.jacobian = observed_ift!("implicit.derivatives.expansion", cancel, {
                Ok((0..n)
                    .flat_map(|i| (0..p).map(move |j| (i, j)))
                    .map(|(i, j)| factor.first[(i, j)])
                    .collect())
            })?;
            if order == DerivativeOrder::First {
                return Ok(output);
            }
            let (pairs, mut second, original) =
                observed_ift!("implicit.derivatives.second_rhs", cancel, {
                    let pairs = (0..p)
                        .flat_map(|a| (a..p).map(move |b| (a, b)))
                        .collect::<Vec<_>>();
                    let mut second = std::mem::replace(&mut factor.rhs, Mat::zeros(n, 0));
                    let mut original = std::mem::replace(&mut factor.original, Mat::zeros(n, 0));
                    second.resize_with(n, pairs.len(), |_,_| 0.);
                    original.resize_with(n, pairs.len(), |_,_| 0.);
                    let tangent = |coordinate: usize, input: usize| {
                        if coordinate < n {
                            factor.first[(coordinate, input)]
                        } else {
                            f64::from(coordinate == n + input)
                        }
                    };
                    // This child measures the support contraction inside RHS construction;
                    // retain the library's single initialization pass over the RHS matrix.
                    observed_ift!("implicit.derivatives.contraction", cancel, {
                        for i in 0..n { for j in 0..pairs.len() {
                            let (a, b) = pairs[j];
                            let mut value = 0.0;
                            for (k, l, partial) in jet.second_entries(i) {
                                value -= partial * tangent(k, a) * tangent(l, b);
                                if k != l {
                                    value -= partial * tangent(l, a) * tangent(k, b);
                                }
                            }
                            second[(i,j)] = value; original[(i,j)] = value;
                        }}
                        Ok(())
                    })?;
                    Ok((pairs, second, original))
                })?;
            let checked = self.solve_factor(&mut factor, second.as_mut(), cancel).and_then(|()| {
                observed_ift!("implicit.derivatives.verification", cancel, {
                    let FactorWorkspace { sums, scales, .. } = &mut *factor;
                    check_solve_into(&matrix, &second, &original, options.derivative_tolerance, self.id, false, sums, scales)
                })
            });
            factor.original = original;
            if let Err(error) = checked { factor.rhs = second; return Err(error); }
            output.hessians = observed_ift!("implicit.derivatives.expansion", cancel, {
                let mut hessians = vec![0.0; n * p * p];
                for (j, (a, b)) in pairs.into_iter().enumerate() {
                    for i in 0..n {
                        hessians[(i * p + a) * p + b] = second[(i, j)];
                        hessians[(i * p + b) * p + a] = second[(i, j)];
                    }
                }
                Ok(hessians)
            })?;
            factor.rhs = second;
            Ok(output)
            })();
            if computed.is_err() { factor.association = None; }
            computed
        })
    }
}
#[cfg(test)]
fn check_solve(
    matrix: &SparseColMat<usize, f64>,
    x: &Mat<f64>,
    rhs: &Mat<f64>,
    tolerance: f64,
    id: SemanticId,
) -> Result<(), MathError> {
    check_solve_into(matrix, x, rhs, tolerance, id, false, &mut vec![0.; x.nrows()], &mut vec![0.; x.nrows()])
}
fn check_solve_into(
    matrix: &SparseColMat<usize, f64>, x: &Mat<f64>, rhs: &Mat<f64>, tolerance: f64,
    id: SemanticId, transpose: bool, sums: &mut [f64], scales: &mut [f64],
) -> Result<(), MathError> {
    let matrix = matrix.as_ref();
    if matrix.nrows() != x.nrows()
        || matrix.ncols() != x.nrows()
        || rhs.nrows() != x.nrows()
        || rhs.ncols() != x.ncols()
        || sums.len() != x.nrows() || scales.len() != x.nrows()
    {
        return Err(MathError::Contract(
            "implicit derivative solve extent".into(),
        ));
    }
    let refused = || MathError::Domain {
        source_id: id,
        requirement: "implicit derivative residual verification failed",
    };
    if matrix.val().iter().any(|value| !value.is_finite()) {
        return Err(refused());
    }
    let context = pse_diagnostics::observation::current();
    for column in 0..x.ncols() {
        sums.fill(0.0);
        for row in 0..x.nrows() {
            if !x[(row, column)].is_finite() || !rhs[(row, column)].is_finite() {
                return Err(refused());
            }
            scales[row] = rhs[(row, column)].abs();
        }
        // Ascending CSC columns retain each row's original summation order. The
        // absolute contributions must be accumulated before cancellation occurs.
        let mut visits = 0;
        for k in 0..matrix.ncols() {
            for (row, coefficient) in matrix.row_idx_of_col(k).zip(matrix.val_of_col(k)) {
                let (destination, input) = if transpose { (k, row) } else { (row, k) };
                let value = coefficient * x[(input, column)];
                sums[destination] += value;
                scales[destination] += value.abs();
                visits += 1;
            }
        }
        if let Some(context) = &context {
            context.count(
                "implicit.derivatives.verification.support_visits",
                "contributions",
                visits,
            );
        }
        for row in 0..x.nrows() {
            let error = (sums[row] - rhs[(row, column)]).abs();
            if !error.is_finite()
                || !scales[row].is_finite()
                || error > tolerance * scales[row].max(f64::MIN_POSITIVE)
            {
                return Err(refused());
            }
        }
    }
    Ok(())
}
/// Injected native capability; it runs on the already admitted outer worker.
pub trait InnerSolver: std::fmt::Debug + Send + Sync {
    /// Residual order the selected algorithm requires to compute values.
    fn minimum_order(&self) -> DerivativeOrder;
    /// Whether this algorithm implements the named semantic operational settings.
    fn honors_operational(&self, _settings: &str) -> bool {
        false
    }
    /// Versioned algorithm identity included in attempt configuration and provider reuse.
    fn identity(&self) -> ContentHash;
    /// Construct exclusive mutable scratch for one admitted provider/attempt.
    fn worker(&self) -> Box<dyn InnerWorker>;
    /// Complete immutable capability storage, reserved by factory admission.
    fn retained_bytes(&self) -> usize {
        size_of_val(self)
    }
    /// Complete private worker storage, reserved before worker construction.
    fn worker_bytes(&self) -> usize {
        size_of_val(self)
    }
}

/// Exclusive attempt-local execution of a retained implicit solver definition.
pub trait InnerWorker: std::fmt::Debug + Send {
    /// Solve within the original options and cancellation owner's budgets.
    fn solve(
        &mut self,
        problem: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        cancel: &pse_diagnostics::Cancellation,
    ) -> Result<Vec<f64>, MathError>;
}

#[derive(Debug)]
struct SolverWorker {
    identity: ContentHash,
    worker: Box<dyn InnerWorker>,
}
impl SolverWorker {
    fn new(definition: &dyn InnerSolver) -> Self {
        Self {
            identity: definition.identity(),
            worker: definition.worker(),
        }
    }
    fn identity(&self) -> ContentHash {
        self.identity
    }
    fn solve(
        &mut self,
        problem: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        cancel: &pse_diagnostics::Cancellation,
    ) -> Result<Vec<f64>, MathError> {
        let parent = pse_diagnostics::observation::current();
        let guard = parent.as_ref().map(|context| {
            context.child(
                "implicit.inner_attempt",
                pse_diagnostics::observation::Relationship::Nested,
            )
        });
        let context = guard
            .as_ref()
            .map(pse_diagnostics::observation::OperationGuard::context);
        let _entered = context
            .as_ref()
            .map(pse_diagnostics::observation::ObservationContext::enter);
        if let Some(context) = &context {
            context.source_ref("implicit.solver", self.identity.as_bytes());
            context.source_ref("implicit.residual.source", problem.identity.as_bytes());
            context.integer(
                "implicit.start.coordinates",
                "coordinates",
                options.start.len() as i64,
            );
            context.integer(
                "implicit.iteration_limit",
                "iterations",
                i64::from(options.iterations),
            );
            context.gauge(
                "implicit.time_allowance",
                "seconds",
                options.time_limit.as_secs_f64(),
            );
            context.gauge(
                "implicit.derivative_tolerance",
                "backward-error",
                options.derivative_tolerance,
            );
        }
        let result = observed!("implicit.inner_solve", {
            self.worker.solve(problem, parameters, options, cancel)
        });
        if let Some(guard) = guard {
            guard.finish(if result.is_ok() {
                pse_diagnostics::observation::Outcome::Completed
            } else {
                pse_diagnostics::observation::Outcome::Failed
            });
        }
        result
    }
}

/// Frame an implementation's explicitly versioned capability name.
pub fn solver_identity(reference: &str) -> ContentHash {
    let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::InnerSolverV1);
    identity.str(reference);
    identity.finish_hash()
}

/// Exact worker-local numerical dependencies. Accuracy controls are deliberately
/// absent from selection proof scope and present in this root/jet realization key.
pub(super) fn numerical_product_key(
    problem: &Problem,
    parameters: &[f64],
    options: &Options,
    solver: ContentHash,
) -> ContentHash {
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitNumericalProductV1);
    h.id(&problem.id)
        .hash(&problem.identity)
        .hash(&solver)
        .u64(problem.inputs as u64)
        .u64(problem.requirements.residual_compilation as u64);
    h.u64(problem.unknowns.len() as u64);
    for unknown in &problem.unknowns {
        h.id(&unknown.id).f64(unknown.lower).f64(unknown.upper);
    }
    h.u64(problem.rows.len() as u64);
    for row in &problem.rows {
        h.id(row);
    }
    for values in [
        parameters,
        &options.start,
        &options.variable_nominals,
        &options.variable_tolerance,
        &options.residual_tolerance,
    ] {
        h.u64(values.len() as u64);
        for value in values {
            h.f64(*value);
        }
    }
    h.u64(options.iterations as u64)
        .u64(options.time_limit.as_secs())
        .u64(options.time_limit.subsec_nanos() as u64)
        .f64(options.derivative_tolerance);
    h.finish_hash()
}
#[derive(Debug)]
struct NumericalProduct {
    key: ContentHash,
    order: DerivativeOrder,
    values: ProviderValues,
    inputs: Vec<u64>,
    fixed_configuration: Option<ContentHash>,
    demand: pse_kernels::ProviderDemand,
    outputs: Vec<usize>,
    bindings: Vec<u64>,
}
impl NumericalProduct {
    fn matches_demand(&self, request: &pse_kernels::PreparedProviderRequest, runtime: pse_kernels::ProviderRuntime<'_>) -> bool {
        self.demand == request.demand
            && (!matches!(request.demand, pse_kernels::ProviderDemand::TransposeAction { .. } | pse_kernels::ProviderDemand::WeightedSecondAction { .. }) || self.outputs == request.outputs)
            && self.bindings.len() == runtime.directions.len()+runtime.cotangents.len()+runtime.weights.len()
            && self.bindings.iter().zip(runtime.directions.iter().chain(runtime.cotangents).chain(runtime.weights)).all(|(a,b)| *a == b.to_bits())
    }
}
pub(super) fn numerical_product_bytes(inputs: usize, unknowns: usize) -> Result<usize, MathError> {
    // Fixed factory admission covers Value/First scientific selection. Second
    // storage belongs to actual demand preparation on the fresh attempt.
    inputs.checked_add(1).and_then(|width| width.checked_mul(unknowns))
        .and_then(|scalars| scalars.checked_add(inputs))
        .and_then(|scalars| scalars.checked_mul(2*size_of::<f64>()))
        .and_then(|bytes| bytes.checked_add(size_of::<NumericalProduct>()))
        .ok_or(MathError::Limit("implicit retained numerical product extent"))
}

/// Attempt-bound provider construction, using the same callback and cancellation owner
/// as outer math evaluation. The factory never acquires runtime CPU permits.
#[derive(Clone, Debug)]
pub struct Factory {
    /// Compiler-issued semantic choice, independent of ordinary numerical starts.
    pub selection: Selection,
    /// Shared capabilities and resolved demand.
    pub requirements: pse_kernels::DerivativeRequirements,
    /// Provider contract: parameters in, solved unknowns out.
    pub spec: pse_kernels::ProviderSpec,
    /// Distinct unknown-only and parameter derivative programs of the same residual definition.
    pub bodies: ResidualBodies,
    /// Unknowns in provider output order.
    pub unknowns: Vec<Unknown>,
    /// Residual equation identities in output order.
    pub rows: Vec<SemanticId>,
    /// Fixed or hint-resolved start, scaling and tolerance configuration.
    pub configuration: Configuration,
    /// Numerical hint program evaluated before each solve.
    pub hints: Option<Arc<CompiledBody>>,
    /// Original additive term program used for residual scaling.
    pub terms: Option<Arc<CompiledBody>>,
    /// Injected native solver.
    pub solver: Arc<dyn InnerSolver>,
    /// Outer cancellation owner for providers created without a scope.
    pub cancel: pse_diagnostics::Cancellation,
    /// Bound on implicit derivative entries.
    pub max_entries: usize,
    /// External provider registrations the residual body calls.
    pub providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
}
/// A single residual or an explicit alternative selector; both create attempt-local workers.
#[derive(Clone, Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "one factory per implicit block, built once per attempt; both variants are large, so boxing either leaves the other"
)]
pub enum ImplicitFactory {
    /// A single root problem.
    Root(Factory),
    /// Selection among alternative root problems.
    Regimes(RegimeFactory),
}
impl ImplicitFactory {
    /// Library programs and conservative derivative scratch retained by the factory.
    pub fn retained_bytes(&self) -> Result<usize, MathError> {
        match self {
            Self::Root(factory) => Self::root_bytes(factory)
                .ok_or(MathError::Limit("implicit factory retained extent")),
            Self::Regimes(factory) => Self::regime_bytes(factory),
        }
    }
    /// Separately charged shared payload and fresh attachment wrappers, counted per
    /// factory occurrence exactly as retained_bytes. Mutable scratch remains local.
    pub fn separately_owned_program_bytes(
        &self,
        allocations: &std::collections::BTreeSet<usize>,
    ) -> Result<(usize, usize), MathError> {
        let add = |(payload, wrappers): (usize, usize), body: &Arc<CompiledBody>| {
            Some((
                payload.checked_add(body.retained_bytes())?,
                wrappers.checked_add(body.owner_attachment_bytes())?,
            ))
        };
        let root = |factory: &Factory| {
            std::iter::once(&factory.bodies.source)
                .chain(std::iter::once(&factory.bodies.unknown))
                .chain(factory.hints.iter())
                .chain(factory.terms.iter())
                .chain(factory.selection.anchor.iter())
                .chain(factory.selection.restriction.iter())
                .filter(|body| allocations.contains(&(Arc::as_ptr(body) as usize)))
                .try_fold((0usize, 0usize), add)
        };
        let bytes = match self {
            Self::Root(factory) => root(factory),
            Self::Regimes(factory) => factory.alternatives.iter().try_fold(
                (0usize, 0usize),
                |(payload, wrappers), branch| {
                    let extra = [&branch.eligibility, &branch.criterion]
                        .into_iter()
                        .filter(|body| allocations.contains(&(Arc::as_ptr(body) as usize)))
                        .try_fold((0usize, 0usize), add)?;
                    let local = root(&branch.residual)?;
                    Some((
                        payload.checked_add(local.0)?.checked_add(extra.0)?,
                        wrappers.checked_add(local.1)?.checked_add(extra.1)?,
                    ))
                },
            ),
        };
        bytes.ok_or(MathError::Limit("implicit separately owned program extent"))
    }
    fn root_bytes(factory: &Factory) -> Option<usize> {
        factory
            .bodies
            .source
            .retained_bytes()
            .checked_add(size_of::<ResidualWorker>())?
            .checked_add(factory.bodies.unknown.retained_bytes())?
            .checked_add(factory.bodies.unknown.worker_bytes())?
            .checked_add(
                (factory.bodies.unknown.input_formals().len()
                    + factory.bodies.source.input_formals().len())
                .checked_mul(size_of::<f64>())?,
            )?
            .checked_add(factory.solver.retained_bytes())?
            .checked_add(factory.solver.worker_bytes())?
            .checked_add(size_of::<SolverWorker>())?
            .checked_add(factory.max_entries.checked_mul(8)?)?
            .checked_add(factor_workspace_bytes(factory.unknowns.len())?)
            .and_then(|n| n.checked_add(factory.configuration.retained_bytes()))
            .and_then(|n| {
                n.checked_add(
                    numerical_product_bytes(0, factory.unknowns.len()).ok()?,
                )
            })
            .and_then(|n| {
                n.checked_add(
                    factory
                        .hints
                        .iter()
                        .chain(&factory.terms)
                        .chain(&factory.selection.anchor)
                        .chain(&factory.selection.restriction)
                        .try_fold(0usize, |n, b| {
                            n.checked_add(b.retained_bytes())?
                                .checked_add(b.scratch_bytes())
                        })?,
                )
            })
    }
    fn regime_bytes(factory: &RegimeFactory) -> Result<usize, MathError> {
        let programs = factory
            .alternatives
            .iter()
            .try_fold(0usize, |bytes, branch| {
                bytes
                    .checked_add(Self::root_bytes(&branch.residual)?)?
                    .checked_add(branch.eligibility.retained_bytes())?
                    .checked_add(branch.eligibility.worker_bytes())?
                    .checked_add(branch.eligibility.evaluation_cache_bytes())?
                    .checked_add(branch.criterion.retained_bytes())?
                    .checked_add(branch.criterion.worker_bytes())?
                    .checked_add(branch.criterion.evaluation_cache_bytes())?
                    .checked_add((branch.eligibility.input_formals().len()+branch.criterion.input_formals().len()).checked_mul(size_of::<f64>())?)?
                    .checked_add(
                        branch
                            .isolation
                            .as_ref()
                            .map_or(0, |program| program.retained_bytes()),
                    )
            });
        let workspace = match &factory.verifier {
            Some(verifier) => {
                let programs = factory
                    .alternatives
                    .iter()
                    .filter_map(|branch| branch.isolation.clone())
                    .collect::<Vec<_>>();
                verifier.workspace_bytes(&programs)?
            }
            None => 0,
        };
        let certificate = regimes::certificate_bytes(
            factory.spec.inputs.len(),
            factory
                .alternatives
                .iter()
                .map(|branch| branch.residual.unknowns.len()),
        )?;
        programs
            .and_then(|bytes| bytes.checked_add(workspace))
            .and_then(|bytes| bytes.checked_add(certificate))
            .ok_or(MathError::Limit("implicit regime factory retained extent"))
    }
    /// Preserve the runtime's admission lease through every separately compiled program.
    pub fn retain(&mut self, owner: Arc<dyn crate::AllocationOwner>) {
        let retain = |body: &mut Arc<CompiledBody>| {
            *body = Arc::new(body.as_ref().clone().with_owner(owner.clone()))
        };
        match self {
            Self::Root(factory) => {
                retain(&mut factory.bodies.source);
                retain(&mut factory.bodies.unknown);
                for body in factory.hints.iter_mut().chain(&mut factory.terms) {
                    retain(body);
                }
                for body in factory
                    .selection
                    .anchor
                    .iter_mut()
                    .chain(&mut factory.selection.restriction)
                {
                    retain(body);
                }
            }
            Self::Regimes(factory) => {
                for branch in &mut factory.alternatives {
                    retain(&mut branch.residual.bodies.source);
                    retain(&mut branch.residual.bodies.unknown);
                    for body in branch
                        .residual
                        .hints
                        .iter_mut()
                        .chain(&mut branch.residual.terms)
                    {
                        retain(body);
                    }
                    for body in branch
                        .residual
                        .selection
                        .anchor
                        .iter_mut()
                        .chain(&mut branch.residual.selection.restriction)
                    {
                        retain(body);
                    }
                    retain(&mut branch.eligibility);
                    retain(&mut branch.criterion);
                }
            }
        }
    }
    /// Bind only dependencies supplied by the admitted definition's provider graph.
    pub fn set_providers(
        &mut self,
        providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    ) {
        match self {
            Self::Root(factory) => factory.providers = providers,
            Self::Regimes(factory) => {
                for branch in &mut factory.alternatives {
                    branch.residual.providers = providers.clone();
                }
            }
        }
    }
}
impl pse_kernels::ProviderFactory for ImplicitFactory {
    fn source_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        match self {
            Self::Root(v) => v.spec(),
            Self::Regimes(v) => v.spec(),
        }
    }
    fn configuration_key(&self) -> ContentHash {
        match self {
            Self::Root(v) => v.configuration_key(),
            Self::Regimes(v) => v.configuration_key(),
        }
    }
    fn create(&self) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
        match self {
            Self::Root(v) => v.create(),
            Self::Regimes(v) => v.create(),
        }
    }
    fn create_scoped(
        &self,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
        match self {
            Self::Root(v) => v.create_scoped(scope),
            Self::Regimes(v) => v.create_scoped(scope),
        }
    }
}
impl pse_kernels::ProviderFactory for Factory {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.spec
    }
    fn configuration_key(&self) -> ContentHash {
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitConfigurationV2);
        h.hash(&self.spec.identity()).hash(&self.solver.identity());
        for order in [
            self.requirements.residual_available,
            self.requirements.output_smoothness,
            self.requirements.selector_neighborhood,
            self.requirements.inner_minimum,
            self.requirements.requested_output,
            self.requirements.residual_compilation,
        ] {
            h.u64(order as u64);
        }
        for u in &self.unknowns {
            h.id(&u.id).u64(u.lower.to_bits()).u64(u.upper.to_bits());
        }
        h.hash(&self.configuration.identity());
        for r in self.providers.values() {
            h.hash(&r.configuration_key());
        }
        h.finish_hash()
    }
    fn create(&self) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
        self.create_scoped(pse_kernels::ExecutionScope::new(self.cancel.clone(), None))
    }
    fn create_scoped(
        &self,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
        scope.check()?;
        let problem = self.problem(scope.clone())?;
        Ok(pse_kernels::ProviderBinding::new(self.spec.clone(), Box::new(Nested {
            problem,
            configuration: ConfigurationWorker::new(
                self.configuration.clone(),
                self.hints.as_ref(),
                self.terms.as_ref(),
            ),
            solver: SolverWorker::new(self.solver.as_ref()),
            cancel: scope.cancellation().clone(),
            scope,
            selection: selection::SelectionWorker::new(&self.selection),
            requested_output: self.requirements.requested_output,
            numerical: None, linearization: None,
        })))
    }
}
impl Factory {
    fn problem(
        &self,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<Arc<Problem>, pse_kernels::ProviderError> {
        scope.check()?;
        if self.requirements.inner_minimum != self.solver.minimum_order()
            || self.bodies.unknown.compiled_order() < self.requirements.inner_minimum
        {
            return Err(pse_kernels::ProviderError::Contract(
                "implicit compiled requirements disagree with selected solver".into(),
            ));
        }
        if self
            .selection
            .settings
            .as_ref()
            .is_some_and(|settings| !self.solver.honors_operational(settings))
        {
            return Err(pse_kernels::ProviderError::Contract(
                "selected realization cannot honor implicit operational settings".into(),
            ));
        }
        let providers = self
            .providers
            .iter()
            .map(|(k, r)| r.worker_scoped(scope.nested()).map(|w| (*k, w)))
            .collect::<Result<_, _>>()?;
        // Admission is a capacity ceiling, not placement or a promise of actual
        // parallel work. Query the pool width only on a Rayon worker: outside one,
        // current_num_threads would initialize the unrelated global pool.
        let threads = if rayon::current_thread_index().is_some() {
            scope
                .admitted_threads()
                .get()
                .min(rayon::current_num_threads())
        } else {
            1
        };
        let parallel = if threads == 1 {
            Par::Seq
        } else {
            Par::rayon(threads)
        };
        let mut problem = Problem::new_with_parallelism(
            self.spec.id,
            self.spec.identity(),
            self.unknowns.clone(),
            self.rows.clone(),
            self.spec.inputs.len(),
            self.bodies.clone(),
            self.max_entries,
            parallel,
        )
        .map_err(provider_error)?
        .with_requirements(self.requirements)
        .map_err(provider_error)?
        .with_providers(providers);
        problem
            .source_worker
            .get_mut()
            .map_err(|_| {
                pse_kernels::ProviderError::Terminal("implicit worker lock poisoned".into())
            })?
            .worker
            .set_scope(scope.clone());
        problem
            .unknown_worker
            .get_mut()
            .map_err(|_| {
                pse_kernels::ProviderError::Terminal("implicit unknown worker lock poisoned".into())
            })?
            .worker
            .set_scope(scope.clone());
        {
            let providers = problem.providers.get_mut().map_err(|_| pse_kernels::ProviderError::Terminal("implicit provider lock poisoned".into()))?;
            problem.unknown_worker.get_mut().map_err(|_| pse_kernels::ProviderError::Terminal("implicit worker lock poisoned".into()))?.worker.prepare_providers(providers).map_err(provider_error)?;
        }
        let problem = Arc::new(problem);
        scope.check()?;
        if let Configuration::Fixed(_, options) = &self.configuration
            && self.selection.anchor.is_none()
        {
            problem.validate_options(options).map_err(provider_error)?;
        }
        Ok(problem)
    }
}

#[derive(Debug)]
struct Nested {
    scope: pse_kernels::ExecutionScope,
    requested_output: DerivativeOrder,
    selection: selection::SelectionWorker,
    problem: Arc<Problem>,
    configuration: ConfigurationWorker,
    solver: SolverWorker,
    cancel: Arc<AtomicBool>,
    numerical: Option<NumericalProduct>,
    linearization: Option<ContentHash>,
}
impl pse_kernels::Provider for Nested {
    fn supports(&self, demand: &pse_kernels::ProviderDemand) -> bool {
        matches!(demand, pse_kernels::ProviderDemand::CompleteJets | pse_kernels::ProviderDemand::SelectedColumns { .. }) || self.supports_linearization()
    }
    fn supports_linearization(&self) -> bool {
        self.problem.providers.lock().is_ok_and(|bank| self.problem.required_provider_keys.as_slice().iter().all(|key| bank.supports_linearization(*key)))
    }
    fn completed_linearization(&self) -> Option<ContentHash> { self.linearization }
    fn prepare_demand(&mut self, request: &pse_kernels::PreparedProviderRequest) -> Result<(), pse_kernels::ProviderError> {
        if request.order > self.requested_output { return Err(pse_kernels::ProviderError::Contract("implicit output exceeds compiled demand".into())); }
        self.problem.prepare_product(request, &self.scope).map_err(provider_error)
    }
    fn fill(
        &mut self,
        inputs: &[f64],
        request: &pse_kernels::PreparedProviderRequest,
        runtime: pse_kernels::ProviderRuntime<'_>,
        output: pse_kernels::ProviderOutput<'_>,
        context: &pse_kernels::EvaluationContext<'_>,
    ) -> Result<(), pse_kernels::ProviderError> {
        observed!("implicit.provider", {
            let result = self.evaluate_inner(inputs, request, runtime, output, context);
            if result.is_err() {
                self.numerical = None; self.linearization = None;
                self.problem.invalidate_factor();
            }
            result
        })
    }
}
impl Nested {
    fn evaluate_inner(
        &mut self,
        inputs: &[f64],
        request: &pse_kernels::PreparedProviderRequest,
        runtime: pse_kernels::ProviderRuntime<'_>,
        output: pse_kernels::ProviderOutput<'_>,
        context: &pse_kernels::EvaluationContext<'_>,
    ) -> Result<(), pse_kernels::ProviderError> {
        self.scope.check()?;
        request.check_context(context)?;
        if request.order > self.requested_output {
            return Err(pse_kernels::ProviderError::Contract(
                "implicit output exceeds compiled demand".into(),
            ));
        }
        if !std::ptr::eq(context.cancelled, self.cancel.as_ref()) {
            return Err(pse_kernels::ProviderError::Contract(
                "nested provider requires its admitted outer cancellation owner".into(),
            ));
        }
        if !self.problem.prepared_demands.lock().map_err(|_| pse_kernels::ProviderError::Terminal("implicit preparation lock poisoned".into()))?.contains(request.request()) {
            return Err(pse_kernels::ProviderError::Contract("implicit demand was not prepared on this binding".into()));
        }
        let started = std::time::Instant::now();
        // A fixed configuration and the immutable worker's residual, provider and
        // pure selector programs cover this premise. Provider-backed selectors
        // and hint resolvers still run in their original refusal order below.
        if let Some((identity, allowance)) = self.configuration.fixed_identity()
            && self.selection.permits_fixed_reuse()
            && !self.problem.effectful_residual
            && let Some(product) = &self.numerical
            && product.fixed_configuration == Some(identity)
            && product.inputs.len() == inputs.len()
            && product
                .inputs
                .iter()
                .zip(inputs)
                .all(|(a, b)| *a == b.to_bits())
            && product.order >= request.order
            && product.matches_demand(request, runtime)
        {
            self.scope.remaining(allowance)?;
            Self::project(&product.values, request, inputs.len(), output)?;
            self.scope
                .remaining(allowance.saturating_sub(started.elapsed()))?;
            if let Some(context) = pse_diagnostics::observation::current() {
                context.count("implicit.product.full_reuse", "requests", 1);
                if product.order > request.order {
                    context.count("implicit.product.order_projection", "requests", 1);
                }
            }
            return Ok(());
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            let reason = match (&self.numerical, self.configuration.fixed_identity()) {
                (None, _) => "implicit.product.miss.empty",
                (Some(_), None) => "implicit.product.miss.dynamic_configuration",
                (Some(_), _) if !self.selection.permits_fixed_reuse() => {
                    "implicit.product.miss.selection_effect"
                }
                (Some(product), Some((identity, _)))
                    if product.fixed_configuration != Some(identity) =>
                {
                    "implicit.product.miss.configuration"
                }
                (Some(product), _)
                    if product.inputs.len() != inputs.len()
                        || product
                            .inputs
                            .iter()
                            .zip(inputs)
                            .any(|(a, b)| *a != b.to_bits()) =>
                {
                    "implicit.product.miss.inputs"
                }
                (Some(product), _) if product.order < request.order => {
                    "implicit.product.miss.order_upgrade"
                }
                _ => "implicit.product.miss.unestablished",
            };
            context.count(reason, "requests", 1);
        }
        let anchor = self
            .selection
            .anchor(&self.problem, inputs, &self.cancel)
            .map_err(provider_error)?;
        let mut options = self
            .configuration
            .resolve(&mut self.problem, inputs, &self.cancel, anchor.as_deref())
            .map_err(provider_error)?;
        let allowance = options.time_limit;
        self.selection
            .configure(&mut self.problem, &options)
            .map_err(provider_error)?;
        let numerical_key =
            numerical_product_key(&self.problem, inputs, &options, self.solver.identity());
        options.time_limit = self
            .scope
            .remaining(allowance.saturating_sub(started.elapsed()))?;
        if options.time_limit.is_zero() {
            return Err(pse_kernels::ProviderError::Limit(
                "implicit configuration time",
            ));
        }
        self.scope.check()?;
        let point = match self
            .numerical
            .as_ref()
            .filter(|product| product.key == numerical_key && !self.problem.effectful_residual)
        {
            Some(product) => {
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("implicit.product.point_reuse", "requests", 1);
                }
                product.values.values.clone()
            }
            None => self
                .solver
                .solve(
                    self.problem.clone(),
                    inputs,
                    &options,
                    self.scope.cancellation_control(),
                )
                .map_err(provider_error)?,
        };
        self.scope
            .remaining(allowance.saturating_sub(started.elapsed()))?;
        if self
            .numerical
            .as_ref()
            .is_none_or(|product| product.key != numerical_key || self.problem.effectful_residual)
        {
            self.problem
                .verify(inputs, &point, &options, &self.cancel)
                .map_err(provider_error)?;
        }
        let all = match self
            .numerical
            .as_ref()
            .filter(|product| product.key == numerical_key && product.order >= request.order && product.matches_demand(request, runtime) && !self.problem.effectful_residual)
        {
            Some(_) => {
                if let Some(context) = pse_diagnostics::observation::current() {
                    context.count("implicit.product.jet_reuse", "requests", 1);
                }
                None
            }
            None => Some(self
                .problem
                .demanded_derivatives(
                    inputs,
                    &point,
                    request,
                    runtime,
                    &options,
                    numerical_key,
                    &self.scope,
                )
                .map_err(provider_error)?),
        };
        self.scope
            .remaining(allowance.saturating_sub(started.elapsed()))?;
        if !self.selection.reusable_restriction()
            || self
                .numerical
                .as_ref()
                .is_none_or(|product| product.key != numerical_key || product.order < request.order)
        {
            self.selection
                .verify(&self.problem, inputs, &point, request.order, &self.cancel)
                .map_err(provider_error)?;
        }
        if started.elapsed() >= allowance {
            return Err(pse_kernels::ProviderError::Limit(
                "implicit evaluation time",
            ));
        }
        let completed = all.as_ref().or_else(|| self.numerical.as_ref().map(|product| &product.values))
            .ok_or_else(|| pse_kernels::ProviderError::Terminal("implicit numerical completion missing".into()))?;
        Self::project(completed, request, inputs.len(), output)?;
        self.scope.check()?;
        self.linearization = if request.order >= DerivativeOrder::First { self.problem.completed_premise(numerical_key, &point) } else { None };
        if let Some(all) = all {
            self.numerical = Some(NumericalProduct {
                key: numerical_key,
                order: request.order,
                values: all,
                inputs: inputs.iter().map(|value| value.to_bits()).collect(),
                fixed_configuration: self
                    .configuration
                    .fixed_identity()
                    .map(|(identity, _)| identity),
                demand: request.demand.clone(), outputs: request.outputs.clone(),
                bindings: runtime.directions.iter().chain(runtime.cotangents).chain(runtime.weights).map(|x| x.to_bits()).collect(),
            });
        }
        Ok(())
    }
    fn project(
        all: &ProviderValues,
        request: &pse_kernels::PreparedProviderRequest,
        n: usize,
        output: pse_kernels::ProviderOutput<'_>,
    ) -> Result<(), pse_kernels::ProviderError> {
        Self::project_fields(&all.values,&all.jacobian,&all.hessians,request,n,output)
    }
    fn project_fields(values:&[f64],jacobian:&[f64],hessians:&[f64],request:&pse_kernels::PreparedProviderRequest,n:usize,output:pse_kernels::ProviderOutput<'_>)->Result<(),pse_kernels::ProviderError> {
        let first_width = match &request.demand {
            pse_kernels::ProviderDemand::SelectedColumns { columns } => columns.len(),
            pse_kernels::ProviderDemand::JacobianAction { directions } => *directions,
            _ => n,
        };
        let contracted = matches!(request.demand, pse_kernels::ProviderDemand::TransposeAction { .. } | pse_kernels::ProviderDemand::WeightedSecondAction { .. });
        if contracted { output.jacobian.copy_from_slice(jacobian); output.hessians.copy_from_slice(hessians); }
        for (row, &i) in request.outputs.iter().enumerate() {
            output.values[row] = values[i];
            if request.order >= DerivativeOrder::First && !contracted {
                output.jacobian[row*first_width..(row+1)*first_width].copy_from_slice(&jacobian[i*first_width..(i+1)*first_width]);
            }
            if request.order >= DerivativeOrder::Second && !contracted {
                let width = first_width*first_width;
                output.hessians[row*width..(row+1)*width].copy_from_slice(&hessians[i*width..(i+1)*width]);
            }
        }
        Ok(())
    }
}
fn provider_error(error: MathError) -> pse_kernels::ProviderError {
    match error {
        MathError::Cancelled => pse_kernels::ProviderError::Cancelled,
        MathError::Limit(limit) => pse_kernels::ProviderError::Limit(limit),
        MathError::Provider { cause, .. } | MathError::Scope(cause) => cause,
        other => {
            let retained = other.retained_bytes();
            pse_kernels::ProviderError::Nested {
                cause: pse_model::diagnostic::DiagnosticCause::new(other),
                retained,
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn residual_bodies_for_test(parameter: Arc<CompiledBody>) -> ResidualBodies {
    let unknowns = parameter.prepared_support().outputs().len();
    let order = parameter.compiled_order().min(DerivativeOrder::First);
    let axes = if order >= DerivativeOrder::First {
        (0..unknowns).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let compile_control = pse_diagnostics::Cancellation::new(false);
    let cancel = compile_control.flag();
    let support = parameter
        .prepared_support()
        .incidence(&axes, &cancel)
        .unwrap()
        .at_order(order)
        .unwrap();
    let unknown = Arc::new(
        support
            .compile(
                crate::library::Optimization::default(),
                crate::jets::EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(compile_control.clone(), None),
            )
            .unwrap(),
    );
    let source=Arc::new(parameter.prepared_support().incidence(&[],&cancel).unwrap().at_order(DerivativeOrder::Value).unwrap().compile(parameter.optimization(),parameter.evaluation_limits(),&pse_kernels::ExecutionScope::new(compile_control,None)).unwrap());
    ResidualBodies { unknown, source }
}

#[cfg(test)]
#[path = "implicit_deadline_tests.rs"]
mod deadline_tests;

#[cfg(test)]
fn provider_values(
    provider: &mut pse_kernels::ProviderBinding, inputs: &[f64], request: &pse_kernels::ProviderRequest,
    context: &pse_kernels::EvaluationContext<'_>,
) -> Result<ProviderValues, pse_kernels::ProviderError> {
    let request = provider.prepare(request.clone())?;
    let mut buffer = pse_kernels::ProviderBuffer::new(&request);
    let completed = buffer.fill(provider, inputs, &request, Default::default(), context)?;
    Ok(ProviderValues { values: completed.values.to_vec(), jacobian: completed.jacobian.to_vec(), hessians: completed.hessians.to_vec() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verification_matrix(n: usize, entries: &[(usize, usize, f64)]) -> SparseColMat<usize, f64> {
        let indices = entries
            .iter()
            .map(|(row, column, _)| faer::sparse::Pair::new(*row, *column))
            .collect::<Vec<_>>();
        let (pattern, _) = SymbolicSparseColMat::try_new_from_indices(n, n, &indices).unwrap();
        let values = (0..n)
            .flat_map(|column| {
                pattern
                    .as_ref()
                    .row_idx_of_col(column)
                    .map(move |row| (row, column))
            })
            .map(|(row, column)| {
                entries
                    .iter()
                    .find(|(i, j, _)| *i == row && *j == column)
                    .unwrap()
                    .2
            })
            .collect();
        SparseColMat::new(pattern, values)
    }

    fn assert_verification_refused(result: Result<(), MathError>) {
        assert!(matches!(
            result,
            Err(MathError::Domain {
                requirement: "implicit derivative residual verification failed",
                ..
            })
        ));
    }

    #[test]
    fn sparse_verification_banded_packed_rhs_and_bad_solve() {
        let id = SemanticId::from_bytes([61; 16]);
        let matrix = verification_matrix(
            4,
            &[
                (0, 0, 2.0),
                (1, 0, 3.0),
                (1, 1, 4.0),
                (2, 1, -5.0),
                (2, 2, 6.0),
                (3, 2, 7.0),
                (3, 3, 8.0),
            ],
        );
        // Three packed second RHS columns for two independent parameters.
        let mut x = faer::mat![
            [1.0, 0.0, -1.0],
            [2.0, 1.0, 0.0],
            [3.0, -1.0, 2.0],
            [4.0, 2.0, -2.0]
        ];
        let rhs = faer::mat![
            [2.0, 0.0, -2.0],
            [11.0, 4.0, -3.0],
            [8.0, -11.0, 12.0],
            [53.0, 9.0, -2.0]
        ];
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("implicit.sparse_verification.packed");
        let _entered = operation.context().enter();
        check_solve(&matrix, &x, &rhs, 1e-12, id).unwrap();
        let snapshot = operation.reader().snapshot(Duration::from_secs(60));
        let visits = snapshot
            .metrics
            .iter()
            .find(|metric| metric.name == "implicit.derivatives.verification.support_visits")
            .unwrap();
        assert_eq!(
            visits.cumulative,
            pse_diagnostics::observation::Value::Count(7 * 3)
        );
        x[(3, 2)] += 0.01;
        assert_verification_refused(check_solve(&matrix, &x, &rhs, 1e-12, id));
    }

    #[test]
    fn sparse_verification_preserves_absolute_contribution_and_rhs_scale() {
        let id = SemanticId::from_bytes([62; 16]);
        let matrix = verification_matrix(
            3,
            &[
                (0, 0, 1e16),
                (0, 1, 1.0),
                (0, 2, -1e16),
                (1, 1, 1.0),
                (2, 2, 1.0),
            ],
        );
        let x = Mat::from_fn(3, 1, |_, _| 1.0);
        // Ascending columns yield (1e16 + 1) - 1e16 = 0. The lost unit
        // is small against the absolute contribution scale, despite cancellation.
        let rhs = Mat::from_fn(3, 1, |_, _| 1.0);
        check_solve(&matrix, &x, &rhs, 1e-12, id).unwrap();
        // A zero tolerance pins the original ascending-column arithmetic: an
        // implementation that groups the cancelling terms first instead yields 1.
        check_solve(&matrix, &x, &faer::mat![[0.0], [1.0], [1.0]], 0.0, id).unwrap();
        // Omitting the RHS from the scale would incorrectly refuse this error:
        // 0.15 <= 0.1 * (1 + 1.15), but 0.15 > 0.1 * 1.
        let scalar = verification_matrix(1, &[(0, 0, 1.0)]);
        check_solve(&scalar, &faer::mat![[1.0]], &faer::mat![[1.15]], 0.1, id).unwrap();
        assert_verification_refused(check_solve(
            &scalar,
            &faer::mat![[1.0]],
            &faer::mat![[1.3]],
            0.1,
            id,
        ));
        // Preserve the minimum-positive scale floor for an all-zero row.
        check_solve(
            &verification_matrix(1, &[]),
            &faer::mat![[0.0]],
            &faer::mat![[f64::MIN_POSITIVE / 4.0]],
            0.5,
            id,
        )
        .unwrap();
    }

    #[test]
    fn sparse_verification_rejects_nonfinite_coordinates_and_arithmetic() {
        let id = SemanticId::from_bytes([63; 16]);
        // The empty second column must not hide a nonfinite solved coordinate.
        let matrix = verification_matrix(2, &[(0, 0, 1.0)]);
        for nonfinite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for row in 0..2 {
                let mut x = Mat::zeros(2, 2);
                x[(row, 1)] = nonfinite;
                assert_verification_refused(check_solve(&matrix, &x, &Mat::zeros(2, 2), 1e-12, id));
                let mut rhs = Mat::zeros(2, 2);
                rhs[(row, 1)] = nonfinite;
                assert_verification_refused(check_solve(
                    &matrix,
                    &Mat::zeros(2, 2),
                    &rhs,
                    1e-12,
                    id,
                ));
            }
            assert_verification_refused(check_solve(
                &verification_matrix(2, &[(0, 0, 1.0), (1, 1, nonfinite)]),
                &Mat::zeros(2, 1),
                &Mat::zeros(2, 1),
                1e-12,
                id,
            ));
        }
        // Finite inputs can still overflow the action and must be refused.
        assert_verification_refused(check_solve(
            &verification_matrix(1, &[(0, 0, f64::MAX)]),
            &faer::mat![[2.0]],
            &faer::mat![[0.0]],
            1e-12,
            id,
        ));
        // Cancelling finite actions must not turn an overflowing absolute scale
        // into an infinite tolerance that accepts an arbitrary finite bad RHS.
        assert_verification_refused(check_solve(
            &verification_matrix(2, &[(0, 0, f64::MAX), (0, 1, -f64::MAX)]),
            &faer::mat![[1.0], [1.0]],
            &faer::mat![[100.0], [0.0]],
            1e-12,
            id,
        ));
        // A structurally empty singular row still independently detects a bad RHS.
        assert_verification_refused(check_solve(
            &matrix,
            &Mat::zeros(2, 1),
            &faer::mat![[0.0], [1.0]],
            1e-12,
            id,
        ));
    }

    #[test]
    fn sparse_verification_diagonal_ift_first_and_packed_second() {
        use crate::guarded::{PreparedBody, Stage};
        use symbolica::atom::Atom;
        crate::initialize().unwrap();
        for n in [4, 8] {
            let id = SemanticId::from_bytes([64; 16]);
            let width = n + 2;
            let p = crate::library::formal(n).unwrap();
            let q = crate::library::formal(n + 1).unwrap();
            // Each residual is strictly increasing in its own unknown, with one
            // global branch: y_i = (i + 1)p² + pq + q². Admit every formal axis
            // explicitly so the actual first and packed-second IFT paths run.
            let expressions = (0..n)
                .map(|i| {
                    crate::library::formal(i).unwrap()
                        - Atom::num((i + 1) as i64) * &p * &p
                        - &p * &q
                        - &q * &q
                })
                .collect::<Vec<_>>();
            let outputs = (width..width + n).collect::<Vec<_>>();
            let source = PreparedBody::new(
                width,
                width + n,
                outputs.clone(),
                vec![Stage::Block {
                    expressions,
                    outputs,
                    source: id,
                }],
                DerivativeOrder::Second,
            )
            .unwrap();
            let control = pse_diagnostics::Cancellation::new(false);
            let cancel = control.flag();
            let body = Arc::new(
                source
                    .compile(
                        &(0..n).collect::<Vec<_>>(),
                        &(0..width).collect::<Vec<_>>(),
                        DerivativeOrder::Second,
                        crate::library::Optimization::default(),
                        crate::jets::EvaluationLimits::default(),
                        &pse_kernels::ExecutionScope::new(control.clone(), None),
                    )
                    .unwrap(),
            );
            let problem = Problem::new(
                id,
                ContentHash::from_bytes([0; 32]),
                (0..n)
                    .map(|i| Unknown {
                        id: pse_ids::named_id(id, &format!("unknown.{i}")),
                        lower: -100.0,
                        upper: 100.0,
                    })
                    .collect(),
                (0..n)
                    .map(|i| pse_ids::named_id(id, &format!("row.{i}")))
                    .collect(),
                2,
                residual_bodies_for_test(body),
                10_000,
            )
            .unwrap();
            let point = (0..n)
                .map(|i| 4.0 * (i + 1) as f64 + 15.0)
                .collect::<Vec<_>>();
            let options = Options {
                start: point.clone(),
                variable_nominals: vec![1.0; n],
                variable_tolerance: vec![1e-9; n],
                residual_tolerance: vec![1e-10; n],
                iterations: 30,
                time_limit: Duration::from_secs(1),
                derivative_tolerance: 1e-12,
            };
            let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
            let operation = hub.start("implicit.sparse_verification.analytic");
            let _entered = operation.context().enter();
            let jet = problem
                .derivatives(
                    &[2.0, 3.0],
                    &point,
                    DerivativeOrder::Second,
                    &options,
                    &cancel,
                )
                .unwrap();
            assert_eq!(jet.values, point);
            for i in 0..n {
                assert_eq!(
                    &jet.jacobian[2 * i..2 * i + 2],
                    &[4.0 * (i + 1) as f64 + 3.0, 8.0]
                );
                assert_eq!(
                    &jet.hessians[4 * i..4 * i + 4],
                    &[2.0 * (i + 1) as f64, 1.0, 1.0, 2.0]
                );
            }
            let snapshot = operation.reader().snapshot(Duration::from_secs(60));
            let count = |name: &str| {
                snapshot
                    .metrics
                    .iter()
                    .find(|metric| metric.name == name)
                    .map(|metric| metric.cumulative.clone())
            };
            use pse_diagnostics::observation::Value;
            assert_eq!(
                count("implicit.factor.numeric.completed"),
                Some(Value::Count(1))
            );
            assert_eq!(
                count("implicit.factor.solve.completed"),
                Some(Value::Count(2))
            );
            assert_eq!(
                count("implicit.factor.right_hand_sides"),
                Some(Value::Count(5))
            );
            assert_eq!(
                count("implicit.derivatives.verification.support_visits"),
                Some(Value::Count((n * 5) as u64))
            );
            control.cancel();
            assert!(matches!(
                problem.derivatives_at_verified_point(
                    &[2.0, 3.0],
                    &jet.values,
                    DerivativeOrder::Second,
                    &options,
                    &cancel,
                    ContentHash::from_bytes([0; 32]),
                ),
                Err(MathError::Cancelled)
            ));
        }
    }

    #[test]
    fn implicit_dual_programs_require_canonical_parameter_axes_and_analytic_ift() {
        use crate::guarded::{PreparedBody, Stage};
        use symbolica::atom::Atom;
        crate::initialize().unwrap();
        let x = crate::library::formal(0).unwrap();
        let p = crate::library::formal(1).unwrap();
        let id = SemanticId::from_bytes([43; 16]);
        let compile_control = pse_diagnostics::Cancellation::new(false);
        let cancel = compile_control.flag();
        // u = 2p + p² independently gives u'(3)=8 and u''(3)=2.
        let source = PreparedBody::new(
            2,
            3,
            vec![2],
            vec![Stage::Block {
                expressions: vec![x - Atom::num(2) * &p - &p * &p],
                outputs: vec![2],
                source: id,
            }],
            DerivativeOrder::Second,
        )
        .unwrap();
        let compile = |axes: &[usize], order| {
            Arc::new(
                source
                    .compile(
                        &[0],
                        axes,
                        order,
                        crate::library::Optimization::default(),
                        crate::jets::EvaluationLimits::default(),
                        &pse_kernels::ExecutionScope::new(compile_control.clone(), None),
                    )
                    .unwrap(),
            )
        };
        let unknown = compile(&[0], DerivativeOrder::First);
        for axes in [&[1, 0][..], &[0][..], &[1][..]] {
            let bodies = ResidualBodies {
                unknown: unknown.clone(),
                source: compile(axes, DerivativeOrder::First),
            };
            assert!(
                matches!(bodies.check(1, 1), Err(MathError::Contract(_))),
                "axes {axes:?}"
            );
        }
        let options = Options {
            start: vec![15.],
            variable_nominals: vec![1.],
            variable_tolerance: vec![1e-10],
            residual_tolerance: vec![1e-10],
            iterations: 10,
            time_limit: Duration::from_secs(1),
            derivative_tolerance: 1e-10,
        };
        for order in [DerivativeOrder::First, DerivativeOrder::Second] {
            let bodies = ResidualBodies {
                unknown: unknown.clone(),
                source: compile(&[0, 1], order),
            };
            let problem = Problem::new(
                id,
                ContentHash::from_bytes([43; 32]),
                vec![Unknown {
                    id,
                    lower: -100.,
                    upper: 100.,
                }],
                vec![id],
                1,
                bodies,
                1024,
            )
            .unwrap();
            let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
            let operation = hub.start("implicit.ift.stage_control");
            let _entered = operation.context().enter();
            let actual = problem
                .derivatives(&[3.], &[15.], order, &options, &cancel)
                .unwrap();
            let snapshot = operation.reader().snapshot(Duration::from_secs(60));
            let count = |name: &str| {
                snapshot
                    .metrics
                    .iter()
                    .find(|metric| metric.name == name)
                    .map(|metric| metric.cumulative.clone())
            };
            use pse_diagnostics::observation::Value;
            for stage in ["jet", "extraction", "first_rhs"] {
                assert_eq!(
                    count(&format!("implicit.derivatives.{stage}.calls")),
                    Some(Value::Count(1))
                );
                assert_eq!(
                    count(&format!("implicit.derivatives.{stage}.completed")),
                    Some(Value::Count(1))
                );
            }
            assert_eq!(
                count("implicit.factor.numeric.completed"),
                Some(Value::Count(1))
            );
            let solves = if order == DerivativeOrder::Second {
                2
            } else {
                1
            };
            assert_eq!(
                count("implicit.factor.solve.completed"),
                Some(Value::Count(solves))
            );
            assert_eq!(
                count("implicit.derivatives.verification.completed"),
                Some(Value::Count(solves))
            );
            assert_eq!(
                count("implicit.derivatives.expansion.completed"),
                Some(Value::Count(solves))
            );
            for stage in ["second_rhs", "contraction"] {
                assert_eq!(
                    count(&format!("implicit.derivatives.{stage}.completed")),
                    (order == DerivativeOrder::Second).then_some(Value::Count(1))
                );
            }
            assert_eq!(actual.values, [15.]);
            assert!((actual.jacobian[0] - 8.).abs() < 1e-12);
            if order == DerivativeOrder::Second {
                assert!((actual.hessians[0] - 2.).abs() < 1e-12);
            } else {
                assert!(actual.hessians.is_empty());
            }
        }
        let value = ResidualBodies {
            unknown: compile(&[], DerivativeOrder::Value),
            source: compile(&[], DerivativeOrder::Value),
        };
        value.check(1, 1).unwrap();
    }
    #[test]
    fn implicit_dual_programs_refuse_a_different_original_definition() {
        use crate::guarded::{PreparedBody, Stage};
        use symbolica::atom::Atom;
        crate::initialize().unwrap();
        let x = crate::library::formal(0).unwrap();
        let p = crate::library::formal(1).unwrap();
        let compile_control = pse_diagnostics::Cancellation::new(false);
        let compile = |value: Atom, axes: &[usize]| {
            Arc::new(
                PreparedBody::new(
                    2,
                    3,
                    vec![2],
                    vec![Stage::Block {
                        expressions: vec![value],
                        outputs: vec![2],
                        source: SemanticId::NIL,
                    }],
                    DerivativeOrder::Second,
                )
                .unwrap()
                .compile(
                    &[0],
                    axes,
                    DerivativeOrder::First,
                    crate::library::Optimization::default(),
                    crate::jets::EvaluationLimits::default(),
                    &pse_kernels::ExecutionScope::new(compile_control.clone(), None),
                )
                .unwrap(),
            )
        };
        let bodies = ResidualBodies {
            unknown: compile(&x - &p, &[0]),
            source: compile(&x + &p, &[0, 1]),
        };
        assert!(matches!(bodies.check(1, 1), Err(MathError::Contract(_))));
    }
    #[test]
    fn compact_value_inputs_preserve_unknown_and_parameter_formal_positions() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = registry.neutral_dimensionless().unwrap();
        let id = SemanticId::from_bytes([96; 16]);
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let mut builder = crate::typed::BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            64,
            crate::typed::BodyLimits::default(),
        )
        .unwrap();
        let unknown = builder
            .input(1, quantity, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let parameter = builder
            .input(47, quantity, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let output = builder
            .binary(crate::typed::Binary::Sub, unknown, parameter, None, id)
            .unwrap();
        let body = builder
            .prepare(&[output])
            .unwrap()
            .compile(
                &[0],
                &[],
                DerivativeOrder::Value,
                crate::library::Optimization::default(),
                crate::jets::EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(control.clone(), None),
            )
            .unwrap();
        let mut worker = body.worker();
        assert_eq!(worker.input_formals(), &[1, 47]);
        let mut parameters = vec![0.0; 62];
        parameters[45] = 3.0;
        let values = selected_inputs(&worker, &[91.0, 7.0], &parameters).unwrap();
        assert_eq!(values, [7.0, 3.0]);
        assert_eq!(
            worker
                .evaluate(
                    &values,
                    DerivativeOrder::Value,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel
                )
                .unwrap()
                .values,
            [4.0]
        );
        assert!(selected_inputs(&worker, &[91.0, 7.0], &[]).is_err());
    }
    #[test]
    fn compact_selection_anchor_and_restriction_use_their_formal_signatures() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = registry.neutral_dimensionless().unwrap();
        let id = SemanticId::from_bytes([98; 16]);
        let compile_control = pse_diagnostics::Cancellation::new(false);
        let cancel = compile_control.flag();
        let mut builder = crate::typed::BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            64,
            crate::typed::BodyLimits::default(),
        )
        .unwrap();
        let unknown = builder
            .input(0, quantity, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let parameter = builder
            .input(47, quantity, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let restriction = builder
            .binary(
                crate::typed::Binary::Sub,
                unknown,
                parameter.clone(),
                None,
                id,
            )
            .unwrap();
        let constant = builder
            .binary(
                crate::typed::Binary::Sub,
                parameter.clone(),
                parameter.clone(),
                None,
                id,
            )
            .unwrap();
        let source = builder
            .prepare(&[restriction, parameter, constant])
            .unwrap();
        let compile = |outputs: &[usize], axes: &[usize], order| {
            Arc::new(
                source
                    .compile(
                        outputs,
                        axes,
                        order,
                        crate::library::Optimization::default(),
                        crate::jets::EvaluationLimits::default(),
                        &pse_kernels::ExecutionScope::new(compile_control.clone(), None),
                    )
                    .unwrap(),
            )
        };
        let problem = Problem::new(
            id,
            ContentHash::from_bytes([98; 32]),
            vec![Unknown {
                id,
                lower: -10.,
                upper: 10.,
            }],
            vec![pse_ids::named_id(id, "equation")],
            63,
            residual_bodies_for_test(compile(
                &[0],
                &(0..64).collect::<Vec<_>>(),
                DerivativeOrder::First,
            )),
            128,
        )
        .unwrap();
        let anchor = compile(&[1], &[], DerivativeOrder::Value);
        let restriction = compile(&[0], &[0, 47], DerivativeOrder::First);
        assert_eq!(anchor.input_formals(), &[47]);
        assert_eq!(restriction.input_formals(), &[0, 47]);
        let mut worker = selection::SelectionWorker::new(&Selection {
            anchor: Some(anchor),
            restriction: Some(restriction),
            ..Default::default()
        });
        let mut parameters = vec![f64::NAN; 63];
        parameters[46] = 3.;
        let unknown_only = problem
            .evaluate_unknown(&parameters, &[4.], DerivativeOrder::First, &cancel)
            .unwrap();
        assert_eq!(unknown_only.values, [1.]);
        assert_eq!(
            unknown_only.jacobian,
            [1.],
            "native residual excludes all 63 parameter derivative axes"
        );
        assert!(
            !problem
                .worker
                .lock()
                .unwrap()
                .worker
                .census()
                .instantiated_orders[1]
        );
        assert!(
            problem
                .unknown_worker
                .lock()
                .unwrap()
                .worker
                .census()
                .instantiated_orders[1]
        );

        assert_eq!(
            worker.anchor(&problem, &parameters, &cancel).unwrap(),
            Some(vec![3.])
        );
        worker
            .verify(
                &problem,
                &parameters,
                &[4.],
                DerivativeOrder::First,
                &cancel,
            )
            .unwrap();
        assert!(
            worker
                .verify(
                    &problem,
                    &parameters,
                    &[4.001],
                    DerivativeOrder::First,
                    &cancel
                )
                .is_err()
        );
        assert!(worker.anchor(&problem, &[], &cancel).is_err());
        parameters[46] = f64::NAN;
        assert!(worker.anchor(&problem, &parameters, &cancel).is_err());
        let constant = compile(&[2], &[], DerivativeOrder::Value);
        assert!(constant.input_formals().is_empty());
        let mut independent = selection::SelectionWorker::new(&Selection {
            anchor: Some(constant),
            ..Default::default()
        });
        assert_eq!(
            independent.anchor(&problem, &[], &cancel).unwrap(),
            Some(vec![0.])
        );
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            independent.anchor(&problem, &[], &cancel),
            Err(MathError::Cancelled)
        ));
    }
    #[test]
    fn compact_hints_and_nominals_use_their_independent_input_signatures() {
        #[derive(Debug)]
        struct Resolver {
            unknown: Unknown,
        }
        impl HintResolver for Resolver {
            fn identity(&self) -> ContentHash {
                ContentHash::from_bytes([96; 32])
            }
            fn retained_bytes(&self) -> usize {
                128
            }
            fn time_limit(&self) -> Duration {
                Duration::from_secs(10)
            }
            fn resolve(
                &self,
                values: &[f64],
                terms: Option<&[f64]>,
                _: Option<&[f64]>,
            ) -> Result<(Vec<Unknown>, Options), MathError> {
                Ok((
                    vec![self.unknown.clone()],
                    Options {
                        start: vec![values[0]],
                        variable_nominals: vec![terms.map_or(2.0, |terms| terms[0])],
                        variable_tolerance: vec![1e-9],
                        residual_tolerance: vec![1e-9],
                        iterations: 10,
                        time_limit: self.time_limit(),
                        derivative_tolerance: 1e-10,
                    },
                ))
            }
        }
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = registry.neutral_dimensionless().unwrap();
        let id = SemanticId::from_bytes([97; 16]);
        let compile_control = pse_diagnostics::Cancellation::new(false);
        let cancel = compile_control.flag();
        let mut builder = crate::typed::BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            64,
            crate::typed::BodyLimits::default(),
        )
        .unwrap();
        let unknown = builder
            .input(0, quantity, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let parameter = builder
            .input(47, quantity, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let residual = builder
            .binary(
                crate::typed::Binary::Sub,
                unknown.clone(),
                parameter.clone(),
                None,
                id,
            )
            .unwrap();
        let nominal = builder
            .binary(
                crate::typed::Binary::Add,
                unknown,
                parameter.clone(),
                None,
                id,
            )
            .unwrap();
        let source = builder.prepare(&[residual, parameter, nominal]).unwrap();
        let compile = |outputs: &[usize], axes: &[usize], order| {
            Arc::new(
                source
                    .compile(
                        outputs,
                        axes,
                        order,
                        crate::library::Optimization::default(),
                        crate::jets::EvaluationLimits::default(),
                        &pse_kernels::ExecutionScope::new(compile_control.clone(), None),
                    )
                    .unwrap(),
            )
        };
        let unknown = Unknown {
            id,
            lower: -10.0,
            upper: 10.0,
        };
        let mut problem = Arc::new(
            Problem::new(
                id,
                ContentHash::from_bytes([97; 32]),
                vec![unknown.clone()],
                vec![pse_ids::named_id(id, "equation")],
                63,
                residual_bodies_for_test(compile(
                    &[0],
                    &(0..64).collect::<Vec<_>>(),
                    DerivativeOrder::First,
                )),
                128,
            )
            .unwrap(),
        );
        let hints = compile(&[1], &[], DerivativeOrder::Value);
        let terms = compile(&[2], &[], DerivativeOrder::Value);
        assert_eq!(hints.input_formals(), [47]);
        assert_eq!(terms.input_formals(), [0, 47]);
        let mut configuration = ConfigurationWorker::new(
            Configuration::Hints(Arc::new(Resolver { unknown })),
            Some(&hints),
            Some(&terms),
        );
        let mut parameters = vec![0.0; 63];
        parameters[46] = 3.0;
        let options = configuration
            .resolve(&mut problem, &parameters, &cancel, None)
            .unwrap();
        assert_eq!(options.start, [3.0]);
        assert_eq!(options.variable_nominals, [5.0]);
    }
    #[derive(Debug, Clone)]
    struct CountedLinear(Arc<std::sync::atomic::AtomicUsize>);
    impl InnerSolver for CountedLinear {
        fn minimum_order(&self) -> DerivativeOrder {
            DerivativeOrder::First
        }
        fn identity(&self) -> ContentHash {
            solver_identity("test.counted-linear.v1")
        }

        fn worker(&self) -> Box<dyn InnerWorker> {
            Box::new(self.clone())
        }
    }
    impl InnerWorker for CountedLinear {
        fn solve(
            &mut self,
            _: Arc<Problem>,
            parameters: &[f64],
            _: &Options,
            _: &pse_diagnostics::Cancellation,
        ) -> Result<Vec<f64>, MathError> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(vec![parameters[0]])
        }
    }
    #[test]
    fn nested_selected_products_reuse_same_point_proposal_and_invalidate_accuracy() {
        use pse_kernels::{EvaluationContext, ProviderFactory};
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let q = registry.neutral_dimensionless().unwrap();
        let unit = registry.quantity_type(q).unwrap().canonical_unit;
        let id = SemanticId::from_bytes([95; 16]);
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let mut builder = crate::typed::BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            crate::typed::BodyLimits::default(),
        )
        .unwrap();
        let y = builder
            .input(0, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let p = builder
            .input(1, q, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let residual = builder
            .binary(crate::typed::Binary::Sub, y, p, None, id)
            .unwrap();
        let body = Arc::new(
            builder
                .finish(
                    &[residual],
                    DerivativeOrder::Second,
                    crate::library::Optimization::default(),
                    &pse_kernels::ExecutionScope::new(control.clone(), None),
                )
                .unwrap(),
        );
        let unknowns = vec![Unknown {
            id,
            lower: -10.0,
            upper: 10.0,
        }];
        let options = Options {
            start: vec![1.0],
            variable_nominals: vec![1.0],
            variable_tolerance: vec![1e-9],
            residual_tolerance: vec![1e-9],
            iterations: 10,
            time_limit: Duration::from_secs(10),
            derivative_tolerance: 1e-10,
        };
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let port = |id| pse_kernels::Port {
            id,
            quantity: q,
            unit,
        };
        let factory = Factory {
            selection: Selection::default(),
            requirements: pse_kernels::DerivativeRequirements::new(
                DerivativeOrder::Second,
                DerivativeOrder::Second,
                DerivativeOrder::Second,
                DerivativeOrder::First,
                DerivativeOrder::Second,
            )
            .unwrap(),
            spec: pse_kernels::ProviderSpec {
                shapes: Default::default(),
                derivative_source: pse_kernels::DerivativeSource::Analytic,
                id,
                revision: ContentHash::from_bytes([1; 32]),
                data: ContentHash::from_bytes([2; 32]),
                inputs: vec![port(pse_ids::named_id(id, "parameter"))],
                outputs: vec![port(id)],
                derivatives: DerivativeOrder::Second,
                smoothness: DerivativeOrder::Second,
            },
            bodies: residual_bodies_for_test(body),
            unknowns: unknowns.clone(),
            rows: vec![pse_ids::named_id(id, "row")],
            configuration: Configuration::Fixed(unknowns, options.clone()),
            hints: None,
            terms: None,
            solver: Arc::new(CountedLinear(calls.clone())),
            cancel: control.clone(),
            max_entries: 100,
            providers: BTreeMap::new(),
        };
        let admitted = pse_kernels::ExecutionScope::new(control.clone(), None)
            .with_admitted_threads(std::num::NonZeroUsize::new(2).unwrap());
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap();
        assert_eq!(
            pool.install(|| factory
                .problem(admitted.clone())
                .unwrap()
                .factor
                .lock()
                .unwrap()
                .parallel),
            Par::rayon(2)
        );
        // A capacity larger than the installed pool cannot create another team.
        assert_eq!(
            pool.install(|| factory
                .problem(
                    admitted
                        .clone()
                        .with_admitted_threads(std::num::NonZeroUsize::new(4).unwrap())
                )
                .unwrap()
                .factor
                .lock()
                .unwrap()
                .parallel),
            Par::rayon(2)
        );
        assert_eq!(
            pool.install(|| factory
                .problem(admitted.nested())
                .unwrap()
                .factor
                .lock()
                .unwrap()
                .parallel),
            Par::Seq
        );
        // A fresh host thread has no installed Rayon owner. Explicit admitted width
        // must fall back to Seq without consulting or initializing the global pool.
        std::thread::scope(|threads| {
            let outside = admitted.clone();
            let factor_parallelism = threads
                .spawn(|| {
                    assert!(rayon::current_thread_index().is_none());
                    factory
                        .problem(outside)
                        .unwrap()
                        .factor
                        .lock()
                        .unwrap()
                        .parallel
                })
                .join()
                .unwrap();
            assert_eq!(factor_parallelism, Par::Seq);
        });
        let serial_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap();
        assert_eq!(
            serial_pool.install(|| factory
                .problem(admitted.clone())
                .unwrap()
                .factor
                .lock()
                .unwrap()
                .parallel),
            Par::Seq
        );
        let mut provider = factory.create().unwrap();
        let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
        let operation = hub.start("implicit.ift.reuse_stage");
        let _entered = operation.context().enter();
        let context = EvaluationContext {
            cancelled: cancel.as_ref(),
            max_result_bytes: 1024 * 1024,
        };
        for order in [
            DerivativeOrder::Value,
            DerivativeOrder::First,
            DerivativeOrder::Second,
            DerivativeOrder::First,
        ] {
            let result = provider_values(&mut provider, 
                    &[2.0],
                    &pse_kernels::ProviderRequest::all(&factory.spec, order),
                    &context,
                )
                .unwrap();
            assert_eq!(result.values, [2.0]);
            if order >= DerivativeOrder::First {
                assert_eq!(result.jacobian, [1.0]);
            }
            if order >= DerivativeOrder::Second {
                assert_eq!(result.hessians, [0.0]);
            }
        }
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        let snapshot = operation.reader().snapshot(Duration::from_secs(60));
        let count = |name: &str| {
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == name)
                .map(|metric| metric.cumulative.clone())
        };
        use pse_diagnostics::observation::Value;
        assert_eq!(
            count("implicit.derivatives.jet.completed"),
            Some(Value::Count(2))
        );
        assert_eq!(
            count("implicit.derivatives.contraction.completed"),
            Some(Value::Count(1))
        );
        assert_eq!(count("implicit.product.full_reuse"), Some(Value::Count(1)));
        assert_eq!(
            count("implicit.product.order_projection"),
            Some(Value::Count(1))
        );
        provider_values(&mut provider, 
                &[2.1],
                &pse_kernels::ProviderRequest::all(&factory.spec, DerivativeOrder::First),
                &context,
            )
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 2);
        // Invalid trial failure must clear carried numerical validity, even if a
        // following trial returns to the last accepted input bit pattern.
        assert!(
            provider_values(&mut provider, 
                    &[11.0],
                    &pse_kernels::ProviderRequest::all(&factory.spec, DerivativeOrder::Value),
                    &context,
                )
                .is_err()
        );
        provider_values(&mut provider, 
                &[2.1],
                &pse_kernels::ProviderRequest::all(&factory.spec, DerivativeOrder::First),
                &context,
            )
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 4);
        for value in [0.0, -0.0] {
            provider_values(&mut provider, 
                    &[value],
                    &pse_kernels::ProviderRequest::all(&factory.spec, DerivativeOrder::Value),
                    &context,
                )
                .unwrap();
        }
        assert_eq!(calls.load(Ordering::Relaxed), 6);
        let problem = factory
            .problem(pse_kernels::ExecutionScope::new(control.clone(), None))
            .unwrap();
        let key = numerical_product_key(&problem, &[2.0], &options, factory.solver.identity());
        let mut tighter = options;
        tighter.derivative_tolerance *= 0.5;
        assert_ne!(
            key,
            numerical_product_key(&problem, &[2.0], &tighter, factory.solver.identity())
        );
    }
    #[test]
    fn implicit_ift_second_derivatives_and_singular_refusal() {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let id = SemanticId::from_bytes([93; 16]);
        let quantity = registry.neutral_dimensionless().unwrap();
        let mut builder = crate::typed::BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            crate::typed::BodyLimits::default(),
        )
        .unwrap();
        let y = builder
            .input(0, quantity, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let p = builder
            .input(1, quantity, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let square = builder
            .binary(crate::typed::Binary::Mul, y.clone(), y, None, id)
            .unwrap();
        let residual = builder
            .binary(crate::typed::Binary::Sub, square, p, None, id)
            .unwrap();
        let control = pse_diagnostics::Cancellation::new(false);
        let cancel = control.flag();
        let body = Arc::new(
            builder
                .finish(
                    &[residual],
                    DerivativeOrder::Second,
                    crate::library::Optimization::default(),
                    &pse_kernels::ExecutionScope::new(control.clone(), None),
                )
                .unwrap(),
        );
        // Explicit operation-local admission reaches scratch, numeric factorization
        // and both IFT solve orders. Ordinary Problem::new remains serial.
        let mut problem = Problem::new_with_parallelism(
            id,
            ContentHash::from_bytes([0; 32]),
            vec![Unknown {
                id,
                lower: 0.0,
                upper: 3.0,
            }],
            vec![pse_ids::named_id(id, "equation")],
            1,
            residual_bodies_for_test(body),
            100,
            Par::rayon(2),
        )
        .unwrap();
        assert_eq!(problem.factor.lock().unwrap().parallel, Par::rayon(2));
        let options = Options {
            start: vec![1.0],
            variable_nominals: vec![2.0],
            variable_tolerance: vec![1e-9],
            residual_tolerance: vec![1e-10],
            iterations: 30,
            time_limit: Duration::from_secs(1),
            derivative_tolerance: 1e-10,
        };
        let jet = problem
            .derivatives(&[4.0], &[2.0], DerivativeOrder::Second, &options, &cancel)
            .unwrap();
        assert_eq!(jet.values, vec![2.0]);
        assert_eq!(jet.jacobian, vec![0.25]);
        assert_eq!(jet.hessians, vec![-0.03125]);
        let scratch = problem.factor.lock().unwrap().memory.as_ptr();
        let current = problem
            .derivatives(&[1.], &[1.], DerivativeOrder::Second, &options, &cancel)
            .unwrap();
        assert_eq!(current.jacobian, [0.5]);
        assert_eq!(current.hessians, [-0.25]);
        assert_eq!(problem.factor.lock().unwrap().memory.as_ptr(), scratch);
        // The root remains an original feasible value at an interval endpoint,
        // but full local derivatives require an interior at the physical budget.
        assert_eq!(
            problem
                .derivatives(&[9.], &[3.], DerivativeOrder::Value, &options, &cancel)
                .unwrap()
                .values,
            vec![3.]
        );
        for order in [DerivativeOrder::First, DerivativeOrder::Second] {
            assert!(matches!(
                problem.derivatives(&[9.], &[3.], order, &options, &cancel),
                Err(MathError::Domain {
                    requirement: "implicit derivative neighborhood meets a declared unknown bound",
                    ..
                })
            ));
        }
        // Exercise singularity separately from the newly enforced bound interior.
        problem.unknowns[0].lower = -1.;
        {
            let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
            let operation = hub.start("implicit.ift.singular_stage");
            let _entered = operation.context().enter();
            assert!(matches!(
                problem.derivatives(&[0.0], &[0.0], DerivativeOrder::Second, &options, &cancel),
                Err(MathError::Domain {
                    requirement: "implicit derivative residual verification failed",
                    ..
                })
            ));
            let snapshot = operation.reader().snapshot(Duration::from_secs(60));
            let count = |name: &str| {
                snapshot
                    .metrics
                    .iter()
                    .find(|metric| metric.name == name)
                    .map(|metric| metric.cumulative.clone())
            };
            use pse_diagnostics::observation::Value;
            assert_eq!(
                count("implicit.derivatives.extraction.completed"),
                Some(Value::Count(1))
            );
            // faer's sparse numeric LU accepts this structurally present zero pivot.
            // Its factor and backsolve calls return; the authoritative derivative residual
            // verification then refuses the nonfinite solve. Report those actual stages,
            // rather than inventing a library factorization error for numerical singularity.
            assert_eq!(count("implicit.factor.numeric.failures"), None);
            assert_eq!(
                count("implicit.factor.numeric.completed"),
                Some(Value::Count(1))
            );
            assert_eq!(
                count("implicit.derivatives.first_rhs.completed"),
                Some(Value::Count(1))
            );
            assert_eq!(
                count("implicit.factor.solve.completed"),
                Some(Value::Count(1))
            );
            assert_eq!(
                count("implicit.derivatives.verification.failures"),
                Some(Value::Count(1))
            );
            assert_eq!(count("implicit.derivatives.verification.completed"), None);
            assert_eq!(count("implicit.derivatives.second_rhs.calls"), None);
            assert_eq!(count("implicit.derivatives.expansion.calls"), None);
        }
        problem.unknowns[0].lower = 0.;
        let recovered = problem
            .derivatives(&[4.], &[2.], DerivativeOrder::Second, &options, &cancel)
            .unwrap();
        assert_eq!(recovered.jacobian, [0.25]);
        assert_eq!(recovered.hessians, [-0.03125]);
        assert_eq!(problem.factor.lock().unwrap().memory.as_ptr(), scratch);
        assert!(
            problem
                .derivatives(&[4.0], &[1.0], DerivativeOrder::First, &options, &cancel)
                .is_err()
        );
        assert!(
            problem
                .evaluate(&[16.0], &[4.0], DerivativeOrder::Value, &cancel)
                .is_err()
        );
        control.cancel();
        assert!(matches!(
            problem.evaluate(&[4.0], &[2.0], DerivativeOrder::Value, &cancel),
            Err(MathError::Cancelled)
        ));
        {
            let hub = pse_diagnostics::observation::ObservationHub::new(Default::default());
            let operation = hub.start("implicit.ift.cancelled_stage");
            let _entered = operation.context().enter();
            assert!(matches!(
                problem.derivatives_at_verified_point(
                    &[4.0],
                    &[2.0],
                    DerivativeOrder::Second,
                    &options,
                    &cancel,
                    ContentHash::from_bytes([0; 32]),
                ),
                Err(MathError::Cancelled)
            ));
            let snapshot = operation.reader().snapshot(Duration::from_secs(60));
            let count = |name: &str| {
                snapshot
                    .metrics
                    .iter()
                    .find(|metric| metric.name == name)
                    .map(|metric| metric.cumulative.clone())
            };
            use pse_diagnostics::observation::Value;
            assert_eq!(
                count("implicit.derivatives.jet.calls"),
                Some(Value::Count(1))
            );
            assert_eq!(
                count("implicit.derivatives.jet.failures"),
                Some(Value::Count(1))
            );
            assert_eq!(count("implicit.derivatives.jet.completed"), None);
            assert_eq!(count("implicit.derivatives.extraction.calls"), None);
            assert_eq!(count("implicit.factor.solve.calls"), None);
        }
    }
}
