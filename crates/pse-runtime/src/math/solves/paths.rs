// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Prepared scalar paths. Library correctors own iteration; auxiliary path points
//! remain starts for the original completion owner.
use super::*;
use crate::math::prediction::Proposal;
use native::{NlpOracle, kkt::path::arclength};
use pse_ids::{ContentHash, SemanticId};
use pse_kernels::{DerivativeOrder, ExecutionScope};
use pse_math::{
    continuation::{ArclengthFamily, Parameter, ParameterizedOracle},
    derived::{Constraint, Coordinate, DerivativeSupport, OriginalContract, OriginalObligations},
    implicit::{
        ChartChainCoverage, ChartChainEvidence, ChartChainRequest, SelectionAlternative,
        SelectionChart, SelectionEvidence, SelectionProofRequest, SelectionScope,
        SelectionVerifier,
    },
    index::{Entry, GlobalCol, GlobalRow},
};
use pse_model::strategy::{BranchPolicy, SemanticProductKey};
use std::sync::atomic::Ordering;

/// Explicit optional numerical-event work; all observations remain estimated.
#[derive(Clone, Debug)]
pub struct EventPolicy {
    /// Complete event observation cap, including explicitly requested endpoints.
    pub observations: usize,
    /// Observe each actual original/accepted endpoint, even without a turning bracket.
    pub endpoints: bool,
    /// Maximum existing-native-corrector calls for each turning bracket.
    pub localization_steps: usize,
    /// Positive normalized source-hyperplane interval width requested for localization.
    pub localization_tolerance: f64,
    /// Absolute normalized singular-value threshold.
    pub rank_threshold: f64,
    /// Required observed F_p/F_xx projection magnitude; no formal rank claim.
    pub nondegeneracy_threshold: f64,
    /// Explicit finite library SVD/storage allowance per observation.
    pub probe: arclength::ProbeLimits,
}
impl EventPolicy {
    fn validate(&self) -> Result<(), ProblemError> {
        if self.observations == 0
            || self.localization_steps == 0
            || self.probe.bytes == 0
            || self.probe.svds < 2
            || [
                self.localization_tolerance,
                self.rank_threshold,
                self.nondegeneracy_threshold,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.)
        {
            return Err(ProblemError::Contract(
                "finite positive path event/localization allowances required".into(),
            ));
        }
        Ok(())
    }
}
/// Genuine compiled Second projection, verified against the frozen original First source.
#[derive(Clone, Debug)]
pub struct PathCurvature(Arc<CurvatureData>);
#[derive(Debug)]
struct CurvatureData {
    source: ContentHash,
    key: ContentHash,
    parameter: SemanticId,
    program: Arc<ExecutableCase>,
    scope: ExecutionScope,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PathCurvature {
    /// Actual original source preparation; not a caller-supplied equation identity.
    pub fn source(&self) -> ContentHash {
        self.0.source
    }
    /// Consumed actual Second source/program identity.
    pub fn key(&self) -> ContentHash {
        self.0.key
    }
    fn worker_bytes(&self) -> Result<usize, MathRuntimeError> {
        self.0
            .program
            .assembly
            .numeric_worker_bytes()
            .checked_add(
                self.0
                    .program
                    .assembly
                    .hessian_pattern()
                    .row_idx()
                    .len()
                    .checked_mul(8)
                    .ok_or(MathRuntimeError::Limit("path curvature scratch extent"))?,
            )
            .and_then(|v| v.checked_add(self.0.program.assembly.columns().len().checked_mul(32)?))
            .ok_or(MathRuntimeError::Limit("path curvature worker extent"))
    }
}
impl MathService {
    /// Bind an actual compiler-issued Second parametric executable to the frozen
    /// original First source, including its bodies, guards, units and row/state maps.
    /// This does not upgrade a First program or compile an invented derivative.
    /// # Errors
    /// Missing actual Second support, source/coordinate mismatch or original scope/resources.
    pub async fn prepare_path_curvature(
        self: &Arc<Self>,
        original: PreparedSolve,
        second: Arc<ExecutableCase>,
        scope: ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<PathCurvature, MathRuntimeError> {
        driver
            .checkpoint()
            .map_err(|_| MathRuntimeError::Cancelled)?;
        scope.check().map_err(ProblemError::Provider)?;
        if scope.deadline().is_none()
            || original.task_scope().as_ref().is_some_and(|bound| {
                !Arc::ptr_eq(bound.cancellation(), scope.cancellation())
                    || bound.deadline() != scope.deadline()
            })
        {
            return Err(ProblemError::Contract(
                "path curvature requires the same finite original scope".into(),
            )
            .into());
        }
        let Representation::Algebraic(case) = &original.representation else {
            return Err(ProblemError::Unsupported(
                "path curvature requires compiled original algebraic source".into(),
            )
            .into());
        };
        let first = case
            .sensitivity
            .as_ref()
            .and_then(ParametricPreparation::available)
            .ok_or_else(|| {
                ProblemError::Unsupported(
                    "path curvature requires an actual scalar First program".into(),
                )
            })?;
        if first.parameters.len() != 1
            || second.assembly.order() < DerivativeOrder::Second
            || second.assembly.available_order() < DerivativeOrder::Second
            || second.assembly.has_directional_actions()
        {
            return Err(ProblemError::Unsupported(
                "path curvature requires a supplied full actual Second program".into(),
            )
            .into());
        }
        let source = original.preparation_identity()?;
        let parameter = first.parameters[0].0;
        let base = first.program.clone();
        let quantities = case.prepared.compiled().quantities.clone();
        let checked = second.clone();
        let check_scope = scope.clone();
        let control = FlightCancellation::default();
        let operation=self.job_retained_scoped(1,self.policy.workspace_bytes,control.clone(),scope.deadline(),move |_| {
            check_scope.check().map_err(ProblemError::Provider)?;
            if !checked.assembly.is_derivative_projection_of(&base.assembly,&quantities,check_scope.cancellation())? {return Err(ProblemError::Contract("path curvature differs from frozen original formulas, guards, quantities or physical maps".into()).into());}
            check_scope.check().map_err(ProblemError::Provider)?;
            let mut h=FramedHasher::new(pse_ids::Frame::DerivedBindingV1);h.str("actual-original-second-path-curvature").hash(&source).id(&parameter).u64(checked.assembly.order() as u64);
            for demand in checked.assembly.demands() {h.hash(&demand.body).u64(demand.order as u64);for v in &demand.outputs {h.u64(*v as u64);}for v in &demand.coordinates {h.u64(*v as u64);}}
            Ok((h.finish_hash(),size_of::<CurvatureData>()))
        });
        tokio::pin!(operation);
        let (key, owner) = tokio::select! {result=&mut operation=>result?,()=driver.cancelled()=>{scope.cancellation().store(true,Ordering::Release);control.cancel();let _=operation.await;return Err(MathRuntimeError::Cancelled);}};
        scope.check().map_err(ProblemError::Provider)?;
        Ok(PathCurvature(Arc::new(CurvatureData {
            source,
            key,
            parameter,
            program: second,
            scope,
            _owner: owner,
        })))
    }
}

/// Explicit geometry and finite operation allowance, independent of shared task counters.
#[derive(Clone, Debug)]
pub struct PathPolicy {
    /// Maximum accepted auxiliary segments.
    pub steps: usize,
    /// Maximum failed numerical trials that may subdivide across the complete operation.
    pub subdivisions: usize,
    /// Positive normalized arclength displacement per segment.
    pub step: f64,
    /// Positive smallest permitted displacement.
    pub minimum_step: f64,
    /// Each sparse tangent's admitted action and storage allowance.
    pub tangent: arclength::Limits,
    /// Required scaled bordered backward error.
    pub backward_limit: f64,
    /// Physical scalar bound budget, separately declared from state budgets.
    pub parameter_tolerance: f64,
    /// Dimensionless hyperplane budget.
    pub hyperplane_tolerance: f64,
    /// Optional explicitly bounded numerical event observation/localization.
    pub events: Option<EventPolicy>,
}
impl PathPolicy {
    fn validate(&self) -> Result<(), ProblemError> {
        if self.steps == 0
            || self.tangent.bytes == 0
            || [
                self.step,
                self.minimum_step,
                self.backward_limit,
                self.parameter_tolerance,
                self.hyperplane_tolerance,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.)
            || self.minimum_step > self.step
        {
            return Err(ProblemError::Contract(
                "finite positive path geometry/accuracy required".into(),
            ));
        }
        if let Some(events) = &self.events {
            events.validate()?;
        }
        Ok(())
    }
}
/// Owned original selected-function proof inputs. The actual verifier supplies mathematics;
/// this declaration never synthesizes a chart from numerical feasibility.
#[derive(Clone, Debug)]
pub struct PathSelection(Arc<SelectionData>);
/// Immutable source-issued chart/program payload shared under its retained owner.
#[doc(hidden)]
#[derive(Debug)]
pub struct SelectionData {
    /// Exact source preparation, including providers and numerical meaning.
    source: ContentHash,
    /// Explicit independent scalar coordinate.
    parameter: SemanticId,
    /// Exact original state coordinate order of every winning chart.
    coordinates: Vec<SemanticId>,
    /// Library-owned validating mathematics, including rounding identity.
    verifier: Arc<dyn SelectionVerifier>,
    /// Independently certified originating selected chart.
    previous: SelectionChart,
    /// Full genuine alternative union, with owned programs and physical domains.
    alternatives: Vec<SelectionScope>,
    /// Original alternative index for this connected sheet.
    winner: usize,
    /// Required continuity coverage, distinct from numerical path continuity.
    coverage: ChartChainCoverage,
    _owner: Option<Arc<dyn pse_math::AllocationOwner>>,
}
/// Source-bound scalar family request. Geometry has no default scientific interpretation.
#[derive(Clone, Debug)]
pub struct PathRequest {
    /// Explicit differentiated input, already present in the prepared First program.
    pub parameter: SemanticId,
    /// Finite physical family interval.
    pub interval: (f64, f64),
    /// Positive physical parameter nominal.
    pub scale: f64,
    /// Explicit mechanical border row identity.
    pub hyperplane: SemanticId,
    /// Explicit auxiliary corrector profile; bounded parameter requires general bounds.
    pub corrector: SolverProfile,
    /// Original target receiving only physical start values.
    pub target: PreparedSolve,
    /// Original branch requirement, preserved on the final start.
    pub branch: BranchPolicy,
    /// Optional actual chart transport supplier; required for Connected.
    pub selection: Option<PathSelection>,
    /// Genuine separately compiled Second projection, when actual curvature is requested.
    pub curvature: Option<PathCurvature>,
    /// Bounded numerical geometry.
    pub policy: PathPolicy,
}
/// Immutable family and the consumed original preparation owners.
#[derive(Clone, Debug)]
pub struct PreparedPath {
    data: Arc<PathData>,
    branch: BranchPolicy,
}
/// Immutable prepared path storage shared by clones under its allocation owner.
#[doc(hidden)]
#[derive(Debug)]
pub struct PathData {
    original: PreparedSolve,
    request: PathRequest,
    family: Arc<ArclengthFamily>,
    source: ContentHash,
    target_identity: ContentHash,
    native_profile: ContentHash,
    program: SensitivityProgram,
    case: AlgebraicCase,
    normalization: Normalization,
    tolerances: Tolerances,
    accuracy: ResolvedAccuracy,
    structure: routing::Structure,
    scope: ExecutionScope,
    task_scope: ExecutionScope,
    _owner: Arc<pse_columnar::AllocationLease>,
    workspace_bytes: usize,
}
impl std::ops::Deref for PreparedPath {
    type Target = PathData;
    fn deref(&self) -> &PathData {
        &self.data
    }
}
impl PreparedPath {
    /// Bound original target; its completion identity remains separate from the path source.
    pub fn original_target(&self) -> &PreparedSolve {
        &self.request.target
    }
    /// Original cancellation owner and finite deadline consumed throughout the path.
    pub fn task_scope(&self) -> &ExecutionScope {
        &self.task_scope
    }
    /// Complete explicitly admitted auxiliary native profile.
    pub fn profile(&self) -> &SolverProfile {
        &self.request.corrector
    }
    /// Family, frozen source, target and explicit operation declaration identity.
    pub fn key(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("prepared-source-bound-root-path")
            .hash(&self.family.key())
            .hash(&self.source)
            .hash(&self.target_identity)
            .hash(&self.native_profile);
        if let Some(branch) = self.branch.connected {
            h.bool(true)
                .hash(&branch.path)
                .hash(&branch.sheet)
                .hash(&branch.transport)
                .hash(&branch.orientation);
        } else {
            h.bool(false);
        }
        if let Some(selection) = &self.request.selection {
            h.bool(true)
                .hash(&selection.verifier.identity())
                .hash(&chart_key(&selection.previous))
                .u64(match selection.coverage {
                    ChartChainCoverage::RootSheet => 0,
                    ChartChainCoverage::SelectedFunction => 1,
                });
        } else {
            h.bool(false);
        }
        let p = &self.request.policy;
        h.u64(p.steps as u64)
            .u64(p.subdivisions as u64)
            .f64(p.step)
            .f64(p.minimum_step)
            .u64(p.tangent.actions as u64)
            .u64(p.tangent.bytes as u64)
            .f64(p.backward_limit)
            .f64(p.parameter_tolerance)
            .f64(p.hyperplane_tolerance);
        if let Some(e) = &p.events {
            h.bool(true)
                .u64(e.observations as u64)
                .bool(e.endpoints)
                .u64(e.localization_steps as u64)
                .f64(e.localization_tolerance)
                .f64(e.rank_threshold)
                .f64(e.nondegeneracy_threshold)
                .u64(e.probe.bytes as u64)
                .u64(e.probe.svds as u64);
        } else {
            h.bool(false);
        }
        if let Some(curvature) = &self.request.curvature {
            h.bool(true).hash(&curvature.key());
        } else {
            h.bool(false);
        }
        h.finish_hash()
    }
    /// Actual consumed source/family support products.
    pub fn support(&self) -> std::collections::BTreeSet<ContentHash> {
        let mut keys = std::collections::BTreeSet::from([self.source, self.family.key()]);
        if let Some(curvature) = &self.request.curvature {
            keys.insert(curvature.key());
        }
        keys
    }
    /// Finite complete worker allowance, including actual optional verifier workspace.
    pub(crate) fn worker_bytes(&self) -> usize {
        self.workspace_bytes
    }
    /// Conservative copied report/proposal allowance for this complete bounded operation.
    pub(crate) fn result_bytes(&self) -> Result<usize, MathRuntimeError> {
        let event_calls = self
            .request
            .policy
            .events
            .as_ref()
            .map(|e| {
                e.observations
                    .checked_mul(e.localization_steps)
                    .ok_or(MathRuntimeError::Limit("path event call extent"))
            })
            .transpose()?
            .unwrap_or(0);
        let count = self
            .request
            .policy
            .steps
            .checked_add(self.request.policy.subdivisions)
            .and_then(|n| n.checked_add(usize::from(self.branch.connected.is_some())))
            .and_then(|n| n.checked_add(event_calls))
            .ok_or(MathRuntimeError::Limit("path attempt count"))?;
        let event_bytes = self
            .request
            .policy
            .events
            .as_ref()
            .map(|e| {
                self.family
                    .coordinates()
                    .len()
                    .checked_mul(128)
                    .and_then(|n| {
                        n.checked_add(size_of::<arclength::Event>() + size_of::<arclength::Work>())
                    })
                    .and_then(|n| n.checked_mul(e.observations))
                    .ok_or(MathRuntimeError::Limit("path event result extent"))
            })
            .transpose()?
            .unwrap_or(0);
        count
            .checked_mul(
                self.request
                    .corrector
                    .controls
                    .report_allowance()?
                    .checked_add(
                        (self.family.coordinates().len() + self.family.constraints().len())
                            .checked_mul(512)
                            .ok_or(MathRuntimeError::Limit("path report extent"))?,
                    )
                    .ok_or(MathRuntimeError::Limit("path report extent"))?,
            )
            .and_then(|v| v.checked_add(size_of::<PathOutcome>()))
            .and_then(|v| v.checked_add(event_bytes))
            .ok_or(MathRuntimeError::Limit("path copied result extent"))
    }
    /// Require actual RootSheet transport from this original-permitted origin. The
    /// source-issued chart supplies the prior product; future endpoint evidence is
    /// produced by execution and is never predicted or supplied by the caller.
    /// # Errors
    /// Missing genuine chart supplier or a changed original endpoint/source.
    pub fn require_connected(mut self, start: &PathStart) -> Result<Self, ProblemError> {
        if self.original.semantic_point_key(start.point())? != start.source() {
            return Err(ProblemError::Contract(
                "connected path origin/source mismatch".into(),
            ));
        }
        let selection = self.request.selection.as_ref().ok_or_else(|| {
            ProblemError::Unsupported(
                "connected path requires a source-issued chart supplier".into(),
            )
        })?;
        let point = augmented(start.point(), self.program.parameters[0].1);
        let orientation = orientation_key(self.source, &point, start.orientation());
        let branch = BranchPolicy {
            kind: pse_model::strategy::BranchKind::Connected,
            connected: Some(pse_model::strategy::ConnectedPath {
                path: self.family.key(),
                sheet: chart_key(&selection.previous),
                transport: origin_transport(&self, &point, orientation),
                orientation,
            }),
        };
        self.branch = branch;
        Ok(self)
    }
    /// Actual entry branch declaration; execution emits a fresh endpoint product.
    pub fn branch(&self) -> BranchPolicy {
        self.branch
    }
    /// Validate the declared prior product against this source-issued original chart.
    /// # Errors
    /// Changed source/point, expired original scope, missing chart or mismatched prior product.
    pub fn origin_connected(&self, start: &PathStart) -> Result<bool, ProblemError> {
        let Some(required) = self.branch.connected else {
            return Ok(false);
        };
        self.scope.check().map_err(ProblemError::Provider)?;
        if self.original.semantic_point_key(start.point())? != start.source() {
            return Err(ProblemError::Contract(
                "connected path original origin mismatch".into(),
            ));
        }
        let selection = self.request.selection.as_ref().ok_or_else(|| {
            ProblemError::Unsupported("connected source chart unavailable".into())
        })?;
        let point = augmented(start.point(), self.program.parameters[0].1);
        let orientation = orientation_key(self.source, &point, start.orientation());
        if required.path != self.family.key()
            || required.sheet != chart_key(&selection.previous)
            || required.orientation != orientation
            || required.transport != origin_transport(self, &point, orientation)
        {
            return Err(ProblemError::Contract(
                "connected path prior source/sheet/orientation/transport mismatch".into(),
            ));
        }
        let alternatives = selection
            .alternatives
            .iter()
            .map(|a| SelectionAlternative {
                id: a.id,
                program: &a.program,
                residual_identity: a.residual_identity,
                unknowns: &a.unknowns,
            })
            .collect::<Vec<_>>();
        let parameters = [point[start.point.len()]];
        let request = SelectionProofRequest {
            selection: selection.previous.selection,
            alternatives: &alternatives,
            winner: selection.winner,
            parameters: &parameters,
            candidate: start.point(),
            order: DerivativeOrder::First,
            time_limit: self
                .scope
                .remaining(self.request.corrector.controls.time_limit)
                .map_err(ProblemError::Provider)?,
            cancel: self.scope.cancellation(),
        };
        selection
            .previous
            .validate(&request, selection.verifier.identity())?;
        Ok(true)
    }
    /// Actual family identity and mechanical correspondence.
    pub fn family(&self) -> &Arc<ArclengthFamily> {
        &self.family
    }
    /// Frozen source preparation consumed by this operation.
    pub fn source(&self) -> ContentHash {
        self.source
    }
}
/// Origin constructed only by an original completion consumer, never a public accepted flag.
#[derive(Clone, Debug)]
pub struct PathStart {
    point: Arc<Vec<f64>>,
    source: SemanticProductKey,
    orientation: Arc<Vec<f64>>,
    owner: Option<Arc<dyn pse_math::AllocationOwner>>,
}
impl PathStart {
    /// Exact original-permitted source key consumed by the path.
    pub fn source(&self) -> SemanticProductKey {
        self.source
    }
    /// Original physical state point, immutable across clones.
    pub fn point(&self) -> &[f64] {
        &self.point
    }
    /// Explicit unit normalized starting orientation.
    pub fn orientation(&self) -> &[f64] {
        &self.orientation
    }
    pub(crate) fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.owner = Some(pse_math::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    pub(crate) fn retained_bytes(&self) -> Result<usize, ProblemError> {
        size_of::<Self>()
            .checked_add(
                self.point
                    .capacity()
                    .checked_add(self.orientation.capacity())
                    .and_then(|n| n.checked_mul(size_of::<f64>()))
                    .ok_or_else(|| ProblemError::memory("path origin extent"))?,
            )
            .ok_or_else(|| ProblemError::memory("path origin extent"))
    }
    pub(crate) fn admitted(
        original: &PreparedSolve,
        point: Vec<f64>,
        orientation: Vec<f64>,
        permission: crate::workflow::numerics::CandidateDecision,
    ) -> Result<Self, ProblemError> {
        if !permission.permits_use() {
            return Err(ProblemError::Contract(
                "path origin requires original completion permission".into(),
            ));
        }
        let source = original.semantic_point_key(&point)?;
        if orientation.len() != point.len() + 1
            || orientation.iter().any(|v| !v.is_finite())
            || (orientation.iter().map(|v| v * v).sum::<f64>() - 1.).abs()
                > 256. * orientation.len() as f64 * f64::EPSILON
        {
            return Err(ProblemError::Contract(
                "path origin requires explicit unit normalized orientation".into(),
            ));
        }
        Ok(Self {
            point: Arc::new(point),
            source,
            orientation: Arc::new(orientation),
            owner: None,
        })
    }
}
/// Actual auxiliary segment, separately retaining numerical and mathematical evidence.
#[derive(Debug)]
pub struct PathObservation {
    /// Physical original states followed by the scalar parameter.
    pub point: Vec<f64>,
    /// Actual normalized oriented displacement.
    pub displacement: f64,
    /// Library sparse tangent observations.
    pub tangent: arclength::Tangent,
    /// Copied native auxiliary correction report; never original permission.
    pub report: Arc<SolveReport>,
    /// Actual selected-sheet transport attempt, when required.
    pub selection: Option<ChartChainEvidence>,
}
/// Copied auxiliary outcome. Original assessment remains the staged driver's responsibility.
#[derive(Debug)]
pub struct PathOutcome {
    reports_owner: Arc<std::sync::Mutex<Vec<Arc<pse_columnar::AllocationLease>>>>,
    endpoint: Option<Arc<PathEndpoint>>,
    /// Complete accepted numerical segments, possibly a bounded partial path.
    pub observations: Vec<PathObservation>,
    /// Actual native correction calls attempted, including calls that returned typed errors.
    pub native_calls: u64,
    /// Every attempted native correction, including failed subdivision trials.
    pub attempts: Vec<Arc<SolveReport>>,
    /// Actual completed source-to-auxiliary-endpoint chart transport, independent of target permission.
    pub connected: Option<pse_model::strategy::ConnectedPath>,
    /// Original-coordinate start only, produced after required transport succeeds.
    pub proposal: Option<Proposal>,
    /// Exact typed stop, independent of partial observations.
    pub terminal: Option<Arc<ProblemError>>,
    /// Tangent work attempted before stopping, including failed tangents.
    pub tangent_work: Vec<arclength::Work>,
    /// Actual numerical event observations, sharing their allocation across consumers.
    pub events: pse_math::SharedAllocation<Vec<arclength::Event>>,
    /// Attempted event probes, including failed probes.
    pub event_work: Vec<arclength::Work>,
    /// Actual original action callbacks outside the native corrector reports.
    pub auxiliary_evaluations: u64,
    /// Same retained completion lease as the owning job.
    _owner: Option<Arc<pse_columnar::AllocationLease>>,
}
#[derive(Debug)]
struct PathEndpoint {
    selection: PathSelection,
    chart: Arc<SelectionChart>,
    parameter: f64,
    target: ContentHash,
    scope: ExecutionScope,
    _owner: Arc<std::sync::Mutex<Vec<Arc<pse_columnar::AllocationLease>>>>,
}
/// Sealed ending-chart scope shared with the common original completion owner.
#[derive(Clone, Debug)]
pub struct PathCompletion(Arc<PathEndpoint>);
impl PathOutcome {
    /// Retain only the genuine ending chart/program scope across original correction.
    pub fn completion_witness(&self) -> Option<PathCompletion> {
        self.endpoint.clone().map(PathCompletion)
    }
    /// Consume the retained ending chart at the actual final original candidate.
    /// # Errors
    /// Missing completed transport, changed source or failed genuine chart validation.
    pub fn validate_original_candidate(
        &self,
        target: &PreparedSolve,
        point: &[f64],
        execution: &Execution,
    ) -> Result<(), ProblemError> {
        self.completion_witness()
            .ok_or_else(|| {
                ProblemError::Unsupported(
                    "path has no completed connected endpoint certificate".into(),
                )
            })?
            .validate_original_candidate(target, point, execution)
    }
}
impl PathCompletion {
    /// Preserve the actual ending sheet when the common original corrector returns.
    /// This consumes the complete source-issued chart/program/verifier scope at the
    /// actual target parameter. Original numerical quality remains its own obligation.
    /// # Errors
    /// A changed target, scope stop or candidate outside the validated ending sheet.
    pub fn validate_original_candidate(
        &self,
        target: &PreparedSolve,
        point: &[f64],
        execution: &Execution,
    ) -> Result<(), ProblemError> {
        execution.check()?;
        let endpoint = &self.0;
        if target.original_identity()? != endpoint.target
            || !Arc::ptr_eq(&execution.cancel, endpoint.scope.cancellation())
            || execution.scope()?.deadline().is_none_or(|d| {
                endpoint
                    .scope
                    .deadline()
                    .is_some_and(|original| d > original)
            })
        {
            return Err(ProblemError::Contract(
                "connected completion target identity mismatch".into(),
            ));
        }
        let selection = &endpoint.selection;
        let alternatives = selection
            .alternatives
            .iter()
            .map(|a| SelectionAlternative {
                id: a.id,
                program: &a.program,
                residual_identity: a.residual_identity,
                unknowns: &a.unknowns,
            })
            .collect::<Vec<_>>();
        let parameters = [endpoint.parameter];
        let scope = execution.scope()?;
        let request = SelectionProofRequest {
            selection: endpoint.chart.selection,
            alternatives: &alternatives,
            winner: selection.winner,
            parameters: &parameters,
            candidate: point,
            order: DerivativeOrder::First,
            time_limit: scope
                .remaining(execution.time_limit)
                .map_err(ProblemError::Provider)?,
            cancel: scope.cancellation(),
        };
        endpoint
            .chart
            .validate(&request, selection.verifier.identity())?;
        execution.check()
    }
}
impl PathOutcome {
    /// Actual operation work. Unavailable native measurements remain unavailable.
    pub fn work(&self) -> pse_model::strategy::WorkObservation {
        let sum = |field: fn(&SolveReport) -> Option<u64>| -> Option<u64> {
            if self.native_calls != self.attempts.len() as u64 {
                return None;
            }
            self.attempts
                .iter()
                .try_fold(0u64, |sum, report| sum.checked_add(field(report)?))
        };
        let factors = self
            .tangent_work
            .iter()
            .chain(self.event_work.iter())
            .try_fold(0u64, |sum, work| {
                sum.checked_add(work.factorizations)?
                    .checked_add(work.rank_probes)
            });
        pse_model::strategy::WorkObservation {
            attempts: self.native_calls,
            evaluations: sum(|r| r.evidence.work.evaluations)
                .and_then(|n| n.checked_add(self.auxiliary_evaluations)),
            iterations: sum(|r| r.evidence.work.iterations),
            factorizations: sum(|r| r.evidence.work.factorizations)
                .and_then(|n| n.checked_add(factors?)),
            proof_steps: sum(|r| r.evidence.work.proof_steps).and_then(|n| {
                self.observations.iter().try_fold(n, |sum, obs| {
                    sum.checked_add(obs.selection.map_or(0, |e| e.work().proof_cells))
                })
            }),
        }
    }
}
impl MathService {
    /// Prepare a scalar root family over actual full First programs. Metadata admission
    /// preserves the original cancellation owner and stamps a finite deadline before waiting.
    /// # Errors
    /// Unsupported source, missing derivatives/proof supplier, changed target or native refusal.
    pub fn prepare_path(
        &self,
        original: PreparedSolve,
        request: PathRequest,
        scope: ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<PreparedPath, MathRuntimeError> {
        driver
            .checkpoint()
            .map_err(|_| MathRuntimeError::Cancelled)?;
        request.policy.validate()?;
        request
            .branch
            .validate()
            .map_err(|e| ProblemError::Contract(e.to_string()))?;
        let task_scope = scope.clone();
        let local = std::time::Instant::now()
            .checked_add(request.corrector.controls.time_limit)
            .ok_or_else(|| ProblemError::Contract("path deadline extent".into()))?;
        let scope = ExecutionScope::new(
            scope.cancellation().clone(),
            Some(scope.deadline().map_or(local, |d| d.min(local))),
        );
        scope.check().map_err(ProblemError::Provider)?;
        let Representation::Algebraic(case) = &original.representation else {
            return Err(ProblemError::Unsupported(
                "path requires a compiled original algebraic source".into(),
            )
            .into());
        };
        if original.profile.intent != SolveIntent::Root {
            return Err(ProblemError::Unsupported(
                "scalar arclength currently admits original root equations".into(),
            )
            .into());
        }
        let program = case
            .sensitivity
            .as_ref()
            .and_then(ParametricPreparation::available)
            .ok_or_else(|| {
                ProblemError::Unsupported(
                    "path requires an attached actual parametric First program".into(),
                )
            })?
            .clone();
        if program.parameters.len() != 1
            || program.parameters[0].0 != request.parameter
            || program.program.assembly.order() < DerivativeOrder::First
            || program.program.assembly.has_directional_actions()
        {
            return Err(ProblemError::Unsupported(
                "path requires one explicit scalar and a full actual First program".into(),
            )
            .into());
        }
        let base = &case.prepared.compiled().plan;
        let n = base.columns().len();
        let assembly = &program.program.assembly;
        if assembly.columns().len() != n + 1
            || assembly.columns()[..n] != *base.columns()
            || assembly.columns()[n] != request.parameter
            || assembly.structure().key() != base.structure().key()
            || base.structure().rows().len() != n
            || base
                .structure()
                .rows()
                .iter()
                .any(|r| !r.lower.is_finite() || r.lower != r.upper)
            || base.structure().objective().is_some()
        {
            return Err(ProblemError::Contract(
                "path scalar/state inventory and finite original equation offsets differ".into(),
            )
            .into());
        }
        let target_parameters = request
            .target
            .root_target_parameters(&original, &program.parameters)?;
        if target_parameters[0].1 < request.interval.0
            || target_parameters[0].1 > request.interval.1
        {
            return Err(ProblemError::Contract(
                "path original target lies outside the declared parameter interval".into(),
            )
            .into());
        }
        let source = original.preparation_identity()?;
        if let Some(curvature) = &request.curvature {
            if request.policy.events.is_none()
                || curvature.source() != source
                || curvature.0.parameter != request.parameter
                || !Arc::ptr_eq(curvature.0.scope.cancellation(), task_scope.cancellation())
                || curvature.0.scope.deadline() != task_scope.deadline()
            {
                return Err(ProblemError::Contract(
                    "path curvature source, parameter, scope or requested event policy mismatch"
                        .into(),
                )
                .into());
            }
            curvature.0.scope.check().map_err(ProblemError::Provider)?;
        }
        if let Some(selection) = &request.selection {
            if selection.source != source
                || selection.parameter != request.parameter
                || selection.coordinates != base.columns()
                || selection.alternatives.is_empty()
                || selection
                    .alternatives
                    .iter()
                    .any(|a| a.program.inputs != n + 1 || a.program.residuals.len() != n)
                || selection.winner >= selection.alternatives.len()
                || selection.alternatives[selection.winner]
                    .unknowns
                    .iter()
                    .map(|u| u.id)
                    .ne(base.columns().iter().copied())
            {
                return Err(ProblemError::Contract(
                    "path selected-sheet supplier source or coordinates mismatch".into(),
                )
                .into());
            }
        } else if request.branch.connected.is_some() {
            return Err(ProblemError::Unsupported(
                "connected path requires actual original selected-sheet verifier payload".into(),
            )
            .into());
        }
        if request.corrector.intent != SolveIntent::Root
            || request.corrector.numerics.key() != original.profile.numerics.key()
            || request.scale.to_bits() != program.normalization.variables[n].to_bits()
            || request.corrector.selection != SolverSelection::Explicit(Backend::Ipopt)
            || request.corrector.controls.hessian != HessianMode::LimitedMemory
            || request.corrector.controls.start != StartPolicy::Explicit
            || request.corrector.sensitivity.is_some()
            || !matches!(request.corrector.presolve, native::presolve::Policy::Off)
        {
            return Err(ProblemError::Unsupported("bounded arclength requires explicit Ipopt limited-memory physical starts without a second analysis/presolve policy".into()).into());
        }
        request.corrector.controls.validate()?;
        admit_profile(
            &request.corrector,
            Route::Native(Backend::Ipopt),
            &original.snapshot,
        )?;
        let meta = (n + 1)
            .checked_mul(1024)
            .and_then(|v| {
                assembly
                    .jacobian_pattern()
                    .row_idx()
                    .len()
                    .checked_add(n + 1)
                    .and_then(|e| e.checked_mul(256))
                    .and_then(|e| v.checked_add(e))
            })
            .and_then(|v| v.checked_add(size_of::<PreparedPath>()))
            .ok_or(MathRuntimeError::Limit("path metadata extent"))?;
        let owner = self.reserve("pse.path.preparation", meta)?;
        let actual = native::assembled::contract(assembly);
        let pattern = assembly.jacobian_pattern();
        let mut incidence = Vec::new();
        let mut parameter_rows = Vec::new();
        for c in 0..=n {
            for r in pattern.row_idx_of_col(c) {
                if c < n {
                    incidence.push(Entry::new(GlobalRow::new(r), GlobalCol::new(c)));
                } else {
                    parameter_rows.push(GlobalRow::new(r));
                }
            }
        }
        let mut zero = FramedHasher::new(pse_ids::Frame::DerivedFamilyV1);
        zero.str("original-numeric-path-row-offsets")
            .hash(&original.original_identity()?)
            .hash(&source);
        for row in base.structure().rows() {
            zero.id(&row.id).f64(row.lower);
        }
        let original_contract = Arc::new(OriginalContract::new(
            zero.finish_hash(),
            original.normalization.key(),
            actual.variables[..n]
                .iter()
                .map(|v| Coordinate {
                    id: v.id,
                    lower: v.lower,
                    upper: v.upper,
                })
                .collect(),
            base.structure()
                .rows()
                .iter()
                .map(|r| Constraint {
                    id: r.id,
                    lower: 0.,
                    upper: 0.,
                })
                .collect(),
            incidence,
            DerivativeSupport {
                order: DerivativeOrder::First,
                jacobian_product: true,
                source,
            },
            OriginalObligations {
                guards: source,
                selection: source,
                objective: None,
            },
        )?);
        let family = Arc::new(ArclengthFamily::new(
            original_contract,
            Parameter {
                id: request.parameter,
                lower: request.interval.0,
                upper: request.interval.1,
                scale: request.scale,
                source,
                action: true,
                rows: parameter_rows,
            },
            request.hyperplane,
            original.normalization.clone(),
        )?);
        let normalization = Normalization {
            variables: family.coordinate_scales(),
            rows: family.row_scales(),
            objective: 1.,
        };
        let mut tolerances = original.tolerances.clone();
        tolerances
            .variables
            .push(request.policy.parameter_tolerance);
        tolerances.rows.push(request.policy.hyperplane_tolerance);
        let accuracy =
            ResolvedAccuracy::resolve(&original.numerics.policy, &tolerances, &normalization)?;
        let contract = native::OracleContract {
            identity: family.key(),
            variables: family
                .coordinates()
                .into_iter()
                .map(|v| native::Variable {
                    id: v.id,
                    lower: v.lower,
                    upper: v.upper,
                })
                .collect(),
            rows: family.constraints().iter().map(|r| r.id).collect(),
            derivatives: DerivativeOrder::First,
            smoothness: actual.smoothness,
        };
        let matrix = pse_math::sparse::AssemblyMatrix::new(
            n + 1,
            n + 1,
            family.incidence(),
            i32::MAX as usize,
        )?;
        let mut structure = native::structural::oracle_structure_with_cancel(
            &contract,
            matrix.matrix().symbolic(),
            &vec![(0., 0.); n + 1],
            false,
            scope.cancellation(),
        )?;
        structure.witness = structure.witness.with_owner(owner.clone());
        let facts = routing::oracle_facts(&contract, false, true);
        routing::Requirements {
            table: &execution::LINKED,
            facts: &facts,
            intent: SolveIntent::Root,
            numerical_psd: false,
            least_squares: false,
            controls: &request.corrector.controls,
            settings: &request.corrector.backend,
            sensitivity: false,
            context: routing::Context {
                snapshot: original.snapshot.clone(),
                pending_classes: &[],
                structure: Some(structure.clone()),
                oracle: Some(&contract),
                guards: &BTreeMap::new(),
                budgets: Some(execution::Budgets {
                    accuracy: &accuracy,
                    tolerances: &tolerances,
                    normalization: &normalization,
                }),
                coefficients: None,
                cone: None,
                factorable: None,
                certificate: None,
                prepared: &[routing::ArtifactDemand::Representation(
                    execution::Representation::Nlp,
                )],
                refusals: &BTreeMap::new(),
            },
        }
        .select(request.corrector.selection)?;
        let proof_bytes = request
            .selection
            .as_ref()
            .map(|s| {
                s.verifier.workspace_bytes(
                    &s.alternatives
                        .iter()
                        .map(|a| a.program.clone())
                        .collect::<Vec<_>>(),
                )
            })
            .transpose()?
            .unwrap_or(0);
        let event_bytes = request.policy.events.as_ref().map_or(0, |e| e.probe.bytes);
        let curvature_bytes = request
            .curvature
            .as_ref()
            .map(PathCurvature::worker_bytes)
            .transpose()?
            .unwrap_or(0);
        let workspace_bytes = program
            .program
            .assembly
            .numeric_worker_bytes()
            .checked_mul(2)
            .and_then(|v| v.checked_add(request.policy.tangent.bytes))
            .and_then(|v| v.checked_add(proof_bytes))
            .and_then(|v| v.checked_add(event_bytes))
            .and_then(|v| v.checked_add(curvature_bytes))
            .ok_or(MathRuntimeError::Limit("path worker extent"))?;
        if workspace_bytes > self.policy.workspace_bytes {
            return Err(MathRuntimeError::Limit("path finite workspace allowance"));
        }
        scope.check().map_err(ProblemError::Provider)?;
        let target_identity = request.target.original_identity()?;
        let native_profile = profile_key(&request.corrector)?.as_id();
        Ok(PreparedPath {
            branch: request.branch,
            data: Arc::new(PathData {
                target_identity,
                native_profile,
                case: case.clone(),
                program,
                original,
                request,
                family,
                source,
                normalization,
                tolerances,
                accuracy,
                structure,
                scope,
                task_scope,
                _owner: owner,
                workspace_bytes,
            }),
        })
    }
}

#[derive(Debug)]
struct Supplier {
    calls: Option<Arc<std::sync::atomic::AtomicU64>>,
    oracle: native::assembled::AlgebraicOracle,
    original: Arc<OriginalContract>,
    parameter: Parameter,
    offsets: Vec<f64>,
    _case: Arc<ExecutableCase>,
    _charge: super::super::WorkerCharge,
}
impl Supplier {
    fn counted(&self) -> Result<(), ProblemError> {
        if let Some(calls) = &self.calls {
            calls
                .try_update(Ordering::AcqRel, Ordering::Acquire, |v| v.checked_add(1))
                .map_err(|_| ProblemError::memory("path source evaluation counter"))?;
        }
        Ok(())
    }
    fn action(
        &mut self,
        point: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let pattern = NlpOracle::jacobian_pattern(&self.oracle)
            .to_owned()
            .map_err(|e| ProblemError::memory(format!("path action pattern: {e:?}")))?;
        if point.len() != pattern.ncols()
            || direction.len() != pattern.ncols()
            || out.len() != pattern.nrows()
            || direction.iter().any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "path physical First action shape".into(),
            ));
        }
        let mut values = vec![0.; pattern.row_idx().len()];
        self.counted()?;
        NlpOracle::jacobian(&mut self.oracle, point, &mut values)?;
        let mut result = vec![0.; out.len()];
        for (column, weight) in direction.iter().enumerate() {
            for index in pattern.col_range(column) {
                result[pattern.row_idx()[index]] += values[index] * weight;
            }
        }
        if result.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "path original First action nonfinite",
            ));
        }
        out.copy_from_slice(&result);
        Ok(())
    }
}
impl ParameterizedOracle for Supplier {
    type Error = ProblemError;
    fn contract(&self) -> &OriginalContract {
        &self.original
    }
    fn parameter(&self) -> &Parameter {
        &self.parameter
    }
    fn values(&mut self, x: &[f64], p: f64, out: &mut [f64]) -> Result<(), ProblemError> {
        let point = augmented(x, p);
        let mut result = vec![0.; self.offsets.len()];
        self.counted()?;
        self.oracle.constraints(&point, &mut result)?;
        for (v, b) in result.iter_mut().zip(&self.offsets) {
            *v -= b;
        }
        if out.len() != result.len() {
            return Err(ProblemError::Contract("path original row shape".into()));
        }
        out.copy_from_slice(&result);
        Ok(())
    }
    fn state_action(
        &mut self,
        x: &[f64],
        p: f64,
        v: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let mut direction = v.to_vec();
        direction.push(0.);
        self.action(&augmented(x, p), &direction, out)
    }
    fn parameter_action(
        &mut self,
        x: &[f64],
        p: f64,
        v: f64,
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let mut direction = vec![0.; x.len() + 1];
        direction[x.len()] = v;
        self.action(&augmented(x, p), &direction, out)
    }
}
fn augmented(x: &[f64], p: f64) -> Vec<f64> {
    x.iter().copied().chain([p]).collect()
}
impl PreparedPath {
    fn supplier(
        &self,
        service: &MathService,
        execution: &Execution,
        budget: &Arc<WorkerBudget>,
    ) -> Result<Supplier, MathRuntimeError> {
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = service.worker(
            self.program.program.clone(),
            &self.case.providers,
            execution.scope()?,
            budget,
        )?;
        let oracle = native::assembled::AlgebraicOracle::new(worker, self.case.values.clone())?;
        Ok(Supplier {
            calls: None,
            oracle,
            original: Arc::new(self.family.original().clone()),
            parameter: self.family.parameter().clone(),
            offsets: self
                .case
                .prepared
                .compiled()
                .plan
                .structure()
                .rows()
                .iter()
                .map(|r| r.lower)
                .collect(),
            _case,
            _charge,
        })
    }
}
/// Normalization declaration for the existing common NLP transport owner.
#[derive(Debug)]
struct NormalizedCorrector {
    inner: arclength::Corrector<Supplier>,
    normalization: Normalization,
}
impl NlpOracle for NormalizedCorrector {
    fn contract(&self) -> &native::OracleContract {
        NlpOracle::contract(&self.inner)
    }
    fn normalization(&self) -> Option<&Normalization> {
        Some(&self.normalization)
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        NlpOracle::jacobian_pattern(&self.inner)
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        None
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        self.inner.constraint_bounds()
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.inner.objective(x)
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.gradient(x, out)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.constraints(x, out)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        NlpOracle::jacobian(&mut self.inner, x, out)
    }
    fn hessian(
        &mut self,
        x: &[f64],
        w: f64,
        l: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.inner.hessian(x, w, l, out)
    }
}
fn chart_key(chart: &SelectionChart) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.str("actual-selected-path-chart")
        .id(&chart.selection)
        .hash(&chart.verifier_identity)
        .u64(chart.winner as u64)
        .u64(chart.order as u64);
    for a in &chart.alternatives {
        h.id(&a.id).hash(&a.residual_identity);
        for u in &a.unknowns {
            h.id(&u.id).f64(u.lower).f64(u.upper);
        }
    }
    for intervals in [&chart.parameters, &chart.existence, &chart.uniqueness] {
        h.u64(intervals.len() as u64);
        for interval in intervals {
            h.f64(interval.lower).f64(interval.upper);
        }
    }
    h.finish_hash()
}
fn orientation_key(source: ContentHash, point: &[f64], orientation: &[f64]) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.str("actual-path-origin-orientation")
        .hash(&source)
        .hash(&native::square_response::point_key(point));
    for v in orientation {
        h.f64(*v);
    }
    h.finish_hash()
}
fn origin_transport(path: &PreparedPath, point: &[f64], orientation: ContentHash) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.str("source-issued-root-path-origin")
        .hash(&path.family.key())
        .hash(&path.source)
        .hash(&native::square_response::point_key(point))
        .hash(&orientation);
    if let Some(selection) = &path.request.selection {
        h.hash(&selection.verifier.identity())
            .hash(&chart_key(&selection.previous))
            .u64(match selection.coverage {
                ChartChainCoverage::RootSheet => 0,
                ChartChainCoverage::SelectedFunction => 1,
            });
    }
    h.finish_hash()
}
struct TransportEdge<'a> {
    previous: &'a SelectionChart,
    next: &'a SelectionChart,
    origin: &'a [f64],
    endpoint: &'a [f64],
    orientation: ContentHash,
}
fn completed_transport(
    prior: ContentHash,
    path: &PreparedPath,
    selection: &PathSelection,
    edge: TransportEdge<'_>,
    evidence: ChartChainEvidence,
) -> ContentHash {
    let TransportEdge {
        previous,
        next,
        origin,
        endpoint,
        orientation,
    } = edge;
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.str("actual-completed-chart-chain")
        .hash(&prior)
        .hash(&path.source)
        .hash(&selection.verifier.identity())
        .hash(&chart_key(previous))
        .hash(&chart_key(next))
        .hash(&native::square_response::point_key(origin))
        .hash(&native::square_response::point_key(endpoint))
        .hash(&orientation);
    if let ChartChainEvidence::Connected(proof) = evidence {
        h.u64(match proof.coverage {
            ChartChainCoverage::RootSheet => 0,
            ChartChainCoverage::SelectedFunction => 1,
        })
        .u64(proof.charts)
        .u64(proof.connections)
        .u64(proof.proof_cells);
    }
    h.finish_hash()
}
impl std::ops::Deref for PathSelection {
    type Target = SelectionData;
    fn deref(&self) -> &SelectionData {
        &self.0
    }
}
impl PathSelection {
    fn connect(
        &self,
        previous: &SelectionChart,
        origin: f64,
        next: &[f64],
        execution: &Execution,
    ) -> Result<(Option<SelectionChart>, ChartChainEvidence), ProblemError> {
        execution.check()?;
        let n = next.len() - 1;
        let parameters = [next[n]];
        let origin = [origin];
        let alternatives = self
            .alternatives
            .iter()
            .map(|a| SelectionAlternative {
                id: a.id,
                program: &a.program,
                residual_identity: a.residual_identity,
                unknowns: &a.unknowns,
            })
            .collect::<Vec<_>>();
        let scope = execution.scope()?;
        let request = SelectionProofRequest {
            selection: previous.selection,
            alternatives: &alternatives,
            winner: self.winner,
            parameters: &parameters,
            candidate: &next[..n],
            order: DerivativeOrder::First,
            time_limit: scope
                .remaining(execution.time_limit)
                .map_err(ProblemError::Provider)?,
            cancel: scope.cancellation(),
        };
        let next_chart = match self.verifier.certify(&request)? {
            SelectionEvidence::Unique(chart) => chart,
            SelectionEvidence::Multiple => {
                return Ok((
                    None,
                    ChartChainEvidence::Incomplete(
                        pse_math::implicit::SelectionProofRefusal::Chart,
                    ),
                ));
            }
            SelectionEvidence::Incomplete(reason) => {
                return Ok((None, ChartChainEvidence::Incomplete(reason)));
            }
        };
        next_chart.validate(&request, self.verifier.identity())?;
        let chain = ChartChainRequest {
            endpoint: &request,
            previous,
            next: &next_chart,
            origin: &origin,
            coverage: self.coverage,
        };
        chain.validate(self.verifier.identity())?;
        let evidence = self.verifier.connect_chain(&chain)?;
        execution.check()?;
        if matches!(evidence,ChartChainEvidence::Connected(proof) if proof.coverage==self.coverage)
        {
            Ok((Some(next_chart), evidence))
        } else {
            Ok((None, evidence))
        }
    }
}
fn retry_cause(cause: &ProblemError) -> bool {
    crate::math::strategy::failure(cause)
        == pse_model::generated::enums::NumericalAttemptObservation::NumericalFailure
}
fn report_failure(report: &SolveReport) -> Option<(Arc<ProblemError>, bool)> {
    if let Some(cause) = report.shared_validation_failure() {
        return Some((cause, false));
    }
    if report.evidence.callback.terminal_failure {
        return Some((
            report.shared_callback_failure().unwrap_or_else(|| {
                Arc::new(ProblemError::internal(
                    "terminal path callback without retained cause",
                ))
            }),
            false,
        ));
    }
    match report.termination.category {
        Termination::Success | Termination::Acceptable
            if report
                .quality
                .as_ref()
                .is_some_and(|q| q.normalized_max <= 1.)
                && report.candidate.is_some() =>
        {
            None
        }
        Termination::Numerical => Some((
            Arc::new(ProblemError::numerical(
                "native path corrector numerical stop",
            )),
            true,
        )),
        Termination::Evaluation if native::callback::retryable_evaluation(report) => Some((
            report.shared_callback_failure().unwrap_or_else(|| {
                Arc::new(ProblemError::numerical(
                    "native path recoverable trials exhausted",
                ))
            }),
            true,
        )),
        category => Some((
            Arc::new(ProblemError::native(
                native::NativeStatus {
                    backend: report.backend,
                    code: report.termination.code,
                    name: report.termination.name.clone(),
                },
                category,
                "auxiliary path corrector did not supply its required physical equations",
            )),
            false,
        )),
    }
}
struct PathExecution<'a> {
    service: &'a MathService,
    path: &'a PreparedPath,
    execution: &'a Execution,
    budget: &'a Arc<WorkerBudget>,
    retained: &'a mut Retained,
}
fn correct(
    context: PathExecution<'_>,
    point: &[f64],
    orientation: &[f64],
    source: ContentHash,
) -> Result<SolveReport, MathRuntimeError> {
    let PathExecution {
        service,
        path,
        execution,
        budget,
        retained,
    } = context;
    let supplier = path.supplier(service, execution, budget)?;
    let binding = path
        .family
        .bind(supplier, point.to_vec(), orientation.to_vec(), source)?;
    let identity = binding.key();
    let smoothness = native::assembled::contract(&path.program.program.assembly).smoothness;
    let inner = arclength::Corrector::new(binding, smoothness)?;
    let oracle = NormalizedCorrector {
        inner,
        normalization: path.normalization.clone(),
    };
    let compatibility = Compatibility {
        layout: path.family.key(),
        profile: profile_key(&path.request.corrector)?.as_id(),
        data: identity,
        backend: Backend::Ipopt,
    };
    let structure = native::structural::Assessment::new(
        native::structural::Mode::Roots,
        path.structure.variables.clone(),
        path.structure.equations.clone(),
        path.structure.witness.clone(),
    )?;
    let adapter = execution::adapter(Backend::Ipopt);
    let warm = WarmStart {
        origin: None,
        compatibility: compatibility.clone(),
        payload: adapter.primal_start(point.to_vec())?,
    };
    let report = execution::nlp(
        execution::Step {
            adapter,
            settings: &path.request.corrector.backend,
            snapshot: &path.original.snapshot,
            structure: Some(&structure),
            controls: &path.request.corrector.controls,
            accuracy: &path.accuracy,
            execution: execution.clone(),
            tolerances: &path.tolerances,
            normalization: &path.normalization,
            compatibility,
            warm: Some(&warm),
        },
        retained,
        execution::Nlp {
            oracle: Box::new(oracle),
            initial: point,
            presolve: &native::presolve::Policy::Off,
            intent: SolveIntent::Root,
            sense: ObjectiveSense::Minimize,
            limit: service.policy.worker_bytes / 256,
            analysis: execution::Analysis::NONE,
        },
    )?;
    Ok(report)
}
/// Mechanical fixed-scalar original equations used solely to bridge a completed
/// arclength endpoint to the actual target. Existing native iteration owns correction.
#[derive(Debug)]
struct FixedCorrector {
    supplier: Supplier,
    parameter: f64,
    contract: native::OracleContract,
    pattern: pse_math::sparse::AssemblyMatrix,
    bounds: Vec<(f64, f64)>,
    normalization: Normalization,
}
impl NlpOracle for FixedCorrector {
    fn contract(&self) -> &native::OracleContract {
        &self.contract
    }
    fn normalization(&self) -> Option<&Normalization> {
        Some(&self.normalization)
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        None
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, _: &[f64]) -> Result<f64, ProblemError> {
        Ok(0.)
    }
    fn gradient(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if out.len() != self.contract.variables.len() {
            return Err(ProblemError::Contract(
                "fixed-target gradient extent".into(),
            ));
        }
        out.fill(0.);
        Ok(())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.supplier.values(x, self.parameter, out)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let pattern = self.pattern.matrix().symbolic();
        let n = self.contract.variables.len();
        if x.len() != n || out.len() != pattern.row_idx().len() {
            return Err(ProblemError::Contract("fixed-target First shape".into()));
        }
        let mut direction = vec![0.; n];
        let mut action = vec![0.; n];
        let mut result = vec![0.; out.len()];
        for column in 0..n {
            direction[column] = 1.;
            self.supplier
                .state_action(x, self.parameter, &direction, &mut action)?;
            direction[column] = 0.;
            for index in pattern.col_range(column) {
                result[index] = action[pattern.row_idx()[index]];
            }
        }
        out.copy_from_slice(&result);
        Ok(())
    }
    fn hessian(&mut self, _: &[f64], _: f64, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
        Err(ProblemError::Unsupported(
            "fixed-target path uses actual First and library limited-memory curvature".into(),
        ))
    }
}
fn correct_target(
    service: &MathService,
    path: &PreparedPath,
    point: &[f64],
    parameter: f64,
    execution: &Execution,
    budget: &Arc<WorkerBudget>,
    retained: &mut Retained,
) -> Result<SolveReport, MathRuntimeError> {
    let supplier = path.supplier(service, execution, budget)?;
    let original = path.family.original();
    let n = original.coordinates().len();
    let pattern =
        pse_math::sparse::AssemblyMatrix::new(n, n, original.incidence(), i32::MAX as usize)?;
    let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
    h.str("actual-fixed-target-path-bridge")
        .hash(&path.family.key())
        .hash(&path.target_identity)
        .f64(parameter);
    let identity = h.finish_hash();
    let contract = native::OracleContract {
        identity,
        variables: original
            .coordinates()
            .iter()
            .map(|v| native::Variable {
                id: v.id,
                lower: v.lower,
                upper: v.upper,
            })
            .collect(),
        rows: original.constraints().iter().map(|r| r.id).collect(),
        derivatives: DerivativeOrder::First,
        smoothness: native::assembled::contract(&path.program.program.assembly).smoothness,
    };
    let oracle = FixedCorrector {
        supplier,
        parameter,
        contract: contract.clone(),
        pattern,
        bounds: vec![(0., 0.); n],
        normalization: path.original.normalization.clone(),
    };
    let mut structure = native::structural::oracle_structure_with_cancel(
        &contract,
        oracle.jacobian_pattern(),
        &oracle.bounds,
        false,
        &execution.cancel,
    )?;
    structure.witness = structure.witness.with_owner(path._owner.clone());
    let facts = routing::oracle_facts(&contract, false, true);
    routing::Requirements {
        table: &execution::LINKED,
        facts: &facts,
        intent: SolveIntent::Root,
        numerical_psd: false,
        least_squares: false,
        controls: &path.request.corrector.controls,
        settings: &path.request.corrector.backend,
        sensitivity: false,
        context: routing::Context {
            snapshot: path.original.snapshot.clone(),
            pending_classes: &[],
            structure: Some(structure.clone()),
            oracle: Some(&contract),
            guards: &BTreeMap::new(),
            budgets: Some(execution::Budgets {
                accuracy: &path.original.accuracy,
                tolerances: &path.original.tolerances,
                normalization: &path.original.normalization,
            }),
            coefficients: None,
            cone: None,
            factorable: None,
            certificate: None,
            prepared: &[routing::ArtifactDemand::Representation(
                execution::Representation::Nlp,
            )],
            refusals: &BTreeMap::new(),
        },
    }
    .select(SolverSelection::Explicit(Backend::Ipopt))?;
    let assessment = native::structural::Assessment::new(
        native::structural::Mode::Roots,
        structure.variables,
        structure.equations,
        structure.witness,
    )?;
    let compatibility = Compatibility {
        layout: original.identity(),
        profile: profile_key(&path.request.corrector)?.as_id(),
        data: identity,
        backend: Backend::Ipopt,
    };
    let adapter = execution::adapter(Backend::Ipopt);
    let warm = WarmStart {
        origin: None,
        compatibility: compatibility.clone(),
        payload: adapter.primal_start(point.to_vec())?,
    };
    Ok(execution::nlp(
        execution::Step {
            adapter,
            settings: &path.request.corrector.backend,
            snapshot: &path.original.snapshot,
            structure: Some(&assessment),
            controls: &path.request.corrector.controls,
            accuracy: &path.original.accuracy,
            execution: execution.clone(),
            tolerances: &path.original.tolerances,
            normalization: &path.original.normalization,
            compatibility,
            warm: Some(&warm),
        },
        retained,
        execution::Nlp {
            oracle: Box::new(oracle),
            initial: point,
            presolve: &native::presolve::Policy::Off,
            intent: SolveIntent::Root,
            sense: ObjectiveSense::Minimize,
            limit: service.policy.worker_bytes / 256,
            analysis: execution::Analysis::NONE,
        },
    )?)
}
#[derive(Debug)]
struct CurvatureSupplier {
    inner: Supplier,
    source: ContentHash,
}
impl arclength::StateCurvature for CurvatureSupplier {
    fn contract(&self) -> &OriginalContract {
        &self.inner.original
    }
    fn parameter(&self) -> &Parameter {
        &self.inner.parameter
    }
    fn source(&self) -> ContentHash {
        self.source
    }
    fn action(
        &mut self,
        state: &[f64],
        parameter: f64,
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        let n = state.len();
        if direction.len() != n || out.len() != n {
            return Err(ProblemError::Contract(
                "original path curvature physical shape".into(),
            ));
        }
        let point = augmented(state, parameter);
        let pattern = self
            .inner
            .oracle
            .hessian_pattern()
            .ok_or_else(|| {
                ProblemError::Unsupported("actual path source has no Second Hessian support".into())
            })?
            .to_owned()
            .map_err(|e| ProblemError::memory(format!("path curvature pattern: {e:?}")))?;
        let mut values = vec![0.; pattern.row_idx().len()];
        let mut weights = vec![0.; n];
        let mut result = vec![0.; n];
        for row in 0..n {
            weights[row] = 1.;
            self.inner.counted()?;
            self.inner
                .oracle
                .hessian(&point, 0., &weights, &mut values)?;
            weights[row] = 0.;
            for column in 0..n {
                for index in pattern.col_range(column) {
                    let r = pattern.row_idx()[index];
                    if r < n {
                        result[row] += values[index]
                            * direction[r]
                            * direction[column]
                            * if r == column { 1. } else { 2. };
                    }
                }
            }
        }
        if result.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "original Second curvature action nonfinite",
            ));
        }
        out.copy_from_slice(&result);
        Ok(())
    }
}
impl PreparedPath {
    fn curvature_supplier(
        &self,
        service: &MathService,
        execution: &Execution,
        budget: &Arc<WorkerBudget>,
        calls: Arc<std::sync::atomic::AtomicU64>,
    ) -> Result<Option<CurvatureSupplier>, MathRuntimeError> {
        let Some(curvature) = &self.request.curvature else {
            return Ok(None);
        };
        curvature.0.scope.check().map_err(ProblemError::Provider)?;
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = service.worker(
            curvature.0.program.clone(),
            &self.case.providers,
            execution.scope()?,
            budget,
        )?;
        let oracle = native::assembled::AlgebraicOracle::new(worker, self.case.values.clone())?;
        Ok(Some(CurvatureSupplier {
            source: curvature.key(),
            inner: Supplier {
                calls: Some(calls),
                oracle,
                original: Arc::new(self.family.original().clone()),
                parameter: self.family.parameter().clone(),
                offsets: self
                    .case
                    .prepared
                    .compiled()
                    .plan
                    .structure()
                    .rows()
                    .iter()
                    .map(|r| r.lower)
                    .collect(),
                _case,
                _charge,
            },
        }))
    }
}
fn event_request<'a>(
    path: &'a PreparedPath,
    point: &'a [f64],
    orientation: &'a [f64],
) -> arclength::Request<'a> {
    arclength::Request {
        family: &path.family,
        point,
        previous: orientation,
        source: native::square_response::point_key(point),
        residual_limits: &path.original.tolerances.rows,
        backward_limit: path.request.policy.backward_limit,
        limits: path.request.policy.tangent,
    }
}
struct EventPoint<'a> {
    point: &'a [f64],
    orientation: &'a [f64],
    localization: Option<arclength::Localization>,
}
fn event_probe(
    path: &PreparedPath,
    input: EventPoint<'_>,
    supplier: &mut Supplier,
    curvature: &mut Option<CurvatureSupplier>,
    execution: &Execution,
    outcome: &mut PathOutcome,
    events: &mut Vec<arclength::Event>,
) -> Result<(), MathRuntimeError> {
    let EventPoint {
        point,
        orientation,
        localization,
    } = input;
    let policy = path
        .request
        .policy
        .events
        .as_ref()
        .ok_or_else(|| ProblemError::internal("path event policy missing"))?;
    if events.len() >= policy.observations {
        return Err(ProblemError::Limit {
            kind: native::LimitKind::Work,
            detail: "path event observation allowance".into(),
        }
        .into());
    }
    let curvature: Option<&mut dyn arclength::StateCurvature> = curvature
        .as_mut()
        .map(|v| -> &mut dyn arclength::StateCurvature { v });
    match arclength::probe(
        event_request(path, point, orientation),
        supplier,
        curvature,
        arclength::ProbePolicy {
            localization,
            rank_threshold: policy.rank_threshold,
            nondegeneracy_threshold: policy.nondegeneracy_threshold,
            limits: policy.probe,
        },
        execution,
    ) {
        Ok(event) => {
            outcome.event_work.push(event.work);
            events.push(event);
            Ok(())
        }
        Err(failure) => {
            outcome.event_work.push(failure.work);
            outcome.terminal = Some(failure.cause);
            Ok(())
        }
    }
}
/// Localize a genuine signed tangent-parameter bracket with the existing native
/// corrector. Its hyperplanes share the source tangent's declared normalized units.
struct EventBracket<'a> {
    left: &'a [f64],
    right: &'a [f64],
    tangent: &'a arclength::Tangent,
    displacement: f64,
}
fn localize_event(
    context: PathExecution<'_>,
    bracket: EventBracket<'_>,
    supplier: &mut Supplier,
    curvature: &mut Option<CurvatureSupplier>,
    outcome: &mut PathOutcome,
    events: &mut Vec<arclength::Event>,
) -> Result<(), MathRuntimeError> {
    let PathExecution {
        service,
        path,
        execution,
        budget,
        retained,
    } = context;
    let EventBracket {
        left,
        right,
        tangent,
        displacement,
    } = bracket;
    let policy = path
        .request
        .policy
        .events
        .as_ref()
        .ok_or_else(|| ProblemError::internal("path event policy missing"))?;
    let n = left.len() - 1;
    let right_tangent = match arclength::tangent(
        event_request(path, right, &tangent.normalized),
        supplier,
        execution,
    ) {
        Ok(value) => {
            outcome.tangent_work.push(value.work);
            value
        }
        Err(failure) => {
            outcome.tangent_work.push(failure.work);
            if retry_cause(&failure.cause) {
                event_probe(
                    path,
                    EventPoint {
                        point: right,
                        orientation: &tangent.normalized,
                        localization: None,
                    },
                    supplier,
                    curvature,
                    execution,
                    outcome,
                    events,
                )?;
                return Ok(());
            }
            outcome.terminal = Some(failure.cause);
            return Ok(());
        }
    };
    let left_sign = tangent.normalized[n];
    let right_sign = right_tangent.normalized[n];
    if left_sign * right_sign > 0. {
        if policy.endpoints {
            event_probe(
                path,
                EventPoint {
                    point: right,
                    orientation: &right_tangent.normalized,
                    localization: None,
                },
                supplier,
                curvature,
                execution,
                outcome,
                events,
            )?;
        }
        return Ok(());
    }
    let mut a = 0.;
    let mut b = displacement;
    let mut a_point = left.to_vec();
    let mut b_point = right.to_vec();
    let mut a_sign = left_sign;
    let mut a_tangent = tangent.clone();
    let mut b_tangent = right_tangent;
    for _ in 0..policy.localization_steps {
        execution.check()?;
        if b - a <= policy.localization_tolerance {
            break;
        }
        let middle = a + (b - a) * 0.5;
        let predicted = left
            .iter()
            .zip(&tangent.physical)
            .map(|(p, t)| p + middle * t)
            .collect::<Vec<_>>();
        path.family.validate_point(&predicted)?;
        outcome.native_calls += 1;
        let report = Arc::new(
            correct(
                PathExecution {
                    service,
                    path,
                    execution,
                    budget,
                    retained,
                },
                &predicted,
                &tangent.normalized,
                native::square_response::point_key(left),
            )?
            .with_owner(outcome.reports_owner.clone()),
        );
        outcome.attempts.push(report.clone());
        if let Some((cause, _)) = report_failure(&report) {
            outcome.terminal = Some(cause);
            return Ok(());
        }
        let point = report
            .candidate
            .as_ref()
            .ok_or_else(|| ProblemError::internal("localized path candidate missing"))?
            .primal
            .clone();
        path.family.validate_point(&point)?;
        let next = match arclength::tangent(
            event_request(path, &point, &tangent.normalized),
            supplier,
            execution,
        ) {
            Ok(value) => {
                outcome.tangent_work.push(value.work);
                value
            }
            Err(failure) => {
                outcome.tangent_work.push(failure.work);
                if retry_cause(&failure.cause) {
                    event_probe(
                        path,
                        EventPoint {
                            point: &point,
                            orientation: &tangent.normalized,
                            localization: None,
                        },
                        supplier,
                        curvature,
                        execution,
                        outcome,
                        events,
                    )?;
                    return Ok(());
                }
                outcome.terminal = Some(failure.cause);
                return Ok(());
            }
        };
        if a_sign * next.normalized[n] <= 0. {
            b = middle;
            b_point = point;
            b_tangent = next;
        } else {
            a = middle;
            a_point = point;
            a_sign = next.normalized[n];
            a_tangent = next;
        }
    }
    let (best, best_at, best_tangent) =
        if a_tangent.normalized[n].abs() < b_tangent.normalized[n].abs() {
            (&a_point, a, &a_tangent)
        } else {
            (&b_point, b, &b_tangent)
        };
    let localized = (b - a <= policy.localization_tolerance).then_some(arclength::Localization {
        left: native::square_response::point_key(&a_point),
        right: native::square_response::point_key(&b_point),
        interval: (a, b),
        at: best_at,
    });
    event_probe(
        path,
        EventPoint {
            point: best,
            orientation: &best_tangent.normalized,
            localization: localized,
        },
        supplier,
        curvature,
        execution,
        outcome,
        events,
    )
}

/// Execute a bounded oriented path inside the original owning worker job. This seam does
/// not create another task clock/session driver or grant original candidate permission.
pub(crate) fn run_path(
    service: &MathService,
    path: &PreparedPath,
    start: PathStart,
    mut execution: Execution,
    budget: &Arc<WorkerBudget>,
    owner: Arc<pse_columnar::AllocationLease>,
) -> PathOutcome {
    let mut outcome = PathOutcome {
        reports_owner: Arc::new(std::sync::Mutex::new(vec![owner.clone()])),
        endpoint: None,
        observations: Vec::new(),
        native_calls: 0,
        attempts: Vec::new(),
        connected: None,
        proposal: None,
        terminal: None,
        tangent_work: Vec::new(),
        events: pse_math::SharedAllocation::from(Arc::new(Vec::new())),
        event_work: Vec::new(),
        auxiliary_evaluations: 0,
        _owner: Some(owner.clone()),
    };
    let calls = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let mut events = Vec::new();
    let result = (|| -> Result<(), MathRuntimeError> {
        execution.check()?;
        if path.original.semantic_point_key(&start.point)? != start.source
            || !Arc::ptr_eq(&execution.cancel, path.scope.cancellation())
            || execution.scope()?.deadline().is_none_or(|d| {
                path.task_scope
                    .deadline()
                    .is_some_and(|original| d > original)
            })
        {
            return Err(ProblemError::Contract(
                "path originating point/source or absolute execution scope mismatch".into(),
            )
            .into());
        }
        let enclosing = execution.scope()?;
        let deadline = enclosing
            .deadline()
            .into_iter()
            .chain(path.scope.deadline())
            .min();
        execution.enclosing_scope = Some(ExecutionScope::new(execution.cancel.clone(), deadline));
        path.origin_connected(&start)?;
        let n = start.point.len();
        let mut point = augmented(&start.point, path.program.parameters[0].1);
        path.family.validate_point(&point)?;
        let orientation_id = orientation_key(path.source, &point, &start.orientation);
        let proof_bytes = path
            .request
            .selection
            .as_ref()
            .map(|s| {
                s.verifier.workspace_bytes(
                    &s.alternatives
                        .iter()
                        .map(|a| a.program.clone())
                        .collect::<Vec<_>>(),
                )
            })
            .transpose()?
            .unwrap_or(0);
        let _proof_work = budget.charge(proof_bytes)?;
        let _tangent_work = budget.charge(path.request.policy.tangent.bytes)?;
        let _event_work = path
            .request
            .policy
            .events
            .as_ref()
            .map(|e| budget.charge(e.probe.bytes))
            .transpose()?;
        let _curvature_scratch = path
            .request
            .curvature
            .as_ref()
            .map(|c| {
                budget.charge(
                    c.worker_bytes()?
                        .checked_sub(c.0.program.assembly.numeric_worker_bytes())
                        .ok_or(MathRuntimeError::Limit("path curvature scratch extent"))?,
                )
            })
            .transpose()?;
        let mut orientation = start.orientation.as_ref().clone();
        let mut supplier = path.supplier(service, &execution, budget)?;
        supplier.calls = Some(calls.clone());
        let mut curvature = path.curvature_supplier(service, &execution, budget, calls.clone())?;
        let mut retained = Retained::default();
        let mut steps = crate::math::strategy::path_control::StepControl::new(
            path.request.policy.step,
            path.request.policy.minimum_step,
            path.request.policy.step,
            1.,
            path.request.policy.subdivisions,
        )?;
        let mut chart = path.request.selection.as_ref().map(|s| s.previous.clone());
        let mut transport = origin_transport(path, &point, orientation_id);
        let mut source = orientation_id;
        if let Some(selection) = &path.request.selection {
            let alternatives = selection
                .alternatives
                .iter()
                .map(|a| SelectionAlternative {
                    id: a.id,
                    program: &a.program,
                    residual_identity: a.residual_identity,
                    unknowns: &a.unknowns,
                })
                .collect::<Vec<_>>();
            let parameters = [point[n]];
            let scope = execution.scope()?;
            let request = SelectionProofRequest {
                selection: selection.previous.selection,
                alternatives: &alternatives,
                winner: selection.winner,
                parameters: &parameters,
                candidate: &point[..n],
                order: DerivativeOrder::First,
                time_limit: scope
                    .remaining(execution.time_limit)
                    .map_err(ProblemError::Provider)?,
                cancel: scope.cancellation(),
            };
            selection
                .previous
                .validate(&request, selection.verifier.identity())?;
            if let Some(required) = path.branch.connected
                && (required.path != path.family.key()
                    || required.sheet != chart_key(&selection.previous)
                    || required.orientation != orientation_id
                    || required.transport != transport)
            {
                return Err(ProblemError::Contract(
                    "connected path/source sheet/orientation binding mismatch".into(),
                )
                .into());
            }
        }
        if path
            .request
            .policy
            .events
            .as_ref()
            .is_some_and(|p| p.endpoints)
        {
            event_probe(
                path,
                EventPoint {
                    point: &point,
                    orientation: &orientation,
                    localization: None,
                },
                &mut supplier,
                &mut curvature,
                &execution,
                &mut outcome,
                &mut events,
            )?;
            if outcome.terminal.is_some() {
                return Ok(());
            }
        }
        for _ in 0..path.request.policy.steps {
            execution.check()?;
            let tangent = match arclength::tangent(
                arclength::Request {
                    family: &path.family,
                    point: &point,
                    previous: &orientation,
                    source,
                    residual_limits: &path.original.tolerances.rows,
                    backward_limit: path.request.policy.backward_limit,
                    limits: path.request.policy.tangent,
                },
                &mut supplier,
                &execution,
            ) {
                Ok(tangent) => {
                    outcome.tangent_work.push(tangent.work);
                    tangent
                }
                Err(failure) => {
                    outcome.tangent_work.push(failure.work);
                    outcome.terminal = Some(failure.cause);
                    return Ok(());
                }
            };
            loop {
                let displacement = steps.step();
                execution.check()?;
                let predictor = point
                    .iter()
                    .zip(&tangent.physical)
                    .map(|(p, t)| p + displacement * t)
                    .collect::<Vec<_>>();
                let trial = path
                    .family
                    .validate_point(&predictor)
                    .map_err(ProblemError::from)
                    .map_err(MathRuntimeError::from)
                    .and_then(|()| {
                        outcome.native_calls += 1;
                        correct(
                            PathExecution {
                                service,
                                path,
                                execution: &execution,
                                budget,
                                retained: &mut retained,
                            },
                            &predictor,
                            &tangent.normalized,
                            source,
                        )
                    });
                let (report, cause, retry) = match trial {
                    Ok(report) => {
                        let report = Arc::new(report.with_owner(outcome.reports_owner.clone()));
                        let failure = report_failure(&report);
                        outcome.attempts.push(report.clone());
                        match failure {
                            None => (Some(report), None, false),
                            Some((cause, retry)) => (Some(report), Some(cause), retry),
                        }
                    }
                    Err(error) => {
                        let cause = Arc::new(error.into_problem());
                        let retry = retry_cause(&cause);
                        (None, Some(cause), retry)
                    }
                };
                if let Some(cause) = cause {
                    if retry && steps.rejected(crate::math::strategy::failure(&cause)) {
                        if path.profile().controls.reuse != ReusePolicy::RequireReuse {
                            retained.clear();
                        }
                        continue;
                    }
                    outcome.terminal = Some(cause);
                    return Ok(());
                }
                let report = report
                    .ok_or_else(|| ProblemError::internal("path corrector report missing"))?;
                let next = report
                    .candidate
                    .as_ref()
                    .ok_or_else(|| ProblemError::internal("path correction candidate missing"))?
                    .primal
                    .clone();
                path.family.validate_point(&next)?;
                execution.check()?;
                let mut selection_evidence = None;
                if let (Some(selection), Some(previous)) = (&path.request.selection, &chart) {
                    let (next_chart, evidence) =
                        selection.connect(previous, point[n], &next, &execution)?;
                    selection_evidence = Some(evidence);
                    if next_chart.is_none() {
                        outcome.observations.push(PathObservation {
                            point: next,
                            displacement,
                            tangent,
                            report,
                            selection: selection_evidence,
                        });
                        outcome.terminal = Some(Arc::new(ProblemError::Unsupported(
                            "required selected sheet coverage remains unestablished".into(),
                        )));
                        return Ok(());
                    }
                    let next_chart = next_chart
                        .ok_or_else(|| ProblemError::internal("connected chart missing"))?;
                    transport = completed_transport(
                        transport,
                        path,
                        selection,
                        TransportEdge {
                            previous,
                            next: &next_chart,
                            origin: &point,
                            endpoint: &next,
                            orientation: orientation_id,
                        },
                        evidence,
                    );
                    chart = Some(next_chart);
                }
                if path.request.policy.events.is_some() {
                    localize_event(
                        PathExecution {
                            service,
                            path,
                            execution: &execution,
                            budget,
                            retained: &mut retained,
                        },
                        EventBracket {
                            left: &point,
                            right: &next,
                            tangent: &tangent,
                            displacement,
                        },
                        &mut supplier,
                        &mut curvature,
                        &mut outcome,
                        &mut events,
                    )?;
                    if outcome.terminal.is_some() {
                        return Ok(());
                    }
                }
                steps.accepted_reset(path.request.policy.step)?;
                orientation = tangent.normalized.clone();
                point = next.clone();
                source = native::square_response::point_key(&point);
                outcome.observations.push(PathObservation {
                    point: next,
                    displacement,
                    tangent,
                    report,
                    selection: selection_evidence,
                });
                break;
            }
        }
        if path.branch.connected.is_some() {
            let target = path
                .request
                .target
                .root_target_parameters(&path.original, &path.program.parameters)?;
            if target[0].1.to_bits() != point[n].to_bits() {
                execution.check()?;
                outcome.native_calls += 1;
                let report = Arc::new(
                    correct_target(
                        service,
                        path,
                        &point[..n],
                        target[0].1,
                        &execution,
                        budget,
                        &mut retained,
                    )?
                    .with_owner(outcome.reports_owner.clone()),
                );
                outcome.attempts.push(report.clone());
                if let Some((cause, _)) = report_failure(&report) {
                    outcome.terminal = Some(cause);
                    return Ok(());
                }
                let candidate = report
                    .candidate
                    .as_ref()
                    .ok_or_else(|| ProblemError::internal("path fixed-target candidate missing"))?;
                let next = augmented(&candidate.primal, target[0].1);
                path.family.validate_point(&next)?;
                let selection =
                    path.request.selection.as_ref().ok_or_else(|| {
                        ProblemError::internal("connected source supplier missing")
                    })?;
                let previous = chart
                    .as_ref()
                    .ok_or_else(|| ProblemError::internal("connected source chart missing"))?;
                let (next_chart, evidence) =
                    selection.connect(previous, point[n], &next, &execution)?;
                let tangent = outcome
                    .observations
                    .last()
                    .ok_or_else(|| {
                        ProblemError::internal(
                            "path target bridge lacks an oriented source segment",
                        )
                    })?
                    .tangent
                    .clone();
                outcome.observations.push(PathObservation {
                    point: next.clone(),
                    displacement: 0.,
                    tangent,
                    report,
                    selection: Some(evidence),
                });
                let Some(next_chart) = next_chart else {
                    outcome.terminal = Some(Arc::new(ProblemError::Unsupported(
                        "fixed original target sheet bridge remains unestablished".into(),
                    )));
                    return Ok(());
                };
                transport = completed_transport(
                    transport,
                    path,
                    selection,
                    TransportEdge {
                        previous,
                        next: &next_chart,
                        origin: &point,
                        endpoint: &next,
                        orientation: orientation_id,
                    },
                    evidence,
                );
                chart = Some(next_chart);
                point = next;
            }
        }
        if let Some(endpoint) = &chart {
            outcome.connected = Some(pse_model::strategy::ConnectedPath {
                path: path.family.key(),
                sheet: chart_key(endpoint),
                transport,
                orientation: orientation_id,
            });
        }
        if path.branch.connected.is_some()
            && let (Some(selection), Some(chart)) = (&path.request.selection, chart.take())
        {
            outcome.endpoint = Some(Arc::new(PathEndpoint {
                selection: selection.clone(),
                chart: Arc::new(chart),
                parameter: point[n],
                target: path.target_identity,
                scope: path.task_scope.clone(),
                _owner: outcome.reports_owner.clone(),
            }));
        }
        let branch = if path.branch.connected.is_some() {
            BranchPolicy {
                kind: pse_model::strategy::BranchKind::Connected,
                connected: outcome.connected,
            }
        } else {
            path.branch
        };
        let mut source_key = start.source;
        source_key.derivation = Some(path.family.key());
        source_key.branch = path.branch.connected.map(|_| transport);
        outcome.proposal =
            Some(
                Proposal::path(
                    path.case.prepared.compiled().plan.columns().to_vec(),
                    point[..n].to_vec(),
                    source_key,
                    path.request.target.original_identity()?,
                    branch,
                )?
                .with_owner(outcome._owner.clone().ok_or_else(|| {
                    ProblemError::internal("path original allocation owner missing")
                })?),
            );
        if outcome
            .proposal
            .as_ref()
            .is_some_and(|p| p.retained_bytes().is_err())
        {
            return Err(ProblemError::memory("path proposal retained extent").into());
        }
        execution.check()?;
        Ok(())
    })();
    if let Err(error) = result {
        outcome.terminal = Some(Arc::new(error.into_problem()));
        outcome.proposal = None;
    }
    outcome.auxiliary_evaluations = calls.load(Ordering::Acquire);
    outcome.events = pse_math::SharedAllocation::from(Arc::new(events))
        .with_owner(outcome.reports_owner.clone());
    outcome
}
struct PathCancellation(Option<Arc<std::sync::atomic::AtomicBool>>);
impl Drop for PathCancellation {
    fn drop(&mut self) {
        if let Some(cancel) = &self.0 {
            cancel.store(true, Ordering::Release);
        }
    }
}
impl MathService {
    /// Submit the bounded auxiliary operation as one completion-owned scoped job.
    /// Waiting, providers, native calls and retained copied reports share its deadline.
    /// # Errors
    /// Job admission, cancellation, native scope failure or late original-scope expiry.
    pub async fn path_task(
        self: &Arc<Self>,
        path: PreparedPath,
        start: PathStart,
        driver: &crate::CancelSource,
    ) -> Result<PathOutcome, MathRuntimeError> {
        path.scope.check().map_err(ProblemError::Provider)?;
        let retained_allowance = path.result_bytes()?;
        let extra_foreign = self
            .policy
            .foreign_allowance(&path.request.corrector.controls)
            .saturating_sub(self.policy.foreign_bytes);
        let bytes = path
            .worker_bytes()
            .checked_add(extra_foreign)
            .and_then(|v| v.checked_add(retained_allowance))
            .ok_or(MathRuntimeError::Limit("path job extent"))?;
        let control = FlightCancellation::default();
        let service = self.clone();
        let scope = path.scope.clone();
        let mut execution = Execution::within(
            scope.cancellation().clone(),
            &path.request.corrector.controls,
            scope.clone(),
        )?;
        execution.memory = Some(
            self.policy
                .foreign_allowance(&path.request.corrector.controls),
        );
        let worker_budget = WorkerBudget::new(path.worker_bytes());
        let mut abandoned = PathCancellation(Some(scope.cancellation().clone()));
        let threads = path.request.corrector.controls.threads;
        let operation = self.job_retained_scoped(
            threads,
            bytes,
            control.clone(),
            scope.deadline(),
            move |_| {
                let mut result = None;
                let mut start = Some(start);
                execution::adapter(Backend::Ipopt).scope(
                    threads,
                    service.policy.stack_bytes,
                    &mut || {
                        if let Some(start) = start.take() {
                            result = Some(run_path(
                                &service,
                                &path,
                                start,
                                execution.clone(),
                                &worker_budget,
                                path._owner.clone(),
                            ));
                        }
                    },
                )?;
                let outcome = result.ok_or_else(|| {
                    ProblemError::internal("path native owning scope did not execute")
                })?;
                Ok((outcome, retained_allowance))
            },
        );
        tokio::pin!(operation);
        let result = tokio::select! {result=&mut operation=>result,()=driver.cancelled()=>{scope.cancellation().store(true,Ordering::Release);control.cancel();let _=operation.await;Err(MathRuntimeError::Cancelled)}};
        abandoned.0 = None;
        result.map(|(mut outcome, owner)| {
            if let Ok(mut owners) = outcome.reports_owner.lock() {
                owners.push(owner.clone());
            }
            outcome.proposal = outcome.proposal.take().map(|p| p.with_owner(owner.clone()));
            outcome._owner = Some(owner);
            outcome
        })
    }
}

#[cfg(test)]
mod tests;
impl PathSelection {
    /// Frozen compiler/provider/numerical preparation covered by this producer.
    pub fn source(&self) -> ContentHash {
        self.source
    }
    /// Actual admitted original state inventory.
    pub fn coordinates(&self) -> &[SemanticId] {
        &self.coordinates
    }
    /// Mathematical transport supported by this source-issued supplier.
    pub fn coverage(&self) -> ChartChainCoverage {
        self.coverage
    }
    /// Actual originating chart identity; a regime label never substitutes for it.
    pub fn sheet(&self) -> ContentHash {
        chart_key(&self.previous)
    }
}
impl MathService {
    /// Produce a RootSheet supplier from the genuine compiled scalar root program and
    /// an original-permitted endpoint. Exact source guards and actual row offsets enter
    /// the same library projection; unsupported or nonfinite domains refuse admission.
    /// A singleton zero-score root union establishes no authored minimum selector.
    /// # Errors
    /// Missing source support, nonexact projection, failed endpoint certification or scope/resources.
    pub async fn prepare_path_selection(
        self: &Arc<Self>,
        original: PreparedSolve,
        start: PathStart,
        parameter: SemanticId,
        interval: (f64, f64),
        scope: ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<PathSelection, MathRuntimeError> {
        #[cfg(not(feature = "solver-root-isolation"))]
        {
            let _ = (original, start, parameter, interval, scope, driver);
            Err(ProblemError::Unsupported(
                "compiled path chart supplier requires linked validated root isolation".into(),
            )
            .into())
        }
        #[cfg(feature = "solver-root-isolation")]
        {
            let local = std::time::Instant::now()
                .checked_add(original.time_limit())
                .ok_or_else(|| ProblemError::Contract("path proof deadline extent".into()))?;
            let scope = ExecutionScope::new(
                scope.cancellation().clone(),
                Some(scope.deadline().map_or(local, |d| d.min(local))),
            );
            let mut abandoned = PathCancellation(Some(scope.cancellation().clone()));
            let result=async {
            scope.check().map_err(ProblemError::Provider)?;
            if original.semantic_point_key(&start.point)?!=start.source || original.profile.intent!=SolveIntent::Root {return Err(ProblemError::Contract("compiled path chart origin does not match original completion source".into()).into());}
            let Representation::Algebraic(case)=&original.representation else {return Err(ProblemError::Unsupported("compiled path proof requires original algebraic equations".into()).into());};
            let program=case.sensitivity.as_ref().and_then(ParametricPreparation::available).ok_or_else(||ProblemError::Unsupported("compiled path proof lacks an actual scalar First program".into()))?;
            if program.parameters.len()!=1 || program.parameters[0].0!=parameter || program.parameters[0].1<interval.0 || program.parameters[0].1>interval.1 || program.program.assembly.order()<DerivativeOrder::First {return Err(ProblemError::Unsupported("compiled path proof requires one actual First parameter".into()).into());}
            let source=original.preparation_identity()?;let coordinates=case.prepared.compiled().plan.columns().to_vec();let values=case.values.clone();let assembly=program.program.assembly.clone();let parameter_value=program.parameters[0].1;
            let actual=native::assembled::contract(&assembly);let n=coordinates.len();
            let unknowns=actual.variables[..n].iter().map(|v|pse_math::implicit::Unknown {id:v.id,lower:v.lower,upper:v.upper}).collect::<Vec<_>>();
            let control=FlightCancellation::default();let projection_scope=scope.clone();let limit=self.policy.worker_bytes/256;
            let projection=self.job_retained_scoped(1,self.policy.worker_bytes,control.clone(),scope.deadline(),move |_| {
                projection_scope.check().map_err(ProblemError::Provider)?;
                let (identity,program)=assembly.root_path_isolation_program(&values,parameter,interval,limit,projection_scope.cancellation()).map_err(|e|match e {pse_math::factorable::FactorableError::Math(cause)=>ProblemError::Math(cause),cause=>ProblemError::Math(pse_math::MathError::Typed {retained:size_of_val(&cause),cause:pse_model::diagnostic::DiagnosticCause::new(cause)})})?.ok_or_else(||ProblemError::Unsupported("compiled path source cannot furnish exact First guards and a finite complete root domain".into()))?;
                projection_scope.check().map_err(ProblemError::Provider)?;let bytes=program.retained_bytes();Ok(((identity,Arc::new(program)),bytes))
            });
            tokio::pin!(projection);
            let ((identity,program),projection_owner)=tokio::select! {result=&mut projection=>result?,()=driver.cancelled()=>{scope.cancellation().store(true,Ordering::Release);control.cancel();let _=projection.await;return Err(MathRuntimeError::Cancelled);}};
            let verifier:Arc<dyn SelectionVerifier>=Arc::new(native::root_isolation::Ibex);
            let workspace=verifier.workspace_bytes(std::slice::from_ref(&program))?;
            if workspace>self.policy.workspace_bytes {return Err(MathRuntimeError::Limit("compiled path proof workspace allowance"));}
            let chart_bytes=n.checked_mul(256).and_then(|v|v.checked_add(size_of::<PathSelection>()+4096)).ok_or(MathRuntimeError::Limit("compiled path chart extent"))?;
            let bytes=workspace.checked_add(chart_bytes).ok_or(MathRuntimeError::Limit("compiled path proof extent"))?;
            let selection=case.prepared.compiled().plan.structure().rows()[0].id;
            let alternative=SelectionScope {id:selection,program,residual_identity:identity,unknowns};let proof_scope=scope.clone();let proof_verifier=verifier.clone();let control=FlightCancellation::default();let allowance=original.time_limit();
            let proof=self.job_retained_scoped(1,bytes,control.clone(),scope.deadline(),move |_| {
                proof_scope.check().map_err(ProblemError::Provider)?;
                let alternatives=[SelectionAlternative {id:alternative.id,program:&alternative.program,residual_identity:alternative.residual_identity,unknowns:&alternative.unknowns}];let parameters=[parameter_value];
                let request=SelectionProofRequest {selection,alternatives:&alternatives,winner:0,parameters:&parameters,candidate:&start.point,order:DerivativeOrder::First,time_limit:proof_scope.remaining(allowance).map_err(ProblemError::Provider)?,cancel:proof_scope.cancellation()};
                let previous=match proof_verifier.certify(&request)? {SelectionEvidence::Unique(chart)=>chart,evidence=>return Err(ProblemError::Unsupported(format!("actual compiled path endpoint chart: {evidence:?}")).into())};
                previous.validate(&request,proof_verifier.identity())?;proof_scope.check().map_err(ProblemError::Provider)?;
                Ok((SelectionData {source,parameter,coordinates,verifier:proof_verifier,previous,alternatives:vec![alternative],winner:0,coverage:ChartChainCoverage::RootSheet,_owner:Some(projection_owner)},chart_bytes))
            });
            tokio::pin!(proof);
            let (mut selected,owner)=tokio::select! {result=&mut proof=>result?,()=driver.cancelled()=>{scope.cancellation().store(true,Ordering::Release);control.cancel();let _=proof.await;return Err(MathRuntimeError::Cancelled);}};
            selected._owner=Some(pse_math::retain_allocation_owner(selected._owner.take(),owner));Ok(PathSelection(Arc::new(selected)))
            }.await;
            abandoned.0 = None;
            result
        }
    }
}
