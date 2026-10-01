// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Nested residual evaluation and implicit-function derivatives. Native solvers own iteration.
#[path = "implicit_accelerators.rs"]
pub mod accelerators;
#[path = "implicit_affine.rs"]
mod affine;
#[path = "implicit_configuration.rs"]
mod configuration;
#[path = "implicit_cubic.rs"]
mod cubic;
#[path = "implicit_regimes.rs"]
mod regimes;
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
    Mat,
    linalg::solvers::Solve,
    sparse::{
        SparseColMat, SymbolicSparseColMat,
        linalg::solvers::{Lu, SymbolicLu},
    },
};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{DerivativeOrder, ProviderValues};
pub use regimes::{Regime, RegimeFactory, RegimeFactoryBranch, RegimeSelection, SelectedRegime};
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
#[derive(Clone, Debug)]
pub struct Unknown {
    /// Semantic identity of the unknown.
    pub id: SemanticId,
    /// Lower end of the closed physical interval.
    pub lower: f64,
    /// Upper end of the closed physical interval.
    pub upper: f64,
}
/// Explicit bounded native solve and derivative verification controls.
#[derive(Clone, Debug)]
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
    /// Actual compiled residual order shared with the native oracle.
    pub compiled_order: DerivativeOrder,
    /// Resolved derivative facts shared with provider and native contracts.
    pub requirements: pse_kernels::DerivativeRequirements,
    worker: Mutex<Worker>,
    providers: Mutex<BTreeMap<pse_kernels::ProviderKey, Box<dyn pse_kernels::Provider>>>,
    pattern: SymbolicSparseColMat<usize>,
    symbolic: SymbolicLu<usize>,
}
impl Problem {
    /// Bind compiled residuals with their library-owned structural Jacobian support.
    pub fn new(
        id: SemanticId,
        identity: ContentHash,
        unknowns: Vec<Unknown>,
        rows: Vec<SemanticId>,
        inputs: usize,
        body: Arc<CompiledBody>,
        max_entries: usize,
    ) -> Result<Self, MathError> {
        let n = unknowns.len();
        let extent = n
            .checked_add(inputs)
            .and_then(|width| match body.compiled_order() {
                DerivativeOrder::Value => Some(n),
                DerivativeOrder::First => n.checked_mul(width),
                DerivativeOrder::Second => n.checked_mul(width).and_then(|v| v.checked_mul(width)),
            });
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
        if body.support().first.len() != n {
            return Err(MathError::Contract(
                "implicit residual support extent".into(),
            ));
        }
        let indices = body
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
        let symbolic =
            SymbolicLu::try_new(pattern.as_ref()).map_err(|e| MathError::Library(e.to_string()))?;
        Ok(Self {
            id,
            identity,
            unknowns,
            rows,
            inputs,
            compiled_order: body.compiled_order(),
            requirements: pse_kernels::DerivativeRequirements::new(
                body.compiled_order(),
                body.compiled_order(),
                body.compiled_order(),
                DerivativeOrder::Value,
                body.compiled_order(),
            )
            .map_err(|e| MathError::Contract(e.to_string()))?,
            worker: Mutex::new(body.worker()),
            providers: Mutex::new(BTreeMap::new()),
            pattern,
            symbolic,
        })
    }
    /// Attach admitted external workers to this residual worker; ownership is attempt-local.
    pub fn with_providers(
        mut self,
        providers: BTreeMap<pse_kernels::ProviderKey, Box<dyn pse_kernels::Provider>>,
    ) -> Self {
        self.providers = Mutex::new(providers);
        self
    }
    /// Attach the compiler's resolved selected-function requirements to its residual problem.
    pub fn with_requirements(
        mut self,
        requirements: pse_kernels::DerivativeRequirements,
    ) -> Result<Self, MathError> {
        if self.compiled_order != requirements.residual_compilation {
            return Err(MathError::Contract(
                "implicit requirements compilation order mismatch".into(),
            ));
        }
        self.requirements = requirements;
        Ok(self)
    }
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
        let values = unknowns
            .iter()
            .chain(parameters)
            .copied()
            .collect::<Vec<_>>();
        self.worker
            .lock()
            .map_err(|_| MathError::Library("implicit worker lock poisoned".into()))?
            .evaluate(
                &values,
                order,
                &mut *self
                    .providers
                    .lock()
                    .map_err(|_| MathError::Library("implicit provider lock poisoned".into()))?,
                cancel,
            )
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
        self.validate_options(options)?;
        let residual = self.evaluate(parameters, point, DerivativeOrder::Value, cancel)?;
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
        self.verify(parameters, point, options, cancel)?;
        let n = self.unknowns.len();
        let p = self.inputs;
        let width = n + p;
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
        if p == 0 {
            return Ok(output);
        }
        let jet = self.evaluate(parameters, point, order, cancel)?;
        if jet.jacobian.len() != n * width
            || order == DerivativeOrder::Second && jet.hessians.len() != n * width * width
        {
            return Err(MathError::Contract("implicit residual jet extent".into()));
        }
        let values = (0..n)
            .flat_map(|j| self.pattern.as_ref().row_idx_of_col(j).map(move |i| (i, j)))
            .map(|(i, j)| jet.jacobian[i * width + j])
            .collect();
        let matrix = SparseColMat::new(self.pattern.clone(), values);
        let lu =
            Lu::try_new_with_symbolic(self.symbolic.clone(), matrix.as_ref()).map_err(|_| {
                MathError::Domain {
                    source_id: self.id,
                    requirement: "implicit Jacobian is singular",
                }
            })?;
        let mut first = Mat::from_fn(n, p, |i, j| -jet.jacobian[i * width + n + j]);
        let original = first.clone();
        lu.solve_in_place(first.as_mut());
        check_solve(
            &jet.jacobian,
            width,
            &first,
            &original,
            options.derivative_tolerance,
            self.id,
        )?;
        output.jacobian = (0..n)
            .flat_map(|i| (0..p).map(move |j| (i, j)))
            .map(|(i, j)| first[(i, j)])
            .collect();
        if order == DerivativeOrder::First {
            return Ok(output);
        }
        let pairs = (0..p)
            .flat_map(|a| (a..p).map(move |b| (a, b)))
            .collect::<Vec<_>>();
        let tangent = |coordinate: usize, input: usize| {
            if coordinate < n {
                first[(coordinate, input)]
            } else {
                f64::from(coordinate == n + input)
            }
        };
        let mut second = Mat::from_fn(n, pairs.len(), |i, j| {
            let (a, b) = pairs[j];
            let mut value = 0.0;
            for k in 0..width {
                for l in 0..width {
                    value -=
                        jet.hessians[(i * width + k) * width + l] * tangent(k, a) * tangent(l, b);
                }
            }
            value
        });
        let original = second.clone();
        lu.solve_in_place(second.as_mut());
        check_solve(
            &jet.jacobian,
            width,
            &second,
            &original,
            options.derivative_tolerance,
            self.id,
        )?;
        output.hessians = vec![0.0; n * p * p];
        for (j, (a, b)) in pairs.into_iter().enumerate() {
            for i in 0..n {
                output.hessians[(i * p + a) * p + b] = second[(i, j)];
                output.hessians[(i * p + b) * p + a] = second[(i, j)];
            }
        }
        Ok(output)
    }
}
fn check_solve(
    jacobian: &[f64],
    width: usize,
    x: &Mat<f64>,
    rhs: &Mat<f64>,
    tolerance: f64,
    id: SemanticId,
) -> Result<(), MathError> {
    for column in 0..x.ncols() {
        for row in 0..x.nrows() {
            let mut sum = 0.0;
            let mut scale = rhs[(row, column)].abs();
            for k in 0..x.nrows() {
                let value = jacobian[row * width + k] * x[(k, column)];
                sum += value;
                scale += value.abs();
            }
            let error = (sum - rhs[(row, column)]).abs();
            if !error.is_finite() || error > tolerance * scale.max(f64::MIN_POSITIVE) {
                return Err(MathError::Domain {
                    source_id: id,
                    requirement: "implicit derivative residual verification failed",
                });
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
    /// Solve for the unknowns at the given parameters, within the options' budgets.
    fn solve(
        &self,
        problem: Arc<Problem>,
        parameters: &[f64],
        options: &Options,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Vec<f64>, MathError>;
}

/// Frame an implementation's explicitly versioned capability name.
pub fn solver_identity(reference: &str) -> ContentHash {
    let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::InnerSolverV1);
    identity.str(reference);
    identity.finish_hash()
}

/// Attempt-bound provider construction, using the same callback and cancellation owner
/// as outer math evaluation. The factory never acquires runtime CPU permits.
#[derive(Debug)]
pub struct Factory {
    /// Compiler-issued semantic choice, independent of ordinary numerical starts.
    pub selection: Selection,
    /// Shared capabilities and resolved demand.
    pub requirements: pse_kernels::DerivativeRequirements,
    /// Provider contract: parameters in, solved unknowns out.
    pub spec: pse_kernels::ProviderSpec,
    /// Compiled residual body, unknowns first.
    pub body: Arc<CompiledBody>,
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
    pub cancel: Arc<AtomicBool>,
    /// Bound on implicit derivative entries.
    pub max_entries: usize,
    /// External provider registrations the residual body calls.
    pub providers: BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
}
/// A single residual or an explicit alternative selector; both create attempt-local workers.
#[derive(Debug)]
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
        let root = |factory: &Factory| {
            factory
                .body
                .retained_bytes()
                .checked_add(factory.body.scratch_bytes())?
                .checked_add(factory.max_entries.checked_mul(8)?)
                .and_then(|n| n.checked_add(factory.configuration.retained_bytes()))
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
        };
        match self {
            Self::Root(factory) => root(factory),
            Self::Regimes(factory) => {
                factory
                    .alternatives
                    .iter()
                    .try_fold(0usize, |bytes, branch| {
                        bytes
                            .checked_add(root(&branch.residual)?)?
                            .checked_add(branch.eligibility.retained_bytes())?
                            .checked_add(branch.criterion.retained_bytes())
                    })
            }
        }
        .ok_or(MathError::Limit("implicit factory retained extent"))
    }
    /// Preserve the runtime's admission lease through every separately compiled program.
    pub fn retain(&mut self, owner: Arc<dyn crate::AllocationOwner>) {
        let retain = |body: &mut Arc<CompiledBody>| {
            *body = Arc::new(body.as_ref().clone().with_owner(owner.clone()))
        };
        match self {
            Self::Root(factory) => {
                retain(&mut factory.body);
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
                    retain(&mut branch.residual.body);
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
    fn create(&self) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        match self {
            Self::Root(v) => v.create(),
            Self::Regimes(v) => v.create(),
        }
    }
    fn create_scoped(
        &self,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        match self {
            Self::Root(v) => v.create_scoped(cancel),
            Self::Regimes(v) => v.create_scoped(cancel),
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
    fn create(&self) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        self.create_scoped(self.cancel.clone())
    }
    fn create_scoped(
        &self,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        let problem = self.problem(cancel.clone())?;
        Ok(Box::new(Nested {
            spec: self.spec.clone(),
            problem,
            configuration: ConfigurationWorker::new(
                self.configuration.clone(),
                self.hints.as_ref(),
                self.terms.as_ref(),
            ),
            solver: self.solver.clone(),
            cancel,
            selection: selection::SelectionWorker::new(&self.selection),
            requested_output: self.requirements.requested_output,
        }))
    }
}
impl Factory {
    fn problem(&self, cancel: Arc<AtomicBool>) -> Result<Arc<Problem>, pse_kernels::ProviderError> {
        if self.requirements.inner_minimum != self.solver.minimum_order()
            || self.body.compiled_order() != self.requirements.residual_compilation
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
            .map(|(k, r)| r.worker_scoped(cancel.clone()).map(|w| (*k, w)))
            .collect::<Result<_, _>>()?;
        let problem = Arc::new(
            Problem::new(
                self.spec.id,
                self.spec.identity(),
                self.unknowns.clone(),
                self.rows.clone(),
                self.spec.inputs.len(),
                self.body.clone(),
                self.max_entries,
            )
            .map_err(provider_error)?
            .with_requirements(self.requirements)
            .map_err(provider_error)?
            .with_providers(providers),
        );
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
    requested_output: DerivativeOrder,
    selection: selection::SelectionWorker,
    spec: pse_kernels::ProviderSpec,
    problem: Arc<Problem>,
    configuration: ConfigurationWorker,
    solver: Arc<dyn InnerSolver>,
    cancel: Arc<AtomicBool>,
}
impl pse_kernels::Provider for Nested {
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        &self.spec
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &pse_kernels::ProviderRequest,
        context: &pse_kernels::EvaluationContext<'_>,
    ) -> Result<ProviderValues, pse_kernels::ProviderError> {
        request.validate(&self.spec, context)?;
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
        let started = std::time::Instant::now();
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
        options.time_limit = allowance.saturating_sub(started.elapsed());
        if options.time_limit.is_zero() {
            return Err(pse_kernels::ProviderError::Limit(
                "implicit configuration time",
            ));
        }
        let point = self
            .solver
            .solve(self.problem.clone(), inputs, &options, &self.cancel)
            .map_err(provider_error)?;
        let all = self
            .problem
            .derivatives(inputs, &point, request.order, &options, &self.cancel)
            .map_err(provider_error)?;
        self.selection
            .verify(&self.problem, inputs, &point, request.order, &self.cancel)
            .map_err(provider_error)?;
        let n = inputs.len();
        if started.elapsed() >= allowance {
            return Err(pse_kernels::ProviderError::Limit(
                "implicit evaluation time",
            ));
        }
        let mut out = ProviderValues {
            values: Vec::new(),
            jacobian: Vec::new(),
            hessians: Vec::new(),
        };
        for i in &request.outputs {
            out.values.push(all.values[*i]);
            if request.order >= DerivativeOrder::First {
                out.jacobian
                    .extend_from_slice(&all.jacobian[i * n..(i + 1) * n]);
            }
            if request.order >= DerivativeOrder::Second {
                out.hessians
                    .extend_from_slice(&all.hessians[i * n * n..(i + 1) * n * n]);
            }
        }
        out.validate(&self.spec, request)?;
        Ok(out)
    }
}
fn provider_error(error: MathError) -> pse_kernels::ProviderError {
    match error {
        MathError::Cancelled => pse_kernels::ProviderError::Cancelled,
        MathError::Limit(limit) => pse_kernels::ProviderError::Limit(limit),
        MathError::Domain { requirement, .. } => {
            pse_kernels::ProviderError::Trial(requirement.into())
        }
        error @ (MathError::OutsideRange { .. } | MathError::Validity(_)) => {
            pse_kernels::ProviderError::Trial(error.to_string())
        }
        MathError::Provider { cause, .. } => cause,
        other => pse_kernels::ProviderError::Terminal(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let cancel = Arc::new(AtomicBool::new(false));
        let body = Arc::new(
            builder
                .finish(
                    &[residual],
                    DerivativeOrder::Second,
                    crate::library::Optimization::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let mut problem = Problem::new(
            id,
            ContentHash::from_bytes([0; 32]),
            vec![Unknown {
                id,
                lower: 0.0,
                upper: 3.0,
            }],
            vec![pse_ids::named_id(id, "equation")],
            1,
            body,
            100,
        )
        .unwrap();
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
        assert!(
            problem
                .derivatives(&[0.0], &[0.0], DerivativeOrder::Second, &options, &cancel)
                .is_err()
        );
        problem.unknowns[0].lower = 0.;
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
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            problem.evaluate(&[4.0], &[2.0], DerivativeOrder::Value, &cancel),
            Err(MathError::Cancelled)
        ));
    }
}
