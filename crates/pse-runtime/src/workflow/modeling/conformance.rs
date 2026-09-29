// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Data-authored fixtures acquire the same bounded checks without model-specific test code.
use super::cases::ModelingCaseResolution;
use super::*;
use crate::math::{
    MathRuntimeError,
    solves::{NumericalInputs, SolverProfile},
};
use pse_compiler::workspace::{ModelingCaseBindings, ModelingHint, ModelingOutput, Profile};
use pse_kernels::DerivativeOrder;
use pse_model::generated::enums::{
    ExtrapolationPolicy, ModelingAnalysisRoute as Route, ModelingCheckKind,
    ModelingConformanceKind as Kind, ModelingConformanceStatus as Status,
    ModelingDeclarationKind as DeclarationKind, ModelingFixtureExecution as Execution,
};
use pse_model::generated::identities::RunId;
pub use pse_model::generated::runtime::modeling_conformance::Row as ModelingConformanceCheck;
use pse_modeling::annotation::AnnotationValue;
use std::{collections::BTreeSet, sync::Arc};

/// An explicit fixture-local execution policy; scientific data remains unchanged.
#[derive(Clone, Debug, Default)]
pub struct ModelingFixturePolicy {
    /// Solver profile for this fixture instead of the run's.
    pub solver: Option<SolverProfile>,
    /// Derivative sampling policy for this fixture instead of the run's.
    pub derivatives: Option<pse_backend_native::derivative_diagnostics::Policy>,
}

/// The solve intent an authored test fixture declares, if any (ADR-0119 Outcome 1).
fn authored_intent(row: &Declaration) -> Option<pse_backend_native::solve::SolveIntent> {
    row.value
        .scope
        .as_ref()
        .and_then(|s| s.fixture.as_ref())
        .and_then(|f| f.intent)
}
/// The solver profile one fixture runs under (ADR-0119 Outcome 1): its runtime fixture
/// policy's, else the run's, with the intent the fixture declares. A runtime fixture policy
/// that names a different intent is refused; neither takes precedence. A fixture without an
/// intent leaves it to the runtime policy.
fn fixture_solver(
    fixture: DeclarationId,
    authored: Option<pse_backend_native::solve::SolveIntent>,
    local: Option<&SolverProfile>,
    run: &SolverProfile,
) -> Result<SolverProfile, WorkflowError> {
    if let (Some(authored), Some(local)) = (authored, local)
        && local.intent != authored
    {
        return Err(WorkflowError::FixtureIntentConflict {
            fixture,
            authored,
            policy: local.intent,
        });
    }
    let mut solver = local.unwrap_or(run).clone();
    if let Some(intent) = authored {
        solver.intent = intent;
    }
    Ok(solver)
}
/// The fixture of package-level checks that belong to no authored fixture (coverage).
pub(super) const NO_FIXTURE: DeclarationId = DeclarationId::from_id(SemanticId::NIL);
/// Execution policy is supplied independently of scientific fixture and oracle data.
#[derive(Clone, Debug)]
pub struct ModelingConformancePolicy {
    /// Compiler profile for every fixture.
    pub compiler: Profile,
    /// Default solver profile.
    pub solver: SolverProfile,
    /// Numerical policy inputs for every fixture.
    pub numerical: NumericalInputs,
    /// Specialization limits for every fixture.
    pub limits: Limits,
    /// Default derivative sampling policy.
    pub derivatives: pse_backend_native::derivative_diagnostics::Policy,
    /// Per-fixture overrides, by fixture declaration.
    pub fixture_policies: BTreeMap<DeclarationId, ModelingFixturePolicy>,
    /// Maximum fixtures executed, at most 4096.
    pub maximum_fixtures: usize,
    /// Maximum checks recorded, at most 100 000; further checks make the report incomplete.
    pub maximum_checks: usize,
}
/// Shared check outcomes retain their pool owner and original solver results.
#[derive(Debug)]
pub struct ModelingConformanceReport {
    /// Run identity of the conformance execution.
    pub run_id: RunId,
    /// Recorded checks, in execution order.
    pub checks: Vec<ModelingConformanceCheck>,
    /// Solve results, by fixture.
    pub results: BTreeMap<DeclarationId, ModelingResult>,
    /// Initialization reports, by fixture.
    pub initializations: BTreeMap<DeclarationId, ModelingInitializationReport>,
    /// Integrated trajectories, by fixture.
    pub trajectories: BTreeMap<DeclarationId, ModelingTrajectory>,
    /// Whether every fixture ran and every check was recorded within the caps.
    pub complete: bool,
    /// Structured causes indexed by each check's failure ordinal.
    pub failures: Vec<pse_model::diagnostic::BoundaryDiagnostic>,
    /// Complete discovery inventory, independent of the detailed check cap.
    pub fixture_statuses: BTreeMap<DeclarationId, Status>,
    pub(super) registry: Arc<pse_schema::Registry>,
    pub(super) pool: Arc<dyn pse_columnar::MemoryPool>,
    _owner: pse_columnar::MemoryReservation,
}
impl ModelingConformanceReport {
    pub(super) fn new(
        registry: Arc<pse_schema::Registry>,
        pool: Arc<dyn pse_columnar::MemoryPool>,
        fixtures: &[DeclarationId],
        cap: usize,
    ) -> Result<Self, WorkflowError> {
        let bytes = cap
            .checked_mul(
                size_of::<ModelingConformanceCheck>()
                    + size_of::<pse_model::diagnostic::BoundaryDiagnostic>(),
            )
            .and_then(|n| {
                fixtures
                    .len()
                    .checked_mul(
                        512 + size_of::<ModelingResult>()
                            + size_of::<ModelingInitializationReport>()
                            + size_of::<ModelingTrajectory>(),
                    )
                    .and_then(|m| n.checked_add(m))
            })
            .ok_or_else(|| contract("conformance report extent"))?;
        let owner =
            pse_columnar::MemoryConsumer::new("modeling:conformance-report").register(&pool);
        owner
            .try_grow(bytes)
            .map_err(pse_columnar::CanonError::from)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        Ok(Self {
            run_id: pse_operations::mint_id(),
            checks: Vec::with_capacity(cap),
            failures: Vec::with_capacity(cap),
            results: BTreeMap::new(),
            initializations: BTreeMap::new(),
            trajectories: BTreeMap::new(),
            fixture_statuses: fixtures
                .iter()
                .map(|id| (*id, Status::Unattempted))
                .collect(),
            complete: true,
            registry,
            pool,
            _owner: owner,
        })
    }
    fn note_status(&mut self, fixture: DeclarationId, status: Status) {
        let rank = |s| match s {
            Status::Unattempted => 0,
            Status::NotApplicable => 1,
            Status::Passed => 2,
            Status::Failed => 3,
            Status::Inconclusive => 4,
            Status::Cancelled => 5,
        };
        if let Some(current) = self.fixture_statuses.get_mut(&fixture)
            && rank(status) > rank(*current)
        {
            *current = status;
        }
    }
    /// Whether the report is complete, nonempty and every check passed or did not apply.
    pub fn passed(&self) -> bool {
        self.complete
            && !self.checks.is_empty()
            && self
                .checks
                .iter()
                .all(|r| matches!(r.status, Status::Passed | Status::NotApplicable))
    }
    /// A check of `fixture` about `target`, attributed to the authored `source`.
    #[expect(
        clippy::too_many_arguments,
        reason = "a check row's attribution (fixture, target, source), classification, message, oracle and cap are independent"
    )]
    pub(super) fn record(
        &mut self,
        fixture: DeclarationId,
        target: SemanticId,
        source: DeclarationId,
        kind: Kind,
        status: Status,
        message: impl AsRef<str>,
        oracle:Option<&pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeOracle>,
        cap: usize,
    ) {
        if self.checks.len() >= cap {
            self.complete = false;
            self.note_status(
                fixture,
                if status == Status::Unattempted {
                    status
                } else {
                    Status::Inconclusive
                },
            );
            return;
        }
        let bytes =
            message.as_ref().len().max(128).checked_add(
                oracle.map_or(0, |o| o.reference.len().saturating_add(o.revision.len())),
            );
        if bytes.is_none_or(|bytes| self._owner.try_grow(bytes).is_err()) {
            self.complete = false;
            self.note_status(
                fixture,
                if status == Status::Unattempted {
                    status
                } else {
                    Status::Inconclusive
                },
            );
            return;
        }
        self.note_status(fixture, status);
        self.checks.push(ModelingConformanceCheck {
            run_id: self.run_id,
            fixture_id: fixture,
            sample_index: 0,
            time: None,
            target_id: target,
            source_id: source,
            kind,
            status,
            message: message.as_ref().to_owned(),
            failure_ordinal: None,
            oracle_reference: oracle.map(|o| o.reference.clone()),
            oracle_revision: oracle.map(|o| o.revision.clone()),
        });
    }
    /// A check about the fixture as a whole: it is its own target and source.
    pub(super) fn record_fixture(
        &mut self,
        fixture: DeclarationId,
        kind: Kind,
        status: Status,
        message: impl AsRef<str>,
        oracle:Option<&pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeOracle>,
        cap: usize,
    ) {
        self.record(
            fixture,
            fixture.as_id(),
            fixture,
            kind,
            status,
            message,
            oracle,
            cap,
        );
    }
    /// All discovered fixtures, including refusals and pure checks without solve results.
    pub fn fixtures(&self) -> BTreeSet<DeclarationId> {
        self.fixture_statuses.keys().copied().collect()
    }
    /// Inventory survives even if no further detailed row can be retained.
    pub fn fixture_statuses_table(
        &self,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_model::generated::runtime::modeling_fixture_status::Row;
        let scratch =
            pse_columnar::MemoryConsumer::new("conformance:inventory-copy").register(&self.pool);
        scratch
            .try_grow(self.fixture_statuses.len() * size_of::<Row>())
            .map_err(pse_columnar::CanonError::from)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        let rows = self
            .fixture_statuses
            .iter()
            .map(|(id, status)| Row {
                run_id: self.run_id,
                fixture_id: *id,
                status: *status,
            })
            .collect::<Vec<_>>();
        self.export(&rows)
    }
    /// The structured failure causes as a checked `modeling_findings` relation.
    pub fn findings_table(
        &self,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_model::HeapUsage;
        let bytes = self
            .failures
            .iter()
            .try_fold(0usize, |n, failure| {
                // JSON escaping in a generated finding row can expand each byte sixfold.
                n.checked_add(failure.owned_bytes().checked_mul(6)?.checked_add(1024)?)
            })
            .ok_or_else(|| contract("conformance finding extent"))?;
        let scratch =
            pse_columnar::MemoryConsumer::new("conformance:findings-copy").register(&self.pool);
        scratch
            .try_grow(bytes)
            .map_err(pse_columnar::CanonError::from)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        let rows = self
            .failures
            .iter()
            .enumerate()
            .map(|(index, failure)| {
                analysis_tables::finding_row(self.run_id, index as i64, failure)
            })
            .collect::<Vec<_>>();
        self.export(&rows)
    }
    fn attach_failure(&mut self, check: usize, failure: pse_model::diagnostic::BoundaryDiagnostic) {
        use pse_model::HeapUsage;
        if let Some(row) = self.checks.get_mut(check) {
            if self._owner.try_grow(failure.heap_bytes()).is_err() {
                self.complete = false;
                row.status = Status::Inconclusive;
                row.message.clear();
                row.message
                    .push_str("diagnostic retention exhausted the memory budget");
                let fixture = row.fixture_id;
                self.note_status(fixture, Status::Inconclusive);
                return;
            }
            row.failure_ordinal = Some(self.failures.len() as i64);
            self.failures.push(failure);
        }
    }
    pub(super) fn failed(
        &mut self,
        fixture: DeclarationId,
        kind: Kind,
        error: &WorkflowError,
        expected: Option<&pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeFixtureExpectedFailure>,
        oracle: Option<&pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeOracle>,
        cap: usize,
    ) {
        use pse_model::diagnostic::BoundaryClass as C;
        let diagnostic = error.boundary_diagnostic();
        let interrupted = matches!(diagnostic.class, C::Cancelled | C::ResourceLimit);
        if interrupted {
            self.complete = false;
        }
        let status = if diagnostic.class == C::Cancelled {
            Status::Cancelled
        } else if interrupted {
            Status::Inconclusive
        } else if expected.is_some_and(|e| e.class == diagnostic.class && e.rule == diagnostic.rule)
        {
            Status::Passed
        } else {
            Status::Failed
        };
        let check = self.checks.len();
        self.record_fixture(fixture, kind, status, error.to_string(), oracle, cap);
        self.attach_failure(check, diagnostic);
    }
    fn model_checks(
        &mut self,
        fixture: DeclarationId,
        checks: &[ModelingCheck],
        oracle: Option<&pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeOracle>,
        cap: usize,
    ) {
        let mut closure = false;
        let mut expectation = false;
        for check in checks {
            let kind = match check.kind {
                ModelingCheckKind::Closure => {
                    closure = true;
                    Kind::Closure
                }
                ModelingCheckKind::Expectation => {
                    expectation = true;
                    Kind::Expectation
                }
                ModelingCheckKind::Validity => continue,
                _ => Kind::Check,
            };
            let index = self.checks.len();
            self.record(
                fixture,
                check.target_id,
                check.source_id,
                kind,
                if check.satisfied {
                    Status::Passed
                } else {
                    Status::Failed
                },
                format!("value {}; tolerance {:?}", check.value, check.tolerance),
                oracle,
                cap,
            );
            if let Some(row) = self.checks.get_mut(index) {
                row.sample_index = check.sample_index;
                row.time = check.time;
            }
        }
        if !closure {
            self.record_fixture(
                fixture,
                Kind::Closure,
                Status::NotApplicable,
                "no conservation closure obligations",
                oracle,
                cap,
            );
        }
        if !expectation {
            self.record_fixture(
                fixture,
                Kind::Expectation,
                Status::NotApplicable,
                "no authored expectations",
                oracle,
                cap,
            );
        }
    }
    pub(super) fn pure_result(
        &mut self,
        fixture: DeclarationId,
        model: &pse_compiler::workspace::PreparedModeling,
        checked: Result<Vec<pse_compiler::workspace::ModelingExpectationResult>, WorkflowError>,
        row: &Declaration,
        cap: usize,
    ) {
        let data = model
            .model
            .fixtures
            .get(&pse_modeling::specialize::root_instance(fixture));
        let expected = data.map_or(0, |f| f.expected_degrees_of_freedom);
        let expected_failure = row
            .value
            .scope
            .as_ref()
            .and_then(|s| s.fixture.as_ref())
            .and_then(|f| f.expected_failure.as_ref());
        let oracle = row.value.scope.as_ref().and_then(|s| s.oracle.as_ref());
        match checked {
            Ok(checks) => {
                self.record_fixture(
                    fixture,
                    Kind::Preparation,
                    Status::Passed,
                    "pure expressions admitted without solver acquisition",
                    oracle,
                    cap,
                );
                let variables = &model.admitted.case;
                let fixed = data
                    .map(|f| {
                        f.specifications
                            .values()
                            .map(|v| (v.target, v.fixed))
                            .collect::<BTreeMap<_, _>>()
                    })
                    .unwrap_or_default();
                let free = variables
                    .variables()
                    .iter()
                    .filter(|v| !fixed.get(&v.port.id).copied().flatten().unwrap_or(v.fixed))
                    .count();
                let equalities = model
                    .model
                    .equations
                    .iter()
                    .filter(|e| {
                        matches!(
                            e.equation.kind,
                            pse_authoring::dsl::EquationKind::Relation {
                                sense: pse_authoring::dsl::EquationSense::Eq,
                                ..
                            }
                        )
                    })
                    .count();
                let dof = free as i64 - equalities as i64;
                self.record_fixture(fixture, Kind::DegreesOfFreedom,
                            if expected == dof { Status::Passed } else { Status::Failed },
                            format!("pure point evaluation; structural DoF {dof}; expected {expected}; no solve attempted"), oracle, cap);
                for check in checks {
                    self.record(
                        fixture,
                        check.id,
                        check.declaration,
                        Kind::Expectation,
                        if check.passed {
                            Status::Passed
                        } else {
                            Status::Failed
                        },
                        format!(
                            "actual {}; expected {}; absolute {}; relative {}; combined {}",
                            check.actual,
                            check.expected,
                            check.absolute_tolerance,
                            check.relative_tolerance,
                            check.tolerance
                        ),
                        oracle,
                        cap,
                    );
                }
                if expected_failure.is_some() {
                    self.record_fixture(
                        fixture,
                        Kind::Check,
                        Status::Failed,
                        "expected failure was not observed",
                        oracle,
                        cap,
                    );
                }
            }
            Err(error) => self.failed(
                fixture,
                Kind::Preparation,
                &error,
                expected_failure,
                oracle,
                cap,
            ),
        }
    }
    /// The recorded checks as a checked `modeling_conformance` relation.
    pub fn table(&self) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        self.export(&self.checks)
    }
    fn export<T: RelationRow + pse_model::HeapUsage + Clone>(
        &self,
        rows: &[T],
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let cancel = pse_columnar::CancellationToken::new();
        let scratch =
            pse_columnar::MemoryConsumer::new("conformance:row-copy").register(&self.pool);
        scratch
            .try_grow(
                rows.iter()
                    .map(pse_model::HeapUsage::owned_bytes)
                    .max()
                    .unwrap_or(0),
            )
            .map_err(pse_columnar::CanonError::from)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        let mut columns =
            pse_relations::columnar::Collection::new(&self.registry, &self.pool, &cancel);
        columns.ensure::<T>().map_err(relation)?;
        for row in rows {
            columns.push(row.clone()).map_err(relation)?;
        }
        columns
            .finish()
            .map_err(relation)?
            .into_values()
            .next()
            .ok_or_else(|| contract("conformance relation absent"))
    }
}
impl ModelingPackage {
    /// Discover authored tests, bind their typed fixtures, and attach the shared checks.
    /// Missing fixture coverage is reported for every concrete definition in this revision.
    pub async fn conform(
        &self,
        policy: ModelingConformancePolicy,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingConformanceReport, WorkflowError> {
        if policy.maximum_fixtures == 0
            || policy.maximum_fixtures > 4096
            || policy.maximum_checks == 0
            || policy.maximum_checks > 100_000
        {
            return Err(contract("bounded conformance policy"));
        }
        policy
            .derivatives
            .allowance()
            .map_err(MathRuntimeError::from)?;
        let fixtures = self
            .revision
            .declarations()
            .iter()
            .filter(|r| r.value.kind == DeclarationKind::Test)
            .collect::<Vec<_>>();
        for (fixture, local) in &policy.fixture_policies {
            let Some(row) = fixtures.iter().find(|row| row.declaration_id == *fixture) else {
                return Err(contract("conformance policy names an unknown fixture"));
            };
            // A conflicting intent is refused before any fixture runs.
            fixture_solver(
                *fixture,
                authored_intent(row),
                local.solver.as_ref(),
                &policy.solver,
            )?;
            if let Some(derivatives) = local.derivatives {
                derivatives.allowance().map_err(MathRuntimeError::from)?;
            }
        }
        let mut report = ModelingConformanceReport::new(
            self.runtime.registry.clone(),
            self.runtime.shared.pool(),
            &fixtures
                .iter()
                .map(|r| r.declaration_id)
                .collect::<Vec<_>>(),
            policy.maximum_checks,
        )?;
        let mut covered = BTreeSet::new();
        let cap = policy.maximum_checks;
        if fixtures.is_empty() {
            report.record_fixture(
                NO_FIXTURE,
                Kind::Coverage,
                Status::Failed,
                "package contains no authored tests",
                None,
                cap,
            );
        }
        for (index, row) in fixtures.iter().enumerate() {
            if index >= policy.maximum_fixtures || cancel.token().is_cancelled() || !report.complete
            {
                report.complete = false;
                for row in &fixtures[index..] {
                    report.record_fixture(
                        row.declaration_id,
                        Kind::Preparation,
                        Status::Unattempted,
                        "fixture was not attempted",
                        None,
                        cap,
                    );
                }
                break;
            }
            let fixture = row.declaration_id;
            let local = policy.fixture_policies.get(&fixture);
            let policy = ModelingConformancePolicy {
                compiler: policy.compiler,
                solver: fixture_solver(
                    fixture,
                    authored_intent(row),
                    local.and_then(|p| p.solver.as_ref()),
                    &policy.solver,
                )?,
                numerical: policy.numerical.clone(),
                limits: policy.limits,
                derivatives: local
                    .and_then(|p| p.derivatives)
                    .unwrap_or(policy.derivatives),
                fixture_policies: BTreeMap::new(),
                maximum_fixtures: policy.maximum_fixtures,
                maximum_checks: policy.maximum_checks,
            };
            let solve_order = match policy.solver.controls.hessian {
                pse_backend_native::solve::HessianMode::LimitedMemory => DerivativeOrder::First,
                pse_backend_native::solve::HessianMode::Exact
                | pse_backend_native::solve::HessianMode::GaussNewton => DerivativeOrder::Second,
            };

            let oracle = row.value.scope.as_ref().and_then(|s| s.oracle.as_ref());
            let authored = row.value.scope.as_ref().and_then(|s| s.fixture.as_ref());
            let expected_failure = authored.and_then(|f| f.expected_failure.as_ref());
            let execution = authored
                .and_then(|f| f.execution)
                .unwrap_or(Execution::Steady);
            let bindings = Bindings::default().with_analysis(match execution {
                Execution::Integrated | Execution::Shooting => Route::Integrated,
                Execution::Simultaneous => Route::Simultaneous,
                _ => Route::Steady,
            });
            let model = match self
                .prepare(
                    fixture,
                    pse_modeling::specialize::root_instance(fixture),
                    bindings.clone(),
                    policy.limits,
                    cancel,
                )
                .await
            {
                Ok(model) => model,
                Err(error) => {
                    report.failed(
                        fixture,
                        Kind::Preparation,
                        &error,
                        expected_failure,
                        oracle,
                        cap,
                    );
                    continue;
                }
            };
            covered.extend(
                model
                    .compiled()
                    .model
                    .instances
                    .values()
                    .map(|i| i.definition),
            );
            let data = model
                .compiled()
                .model
                .fixtures
                .get(&pse_modeling::specialize::root_instance(fixture));
            let expected = data.map_or(0, |f| f.expected_degrees_of_freedom);
            let case = data.map(ModelingCaseBindings::from).unwrap_or_default();
            if execution == Execution::Pure {
                let checked = self
                    .runtime
                    .shared
                    .math()
                    .modeling_expectations(
                        self.workspace.clone(),
                        self.revision.clone(),
                        fixture,
                        bindings,
                        policy.limits,
                        policy.compiler,
                        cancel,
                    )
                    .await;
                report.pure_result(
                    fixture,
                    model.compiled(),
                    checked.map_err(WorkflowError::from),
                    row,
                    cap,
                );
                continue;
            }
            // A shooting fixture's controls are scheduled inputs held free: until the
            // fixture declares schedules (ADR-0119 Outcome 2) it has nothing to shoot for,
            // and the runtime route (`ModelingSimulation::shooting`) takes them directly.
            if execution == Execution::Shooting {
                report.failed(
                    fixture,
                    Kind::Preparation,
                    &contract(
                        "a shooting fixture needs authored scheduled inputs held free as its controls",
                    ),
                    expected_failure,
                    oracle,
                    cap,
                );
                continue;
            }
            if execution == Execution::Integrated {
                match self
                    .conform_integrated(fixture, &model, data, &policy, cancel)
                    .await
                {
                    Ok(trajectory) => {
                        if let Some(failure) = trajectory.diagnostic() {
                            report.failed(
                                fixture,
                                Kind::StartToSolve,
                                &failure.into(),
                                expected_failure,
                                oracle,
                                cap,
                            );
                            if expected_failure.is_none() {
                                report.model_checks(fixture, &trajectory.checks, oracle, cap);
                            }
                            report.trajectories.insert(fixture, trajectory);
                            continue;
                        }
                        report.record_fixture(fixture, Kind::DegreesOfFreedom,
                            if expected == 0 { Status::Passed } else { Status::Failed },
                            "native admission verified square dynamics with an initial condition per differential state; DoF 0", oracle, cap);
                        report.record_fixture(
                            fixture,
                            Kind::StartToSolve,
                            if trajectory.accepted && expected_failure.is_none() {
                                Status::Passed
                            } else {
                                Status::Failed
                            },
                            "integrated trajectory and sampled original-model checks",
                            oracle,
                            cap,
                        );
                        match trajectory
                            .derivative_samples(
                                policy.derivatives,
                                policy.solver.controls.clone(),
                                cancel,
                            )
                            .await
                        {
                            Ok((samples, _sample_owner)) => {
                                for (sample_index, time, complete, passed, message) in samples {
                                    let index = report.checks.len();
                                    report.record_fixture(
                                        fixture,
                                        Kind::Derivatives,
                                        if !complete {
                                            Status::Inconclusive
                                        } else if passed {
                                            Status::Passed
                                        } else {
                                            Status::Failed
                                        },
                                        format!(
                                            "step={}; relative_tolerance={}; cells={}; {message}",
                                            policy.derivatives.perturbation,
                                            policy.derivatives.relative_tolerance,
                                            policy.derivatives.maximum_cells
                                        ),
                                        oracle,
                                        cap,
                                    );
                                    if let Some(row) = report.checks.get_mut(index) {
                                        row.sample_index = sample_index;
                                        row.time = Some(time);
                                    }
                                }
                            }
                            Err(error) => {
                                report.failed(fixture, Kind::Derivatives, &error, None, oracle, cap)
                            }
                        }
                        for (index, sample) in trajectory.report.samples.iter().enumerate() {
                            let (model, values, providers) = trajectory.sample_context(sample)?;
                            let ranges = model
                                .compiled()
                                .model
                                .annotations
                                .iter()
                                .filter(|a| matches!(a.value, AnnotationValue::Valid { .. }))
                                .collect::<Vec<_>>();
                            if ranges.is_empty() {
                                if index == 0 {
                                    report.record_fixture(
                                        fixture,
                                        Kind::Envelope,
                                        Status::NotApplicable,
                                        "no declared validity envelope",
                                        oracle,
                                        cap,
                                    );
                                }
                                continue;
                            }
                            for range in ranges {
                                let check_index = report.checks.len();
                                let (status, message, failure) = if matches!(
                                    &range.value,
                                    AnnotationValue::Valid {
                                        policy: ExtrapolationPolicy::Extrapolate,
                                        ..
                                    }
                                ) {
                                    (
                                        Status::NotApplicable,
                                        "extrapolation explicitly selected".into(),
                                        None,
                                    )
                                } else {
                                    match self.conformance_envelope(&model, &values, &providers, range, policy.compiler, cancel).await {
                                    Ok(Some(true)) => (Status::Passed, "both outside-envelope trials rejected at this sample".into(), None),
                                    Ok(Some(false)) => (Status::Failed, "outside-envelope trial accepted".into(), None),
                                    Ok(None) => (Status::Inconclusive, "computed validity target requires an independent envelope sample".into(), None),
                                    Err(error) => (Status::Inconclusive, error.to_string(), Some(error.boundary_diagnostic())),
                                }
                                };
                                report.record(
                                    fixture,
                                    range.target,
                                    range.lineage.declaration,
                                    Kind::Envelope,
                                    status,
                                    message,
                                    oracle,
                                    cap,
                                );
                                if let Some(row) = report.checks.get_mut(check_index) {
                                    row.sample_index = index as i64;
                                    row.time = Some(sample.time);
                                }
                                if let Some(failure) = failure {
                                    report.attach_failure(check_index, failure);
                                }
                            }
                        }
                        report.model_checks(fixture, &trajectory.checks, oracle, cap);
                        report.trajectories.insert(fixture, trajectory);
                    }
                    Err(error) => report.failed(
                        fixture,
                        Kind::StartToSolve,
                        &error,
                        expected_failure,
                        oracle,
                        cap,
                    ),
                }
                continue;
            }
            let mut seed = BTreeMap::new();
            let mut initialized = None;
            if execution == Execution::Initialized {
                let analysis = ModelingAnalysis {
                    root: fixture,
                    instance: pse_modeling::specialize::root_instance(fixture),
                    bindings: bindings.clone(),
                    limits: policy.limits,
                    case: case.clone(),
                    order: solve_order,
                    compiler: policy.compiler,
                    solver: policy.solver.clone(),
                    numerical: policy.numerical.clone(),
                };
                let mut initialization = ModelingInitialization {
                    stages: data.map(|f| f.stages.clone()).unwrap_or_default(),
                    ..ModelingInitialization::default()
                };
                if let Some(policy) = data.and_then(|f| f.initialization.as_ref()) {
                    initialization.homotopy = policy.homotopy;
                    initialization.initial_step = policy.initial_step;
                    initialization.minimum_step = policy.minimum_step;
                    initialization.growth = policy.growth;
                    initialization.maximum_attempts = usize::try_from(policy.maximum_attempts)
                        .map_err(|_| contract("initialization attempt extent"))?;
                    initialization.time_limit =
                        std::time::Duration::try_from_secs_f64(policy.time_limit_seconds)
                            .map_err(|_| contract("initialization time extent"))?;
                }
                match self
                    .initialize_model(&analysis, initialization, cancel)
                    .await
                {
                    Ok(initialization) => {
                        if initialization.completed {
                            seed = initialization.committed.clone().unwrap_or_default();
                            initialized = initialization
                                .attempts
                                .last()
                                .and_then(|a| a.result.as_ref().ok())
                                .cloned();
                        }
                        let failure = if initialization.completed {
                            None
                        } else {
                            initialization.failure.clone().or_else(|| {
                                Some(pse_model::diagnostic::BoundaryDiagnostic::new(
                                    pse_model::diagnostic::BoundaryClass::ResourceLimit,
                                    "initialization",
                                    [fixture.as_id()],
                                    "modeling.initialization.incomplete",
                                ))
                            })
                        };
                        report.initializations.insert(fixture, initialization);
                        if let Some(failure) = failure {
                            report.failed(
                                fixture,
                                Kind::StartToSolve,
                                &failure.into(),
                                expected_failure,
                                oracle,
                                cap,
                            );
                            continue;
                        }
                    }
                    Err(error) => {
                        report.failed(
                            fixture,
                            Kind::StartToSolve,
                            &error,
                            expected_failure,
                            oracle,
                            cap,
                        );
                        continue;
                    }
                }
            }
            let resolution = match self
                .resolve_case(
                    fixture,
                    pse_modeling::specialize::root_instance(fixture),
                    bindings,
                    policy.limits,
                    case,
                    solve_order,
                    policy.compiler,
                    policy.solver.clone(),
                    policy.numerical.clone(),
                    seed,
                    BTreeMap::new(),
                    false,
                    cancel,
                )
                .await
            {
                Ok(value) => value,
                Err(error) => {
                    report.failed(
                        fixture,
                        Kind::Preparation,
                        &error,
                        expected_failure,
                        oracle,
                        cap,
                    );
                    continue;
                }
            };
            report.record_fixture(
                fixture,
                Kind::Preparation,
                Status::Passed,
                "typed fixture and numerical sources resolved",
                oracle,
                cap,
            );
            let structure = resolution.model.case.compiled().plan.structure();
            let free = resolution.model.case.compiled().plan.columns().len();
            let equalities = structure
                .rows()
                .iter()
                .filter(|r| r.lower == r.upper)
                .count();
            let dof = i64::try_from(free).map_err(|_| contract("DoF variable extent"))?
                - i64::try_from(equalities).map_err(|_| contract("DoF equality extent"))?;
            report.record_fixture(
                fixture,
                Kind::DegreesOfFreedom,
                if dof == expected {
                    Status::Passed
                } else {
                    Status::Failed
                },
                format!(
                    "free variables {free}; equalities {equalities}; DoF {dof}; expected {expected}"
                ),
                oracle,
                cap,
            );
            if free == 0 {
                report.record_fixture(
                    fixture,
                    Kind::Derivatives,
                    Status::NotApplicable,
                    "no free continuous coordinates",
                    oracle,
                    cap,
                );
            } else if structure
                .variables()
                .iter()
                .any(|v| !v.fixed && v.domain.is_discrete())
            {
                // Sampling perturbs a continuous oracle; integrality is never relaxed
                // implicitly (ADR-0103), so free discrete columns leave it unsampled.
                report.record_fixture(
                    fixture,
                    Kind::Derivatives,
                    Status::NotApplicable,
                    "free discrete coordinates; derivative sampling needs a fixed assignment",
                    oracle,
                    cap,
                );
            } else {
                match self
                    .conformance_derivatives(&resolution, policy.derivatives, cancel)
                    .await
                {
                    Ok((complete, passed, message)) => {
                        let status = if !complete {
                            Status::Inconclusive
                        } else if passed {
                            Status::Passed
                        } else {
                            Status::Failed
                        };
                        report.record_fixture(
                            fixture,
                            Kind::Derivatives,
                            status,
                            message,
                            oracle,
                            cap,
                        );
                    }
                    Err(error) => {
                        report.failed(fixture, Kind::Derivatives, &error, None, oracle, cap)
                    }
                }
            }
            let ranges = resolution
                .model
                .model
                .compiled()
                .model
                .annotations
                .iter()
                .filter(|a| matches!(a.value, AnnotationValue::Valid { .. }))
                .cloned()
                .collect::<Vec<_>>();
            if ranges.is_empty() {
                report.record_fixture(
                    fixture,
                    Kind::Envelope,
                    Status::NotApplicable,
                    "no declared validity envelope",
                    oracle,
                    cap,
                );
            }
            for range in ranges {
                if report.checks.len() >= cap {
                    report.complete = false;
                    break;
                }
                if matches!(
                    &range.value,
                    AnnotationValue::Valid {
                        policy: ExtrapolationPolicy::Extrapolate,
                        ..
                    }
                ) {
                    report.record(fixture,range.target,range.lineage.declaration,Kind::Envelope,Status::NotApplicable,"extrapolation explicitly selected; validity membership is reported separately",oracle,cap);
                    continue;
                }
                let tested = self
                    .conformance_envelope(
                        &resolution.model.model,
                        &resolution.model.values,
                        &resolution.providers,
                        &range,
                        policy.compiler,
                        cancel,
                    )
                    .await;
                let (status, message, failure) = match tested {
                    Ok(Some(true)) => (
                        Status::Passed,
                        "both out-of-envelope trials rejected by the declared guard".into(),
                        None,
                    ),
                    Ok(Some(false)) => (
                        Status::Failed,
                        "an out-of-envelope trial was accepted".into(),
                        None,
                    ),
                    Ok(None) => (
                        Status::Inconclusive,
                        "computed validity target requires a supplied independent envelope sample"
                            .into(),
                        None,
                    ),
                    Err(error) => (
                        Status::Inconclusive,
                        error.to_string(),
                        Some(error.boundary_diagnostic()),
                    ),
                };
                let check = report.checks.len();
                report.record(
                    fixture,
                    range.target,
                    range.lineage.declaration,
                    Kind::Envelope,
                    status,
                    message,
                    oracle,
                    cap,
                );
                if let Some(failure) = failure {
                    report.attach_failure(check, failure);
                }
            }
            if cancel.token().is_cancelled() {
                report.complete = false;
                break;
            }
            let solved = if let Some(result) = initialized {
                Ok(result)
            } else {
                match self.finish_case(resolution).await {
                    Ok(prepared) => self.solve_case(prepared, policy.compiler, cancel).await,
                    Err(error) => Err(error),
                }
            };
            match solved {
                Ok(result) => {
                    if let Some(failure) = result.diagnostic() {
                        report.failed(
                            fixture,
                            Kind::StartToSolve,
                            &failure.into(),
                            expected_failure,
                            oracle,
                            cap,
                        );
                        if expected_failure.is_none() {
                            report.model_checks(fixture, &result.checks, oracle, cap);
                        }
                        report.results.insert(fixture, result);
                        continue;
                    }
                    report.record_fixture(
                        fixture,
                        Kind::StartToSolve,
                        if result.accepted {
                            Status::Passed
                        } else {
                            Status::Failed
                        },
                        result
                            .validation_error
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| {
                                if result.accepted {
                                    "original model accepted".into()
                                } else {
                                    "solve or independent model qualification failed".into()
                                }
                            }),
                        oracle,
                        cap,
                    );
                    report.model_checks(fixture, &result.checks, oracle, cap);
                    if expected_failure.is_some() {
                        report.record_fixture(
                            fixture,
                            Kind::Check,
                            Status::Failed,
                            "expected failure was not observed",
                            oracle,
                            cap,
                        );
                    }
                    report.results.insert(fixture, result);
                }
                Err(error) => report.failed(
                    fixture,
                    Kind::StartToSolve,
                    &error,
                    expected_failure,
                    oracle,
                    cap,
                ),
            }
        }
        for row in self
            .revision
            .declarations()
            .iter()
            .filter(|r| r.value.kind == DeclarationKind::Definition)
        {
            report.record(
                NO_FIXTURE,
                row.declaration_id.as_id(),
                row.declaration_id,
                Kind::Coverage,
                if covered.contains(&row.declaration_id) {
                    Status::Passed
                } else {
                    Status::Failed
                },
                if covered.contains(&row.declaration_id) {
                    "definition instantiated by an authored fixture"
                } else {
                    "definition lacks a concrete authored fixture"
                },
                None,
                cap,
            );
        }
        if cancel.token().is_cancelled() {
            report.complete = false;
        }
        Ok(report)
    }
    async fn conform_integrated(
        &self,
        fixture: DeclarationId,
        model: &ModelingPreparation,
        data: Option<&pse_modeling::specialize::Fixture>,
        policy: &ModelingConformancePolicy,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingTrajectory, WorkflowError> {
        let data = data.ok_or_else(|| contract("integrated fixture data absent"))?;
        let profile = self.integration_profile(model, data, &policy.solver.numerics)?;
        self.declared_simulation(
            fixture,
            policy.compiler,
            Some(profile),
            policy.limits,
            cancel,
        )
        .await?
        .run(cancel)
        .await
    }
    async fn conformance_derivatives(
        &self,
        resolution: &ModelingCaseResolution,
        policy: pse_backend_native::derivative_diagnostics::Policy,
        cancel: &crate::CancelSource,
    ) -> Result<(bool, bool, String), WorkflowError> {
        let service = self.runtime.shared.math();
        let owner = service.reserve(
            "modeling:derivative-sample",
            policy.allowance().map_err(MathRuntimeError::from)?,
        )?;
        let plan = &resolution.model.case.compiled().plan;
        let mut targets = plan
            .numerical_targets(&self.quantities)
            .map_err(MathRuntimeError::from)?;
        targets.extend(resolution.numerical.targets.clone());
        let numerics = pse_math::numerics::resolve(
            &self.quantities,
            &targets,
            &resolution.numerical.declarations,
            &resolution.solver.numerics,
        )
        .map_err(MathRuntimeError::from)?;
        let rows = plan
            .structure()
            .rows()
            .iter()
            .map(|r| r.id)
            .collect::<Vec<_>>();
        let normalization =
            pse_math::normalization::Normalization::from_policy(&numerics, plan.columns(), &rows)
                .map_err(MathRuntimeError::from)?;
        let values = resolution.model.values.clone();
        let initial = plan.columns().iter().map(|id| values.scalars[id]).collect();
        let controls = resolution.solver.controls.clone();
        let assembly = service.assemble(resolution.model.case.clone()).await?;
        Ok(service
            .with_owned_worker(
                assembly,
                resolution.providers.clone(),
                cancel,
                move |worker| {
                    let execution = pse_backend_native::solve::Execution::new(
                        worker.cancellation().clone(),
                        &controls,
                    );
                    let oracle =
                        pse_backend_native::assembled::AlgebraicOracle::new(worker, values)?;
                    let result = pse_backend_native::derivative_diagnostics::analyze(
                        Box::new(oracle),
                        initial,
                        normalization,
                        policy,
                        execution,
                    )?;
                    let summary = format!(
                        "step={}; relative_tolerance={}; cells={}; {}",
                        policy.perturbation,
                        policy.relative_tolerance,
                        policy.maximum_cells,
                        result.summary()
                    );
                    let status = (result.complete, result.passed(), summary);
                    drop(result);
                    drop(owner);
                    Ok(status)
                },
            )
            .await?)
    }
    async fn conformance_envelope(
        &self,
        model: &ModelingPreparation,
        values: &pse_math::binding::CaseValues,
        providers: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        range: &pse_modeling::annotation::Annotation,
        compiler: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Option<bool>, WorkflowError> {
        let AnnotationValue::Valid { policy, .. } = &range.value else {
            return Err(contract("validity sample contract"));
        };
        if *policy == ExtrapolationPolicy::Extrapolate {
            return Ok(None);
        }
        if !values.scalars.contains_key(&range.target) {
            return Ok(None);
        }
        let source = range.lineage.declaration;
        let low = ModelingOutput::Hint {
            target: range.target,
            declaration: source,
            kind: ModelingHint::ValidLower,
        }
        .row_id();
        let high = ModelingOutput::Hint {
            target: range.target,
            declaration: source,
            kind: ModelingHint::ValidUpper,
        }
        .row_id();
        let observed = self
            .observe_registered(
                model.clone(),
                BTreeSet::from([low, high]),
                values.clone(),
                compiler,
                providers.clone(),
                cancel,
            )
            .await?;
        let lo = observed[&low];
        let hi = observed[&high];
        if !lo.is_finite() || !hi.is_finite() || lo > hi {
            return Err(contract("invalid envelope endpoints"));
        }
        let trials = [lo.next_down(), hi.next_up()];
        if trials.iter().any(|v| !v.is_finite()) {
            return Err(contract("finite outside-envelope sample unavailable"));
        }
        for trial in trials {
            let mut values = values.clone();
            values.scalars.insert(range.target, trial);
            match self
                .observe_registered(
                    model.clone(),
                    BTreeSet::from([ModelingOutput::Member(range.target).row_id()]),
                    values,
                    compiler,
                    providers.clone(),
                    cancel,
                )
                .await
            {
                Ok(_) => return Ok(Some(false)),
                Err(error) if range_rejected(&error, source.as_id()) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(Some(true))
    }
}
fn range_rejected(error: &WorkflowError, source: SemanticId) -> bool {
    fn math(e: &pse_math::MathError, s: SemanticId) -> bool {
        match e {
            pse_math::MathError::Domain { source_id, .. }
            | pse_math::MathError::OutsideRange { source_id, .. } => *source_id == s,
            pse_math::MathError::Instance { cause, .. } => math(cause, s),
            _ => false,
        }
    }
    fn runtime(e: &MathRuntimeError, s: SemanticId) -> bool {
        match e {
            MathRuntimeError::Math(e) => math(e, s),
            MathRuntimeError::Solve(pse_backend_native::ProblemError::Math(e)) => math(e, s),
            MathRuntimeError::Compile(pse_compiler::workspace::CompileError::Math(e)) => math(e, s),
            MathRuntimeError::Shared(e) => runtime(e, s),
            _ => false,
        }
    }
    match error {
        WorkflowError::Math(e) => runtime(e, source),
        WorkflowError::Shared(e) => range_rejected(e, source),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kernel_conformance_retains_inventory_and_accounts_variable_payloads() {
        use pse_columnar::MemoryPool;
        use pse_model::{
            HeapUsage,
            diagnostic::{BoundaryClass, BoundaryDiagnostic, Observation},
        };
        let pool = Arc::new(pse_columnar::GreedyMemoryPool::new(1 << 20));
        let ids = [
            DeclarationId::from_bytes([1; 16]),
            DeclarationId::from_bytes([2; 16]),
        ];
        let mut report = ModelingConformanceReport::new(
            pse_schema::shared_registry().unwrap(),
            pool.clone(),
            &ids,
            1,
        )
        .unwrap();
        let base = pool.reserved();
        let oracle = pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeOracle {
            reference: "source".repeat(10000), revision: "revision".into(),
        };
        let message = "derivative comparison\n".repeat(500);
        report.record_fixture(
            ids[0],
            Kind::Preparation,
            Status::Failed,
            &message,
            Some(&oracle),
            1,
        );
        assert!(pool.reserved() >= base + oracle.reference.len() + message.len());
        assert_eq!(report.checks[0].message, message);
        assert_eq!(
            report.checks[0].oracle_reference.as_deref(),
            Some(oracle.reference.as_str())
        );
        let mut failure = BoundaryDiagnostic::new(
            BoundaryClass::InvalidModel,
            "test",
            ids.map(DeclarationId::as_id),
            "synthetic",
        );
        failure
            .observations
            .insert("detail".into(), Observation::Text("x".repeat(100000)));
        let before = pool.reserved();
        let extent = failure.heap_bytes();
        report.attach_failure(0, failure);
        assert_eq!(pool.reserved(), before + extent);
        report.record_fixture(
            ids[0],
            Kind::Expectation,
            Status::Passed,
            "past cap",
            None,
            1,
        );
        report.record_fixture(
            ids[1],
            Kind::Preparation,
            Status::Unattempted,
            "unattempted",
            None,
            1,
        );
        assert!(!report.complete);
        assert_eq!(report.checks.len(), 1);
        assert_eq!(report.fixture_statuses[&ids[0]], Status::Inconclusive);
        assert_eq!(report.fixture_statuses[&ids[1]], Status::Unattempted);
        assert_eq!(report.fixtures().len(), 2);
        let inventory = report.fixture_statuses_table().unwrap();
        drop(report);
        assert_eq!(inventory.batch().num_rows(), 2);
        drop(inventory);
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn kernel_conformance_memory_refusal_cannot_erase_or_pass_a_fixture() {
        use pse_columnar::MemoryPool;
        use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic, Observation};
        let pool = Arc::new(pse_columnar::GreedyMemoryPool::new(32000));
        let id = DeclarationId::from_bytes([1; 16]);
        let mut report = ModelingConformanceReport::new(
            pse_schema::shared_registry().unwrap(),
            pool.clone(),
            &[id],
            1,
        )
        .unwrap();
        report.record_fixture(
            id,
            Kind::Preparation,
            Status::Passed,
            "expected failure",
            None,
            1,
        );
        let mut failure = BoundaryDiagnostic::new(
            BoundaryClass::InvalidModel,
            "test",
            [id.as_id()],
            "synthetic",
        );
        failure
            .observations
            .insert("detail".into(), Observation::Text("x".repeat(100000)));
        report.attach_failure(0, failure);
        assert!(!report.passed());
        assert!(!report.complete);
        assert_eq!(report.fixture_statuses[&id], Status::Inconclusive);
        assert_eq!(report.checks[0].status, Status::Inconclusive);
        assert!(report.failures.is_empty());
        assert!(pool.reserved() < 32000);
        drop(report);
        assert_eq!(pool.reserved(), 0);
    }
    fn package(text: &str) -> ModelingPackage {
        let runtime = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let names = BTreeMap::from([(
            "Scalar".into(),
            physical.quantities.neutral_dimensionless().unwrap(),
        )]);
        let rows = pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        runtime.modeling_package(rows, physical, names).unwrap()
    }
    fn policy() -> ModelingConformancePolicy {
        ModelingConformancePolicy {
            compiler: super::super::super::tests::compiler_profile(),
            solver: super::super::super::tests::profile(),
            numerical: NumericalInputs::default(),
            limits: Limits::default(),
            derivatives: pse_backend_native::derivative_diagnostics::Policy {
                perturbation: 1e-6,
                relative_tolerance: 1e-4,
                maximum_cells: 100,
            },
            fixture_policies: BTreeMap::new(),
            maximum_fixtures: 10,
            maximum_checks: 50,
        }
    }
    #[tokio::test]
    async fn kernel_conformance_discovers_pure_tests_and_reports_uncovered_definitions() {
        let p = package(
            "package p { fn cube(x:Scalar)->Scalar=x*x*x; test pure source \"analytic:cube\" revision \"v1\" { expect cube(2)==8 tolerance 1e-8; } def Missing { var x:Scalar; eq e:x==1; } }",
        );
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!report.passed());
        assert!(report.complete);
        assert!(report.checks.iter().any(|c| c.kind == Kind::Expectation
            && c.status == Status::Passed
            && c.oracle_reference.as_deref() == Some("analytic:cube")));
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.kind == Kind::Coverage && c.status == Status::Failed)
        );
        let table = report.table().unwrap();
        let expected = report.checks.clone();
        drop(report);
        drop(p);
        assert_eq!(ModelingConformanceCheck::rows(&table).unwrap(), expected);
        let p = package("package p { test pure { expect 1==1 tolerance 1e-8; } }");
        let mut limited = policy();
        limited.maximum_checks = 1;
        let result = p
            .conform(limited, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!result.complete);
        assert!(!result.passed());
    }
    #[test]
    fn kernel_conformance_refuses_mismatched_or_unbounded_fixture_policies() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        for metadata in [
            "run steady; stages(warm);",
            "run initialized; initialize homotopy(true) step(0) minimum(1e-6) growth(2) attempts(4) seconds(30);",
            "run initialized; initialize homotopy(true) step(0.5) minimum(0.6) growth(2) attempts(4) seconds(30);",
            "run initialized; initialize homotopy(true) step(0.5) minimum(1e-6) growth(1) attempts(4) seconds(30);",
            "run integrated;",
        ] {
            let rows = pse_authoring::language::parse(&format!("package p {{test invalid fixture {{dof 0; {metadata}}} {{expect 1==1 tolerance 1e-6;}}}}"), SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, pse_authoring::ParseBudget::default()).unwrap();
            assert!(
                rt.modeling_package(rows, physical.clone(), BTreeMap::new())
                    .is_err(),
                "{metadata}"
            );
        }
    }
    #[tokio::test]
    async fn kernel_conformance_uses_initialized_original_results() {
        let p = package(
            "package p { def D { var x:Scalar; } test initialized fixture {dof 0; run initialized; initialize homotopy(false) step(0.5) minimum(1e-6) growth(2) attempts(2) seconds(30); fix root.x=2;} {child root:D=D(); expect root.x==2 tolerance 1e-8;} }",
        );
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        assert_eq!(report.initializations.len(), 1);
        let initialization = report.initializations.values().next().unwrap();
        assert!(initialization.completed);
        assert_eq!(initialization.attempts.len(), 1);
        assert!(matches!(
            initialization.attempts[0].step,
            ModelingInitializationStep::Original
        ));
        assert_eq!(
            report.results.values().next().unwrap().run_id,
            initialization.attempts[0].result.as_ref().unwrap().run_id
        );
        assert_eq!(
            initialization.findings_table().unwrap().batch().num_rows(),
            0
        );
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn kernel_conformance_initializes_inherited_child_stages() {
        let source = r#"package p {
 interface Base {var x:Scalar; eq e:x==4; annotation start x(1);
   stage "warm" {override eq e:x==2;} }
 interface Derived extends Base {}
 def D:Derived {}
 test initialized fixture {dof 0; run initialized; stages("warm");}
 {child root:D=D; expect root.x==4 tolerance 1e-8;}
 }"#;
        let p = package(source);
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        let initialization = report.initializations.values().next().unwrap();
        assert!(initialization.completed);
        assert_eq!(initialization.attempts.len(), 2);
        assert!(matches!(&initialization.attempts[0].step,
            ModelingInitializationStep::Stage(name) if name == "warm"));
        assert!(matches!(
            initialization.attempts[1].step,
            ModelingInitializationStep::Original
        ));
        let p = package(&source.replace("root.x==4", "root.x==5"));
        let rejected = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!rejected.passed());
        let initialization = rejected.initializations.values().next().unwrap();
        assert!(!initialization.completed);
        assert!(initialization.committed.is_none());
        assert_eq!(initialization.attempts.len(), 2);
        assert!(initialization.attempts[0].accepted());
        assert!(!initialization.attempts[1].accepted());
    }
    #[tokio::test]
    async fn fixture_policy_intent_conflict_refused() {
        use pse_backend_native::solve::SolveIntent as Intent;
        use pse_model::diagnostic::{BoundaryClass, Observation};
        let p = package(
            "package p { def D { var x:Scalar; eq e:x==1; } test certified fixture {dof 0; run steady; intent certify;} {child root:D=D();} test open fixture {dof 0; run steady;} {child root:D=D();} }",
        );
        let fixture = |name: &str| {
            p.declarations()
                .iter()
                .find(|r| r.name == name)
                .unwrap()
                .declaration_id
        };
        let (certified, open) = (fixture("certified"), fixture("open"));
        let rows = p.declarations();
        let row = |id| rows.iter().find(|r| r.declaration_id == id).unwrap();
        assert_eq!(authored_intent(row(certified)), Some(Intent::Certify));
        assert_eq!(authored_intent(row(open)), None);
        // The specialized kernel fixture carries the declared intent.
        let cancel = crate::CancelSource::new();
        for (id, expected) in [(certified, Some(Intent::Certify)), (open, None)] {
            let instance = pse_modeling::specialize::root_instance(id);
            let model = p
                .prepare(id, instance, Bindings::default(), Limits::default(), &cancel)
                .await
                .unwrap();
            assert_eq!(model.compiled().model.fixtures[&instance].intent, expected);
        }
        let run = policy().solver;
        let with = |intent| {
            let mut solver = run.clone();
            solver.intent = intent;
            solver
        };
        let intent = |authored, local: Option<&SolverProfile>| {
            fixture_solver(certified, authored, local, &run).map(|s| s.intent)
        };
        // The declared intent replaces the run's; a runtime fixture policy may repeat it.
        assert_eq!(intent(Some(Intent::Certify), None).unwrap(), Intent::Certify);
        assert_eq!(
            intent(Some(Intent::Certify), Some(&with(Intent::Certify))).unwrap(),
            Intent::Certify
        );
        // Without a declared intent the runtime policy decides.
        assert_eq!(
            intent(None, Some(&with(Intent::Optimize))).unwrap(),
            Intent::Optimize
        );
        assert_eq!(intent(None, None).unwrap(), run.intent);
        // A runtime fixture policy that names another intent is refused, naming both,
        // before any fixture runs: neither takes precedence.
        let mut conflicting = policy();
        conflicting.fixture_policies.insert(
            certified,
            ModelingFixturePolicy {
                solver: Some(with(Intent::Optimize)),
                derivatives: None,
            },
        );
        let error = p.conform(conflicting, &cancel).await.unwrap_err();
        assert!(
            matches!(
                error,
                WorkflowError::FixtureIntentConflict {
                    fixture,
                    authored: Intent::Certify,
                    policy: Intent::Optimize,
                } if fixture == certified
            ),
            "{error}"
        );
        let diagnostic = error.boundary_diagnostic();
        assert_eq!(diagnostic.class, BoundaryClass::Conflict);
        assert_eq!(diagnostic.rule, "workflow.fixture_intent_conflict");
        assert_eq!(diagnostic.sources, vec![certified.as_id()]);
        for (name, spelling) in [("authored", "certify"), ("policy", "optimize")] {
            assert!(
                matches!(diagnostic.observations.get(name), Some(Observation::Text(t)) if t == spelling),
                "{diagnostic:?}"
            );
        }
    }
    #[tokio::test]
    async fn kernel_conformance_refuses_unused_or_invalid_fixture_policy() {
        let p = package(
            "package p {test sample fixture {dof 0; run pure;} {expect 1==1 tolerance 0;}}",
        );
        let fixture = p
            .declarations()
            .iter()
            .find(|r| r.value.kind == DeclarationKind::Test)
            .unwrap()
            .declaration_id;
        let mut policy = policy();
        policy
            .fixture_policies
            .insert(NO_FIXTURE, ModelingFixturePolicy::default());
        assert!(
            p.conform(policy.clone(), &crate::CancelSource::new())
                .await
                .is_err()
        );
        policy.fixture_policies.clear();
        policy.fixture_policies.insert(
            fixture,
            ModelingFixturePolicy {
                derivatives: Some(pse_backend_native::derivative_diagnostics::Policy {
                    perturbation: f64::NAN,
                    relative_tolerance: 1e-4,
                    maximum_cells: 100,
                }),
                ..Default::default()
            },
        );
        assert!(
            p.conform(policy, &crate::CancelSource::new())
                .await
                .is_err()
        );
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn kernel_conformance_mixes_explicit_fixture_solver_and_derivative_policies() {
        let p = package(
            r#"package p {
            test root fixture {dof 0; run steady;} {
                var x:Scalar; eq e:x*x==4; annotation start x(1); annotation bounds x(0.5,3);
                expect x==2 tolerance 1e-6;
            }
            test optimization fixture {dof 1; run steady;} {
                var x:Scalar; let cost:Scalar=(x-3)^2; annotation objective cost(minimize);
                annotation start x(1); expect x==3 tolerance 1e-6;
            }
        }"#,
        );
        let fixture = p
            .declarations()
            .iter()
            .find(|r| r.name == "optimization")
            .unwrap()
            .declaration_id;
        let mut policy = policy();
        policy.solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Ipopt,
        );
        let mut optimize = policy.solver.clone();
        optimize.intent = pse_backend_native::solve::SolveIntent::Optimize;
        policy.fixture_policies.insert(
            fixture,
            ModelingFixturePolicy {
                solver: Some(optimize),
                derivatives: Some(pse_backend_native::derivative_diagnostics::Policy {
                    perturbation: 1e-7,
                    relative_tolerance: 1e-4,
                    maximum_cells: 100,
                }),
            },
        );
        let report = p
            .conform(policy, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        assert_eq!(report.results.len(), 2);
        assert!(report.checks.iter().any(|r| r.fixture_id == fixture
            && r.kind == Kind::Derivatives
            && r.message.contains("step=0.0000001")));
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn kernel_conformance_prepares_the_requested_exact_hessian() {
        let p = package(
            "package p {def D {var x:Scalar; eq e:x*x==4; annotation start x(1); annotation bounds x(0.5,3);} test bounded fixture {dof 0; run steady;} {child root:D=D(); expect root.x==2 tolerance 1e-6;}}",
        );
        let mut policy = policy();
        policy.solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Ipopt,
        );
        policy.solver.controls.hessian = pse_backend_native::solve::HessianMode::Exact;
        let report = p
            .conform(policy, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn kernel_conformance_attaches_shared_checks_to_authored_model_fixtures() {
        let p = package(
            r#"package p {
 def D { var x:Scalar; accumulate balance:Scalar conservation tolerance 1e-6;
   contribute balance role inflow = x*x; contribute balance role outflow = 4;
   annotation start x(1); annotation valid x(0,10,reject); }
 test fixture source "idaes-oracle:synthetic-fixture::0" revision "source-v1" fixture { dof 0; lower root.x = 0; } {
   child root:D=D(); expect root.x==2 tolerance 1e-6;
 }
 }"#,
        );
        let mut policy = policy();
        policy.solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Kinsol,
        );
        let report = p
            .conform(policy, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        for kind in [
            Kind::Coverage,
            Kind::DegreesOfFreedom,
            Kind::Derivatives,
            Kind::Envelope,
            Kind::StartToSolve,
            Kind::Closure,
            Kind::Expectation,
        ] {
            assert!(
                report
                    .checks
                    .iter()
                    .any(|c| c.kind == kind && c.status == Status::Passed),
                "{kind:?}"
            );
        }
    }
    #[tokio::test]
    async fn kernel_conformance_dispatches_pure_checks_and_preserves_expected_failures() {
        let runtime = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let names = BTreeMap::from([(
            "Scalar".into(),
            physical.quantities.neutral_dimensionless().unwrap(),
        )]);
        let source = r#"package p {
            fn square(x:Scalar)->Scalar=x*x;
            test pure fixture { dof 0; run pure; } {
                expect square(3)==9 tolerance 1e-12 relative 1e-6;
            }
            test negative fixture { dof 0; run pure; failure invalid_model "compiler.missing"; } {
                expect 1==1 tolerance 0;
            }
        }"#;
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::from_bytes([93; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let rendered = pse_authoring::language::render(&rows).unwrap();
        let roundtrip = pse_authoring::language::parse(
            &rendered,
            SemanticId::from_bytes([93; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        assert_eq!(
            rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
            roundtrip.iter().map(|r| &r.value).collect::<Vec<_>>()
        );
        let package = runtime.modeling_package(rows, physical, names).unwrap();
        let report = package
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        assert_eq!(report.fixtures().len(), 2);
        assert!(report.results.is_empty());
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].rule, "compiler.missing");
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.failure_ordinal == Some(0) && c.status == Status::Passed)
        );
        let table = report.table().unwrap();
        let findings = report.findings_table().unwrap();
        drop(report);
        assert!(table.batch().num_rows() > 0);
        assert_eq!(findings.batch().num_rows(), 1);
        let cancelled = crate::CancelSource::new();
        cancelled.cancel();
        let report = package.conform(policy(), &cancelled).await.unwrap();
        assert!(!report.complete && !report.passed());
        assert!(
            report
                .checks
                .iter()
                .all(|c| c.status == Status::Unattempted)
        );
    }
    /// The authored price-taker package fixture runs through admission, routing, HiGHS and
    /// the original-model checks; its optimum differs from the linear relaxation (85 W).
    #[cfg(feature = "solver-highs")]
    #[tokio::test]
    async fn authored_milp_routes_to_highs() {
        use crate::math::solves::Outcome;
        use pse_backend_native::solve::{Backend, SolveIntent};
        use pse_model::generated::enums::ModelingVariableDomain as Domain;
        use pse_relations::columnar::RelationRow;
        let text =
            include_str!("../../../../../packages/reference/seed-data/models/price-taker.pse");
        let rows = pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Explicit,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let fixture = rows
            .iter()
            .find(|r| r.name == "price_taker" && r.value.kind == DeclarationKind::Test)
            .unwrap()
            .declaration_id;
        let physical = super::super::super::tests::physical();
        let mut names = super::super::super::tests::discrete_names();
        names.insert(
            "Scalar".into(),
            physical.quantities.neutral_dimensionless().unwrap(),
        );
        let package = super::super::super::tests::runtime()
            .modeling_package(rows, physical, names)
            .unwrap();
        let mut policy = policy();
        policy.solver.intent = SolveIntent::Optimize;
        let report = package
            .conform(policy.clone(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        assert!(
            report
                .checks
                .iter()
                .any(|c| c.kind == Kind::DegreesOfFreedom
                    && c.status == Status::Passed
                    && c.message.contains("free variables 6"))
        );
        let Outcome::Native(native) = &report.results[&fixture].outcome else {
            panic!("expected a native MILP outcome");
        };
        assert_eq!(native.backend, Backend::Highs);
        let evidence = native.evidence.coefficient.as_ref().unwrap();
        assert!(evidence.discrete);
        assert!((evidence.objective.unwrap().abs() - 70.).abs() < 1e-6);
        // The published variable rows state each declared domain.
        let prepared = package
            .prepare_solve(
                fixture,
                pse_modeling::specialize::root_instance(fixture),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::First,
                policy.compiler,
                policy.solver,
                NumericalInputs::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let result = prepared.start().unwrap().wait().await.unwrap();
        let table = result.table("runtime.solve_variables").unwrap();
        let rows = pse_relations::generated::runtime::solve_variables::Row::rows(&table).unwrap();
        let domains = rows
            .iter()
            .filter(|r| !r.parameter)
            .map(|r| r.domain)
            .collect::<Vec<_>>();
        assert_eq!(domains.len(), 6);
        assert_eq!(
            domains
                .iter()
                .filter(|d| **d == Some(Domain::Binary))
                .count(),
            3
        );
        assert!(domains.iter().all(|d| d.is_some()));
        assert!(
            rows.iter()
                .filter(|r| r.parameter)
                .all(|r| r.domain.is_none())
        );
    }
}
