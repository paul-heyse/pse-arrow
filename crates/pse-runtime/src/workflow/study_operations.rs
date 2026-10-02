// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reconstructable study operations dispatch through their existing operation owners.
use super::Runtime;
use super::{
    AdvancedStep, Arrival, DeclaredProcedure, FitProfile, FitUncertainty, Horizon,
    HorizonController, HorizonEstimator, HorizonInput, HorizonSignal, ModelingPackage,
    ModelingSimulation, ModelingSolvePreparation, PreparedFit, RunHandle, RunResult,
    SimulationProfile, WindowInput, WorkflowError,
};
use crate::CancelSource;
use crate::math::settings::SolveSettings;
use pse_compiler::workspace::Profile;
use pse_diagnostics::{DiagnosticRule, DiagnosticStage};
use pse_ids::{ContentHash, SemanticId};
use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic, Observation};
use pse_model::document::Version;
use pse_model::generated::enums::{FitDerivatives, ModelingAnalysisRoute};
use pse_model::generated::identities::{DeclarationId, FitId, InstanceId};
use pse_model::study::{ScientificFacts, SeedNeed, SeedRole};
use pse_modeling::Limits;
use std::collections::BTreeMap;

/// Exact source/context required when reconstructing an operation from stored bundles.
#[derive(
    Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct OperationSource {
    /// Immutable modeling revision, including included scientific data.
    pub revision: pse_ids::roles::SourceRevisionHash,
    /// Complete admitted physical context identity.
    pub physical_context: ContentHash,
}
impl OperationSource {
    /// Capture the selected immutable operation inputs.
    pub fn of(package: &ModelingPackage) -> Self {
        Self {
            revision: package.revision.identity(),
            physical_context: package.physical.identity(),
        }
    }
    pub(in crate::workflow) fn check(
        &self,
        package: &ModelingPackage,
    ) -> Result<(), WorkflowError> {
        if *self != Self::of(package) {
            return Err(refusal(
                DiagnosticRule::StudyBindingRevision,
                BoundaryClass::Incompatible,
                [],
                "operation source/context differs from the selected immutable revision",
            ));
        }
        Ok(())
    }
}

/// Serialization of the existing compiler/expansion controls, with no second defaults.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparationSettings {
    /// Existing evaluator profile, encoded through its complete remote projection.
    #[serde(with = "ProfileWire")]
    #[schemars(with = "ProfileWire")]
    pub compiler: Profile,
    /// Existing expansion admission limits.
    #[serde(with = "LimitsWire")]
    #[schemars(with = "LimitsWire")]
    pub limits: Limits,
}
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(remote = "Profile", deny_unknown_fields)]
#[schemars(rename = "StudyCompilerProfile")]
struct ProfileWire {
    #[serde(with = "OptimizationWire")]
    #[schemars(with = "OptimizationWire")]
    optimization: pse_math::library::Optimization,
    #[serde(with = "EvaluationWire")]
    #[schemars(with = "EvaluationWire")]
    evaluation: pse_math::jets::EvaluationLimits,
}
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(remote = "pse_math::library::Optimization", deny_unknown_fields)]
struct OptimizationWire {
    cores: usize,
    horner_iterations: usize,
    cpe_iterations: usize,
}
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(remote = "pse_math::jets::EvaluationLimits", deny_unknown_fields)]
struct EvaluationWire {
    derivative_components: usize,
    operations: usize,
    scratch_bytes: usize,
    provider_calls: usize,
}
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(remote = "Limits", deny_unknown_fields)]
struct LimitsWire {
    depth: usize,
    items: usize,
    members: usize,
    body_occurrences: Option<usize>,
    body_slots: Option<usize>,
}

/// One authored solve operation under its admitted route and existing settings document.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaseOperation {
    /// Authored case identity.
    pub case: DeclarationId,
    /// Route captured from the declaration's admission.
    pub route: ModelingAnalysisRoute,
    /// Complete existing solve settings.
    pub settings: SolveSettings,
}
/// One authored integrated simulation and its existing profile document.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SimulationOperation {
    /// Authored integration case.
    pub case: DeclarationId,
    /// Explicit existing integration controls; absence requests authored controls.
    pub profile: Option<SimulationProfile>,
}
/// Fit-only controls compose the existing solve/integration settings documents once.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FitOperationSettings {
    /// Existing native solve controls.
    pub solver: SolveSettings,
    /// Explicit profiles for transient fit experiments.
    pub simulations: BTreeMap<InstanceId, SimulationProfile>,
    /// Existing local response rank cutoff.
    pub rank_tolerance: f64,
    /// Existing derivative/rank allocation bound.
    pub max_cells: usize,
    /// Existing fit derivative source vocabulary.
    pub derivatives: FitDerivatives,
    /// Existing uncertainty/interval request.
    pub uncertainty: Option<FitUncertainty>,
}
impl FitOperationSettings {
    /// Project into the fitting owner's profile; its preparation performs admission.
    /// # Errors
    /// Invalid solve settings retain their original typed cause.
    pub fn profile(&self) -> Result<FitProfile, WorkflowError> {
        Ok(FitProfile {
            solver: self.solver.clone().profile()?,
            simulations: self.simulations.clone(),
            rank_tolerance: self.rank_tolerance,
            max_cells: self.max_cells,
            derivatives: self.derivatives,
            uncertainty: self.uncertainty.clone(),
        })
    }
}
/// One authored fit selection; experiment data stays in the pinned source revision.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FitOperation {
    /// Existing authored fit identity.
    pub fit: FitId,
    /// Complete fit controls.
    pub settings: FitOperationSettings,
}

/// Serializes the existing horizon signal vocabulary without replacing its semantics.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HorizonBinding {
    /// Controller case target path, resolved by the horizon owner.
    pub target: String,
    /// Existing horizon signal.
    pub signal: HorizonSignalDocument,
}
/// Physically typed public signal intent; canonical numerical signals are admitted once.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum HorizonSignalDocument {
    /// Existing measured plant output.
    Measured(SemanticId),
    /// Existing estimated case path.
    Estimated(String),
    /// Existing driven input index.
    Applied(usize),
    /// Physically typed desired values, admitted against the controller target.
    Trajectory(Vec<super::BindingQuantity>),
}
/// A plant input projected into the existing horizon owner.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HorizonInputDocument {
    /// Existing plant parameter identity.
    pub parameter: SemanticId,
    /// Initial held quantity with complete physical meaning.
    pub initial: super::BindingQuantity,
}
/// Serializable controller inputs; package/prepared analysis handles are reconstructed.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControllerOperation {
    /// Existing simultaneous controller case.
    pub case: CaseOperation,
    /// Existing signal binding intent.
    pub bindings: Vec<HorizonBinding>,
    /// Existing move path/input-index mapping.
    pub moves: Vec<(String, usize)>,
    /// Existing advanced-step prediction mapping; absence means full solves.
    pub predictions: Option<Vec<(String, String)>>,
}
/// Existing estimator window input intent.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstimatorInput {
    /// Driven input index.
    pub input: usize,
    /// Existing whole-window or per-period bindings.
    #[serde(with = "WindowWire")]
    #[schemars(with = "WindowWire")]
    pub binding: WindowInput,
}
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    remote = "WindowInput",
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum WindowWire {
    Constant(String),
    Periods(Vec<String>),
}
/// Existing arrival cost intent with a checked finite initial prior.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArrivalDocument {
    /// Existing prior case path.
    pub prior: String,
    /// Existing next-prior solution path.
    pub next: String,
    /// Initial prior quantity with complete physical meaning.
    pub initial: super::BindingQuantity,
}
/// Serializable estimator inputs for the existing horizon owner.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstimatorOperation {
    /// Existing simultaneous estimator case.
    pub case: CaseOperation,
    /// Existing estimation window.
    pub window: usize,
    /// Existing measurement-to-case-path mapping.
    pub measurements: Vec<(SemanticId, Vec<String>)>,
    /// Existing driven-input mapping.
    pub inputs: Vec<EstimatorInput>,
    /// Existing arrival prior mapping.
    pub arrival: Vec<ArrivalDocument>,
}
/// Reconstructable closed-loop operation in one pinned source/context.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HorizonOperation {
    /// Existing authored plant simulation.
    pub plant: SimulationOperation,
    /// Existing positive physical sample period in seconds.
    pub period: pse_model::scalars::Tolerance,
    /// Existing closed-loop sample count.
    pub steps: pse_model::scalars::PositiveCount,
    /// Driven plant inputs.
    pub inputs: Vec<HorizonInputDocument>,
    /// Optional existing estimator.
    pub estimator: Option<EstimatorOperation>,
    /// Optional existing controller.
    pub controller: Option<ControllerOperation>,
}
/// Supported operation owner selection, with no arbitrary closure or second engine.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "request",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum OperationRequest {
    /// Existing authored algebraic solve owner.
    DeclaredCase(CaseOperation),
    /// Existing authored integration owner.
    Simulation(SimulationOperation),
    /// Existing shared-parameter fitting owner.
    Fit(FitOperation),
    /// Existing closed-loop horizon owner.
    Horizon(Box<HorizonOperation>),
}
/// Registry-owned expected products; these roles never assert result availability.
pub use pse_model::generated::enums::StudyResultRole;
/// Canonical horizon assignments admitted in the selected physical context.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdmittedHorizonValues {
    /// Canonical plant parameter assignments in declared input order.
    pub inputs: Vec<super::AdmittedBindingEntry>,
    /// Canonical estimator priors in declared arrival order.
    pub arrival: Vec<super::AdmittedBindingEntry>,
    /// Canonical controller trajectory values by declared binding index.
    pub trajectories: BTreeMap<usize, Vec<super::AdmittedBindingEntry>>,
}
/// Immutable request recorded by a study occurrence and replayed by either executor.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyOperation {
    /// Interpretation of this operation descriptor.
    pub version: Version<1>,
    /// Exact source/context reconstruction precondition.
    pub source: OperationSource,
    /// Complete preparation controls using existing owners' remote projections.
    pub preparation: PreparationSettings,
    /// Existing operation-owned input document.
    pub operation: OperationRequest,
    /// Canonical physical assignments; required for every horizon descriptor.
    pub admitted_horizon: Option<AdmittedHorizonValues>,
}
/// Prepared operation products; only the existing operation supervisors execute these.
#[derive(Clone, Debug)]
pub enum PreparedStudyOperation {
    /// Existing algebraic preparation.
    DeclaredCase(Box<ModelingSolvePreparation>),
    /// Existing integration preparation.
    Simulation(Box<ModelingSimulation>),
    /// Existing fitting preparation.
    Fit(Box<PreparedFit>),
    /// Existing closed-loop specification reconstructed from pinned cases.
    Horizon(Box<Horizon>),
}
impl StudyOperation {
    /// Admit raw horizon quantities once; replay validates these canonical entries only.
    pub(crate) async fn admit_horizon_values(
        &mut self,
        package: &ModelingPackage,
        cancel: &CancelSource,
    ) -> Result<(), WorkflowError> {
        self.source.check(package)?;
        let OperationRequest::Horizon(spec) = &self.operation else {
            return Ok(());
        };
        let compiler = self.preparation.compiler;
        let limits = self.preparation.limits;
        let plant = package
            .declared_simulation(
                spec.plant.case,
                compiler,
                spec.plant.profile.clone(),
                limits,
                cancel,
            )
            .await?;
        let inputs = spec
            .inputs
            .iter()
            .map(|input| {
                package.admit_target_quantity(
                    plant.model().compiled(),
                    input.parameter,
                    &input.initial,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut arrival = Vec::new();
        let mut trajectories = BTreeMap::new();
        if let Some(estimate) = &spec.estimator {
            let execution = declared_paths(
                package,
                &estimate.case,
                compiler,
                limits,
                cancel,
                estimate
                    .arrival
                    .iter()
                    .map(|prior| prior.prior.clone())
                    .collect(),
            )
            .await?;
            for prior in &estimate.arrival {
                let member = target_member(execution.model.compiled(), &prior.prior)?;
                arrival.push(package.admit_target_quantity(
                    execution.model.compiled(),
                    member,
                    &prior.initial,
                )?);
            }
        }
        if let Some(control) = &spec.controller {
            let execution = declared_paths(
                package,
                &control.case,
                compiler,
                limits,
                cancel,
                control
                    .bindings
                    .iter()
                    .filter(|binding| {
                        matches!(binding.signal, HorizonSignalDocument::Trajectory(_))
                    })
                    .map(|binding| binding.target.clone())
                    .collect(),
            )
            .await?;
            for (index, binding) in control.bindings.iter().enumerate() {
                if let HorizonSignalDocument::Trajectory(values) = &binding.signal {
                    let member = target_member(execution.model.compiled(), &binding.target)?;
                    trajectories.insert(
                        index,
                        values
                            .iter()
                            .map(|q| {
                                package.admit_target_quantity(execution.model.compiled(), member, q)
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
            }
        }
        self.admitted_horizon = Some(AdmittedHorizonValues {
            inputs,
            arrival,
            trajectories,
        });
        Ok(())
    }
    fn validate_entry(
        &self,
        package: &ModelingPackage,
        model: &pse_compiler::workspace::PreparedModeling,
        target: SemanticId,
        entry: &super::AdmittedBindingEntry,
    ) -> Result<f64, WorkflowError> {
        if entry.member != target {
            let mut diagnostic = BoundaryDiagnostic::new(
                BoundaryClass::Incompatible,
                DiagnosticStage::StudyBinding,
                [target, entry.member],
                DiagnosticRule::StudyBindingTarget,
            );
            diagnostic.observations.insert(
                "expected_member".into(),
                Observation::Text(target.to_string()),
            );
            diagnostic.observations.insert(
                "actual_member".into(),
                Observation::Text(entry.member.to_string()),
            );
            diagnostic
                .locations
                .push(pse_model::diagnostic::SourceLocation {
                    source: target,
                    revision: Some(self.source.revision.as_id()),
                    path: target.to_string(),
                    name: None,
                    start: None,
                    end: None,
                });
            return Err(diagnostic.into());
        }
        package.validate_binding(
            model,
            &super::AdmittedBinding {
                revision: self.source.revision,
                context: self.source.physical_context,
                entries: BTreeMap::from([(entry.member, entry.clone())]),
            },
        )?;
        Ok(entry.canonical.into_inner())
    }
    /// Declared owner products, never a claim that the run produced usable members.
    pub fn output_roles(&self) -> Vec<StudyResultRole> {
        match &self.operation {
            OperationRequest::DeclaredCase(_) => vec![StudyResultRole::CaseResult],
            OperationRequest::Simulation(_) => vec![StudyResultRole::Trajectory],
            OperationRequest::Fit(fit) => {
                let mut roles = vec![
                    StudyResultRole::ParameterEstimates,
                    StudyResultRole::ParameterCovariance,
                ];
                if fit
                    .settings
                    .uncertainty
                    .as_ref()
                    .is_some_and(|uncertainty| uncertainty.profile.is_some())
                {
                    roles.push(StudyResultRole::ProfileIntervals);
                }
                roles
            }
            OperationRequest::Horizon(_) => vec![StudyResultRole::HorizonHistory],
        }
    }

    /// Reconstruct and admit the selected existing operation under its exact revision.
    /// # Errors
    /// Source/route mismatch, unsupported procedures and original operation-owner refusals.
    pub async fn prepare(
        &self,
        package: &ModelingPackage,
        cancel: &CancelSource,
    ) -> Result<PreparedStudyOperation, WorkflowError> {
        self.source.check(package)?;
        if self.admitted_horizon.is_some()
            && !matches!(self.operation, OperationRequest::Horizon(_))
        {
            return Err(super::contract(
                "canonical horizon assignments supplied to another operation",
            ));
        }
        let compiler = self.preparation.compiler;
        let limits = self.preparation.limits;
        match &self.operation {
            OperationRequest::DeclaredCase(case) => {
                let execution = declared(package, case, compiler, limits, cancel).await?;
                Ok(PreparedStudyOperation::DeclaredCase(Box::new(
                    package.prepare_declared(&execution, cancel).await?,
                )))
            }
            OperationRequest::Simulation(simulation) => {
                Ok(PreparedStudyOperation::Simulation(Box::new(
                    package
                        .declared_simulation(
                            simulation.case,
                            compiler,
                            simulation.profile.clone(),
                            limits,
                            cancel,
                        )
                        .await?,
                )))
            }
            OperationRequest::Fit(fit) => Ok(PreparedStudyOperation::Fit(Box::new(
                package
                    .prepare_fit(fit.fit, fit.settings.profile()?, compiler, limits, cancel)
                    .await?,
            ))),
            OperationRequest::Horizon(spec) => {
                if !cfg!(feature = "solver-diffsol") {
                    return Err(refusal(
                        DiagnosticRule::StudyOperationUnsupported,
                        BoundaryClass::Unsupported,
                        [spec.plant.case.as_id()],
                        "horizon execution requires the linked horizon owner",
                    ));
                }
                let canonical = self.admitted_horizon.as_ref().ok_or_else(|| {
                    super::contract("horizon quantities require contextual admission")
                })?;
                if canonical.inputs.len() != spec.inputs.len()
                    || canonical.arrival.len()
                        != spec.estimator.as_ref().map_or(0, |e| e.arrival.len())
                {
                    return Err(super::contract(
                        "horizon canonical assignment cardinality differs from request",
                    ));
                }
                let plant = package
                    .declared_simulation(
                        spec.plant.case,
                        compiler,
                        spec.plant.profile.clone(),
                        limits,
                        cancel,
                    )
                    .await?;
                if spec.controller.is_none() && !canonical.trajectories.is_empty() {
                    return Err(super::contract(
                        "canonical trajectories require a controller",
                    ));
                }
                let controller = if let Some(control) = &spec.controller {
                    let execution = declared_paths(
                        package,
                        &control.case,
                        compiler,
                        limits,
                        cancel,
                        control
                            .bindings
                            .iter()
                            .map(|binding| binding.target.clone())
                            .collect(),
                    )
                    .await?;
                    let mut bindings = Vec::new();
                    for (index, binding) in control.bindings.iter().enumerate() {
                        let signal = match &binding.signal {
                            HorizonSignalDocument::Measured(id) => HorizonSignal::Measured(*id),
                            HorizonSignalDocument::Estimated(path) => {
                                HorizonSignal::Estimated(path.clone())
                            }
                            HorizonSignalDocument::Applied(input) => HorizonSignal::Applied(*input),
                            HorizonSignalDocument::Trajectory(raw) => {
                                let entries =
                                    canonical.trajectories.get(&index).ok_or_else(|| {
                                        super::contract("missing canonical horizon trajectory")
                                    })?;
                                if entries.len() != raw.len() {
                                    return Err(super::contract(
                                        "canonical trajectory cardinality differs",
                                    ));
                                }
                                let target =
                                    target_member(execution.model.compiled(), &binding.target)?;
                                HorizonSignal::Trajectory(
                                    entries
                                        .iter()
                                        .map(|entry| {
                                            self.validate_entry(
                                                package,
                                                execution.model.compiled(),
                                                target,
                                                entry,
                                            )
                                        })
                                        .collect::<Result<Vec<_>, _>>()?,
                                )
                            }
                        };
                        bindings.push((binding.target.clone(), signal));
                    }
                    let expected = control
                        .bindings
                        .iter()
                        .filter(|b| matches!(b.signal, HorizonSignalDocument::Trajectory(_)))
                        .count();
                    if canonical.trajectories.len() != expected {
                        return Err(super::contract("extra canonical horizon trajectories"));
                    }
                    Some(HorizonController {
                        package: package.clone(),
                        analysis: execution.analysis,
                        bindings,
                        moves: control.moves.clone(),
                        advanced: control
                            .predictions
                            .as_ref()
                            .map(|predictions| AdvancedStep {
                                predictions: predictions.clone(),
                            }),
                    })
                } else {
                    None
                };
                let estimator = if let Some(estimate) = &spec.estimator {
                    let execution = declared_paths(
                        package,
                        &estimate.case,
                        compiler,
                        limits,
                        cancel,
                        estimate
                            .arrival
                            .iter()
                            .map(|prior| prior.prior.clone())
                            .collect(),
                    )
                    .await?;
                    let arrival = estimate
                        .arrival
                        .iter()
                        .zip(&canonical.arrival)
                        .map(|(arrival, entry)| {
                            let target = target_member(execution.model.compiled(), &arrival.prior)?;
                            Ok(Arrival {
                                prior: arrival.prior.clone(),
                                next: arrival.next.clone(),
                                initial: self.validate_entry(
                                    package,
                                    execution.model.compiled(),
                                    target,
                                    entry,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, WorkflowError>>()?;
                    Some(HorizonEstimator {
                        package: package.clone(),
                        analysis: execution.analysis,
                        window: estimate.window,
                        measurements: estimate.measurements.clone(),
                        inputs: estimate
                            .inputs
                            .iter()
                            .map(|input| (input.input, input.binding.clone()))
                            .collect(),
                        arrival,
                    })
                } else {
                    None
                };
                let horizon = Horizon {
                    period: spec.period.into_inner(),
                    steps: spec.steps.into_inner(),
                    inputs: spec
                        .inputs
                        .iter()
                        .zip(&canonical.inputs)
                        .map(|(input, entry)| {
                            Ok(HorizonInput {
                                parameter: input.parameter,
                                initial: self.validate_entry(
                                    package,
                                    plant.model().compiled(),
                                    input.parameter,
                                    entry,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, WorkflowError>>()?,
                    plant,
                    estimator,
                    controller,
                };
                #[cfg(feature = "solver-diffsol")]
                package
                    .runtime
                    .admit_horizon(horizon.clone(), cancel)
                    .await?;
                Ok(PreparedStudyOperation::Horizon(Box::new(horizon)))
            }
        }
    }

    /// Consumed seed role: unsupported combinations refuse before scheduling.
    /// # Errors
    /// An operation owner has no admitted seed input matching the requested role.
    pub fn admit_seed_role(&self, role: SeedRole) -> Result<(), WorkflowError> {
        if matches!(self.operation, OperationRequest::DeclaredCase(_))
            && role == SeedRole::PrimalSolution
        {
            return Ok(());
        }
        Err(refusal(
            DiagnosticRule::StudySeedIncompatible,
            BoundaryClass::Incompatible,
            [],
            "selected operation has no compatible admitted seed input role",
        ))
    }
}
fn target_member(
    model: &pse_compiler::workspace::PreparedModeling,
    path: &str,
) -> Result<SemanticId, WorkflowError> {
    model
        .model
        .paths
        .get(path)
        .copied()
        .ok_or_else(|| super::contract(format!("horizon target {path} is unavailable")))
}
impl PreparedStudyOperation {
    /// Operation-owned seed consumption, after constant evaluation is known.
    pub fn seed_need(&self) -> SeedNeed {
        match self {
            Self::DeclaredCase(case) if case.solve.compatibility().is_some() => SeedNeed::Required,
            Self::DeclaredCase(_) | Self::Simulation(_) | Self::Fit(_) | Self::Horizon(_) => {
                SeedNeed::NotNeeded
            }
        }
    }
    /// Reuse the worker's claimed attempt and its one run identity through the same owner.
    pub(crate) async fn start_attempt(
        &self,
        runtime: &Runtime,
        cancel: &CancelSource,
        attempt: super::durable::DurableAttempt,
    ) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(crate::math::MathRuntimeError::Cancelled.into());
        }
        match self {
            Self::DeclaredCase(case) => runtime.start_attempt(vec![case.as_ref().clone()], attempt),
            Self::Simulation(simulation) => simulation.start_attempt(attempt),
            Self::Fit(fit) => fit.start_attempt(attempt),
            Self::Horizon(horizon) => {
                #[cfg(feature = "solver-diffsol")]
                {
                    runtime
                        .start_horizon_attempt(horizon.as_ref().clone(), cancel, attempt)
                        .await
                }
                #[cfg(not(feature = "solver-diffsol"))]
                {
                    let _ = (horizon, attempt);
                    Err(refusal(
                        DiagnosticRule::StudyOperationUnsupported,
                        BoundaryClass::Unsupported,
                        [],
                        "horizon owner is unavailable",
                    ))
                }
            }
        }
    }
    /// Dispatch through the existing joined operation supervisor.
    /// # Errors
    /// Native admission/cancellation and existing supervisor failures retain their causes.
    pub async fn start(
        &self,
        runtime: &Runtime,
        cancel: &CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        if cancel.token().is_cancelled() {
            return Err(crate::math::MathRuntimeError::Cancelled.into());
        }
        match self {
            Self::DeclaredCase(case) => {
                runtime
                    .start_modeling(vec![case.as_ref().clone()], false, cancel)
                    .await
            }
            Self::Simulation(simulation) => simulation.start(),
            Self::Fit(fit) => fit.start(),
            Self::Horizon(horizon) => {
                #[cfg(feature = "solver-diffsol")]
                {
                    runtime
                        .start_horizon(horizon.as_ref().clone(), cancel)
                        .await
                }
                #[cfg(not(feature = "solver-diffsol"))]
                {
                    let _ = horizon;
                    Err(refusal(
                        DiagnosticRule::StudyOperationUnsupported,
                        BoundaryClass::Unsupported,
                        [],
                        "horizon execution requires the linked horizon owner",
                    ))
                }
            }
        }
    }
}

/// Project E's existing final aggregate permission without promoting a first result table.
pub(in crate::workflow) fn scientific_facts(result: &RunResult) -> ScientificFacts {
    let single = match result.assessments() {
        [assessment] => Some(assessment),
        _ => None,
    };
    ScientificFacts {
        usable: result.usable(),
        candidate_use: single.map(|assessment| assessment.usability),
        seed_permission: single.is_some_and(|assessment| assessment.permits_seed),
    }
}
async fn declared(
    package: &ModelingPackage,
    case: &CaseOperation,
    compiler: Profile,
    limits: Limits,
    cancel: &CancelSource,
) -> Result<super::DeclaredExecution, WorkflowError> {
    let execution = package
        .declared_execution(
            case.case,
            compiler,
            case.settings.clone().profile()?,
            Default::default(),
            limits,
            cancel,
        )
        .await?;
    if execution.route != case.route || !matches!(execution.procedure, DeclaredProcedure::Solve) {
        return Err(refusal(
            DiagnosticRule::StudyOperationUnsupported,
            BoundaryClass::Unsupported,
            [case.case.as_id()],
            "study operation requires its admitted authored solve route/procedure",
        ));
    }
    Ok(execution)
}
async fn declared_paths(
    package: &ModelingPackage,
    case: &CaseOperation,
    compiler: Profile,
    limits: Limits,
    cancel: &CancelSource,
    paths: Vec<String>,
) -> Result<super::DeclaredExecution, WorkflowError> {
    let mut execution = declared(package, case, compiler, limits, cancel).await?;
    execution.analysis.bindings.demand.extend(paths);
    execution.analysis.bindings.demand.sort();
    execution.analysis.bindings.demand.dedup();
    execution.model = package
        .prepare(
            execution.analysis.root,
            execution.analysis.instance,
            execution.analysis.bindings.clone(),
            execution.analysis.limits,
            cancel,
        )
        .await?;
    Ok(execution)
}
fn refusal(
    rule: DiagnosticRule,
    class: BoundaryClass,
    sources: impl IntoIterator<Item = SemanticId>,
    reason: &str,
) -> WorkflowError {
    let mut diagnostic =
        BoundaryDiagnostic::new(class, DiagnosticStage::StudyAdmission, sources, rule);
    diagnostic
        .observations
        .insert("reason".into(), Observation::Text(reason.to_owned()));
    WorkflowError::Boundary(Box::new(diagnostic))
}

#[cfg(test)]
mod study_operation_unit {
    use super::*;

    fn operation(operation: OperationRequest) -> StudyOperation {
        StudyOperation {
            version: Version,
            source: OperationSource {
                revision: ContentHash::from_bytes([3; 32]).into(),
                physical_context: ContentHash::from_bytes([4; 32]),
            },
            preparation: PreparationSettings::default(),
            operation,
            admitted_horizon: None,
        }
    }
    fn case() -> CaseOperation {
        CaseOperation {
            case: DeclarationId::from_bytes([5; 16]),
            route: ModelingAnalysisRoute::Steady,
            settings: SolveSettings::default(),
        }
    }

    #[test]
    fn preparation_document_preserves_all_existing_controls() {
        let settings = PreparationSettings {
            compiler: Profile {
                optimization: pse_math::library::Optimization {
                    cores: 2,
                    horner_iterations: 31,
                    cpe_iterations: 9,
                },
                evaluation: pse_math::jets::EvaluationLimits {
                    derivative_components: 7,
                    operations: 13,
                    scratch_bytes: 19,
                    provider_calls: 23,
                },
            },
            limits: Limits {
                depth: 17,
                items: 29,
                members: 43,
                body_occurrences: Some(71),
                body_slots: Some(97),
            },
        };
        let bytes = serde_json::to_vec(&settings).unwrap();
        let decoded: PreparationSettings = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded.compiler, settings.compiler);
        assert_eq!(decoded.limits, settings.limits);
    }

    #[test]
    fn mixed_operation_descriptors_preserve_kind_source_and_existing_profiles() {
        let requests = [
            OperationRequest::DeclaredCase(case()),
            OperationRequest::Simulation(SimulationOperation {
                case: case().case,
                profile: Some(SimulationProfile::default()),
            }),
            OperationRequest::Fit(FitOperation {
                fit: FitId::from_bytes([7; 16]),
                settings: FitOperationSettings {
                    solver: SolveSettings::default(),
                    simulations: BTreeMap::new(),
                    rank_tolerance: 1e-6,
                    max_cells: 12_345,
                    derivatives: FitDerivatives::Responses,
                    uncertainty: None,
                },
            }),
        ];
        for request in requests {
            let original = operation(request);
            let value = serde_json::to_value(&original).unwrap();
            let decoded: StudyOperation = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(decoded.source, original.source);
            assert_eq!(decoded.output_roles(), original.output_roles());
            assert_eq!(serde_json::to_value(decoded).unwrap(), value);
        }
    }

    #[test]
    fn operation_role_admission_refuses_cross_kind_seeds_before_dispatch() {
        let solve = operation(OperationRequest::DeclaredCase(case()));
        assert!(solve.admit_seed_role(SeedRole::PrimalSolution).is_ok());
        let error = solve.admit_seed_role(SeedRole::Trajectory).unwrap_err();
        assert_eq!(
            error.boundary_diagnostic().rule,
            DiagnosticRule::StudySeedIncompatible
        );
        let simulation = operation(OperationRequest::Simulation(SimulationOperation {
            case: case().case,
            profile: None,
        }));
        assert!(
            simulation
                .admit_seed_role(SeedRole::PrimalSolution)
                .is_err()
        );
        assert!(simulation.admit_seed_role(SeedRole::Trajectory).is_err());
    }

    #[test]
    fn horizon_quantities_unit_rejects_bare_coordinates_and_preserves_full_meaning() {
        let id = SemanticId::from_bytes([8; 16]);
        let value = super::super::BindingQuantity {
            magnitude: pse_model::scalars::FiniteBound::try_new(12.0).unwrap(),
            quantity: id,
            unit: SemanticId::from_bytes([9; 16]),
        };
        let input = HorizonInputDocument {
            parameter: id,
            initial: value.clone(),
        };
        let json = serde_json::to_value(&input).unwrap();
        let decoded: HorizonInputDocument = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(decoded.initial, value);
        let mut bare = json;
        bare["initial"] = serde_json::json!(12.0);
        assert!(serde_json::from_value::<HorizonInputDocument>(bare).is_err());
        assert!(
            serde_json::from_value::<ArrivalDocument>(
                serde_json::json!({"prior":"p","next":"x","initial":12.0})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<HorizonSignalDocument>(
                serde_json::json!({"kind":"trajectory","value":[12.0]})
            )
            .is_err()
        );
    }

    #[test]
    fn horizon_signal_and_window_roles_reject_unknown_vocabulary() {
        let bindings = vec![
            HorizonBinding {
                target: "temperature".into(),
                signal: HorizonSignalDocument::Measured(SemanticId::from_bytes([1; 16])),
            },
            HorizonBinding {
                target: "setpoint".into(),
                signal: HorizonSignalDocument::Trajectory(vec![super::super::BindingQuantity {
                    magnitude: pse_model::scalars::FiniteBound::try_new(300.0).unwrap(),
                    quantity: SemanticId::from_bytes([2; 16]),
                    unit: SemanticId::from_bytes([3; 16]),
                }]),
            },
        ];
        let bytes = serde_json::to_vec(&bindings).unwrap();
        let decoded: Vec<HorizonBinding> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded[0].signal, bindings[0].signal);
        assert_eq!(decoded[1].signal, bindings[1].signal);
        assert!(
            serde_json::from_str::<HorizonBinding>(
                r#"{"target":"x","signal":{"kind":"implicit_first_result","value":0}}"#
            )
            .is_err()
        );
        let input = EstimatorInput {
            input: 2,
            binding: WindowInput::Periods(vec!["u[0]".into(), "u[1]".into()]),
        };
        let decoded: EstimatorInput =
            serde_json::from_slice(&serde_json::to_vec(&input).unwrap()).unwrap();
        assert_eq!(decoded.binding, input.binding);
    }
}

#[cfg(all(test, feature = "native-solvers"))]
#[path = "study_operation_admission_tests.rs"]
mod study_operation_admission_tests;
