// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The backend-execution seam (S18). One adapter per `Backend` owns its pse-owned settings
//! type, one capability record, admission of its settings, its native session lifecycle
//! on the owning worker, its warm-start payload and its typed evidence. A static table maps
//! every registry `Backend` to its adapter; there is no string lookup. Routing, the
//! published `runtime.solver_capabilities` inventory and the representation runners
//! consume only this seam, so adding a backend is one adapter, one registry value and its
//! capability record, with no workflow edits.
use crate::{
    CoefficientProblem, ConicProblem, NleOracle, NlpOracle, OracleContract, ProblemError,
    quality::Tolerances,
    routing::{Ineligible, Requirements},
    solve::{
        Backend, Compatibility, Controls, DerivativeCapability, Execution, ProblemClass,
        ResolvedAccuracy, SolveReport, WarmCapability, WarmPayload, WarmStart,
    },
};
use pse_ids::{ContentHash, SemanticId};
use pse_math::{
    binding::ObjectiveSense, convexity::QuadraticEvidence, normalization::Normalization,
    presolve::GuardSign,
};
use std::{any::Any, collections::BTreeMap};

mod clarabel;
mod dynamics;
mod highs;
mod ipopt;
mod kinsol;
mod pounce;
mod runner;
pub use runner::{
    Coefficients, Evaluation, Nlp, OriginalModel, Roots, Step, coefficients, cone, nlp, roots,
};

/// The native input an adapter consumes. Runners build exactly this representation, so a
/// workflow selects a runner by representation, never by backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Representation {
    /// Smooth NLP callbacks through the shared presolve pipeline ([`NlpOracle`]).
    Nlp,
    /// Square residual equations in normalized coordinates ([`NleOracle`]).
    Roots,
    /// Sparse LP/MILP/QP coefficients in normalized coordinates ([`CoefficientProblem`]).
    Coefficients,
    /// Explicit cone data in normalized coordinates ([`ConicProblem`]).
    Cone,
    /// Trajectories owned by the integrator workflows; never algebraically routed.
    Trajectory,
}
impl Representation {
    /// Whether the algebraic router assesses adapters of this representation.
    pub const fn algebraic(self) -> bool {
        !matches!(self, Self::Trajectory)
    }
}

/// The one capability record of an adapter (F21). Routing eligibility and the published
/// `runtime.solver_capabilities` row both derive from these fields; nothing else grants
/// eligibility. Model-specific admission is a separate, contextual result.
#[derive(Clone, Copy, Debug)]
pub struct Capability {
    /// Representable mathematical classes.
    pub classes: &'static [ProblemClass],
    /// Required derivative representation.
    pub derivatives: DerivativeCapability,
    /// Externally supplied starting-state support.
    pub warm: WarmCapability,
    /// Arbitrary variable bounds are representable.
    pub general_bounds: bool,
    /// Exact sign bounds are representable when general bounds are not.
    pub sign_bounds: bool,
    /// The adapter can consume more than one admitted native thread.
    pub parallel: bool,
    /// Native allocation/data reuse boundary.
    pub reuse: &'static str,
    /// Actual interrupt checkpoints.
    pub cancellation: &'static str,
    /// Available native diagnostic families.
    pub diagnostics: &'static str,
}
impl Capability {
    /// The published inventory row of this record.
    pub fn row(&self, backend: Backend) -> pse_model::generated::runtime::solver_capabilities::Row {
        pse_model::generated::runtime::solver_capabilities::Row {
            backend,
            classes: self.classes.to_vec(),
            derivatives: self.derivatives,
            warm: self.warm,
            reuse: self.reuse.into(),
            cancellation: self.cancellation.into(),
            diagnostics: self.diagnostics.into(),
            general_bounds: self.general_bounds,
            sign_bounds: self.sign_bounds,
            parallel: self.parallel,
        }
    }
}

/// Original acceptance budgets, coordinate transport and the normalized feasibility budget
/// of one solved function: the policy inputs an adapter derives native scaling from.
#[derive(Clone, Copy, Debug)]
pub struct Budgets<'a> {
    /// Original physical acceptance budgets.
    pub tolerances: &'a Tolerances,
    /// Coordinate transport of the function the adapter evaluates.
    pub normalization: &'a Normalization,
    /// Normalized feasibility budget (`ResolvedAccuracy::feasibility`).
    pub feasibility: f64,
}

/// One execution of a selected adapter on the owning worker, in native coordinates.
pub struct Input<'a> {
    /// The adapter's representation.
    pub problem: Problem<'a>,
    /// Finite shared controls.
    pub controls: &'a Controls,
    /// Stopping budgets resolved for this attempt.
    pub accuracy: &'a ResolvedAccuracy,
    /// Typed settings, admitted for this adapter.
    pub settings: &'a BackendSettings,
    /// Cancellation, deadline and bounded progress.
    pub execution: Execution,
    /// Native-coordinate acceptance budgets.
    pub tolerances: &'a Tolerances,
    /// Native-coordinate seed, when one is submitted.
    pub warm: Option<&'a WarmStart>,
    /// Native reuse identity.
    pub compatibility: Compatibility,
}
/// Representation-specific native input.
pub enum Problem<'a> {
    /// Preprocessed NLP transport from the shared pipeline.
    Nlp {
        /// Minimization callbacks in native coordinates.
        oracle: Box<dyn NlpOracle>,
        /// Projected start.
        initial: &'a [f64],
        /// Authored sense, used only to recover authored output.
        sense: ObjectiveSense,
    },
    /// Square residual equations in normalized coordinates.
    Roots {
        /// Normalized residual transport.
        oracle: Box<dyn NleOracle>,
        /// Normalized start.
        initial: &'a [f64],
        /// Policy inputs for native scaling.
        budgets: Budgets<'a>,
        /// Owner of the evaluators behind `oracle`; retained as long as a session retains it.
        owner: Option<Box<dyn Any>>,
    },
    /// Normalized coefficient model.
    Coefficients {
        /// Normalized coefficients.
        problem: &'a CoefficientProblem,
        /// Transported convexity evidence for a quadratic objective.
        certificate: Option<&'a dyn QuadraticEvidence>,
        /// Model coordinate transport, for native diagnostics and sparse starts.
        normalization: &'a Normalization,
        /// Original row constants, for native diagnostics.
        row_constants: &'a [f64],
    },
    /// Normalized explicit cone model.
    Cone {
        /// Normalized cone data.
        problem: &'a ConicProblem,
        /// Transported convexity evidence.
        certificate: &'a dyn QuadraticEvidence,
    },
}

impl std::fmt::Debug for Input<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Input")
            .field("problem", &self.problem)
            .field("settings", &self.settings)
            .field("compatibility", &self.compatibility)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for Problem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Nlp { .. } => "Problem::Nlp",
            Self::Roots { .. } => "Problem::Roots",
            Self::Coefficients { .. } => "Problem::Coefficients",
            Self::Cone { .. } => "Problem::Cone",
        })
    }
}

/// The backend-execution adapter. Implementations are unit values in the static table.
pub trait BackendExecution: Sync + std::fmt::Debug {
    /// Registry identity of this adapter.
    fn backend(&self) -> Backend;
    /// The one capability record.
    fn capability(&self) -> &'static Capability;
    /// Native input this adapter consumes.
    fn representation(&self) -> Representation;
    /// Whether this binary links the native implementation.
    fn linked(&self) -> bool;
    /// Automatic-selection preference among eligible adapters (lower first); `None`
    /// is explicit-only. It orders a choice and never grants eligibility.
    fn automatic(&self) -> Option<u8>;
    /// Contextual eligibility with every typed reason; empty means eligible. It derives
    /// only from the capability record and linkage, so the published row is the routing
    /// rule; adapters do not override it.
    fn admit(&self, requirements: &Requirements<'_>) -> Vec<Ineligible> {
        crate::routing::admit(self.capability(), self.linked(), requirements)
    }
    /// Typed settings and native controls of a selected route: settings must belong to
    /// this adapter and the thread count to its capability.
    ///
    /// # Errors
    /// Foreign settings or an unadmitted thread count.
    fn admit_settings(
        &self,
        settings: &BackendSettings,
        controls: &Controls,
    ) -> Result<(), ProblemError> {
        if settings.backend().is_some_and(|b| b != self.backend()) {
            return Err(ProblemError::Contract(
                "backend settings do not match selected route".into(),
            ));
        }
        if controls.threads != 1 && !self.capability().parallel {
            return Err(ProblemError::Unsupported(
                "selected linked native profile is serial".into(),
            ));
        }
        Ok(())
    }
    /// Model-contract admission of a selected route before any worker exists.
    ///
    /// # Errors
    /// The contract, guards or settings cannot be represented natively.
    fn admit_contract(
        &self,
        _contract: &OracleContract,
        _guards: &BTreeMap<SemanticId, GuardSign>,
        _settings: &BackendSettings,
        _budgets: Budgets<'_>,
    ) -> Result<(), ProblemError> {
        Ok(())
    }
    /// This adapter's warm-start payload for a primal-only explicit start.
    ///
    /// # Errors
    /// The adapter has no primal-start interface.
    fn primal_start(&self, _primal: Vec<f64>) -> Result<WarmPayload, ProblemError> {
        Err(ProblemError::Unsupported(
            "selected backend has no primal-start interface".into(),
        ))
    }
    /// Whether `payload` is this adapter's warm-start variant.
    fn accepts(&self, _payload: &WarmPayload) -> bool {
        false
    }
    /// Run `work` inside this adapter's execution scope, such as an admitted local thread
    /// pool. Sessions built inside the scope are also torn down inside it.
    ///
    /// # Errors
    /// The scope could not be entered.
    fn scope(
        &self,
        _threads: usize,
        _stack: usize,
        work: &mut (dyn FnMut() + Send),
    ) -> Result<(), ProblemError> {
        work();
        Ok(())
    }
    /// Prepare (reusing compatible retained state), solve and retain on the owning worker.
    ///
    /// # Errors
    /// Refused input, unavailable reuse or a failed native call.
    fn execute(
        &self,
        retained: &mut Retained,
        input: Input<'_>,
    ) -> Result<SolveReport, ProblemError>;
}

/// The adapter of every registry `Backend`: a static map with an exhaustive match, so a
/// new registry value cannot compile without its adapter.
pub const fn adapter(backend: Backend) -> &'static dyn BackendExecution {
    match backend {
        Backend::Ipopt => &ipopt::ADAPTER,
        Backend::Pounce => &pounce::ADAPTER,
        Backend::Kinsol => &kinsol::ADAPTER,
        Backend::Highs => &highs::ADAPTER,
        Backend::Clarabel => &clarabel::ADAPTER,
        Backend::Diffsol => &dynamics::DIFFSOL,
        Backend::Idas => &dynamics::IDAS,
    }
}
static ADAPTERS: [&dyn BackendExecution; Backend::ALL.len()] = {
    let mut out: [&dyn BackendExecution; Backend::ALL.len()] =
        [adapter(Backend::Ipopt); Backend::ALL.len()];
    let mut i = 0;
    while i < Backend::ALL.len() {
        out[i] = adapter(Backend::ALL[i]);
        i += 1;
    }
    out
};
/// Every adapter in registry order, linked or not.
pub static LINKED: Table = Table::new(&ADAPTERS);

/// An adapter table in a fixed order. Production uses [`LINKED`]; a caller may expose a
/// subset, and a test may expose its own adapter.
#[derive(Clone, Copy, Debug)]
pub struct Table {
    adapters: &'static [&'static dyn BackendExecution],
}
impl Table {
    /// A table over exactly these adapters, in this order.
    pub const fn new(adapters: &'static [&'static dyn BackendExecution]) -> Self {
        Self { adapters }
    }
    /// The adapter registered for `backend`.
    pub fn get(&self, backend: Backend) -> Option<&'static dyn BackendExecution> {
        self.adapters
            .iter()
            .copied()
            .find(|a| a.backend() == backend)
    }
    /// Every adapter in table order.
    pub fn adapters(&self) -> impl Iterator<Item = &'static dyn BackendExecution> + '_ {
        self.adapters.iter().copied()
    }
    /// The published inventory: one row per linked adapter.
    pub fn published(&self) -> Vec<pse_model::generated::runtime::solver_capabilities::Row> {
        self.adapters()
            .filter(|a| a.linked())
            .map(|a| a.capability().row(a.backend()))
            .collect()
    }
}

/// Typed backend settings of a request. Each variant is its adapter's pse-owned settings
/// type, and identity derives from serde, never from a hand-written field list (F09).
#[derive(Clone, Debug, Default, serde::Serialize)]
pub enum BackendSettings {
    /// Native defaults and the common semantic controls on the routed backend.
    #[default]
    Default,
    /// POUNCE method and complete native FERAL configuration.
    #[cfg(feature = "pounce")]
    Pounce(crate::pounce::Settings),
    /// KINSOL method controls; scales and the step tolerance derive from policy.
    #[cfg(feature = "kinsol")]
    Kinsol(crate::kinsol::Method),
    /// HiGHS method, opt-in diagnostics and partial MIP start.
    #[cfg(feature = "highs")]
    Highs(crate::highs::Settings),
    /// Clarabel's complete settings and preprocessing/data-update mode.
    Clarabel {
        /// Native settings.
        native: Box<crate::conic::Settings>,
        /// Reuse/preprocessing mode.
        mode: crate::conic::Mode,
    },
}
impl BackendSettings {
    /// The backend these settings belong to; `None` for native defaults.
    pub fn backend(&self) -> Option<Backend> {
        match self {
            Self::Default => None,
            #[cfg(feature = "pounce")]
            Self::Pounce(_) => Some(Backend::Pounce),
            #[cfg(feature = "kinsol")]
            Self::Kinsol(_) => Some(Backend::Kinsol),
            #[cfg(feature = "highs")]
            Self::Highs(_) => Some(Backend::Highs),
            Self::Clarabel { .. } => Some(Backend::Clarabel),
        }
    }
    /// The partial explicit start these settings submit, in original coordinates.
    pub fn partial_start(&self) -> Option<&BTreeMap<SemanticId, f64>> {
        match self {
            #[cfg(feature = "highs")]
            Self::Highs(settings) => settings.sparse_start.as_ref(),
            _ => None,
        }
    }
    /// Complete settings identity, derived from serde.
    ///
    /// # Errors
    /// A native settings serializer refused its value.
    pub fn identity(&self) -> Result<ContentHash, ProblemError> {
        crate::identity::of("pse.backend.settings.v1", self)
    }
}

/// Worker-owned native state retained from the previous step of a finite sequence.
/// Opaque to workflows: each adapter recognizes only its own session. It never crosses a
/// worker boundary and is dropped on the owning thread.
#[derive(Default)]
pub struct Retained {
    session: Option<(Backend, Box<dyn Any>)>,
}
impl std::fmt::Debug for Retained {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Retained")
            .field("backend", &self.session.as_ref().map(|s| s.0))
            .finish_non_exhaustive()
    }
}
impl Retained {
    /// Drop retained native state now.
    pub fn clear(&mut self) {
        self.session = None;
    }
    /// Whether any adapter retains native state.
    pub fn is_empty(&self) -> bool {
        self.session.is_none()
    }
    /// The backend whose native state is retained.
    pub fn backend(&self) -> Option<Backend> {
        self.session.as_ref().map(|s| s.0)
    }
    /// Reuse the retained session of `backend` when `reuse` accepts it (refreshing it with
    /// this step's data); otherwise tear the old state down and build a new session when
    /// `policy` allows. Returns the session and whether it was reused.
    ///
    /// # Errors
    /// `RequireReuse` with incompatible retained state, or a failed update or build.
    pub fn session<S: Any>(
        &mut self,
        backend: Backend,
        policy: crate::solve::ReusePolicy,
        reuse: impl FnOnce(&mut S) -> Result<bool, ProblemError>,
        build: impl FnOnce() -> Result<S, ProblemError>,
    ) -> Result<(&mut S, bool), ProblemError> {
        let reused = match &mut self.session {
            Some((held, state)) if *held == backend => match state.downcast_mut::<S>() {
                Some(session) => reuse(session)?,
                None => false,
            },
            _ => false,
        };
        if !reused {
            if policy == crate::solve::ReusePolicy::RequireReuse && self.session.is_some() {
                return Err(ProblemError::Unsupported(format!(
                    "required {} reuse unavailable",
                    backend.as_str()
                )));
            }
            // Native teardown precedes construction of the replacement.
            self.session = None;
            self.session = Some((backend, Box::new(build()?)));
        }
        let session = self
            .session
            .as_mut()
            .and_then(|(_, state)| state.downcast_mut::<S>())
            .ok_or_else(|| ProblemError::Internal("lost retained native session".into()))?;
        Ok((session, reused))
    }
}

/// Run `work` inside the scope of every distinct adapter it executes, such as an admitted
/// local thread pool, so native sessions are built and torn down inside those scopes.
///
/// # Errors
/// A scope could not be entered, or `work` failed.
pub fn scoped<T: Send, E: From<ProblemError> + Send>(
    adapters: &[&dyn BackendExecution],
    threads: usize,
    stack: usize,
    work: impl FnOnce() -> Result<T, E> + Send,
) -> Result<T, E> {
    let mut distinct: Vec<&dyn BackendExecution> = Vec::with_capacity(adapters.len());
    for adapter in adapters {
        if !distinct.iter().any(|a| a.backend() == adapter.backend()) {
            distinct.push(*adapter);
        }
    }
    let mut work = Some(work);
    let mut out = None;
    let mut run = || {
        if let Some(work) = work.take() {
            out = Some(work());
        }
    };
    enter(&distinct, threads, stack, &mut run)?;
    out.unwrap_or_else(|| {
        Err(ProblemError::Internal("execution scope did not run its work".into()).into())
    })
}
fn enter(
    adapters: &[&dyn BackendExecution],
    threads: usize,
    stack: usize,
    work: &mut (dyn FnMut() + Send),
) -> Result<(), ProblemError> {
    let Some((first, rest)) = adapters.split_first() else {
        work();
        return Ok(());
    };
    let mut inner = Ok(());
    first.scope(threads, stack, &mut || {
        inner = enter(rest, threads, stack, work);
    })?;
    inner
}

/// Another adapter's settings variant reached `backend`.
fn foreign(backend: Backend) -> ProblemError {
    ProblemError::Contract(format!("wrong {} settings", backend.as_str()))
}
#[cfg(not(all(
    feature = "ipopt",
    feature = "pounce",
    feature = "kinsol",
    feature = "highs"
)))]
fn unlinked(backend: Backend) -> ProblemError {
    ProblemError::Unavailable {
        backend,
        alternatives: vec![],
    }
}
fn representation(backend: Backend) -> ProblemError {
    ProblemError::Internal(format!(
        "{} received another adapter's representation",
        backend.as_str()
    ))
}

#[cfg(test)]
mod tests;
