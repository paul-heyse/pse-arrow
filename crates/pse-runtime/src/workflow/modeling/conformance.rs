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
    ModelingCheckKind, ModelingConformanceKind as Kind, ModelingConformanceStatus as Status,
    ModelingDeclarationKind as DeclarationKind,
};
use pse_model::generated::identities::RunId;
pub use pse_model::generated::runtime::modeling_conformance::Row as ModelingConformanceCheck;
use pse_modeling::annotation::AnnotationValue;
use std::{collections::BTreeSet, sync::Arc};

/// The authored fixture data of a test declaration, if any.
fn authored_fixture(
    row: &Declaration,
) -> Option<
    &pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeFixture,
>{
    row.value.scope.as_ref().and_then(|s| s.fixture.as_ref())
}
/// The policy one fixture runs under (ADR-0119): the run's, with the solve intent and each
/// execution-policy setting its declaration states, for this fixture only; a declared
/// foreign allowance becomes its solves' control. The time limit and every other control
/// stay the run's. The fixture's derivative policy is validated
/// here, so a caller that resolves every fixture first refuses before any runs.
fn fixture_policy(
    row: &Declaration,
    run: &ModelingConformancePolicy,
) -> Result<ModelingConformancePolicy, WorkflowError> {
    let fixture = row.declaration_id;
    let authored = authored_fixture(row);
    let solver = declared::declared_solver(row, &run.solver)?;
    let mut derivatives = run.derivatives;
    if let Some(declared) = authored.and_then(|f| f.policy.as_ref()) {
        derivatives.perturbation = declared.derivative_step.unwrap_or(derivatives.perturbation);
        derivatives.relative_tolerance = declared
            .derivative_tolerance
            .unwrap_or(derivatives.relative_tolerance);
        if let Some(cells) = declared.derivative_cells {
            derivatives.maximum_cells = usize::try_from(cells).map_err(|_| {
                contract(format!(
                    "fixture {fixture} policy derivative cell allowance"
                ))
            })?;
        }
    }
    derivatives.allowance().map_err(|error| {
        contract(format!(
            "fixture {fixture} derivative policy: {}",
            MathRuntimeError::from(error)
        ))
    })?;
    if authored.is_some_and(|f| !f.diagnostics.is_empty()) && run.diagnostics.is_none() {
        return Err(contract(format!(
            "fixture {fixture} expects diagnostic findings; the run states no diagnostic thresholds"
        )));
    }
    Ok(ModelingConformancePolicy {
        compiler: run.compiler,
        solver,
        numerical: run.numerical.clone(),
        limits: declared::declared_limits(row, run.limits)?,
        derivatives,
        maximum_fixtures: run.maximum_fixtures,
        maximum_checks: run.maximum_checks,
        fixtures: run.fixtures.clone(),
        diagnostics: run.diagnostics.clone(),
    })
}
/// The fixture of package-level checks that belong to no authored fixture (coverage).
pub(super) const NO_FIXTURE: DeclarationId = DeclarationId::from_id(SemanticId::NIL);
/// The fixtures one conformance run executes: every authored test of the revision, or the
/// tests a caller selects by declaration identity. A selected run executes and inventories
/// only its selection, and assesses no package coverage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ModelingFixtureSelection {
    /// Every authored test; coverage is assessed over every concrete definition.
    #[default]
    Package,
    /// Exactly these test declarations.
    Selected(BTreeSet<DeclarationId>),
}
impl ModelingFixtureSelection {
    /// The selected tests of `declarations`, in declaration order. A selection that is empty
    /// or names a declaration that is no authored test is refused before any fixture runs.
    pub(super) fn tests<'a>(
        &self,
        declarations: &'a [Declaration],
    ) -> Result<Vec<&'a Declaration>, WorkflowError> {
        let tests = declarations
            .iter()
            .filter(|r| r.value.kind == DeclarationKind::Test);
        let Self::Selected(selected) = self else {
            return Ok(tests.collect());
        };
        if selected.is_empty() {
            return Err(contract("a fixture selection names at least one test"));
        }
        let chosen = tests
            .filter(|r| selected.contains(&r.declaration_id))
            .collect::<Vec<_>>();
        if chosen.len() != selected.len() {
            let found = chosen
                .iter()
                .map(|r| r.declaration_id)
                .collect::<BTreeSet<_>>();
            let unknown = selected
                .difference(&found)
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            return Err(contract(format!(
                "fixture selection names no authored test: {}",
                unknown.join(", ")
            )));
        }
        Ok(chosen)
    }
}
/// The run's execution policy, independent of scientific fixture and oracle data. A
/// fixture's declared execution policy replaces a setting for that fixture only (ADR-0119).
#[derive(Clone, Debug)]
pub struct ModelingConformancePolicy {
    /// Compiler profile for every fixture.
    pub compiler: Profile,
    /// Default solver profile.
    pub solver: SolverProfile,
    /// Numerical policy inputs for every fixture.
    pub numerical: NumericalInputs,
    /// Default specialization limits.
    pub limits: Limits,
    /// Default derivative sampling policy.
    pub derivatives: pse_backend_native::derivative_diagnostics::Policy,
    /// Maximum fixtures executed, at most 4096.
    pub maximum_fixtures: usize,
    /// Maximum checks recorded, at most 100 000; further checks make the report incomplete.
    pub maximum_checks: usize,
    /// The fixtures the run executes.
    pub fixtures: ModelingFixtureSelection,
    /// The numerical diagnostic thresholds of the run, a named knowledge profile; a fixture
    /// that expects diagnostic findings needs them.
    pub diagnostics: Option<ModelingDiagnosticPolicy>,
}
/// Shared check outcomes retain their pool owner and original solver results.
#[derive(Debug)]
pub struct ModelingConformanceReport {
    /// Run identity of the conformance execution.
    pub run_id: RunId,
    /// Recorded checks, in execution order.
    pub checks: Vec<ModelingConformanceCheck>,
    /// Demanded pure-point evidence retains complete typed authorization and input lineage.
    pub applicability: BTreeMap<DeclarationId, Vec<pse_model::applicability::Observation>>,
    /// Retained typed route and original structural admission facts, including refusals.
    pub admissions:
        BTreeMap<DeclarationId, BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>>,
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
    /// The fixtures the run executed; a selected run assesses no package coverage.
    pub selection: ModelingFixtureSelection,
    /// The definitions each prepared fixture instantiates directly: its units under test
    /// (Plan 23 H6).
    pub units: BTreeMap<DeclarationId, BTreeSet<DeclarationId>>,
    /// The release each named oracle's values come from, if its kind names one (Plan 23 H6).
    pub releases: BTreeMap<DeclarationId, Option<DeclarationId>>,
    pub(super) registry: Arc<pse_schema::Registry>,
    pub(super) pool: Arc<dyn pse_columnar::MemoryPool>,
    validation: Arc<pse_relations::validate::ValidationContext>,
    _owner: pse_columnar::MemoryReservation,
}
impl ModelingConformanceReport {
    pub(super) fn new(
        registry: Arc<pse_schema::Registry>,
        pool: Arc<dyn pse_columnar::MemoryPool>,
        validation: Arc<pse_relations::validate::ValidationContext>,
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
            applicability: BTreeMap::new(),
            admissions: BTreeMap::new(),
            failures: Vec::with_capacity(cap),
            results: BTreeMap::new(),
            initializations: BTreeMap::new(),
            trajectories: BTreeMap::new(),
            fixture_statuses: fixtures
                .iter()
                .map(|id| (*id, Status::Unattempted))
                .collect(),
            units: BTreeMap::new(),
            releases: BTreeMap::new(),
            complete: true,
            selection: ModelingFixtureSelection::Package,
            registry,
            pool,
            validation,
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
    /// Package coverage: whether an executed fixture instantiated each concrete definition
    /// of `declarations`. A selected run records one row stating that it assessed none, so
    /// it neither claims nor refutes whole-package coverage.
    pub(super) fn coverage(
        &mut self,
        declarations: &[Declaration],
        covered: &BTreeSet<DeclarationId>,
        cap: usize,
    ) {
        if let ModelingFixtureSelection::Selected(selected) = &self.selection {
            let message = format!(
                "a run of {} selected fixtures does not assess package coverage",
                selected.len()
            );
            self.record_fixture(
                NO_FIXTURE,
                Kind::Coverage,
                Status::NotApplicable,
                message,
                None,
                cap,
            );
            return;
        }
        for row in declarations
            .iter()
            .filter(|r| r.value.kind == DeclarationKind::Definition)
        {
            let instantiated = covered.contains(&row.declaration_id);
            self.record(
                NO_FIXTURE,
                row.declaration_id.as_id(),
                row.declaration_id,
                Kind::Coverage,
                if instantiated {
                    Status::Passed
                } else {
                    Status::Failed
                },
                if instantiated {
                    "definition instantiated by an authored fixture"
                } else {
                    "definition lacks a concrete authored fixture"
                },
                None,
                cap,
            );
        }
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
        oracle: Option<DeclarationId>,
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
        if self
            ._owner
            .try_grow(message.as_ref().len().max(128))
            .is_err()
        {
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
            oracle_source_id: oracle,
            deviation: None,
            tolerance: None,
        });
    }
    /// The deviation and combined tolerance of the expectation recorded at `check`
    /// (Plan 23 H6).
    fn compared(&mut self, check: usize, deviation: f64, tolerance: Option<f64>) {
        if let Some(row) = self.checks.get_mut(check) {
            row.deviation = Some(deviation);
            row.tolerance = tolerance;
        }
    }
    /// A check about the fixture as a whole: it is its own target and source.
    pub(super) fn record_fixture(
        &mut self,
        fixture: DeclarationId,
        kind: Kind,
        status: Status,
        message: impl AsRef<str>,
        oracle: Option<DeclarationId>,
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
    /// Note the oracle a fixture names and its release (Plan 23 H6).
    pub(super) fn note_oracle(
        &mut self,
        oracle: Option<DeclarationId>,
        release: impl FnOnce(DeclarationId) -> Option<DeclarationId>,
    ) {
        if let Some(oracle) = oracle
            && !self.releases.contains_key(&oracle)
        {
            self.releases.insert(oracle, release(oracle));
        }
    }
    /// Note the definitions a prepared fixture's root instantiates directly (Plan 23 H6).
    pub(super) fn note_units(
        &mut self,
        fixture: DeclarationId,
        model: &pse_modeling::specialize::SpecializedModel,
    ) {
        let root = pse_modeling::specialize::root_instance(fixture);
        self.units.insert(
            fixture,
            model
                .instances
                .values()
                .filter(|i| i.parent == Some(root))
                .map(|i| i.definition)
                .collect(),
        );
    }
    /// The oracle parity report (Plan 23 H6, CT-S14) as a checked `modeling_parity`
    /// relation: a projection of this run's checks, never a second record of them. Every
    /// check of a fixture naming an oracle that is not merely inapplicable appears once for
    /// each definition the fixture instantiates directly, or once under the fixture itself
    /// when it instantiates none or was not prepared, with the oracle's release, the check's
    /// deviation and tolerance and both dispositions.
    pub fn parity_table(
        &self,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_model::generated::runtime::modeling_parity::Row;
        let fixture_only = BTreeSet::new();
        let checks = || {
            self.checks
                .iter()
                .filter(|c| c.status != Status::NotApplicable)
                .filter_map(|c| c.oracle_source_id.map(|oracle| (c, oracle)))
        };
        let count = checks()
            .map(|(c, _)| self.units.get(&c.fixture_id).map_or(1, |u| u.len().max(1)))
            .sum::<usize>();
        let scratch =
            pse_columnar::MemoryConsumer::new("conformance:parity-copy").register(&self.pool);
        scratch
            .try_grow(count * size_of::<Row>())
            .map_err(pse_columnar::CanonError::from)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        let rows = checks()
            .flat_map(|(c, oracle)| {
                let units = self.units.get(&c.fixture_id).unwrap_or(&fixture_only);
                let definitions = if units.is_empty() {
                    vec![c.fixture_id]
                } else {
                    units.iter().copied().collect()
                };
                definitions.into_iter().map(move |definition| Row {
                    run_id: self.run_id,
                    definition_id: definition,
                    oracle_source_id: oracle,
                    release_id: self.releases.get(&oracle).copied().flatten(),
                    fixture_id: c.fixture_id,
                    sample_index: c.sample_index,
                    target_id: c.target_id,
                    source_id: c.source_id,
                    kind: c.kind,
                    status: c.status,
                    fixture_status: self
                        .fixture_statuses
                        .get(&c.fixture_id)
                        .copied()
                        .unwrap_or(Status::Unattempted),
                    deviation: c.deviation,
                    tolerance: c.tolerance,
                })
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
        expected: Option<&pse_modeling::specialize::ExpectedFailure>,
        oracle: Option<DeclarationId>,
        cap: usize,
    ) {
        use pse_model::diagnostic::BoundaryClass as C;
        let diagnostic = error.boundary_diagnostic();
        let interrupted = matches!(diagnostic.class, C::Cancelled | C::ResourceLimit);
        if interrupted {
            self.complete = false;
        }
        // Plan 23 H5: an expected failure is its class and its lineage, never a class and a
        // rule text alone.
        let status = if diagnostic.class == C::Cancelled {
            Status::Cancelled
        } else if interrupted {
            Status::Inconclusive
        } else if expected.is_some_and(|e| e.matches(&diagnostic)) {
            Status::Passed
        } else {
            Status::Failed
        };
        let check = self.checks.len();
        self.record_fixture(fixture, kind, status, error.to_string(), oracle, cap);
        self.attach_failure(check, diagnostic);
    }
    /// Each expected diagnostic passes when a finding of its rule names every expected
    /// member, one of the member's equations or variables at least; the matching finding is
    /// retained with the check.
    fn expected_diagnostics(
        &mut self,
        fixture: DeclarationId,
        expected: &[pse_modeling::specialize::FixtureDiagnostic],
        diagnosed: Result<ModelingDiagnostics, WorkflowError>,
        oracle: Option<DeclarationId>,
        cap: usize,
    ) {
        let diagnostics = match diagnosed {
            Ok(diagnostics) => diagnostics,
            Err(error) => {
                self.failed(fixture, Kind::Expectation, &error, None, oracle, cap);
                return;
            }
        };
        for expectation in expected {
            let members = expectation
                .members
                .iter()
                .map(|(path, _)| path.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let found = diagnostics.findings.iter().find(|finding| {
                finding.rule == expectation.rule
                    && expectation
                        .members
                        .iter()
                        .all(|(_, ids)| finding.sources.iter().any(|s| ids.contains(s)))
            });
            let check = self.checks.len();
            match found {
                Some(finding) => {
                    let mut named = finding
                        .locations
                        .iter()
                        .map(|l| l.path.as_str())
                        .collect::<Vec<_>>();
                    named.sort_unstable();
                    named.dedup();
                    self.record_fixture(
                        fixture,
                        Kind::Expectation,
                        Status::Passed,
                        format!(
                            "{} names {members} (profile {}; the finding names {})",
                            expectation.rule,
                            diagnostics.profile,
                            named.join(", ")
                        ),
                        oracle,
                        cap,
                    );
                    self.attach_failure(check, finding.clone());
                }
                None => self.record_fixture(
                    fixture,
                    Kind::Expectation,
                    if diagnostics.complete {
                        Status::Failed
                    } else {
                        Status::Inconclusive
                    },
                    format!(
                        "no {} finding names {members} (profile {}; {} findings{})",
                        expectation.rule,
                        diagnostics.profile,
                        diagnostics.findings.len(),
                        if diagnostics.complete {
                            ""
                        } else {
                            "; the analysis is incomplete"
                        }
                    ),
                    oracle,
                    cap,
                ),
            }
        }
    }
    fn model_checks(
        &mut self,
        fixture: DeclarationId,
        checks: &[ModelingCheck],
        oracle: Option<DeclarationId>,
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
            if kind == Kind::Expectation {
                self.compared(index, check.value, check.tolerance);
            }
        }
        if !closure {
            self.record_fixture(
                fixture,
                Kind::Closure,
                Status::NotApplicable,
                "no evaluated conservation closure checks",
                oracle,
                cap,
            );
        }
        if !expectation {
            self.record_fixture(
                fixture,
                Kind::Expectation,
                Status::NotApplicable,
                "no evaluated expectation checks",
                oracle,
                cap,
            );
        }
    }
    pub(super) fn pure_result(
        &mut self,
        fixture: DeclarationId,
        model: &pse_compiler::workspace::PreparedModeling,
        checked: Result<pse_compiler::workspace::ModelingPointChecks, WorkflowError>,
        cap: usize,
    ) {
        let data = model
            .model
            .fixtures
            .get(&pse_modeling::specialize::root_instance(fixture));
        let expected = data.map_or(0, |f| f.expected_degrees_of_freedom);
        let expected_failure = data.and_then(|f| f.expected_failure.as_ref());
        let oracle = data.and_then(|f| f.oracle);
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
                let variables = &model.admitted.case();
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
                self.point_validity(fixture, &checks.validity, oracle, cap);
                let bytes = checks.applicability.capacity()
                    * size_of::<pse_model::applicability::Observation>()
                    + checks
                        .applicability
                        .iter()
                        .map(|o| o.retained_bytes())
                        .sum::<usize>();
                if self._owner.try_grow(bytes).is_err() {
                    self.complete = false;
                    self.record_fixture(
                        fixture,
                        Kind::Check,
                        Status::Failed,
                        "scientific evidence exceeds report memory budget",
                        oracle,
                        cap,
                    );
                    return;
                }
                let evidence = results::applicability_checks(self.run_id, &checks.applicability);
                self.model_checks(fixture, &evidence, oracle, cap);
                self.applicability.insert(fixture, checks.applicability);
                for check in checks.expectations {
                    let index = self.checks.len();
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
                    self.compared(
                        index,
                        (check.actual - check.expected).abs(),
                        Some(check.tolerance),
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
    /// Report the annotated hard ranges of a pure point using the same bounds as a
    /// solved point. Out-of-domain evaluation is never waived.
    fn point_validity(
        &mut self,
        fixture: DeclarationId,
        validity: &[pse_compiler::workspace::ModelingValidityResult],
        oracle: Option<DeclarationId>,
        cap: usize,
    ) {
        if validity.is_empty() {
            self.record_fixture(
                fixture,
                Kind::Envelope,
                Status::NotApplicable,
                "no declared validity envelope",
                oracle,
                cap,
            );
        }
        for v in validity {
            let within = v.within();
            self.record(
                fixture,
                v.target,
                v.source,
                Kind::Envelope,
                if within {
                    Status::Passed
                } else {
                    Status::Failed
                },
                format!(
                    "{}-layer value {} {} [{}, {}]",
                    v.layer.as_str(),
                    v.value,
                    if within { "within" } else { "outside" },
                    v.lower,
                    v.upper,
                ),
                oracle,
                cap,
            );
        }
    }
    /// Pure-point scientific evidence as the registry-owned `modeling_checks` relation.
    pub fn applicability_table(
        &self,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let scratch = pse_columnar::MemoryConsumer::new("conformance:applicability-export")
            .register(&self.pool);
        let bytes = self
            .applicability
            .values()
            .flatten()
            .try_fold(0usize, |bytes, o| {
                bytes
                    .checked_add(size_of::<ModelingCheck>())
                    .and_then(|bytes| bytes.checked_add(o.retained_bytes()))
                    .and_then(|bytes| bytes.checked_add(512))
            })
            .and_then(|bytes| bytes.checked_mul(2))
            .ok_or_else(|| contract("applicability projection byte allowance overflow"))?;
        scratch
            .try_grow(bytes)
            .map_err(pse_columnar::CanonError::from)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        let rows = self
            .applicability
            .values()
            .flat_map(|observations| results::applicability_checks(self.run_id, observations))
            .collect::<Vec<_>>();
        self.export(&rows)
    }
    /// Typed admission facts from every resolved fixture, including refused fixtures.
    pub fn admission_tables(
        &self,
    ) -> Result<BTreeMap<SemanticId, pse_relations::columnar::FieldCheckedBatch>, WorkflowError>
    {
        use pse_relations::columnar::FieldCheckedBatch;
        let cancel = pse_columnar::CancellationToken::new();
        ["runtime.route_decisions", "runtime.structural_assessments"]
            .into_iter()
            .map(|name| {
                let spec = self
                    .registry
                    .relation(name)
                    .ok_or_else(|| contract("conformance admission relation absent"))?;
                let inputs = self
                    .admissions
                    .values()
                    .filter_map(|tables| tables.get(&spec.id))
                    .cloned()
                    .collect::<Vec<_>>();
                let batch = FieldCheckedBatch::concat_reserved(
                    &self.registry,
                    spec,
                    &inputs,
                    &self.pool,
                    &cancel,
                )
                .map_err(relation)?;
                Ok((batch.relation_id(), batch))
            })
            .collect()
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
        let mut columns = pse_relations::columnar::Collection::new(
            &self.registry,
            &self.pool,
            &cancel,
            &self.validation,
        );
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
        let source_rows = self.declarations().await?;
        let fixtures = policy.fixtures.tests(&source_rows)?;
        // Every fixture's declared execution policy is resolved, and refused, before any
        // fixture runs.
        let policies = fixtures
            .iter()
            .map(|row| fixture_policy(row, &policy))
            .collect::<Result<Vec<_>, _>>()?;
        let mut report = ModelingConformanceReport::new(
            self.runtime.registry.clone(),
            self.runtime.shared.pool(),
            self.runtime.validation_context()?,
            &fixtures
                .iter()
                .map(|r| r.declaration_id)
                .collect::<Vec<_>>(),
            policy.maximum_checks,
        )?;
        report.selection = policy.fixtures.clone();
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
        for (index, (row, policy)) in fixtures.iter().zip(policies).enumerate() {
            // A fixture-local resource refusal makes the report incomplete, but does
            // not prevent independent fixtures from running within the remaining bounds.
            if index >= policy.maximum_fixtures
                || cancel.token().is_cancelled()
                || report.checks.len() >= cap
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

            let revision = match self.selected_revision(fixture, cancel).await {
                Ok(revision) => revision,
                Err(error) => {
                    report.failed(fixture, Kind::Preparation, &error, None, None, cap);
                    continue;
                }
            };
            let oracle = revision.oracle(fixture);
            report.note_oracle(oracle, |oracle| revision.release_of(oracle));
            let admitted = match self
                .declared_execution(
                    fixture,
                    policy.compiler,
                    policy.solver.clone(),
                    policy.numerical.clone(),
                    policy.limits,
                    cancel,
                )
                .await
            {
                Ok(execution) => execution,
                // Expected failures are resolved within the admitted fixture model.
                Err(error) => {
                    report.failed(fixture, Kind::Preparation, &error, None, oracle, cap);
                    continue;
                }
            };
            let model = admitted.model.clone();
            let bindings = admitted.analysis.bindings.clone();
            let solve_order = admitted.analysis.order;
            covered.extend(
                model
                    .compiled()
                    .model
                    .instances
                    .values()
                    .map(|i| i.definition),
            );
            report.note_units(fixture, &model.compiled().model);
            let data = model
                .compiled()
                .model
                .fixtures
                .get(&pse_modeling::specialize::root_instance(fixture));
            let expected = data.map_or(0, |f| f.expected_degrees_of_freedom);
            let mut expected_failure = data.and_then(|f| f.expected_failure.as_ref());
            let case = admitted.analysis.case.clone();
            if matches!(&admitted.procedure, DeclaredProcedure::Check) {
                let checked = self
                    .canonical_point(
                        fixture,
                        bindings,
                        admitted.analysis.limits,
                        admitted.analysis.compiler,
                        cancel,
                    )
                    .await;
                report.pure_result(fixture, model.compiled(), checked, cap);
                continue;
            }
            // A shooting fixture solves the shooting problem it declares (ADR-0110 Outcome 5):
            // its schedules held free are the controls and the model's objective level is
            // minimized; the stitched trajectory carries the model's checks.
            if matches!(&admitted.procedure, DeclaredProcedure::Shooting { .. }) {
                #[cfg(feature = "solver-diffsol")]
                match self.conform_shooting(&admitted, cancel).await {
                    Ok(result) => {
                        let crate::workflow::RunReport::Shooting(shooting) = result
                            .report
                            .as_ref()
                            .map_err(|error| WorkflowError::Shared(error.clone()))?
                        else {
                            return Err(contract("shooting conformance report mismatch"));
                        };
                        report.record_fixture(fixture, Kind::StartToSolve,
                            if result.usable() && expected_failure.is_none() { Status::Passed } else { Status::Failed },
                            format!("{} shooting over the declared controls; objective {:?}, continuity {:?}",
                                shooting.method.as_str(), shooting.objective, shooting.continuity), oracle, cap);
                        report.model_checks(fixture, &shooting.checks, oracle, cap);
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
                    }
                }
                #[cfg(not(feature = "solver-diffsol"))]
                report.failed(
                    fixture,
                    Kind::Preparation,
                    &contract("shooting needs a linked integrator"),
                    expected_failure,
                    oracle,
                    cap,
                );
                continue;
            }
            if matches!(&admitted.procedure, DeclaredProcedure::Integrate(_)) {
                match self.conform_integrated(&admitted, cancel).await {
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
                                report.model_checks(fixture, trajectory.checks(), oracle, cap);
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
                            if trajectory.accepted() && expected_failure.is_none() {
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
                        for (index, sample) in trajectory.report().samples.iter().enumerate() {
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
                                let (status, message, failure) = {
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
                        report.model_checks(fixture, trajectory.checks(), oracle, cap);
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
            if matches!(&admitted.procedure, DeclaredProcedure::Initialize(_)) {
                match self
                    .initialize_declared(&admitted, InitializationOverrides::default(), cancel)
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
                                    pse_diagnostics::DiagnosticStage::Initialization,
                                    [fixture.as_id()],
                                    pse_diagnostics::DiagnosticRule::ModelingInitializationIncomplete,
                                ))
                            })
                        };
                        let committed = initialization.committed.is_some();
                        report.initializations.insert(fixture, initialization);
                        if let Some(failure) = failure {
                            let error: WorkflowError = failure.into();
                            let observed = expected_failure
                                .is_some_and(|e| e.matches(&error.boundary_diagnostic()));
                            report.failed(
                                fixture,
                                Kind::StartToSolve,
                                &error,
                                expected_failure,
                                oracle,
                                cap,
                            );
                            if !observed {
                                continue;
                            }
                            // An expected initialization failure leaves the specification
                            // intact (PS-08): the failed attempts commit nothing, and the
                            // unchanged specification is solved from its own starts and
                            // answers to the fixture's checks and expectations.
                            report.record_fixture(
                                fixture,
                                Kind::Check,
                                if committed {
                                    Status::Failed
                                } else {
                                    Status::Passed
                                },
                                if committed {
                                    "a failed initialization committed values"
                                } else {
                                    "the failed initialization committed nothing; the unchanged specification is solved from its own starts"
                                },
                                oracle,
                                cap,
                            );
                            expected_failure = None;
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
            // A fixture that expects diagnostic findings diagnoses its solved point under
            // the same bindings and case (Plan 23 CT-S13).
            let expected_diagnostics = data.map(|f| f.diagnostics.clone()).unwrap_or_default();
            let diagnostic_inputs =
                (!expected_diagnostics.is_empty()).then(|| (bindings.clone(), case.clone()));
            let resolution = match self
                .resolve_case(
                    fixture,
                    pse_modeling::specialize::root_instance(fixture),
                    bindings,
                    admitted.analysis.limits,
                    case,
                    solve_order,
                    admitted.analysis.compiler,
                    admitted.analysis.solver.clone(),
                    admitted.analysis.numerical.clone(),
                    cases::CaseOverrides {
                        seed,
                        ..Default::default()
                    },
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
            let decision = match resolution.route_decision().and_then(|decision| {
                let identity = resolution.admission_identity(&decision)?;
                let tables =
                    analysis_tables::admission_tables(&self.runtime, &decision, identity, 0)?;
                Ok((decision, tables))
            }) {
                Ok((decision, tables)) => {
                    report.admissions.insert(fixture, tables);
                    decision
                }
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
            // An unresolved or refused route has no selected assessment. Each candidate
            // still retains the same original variable/equality inventory; using that
            // inventory does not select its structural mode or bypass evidence acquisition.
            let assessment = decision.structure.as_ref().or_else(|| {
                decision
                    .eligibility
                    .iter()
                    .find_map(|candidate| candidate.structure.as_ref())
            });
            if let Some(assessment) = assessment {
                let free = assessment.variables.len();
                let dof = assessment.inventory_difference();
                let equalities = free as i64 - dof;
                report.record_fixture(
                    fixture,
                    Kind::DegreesOfFreedom,
                    if dof == expected {
                        Status::Passed
                    } else {
                        Status::Failed
                    },
                    format!(
                        "original free variables {free}; equalities {equalities}; inventory difference {dof}; expected {expected}"
                    ),
                    oracle,
                    cap,
                );
            } else {
                report.complete = false;
                report.record_fixture(
                    fixture,
                    Kind::DegreesOfFreedom,
                    Status::Inconclusive,
                    "original fixture structural assessment unavailable; inventory difference unassessed",
                    oracle,
                    cap,
                );
            }
            let free = resolution.model.case.compiled().plan.columns().len();
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
                    // A derivative inspection refused for the fixture's expected reason,
                    // such as the structural refusal of an over-specified model, observes
                    // that expected failure.
                    Err(error) => report.failed(
                        fixture,
                        Kind::Derivatives,
                        &error,
                        expected_failure,
                        oracle,
                        cap,
                    ),
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
                match self.finish_case(resolution, cancel).await {
                    Ok(prepared) => {
                        match prepared
                            .admission_identity()
                            .and_then(|identity| prepared.admission_tables(identity, 0))
                        {
                            Ok(tables) => {
                                report.admissions.insert(fixture, tables);
                            }
                            Err(error) => {
                                report.complete = false;
                                report.failed(
                                    fixture,
                                    Kind::Preparation,
                                    &error,
                                    None,
                                    oracle,
                                    cap,
                                );
                                continue;
                            }
                        }
                        self.solve_case(prepared, policy.compiler, cancel).await
                    }
                    Err(error) => Err(error),
                }
            };
            match solved {
                Ok(result) => {
                    // Keep the actual completed attempt even if its admission export
                    // refuses. Publication failure does not reinterpret the native result.
                    report.results.insert(fixture, result.clone());
                    match result
                        .prepared
                        .admission_identity()
                        .and_then(|identity| result.prepared.admission_tables(identity, 0))
                    {
                        Ok(tables) => {
                            report.admissions.insert(fixture, tables);
                        }
                        Err(error) => {
                            report.complete = false;
                            report.failed(fixture, Kind::Preparation, &error, None, oracle, cap);
                        }
                    }
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
                    if let Some((bindings, case)) = diagnostic_inputs {
                        let diagnosed = if result.accepted {
                            self.diagnose_solution(
                                fixture,
                                bindings,
                                case,
                                solve_order,
                                &policy,
                                &result,
                                cancel,
                            )
                            .await
                        } else {
                            Err(contract("no accepted solution to diagnose"))
                        };
                        report.expected_diagnostics(
                            fixture,
                            &expected_diagnostics,
                            diagnosed,
                            oracle,
                            cap,
                        );
                    }
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
        report.coverage(&source_rows, &covered, cap);
        if cancel.token().is_cancelled() {
            report.complete = false;
        }
        Ok(report)
    }
    /// Numerical diagnostics at a fixture's accepted solution, under its bindings, case and
    /// the run's diagnostic thresholds: the solved values replace the specification's
    /// values of the same coordinates.
    #[expect(
        clippy::too_many_arguments,
        reason = "one fixture's bindings, case, derivative order, policy and result"
    )]
    async fn diagnose_solution(
        &self,
        fixture: DeclarationId,
        bindings: Bindings,
        case: ModelingCaseBindings,
        order: DerivativeOrder,
        policy: &ModelingConformancePolicy,
        result: &ModelingResult,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingDiagnostics, WorkflowError> {
        let thresholds = policy
            .diagnostics
            .clone()
            .ok_or_else(|| contract("the run states no diagnostic thresholds"))?;
        let analysis = ModelingAnalysis {
            root: fixture,
            instance: pse_modeling::specialize::root_instance(fixture),
            bindings,
            limits: policy.limits,
            case,
            order,
            compiler: policy.compiler,
            solver: policy.solver.clone(),
            numerical: policy.numerical.clone(),
        };
        let prepared = self.prepare_diagnostics(&analysis, cancel).await?;
        let mut values = prepared.model.values.clone();
        for (id, value) in &mut values.scalars {
            if let Some(solved) = result.values.scalars.get(id) {
                *value = *solved;
            }
        }
        self.diagnose_case(prepared, values, thresholds, policy.compiler, cancel)
            .await
    }
    async fn conform_integrated(
        &self,
        execution: &DeclaredExecution,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingTrajectory, WorkflowError> {
        self.simulation_for_declared(execution, None, cancel)
            .await?
            .run(cancel)
            .await
    }
    /// Solve an authored shooting fixture's problem on the math service.
    #[cfg(feature = "solver-diffsol")]
    async fn conform_shooting(
        &self,
        execution: &DeclaredExecution,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<crate::workflow::RunResult>, WorkflowError> {
        let problem = Arc::new(self.shooting_for_declared(execution, None, cancel).await?);
        let handle = problem.start()?;
        let finish = handle.wait();
        tokio::pin!(finish);
        let result = tokio::select! {
            result = &mut finish => result,
            () = cancel.cancelled() => { handle.cancel(); finish.await }
        }?;
        result
            .report
            .as_ref()
            .map_err(|error| WorkflowError::Shared(error.clone()))?;
        Ok(result)
    }
    async fn conformance_derivatives(
        &self,
        resolution: &ModelingCaseResolution,
        policy: pse_backend_native::derivative_diagnostics::Policy,
        cancel: &crate::CancelSource,
    ) -> Result<(bool, bool, String), WorkflowError> {
        let service = self.runtime.shared.math();
        let controls = resolution.solver.controls.clone();
        controls.validate().map_err(MathRuntimeError::from)?;
        let control = pse_columnar::flight::FlightCancellation::default();
        let deadline = std::time::Instant::now()
            .checked_add(controls.time_limit)
            .ok_or(MathRuntimeError::Limit(
                "derivative diagnostic deadline extent",
            ))?;
        let scope = pse_kernels::ExecutionScope::new(control.flag(), Some(deadline));
        let owner = service.reserve(
            "modeling:derivative-sample",
            policy.allowance().map_err(MathRuntimeError::from)?,
        )?;
        let prepared = service
            .prepare_order_within_task(
                resolution.model.case.clone(),
                DerivativeOrder::First,
                &scope,
                cancel,
            )
            .await?;
        let plan = &prepared.compiled().plan;
        let providers = crate::math::MathService::within_task(
            &scope,
            cancel,
            self.inner_registrations(
                resolution.model.model.clone(),
                &resolution.case_bindings,
                &resolution.numerical,
                &resolution.solver.numerics,
                &resolution.solver.controls,
                DerivativeOrder::First,
                resolution.compiler,
                cancel,
                implicit::ProviderDemand::Case(plan),
            ),
        )
        .await?;
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
        let assembly =
            crate::math::MathService::within_task(&scope, cancel, service.assemble(prepared))
                .await?;
        Ok(service
            .with_owned_worker(
                assembly,
                providers,
                cancel,
                Some((scope.clone(), control)),
                move |worker| {
                    let execution = pse_backend_native::solve::Execution::within(
                        worker.cancellation().clone(),
                        &controls,
                        scope,
                    )?;
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
        let AnnotationValue::Valid { .. } = &range.value else {
            return Err(contract("validity sample contract"));
        };
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
            // A closure range's rejection names its annotation (Plan 23 H5).
            pse_math::MathError::Validity(lineage) => lineage.source == s,
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
            MathRuntimeError::Strategy { cause, .. }
            | MathRuntimeError::StrategyTraceUnavailable { cause, .. } => runtime(cause, s),
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
    #[cfg(feature = "solver-kinsol")]
    #[derive(Debug)]
    struct ObservedNestedFactory {
        original: pse_kernels::Registration,
        scopes: Arc<Mutex<Vec<pse_kernels::ExecutionScope>>>,
        delay: std::time::Duration,
    }
    #[cfg(feature = "solver-kinsol")]
    impl pse_kernels::ProviderFactory for ObservedNestedFactory {
        fn spec(&self) -> &pse_kernels::ProviderSpec {
            self.original.spec()
        }
        fn configuration_key(&self) -> pse_ids::ContentHash {
            self.original.configuration_key()
        }
        fn create(&self) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
            self.original.worker()
        }
        fn create_scoped(
            &self,
            scope: pse_kernels::ExecutionScope,
        ) -> Result<Box<dyn pse_kernels::Provider>, pse_kernels::ProviderError> {
            self.scopes.lock().unwrap().push(scope.clone());
            let provider = self.original.worker_scoped(scope.clone())?;
            std::thread::sleep(self.delay);
            scope.check()?;
            Ok(provider)
        }
        fn envelope(&self) -> Option<Vec<(f64, f64)>> {
            self.original.envelope().map(<[_]>::to_vec)
        }
    }
    #[cfg(feature = "solver-kinsol")]
    async fn nested_scope_fixture() -> (ModelingPackage, ModelingCaseResolution) {
        use super::super::super::tests as fixture;
        let runtime = fixture::runtime_with(128 << 20, 16 << 20, 1 << 30);
        let rows = pse_authoring::language::parse(
            "package p { def Root {param p:Scalar=4; var x:Scalar; annotation start x(2); implicit root select operational(y=1) settings(\"native.kinsol.v1\") {var y:Scalar; eq e:y*y==p; annotation start y(1); annotation bounds y(0.1,10);} realize policy on root using nested; eq e:x==root.y;} }",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        ).unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, fixture::physical())
            .await
            .unwrap();
        let resolution = package
            .resolve_case(
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                ModelingCaseBindings::default(),
                DerivativeOrder::Value,
                fixture::compiler_profile(),
                fixture::profile(),
                NumericalInputs::default(),
                cases::CaseOverrides::default(),
                false,
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        assert_eq!(resolution.providers.len(), 1);
        (package, resolution)
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn derivative_worker_keeps_submitted_scope_in_actual_nested_factory_and_late_exit() {
        use crate::math::MathService;
        use pse_backend_native::solve::{Controls, Execution};
        let (package, resolution) = nested_scope_fixture().await;
        let service = package.runtime.shared.math();
        let cancel = crate::CancelSource::new();
        let prepared = service
            .prepare_order(
                resolution.model.case.clone(),
                DerivativeOrder::First,
                Default::default(),
            )
            .await
            .unwrap();
        let plan = &prepared.compiled().plan;
        let row_ids = plan
            .structure()
            .rows()
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>();
        let tolerances = pse_backend_native::quality::Tolerances::from_policy(
            &resolution.numerics,
            plan.columns(),
            &row_ids,
        )
        .unwrap();
        let assembly = service.assemble(prepared).await.unwrap();
        let scopes = Arc::new(Mutex::new(Vec::new()));
        let providers = |delay| {
            resolution
                .providers
                .iter()
                .map(|(&key, registration)| {
                    let factory = Arc::new(ObservedNestedFactory {
                        original: registration.clone(),
                        scopes: scopes.clone(),
                        delay,
                    });
                    (
                        key,
                        pse_kernels::Registration::bind(registration.descriptor(), factory)
                            .unwrap(),
                    )
                })
                .collect()
        };
        let control = pse_columnar::flight::FlightCancellation::default();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let scope = pse_kernels::ExecutionScope::new(control.flag(), Some(deadline));
        let expected = scope.clone();
        let values = resolution.model.values.clone();
        let result = service
            .with_owned_worker(
                assembly.clone(),
                providers(std::time::Duration::ZERO),
                &cancel,
                Some((scope.clone(), control)),
                move |mut worker| {
                    let execution = Execution::within(
                        worker.cancellation().clone(),
                        &Controls::default(),
                        scope,
                    )?;
                    assert_eq!(execution.scope()?.deadline(), Some(deadline));
                    assert!(Arc::ptr_eq(&execution.cancel, worker.cancellation()));
                    let rows = worker.constraints(&values)?;
                    execution.check()?;
                    Ok(rows)
                },
            )
            .await
            .unwrap();
        assert_eq!(result.len(), tolerances.rows.len());
        for ((id, value), allowance) in row_ids.iter().zip(&result).zip(&tolerances.rows) {
            assert!(
                value.abs() <= *allowance,
                "row {id}: {value} exceeds {allowance}"
            );
        }
        let observed = scopes.lock().unwrap()[0].clone();
        assert_eq!(observed.deadline(), expected.deadline());
        assert!(Arc::ptr_eq(
            observed.cancellation(),
            expected.cancellation()
        ));
        let control = pse_columnar::flight::FlightCancellation::default();
        let scope = pse_kernels::ExecutionScope::new(
            control.flag(),
            Some(std::time::Instant::now() + std::time::Duration::from_millis(100)),
        );
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let observed_call = called.clone();
        let result = service
            .with_owned_worker(
                assembly.clone(),
                providers(std::time::Duration::from_millis(150)),
                &cancel,
                Some((scope.clone(), control.clone())),
                move |_| {
                    observed_call.store(true, std::sync::atomic::Ordering::Release);
                    Ok(())
                },
            )
            .await;
        assert!(matches!(
            result,
            Err(MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Provider(pse_kernels::ProviderError::Deadline)
            ))
        ));
        assert!(!called.load(std::sync::atomic::Ordering::Acquire));
        assert!(
            !scope
                .cancellation()
                .load(std::sync::atomic::Ordering::Acquire)
        );
        assert_eq!(scopes.lock().unwrap()[1].deadline(), scope.deadline());
        // Already-expired prepared work does not construct another provider.
        let result = service
            .with_owned_worker(
                assembly,
                providers(std::time::Duration::ZERO),
                &cancel,
                Some((scope.clone(), control)),
                |_| Ok(()),
            )
            .await;
        assert!(matches!(
            result,
            Err(MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Provider(pse_kernels::ProviderError::Deadline)
            ))
        ));
        let result = MathService::within_task(
            &scope,
            &cancel,
            service.assemble(resolution.model.case.clone()),
        )
        .await;
        assert!(matches!(
            result,
            Err(MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Provider(pse_kernels::ProviderError::Deadline)
            ))
        ));
        assert_eq!(scopes.lock().unwrap().len(), 2);
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn derivative_submission_deadline_expires_during_first_preparation_queue() {
        let (package, mut resolution) = nested_scope_fixture().await;
        let service = package.runtime.shared.math();
        let entered = Arc::new(tokio::sync::Notify::new());
        let gate = Arc::new((Mutex::new(false), std::sync::Condvar::new()));
        let e = entered.clone();
        let g = gate.clone();
        let admitted = service.clone();
        let job = tokio::spawn(async move {
            admitted
                .job(2, 0, Default::default(), move |_| {
                    e.notify_one();
                    let mut released = g.0.lock().unwrap();
                    while !*released {
                        released = g.1.wait(released).unwrap();
                    }
                    Ok(())
                })
                .await
        });
        entered.notified().await;
        assert_eq!(
            resolution.model.case.compiled().plan.order(),
            DerivativeOrder::Value
        );
        let control = pse_columnar::flight::FlightCancellation::default();
        let scope = pse_kernels::ExecutionScope::new(
            control.flag(),
            Some(std::time::Instant::now() + std::time::Duration::from_millis(25)),
        );
        let driver = crate::CancelSource::new();
        let result = service
            .prepare_order_within_task(
                resolution.model.case.clone(),
                DerivativeOrder::First,
                &scope,
                &driver,
            )
            .await;
        assert!(matches!(
            result,
            Err(MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Provider(pse_kernels::ProviderError::Deadline)
            ))
        ));
        assert!(
            !scope
                .cancellation()
                .load(std::sync::atomic::Ordering::Acquire)
        );
        assert!(!driver.token().is_cancelled());
        resolution.solver.controls.time_limit = std::time::Duration::from_millis(25);
        let result = package
            .conformance_derivatives(
                &resolution,
                policy().derivatives,
                &crate::CancelSource::new(),
            )
            .await;
        *gate.0.lock().unwrap() = true;
        gate.1.notify_one();
        job.await.unwrap().unwrap();
        assert!(matches!(
            result,
            Err(WorkflowError::Math(MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Provider(pse_kernels::ProviderError::Deadline)
            )))
        ));
    }
    #[tokio::test]
    async fn kernel_conformance_retains_inventory_and_accounts_variable_payloads() {
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
            pse_relations::validate::ValidationContext::local(
                &pse_schema::shared_registry().unwrap(),
            )
            .unwrap(),
            &ids,
            1,
        )
        .unwrap();
        let base = pool.reserved();
        let oracle = DeclarationId::from_bytes([3; 16]);
        let message = "derivative comparison\n".repeat(500);
        report.record_fixture(
            ids[0],
            Kind::Preparation,
            Status::Failed,
            &message,
            Some(oracle),
            1,
        );
        assert!(pool.reserved() >= base + message.len());
        assert_eq!(report.checks[0].message, message);
        assert_eq!(report.checks[0].oracle_source_id, Some(oracle));
        let mut failure = BoundaryDiagnostic::new(
            BoundaryClass::InvalidModel,
            pse_diagnostics::DiagnosticStage::Test,
            ids.map(DeclarationId::as_id),
            pse_diagnostics::DiagnosticRule::ModelingDiagnosticSamples,
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
    #[tokio::test]
    async fn kernel_conformance_memory_refusal_cannot_erase_or_pass_a_fixture() {
        use pse_columnar::MemoryPool;
        use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic, Observation};
        let pool = Arc::new(pse_columnar::GreedyMemoryPool::new(32000));
        let id = DeclarationId::from_bytes([1; 16]);
        let mut report = ModelingConformanceReport::new(
            pse_schema::shared_registry().unwrap(),
            pool.clone(),
            pse_relations::validate::ValidationContext::local(
                &pse_schema::shared_registry().unwrap(),
            )
            .unwrap(),
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
            pse_diagnostics::DiagnosticStage::Test,
            [id.as_id()],
            pse_diagnostics::DiagnosticRule::ModelingDiagnosticSamples,
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
    async fn package(text: &str) -> ModelingPackage {
        let runtime = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let rows = pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        runtime.modeling_package(rows, physical).await.unwrap()
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
            maximum_fixtures: 10,
            maximum_checks: 50,
            fixtures: Default::default(),
            diagnostics: None,
        }
    }
    /// Every check of a fixture that names an oracle carries the oracle's source entity
    /// identity, and the published relation keeps it (ADR-0123 Outcome 5).
    #[tokio::test]
    async fn conformance_publishes_oracle_source_id() {
        let p = package(
            "package p { entity kind source provenance { attribute title: Text; } entity kind release extends source { attribute version: Text; } entity release upstream { title = \"Upstream\", version = \"2.13.0\" } fn cube(x:Scalar)->Scalar=x*x*x; test compared oracle upstream { expect cube(2)==8 tolerance 1e-8; } test analytic { expect cube(3)==27 tolerance 1e-8; } }",
        ).await;
        let declarations = p.declarations().await.unwrap();
        let id = |name: &str| {
            declarations
                .iter()
                .find(|r| r.name == name)
                .unwrap()
                .declaration_id
        };
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        let (compared, analytic) = (id("compared"), id("analytic"));
        let checks = |fixture| {
            report
                .checks
                .iter()
                .filter(move |c| c.fixture_id == fixture)
        };
        assert!(checks(compared).count() > 0 && checks(analytic).count() > 0);
        assert!(checks(compared).all(|c| c.oracle_source_id == Some(id("upstream"))));
        assert!(checks(analytic).all(|c| c.oracle_source_id.is_none()));
        let table = report.table().unwrap();
        let rows = ModelingConformanceCheck::rows(&table).unwrap();
        assert_eq!(rows, report.checks);
        assert!(
            rows.iter()
                .any(|c| c.oracle_source_id == Some(id("upstream")))
        );
    }
    #[tokio::test]
    async fn kernel_conformance_discovers_pure_tests_and_reports_uncovered_definitions() {
        let p = package(
            "package p { fn cube(x:Scalar)->Scalar=x*x*x; test pure { expect cube(2)==8 tolerance 1e-8; } def Missing { var x:Scalar; eq e:x==1; } }",
        ).await;
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!report.passed());
        assert!(report.complete);
        assert!(report.checks.iter().any(|c| c.kind == Kind::Expectation
            && c.status == Status::Passed
            && c.oracle_source_id.is_none()));
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
        let p = package("package p { test pure { expect 1==1 tolerance 1e-8; } }").await;
        let mut limited = policy();
        limited.maximum_checks = 1;
        let result = p
            .conform(limited, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!result.complete);
        assert!(!result.passed());
    }
    /// A typed selection runs only the named tests and inventories only them; it records
    /// that it assessed no package coverage. Names that are no authored test are refused
    /// before any fixture runs.
    #[tokio::test]
    async fn kernel_conformance_runs_only_selected_fixtures() {
        let p = package(
            "package p { fn cube(x:Scalar)->Scalar=x*x*x; test first { expect cube(2)==8 tolerance 1e-8; } test second { expect cube(3)==27 tolerance 1e-8; } test wrong { expect cube(1)==2 tolerance 1e-8; } def Missing { var x:Scalar; eq e:x==1; } }",
        ).await;
        let declarations = p.declarations().await.unwrap();
        let id = |name: &str| {
            declarations
                .iter()
                .find(|r| r.name == name)
                .unwrap()
                .declaration_id
        };
        let selected = BTreeSet::from([id("first"), id("second")]);
        let mut chosen = policy();
        chosen.fixtures = ModelingFixtureSelection::Selected(selected.clone());
        let report = p
            .conform(chosen.clone(), &crate::CancelSource::new())
            .await
            .unwrap();
        // The failing and the uncovered members of the package are outside the selection.
        assert!(report.passed(), "{:?}", report.checks);
        assert!(report.complete);
        assert_eq!(report.fixtures(), selected);
        assert_eq!(report.selection, chosen.fixtures);
        assert!(
            report
                .checks
                .iter()
                .all(|c| c.fixture_id == NO_FIXTURE || selected.contains(&c.fixture_id))
        );
        let coverage = report
            .checks
            .iter()
            .filter(|c| c.kind == Kind::Coverage)
            .collect::<Vec<_>>();
        assert_eq!(coverage.len(), 1, "{coverage:?}");
        assert_eq!(coverage[0].status, Status::NotApplicable);
        // Control: the whole package fails its wrong fixture and its uncovered definition.
        let whole = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(whole.selection, ModelingFixtureSelection::Package);
        assert_eq!(whole.fixtures().len(), 3);
        assert!(!whole.passed());
        assert!(
            whole
                .checks
                .iter()
                .any(|c| c.kind == Kind::Coverage && c.status == Status::Failed)
        );
        // A definition, an unknown identity or an empty selection is refused, naming it.
        let unknown = DeclarationId::from_bytes([7; 16]);
        for (selection, named) in [
            (
                BTreeSet::from([id("first"), id("Missing")]),
                Some(id("Missing")),
            ),
            (BTreeSet::from([unknown]), Some(unknown)),
            (BTreeSet::new(), None),
        ] {
            let mut refused = policy();
            refused.fixtures = ModelingFixtureSelection::Selected(selection);
            let error = p
                .conform(refused, &crate::CancelSource::new())
                .await
                .unwrap_err();
            if let Some(named) = named {
                assert!(error.to_string().contains(&named.to_string()), "{error}");
            }
        }
    }
    /// A fixture's metadata or execution policy that disagrees with its route or states no
    /// allowance is refused at admission; a declared derivative policy outside its bounds
    /// refuses the whole run before any fixture runs.
    #[tokio::test]
    async fn kernel_conformance_refuses_mismatched_or_unbounded_fixture_policies() {
        let rt = super::super::super::tests::runtime();
        let physical = super::super::super::tests::physical();
        let rows = |metadata: &str| {
            pse_authoring::language::parse(&format!("package p {{test valid fixture {{dof 0; route steady; procedure check;}} {{expect 1==1 tolerance 1e-6;}} test invalid fixture {{dof 0; {metadata}}} {{expect 1==1 tolerance 1e-6;}}}}"), SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, pse_authoring::ParseBudget::default()).unwrap()
        };
        for metadata in [
            "route steady; procedure solve; stages(warm);",
            "route steady; procedure initialize; initialize homotopy(true) step(0) minimum(1e-6) growth(2) attempts(4) seconds(30);",
            "route steady; procedure initialize; initialize homotopy(true) step(0.5) minimum(0.6) growth(2) attempts(4) seconds(30);",
            "route steady; procedure initialize; initialize homotopy(true) step(0.5) minimum(1e-6) growth(1) attempts(4) seconds(30);",
            "route integrated; procedure integrate;",
            // A pure fixture starts no solver; allowances are positive.
            "route steady; procedure check; policy { backend ipopt; }",
            "route steady; procedure check; policy { derivatives step(1e-7); }",
            "route steady; procedure solve; policy { limits items(0); }",
            "route steady; procedure solve; policy { derivatives cells(0); }",
            // A foreign allowance is a solve's, and positive.
            "route steady; procedure check; policy { limits foreign_bytes(1048576); }",
            "route steady; procedure solve; policy { limits foreign_bytes(0); }",
        ] {
            let declarations = rows(metadata);
            let invalid = declarations
                .iter()
                .find(|row| row.name == "invalid")
                .unwrap()
                .declaration_id;
            let package = rt
                .modeling_package(declarations, physical.clone())
                .await
                .unwrap();
            assert!(
                package
                    .selected_revision(invalid, &crate::CancelSource::new())
                    .await
                    .is_err(),
                "{metadata}"
            );
        }
        // Admitted, but the derivative policy it declares is outside its bounds: the whole
        // run is refused, before the valid fixture ahead of it runs.
        for metadata in [
            "route steady; procedure solve; policy { derivatives step(1.5); }",
            "route steady; procedure solve; policy { derivatives step(0) tolerance(1e-4); }",
            "route steady; procedure solve; policy { derivatives tolerance(-1); }",
        ] {
            let package = rt
                .modeling_package(rows(metadata), physical.clone())
                .await
                .unwrap();
            let error = package
                .conform(policy(), &crate::CancelSource::new())
                .await
                .unwrap_err();
            let invalid = package
                .declarations()
                .await
                .unwrap()
                .iter()
                .find(|r| r.name == "invalid")
                .unwrap()
                .declaration_id;
            assert!(
                error.to_string().contains(&invalid.to_string()),
                "{metadata}: {error}"
            );
        }
    }
    #[tokio::test]
    async fn kernel_conformance_uses_initialized_original_results() {
        let p = package(
            "package p { def D { var x:Scalar; } test initialized fixture {dof 0; route steady; procedure initialize; initialize homotopy(false) step(0.5) minimum(1e-6) growth(2) attempts(2) seconds(30); fix root.x=2;} {child root:D=D(); expect root.x==2 tolerance 1e-8;} }",
        ).await;
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
 test initialized fixture {dof 0; route steady; procedure initialize; stages("warm");}
 {child root:D=D; expect root.x==4 tolerance 1e-8;}
 }"#;
        let p = package(source).await;
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
        let p = package(&source.replace("root.x==4", "root.x==5")).await;
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
    /// A fixture's expected diagnostics are checked at its accepted solution under the run's
    /// thresholds: two nearly parallel equations are reported as near-parallel rows, and the
    /// rank deficiency they cause names them and the variables of its near-null mode.
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn fixture_expects_diagnostic_findings_naming_members() {
        let source = r#"package p {
 def D {var x:Scalar; var y:Scalar; eq a:x+y==2; eq b:x+1.000001*y==2.000001;
   annotation start x(0.5); annotation start y(0.5);}
 test near fixture {dof 0; route steady; procedure solve; EXPECTED} {child root:D=D(); expect root.x==1 tolerance 1e-4;}
 }"#;
        let thresholds: ModelingDiagnosticPolicy = serde_json::from_str(include_str!(
            "../../../../../packages/reference/diagnostics/idaes-2.13.json"
        ))
        .unwrap();
        let run = |expected: &str, diagnostics: Option<ModelingDiagnosticPolicy>| {
            let text = source.replace("EXPECTED", expected);
            async move {
                let p = package(&text).await;
                p.conform(
                    ModelingConformancePolicy {
                        diagnostics,
                        ..policy()
                    },
                    &crate::CancelSource::new(),
                )
                .await
            }
        };
        let named = "diagnose \"jacobian.parallel_rows\" at(root.a, root.b); diagnose \"jacobian.numerical_rank_deficiency\" at(root.a, root.b, root.x, root.y);";
        let report = run(named, Some(thresholds.clone())).await.unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        let diagnosed = report
            .checks
            .iter()
            .filter(|c| c.kind == Kind::Expectation && c.message.contains("jacobian."))
            .collect::<Vec<_>>();
        assert_eq!(diagnosed.len(), 2);
        assert!(
            diagnosed
                .iter()
                .all(|c| c.status == Status::Passed && c.failure_ordinal.is_some())
        );
        // A finding that does not name the expected member fails the fixture.
        let unnamed = run(
            "diagnose \"jacobian.parallel_rows\" at(root.a, root.x);",
            Some(thresholds.clone()),
        )
        .await
        .unwrap();
        assert!(!unnamed.passed());
        // Expected findings need the run's thresholds.
        assert!(run(named, None).await.is_err());
    }
    #[tokio::test]
    async fn conformance_publishes_original_structural_refusal_without_a_run_result() {
        let p = package(
            r#"package p {
 def D { var x:Scalar; var y:Scalar; eq a:x==1; eq b:y==2; eq c:x+y==3;
 annotation start x(0); annotation start y(0); }
 test over fixture { dof -1; route steady; procedure solve; } { child root:D=D(); }
 }"#,
        )
        .await;
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.results.is_empty());
        let tables = report.admission_tables().unwrap();
        let route = pse_relations::generated::runtime::route_decisions::Row::rows(
            &tables[&pse_relations::generated::runtime::route_decisions::spec(&p.runtime.registry)
                .unwrap()
                .id],
        )
        .unwrap();
        let structural_id =
            pse_relations::generated::runtime::structural_assessments::spec(&p.runtime.registry)
                .unwrap()
                .id;
        let structure = pse_relations::generated::runtime::structural_assessments::Row::rows(
            &tables[&structural_id],
        )
        .unwrap();
        assert_eq!(route.len(), 1);
        assert_eq!(
            route[0].refusal,
            Some(pse_model::generated::enums::NativeRouteRefusal::NoEligible)
        );
        assert!(route[0].selected.is_none());
        assert_eq!(structure.len(), 1);
        assert_eq!(structure[0].request_identity, route[0].request_identity);
        assert_eq!(
            structure[0].mode,
            pse_model::generated::enums::NativeStructuralMode::Roots
        );
        assert!(!structure[0].admitted);
        assert_eq!(structure[0].variables.len(), 2);
        assert_eq!(structure[0].equations.len(), 3);
        assert_eq!(structure[0].matching.len(), 2);
        assert_eq!(structure[0].unmatched_rows.len(), 1);
        assert!(structure[0].unmatched_columns.is_empty());
        let fixture = p
            .declarations()
            .await
            .unwrap()
            .iter()
            .find(|row| row.name == "over")
            .unwrap()
            .declaration_id;
        let defaults = policy();
        let cancel = crate::CancelSource::new();
        let execution = p
            .declared_execution(
                fixture,
                defaults.compiler,
                defaults.solver,
                defaults.numerical,
                defaults.limits,
                &cancel,
            )
            .await
            .unwrap();
        let error = p.prepare_declared(&execution, &cancel).await.unwrap_err();
        let WorkflowError::ModelingAdmission { diagnostic, cause } = error else {
            panic!("attributed typed admission refusal expected");
        };
        assert!(
            matches!(cause.as_ref(), MathRuntimeError::Solve(pse_backend_native::ProblemError::RouteRefused(decision))
            if decision.structure.as_ref().is_some_and(|assessment| assessment.refusal.is_some()))
        );
        assert_eq!(
            diagnostic.class,
            pse_model::diagnostic::BoundaryClass::InvalidModel
        );
        assert_eq!(
            diagnostic.rule,
            pse_diagnostics::DiagnosticRule::NativeStructural
        );
        assert!(!diagnostic.locations.is_empty());
        for member in ["root.a", "root.b", "root.c"] {
            assert!(
                diagnostic
                    .locations
                    .iter()
                    .any(|location| location.path.ends_with(member)),
                "{diagnostic:?}"
            );
        }
        // The retained buffers remain inspectable after the fixture report and package drop.
        drop(report);
        drop(p);
        assert_eq!(
            pse_relations::generated::runtime::structural_assessments::Row::rows(
                &tables[&structural_id]
            )
            .unwrap(),
            structure
        );
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn accounting_totals_do_not_create_missing_closure_obligations() {
        let p = package(
            r#"package p {
 def D(net:Power) {
  var x:Scalar; eq e:x==1; annotation start x(0);
  accumulate tally:Power accounting tolerance 1e-6{W};
  contribute tally role positive=7{W};
  accumulate balance:Power observation tolerance 1e-6{W};
  contribute balance role positive=net;
 }
 test closed fixture {dof 0; route steady; procedure solve;} {child root:D=D(net=0{W});}
 test unclosed fixture {dof 0; route steady; procedure solve;} {child root:D=D(net=1{W});}
 }"#,
        )
        .await;
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        let declarations = p.declarations().await.unwrap();
        let id = |name: &str| {
            declarations
                .iter()
                .find(|r| r.name == name)
                .unwrap()
                .declaration_id
        };
        assert!(report.complete);
        assert_eq!(
            report.fixture_statuses[&id("closed")],
            Status::Passed,
            "{:?}; {:?}",
            report.checks,
            report.failures
        );
        assert_eq!(report.fixture_statuses[&id("unclosed")], Status::Failed);
        assert!(report.results[&id("closed")].accepted);
        assert!(!report.results[&id("unclosed")].accepted);
        for (fixture, expected) in [("closed", Status::Passed), ("unclosed", Status::Failed)] {
            let closure = report
                .checks
                .iter()
                .filter(|c| c.fixture_id == id(fixture) && c.kind == Kind::Closure)
                .collect::<Vec<_>>();
            assert_eq!(closure.len(), 1);
            assert_eq!(closure[0].status, expected);
        }
        assert!(!report.failures.iter().any(|f| {
            f.observations.values().any(|v| matches!(v,
            pse_model::diagnostic::Observation::Text(text) if text.contains("closure_unavailable")
        ))
        }));
    }
    /// An over-specified root model is refused structurally before any route is selected,
    /// naming its over-determined equations; its fixture expects that typed refusal, which
    /// the derivative inspection observes as well.
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn overspecified_root_is_refused_structurally_naming_members() {
        let source = r#"package p {
 def D {var x:Scalar; var y:Scalar; eq a:x==1; eq b:y==2; eq c:x+y==3; annotation start x(0); annotation start y(0);}
 test over fixture {dof -1; route steady; procedure solve;FAILURE} {child root:D=D();}
 }"#;
        let p = package(&source.replace(
            "FAILURE",
            " failure invalid_model members(root.a, root.b, root.c);",
        ))
        .await;
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        assert!(report.results.is_empty());
        let refusals = report
            .failures
            .iter()
            .filter(|f| f.rule == pse_diagnostics::DiagnosticRule::NativeStructural)
            .collect::<Vec<_>>();
        assert!(!refusals.is_empty());
        // The three equations over two variables form the over-determined part.
        assert!(
            refusals.iter().all(|f| f.sources.len() == 3),
            "{refusals:?}"
        );
        assert!(report.checks.iter().any(|c| {
            c.kind == Kind::StartToSolve
                && c.status == Status::Passed
                && c.failure_ordinal
                    .and_then(|ordinal| usize::try_from(ordinal).ok())
                    .and_then(|ordinal| report.failures.get(ordinal))
                    .is_some_and(|failure| {
                        failure.rule == pse_diagnostics::DiagnosticRule::NativeStructural
                    })
        }));
        // Unexpected, the same refusal fails the fixture.
        let unexpected = package(&source.replace("FAILURE", ""))
            .await
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!unexpected.passed());
    }
    /// An initialized fixture that expects its initialization to fail asserts that the
    /// failure leaves the specification intact: nothing is committed, and the unchanged
    /// specification is solved from its own starts and meets the fixture's expectations.
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn expected_initialization_failure_leaves_the_specification_intact() {
        let source = r#"package p {
 def D {var x:Scalar; eq e:x==4; annotation start x(1); annotation bounds x(0,10);
   stage "out_of_range" {override eq e:x==20;} }
 test failed fixture {dof 0; route steady; procedure initialize; stages("out_of_range");FAILURE}
 {child root:D=D; expect root.x==4 tolerance 1e-8;}
 }"#;
        let run = |text: String| async move {
            let mut execution = policy();
            execution.solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
                pse_backend_native::solve::Backend::Ipopt,
            );
            package(&text)
                .await
                .conform(execution, &crate::CancelSource::new())
                .await
                .unwrap()
        };
        // Unexpected, the stage's failure fails the fixture and nothing else is solved.
        let unexpected = run(source.replace("FAILURE", "")).await;
        assert!(!unexpected.passed());
        assert!(unexpected.results.is_empty());
        let initialization = unexpected.initializations.values().next().unwrap();
        assert!(!initialization.completed && initialization.committed.is_none());
        assert_eq!(initialization.attempts.len(), 1);
        let failure = &unexpected.failures[0];
        assert!(!failure.sources.is_empty(), "{failure:?}");
        assert!(
            matches!(failure.observations.get("stage"), Some(pse_model::diagnostic::Observation::Text(stage)) if stage == "out_of_range")
        );
        let expected = format!(" failure {} members(root.e);", failure.class.as_str());
        // Expected, the failure passes and the intact specification is solved and checked.
        let intact = run(source.replace("FAILURE", &expected)).await;
        assert!(
            intact.passed(),
            "checks={:?}; failures={:?}",
            intact.checks,
            intact.failures
        );
        assert_eq!(intact.results.len(), 1);
        assert!(intact.results.values().next().unwrap().accepted);
        assert!(intact.checks.iter().any(|c| c.kind == Kind::Check
            && c.status == Status::Passed
            && c.message.contains("committed nothing")));
        assert!(
            intact
                .checks
                .iter()
                .any(|c| c.kind == Kind::Expectation && c.status == Status::Passed)
        );
        // The intact specification still answers to the fixture's expectations.
        let wrong = run(source
            .replace("FAILURE", &expected)
            .replace("root.x==4", "root.x==5"))
        .await;
        assert!(!wrong.passed());
        // A different expected failure is not observed; the specification is not solved.
        let other = run(source.replace("FAILURE", " failure invalid_model members(root.e);")).await;
        assert!(!other.passed());
        assert!(other.results.is_empty());
        // Matching the class with an unrelated member must not hide the failed stage.
        let unrelated = run(source.replace(
            "FAILURE",
            &format!(" failure {} members(root.x);", failure.class.as_str()),
        ))
        .await;
        assert!(!unrelated.passed());
        assert!(unrelated.results.is_empty());
    }
    /// Each fixture runs under the run's policy with the solve intent and execution policy
    /// its declaration states, for that fixture only (ADR-0119); the specialized kernel
    /// fixture carries the declared intent.
    #[tokio::test]
    async fn kernel_conformance_reads_fixture_policies_from_declarations() {
        use pse_backend_native::presolve::Policy as Presolve;
        use pse_backend_native::solve::{Backend, SolveIntent as Intent, SolverSelection};
        let p = package(
            "package p { def D { var x:Scalar; eq e:x==1; } test declared fixture {dof 0; route steady; procedure solve; intent certify; policy { backend ipopt; presolve off; options { \"print_level\" = 0; }; derivatives step(1e-7) cells(64); limits items(12) body_occurrences(4096) foreign_bytes(2147483648); }} {child root:D=D();} test open fixture {dof 0; route steady; procedure solve;} {child root:D=D();} }",
        ).await;
        let rows = p.declarations().await.unwrap();
        let row = |name: &str| rows.iter().find(|r| r.name == name).unwrap();
        let mut run = policy();
        run.solver.controls.options.insert(
            "print_level".into(),
            pse_backend_native::solve::OptionValue::Integer(5),
        );
        let declared = fixture_policy(row("declared"), &run).unwrap();
        assert_eq!(declared.solver.intent, Intent::Certify);
        assert_eq!(
            declared.solver.selection,
            SolverSelection::Explicit(Backend::Ipopt)
        );
        assert!(matches!(declared.solver.presolve, Presolve::Off));
        assert_eq!(declared.derivatives.perturbation, 1e-7);
        assert_eq!(declared.derivatives.maximum_cells, 64);
        assert_eq!(
            declared.derivatives.relative_tolerance,
            run.derivatives.relative_tolerance
        );
        assert_eq!(
            declared.limits,
            Limits {
                items: 12,
                body_occurrences: Some(4096),
                ..run.limits
            }
        );
        // The declared foreign allowance becomes its solves' control; the time limit and
        // every other control stay the run's.
        let mut expected = run.solver.controls.clone();
        expected.foreign_bytes = Some(2 << 30);
        expected.options.insert(
            "print_level".into(),
            pse_backend_native::solve::OptionValue::Integer(0),
        );
        assert_eq!(declared.solver.controls, expected);
        let open = fixture_policy(row("open"), &run).unwrap();
        assert_eq!(open.solver.controls, run.solver.controls);
        assert_eq!(open.solver.intent, run.solver.intent);
        assert_eq!(open.solver.selection, run.solver.selection);
        assert!(matches!(open.solver.presolve, Presolve::Auto));
        assert_eq!(open.limits, run.limits);
        assert_eq!(
            (
                open.derivatives.perturbation,
                open.derivatives.relative_tolerance,
                open.derivatives.maximum_cells
            ),
            (
                run.derivatives.perturbation,
                run.derivatives.relative_tolerance,
                run.derivatives.maximum_cells
            )
        );
        let cancel = crate::CancelSource::new();
        for (name, expected) in [("declared", Some(Intent::Certify)), ("open", None)] {
            let id = row(name).declaration_id;
            let instance = pse_modeling::specialize::root_instance(id);
            let model = p
                .prepare(
                    id,
                    instance,
                    Bindings::default(),
                    Limits::default(),
                    &cancel,
                )
                .await
                .unwrap();
            assert_eq!(model.compiled().model.fixtures[&instance].intent, expected);
            let analysis = p
                .declared_execution(
                    id,
                    Default::default(),
                    run.solver.clone(),
                    Default::default(),
                    run.limits,
                    &cancel,
                )
                .await
                .unwrap()
                .analysis;
            let resolved = fixture_policy(row(name), &run).unwrap();
            assert_eq!(analysis.solver.controls, resolved.solver.controls);
            assert_eq!(analysis.solver.intent, resolved.solver.intent);
            assert_eq!(analysis.solver.selection, resolved.solver.selection);
            assert_eq!(analysis.limits, resolved.limits);
        }
        // A declared allowance applies to its own fixture only.
        let p = package(
            "package p { def D { var x:Scalar; let y:Scalar=x*x; } test large fixture {dof 0; route steady; procedure check; fix root.x=2;} {child root:D=D(); expect root.y==4 tolerance 1e-12;} test small fixture {dof 0; route steady; procedure check; policy { limits items(1); } fix root.x=2;} {child root:D=D(); expect root.y==4 tolerance 1e-12;} }",
        ).await;
        let declarations = p.declarations().await.unwrap();
        let id = |name: &str| {
            declarations
                .iter()
                .find(|r| r.name == name)
                .unwrap()
                .declaration_id
        };
        let report = p.conform(policy(), &cancel).await.unwrap();
        assert_eq!(
            report.fixture_statuses[&id("large")],
            Status::Passed,
            "{:?}",
            report.checks
        );
        // The exhausted allowance is its own: two items required, one allowed.
        assert_eq!(report.fixture_statuses[&id("small")], Status::Inconclusive);
        assert!(
            report.checks.iter().any(|c| c.fixture_id == id("small")
                && c.kind == Kind::Preparation
                && c.message.ends_with("required 2, allowed 1")),
            "{:?}",
            report.checks
        );
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn kernel_conformance_mixes_explicit_fixture_solver_and_derivative_policies() {
        let p = package(
            r#"package p {
            test root fixture {dof 0; route steady; procedure solve;} {
                var x:Scalar; eq e:x*x==4; annotation start x(1); annotation bounds x(0.5,3);
                expect x==2 tolerance 1e-6;
            }
            test optimization fixture {dof 1; route steady; procedure solve; intent optimize; policy { backend ipopt; derivatives step(1e-7) tolerance(1e-4) cells(100); }} {
                var x:Scalar; let cost:Scalar=(x-3)^2; annotation objective cost(minimize);
                annotation start x(1); expect x==3 tolerance 1e-6;
            }
        }"#,
        ).await;
        let fixture = p
            .declarations()
            .await
            .unwrap()
            .iter()
            .find(|r| r.name == "optimization")
            .unwrap()
            .declaration_id;
        let mut policy = policy();
        policy.solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Ipopt,
        );
        let report = p
            .conform(policy, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        assert_eq!(report.results.len(), 2);
        // The declared derivative step applies to its own fixture only.
        for (id, step) in report
            .checks
            .iter()
            .filter(|r| r.kind == Kind::Derivatives)
            .map(|r| (r.fixture_id, r.message.contains("step=0.0000001")))
        {
            assert_eq!(id == fixture, step);
        }
    }
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn kernel_conformance_prepares_the_requested_exact_hessian() {
        let p = package(
            "package p {def D {var x:Scalar; eq e:x*x==4; annotation start x(1); annotation bounds x(0.5,3);} test bounded fixture {dof 0; route steady; procedure solve;} {child root:D=D(); expect root.x==2 tolerance 1e-6;}}",
        ).await;
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
   annotation start x(1); annotation valid x(0,10); }
 entity kind source provenance { attribute title: Text; }
 entity source synthetic { title = "synthetic fixture" }
 test fixture oracle synthetic fixture { dof 0; lower root.x = 0; } {
   child root:D=D(); expect root.x==2 tolerance 1e-6;
 }
 }"#,
        )
        .await;
        let mut policy = policy();
        // One coordinate admits Value/First (two Taylor components), while Second
        // would need three. Shared diagnostic sampling must request only First.
        policy.compiler.evaluation.derivative_components = 2;
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
        let source = r#"package p {
            fn square(x:Scalar)->Scalar=x*x;
            test pure fixture { dof 0; route steady; procedure check; } {
                expect square(3)==9 tolerance 1e-12 relative 1e-6;
            }
            fn positive(x:Scalar)->Scalar valid(x > 0) = x;
            test negative fixture { dof 0; route steady; procedure check; failure trial_rejected validity(form) form(positive) variable(x); } {
                expect positive(-1)==1 tolerance 0;
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
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let report = package
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        assert_eq!(report.fixtures().len(), 2);
        assert!(report.results.is_empty());
        assert_eq!(report.failures.len(), 1);
        assert_eq!(
            report.failures[0].rule,
            pse_diagnostics::DiagnosticRule::MathValidity
        );
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
    /// A bank of two rows whose temperature envelope is data, a form guarding it at one
    /// argument beside its own domain, and an increment guarding its interval (Plan 23 H5).
    const ENVELOPE_BANK: &str = r#"entity kind source provenance { attribute title: Text; } enum role { given } entity source s { title = "synthetic bank" }
 entity kind item {} entity item a {} entity item b {} set items: Set<item> = {a, b};
 table cp_data[j: item]: {c: MolarCp, low: Temperature, high: Temperature} envelope T: Temperature in low..high complete_over(j in items);
 dataset bank: cp_data provenance(s, role.given) { [a] = [75{J/(mol*K)}, 250{K}, 400{K}]; [b] = [80{J/(mol*K)}, 300{K}, 500{K}]; }
 fn cp(T: Temperature, p: Row<cp_data>) -> MolarCp guards(p.T: T) valid(T > 0{K}) = p.c;
 fn dh(T0: Temperature, T: Temperature, p: Row<cp_data>) -> DeltaH guards(p.T: [T0, T]) = p.c*(T - T0);"#;
    async fn statuses(
        p: &ModelingPackage,
        report: &ModelingConformanceReport,
    ) -> BTreeMap<String, Status> {
        p.declarations()
            .await
            .unwrap()
            .iter()
            .filter(|r| r.value.kind == DeclarationKind::Test)
            .map(|r| (r.name.clone(), report.fixture_statuses[&r.declaration_id]))
            .collect()
    }
    /// Plan 23 H5: an expected envelope failure matches its class and its lineage: the data
    /// layer, the form, the row whose envelope rejects and the guarded argument, or the
    /// whole integration interval. A form-layer rejection names the form's own domain. The
    /// published finding carries the same typed lineage, and the clause round-trips.
    #[tokio::test]
    async fn expected_envelope_failure_matches_set_layer_and_variable() {
        use pse_model::generated::enums::ModelingValidityLayer as Layer;
        let source = format!(
            r#"package p {{ {ENVELOPE_BANK}
 test above fixture {{ dof 0; route steady; procedure check; failure trial_rejected validity(data) form(cp) set(cp_data[a]) variable(T); }} {{ expect cp(450{{K}}, cp_data[a]) == 75{{J/(mol*K)}} tolerance 1e-9{{J/(mol*K)}}; }}
 test interval fixture {{ dof 0; route steady; procedure check; failure trial_rejected validity(data) form(dh) set(cp_data[b]) variable(T0, T); }} {{ expect dh(250{{K}}, 350{{K}}, cp_data[b]) == 8000{{J/mol}} tolerance 1e-6{{J/mol}}; }}
 test negative fixture {{ dof 0; route steady; procedure check; failure trial_rejected validity(form) form(cp) variable(T); }} {{ expect cp(-5{{K}}, cp_data[a]) == 75{{J/(mol*K)}} tolerance 1e-9{{J/(mol*K)}}; }}
 test inside fixture {{ dof 0; route steady; procedure check; }} {{ expect cp(300{{K}}, cp_data[a]) == 75{{J/(mol*K)}} tolerance 1e-9{{J/(mol*K)}}; }}
}}"#
        );
        let rows = pse_authoring::language::parse(
            &source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let rendered = pse_authoring::language::render(&rows).unwrap();
        assert!(
            rendered.contains(
                "failure trial_rejected validity(data) form(dh) set(cp_data[b]) variable(T0, T);"
            ),
            "{rendered}"
        );
        let roundtrip = pse_authoring::language::parse(
            &rendered,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        assert_eq!(
            rows.iter().map(|r| &r.value).collect::<Vec<_>>(),
            roundtrip.iter().map(|r| &r.value).collect::<Vec<_>>()
        );
        let p = package(&source).await;
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        assert!(
            statuses(&p, &report)
                .await
                .values()
                .all(|s| *s == Status::Passed)
        );
        let declarations = p.declarations().await.unwrap();
        let id = |name: &str| {
            declarations
                .iter()
                .find(|r| r.name == name)
                .unwrap()
                .declaration_id
        };
        let row = |key: &str| {
            pse_modeling::data::row_identity(
                id("cp_data"),
                &[pse_modeling::specialize::Value::Entity {
                    id: id(key),
                    kind: id("item"),
                }],
            )
        };
        let lineages = report
            .failures
            .iter()
            .map(|f| {
                assert_eq!(f.class, pse_model::diagnostic::BoundaryClass::TrialRejected);
                let v = f
                    .validity
                    .clone()
                    .expect("a validity rejection carries its lineage");
                (v.layer, v.source, v.form, v.sets, v.variables)
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            lineages,
            BTreeSet::from([
                (
                    Layer::Data,
                    id("cp_data").as_id(),
                    Some(id("cp").as_id()),
                    vec![row("a")],
                    vec![0]
                ),
                (
                    Layer::Data,
                    id("cp_data").as_id(),
                    Some(id("dh").as_id()),
                    vec![row("b")],
                    vec![0, 1]
                ),
                (
                    Layer::Form,
                    id("cp").as_id(),
                    Some(id("cp").as_id()),
                    vec![],
                    vec![0]
                ),
            ])
        );
        // The published findings keep the lineage.
        let findings = report.findings_table().unwrap();
        let published =
            pse_model::generated::runtime::modeling_findings::Row::rows(&findings).unwrap();
        assert_eq!(published.iter().filter(|f| f.validity.is_some()).count(), 3);
        assert!(
            published.iter().any(
                |f| f.validity.as_ref().is_some_and(|v| v.layer == Layer::Data
                    && v.form_id == Some(id("dh").as_id())
                    && v.set_ids == [row("b")]
                    && v.variables == [0, 1])
            )
        );
    }
    /// Plan 23 H5: a failure of the expected class whose lineage differs fails its fixture,
    /// whichever part differs: the layer, the form, the parameter set or the variable. A
    /// members lineage matches only a finding naming every expected member. A fixture
    /// expecting a failure its evaluation does not produce fails too.
    #[tokio::test]
    async fn expected_failure_with_wrong_lineage_fails_the_fixture() {
        let call = "expect cp(450{K}, cp_data[a]) == 75{J/(mol*K)} tolerance 1e-9{J/(mol*K)};";
        let fixtures = [
            (
                "right",
                "validity(data) form(cp) set(cp_data[a]) variable(T)",
                Status::Passed,
            ),
            (
                "wrong_layer",
                "validity(form) form(cp) set(cp_data[a]) variable(T)",
                Status::Failed,
            ),
            (
                "wrong_form",
                "validity(data) form(dh) set(cp_data[a]) variable(T)",
                Status::Failed,
            ),
            (
                "wrong_set",
                "validity(data) form(cp) set(cp_data[b]) variable(T)",
                Status::Failed,
            ),
            (
                "wrong_variable",
                "validity(data) form(cp) set(cp_data[a]) variable(p)",
                Status::Failed,
            ),
        ];
        let tests = fixtures
            .iter()
            .map(|(name, lineage, _)| {
                format!("test {name} fixture {{ dof 0; route steady; procedure check; failure trial_rejected {lineage}; }} {{ {call} }}")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let source = format!(
            "package p {{ {ENVELOPE_BANK}\n{tests}\n test unobserved fixture {{ dof 0; route steady; procedure check; failure trial_rejected validity(data) form(cp) set(cp_data[a]) variable(T); }} {{ expect cp(300{{K}}, cp_data[a]) == 75{{J/(mol*K)}} tolerance 1e-9{{J/(mol*K)}}; }} }}"
        );
        let p = package(&source).await;
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        let statuses = statuses(&p, &report).await;
        for (name, _, expected) in fixtures {
            assert_eq!(statuses[name], expected, "{name}: {:?}", report.checks);
        }
        assert_eq!(statuses["unobserved"], Status::Failed);
        assert!(!report.passed());
        // A members lineage: every expected member is named by the finding.
        let fixture = pse_modeling::specialize::ExpectedFailure {
            class: pse_model::diagnostic::BoundaryClass::InvalidModel,
            lineage: pse_modeling::specialize::ExpectedLineage::Members(BTreeSet::from([
                SemanticId::from_bytes([1; 16]),
                SemanticId::from_bytes([2; 16]),
            ])),
        };
        let finding = |sources: &[u8]| {
            pse_model::diagnostic::BoundaryDiagnostic::new(
                pse_model::diagnostic::BoundaryClass::InvalidModel,
                pse_diagnostics::DiagnosticStage::Native,
                sources.iter().map(|b| SemanticId::from_bytes([*b; 16])),
                pse_diagnostics::DiagnosticRule::NativeStructural,
            )
        };
        assert!(fixture.matches(&finding(&[1, 2, 3])));
        assert!(!fixture.matches(&finding(&[1, 3])));
        let mut other_class = finding(&[1, 2]);
        other_class.class = pse_model::diagnostic::BoundaryClass::Numerical;
        assert!(!fixture.matches(&other_class));
    }
    /// Plan 23 H6 (CT-S14): the parity report lists every fixture that names an oracle,
    /// grouped by the definition it exercises and by its oracle, with the oracle's release,
    /// each check's deviation and tolerance and both dispositions; a fixture without an
    /// oracle is not parity evidence. It is a projection of the run's checks.
    #[tokio::test]
    async fn parity_report_lists_every_oracle_fixture_with_release_and_tolerance() {
        use pse_model::generated::runtime::modeling_parity::Row;
        let p = package(
            r#"package p {
 entity kind source provenance { attribute title: Text; }
 entity kind software_release extends source release { attribute version: Text; }
 entity kind oracle_test extends source { attribute release: software_release; attribute locator: Text; }
 entity kind publication extends source { attribute year: Integer; }
 entity software_release upstream { title = "Upstream", version = "2.13.0" }
 entity oracle_test upstream_test { title = "Upstream unit test", release = upstream, locator = "tests/test_unit.py" }
 entity publication handbook { title = "Handbook", year = 1997 }
 fn positive(x: Scalar) -> Scalar valid(x > 0) = x;
 def Unit { param x: Scalar; let y: Scalar = x*x; }
 test unit_fixture oracle upstream_test fixture { dof 0; route steady; procedure check; value root.x = 2; } { child root: Unit = Unit(); expect root.y == 4 tolerance 1e-9; }
 test release_fixture oracle upstream fixture { dof 0; route steady; procedure check; } { expect positive(3) == 3 tolerance 1e-6 relative 1e-3; }
 test handbook_fixture oracle handbook fixture { dof 0; route steady; procedure check; failure trial_rejected validity(form) form(positive) variable(x); } { expect positive(-1) == 1 tolerance 0.1; }
 test analytic fixture { dof 0; route steady; procedure check; } { expect positive(1) == 1 tolerance 1e-9; }
}"#,
        ).await;
        let declarations = p.declarations().await.unwrap();
        let id = |name: &str| {
            declarations
                .iter()
                .find(|r| r.name == name)
                .unwrap()
                .declaration_id
        };
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.passed(), "{:?}", report.checks);
        let table = report.parity_table().unwrap();
        let rows = Row::rows(&table).unwrap();
        // Every oracle fixture, and only those, with its oracle and release.
        let fixtures = rows
            .iter()
            .map(|r| {
                (
                    r.fixture_id,
                    r.definition_id,
                    r.oracle_source_id,
                    r.release_id,
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            fixtures,
            BTreeSet::from([
                (
                    id("unit_fixture"),
                    id("Unit"),
                    id("upstream_test"),
                    Some(id("upstream"))
                ),
                (
                    id("release_fixture"),
                    id("release_fixture"),
                    id("upstream"),
                    Some(id("upstream"))
                ),
                (
                    id("handbook_fixture"),
                    id("handbook_fixture"),
                    id("handbook"),
                    None
                ),
            ])
        );
        assert!(rows.iter().all(|r| r.status != Status::NotApplicable
            && r.fixture_status == Status::Passed
            && r.run_id == report.run_id));
        // Each compared value carries its deviation and combined tolerance.
        let compared = |fixture: &str| {
            rows.iter()
                .filter(|r| r.fixture_id == id(fixture) && r.kind == Kind::Expectation)
                .map(|r| (r.deviation.unwrap(), r.tolerance.unwrap(), r.status))
                .collect::<Vec<_>>()
        };
        assert_eq!(compared("unit_fixture"), [(0.0, 1e-9, Status::Passed)]);
        let [(deviation, tolerance, Status::Passed)] = compared("release_fixture")[..] else {
            panic!("{rows:?}");
        };
        assert_eq!(deviation, 0.0);
        assert!((tolerance - (1e-6 + 3e-3)).abs() < 1e-15);
        // An expected failure is parity evidence too: its check passes without a value.
        assert!(rows.iter().any(|r| r.fixture_id == id("handbook_fixture")
            && r.status == Status::Passed
            && r.deviation.is_none()
            && r.tolerance.is_none()));
        // The projection is exactly the run's oracle checks that applied, per unit.
        let applied = report
            .checks
            .iter()
            .filter(|c| c.oracle_source_id.is_some() && c.status != Status::NotApplicable)
            .count();
        assert_eq!(rows.len(), applied);
        // A release is a source.
        let refused = super::super::super::tests::runtime()
            .modeling_package(
                pse_authoring::language::parse(
                    "package q { entity kind version release { attribute name: Text; } }",
                    SemanticId::NIL,
                    pse_authoring::language::IdentityPolicy::Named,
                    pse_authoring::ParseBudget::default(),
                )
                .unwrap(),
                super::super::super::tests::physical(),
            )
            .await;
        assert!(
            refused
                .unwrap_err()
                .to_string()
                .contains("which are sources")
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
        // The fixture's source and role are declared by the seed's references and the
        // domain's provenance module (ADR-0123 Outcome 5).
        let rows = [
            include_str!("../../../../../packages/reference/seed-data/models/price-taker.pse"),
            include_str!("../../../../../packages/reference/data/references/models/references.pse"),
            include_str!("../../../../../packages/reference/domain/models/provenance.pse"),
            include_str!("../../../../../packages/reference/domain/models/numerical-policy.pse"),
            include_str!("../../../../../packages/reference/domain/models/properties.pse"),
            include_str!("../../../../../packages/reference/domain/models/constants.pse"),
            include_str!("../../../../../packages/reference/physical/models/chemistry.pse"),
            include_str!("../../../../../packages/reference/physical/models/compatibility.pse"),
        ]
        .into_iter()
        .flat_map(|text| {
            pse_authoring::language::parse(
                text,
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Explicit,
                pse_authoring::ParseBudget::default(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
        let fixture = rows
            .iter()
            .find(|r| r.name == "price_taker" && r.value.kind == DeclarationKind::Test)
            .unwrap()
            .declaration_id;
        let physical = super::super::super::tests::physical();
        // Admit the complete imported schema closure alongside its checked projection.
        let package = super::super::super::tests::runtime_with_workspace(64 << 20)
            .modeling_package(rows, physical)
            .await
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

    #[tokio::test]
    async fn kernel_conformance_retains_route_refusal_and_runs_following_fixture() {
        use pse_backend_native::routing::{AssessmentState, Refusal};
        use pse_relations::columnar::RelationRow;
        let p = package(
            r#"package p {
 test refused fixture { dof -1; route steady; procedure solve; } {
   var x:Scalar; eq a:x==1; eq b:x==2; annotation start x(0);
 }
 test following fixture { dof 0; route steady; procedure solve; } {
   param value:Scalar=2; expect value==2 tolerance 1e-8;
 }
 }"#,
        )
        .await;
        let declarations = p.declarations().await.unwrap();
        let id = |name: &str| {
            declarations
                .iter()
                .find(|row| row.name == name)
                .unwrap()
                .declaration_id
        };
        let cancel = crate::CancelSource::new();
        let defaults = policy();
        let admitted = p
            .declared_execution(
                id("refused"),
                defaults.compiler,
                defaults.solver,
                defaults.numerical,
                defaults.limits,
                &cancel,
            )
            .await
            .unwrap();
        let resolution = p
            .resolve_case(
                id("refused"),
                pse_modeling::specialize::root_instance(id("refused")),
                admitted.analysis.bindings.clone(),
                admitted.analysis.limits,
                admitted.analysis.case.clone(),
                admitted.analysis.order,
                admitted.analysis.compiler,
                admitted.analysis.solver.clone(),
                admitted.analysis.numerical.clone(),
                cases::CaseOverrides::default(),
                false,
                &cancel,
            )
            .await
            .unwrap();
        let decision = resolution.route_decision().unwrap();
        assert_eq!(decision.state, AssessmentState::Refused);
        assert!(matches!(decision.refusal, Some(Refusal::NoEligible)));
        assert!(decision.selected.is_none());
        assert!(decision.structure.as_ref().is_some_and(|assessment| {
            assessment.refusal.is_some() && assessment.inventory_difference() == -1
        }));
        assert!(decision.eligibility.iter().any(|candidate| {
            candidate.structure.as_ref().is_some_and(|assessment| {
                assessment.variables.len() == 1 && assessment.inventory_difference() == -1
            })
        }));
        let diagnostic = p
            .finish_case(resolution, &cancel)
            .await
            .unwrap_err()
            .boundary_diagnostic();
        let report = p.conform(policy(), &cancel).await.unwrap();
        assert!(report.complete);
        assert!(!report.passed());
        assert_eq!(report.fixture_statuses[&id("refused")], Status::Failed);
        assert_eq!(report.fixture_statuses[&id("following")], Status::Passed);
        assert!(!report.results.contains_key(&id("refused")));
        assert!(report.results[&id("following")].accepted);
        let refusal = report
            .checks
            .iter()
            .find(|check| check.fixture_id == id("refused") && check.kind == Kind::StartToSolve)
            .unwrap();
        assert_eq!(refusal.status, Status::Failed);
        assert_eq!(
            serde_json::to_value(&report.failures[refusal.failure_ordinal.unwrap() as usize])
                .unwrap(),
            serde_json::to_value(&diagnostic).unwrap()
        );
        assert!(report.checks.iter().any(|check| {
            check.fixture_id == id("refused")
                && check.kind == Kind::DegreesOfFreedom
                && check.status == Status::Passed
                && check.message.contains("inventory difference -1")
        }));
        let tables = report.admission_tables().unwrap();
        let route_id =
            pse_relations::generated::runtime::route_decisions::spec(&p.runtime.registry)
                .unwrap()
                .id;
        let routes =
            pse_relations::generated::runtime::route_decisions::Row::rows(&tables[&route_id])
                .unwrap();
        assert!(routes.iter().any(|route| {
            route.refusal == Some(pse_model::generated::enums::NativeRouteRefusal::NoEligible)
                && route.selected.is_none()
        }));
    }

    #[tokio::test]
    async fn kernel_conformance_continues_after_fixture_resource_refusal() {
        let p = package(
            r#"package p {
 def D { var x:Scalar; let y:Scalar=x*x; }
 test exhausted fixture { dof 0; route steady; procedure check;
   policy { limits items(1); } fix root.x=2;
 } { child root:D=D(); expect root.y==4 tolerance 1e-12; }
 test following fixture { dof 0; route steady; procedure solve; } {
   param value:Scalar=2; expect value==2 tolerance 1e-8;
 }
 }"#,
        )
        .await;
        let declarations = p.declarations().await.unwrap();
        let id = |name: &str| {
            declarations
                .iter()
                .find(|row| row.name == name)
                .unwrap()
                .declaration_id
        };
        let report = p
            .conform(policy(), &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(!report.complete);
        assert!(!report.passed());
        assert_eq!(
            report.fixture_statuses[&id("exhausted")],
            Status::Inconclusive
        );
        assert_eq!(report.fixture_statuses[&id("following")], Status::Passed);
        assert!(report.results[&id("following")].accepted);
        let refused = report
            .checks
            .iter()
            .find(|check| check.fixture_id == id("exhausted") && check.kind == Kind::Preparation)
            .unwrap();
        assert_eq!(refused.status, Status::Inconclusive);
        assert!(refused.message.ends_with("required 2, allowed 1"));
        let failure = &report.failures[refused.failure_ordinal.unwrap() as usize];
        assert_eq!(
            failure.class,
            pse_model::diagnostic::BoundaryClass::ResourceLimit
        );
        assert_eq!(
            failure.rule,
            pse_diagnostics::DiagnosticRule::ModelingBudget
        );
    }

    #[cfg(feature = "solver-highs")]
    #[tokio::test]
    async fn kernel_conformance_acquires_pending_class_evidence_and_continues() {
        use pse_backend_native::routing::AssessmentState;
        let p = package(
            r#"package p {
 test pending fixture { dof 1; route steady; procedure solve; intent optimize; } {
   var x:Scalar; let cost:Scalar=(x-3)^2; annotation objective cost(minimize);
   annotation start x(1); annotation bounds x(0,10); expect x==3 tolerance 1e-6;
 }
 test following fixture { dof 0; route steady; procedure solve; } {
   param value:Scalar=2; expect value==2 tolerance 1e-8;
 }
 }"#,
        )
        .await;
        let declarations = p.declarations().await.unwrap();
        let id = |name: &str| {
            declarations
                .iter()
                .find(|row| row.name == name)
                .unwrap()
                .declaration_id
        };
        let cancel = crate::CancelSource::new();
        let defaults = policy();
        let admitted = p
            .declared_execution(
                id("pending"),
                defaults.compiler,
                defaults.solver,
                defaults.numerical,
                defaults.limits,
                &cancel,
            )
            .await
            .unwrap();
        let resolution = p
            .resolve_case(
                id("pending"),
                pse_modeling::specialize::root_instance(id("pending")),
                admitted.analysis.bindings.clone(),
                admitted.analysis.limits,
                admitted.analysis.case.clone(),
                admitted.analysis.order,
                admitted.analysis.compiler,
                admitted.analysis.solver.clone(),
                admitted.analysis.numerical.clone(),
                cases::CaseOverrides::default(),
                false,
                &cancel,
            )
            .await
            .unwrap();
        let decision = resolution.route_decision().unwrap();
        assert_eq!(decision.state, AssessmentState::PendingEvidence);
        assert!(decision.selected.is_none() && decision.structure.is_none());
        assert!(!decision.evidence.is_empty());
        // Distinct candidate modes remain nested route facts; the flat assessment
        // relation has one request/step key and cannot claim an ambiguous common mode.
        use pse_relations::columnar::RelationRow;
        let identity = resolution.admission_identity(&decision).unwrap();
        let tables = analysis_tables::admission_tables(&p.runtime, &decision, identity, 0).unwrap();
        let structural_id =
            pse_relations::generated::runtime::structural_assessments::spec(&p.runtime.registry)
                .unwrap()
                .id;
        assert!(!tables.contains_key(&structural_id));
        let route_id =
            pse_relations::generated::runtime::route_decisions::spec(&p.runtime.registry)
                .unwrap()
                .id;
        let routes =
            pse_relations::generated::runtime::route_decisions::Row::rows(&tables[&route_id])
                .unwrap();
        assert_eq!(routes, vec![decision.row(identity, 0)]);
        for (projected, candidate) in routes[0].eligibility.iter().zip(&decision.eligibility) {
            assert_eq!(projected.backend, candidate.backend);
            assert_eq!(
                projected.structural_mode,
                candidate.structure.as_ref().map(|s| s.mode)
            );
            assert_eq!(
                projected.structurally_admitted,
                candidate.structure.as_ref().map(|s| s.refusal.is_none())
            );
        }
        drop(resolution);
        let report = p.conform(policy(), &cancel).await.unwrap();
        assert!(report.complete);
        assert!(report.passed(), "{:?}", report.checks);
        for fixture in [id("pending"), id("following")] {
            assert_eq!(report.fixture_statuses[&fixture], Status::Passed);
            assert!(report.results[&fixture].accepted);
        }
        assert!(report.failures.is_empty());
    }
}
