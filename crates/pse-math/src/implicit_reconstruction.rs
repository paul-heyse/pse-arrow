// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Concrete reduced reconstruction over the existing selected implicit worker.
use crate::{
    MathError,
    derived::{
        Coordinate, DerivativeSupport, OriginalContract, ReconstructionAdmission,
        ReconstructionContract, ReconstructionObservation, ReconstructionOracle,
        ReconstructionProducer,
    },
    implicit::{RegimeFactory, RegimeSelection, RootActionEvidence, SelectionProofRefusal},
    index::{Entry, GlobalCol, GlobalRow},
    normalization::Normalization,
};
use pse_ids::{ContentHash, FramedHasher};
use pse_kernels::{DerivativeOrder, ExecutionScope, ProviderFactory};
use pse_model::strategy::{AccuracyClass, AccuracyDemand, AccuracyEvidence};
use std::{
    collections::BTreeSet,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Checked realization of named authored equality values as zero residuals.
/// The factory's named rows compute authored row values; every selected alternative
/// receives the identical subtraction in numerical evaluation and verifier projection.
#[derive(Clone, Debug, PartialEq)]
pub enum SelectedResidualRealization {
    /// Actual factory outputs are authored row values; subtract checked RHS values.
    AuthoredValues,
    /// Actual compiler producer already subtracted the named authored RHS values.
    /// Every declared offset is checked mechanically against the original row bounds.
    ZeroResiduals {
        /// Named RHS values already subtracted by the actual residual producer.
        authored_offsets: Vec<(pse_ids::SemanticId, f64)>,
    },
}
/// Producer-owned reconstruction interpretation of an actual implicit provider.
/// The ordinary provider continues to execute the unchanged Root/Regimes factory.
#[derive(Clone, Debug)]
pub struct ReconstructionFactory {
    factory: super::ImplicitFactory,
    realization: SelectedResidualRealization,
    root_isolation: Option<Arc<crate::factorable::RootIsolationProgram>>,
    root_verifier: Option<Arc<dyn super::SelectionVerifier>>,
}
impl ReconstructionFactory {
    /// Bind producer-issued source interpretation and optional exact single-root proof.
    pub fn new(
        factory: super::ImplicitFactory,
        realization: SelectedResidualRealization,
        root_isolation: Option<Arc<crate::factorable::RootIsolationProgram>>,
        root_verifier: Option<Arc<dyn super::SelectionVerifier>>,
    ) -> Self {
        let (root_isolation, root_verifier) = if matches!(factory, super::ImplicitFactory::Root(_))
        {
            (root_isolation, root_verifier)
        } else {
            (None, None)
        };
        Self {
            factory,
            realization,
            root_isolation,
            root_verifier,
        }
    }
    /// Bind an actual regime source with its producer-issued residual interpretation.
    pub fn regimes(factory: RegimeFactory, realization: SelectedResidualRealization) -> Self {
        Self::new(
            super::ImplicitFactory::Regimes(factory),
            realization,
            None,
            None,
        )
    }
    /// Actual numerical factory, never a manufactured one-alternative regime.
    pub fn factory(&self) -> &super::ImplicitFactory {
        &self.factory
    }
    /// Producer-issued interpretation, checked again against original named rows.
    pub fn realization(&self) -> &SelectedResidualRealization {
        &self.realization
    }
    /// Whether original-coordinate regularity/accuracy evidence has a real producer.
    pub fn supports_reconstruction(&self) -> bool {
        self.spec().derivatives >= DerivativeOrder::First
            && self.spec().smoothness >= DerivativeOrder::First
            && self
                .residuals()
                .iter()
                .all(|r| r.requirements.requested_output >= DerivativeOrder::First)
            && match &self.factory {
                super::ImplicitFactory::Root(_) => {
                    self.root_isolation.is_some() && self.root_verifier.is_some()
                }
                super::ImplicitFactory::Regimes(f) => {
                    f.verifier.is_some() && f.alternatives.iter().all(|b| b.isolation.is_some())
                }
            }
    }
    /// Immutable numerical source and producer-owned proof metadata. An ordinary
    /// provider invokes the numerical source without allocating reconstruction proof work.
    pub fn retained_bytes(&self) -> Result<usize, MathError> {
        let offsets = match &self.realization {
            SelectedResidualRealization::AuthoredValues => 0,
            SelectedResidualRealization::ZeroResiduals { authored_offsets } => {
                authored_offsets.capacity() * size_of::<(pse_ids::SemanticId, f64)>()
            }
        };
        self.factory
            .retained_bytes()?
            .checked_add(size_of::<Self>() + offsets)
            .and_then(|bytes| {
                bytes.checked_add(
                    self.root_isolation
                        .as_ref()
                        .map_or(0, |program| program.retained_bytes()),
                )
            })
            .ok_or(MathError::Limit("reconstruction source metadata extent"))
    }
    /// Bind the producer's hint-based domain to the complete original case. Every
    /// later trial must resolve precisely that frozen box before proof is consumed.
    pub fn bind_original_bounds(&self, unknowns: Vec<super::Unknown>) -> Result<Self, MathError> {
        fn bind(
            factory: &mut super::Factory,
            unknowns: &[super::Unknown],
        ) -> Result<(), MathError> {
            if factory.unknowns.len() != unknowns.len()
                || factory.unknowns.iter().zip(unknowns).any(|(a, b)| {
                    a.id != b.id || b.lower.is_nan() || b.upper.is_nan() || b.lower > b.upper
                })
            {
                return Err(MathError::Contract(
                    "original supplier bound identity/extent".into(),
                ));
            }
            match &factory.configuration {
                super::Configuration::Hints(source) => {
                    factory.configuration =
                        super::Configuration::Hints(Arc::new(OriginalBoundsHints {
                            source: source.clone(),
                            unknowns: unknowns.to_vec(),
                        }));
                }
                super::Configuration::Fixed(existing, _)
                    if existing.iter().zip(unknowns).any(|(a, b)| {
                        a.lower.to_bits() != b.lower.to_bits()
                            || a.upper.to_bits() != b.upper.to_bits()
                    }) =>
                {
                    return Err(MathError::Contract(
                        "fixed supplier domain differs from original".into(),
                    ));
                }
                super::Configuration::Fixed(_, _) => {}
            }
            factory.unknowns = unknowns.to_vec();
            Ok(())
        }
        let mut source = self.clone();
        match &mut source.factory {
            super::ImplicitFactory::Root(factory) => bind(factory, &unknowns)?,
            super::ImplicitFactory::Regimes(factory) => {
                for branch in &mut factory.alternatives {
                    bind(&mut branch.residual, &unknowns)?;
                }
            }
        }
        Ok(source)
    }
    /// Native proof storage admitted only by an actual reconstruction operation.
    pub fn reconstruction_workspace_bytes(&self, max_cells: u64) -> Result<usize, MathError> {
        let (Some(verifier), Some(program)) = (&self.root_verifier, &self.root_isolation) else {
            return Ok(0);
        };
        let roots = self.residuals();
        let root = roots
            .first()
            .ok_or_else(|| MathError::Contract("root proof source absent".into()))?;
        let certificate = size_of::<super::SelectionChart>()
            + size_of::<super::SelectionScope>()
            + root.spec.inputs.len() * size_of::<super::ProofInterval>()
            + root.unknowns.len()
                * (2 * size_of::<super::ProofInterval>() + size_of::<super::Unknown>());
        // Nonzero original equalities add one constant and sum to each exact
        // residual; admit that same transport extent before binding any offsets.
        let mut proof_program = program.as_ref().clone();
        for residual in &mut proof_program.residuals {
            let constant = proof_program.nodes.len();
            proof_program.nodes.push(crate::factorable::Node::Const(
                crate::factorable::Constant::Float(0.0),
            ));
            let difference = proof_program.nodes.len();
            proof_program
                .nodes
                .push(crate::factorable::Node::Sum(vec![*residual, constant]));
            *residual = difference;
        }
        let proof_program = Arc::new(proof_program);
        verifier
            .bounded_workspace_bytes(std::slice::from_ref(&proof_program), max_cells)?
            .checked_add(proof_program.retained_bytes())
            .and_then(|bytes| bytes.checked_add(2 * certificate))
            .and_then(|bytes| {
                bytes.checked_add(
                    size_of::<RootSelection>() + root.spec.inputs.len() * size_of::<f64>(),
                )
            })
            .ok_or(MathError::Limit("root reconstruction workspace extent"))
    }
    /// Preserve admission leases through the numerical producer's compiled programs.
    pub fn retain(&mut self, owner: Arc<dyn crate::AllocationOwner>) {
        self.factory.retain(owner);
    }
    /// Bind dependencies in the actual numerical producer graph.
    pub fn set_providers(
        &mut self,
        providers: std::collections::BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
    ) {
        self.factory.set_providers(providers);
    }
}
#[derive(Debug)]
struct OriginalBoundsHints {
    source: Arc<dyn super::HintResolver>,
    unknowns: Vec<super::Unknown>,
}
impl super::HintResolver for OriginalBoundsHints {
    fn identity(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV2);
        h.str("original-case-bound-supplier-hints")
            .hash(&self.source.identity());
        for u in &self.unknowns {
            h.id(&u.id).f64(u.lower).f64(u.upper);
        }
        h.finish_hash()
    }
    fn retained_bytes(&self) -> usize {
        self.source.retained_bytes()
            + size_of::<Self>()
            + self.unknowns.capacity() * size_of::<super::Unknown>()
    }
    fn time_limit(&self) -> std::time::Duration {
        self.source.time_limit()
    }
    fn resolve(
        &self,
        values: &[f64],
        terms: Option<&[f64]>,
        anchor: Option<&[f64]>,
    ) -> Result<(Vec<super::Unknown>, super::Options), MathError> {
        let (unknowns, options) = self.source.resolve(values, terms, anchor)?;
        if unknowns.len() != self.unknowns.len()
            || unknowns.iter().zip(&self.unknowns).any(|(a, b)| {
                a.id != b.id
                    || a.lower.to_bits() != b.lower.to_bits()
                    || a.upper.to_bits() != b.upper.to_bits()
            })
        {
            return Err(MathError::Contract(
                "supplier trial domain differs from frozen original case".into(),
            ));
        }
        Ok((unknowns, options))
    }
}
impl ProviderFactory for ReconstructionFactory {
    fn source_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn spec(&self) -> &pse_kernels::ProviderSpec {
        self.factory.spec()
    }
    fn configuration_key(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("producer-issued-reconstruction-source")
            .hash(&self.factory.configuration_key());
        match &self.realization {
            SelectedResidualRealization::AuthoredValues => {
                h.str("authored-values");
            }
            SelectedResidualRealization::ZeroResiduals { authored_offsets } => {
                h.str("zero-residuals").u64(authored_offsets.len() as u64);
                for (id, value) in authored_offsets {
                    h.id(id).f64(*value);
                }
            }
        }
        h.bool(self.root_isolation.is_some());
        if let Some(program) = &self.root_isolation {
            h.hash(&program.identity());
        }
        h.bool(self.root_verifier.is_some());
        if let Some(verifier) = &self.root_verifier {
            h.hash(&verifier.identity());
        }
        h.finish_hash()
    }
    fn create(&self) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        self.factory.create()
    }
    fn create_scoped(
        &self,
        scope: ExecutionScope,
    ) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
        self.factory.create_scoped(scope)
    }
}
/// Actual factory capabilities used by the common reconstruction contract binder.
pub trait SupplierFactory: ProviderFactory {
    /// Producer-issued residual interpretation, when this registration owns it.
    fn declared_realization(&self) -> Option<&SelectedResidualRealization> {
        None
    }
    /// Every original residual realization, preserving authored alternative order.
    fn residuals(&self) -> Vec<&super::Factory>;
    /// Bind the actual native worker with the same checked numerical/proof offsets.
    fn reconstruction_worker(
        &self,
        scope: ExecutionScope,
        offsets: Option<&[f64]>,
    ) -> Result<SupplierSelection, pse_kernels::ProviderError>;
}
impl SupplierFactory for RegimeFactory {
    fn residuals(&self) -> Vec<&super::Factory> {
        self.alternatives.iter().map(|b| &b.residual).collect()
    }
    fn reconstruction_worker(
        &self,
        scope: ExecutionScope,
        offsets: Option<&[f64]>,
    ) -> Result<SupplierSelection, pse_kernels::ProviderError> {
        self.prepare_selection_offsets(scope, offsets)
            .map(|worker| SupplierSelection::Regimes(Box::new(worker)))
    }
}
impl SupplierFactory for ReconstructionFactory {
    fn declared_realization(&self) -> Option<&SelectedResidualRealization> {
        Some(&self.realization)
    }
    fn residuals(&self) -> Vec<&super::Factory> {
        match &self.factory {
            super::ImplicitFactory::Root(f) => vec![f],
            super::ImplicitFactory::Regimes(f) => f.residuals(),
        }
    }
    fn reconstruction_worker(
        &self,
        scope: ExecutionScope,
        offsets: Option<&[f64]>,
    ) -> Result<SupplierSelection, pse_kernels::ProviderError> {
        match &self.factory {
            super::ImplicitFactory::Regimes(f) => f.reconstruction_worker(scope, offsets),
            super::ImplicitFactory::Root(f) => {
                let program = self.root_isolation.as_ref().ok_or_else(|| {
                    pse_kernels::ProviderError::Contract(
                        "single-root reconstruction exact projection unavailable".into(),
                    )
                })?;
                let verifier = self.root_verifier.as_ref().ok_or_else(|| {
                    pse_kernels::ProviderError::Contract(
                        "single-root reconstruction verifier unavailable".into(),
                    )
                })?;
                RootSelection::new(f, program.clone(), verifier.clone(), scope, offsets)
                    .map(|worker| SupplierSelection::Root(Box::new(worker)))
            }
        }
    }
}
/// Native reconstruction worker retaining its actual source kind.
#[derive(Debug)]
pub enum SupplierSelection {
    /// Genuine single-root numerical worker and independent regularity proof.
    Root(Box<RootSelection>),
    /// Authored competitive regime selector.
    Regimes(Box<RegimeSelection>),
}
impl SupplierSelection {
    /// Observation from the actual admitted reconstruction producer.
    pub fn sheet_identity(&self) -> Option<ContentHash> {
        match self {
            Self::Root(v) => v.sheet_identity(),
            Self::Regimes(v) => v.sheet_identity(),
        }
    }
    /// Observation from the actual admitted reconstruction producer.
    pub fn selected_chart(&self) -> Option<&super::SelectionChart> {
        match self {
            Self::Root(v) => v.selected_chart(),
            Self::Regimes(v) => v.selected_chart(),
        }
    }
    /// Observation from the actual admitted reconstruction producer.
    pub fn observed_point_cells(&self) -> u64 {
        match self {
            Self::Root(v) => v.observed_point_cells(),
            Self::Regimes(v) => v.observed_point_cells(),
        }
    }
    /// Observation from the actual admitted reconstruction producer.
    pub fn observed_action_cells(&self) -> u64 {
        match self {
            Self::Root(v) => v.observed_action_cells(),
            Self::Regimes(v) => v.observed_action_cells(),
        }
    }
    fn evaluate(
        &mut self,
        parameters: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<super::SelectedRegime, MathError> {
        match self {
            Self::Root(v) => v.evaluate(parameters, order, cancel),
            Self::Regimes(v) => v.evaluate(parameters, order, cancel),
        }
    }
    fn refinement_deadline(&self) -> Result<std::time::Instant, MathError> {
        match self {
            Self::Root(v) => v.refinement_deadline(),
            Self::Regimes(v) => v.refinement_deadline(),
        }
    }
    fn refinement_checkpoint(
        &self,
        deadline: std::time::Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), MathError> {
        match self {
            Self::Root(v) => v.refinement_checkpoint(deadline, cancel),
            Self::Regimes(v) => v.refinement_checkpoint(deadline, cancel),
        }
    }
    fn refine_point(
        &mut self,
        parameters: &[f64],
        unknown_scales: &[f64],
        row_scales: &[f64],
        max_cells: u64,
        deadline: std::time::Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<super::RootPointEvidence, MathError> {
        match self {
            Self::Root(v) => v.refine_point(
                parameters,
                unknown_scales,
                row_scales,
                max_cells,
                deadline,
                cancel,
            ),
            Self::Regimes(v) => v.refine_point(
                parameters,
                unknown_scales,
                row_scales,
                max_cells,
                deadline,
                cancel,
            ),
        }
    }
    fn enclose_action_bounded(
        &mut self,
        parameters: &[f64],
        direction: &[f64],
        max_cells: u64,
        deadline: std::time::Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<RootActionEvidence, MathError> {
        match self {
            Self::Root(v) => {
                v.enclose_action_bounded(parameters, direction, max_cells, deadline, cancel)
            }
            Self::Regimes(v) => {
                v.enclose_action_bounded(parameters, direction, max_cells, deadline, cancel)
            }
        }
    }
    fn enclose_neighborhood(
        &mut self,
        parameters: &[f64],
        parameter_intervals: &[super::ProofInterval],
        direction_intervals: Option<&[super::ProofInterval]>,
        max_cells: u64,
        deadline: std::time::Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<super::RootNeighborhoodEvidence, MathError> {
        match self {
            Self::Root(v) => v.enclose_neighborhood(
                parameters,
                parameter_intervals,
                direction_intervals,
                max_cells,
                deadline,
                cancel,
            ),
            Self::Regimes(v) => v.enclose_neighborhood(
                parameters,
                parameter_intervals,
                direction_intervals,
                max_cells,
                deadline,
                cancel,
            ),
        }
    }
    fn refine_numerical(
        &mut self,
        controls: crate::implicit::NumericalRefinement<'_>,
    ) -> Result<super::SelectedRegime, MathError> {
        match self {
            Self::Root(v) => v.refine_numerical(controls),
            Self::Regimes(v) => v.refine_numerical(controls),
        }
    }
}
/// Single-root worker. Numerical selection, original residual validation and IFT
/// remain the actual Factory's contracts; interval evidence supplies no iteration.
#[derive(Debug)]
pub struct RootSelection {
    problem: Arc<super::Problem>,
    configuration: super::ConfigurationWorker,
    selection: super::selection::SelectionWorker,
    solver: Arc<dyn super::InnerSolver>,
    program: Arc<crate::factorable::RootIsolationProgram>,
    verifier: Arc<dyn super::SelectionVerifier>,
    scope: ExecutionScope,
    time_limit: std::time::Duration,
    numerical: Option<(ContentHash, super::SelectedRegime, super::Options)>,
    chart: Option<super::SelectionChart>,
    sheet: Option<ContentHash>,
    sheet_parameters: Vec<f64>,
    proof_limit: u64,
    proof_ceiling: u64,
    point_cells: u64,
    action_cells: u64,
}
impl RootSelection {
    fn new(
        factory: &super::Factory,
        program: Arc<crate::factorable::RootIsolationProgram>,
        verifier: Arc<dyn super::SelectionVerifier>,
        scope: ExecutionScope,
        offsets: Option<&[f64]>,
    ) -> Result<Self, pse_kernels::ProviderError> {
        scope.check()?;
        let problem = factory.problem(scope.clone())?;
        let problem = if let Some(offsets) = offsets {
            let owned = Arc::try_unwrap(problem).map_err(|_| {
                pse_kernels::ProviderError::Contract(
                    "fresh root residual unexpectedly shared".into(),
                )
            })?;
            Arc::new(
                owned
                    .with_residual_offsets(offsets)
                    .map_err(super::provider_error)?,
            )
        } else {
            problem
        };
        let program = if let Some(offsets) = offsets {
            if program.residuals.len() != offsets.len() || offsets.iter().any(|v| !v.is_finite()) {
                return Err(pse_kernels::ProviderError::Contract(
                    "root proof offset extent/value".into(),
                ));
            }
            let mut program = program.as_ref().clone();
            for (residual, offset) in program.residuals.iter_mut().zip(offsets) {
                let constant = program.nodes.len();
                program.nodes.push(crate::factorable::Node::Const(
                    crate::factorable::Constant::Float(-*offset),
                ));
                let difference = program.nodes.len();
                program
                    .nodes
                    .push(crate::factorable::Node::Sum(vec![*residual, constant]));
                *residual = difference;
            }
            Arc::new(program)
        } else {
            program
        };
        if program.inputs != factory.unknowns.len() + factory.spec.inputs.len()
            || program.residuals.len() != factory.rows.len()
        {
            return Err(pse_kernels::ProviderError::Contract(
                "single-root exact projection layout".into(),
            ));
        }
        let time_limit = factory.configuration.time_limit();
        Ok(Self {
            problem,
            configuration: super::ConfigurationWorker::new(
                factory.configuration.clone(),
                factory.hints.as_ref(),
                factory.terms.as_ref(),
            ),
            selection: super::selection::SelectionWorker::new(&factory.selection),
            solver: factory.solver.clone(),
            program,
            verifier,
            scope,
            time_limit,
            numerical: None,
            chart: None,
            sheet: None,
            sheet_parameters: Vec::new(),
            proof_limit: 1,
            proof_ceiling: u64::MAX,
            point_cells: 0,
            action_cells: 0,
        })
    }
    fn refinement_deadline(&self) -> Result<std::time::Instant, MathError> {
        let now = std::time::Instant::now();
        let local = now.checked_add(self.time_limit);
        match (local, self.scope.deadline()) {
            (Some(a), Some(b)) => Ok(a.min(b)),
            (Some(a), None) => Ok(a),
            (None, Some(b)) => Ok(b),
            (None, None) => Err(MathError::Limit("root refinement deadline unavailable")),
        }
    }
    fn refinement_checkpoint(
        &self,
        deadline: std::time::Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), MathError> {
        if !std::ptr::eq(self.scope.cancellation().as_ref(), cancel.as_ref()) {
            return Err(MathError::Contract(
                "root reconstruction cancellation owner".into(),
            ));
        }
        self.scope.check().map_err(|cause| MathError::Provider {
            source_id: self.problem.id,
            provider: self.problem.id,
            cause,
        })?;
        if std::time::Instant::now() >= deadline {
            return Err(MathError::Limit("root refinement deadline"));
        }
        Ok(())
    }
    fn with_request<T>(
        &self,
        parameters: &[f64],
        candidate: &[f64],
        deadline: std::time::Instant,
        cancel: &Arc<AtomicBool>,
        operation: impl FnOnce(&super::SelectionProofRequest<'_>) -> Result<T, MathError>,
    ) -> Result<T, MathError> {
        self.refinement_checkpoint(deadline, cancel)?;
        // A single exact residual source asks for uniqueness over its original domain.
        // This transport does not create a Regime, criterion worker, or regime solver.
        let alternatives = [super::SelectionAlternative {
            id: self.problem.id,
            program: &self.program,
            residual_identity: self.problem.identity,
            unknowns: &self.problem.unknowns,
        }];
        let request = super::SelectionProofRequest {
            selection: self.problem.id,
            alternatives: &alternatives,
            winner: 0,
            parameters,
            candidate,
            order: DerivativeOrder::First,
            time_limit: deadline.saturating_duration_since(std::time::Instant::now()),
            cancel,
        };
        operation(&request)
    }
    fn evaluate(
        &mut self,
        parameters: &[f64],
        order: DerivativeOrder,
        cancel: &Arc<AtomicBool>,
    ) -> Result<super::SelectedRegime, MathError> {
        let started = std::time::Instant::now();
        self.scope.check().map_err(|cause| MathError::Provider {
            source_id: self.problem.id,
            provider: self.problem.id,
            cause,
        })?;
        let anchor = self.selection.anchor(&self.problem, parameters, cancel)?;
        let mut options =
            self.configuration
                .resolve(&mut self.problem, parameters, cancel, anchor.as_deref())?;
        self.time_limit = options.time_limit;
        self.selection.configure(&mut self.problem, &options)?;
        let key = super::numerical_product_key(
            &self.problem,
            parameters,
            &options,
            self.solver.identity(),
        );
        let deadline = self.refinement_deadline()?.min(
            started
                .checked_add(options.time_limit)
                .ok_or(MathError::Limit("root deadline"))?,
        );
        self.refinement_checkpoint(deadline, cancel)?;
        if let Some((cached, values, _)) = &self.numerical
            && *cached == key
        {
            return Ok(values.clone());
        }
        options.time_limit = deadline.saturating_duration_since(std::time::Instant::now());
        let point = self
            .solver
            .solve(self.problem.clone(), parameters, &options, cancel)?;
        self.problem.verify(parameters, &point, &options, cancel)?;
        self.selection
            .verify(&self.problem, parameters, &point, order, cancel)?;
        let jet = self.problem.derivatives(
            parameters,
            &point,
            DerivativeOrder::First,
            &options,
            cancel,
        )?;
        let selected = super::SelectedRegime {
            id: self.problem.id,
            values: point,
            jacobian: jet.jacobian,
            hessians: Vec::new(),
            examined: 1,
            eligible: 1,
        };
        let chart =
            self.with_request(parameters, &selected.values, deadline, cancel, |request| {
                if let Some(chart) = &self.chart
                    && chart.validate(request, self.verifier.identity()).is_ok()
                {
                    return Ok(chart.clone());
                }
                match self.verifier.certify_bounded(request, self.proof_limit)? {
                    super::SelectionEvidence::Unique(chart) => {
                        chart.validate(request, self.verifier.identity())?;
                        Ok(chart)
                    }
                    super::SelectionEvidence::Incomplete(reason) => Err(MathError::Refinement {
                        product: key,
                        source_key: self.problem.identity,
                        validity: key,
                        reason: crate::derived::RefinementRefusal::Unavailable(reason),
                    }),
                    super::SelectionEvidence::Multiple => Err(MathError::Domain {
                        source_id: self.problem.id,
                        requirement: "single-root reconstruction uniqueness is unestablished",
                    }),
                }
            })?;
        if let Some(previous) = &self.chart
            && (previous.parameters != chart.parameters || previous.uniqueness != chart.uniqueness)
        {
            self.with_request(parameters, &selected.values, deadline, cancel, |request| {
                let chain = super::ChartChainRequest {
                    endpoint: request,
                    previous,
                    next: &chart,
                    origin: &self.sheet_parameters,
                    coverage: super::ChartChainCoverage::RootSheet,
                };
                chain.validate(self.verifier.identity())?;
                match self
                    .verifier
                    .connect_chain_bounded(&chain, self.proof_limit)?
                {
                    super::ChartChainEvidence::Connected(_) => Ok(()),
                    super::ChartChainEvidence::Interrupted(_) => Err(MathError::Cancelled),
                    _ => Err(MathError::Domain {
                        source_id: self.problem.id,
                        requirement: "single-root reconstruction sheet transport is unestablished",
                    }),
                }
            })?;
        }
        self.refinement_checkpoint(deadline, cancel)?;
        if self.sheet.is_none() {
            let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
            h.str("actual-single-root-sheet")
                .hash(&self.problem.identity)
                .hash(&self.verifier.identity());
            for p in parameters {
                h.f64(*p);
            }
            for e in &chart.existence {
                h.f64(e.lower).f64(e.upper);
            }
            self.sheet = Some(h.finish_hash());
        }
        self.chart = Some(chart);
        self.sheet_parameters = parameters.to_vec();
        self.numerical = Some((key, selected.clone(), options));
        Ok(selected)
    }
    fn sheet_identity(&self) -> Option<ContentHash> {
        self.sheet
    }
    fn selected_chart(&self) -> Option<&super::SelectionChart> {
        self.chart.as_ref()
    }
    fn observed_point_cells(&self) -> u64 {
        self.point_cells
    }
    fn observed_action_cells(&self) -> u64 {
        self.action_cells
    }
    fn refine_point(
        &mut self,
        parameters: &[f64],
        unknown_scales: &[f64],
        row_scales: &[f64],
        max_cells: u64,
        deadline: std::time::Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<super::RootPointEvidence, MathError> {
        let max_cells = max_cells.min(self.proof_ceiling);
        let (_, selected, _) = self.numerical.as_ref().ok_or_else(|| {
            MathError::Contract("root point requires actual numerical product".into())
        })?;
        let chart = self
            .chart
            .as_ref()
            .ok_or_else(|| MathError::Contract("root point requires actual chart".into()))?;
        let result =
            self.with_request(parameters, &selected.values, deadline, cancel, |request| {
                chart.validate(request, self.verifier.identity())?;
                self.verifier
                    .refine_point(request, chart, unknown_scales, row_scales, max_cells)
            })?;
        let cells = match &result {
            super::RootPointEvidence::Enclosed { proof_cells, .. }
            | super::RootPointEvidence::Incomplete { proof_cells, .. }
            | super::RootPointEvidence::Interrupted { proof_cells } => *proof_cells,
        };
        self.point_cells = self
            .point_cells
            .checked_add(cells)
            .ok_or(MathError::Limit("root point proof work"))?;
        if cells > max_cells {
            return Err(MathError::Contract(
                "root point verifier exceeded admitted cells".into(),
            ));
        }
        self.refinement_checkpoint(deadline, cancel)?;
        Ok(result)
    }
    fn enclose_action_bounded(
        &mut self,
        parameters: &[f64],
        direction: &[f64],
        max_cells: u64,
        deadline: std::time::Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<RootActionEvidence, MathError> {
        let max_cells = max_cells.min(self.proof_ceiling);
        let (_, selected, _) = self
            .numerical
            .as_ref()
            .ok_or_else(|| MathError::Contract("root action requires actual product".into()))?;
        let chart = self
            .chart
            .as_ref()
            .ok_or_else(|| MathError::Contract("root action requires actual chart".into()))?;
        let result =
            self.with_request(parameters, &selected.values, deadline, cancel, |request| {
                chart.validate(request, self.verifier.identity())?;
                self.verifier
                    .enclose_action_bounded(request, chart, direction, max_cells)
            })?;
        let cells = match &result {
            RootActionEvidence::Enclosed { proof_cells, .. }
            | RootActionEvidence::Incomplete { proof_cells, .. }
            | RootActionEvidence::Interrupted { proof_cells } => *proof_cells,
        };
        self.action_cells = self
            .action_cells
            .checked_add(cells)
            .ok_or(MathError::Limit("root action proof work"))?;
        if cells > max_cells {
            return Err(MathError::Contract(
                "root action verifier exceeded admitted cells".into(),
            ));
        }
        self.refinement_checkpoint(deadline, cancel)?;
        Ok(result)
    }
    fn enclose_neighborhood(
        &mut self,
        parameters: &[f64],
        parameter_intervals: &[super::ProofInterval],
        direction_intervals: Option<&[super::ProofInterval]>,
        max_cells: u64,
        deadline: std::time::Instant,
        cancel: &Arc<AtomicBool>,
    ) -> Result<super::RootNeighborhoodEvidence, MathError> {
        let max_cells = max_cells.min(self.proof_ceiling);
        let (_, selected, _) = self.numerical.as_ref().ok_or_else(|| {
            MathError::Contract("root neighborhood requires actual product".into())
        })?;
        let chart = self
            .chart
            .as_ref()
            .ok_or_else(|| MathError::Contract("root neighborhood requires actual chart".into()))?;
        let result =
            self.with_request(parameters, &selected.values, deadline, cancel, |request| {
                chart.validate(request, self.verifier.identity())?;
                self.verifier.enclose_neighborhood(
                    request,
                    chart,
                    parameter_intervals,
                    direction_intervals,
                    max_cells,
                )
            })?;
        let cells = match &result {
            super::RootNeighborhoodEvidence::Enclosed { proof_cells, .. }
            | super::RootNeighborhoodEvidence::Incomplete { proof_cells, .. }
            | super::RootNeighborhoodEvidence::Interrupted { proof_cells } => *proof_cells,
        };
        self.action_cells = self
            .action_cells
            .checked_add(cells)
            .ok_or(MathError::Limit("root neighborhood proof work"))?;
        if cells > max_cells {
            return Err(MathError::Contract(
                "root neighborhood verifier exceeded admitted cells".into(),
            ));
        }
        self.refinement_checkpoint(deadline, cancel)?;
        Ok(result)
    }
    fn refine_numerical(
        &mut self,
        controls: super::NumericalRefinement<'_>,
    ) -> Result<super::SelectedRegime, MathError> {
        let super::NumericalRefinement {
            parameters,
            unknown_scales,
            row_scales,
            root_allowance,
            residual_allowance,
            linear,
            product,
            deadline,
            cancel,
        } = controls;
        self.refinement_checkpoint(deadline, cancel)?;
        let (key, previous, mut options) = self
            .numerical
            .clone()
            .ok_or_else(|| MathError::Contract("root refinement requires actual product".into()))?;
        let precision = || MathError::Refinement {
            product: product.0,
            source_key: product.1,
            validity: product.2,
            reason: crate::derived::RefinementRefusal::Precision,
        };
        if unknown_scales.len() != options.variable_tolerance.len()
            || row_scales.len() != options.residual_tolerance.len()
            || !root_allowance.is_finite()
            || root_allowance <= 0.0
            || !residual_allowance.is_finite()
            || residual_allowance <= 0.0
        {
            return Err(precision());
        }
        for (t, s) in options.variable_tolerance.iter_mut().zip(unknown_scales) {
            *t = t.min(s * root_allowance);
        }
        for (t, s) in options.residual_tolerance.iter_mut().zip(row_scales) {
            *t = t.min(s * residual_allowance);
        }
        if options
            .variable_tolerance
            .iter()
            .chain(&options.residual_tolerance)
            .any(|t| !t.is_finite() || *t <= 0.0)
        {
            return Err(precision());
        }
        if let Some((direction, allowance)) = linear {
            if direction.len() != parameters.len() || !allowance.is_finite() || allowance <= 0.0 {
                return Err(precision());
            }
            let jet = self.problem.evaluate(
                parameters,
                &previous.values,
                DerivativeOrder::First,
                cancel,
            )?;
            let n = previous.values.len();
            let p = parameters.len();
            let width = n + p;
            let mut denominator = 0.0_f64;
            for row in 0..n {
                let mut weighted = 0.0;
                for j in 0..p {
                    let mut scale = jet.jacobian[row * width + n + j].abs();
                    for k in 0..n {
                        scale = (scale
                            + (jet.jacobian[row * width + k] * previous.jacobian[k * p + j])
                                .abs()
                                .next_up())
                        .next_up();
                    }
                    weighted = (weighted + (direction[j].abs() * scale).next_up()).next_up();
                }
                denominator = denominator.max((weighted / row_scales[row]).next_up());
            }
            if !denominator.is_finite() {
                return Err(precision());
            }
            if denominator > 0.0 {
                options.derivative_tolerance =
                    options.derivative_tolerance.min(allowance / denominator);
            }
            if !options.derivative_tolerance.is_finite() || options.derivative_tolerance <= 0.0 {
                return Err(precision());
            }
        }
        options.start = previous.values.clone();
        options.time_limit = options
            .time_limit
            .min(deadline.saturating_duration_since(std::time::Instant::now()));
        let point = self
            .solver
            .solve(self.problem.clone(), parameters, &options, cancel)?;
        self.problem.verify(parameters, &point, &options, cancel)?;
        self.selection.verify(
            &self.problem,
            parameters,
            &point,
            DerivativeOrder::First,
            cancel,
        )?;
        let chart = self
            .chart
            .as_ref()
            .ok_or_else(|| MathError::Contract("root refinement requires original chart".into()))?;
        if point
            .iter()
            .zip(&chart.uniqueness)
            .any(|(p, u)| !u.interior_contains(*p))
        {
            return Err(MathError::Domain {
                source_id: self.problem.id,
                requirement: "refined root left original uniqueness chart",
            });
        }
        let jet = self.problem.derivatives(
            parameters,
            &point,
            DerivativeOrder::First,
            &options,
            cancel,
        )?;
        self.refinement_checkpoint(deadline, cancel)?;
        let selected = super::SelectedRegime {
            values: point,
            jacobian: jet.jacobian,
            hessians: Vec::new(),
            ..previous
        };
        self.numerical = Some((key, selected.clone(), options));
        Ok(selected)
    }
}
/// Checked source realization and exact named original equality correspondence.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectedResidualBinding {
    original: ContentHash,
    source: ContentHash,
    rows: Vec<(GlobalRow, pse_ids::SemanticId, f64)>,
    realization: SelectedResidualRealization,
    key: ContentHash,
}
impl SelectedResidualBinding {
    /// Derive offsets mechanically from the actual original equality intervals.
    pub fn for_original(
        factory: &impl SupplierFactory,
        original: &OriginalContract,
        eliminated: &[GlobalRow],
    ) -> Result<Self, MathError> {
        Self::for_source(
            factory,
            original,
            eliminated,
            SelectedResidualRealization::AuthoredValues,
        )
    }
    /// Bind an explicit compiler/producer realization, without inferring subtraction
    /// from equation IDs or applying an original offset twice.
    pub fn for_source(
        factory: &impl SupplierFactory,
        original: &OriginalContract,
        eliminated: &[GlobalRow],
        realization: SelectedResidualRealization,
    ) -> Result<Self, MathError> {
        if factory
            .declared_realization()
            .is_some_and(|declared| declared != &realization)
        {
            return Err(MathError::Contract(
                "residual binding differs from producer-issued realization".into(),
            ));
        }
        let rows = eliminated
            .iter()
            .map(|&row| {
                let constraint = original.constraints().get(row.get()).ok_or_else(|| {
                    MathError::Contract("selected residual row outside original".into())
                })?;
                if !constraint.lower.is_finite()
                    || constraint.lower.to_bits() != constraint.upper.to_bits()
                {
                    return Err(MathError::Contract(
                        "selected residual offset requires an authored finite equality".into(),
                    ));
                }
                Ok((row, constraint.id, constraint.lower))
            })
            .collect::<Result<Vec<_>, MathError>>()?;
        if rows.is_empty()
            || rows
                .iter()
                .map(|(r, _, _)| r)
                .collect::<BTreeSet<_>>()
                .len()
                != rows.len()
            || factory.residuals().iter().any(|b| {
                b.rows.len() != rows.len()
                    || b.rows
                        .iter()
                        .zip(&rows)
                        .any(|(id, (_, expected, _))| id != expected)
            })
        {
            return Err(MathError::Contract(
                "selected residual named row coverage".into(),
            ));
        }
        if let SelectedResidualRealization::ZeroResiduals { authored_offsets } = &realization
            && (authored_offsets.len() != rows.len()
                || authored_offsets.iter().zip(&rows).any(
                    |((id, offset), (_, expected, value))| {
                        id != expected || offset.to_bits() != value.to_bits()
                    },
                ))
        {
            return Err(MathError::Contract(
                "realized zero residual offsets differ from authored named equalities".into(),
            ));
        }
        let source = factory.configuration_key();
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("selected-authored-equality-offset-binding");
        original.frame(&mut h);
        h.hash(&source);
        for (row, id, offset) in &rows {
            h.u64(row.get() as u64).id(id).f64(*offset);
        }
        h.str(match realization {
            SelectedResidualRealization::AuthoredValues => "authored-values",
            SelectedResidualRealization::ZeroResiduals { .. } => "already-realized-zero-residuals",
        });
        Ok(Self {
            original: original.identity(),
            source,
            rows,
            realization,
            key: h.finish_hash(),
        })
    }
    /// Additional owned binding metadata, excluding source factory/contracts.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.rows.capacity() * size_of::<(GlobalRow, pse_ids::SemanticId, f64)>()
            + match &self.realization {
                SelectedResidualRealization::AuthoredValues => 0,
                SelectedResidualRealization::ZeroResiduals { authored_offsets } => {
                    authored_offsets.capacity() * size_of::<(pse_ids::SemanticId, f64)>()
                }
            }
    }
    /// Actual factory, original contract, row ordering and offsets identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Preserved named authored subtraction, in selected residual output order.
    pub fn rows(&self) -> &[(GlobalRow, pse_ids::SemanticId, f64)] {
        &self.rows
    }
    /// Actual admitted producer realization, including any already-applied offsets.
    pub fn realization(&self) -> &SelectedResidualRealization {
        &self.realization
    }
    fn offsets(&self) -> Option<Vec<f64>> {
        match self.realization {
            SelectedResidualRealization::AuthoredValues => {
                Some(self.rows.iter().map(|(_, _, value)| *value).collect())
            }
            SelectedResidualRealization::ZeroResiduals { .. } => None,
        }
    }
    fn check(
        &self,
        factory: &impl SupplierFactory,
        original: &OriginalContract,
        eliminated: &[GlobalRow],
    ) -> Result<(), MathError> {
        if self != &Self::for_source(factory, original, eliminated, self.realization.clone())? {
            return Err(MathError::Contract(
                "selected residual binding differs from actual original source/bounds".into(),
            ));
        }
        Ok(())
    }
}

/// Actual selected-root reconstruction. Native iteration and IFT jets stay owned by
/// the admitted implicit factory; interval evidence stays owned by its verifier.
/// Runtime admission must retain the original factory lease and this wrapper's extent.
#[derive(Debug)]
pub struct SelectedImplicitReconstruction<E = MathError> {
    contract: Arc<ReconstructionContract>,
    selection: SupplierSelection,
    unknown_columns: Vec<GlobalCol>,
    normalization: Normalization,
    cancel: Arc<AtomicBool>,
    marker: PhantomData<fn() -> E>,
    last_refusal: Option<crate::derived::RefinementRefusal>,
}
impl<E> SelectedImplicitReconstruction<E> {
    /// Source identity of the actual factory's compiled interpretation/configuration.
    pub fn source(factory: &impl SupplierFactory) -> ContentHash {
        factory.configuration_key()
    }
    /// Actual numerical IFT/action-enclosure producer identity, separately framed.
    pub fn derivative_source(factory: &impl SupplierFactory) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("selected-implicit-ift-with-interval-action")
            .hash(&Self::source(factory));
        h.finish_hash()
    }
    /// Derive reconstruction structure from admitted original coordinates and actual
    /// residual support. POUNCE matching/DM/BTF owns structural inverse dependencies;
    /// numerical rank, regularity, root selection and accuracy are established later.
    /// Every branch must represent the same declared original eliminated equalities.
    pub fn prepare_contract(
        factory: &impl SupplierFactory,
        original: Arc<OriginalContract>,
        retained: Vec<GlobalCol>,
        eliminated: Vec<GlobalRow>,
        validity: ContentHash,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<ReconstructionContract>, MathError> {
        if eliminated.iter().any(|r| {
            original
                .constraints()
                .get(r.get())
                .is_none_or(|c| c.lower != 0.0 || c.upper != 0.0)
        }) {
            return Err(MathError::Contract(
                "nonzero selected equality requires an explicit residual offset binding".into(),
            ));
        }
        Self::prepare_contract_inner(
            factory, original, retained, eliminated, validity, cancel, None,
        )
    }
    /// Prepare reconstruction with mechanically checked nonzero equality offsets.
    pub fn prepare_contract_with_binding(
        factory: &impl SupplierFactory,
        original: Arc<OriginalContract>,
        retained: Vec<GlobalCol>,
        eliminated: Vec<GlobalRow>,
        validity: ContentHash,
        cancel: &Arc<AtomicBool>,
        binding: &SelectedResidualBinding,
    ) -> Result<Arc<ReconstructionContract>, MathError> {
        binding.check(factory, &original, &eliminated)?;
        Self::prepare_contract_inner(
            factory,
            original,
            retained,
            eliminated,
            validity,
            cancel,
            Some(binding),
        )
    }
    fn prepared_source(
        factory: &impl SupplierFactory,
        binding: Option<&SelectedResidualBinding>,
    ) -> ContentHash {
        match binding {
            None => Self::source(factory),
            Some(binding) => {
                let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
                h.str("selected-offset-residual-source")
                    .hash(&Self::source(factory))
                    .hash(&binding.key());
                h.finish_hash()
            }
        }
    }
    fn prepared_derivative_source(
        factory: &impl SupplierFactory,
        binding: Option<&SelectedResidualBinding>,
    ) -> ContentHash {
        match binding {
            None => Self::derivative_source(factory),
            Some(binding) => {
                let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
                h.str("selected-offset-residual-ift-source")
                    .hash(&Self::prepared_source(factory, Some(binding)));
                h.finish_hash()
            }
        }
    }
    fn prepare_contract_inner(
        factory: &impl SupplierFactory,
        original: Arc<OriginalContract>,
        retained: Vec<GlobalCol>,
        eliminated: Vec<GlobalRow>,
        validity: ContentHash,
        cancel: &Arc<AtomicBool>,
        binding: Option<&SelectedResidualBinding>,
    ) -> Result<Arc<ReconstructionContract>, MathError> {
        let unknown_columns = maps(factory, &original, &retained, &eliminated)?;
        let n = unknown_columns.len();
        let p = retained.len();
        let mut incidence = retained
            .iter()
            .enumerate()
            .map(|(j, &c)| Entry::new(c, GlobalCol::new(j)))
            .collect::<Vec<_>>();
        for branch in factory.residuals() {
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            let structural = branch.body.incidence(cancel)?;
            let mut unknown_support = Vec::with_capacity(n);
            let mut parameter_support = Vec::with_capacity(n);
            for output in structural.outputs() {
                let support = structural.first_for_output(*output).ok_or_else(|| {
                    MathError::Contract("reconstruction residual lacks structural support".into())
                })?;
                unknown_support.push(
                    support
                        .iter()
                        .copied()
                        .filter(|i| *i < n)
                        .collect::<Vec<_>>(),
                );
                parameter_support.push(
                    support
                        .iter()
                        .copied()
                        .filter_map(|i| i.checked_sub(n).filter(|j| *j < p))
                        .collect::<BTreeSet<_>>(),
                );
            }
            if unknown_support.len() != n {
                return Err(MathError::Contract(
                    "reconstruction residual structural row coverage".into(),
                ));
            }
            let mut adj_ptr = vec![0];
            let mut vars = Vec::new();
            for row in &unknown_support {
                vars.extend_from_slice(row);
                adj_ptr.push(vars.len());
            }
            let input = pounce_presolve::EqualityIncidence {
                n_vars: n,
                eq_row_inner_idx: (0..n).collect(),
                adj_ptr,
                vars,
            };
            let matching = pounce_presolve::matching::hopcroft_karp(&input);
            if matching.size != n {
                return Err(MathError::Contract(
                    "reconstruction has no complete structural unknown matching".into(),
                ));
            }
            let dm = pounce_presolve::DulmageMendelsohnPartition::from_matching(&input, &matching);
            let components =
                pounce_presolve::SquareComponents::of_square_part(&input, &matching, &dm);
            let mut dependencies = vec![BTreeSet::<usize>::new(); n];
            let mut covered = BTreeSet::new();
            for component in &components.components {
                for block in
                    pounce_presolve::BlockTriangularForm::of_component(&input, &matching, component)
                        .blocks
                {
                    let mut inputs = BTreeSet::new();
                    for &row in &block.eq_rows {
                        inputs.extend(parameter_support[row].iter().copied());
                        for &column in &unknown_support[row] {
                            if !block.cols.contains(&column) {
                                inputs.extend(dependencies[column].iter().copied());
                            }
                        }
                    }
                    for column in block.cols {
                        covered.insert(column);
                        dependencies[column] = inputs.clone();
                    }
                }
            }
            if covered.len() != n {
                return Err(MathError::Contract(
                    "reconstruction BTF did not cover every unknown".into(),
                ));
            }
            for (unknown, columns) in dependencies.iter().enumerate() {
                for &column in columns {
                    incidence.push(Entry::new(unknown_columns[unknown], GlobalCol::new(column)));
                }
            }
        }
        let source = Self::prepared_source(factory, binding);
        let support = DerivativeSupport {
            order: DerivativeOrder::First,
            jacobian_product: true,
            source: Self::prepared_derivative_source(factory, binding),
        };
        let contract = match binding {
            None => ReconstructionContract::new(
                original, source, validity, retained, eliminated, incidence, support,
            )?,
            Some(binding) => ReconstructionContract::new_with_offsets(
                original,
                ReconstructionProducer {
                    source,
                    validity,
                    support,
                },
                retained,
                eliminated,
                incidence,
                &binding
                    .rows()
                    .iter()
                    .map(|(row, _, offset)| (*row, *offset))
                    .collect::<Vec<_>>(),
            )?,
        };
        Ok(Arc::new(contract))
    }
    /// Bind one existing factory worker to exact original maps and normalized error
    /// coordinates. A reconstruction cannot infer elimination from a matching alone.
    pub fn new(
        factory: &impl SupplierFactory,
        contract: Arc<ReconstructionContract>,
        normalization: Normalization,
        scope: ExecutionScope,
    ) -> Result<Self, MathError> {
        if contract
            .eliminated()
            .iter()
            .any(|r| contract.original().constraints()[r.get()].lower != 0.0)
        {
            return Err(MathError::Contract(
                "nonzero selected equality requires explicit residual binding".into(),
            ));
        }
        Self::new_inner(factory, contract, normalization, scope, None)
    }
    /// Bind the exact same checked offsets to actual native residual and verifier DAG.
    pub fn new_with_binding(
        factory: &impl SupplierFactory,
        contract: Arc<ReconstructionContract>,
        normalization: Normalization,
        scope: ExecutionScope,
        binding: &SelectedResidualBinding,
    ) -> Result<Self, MathError> {
        binding.check(factory, contract.original(), contract.eliminated())?;
        Self::new_inner(factory, contract, normalization, scope, Some(binding))
    }
    fn new_inner(
        factory: &impl SupplierFactory,
        contract: Arc<ReconstructionContract>,
        normalization: Normalization,
        scope: ExecutionScope,
        binding: Option<&SelectedResidualBinding>,
    ) -> Result<Self, MathError> {
        normalization.validate(
            contract.original().coordinates().len(),
            contract.original().constraints().len(),
        )?;
        if normalization.key() != contract.original().normalization()
            || contract.source() != Self::prepared_source(factory, binding)
            || contract.support().source != Self::prepared_derivative_source(factory, binding)
            || !contract.support().jacobian_product
            || contract.support().order != DerivativeOrder::First
        {
            return Err(MathError::Contract(
                "selected reconstruction source/support/normalization mismatch".into(),
            ));
        }
        let unknown_columns = maps(
            factory,
            contract.original(),
            contract.retained(),
            contract.eliminated(),
        )?;
        let cancel = scope.cancellation().clone();
        let offsets = binding.and_then(SelectedResidualBinding::offsets);
        let selection = factory
            .reconstruction_worker(scope, offsets.as_deref())
            .map_err(|cause| MathError::Provider {
                source_id: factory.spec().id,
                provider: factory.spec().id,
                cause,
            })?;
        Ok(Self {
            contract,
            selection,
            unknown_columns,
            normalization,
            cancel,
            marker: PhantomData,
            last_refusal: None,
        })
    }
    /// Bind the finite proof workspace ceiling before initial composite admission.
    /// Existing regime selection keeps its declared complete covering contract.
    pub fn with_proof_cell_limit(mut self, max_cells: u64) -> Result<Self, MathError> {
        if max_cells == 0 {
            return Err(MathError::Limit("root initial proof cell allowance"));
        }
        if let SupplierSelection::Root(root) = &mut self.selection {
            root.proof_limit = max_cells;
            root.proof_ceiling = max_cells;
        }
        Ok(self)
    }
    /// Additional retained wrapper extent. Original contracts/factory programs and
    /// native verifier workspace remain charged by their existing owners, not twice.
    pub fn retained_bytes(&self) -> Result<usize, MathError> {
        size_of::<Self>()
            .checked_add(
                self.unknown_columns
                    .capacity()
                    .checked_mul(size_of::<GlobalCol>())
                    .ok_or(MathError::Limit("reconstruction map extent"))?,
            )
            .and_then(|n| {
                n.checked_add(
                    (self.normalization.variables.capacity() + self.normalization.rows.capacity())
                        * size_of::<f64>(),
                )
            })
            .ok_or(MathError::Limit("reconstruction wrapper extent"))
    }
    /// Actual selected worker for observational work accounting and final owner reuse.
    pub fn selection(&self) -> &SupplierSelection {
        &self.selection
    }
    fn check_input(&self, x: &[f64]) -> Result<(), MathError> {
        if x.len() != self.contract.retained().len() {
            return Err(MathError::Contract(
                "selected reconstruction retained extent".into(),
            ));
        }
        for (&column, &value) in self.contract.retained().iter().zip(x) {
            let Coordinate { lower, upper, id } =
                self.contract.original().coordinates()[column.get()];
            if !value.is_finite() || value < lower || value > upper {
                return Err(MathError::Domain {
                    source_id: id,
                    requirement: "retained reconstruction coordinate violates its original interval",
                });
            }
        }
        Ok(())
    }
    fn full(&self, x: &[f64], unknowns: &[f64]) -> Result<Vec<f64>, MathError> {
        if unknowns.len() != self.unknown_columns.len() {
            return Err(MathError::Contract(
                "selected reconstruction unknown extent".into(),
            ));
        }
        let mut result = vec![0.0; self.contract.original().coordinates().len()];
        for (&column, &value) in self.contract.retained().iter().zip(x) {
            result[column.get()] = value;
        }
        for (&column, &value) in self.unknown_columns.iter().zip(unknowns) {
            result[column.get()] = value;
        }
        if result.iter().any(|v| !v.is_finite()) {
            return Err(MathError::Contract(
                "nonfinite selected reconstruction".into(),
            ));
        }
        Ok(result)
    }
    fn accuracy(
        &self,
        values: &[f64],
        intervals: &[super::ProofInterval],
        demand: &AccuracyDemand,
    ) -> Result<AccuracyEvidence, MathError> {
        demand
            .validate()
            .map_err(|e| MathError::Contract(e.to_string()))?;
        if demand.normalization != self.normalization.key()
            || intervals.len() != self.unknown_columns.len()
        {
            return Err(MathError::Contract(
                "selected reconstruction accuracy scope/extent".into(),
            ));
        }
        let mut error = 0.0_f64;
        for ((&column, &value), interval) in self.unknown_columns.iter().zip(values).zip(intervals)
        {
            if !interval.valid() || !value.is_finite() {
                return Err(MathError::Contract(
                    "invalid actual root/action enclosure".into(),
                ));
            }
            // Each positive primitive is widened separately. Certified library interval
            // endpoints and declared binary64 coordinate scales are treated exactly;
            // a rounded-to-nearest subtraction/division is never used as an upper bound.
            let distance = (value - interval.lower)
                .abs()
                .next_up()
                .max((value - interval.upper).abs().next_up());
            let normalized = (distance / self.normalization.variables[column.get()]).next_up();
            error = error.max(normalized);
        }
        let evidence = AccuracyEvidence {
            product: demand.product,
            normalization: demand.normalization,
            class: AccuracyClass::Certified,
            error: Some(error),
        };
        Ok(evidence)
    }
    // The interval inverse is uniform over the entire candidate/root hull. Thus
    // K already covers the nonlinear mean-value map; no local linearization
    // remainder is added. This only chooses controls, never certifies a product.
    fn backward_allowance(
        &self,
        selected: &super::SelectedRegime,
        demand: &AccuracyDemand,
        inverse: f64,
    ) -> Result<f64, MathError> {
        let mut h = FramedHasher::new(pse_ids::Frame::AccuracyProductV1);
        h.str("selected-normalized-residual")
            .hash(&self.contract.source())
            .hash(&demand.product);
        for r in self.contract.eliminated() {
            h.id(&self.contract.original().constraints()[r.get()].id);
        }
        for v in &selected.values {
            h.f64(*v);
        }
        let representable = demand.allowance / inverse;
        if !representable.is_finite() || representable <= 0. {
            return Err(self.refusal(demand, crate::derived::RefinementRefusal::Precision));
        }
        crate::derived::ErrorAmplification {
            input_product: h.finish_hash(),
            output_product: demand.product,
            normalization: demand.normalization,
            inverse_norm: inverse,
            remainder: 0.0,
            class: AccuracyClass::Certified,
        }
        .inner_allowance(demand)
    }
    fn refusal(
        &self,
        demand: &AccuracyDemand,
        reason: crate::derived::RefinementRefusal,
    ) -> MathError {
        MathError::Refinement {
            product: demand.product,
            source_key: self.contract.source(),
            validity: self.contract.validity(),
            reason,
        }
    }
    fn refined(
        &mut self,
        x: &[f64],
        direction: Option<&[f64]>,
        demand: &AccuracyDemand,
        limits: crate::derived::RefinementLimits,
    ) -> Result<ReconstructionObservation, MathError> {
        limits.validate()?;
        self.check_input(x)?;
        if direction.is_some_and(|v| v.len() != x.len() || v.iter().any(|a| !a.is_finite())) {
            return Err(MathError::Contract(
                "selected reconstruction direction extent/value".into(),
            ));
        }
        demand
            .validate()
            .map_err(|e| MathError::Contract(e.to_string()))?;
        let deadline = self.selection.refinement_deadline()?;
        self.selection
            .refinement_checkpoint(deadline, &self.cancel)?;
        if let SupplierSelection::Root(root) = &mut self.selection {
            root.proof_limit = limits.proof_cells.min(root.proof_ceiling);
        }
        let mut selected = self
            .selection
            .evaluate(x, DerivativeOrder::First, &self.cancel)?;
        self.selection
            .refinement_checkpoint(deadline, &self.cancel)?;
        let scales = self
            .unknown_columns
            .iter()
            .map(|c| self.normalization.variables[c.get()])
            .collect::<Vec<_>>();
        let rows = self
            .contract
            .eliminated()
            .iter()
            .map(|r| self.normalization.rows[r.get()])
            .collect::<Vec<_>>();
        let mut remaining = limits.proof_cells;
        for round in 0..limits.rounds {
            self.selection
                .refinement_checkpoint(deadline, &self.cancel)?;
            let (intervals, inverse, cells) = match self.selection.refine_point(
                x,
                &scales,
                &rows,
                remaining,
                deadline,
                &self.cancel,
            )? {
                super::RootPointEvidence::Enclosed {
                    intervals,
                    inverse_norm_upper,
                    proof_cells,
                } => (intervals, inverse_norm_upper, proof_cells),
                super::RootPointEvidence::Interrupted { .. } => return Err(MathError::Cancelled),
                super::RootPointEvidence::Incomplete {
                    reason: SelectionProofRefusal::Resource,
                    proof_cells,
                } => {
                    return Err(self.refusal(
                        demand,
                        if proof_cells >= remaining {
                            crate::derived::RefinementRefusal::ProofCells
                        } else {
                            crate::derived::RefinementRefusal::Unavailable(
                                SelectionProofRefusal::Resource,
                            )
                        },
                    ));
                }
                super::RootPointEvidence::Incomplete { reason, .. } => {
                    return Err(self.refusal(
                        demand,
                        crate::derived::RefinementRefusal::Unavailable(reason),
                    ));
                }
            };
            remaining = remaining.checked_sub(cells).ok_or_else(|| {
                MathError::Contract("point verifier exceeded refinement allowance".into())
            })?;
            let mut action_uncertainty = 0.0_f64;
            let (values, evidence) = if let Some(direction) = direction {
                if selected.jacobian.len() != self.unknown_columns.len() * x.len() {
                    return Err(MathError::Contract(
                        "selected reconstruction actual IFT extent".into(),
                    ));
                }
                let values = (0..self.unknown_columns.len())
                    .map(|i| {
                        selected.jacobian[i * x.len()..(i + 1) * x.len()]
                            .iter()
                            .zip(direction)
                            .map(|(a, v)| a * v)
                            .sum::<f64>()
                    })
                    .collect::<Vec<_>>();
                let (action, cells) = match self.selection.enclose_action_bounded(
                    x,
                    direction,
                    remaining,
                    deadline,
                    &self.cancel,
                )? {
                    RootActionEvidence::Enclosed {
                        intervals,
                        proof_cells,
                    } => (intervals, proof_cells),
                    RootActionEvidence::Interrupted { .. } => return Err(MathError::Cancelled),
                    RootActionEvidence::Incomplete {
                        reason: SelectionProofRefusal::Resource,
                        proof_cells,
                    } => {
                        return Err(self.refusal(
                            demand,
                            if proof_cells >= remaining {
                                crate::derived::RefinementRefusal::ProofCells
                            } else {
                                crate::derived::RefinementRefusal::Unavailable(
                                    SelectionProofRefusal::Resource,
                                )
                            },
                        ));
                    }
                    RootActionEvidence::Incomplete { reason, .. } => {
                        return Err(self.refusal(
                            demand,
                            crate::derived::RefinementRefusal::Unavailable(reason),
                        ));
                    }
                };
                remaining = remaining.checked_sub(cells).ok_or_else(|| {
                    MathError::Contract("action verifier exceeded refinement allowance".into())
                })?;
                for (interval, scale) in action.iter().zip(&scales) {
                    action_uncertainty = action_uncertainty
                        .max(((interval.upper - interval.lower).next_up() / 2.0 / scale).next_up());
                }
                let evidence = self.accuracy(&values, &action, demand)?;
                (values, evidence)
            } else {
                (
                    selected.values.clone(),
                    self.accuracy(&selected.values, &intervals, demand)?,
                )
            };
            if evidence.satisfies(demand) {
                self.selection
                    .refinement_checkpoint(deadline, &self.cancel)?;
                return Ok(ReconstructionObservation {
                    values: self.full(direction.unwrap_or(x), &values)?,
                    accuracy: evidence,
                });
            }
            if round + 1 == limits.rounds {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::Rounds));
            }
            if remaining == 0 {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::ProofCells));
            }
            let mut root_allowance = demand.allowance;
            // Action error is not inferred from K. The observed action enclosure error
            // supplies a numerical tightening ratio only; the next action enclosure
            // remains the sole certifier. No estimate becomes accuracy evidence.
            if direction.is_some() {
                root_allowance *= demand.allowance
                    / evidence.error.ok_or_else(|| {
                        MathError::Contract("missing action enclosure error".into())
                    })?;
            }
            if !root_allowance.is_finite() || root_allowance <= 0. {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::Precision));
            }
            let allowance_demand = AccuracyDemand {
                allowance: root_allowance,
                ..*demand
            };
            let residual_allowance =
                self.backward_allowance(&selected, &allowance_demand, inverse)?;
            let linear = if let Some(direction) = direction {
                let slack = (demand.allowance - action_uncertainty).next_down();
                if slack <= 0. {
                    return Err(self.refusal(demand, crate::derived::RefinementRefusal::Precision));
                }
                let linear_demand = AccuracyDemand {
                    allowance: slack,
                    ..*demand
                };
                Some((
                    direction,
                    self.backward_allowance(&selected, &linear_demand, inverse)?,
                ))
            } else {
                None
            };
            selected = self
                .selection
                .refine_numerical(crate::implicit::NumericalRefinement {
                    parameters: x,
                    unknown_scales: &scales,
                    row_scales: &rows,
                    root_allowance,
                    residual_allowance,
                    linear,
                    product: (
                        demand.product,
                        self.contract.source(),
                        self.contract.validity(),
                    ),
                    deadline,
                    cancel: &self.cancel,
                })?;
        }
        Err(self.refusal(demand, crate::derived::RefinementRefusal::Rounds))
    }
}
impl<E: From<MathError> + std::fmt::Debug> ReconstructionOracle
    for SelectedImplicitReconstruction<E>
{
    type Error = E;
    fn contract(&self) -> &ReconstructionContract {
        &self.contract
    }
    fn supports_uncertainty(&self) -> bool {
        true
    }
    fn refinement_refusal(&self) -> Option<crate::derived::RefinementRefusal> {
        self.last_refusal
    }
    fn realization(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("actual-selected-reconstruction")
            .hash(&self.contract.source());
        match self.selection.sheet_identity() {
            Some(sheet) => {
                h.bool(true).hash(&sheet);
            }
            None => {
                h.bool(false);
            }
        }
        h.finish_hash()
    }
    fn admit(&mut self, x: &[f64]) -> Result<ReconstructionAdmission, E> {
        self.check_input(x)?;
        let selected = self
            .selection
            .evaluate(x, DerivativeOrder::First, &self.cancel)?;
        if self.selection.sheet_identity().is_none() || self.selection.selected_chart().is_none() {
            return Err(MathError::Contract(
                "selected reconstruction regular root sheet is unestablished".into(),
            )
            .into());
        }
        Ok(ReconstructionAdmission {
            values: self.full(x, &selected.values)?,
        })
    }
    fn point(
        &mut self,
        x: &[f64],
        demand: &AccuracyDemand,
        refinement: crate::derived::RefinementLimits,
    ) -> Result<ReconstructionObservation, E> {
        let result = self.refined(x, None, demand, refinement);
        self.last_refusal = match &result {
            Err(MathError::Refinement { reason, .. }) => Some(*reason),
            _ => None,
        };
        result.map_err(E::from)
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        demand: &AccuracyDemand,
        refinement: crate::derived::RefinementLimits,
    ) -> Result<ReconstructionObservation, E> {
        let result = self.refined(x, Some(direction), demand, refinement);
        self.last_refusal = match &result {
            Err(MathError::Refinement { reason, .. }) => Some(*reason),
            _ => None,
        };
        result.map_err(E::from)
    }
    fn uncertain(
        &mut self,
        x: &[f64],
        direction: Option<&[f64]>,
        incoming: crate::derived::ReconstructionUncertainty<'_>,
        demand: &AccuracyDemand,
        limits: crate::derived::RefinementLimits,
    ) -> Result<ReconstructionObservation, E> {
        let result = (|| -> Result<_, MathError> {
            if incoming.point.len() != x.len()
                || incoming.action.is_some_and(|v| v.len() != x.len())
                || incoming
                    .point
                    .iter()
                    .chain(incoming.action.into_iter().flatten())
                    .any(|e| !e.is_finite() || *e < 0.0)
                || direction.is_some() != incoming.action.is_some()
            {
                return Err(MathError::Contract(
                    "selected incoming uncertainty extent/value".into(),
                ));
            }
            if incoming
                .point
                .iter()
                .chain(incoming.action.into_iter().flatten())
                .all(|e| *e == 0.0)
            {
                return self.refined(x, direction, demand, limits);
            }
            if incoming.class != AccuracyClass::Certified
                && demand.class == AccuracyClass::Certified
            {
                return Err(self.refusal(
                    demand,
                    crate::derived::RefinementRefusal::Unavailable(SelectionProofRefusal::Coverage),
                ));
            }
            limits.validate()?;
            let deadline = self.selection.refinement_deadline()?;
            let before = self
                .selection
                .observed_point_cells()
                .checked_add(self.selection.observed_action_cells())
                .ok_or(MathError::Limit("incoming uncertainty proof observation"))?;
            let exact = self.refined(x, direction, demand, limits)?;
            let after = self
                .selection
                .observed_point_cells()
                .checked_add(self.selection.observed_action_cells())
                .ok_or(MathError::Limit("incoming uncertainty proof observation"))?;
            let remaining = limits
                .proof_cells
                .checked_sub(after.saturating_sub(before))
                .ok_or(MathError::Limit("incoming uncertainty proof allowance"))?;
            if remaining == 0 {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::ProofCells));
            }
            let interval = |value: f64,
                            error: f64,
                            column: &GlobalCol|
             -> Result<super::ProofInterval, MathError> {
                let radius = (error * self.normalization.variables[column.get()]).next_up();
                let interval = super::ProofInterval {
                    lower: (value - radius).next_down(),
                    upper: (value + radius).next_up(),
                };
                if !interval.valid() {
                    return Err(MathError::Contract(
                        "incoming physical uncertainty interval".into(),
                    ));
                }
                Ok(interval)
            };
            let parameters = x
                .iter()
                .zip(incoming.point)
                .zip(self.contract.retained())
                .map(|((x, error), c)| interval(*x, *error, c))
                .collect::<Result<Vec<_>, _>>()?;
            let directions = direction
                .zip(incoming.action)
                .map(|(v, errors)| {
                    v.iter()
                        .zip(errors)
                        .zip(self.contract.retained())
                        .map(|((v, error), c)| interval(*v, *error, c))
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            let (points, actions) = match self.selection.enclose_neighborhood(
                x,
                &parameters,
                directions.as_deref(),
                remaining,
                deadline,
                &self.cancel,
            )? {
                super::RootNeighborhoodEvidence::Enclosed {
                    points, actions, ..
                } => (points, actions),
                super::RootNeighborhoodEvidence::Incomplete { reason, .. } => {
                    return Err(self.refusal(
                        demand,
                        crate::derived::RefinementRefusal::Unavailable(reason),
                    ));
                }
                super::RootNeighborhoodEvidence::Interrupted { .. } => {
                    return Err(MathError::Cancelled);
                }
            };
            let intervals = if direction.is_some() {
                actions
                    .as_deref()
                    .ok_or_else(|| MathError::Contract("uniform incoming action absent".into()))?
            } else {
                &points
            };
            let unknowns: Vec<_> = self
                .unknown_columns
                .iter()
                .map(|c| exact.values[c.get()])
                .collect();
            let mut evidence = self.accuracy(&unknowns, intervals, demand)?;
            if incoming.class != AccuracyClass::Certified {
                evidence.class = AccuracyClass::Estimated;
            }
            if !evidence.satisfies(demand) {
                return Err(self.refusal(demand, crate::derived::RefinementRefusal::Precision));
            }
            self.selection
                .refinement_checkpoint(deadline, &self.cancel)?;
            Ok(ReconstructionObservation {
                values: exact.values,
                accuracy: evidence,
            })
        })();
        self.last_refusal = match &result {
            Err(MathError::Refinement { reason, .. }) => Some(*reason),
            _ => None,
        };
        result.map_err(E::from)
    }
}
fn maps(
    factory: &impl SupplierFactory,
    original: &OriginalContract,
    retained: &[GlobalCol],
    eliminated: &[GlobalRow],
) -> Result<Vec<GlobalCol>, MathError> {
    if factory.residuals().is_empty()
        || factory.spec().inputs.len() != retained.len()
        || retained
            .iter()
            .any(|c| c.get() >= original.coordinates().len())
        || eliminated
            .iter()
            .any(|r| r.get() >= original.constraints().len())
        || factory
            .spec()
            .inputs
            .iter()
            .zip(retained)
            .any(|(p, c)| p.id != original.coordinates()[c.get()].id)
        || factory.spec().derivatives < DerivativeOrder::First
        || factory.spec().smoothness < DerivativeOrder::First
    {
        return Err(MathError::Contract(
            "selected reconstruction actual input/support map".into(),
        ));
    }
    let unknown_columns = factory
        .spec()
        .outputs
        .iter()
        .map(|p| {
            original
                .coordinates()
                .iter()
                .position(|c| c.id == p.id)
                .map(GlobalCol::new)
                .ok_or_else(|| {
                    MathError::Contract("selected unknown is not an original coordinate".into())
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut covered = retained.iter().copied().collect::<BTreeSet<_>>();
    if unknown_columns.is_empty()
        || unknown_columns.iter().any(|c| !covered.insert(*c))
        || covered.len() != original.coordinates().len()
        || factory.residuals().iter().any(|b| {
            b.rows.len() != eliminated.len()
                || b.rows
                    .iter()
                    .zip(eliminated)
                    .any(|(id, r)| *id != original.constraints()[r.get()].id)
                || b.unknowns.len() != unknown_columns.len()
                || b.unknowns.iter().zip(&unknown_columns).any(|(u, c)| {
                    u.id != original.coordinates()[c.get()].id
                        || u.lower.to_bits() != original.coordinates()[c.get()].lower.to_bits()
                        || u.upper.to_bits() != original.coordinates()[c.get()].upper.to_bits()
                })
                || b.requirements.requested_output < DerivativeOrder::First
        })
    {
        return Err(MathError::Contract(
            "selected reconstruction original equality/unknown/bound correspondence".into(),
        ));
    }
    Ok(unknown_columns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        derived::{Constraint, OriginalObligations},
        implicit::{Configuration, Factory, ImplicitFactory, Options, Selection, Unknown},
        typed::{Binary, BodyBuilder, BodyLimits},
    };
    use std::{collections::BTreeMap, time::Duration};

    #[derive(Debug)]
    struct CheckOffset;
    impl super::super::SelectionVerifier for CheckOffset {
        fn identity(&self) -> ContentHash {
            ContentHash::from_bytes([92; 32])
        }
        fn workspace_bytes(
            &self,
            _: &[Arc<crate::factorable::RootIsolationProgram>],
        ) -> Result<usize, MathError> {
            Ok(0)
        }
        fn certify(
            &self,
            request: &super::super::SelectionProofRequest<'_>,
        ) -> Result<super::super::SelectionEvidence, MathError> {
            // Actual faer affine solve and original validation must precede this
            // intentionally unavailable proof. Offsets are applied exactly once.
            assert_eq!(request.candidate, &[5.0]);
            assert_eq!(request.alternatives.len(), 1);
            let program = request.alternatives[0].program;
            assert!(matches!(
                program.nodes[program.residuals[0]],
                crate::factorable::Node::Sum(_)
            ));
            Ok(super::super::SelectionEvidence::Incomplete(
                SelectionProofRefusal::Unsupported,
            ))
        }
    }
    fn source() -> (
        Factory,
        Arc<crate::factorable::RootIsolationProgram>,
        Arc<OriginalContract>,
        Normalization,
    ) {
        let registry = pse_quantity::standard::standard_registry().unwrap();
        let quantity = registry.neutral_dimensionless().unwrap();
        let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
        let id = pse_ids::SemanticId::from_bytes([91; 16]);
        let parameter = pse_ids::named_id(id, "parameter");
        let row = pse_ids::named_id(id, "row");
        let cancel = Arc::new(AtomicBool::new(false));
        let mut builder = BodyBuilder::new(
            crate::initialize().unwrap(),
            &registry,
            &pse_quantity::standard::StandardInvariantChecker,
            2,
            BodyLimits::default(),
        )
        .unwrap();
        let y = builder
            .input(0, quantity, pse_quantity::IndexSet::new(), id)
            .unwrap();
        let p = builder
            .input(1, quantity, pse_quantity::IndexSet::new(), parameter)
            .unwrap();
        let value = builder.binary(Binary::Sub, y, p, None, row).unwrap();
        let prepared = builder.prepare(&[value]).unwrap();
        let proof = crate::factorable::single_root_isolation_program(row, &prepared, &cancel, 100)
            .unwrap()
            .unwrap();
        let body = Arc::new(
            prepared
                .compile(
                    &[0],
                    &[0, 1],
                    DerivativeOrder::First,
                    crate::library::Optimization::default(),
                    crate::jets::EvaluationLimits::default(),
                    &cancel,
                )
                .unwrap(),
        );
        let unknowns = vec![Unknown {
            id,
            lower: -10.0,
            upper: 10.0,
        }];
        let port = |id| pse_kernels::Port { id, quantity, unit };
        let factory = Factory {
            selection: Selection::default(),
            requirements: pse_kernels::DerivativeRequirements::new(
                DerivativeOrder::Second,
                DerivativeOrder::First,
                DerivativeOrder::First,
                DerivativeOrder::First,
                DerivativeOrder::First,
            )
            .unwrap(),
            spec: pse_kernels::ProviderSpec {
                id,
                revision: ContentHash::from_bytes([1; 32]),
                data: ContentHash::from_bytes([2; 32]),
                derivative_source: pse_kernels::DerivativeSource::Implicit,
                shapes: Default::default(),
                inputs: vec![port(parameter)],
                outputs: vec![port(id)],
                derivatives: DerivativeOrder::First,
                smoothness: DerivativeOrder::First,
            },
            body,
            unknowns: unknowns.clone(),
            rows: vec![row],
            configuration: Configuration::Fixed(
                unknowns,
                Options {
                    start: vec![1.0],
                    variable_nominals: vec![1.0],
                    variable_tolerance: vec![1e-9],
                    residual_tolerance: vec![1e-9],
                    iterations: 20,
                    time_limit: Duration::from_secs(10),
                    derivative_tolerance: 1e-10,
                },
            ),
            hints: None,
            terms: None,
            solver: Arc::new(super::super::Affine::new(&prepared, 1).unwrap()),
            cancel,
            max_entries: 100,
            providers: BTreeMap::new(),
        };
        let normalization = Normalization {
            variables: vec![1.0, 1.0],
            rows: vec![1.0],
            objective: 1.0,
        };
        let original = Arc::new(
            OriginalContract::new(
                factory.spec.identity(),
                normalization.key(),
                vec![
                    Coordinate {
                        id: parameter,
                        lower: -10.0,
                        upper: 10.0,
                    },
                    Coordinate {
                        id,
                        lower: -10.0,
                        upper: 10.0,
                    },
                ],
                vec![Constraint {
                    id: row,
                    lower: 3.0,
                    upper: 3.0,
                }],
                vec![
                    Entry::new(GlobalRow::new(0), GlobalCol::new(0)),
                    Entry::new(GlobalRow::new(0), GlobalCol::new(1)),
                ],
                DerivativeSupport {
                    order: DerivativeOrder::First,
                    jacobian_product: true,
                    source: factory.configuration_key(),
                },
                OriginalObligations {
                    objective: None,
                    guards: ContentHash::from_bytes([4; 32]),
                    selection: ContentHash::from_bytes([5; 32]),
                },
            )
            .unwrap(),
        );
        (factory, Arc::new(proof), original, normalization)
    }
    #[test]
    fn genuine_root_reconstruction_applies_nonzero_authored_offset_before_actual_native_ift() {
        let (factory, proof, original, normalization) = source();
        let flag = factory.cancel.clone();
        let supplier = ReconstructionFactory::new(
            ImplicitFactory::Root(factory),
            SelectedResidualRealization::AuthoredValues,
            Some(proof),
            Some(Arc::new(CheckOffset)),
        );
        let binding = SelectedResidualBinding::for_source(
            &supplier,
            &original,
            &[GlobalRow::new(0)],
            supplier.realization().clone(),
        )
        .unwrap();
        let contract = SelectedImplicitReconstruction::<MathError>::prepare_contract_with_binding(
            &supplier,
            original,
            vec![GlobalCol::new(0)],
            vec![GlobalRow::new(0)],
            ContentHash::from_bytes([3; 32]),
            &flag,
            &binding,
        )
        .unwrap();
        let mut worker = SelectedImplicitReconstruction::<MathError>::new_with_binding(
            &supplier,
            contract,
            normalization,
            ExecutionScope::new(flag, None),
            &binding,
        )
        .unwrap();
        assert!(matches!(worker.selection(), SupplierSelection::Root(_)));
        assert!(matches!(
            worker.admit(&[2.0]),
            Err(MathError::Refinement {
                reason: crate::derived::RefinementRefusal::Unavailable(
                    SelectionProofRefusal::Unsupported
                ),
                ..
            })
        ));
    }
    #[test]
    fn root_supplier_requires_producer_metadata_and_preserves_its_identity() {
        let (factory, proof, original, _) = source();
        let absent = ReconstructionFactory::new(
            ImplicitFactory::Root(factory.clone()),
            SelectedResidualRealization::AuthoredValues,
            None,
            None,
        );
        assert!(!absent.supports_reconstruction());
        let supplied = ReconstructionFactory::new(
            ImplicitFactory::Root(factory),
            SelectedResidualRealization::ZeroResiduals {
                authored_offsets: vec![(original.constraints()[0].id, 3.0)],
            },
            Some(proof),
            Some(Arc::new(CheckOffset)),
        );
        assert!(supplied.supports_reconstruction());
        assert_ne!(absent.configuration_key(), supplied.configuration_key());
        SelectedResidualBinding::for_source(
            &supplied,
            &original,
            &[GlobalRow::new(0)],
            supplied.realization().clone(),
        )
        .unwrap();
        assert!(
            SelectedResidualBinding::for_source(
                &supplied,
                &original,
                &[GlobalRow::new(0)],
                SelectedResidualRealization::AuthoredValues
            )
            .is_err(),
            "consumer cannot change producer-issued subtraction"
        );
        assert!(
            SelectedResidualBinding::for_source(
                &supplied,
                &original,
                &[GlobalRow::new(0)],
                SelectedResidualRealization::ZeroResiduals {
                    authored_offsets: vec![(original.constraints()[0].id, 6.0)]
                }
            )
            .is_err()
        );
    }
}
