// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Lexicographic multi-objective solves (ADR-0111; Plan 22 C3, improvement I15).
//!
//! A model with several objective levels is optimized level by level, each earlier level
//! held within its degradation of its optimum f*: f ≤ f* + max(abs, rel·|f*|) when
//! minimized, f ≥ f* − max(abs, rel·|f*|) when maximized. The route follows from the
//! class (PS-09):
//!
//! - **Native.** LP and MILP levels go to HiGHS as one solve over a case structure with
//!   every level's objective (`Highs_passLinearObjectives`, blending off). HiGHS bounds an
//!   earlier level by the tighter of its two tolerances, so the native route takes only
//!   levels whose bound it states exactly: one of the two tolerances zero.
//! - **Staged.** Every other level runs as one step of a staged sequence ([`Staged`]): the
//!   `objective.level` fact selects the level, whose structure bounds every earlier level
//!   by a generated parameter β (`objective_bounds`). Bounds are structure and β values, so
//!   K levels prepare at most K structures and a repeated solve rebinds values only. Each
//!   step starts from the previous level's accepted step. A staged level needs a positive
//!   degradation: an exactly active objective bound breaks the constraint qualification an
//!   interior-point method needs (ADR-0111 item 3).
//!
//! Level values are the weighted sums of the members' terms, observed through the model
//! at each step's solved values in original coordinates, and recorded in
//! `runtime.objective_levels`.
use super::{
    ModelingAnalysis, ModelingPackage, ModelingResult, Runtime, WorkflowError, contract,
    modeling::{assessment::Obligations, cases::CaseOverrides},
    modeling_error, relation,
    staged::{Overlay, Staged, Start},
};
use pse_compiler::workspace::ModelingOutput;
use pse_ids::SemanticId;
use pse_model::diagnostic::BoundaryDiagnostic;
use pse_model::generated::{
    enums::{ModelingObjectiveRoute, NativeObjectiveSense},
    identities::RunId,
};
use pse_modeling::{
    ObjectiveRefusal,
    annotation::ObjectiveSense,
    specialize::{ObjectiveLevel, ObjectiveMember, Value},
};
use pse_relations::columnar::FieldCheckedBatch;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// One level of a lexicographic solve.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelingObjectiveLevel {
    /// Position of the level in optimization order.
    pub level: usize,
    /// Authored priority, when declared.
    pub priority: Option<i64>,
    /// Orientation of the level value.
    pub sense: ObjectiveSense,
    /// Canonical physical type of the level value.
    pub quantity: pse_quantity::QuantityTypeId,
    /// The staged step that optimized the level; the one native step on the native route.
    pub attempt: usize,
    /// The level's optimum at its own step; staged route only.
    pub optimum: Option<f64>,
    /// The bound every later staged step held the level to.
    pub bound: Option<f64>,
    /// The level's value at the final candidate.
    pub value: Option<f64>,
}
/// A lexicographic solve's levels and every step it ran, including a failed last one.
#[derive(Clone, Debug)]
pub struct ModelingLevelsReport {
    /// Identity of this solve.
    pub run_id: RunId,
    runtime: Runtime,
    /// Native lexicographic, or one staged step per level.
    pub route: ModelingObjectiveRoute,
    /// Every level, in optimization order.
    pub levels: Vec<ModelingObjectiveLevel>,
    /// Every step in order: one on the native route, one per level reached when staged.
    pub steps: Vec<Result<ModelingResult, Arc<WorkflowError>>>,
    /// Every level was optimized and the final candidate is a result.
    pub completed: bool,
    /// Why the solve stopped before completing.
    pub failure: Option<BoundaryDiagnostic>,
}
impl ModelingLevelsReport {
    /// The result of the final step when the solve completed: the lexicographic optimum.
    pub fn result(&self) -> Option<&ModelingResult> {
        self.completed
            .then(|| self.steps.last().and_then(|s| s.as_ref().ok()))
            .flatten()
    }
    /// The levels as a checked `objective_levels` relation.
    ///
    /// # Errors
    /// A level that does not fit the relation.
    pub fn table(&self) -> Result<FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::objective_levels as rows;
        let mut builder = rows::Builder::with_registry(&self.runtime.registry, self.levels.len())
            .map_err(relation)?;
        for level in &self.levels {
            builder
                .push(rows::Row {
                    run_id: self.run_id,
                    level: i64::try_from(level.level).map_err(|_| contract("objective level"))?,
                    priority: level.priority,
                    sense: match level.sense {
                        ObjectiveSense::Minimize => NativeObjectiveSense::Minimize,
                        ObjectiveSense::Maximize => NativeObjectiveSense::Maximize,
                    },
                    quantity_id: level.quantity.as_id(),
                    route: self.route,
                    attempt: i64::try_from(level.attempt)
                        .map_err(|_| contract("objective level attempt"))?,
                    result_id: self
                        .steps
                        .get(level.attempt)
                        .and_then(|s| s.as_ref().ok())
                        .map(|r| r.run_id),
                    optimum: level.optimum,
                    bound: level.bound,
                    value: level.value,
                })
                .map_err(relation)?;
        }
        builder.finish().map_err(relation)
    }
}

/// The objectives a lexicographic solve optimizes.
struct Levels {
    levels: Vec<ObjectiveLevel>,
    members: Vec<ObjectiveMember>,
}
impl Levels {
    /// Every level but the last states a degradation the native route states exactly:
    /// one of its tolerances is zero.
    fn single_nonzero(&self) -> bool {
        self.levels.len() > 1
            && self.levels[..self.levels.len() - 1]
                .iter()
                .all(|l| l.absolute_tolerance == Some(0.0) || l.relative_tolerance == Some(0.0))
    }
    /// The output rows of every member's term, which level values are observed through.
    fn rows(&self) -> BTreeSet<SemanticId> {
        self.members
            .iter()
            .map(|m| ModelingOutput::Member(m.term).row_id())
            .collect()
    }
    /// Each level's value, Σ scale·term, from observed terms.
    fn values(&self, observed: &BTreeMap<SemanticId, f64>) -> Result<Vec<f64>, WorkflowError> {
        self.levels
            .iter()
            .map(|level| {
                level.members.iter().try_fold(0.0, |sum, &m| {
                    let member = &self.members[m];
                    let term = observed
                        .get(&ModelingOutput::Member(member.term).row_id())
                        .ok_or_else(|| contract("objective term not observed"))?;
                    Ok(sum + member.scale * term)
                })
            })
            .collect()
    }
    /// The staged refusal of a level whose degradation is zero (ADR-0111 item 3).
    fn zero_tolerance(&self, level: usize) -> WorkflowError {
        let lead = &self.members[self.levels[level].members[0]];
        modeling_error(pse_modeling::ModelingError::Objective {
            declaration: lead.lineage.declaration.as_id(),
            reason: ObjectiveRefusal::ZeroTolerance,
        })
    }
}

/// An incumbent permission does not establish an earlier priority's optimum.
fn priority_completed(result: &ModelingResult) -> bool {
    use pse_backend_native::solve::{Assurance, Termination};
    result.accepted
        && match &result.outcome {
            crate::math::solves::Outcome::Constant(_) => true,
            crate::math::solves::Outcome::Native(report) => {
                matches!(
                    report.termination.category,
                    Termination::Success | Termination::Acceptable
                ) && matches!(
                    report.termination.assurance,
                    Assurance::LocalStationary
                        | Assurance::NativeOptimal
                        | Assurance::ExactCertificate
                )
            }
            _ => false,
        }
}

fn priority_failure() -> BoundaryDiagnostic {
    BoundaryDiagnostic::new(
        pse_model::diagnostic::BoundaryClass::Numerical,
        "objective-levels",
        [],
        "objective.priority_optimization_incomplete",
    )
}

impl ModelingPackage {
    /// Optimize authored priorities using the admitted native capability or one admitted
    /// staged backend, preserving each authored degradation bound. A model with one level is one
    /// solve. The analysis names no `objective.level`; the solve owns level selection.
    ///
    /// # Errors
    /// A selected level in the analysis, a model without objectives, a zero degradation on
    /// a staged level (before any step, or at the optimum of its step), or a failure to
    /// prepare the first step. A failed later step ends the report instead.
    pub async fn optimize_levels(
        &self,
        analysis: &ModelingAnalysis,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingLevelsReport, WorkflowError> {
        if analysis
            .bindings
            .facts
            .contains_key(&pse_modeling::analysis::Fact::ObjectiveLevel)
        {
            return Err(contract(
                "a lexicographic solve owns level selection; the analysis names no objective level",
            ));
        }
        let base = self
            .prepare(
                analysis.root,
                analysis.instance,
                analysis.bindings.clone(),
                analysis.limits,
                cancel,
            )
            .await?;
        let objectives = &base.compiled().model.objectives;
        let levels = Levels {
            levels: objectives.levels.clone(),
            members: objectives.members.clone(),
        };
        if levels.levels.is_empty() {
            return Err(contract("a lexicographic solve needs an objective"));
        }
        let mut staged = Staged::open(&self.runtime, None)?;
        let (decision, mut resolution) = self
            .level_route(analysis, levels.single_nonzero(), cancel)
            .await?;
        let mut selected = analysis.clone();
        if let pse_backend_native::routing::Route::Native(backend) = decision
            .route()
            .map_err(crate::math::MathRuntimeError::from)?
        {
            selected.solver.selection =
                pse_backend_native::solve::SolverSelection::Explicit(backend);
            resolution.solver.selection = selected.solver.selection;
        }
        let report =
            if decision.lexicographic == Some(pse_backend_native::routing::Lexicographic::Native) {
                let prepared = self.finish_case(resolution).await?;
                self.native_solve(&mut staged, &levels, prepared, cancel)
                    .await
            } else {
                self.staged_solve(&mut staged, &selected, &levels, cancel)
                    .await
            };
        staged.close().await;
        report
    }
    /// Query declared native priority semantics, otherwise admit one backend for staging.
    async fn level_route(
        &self,
        analysis: &ModelingAnalysis,
        single_nonzero: bool,
        cancel: &crate::CancelSource,
    ) -> Result<
        (
            pse_backend_native::routing::Decision,
            super::modeling::cases::ModelingCaseResolution,
        ),
        WorkflowError,
    > {
        let resolution = self
            .resolve_case(
                analysis.root,
                analysis.instance,
                analysis.bindings.clone(),
                analysis.limits,
                analysis.case.clone(),
                analysis.order,
                analysis.compiler,
                analysis.solver.clone(),
                analysis.numerical.clone(),
                CaseOverrides::default(),
                false,
                cancel,
            )
            .await?;
        let facts = &resolution.model.case.compiled().facts;
        let requirements = pse_backend_native::routing::Requirements {
            table: &pse_backend_native::execution::LINKED,
            facts,
            intent: resolution.solver.intent,
            numerical_psd: false,
            least_squares: false,
            controls: &resolution.solver.controls,
            settings: &resolution.solver.backend,
            sensitivity: resolution.solver.sensitivity.is_some(),
        };
        let decision = requirements.lexicographic(analysis.solver.selection, single_nonzero);
        decision
            .route()
            .map_err(crate::math::MathRuntimeError::from)?;
        Ok((decision, resolution))
    }
    async fn native_solve(
        &self,
        staged: &mut Staged,
        levels: &Levels,
        prepared: super::ModelingSolvePreparation,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingLevelsReport, WorkflowError> {
        let run_id = pse_operations::mint_id();
        let result = staged
            .run(prepared, Obligations::Final, run_id, 0, None, cancel)
            .await;
        let mut report = self.report(ModelingObjectiveRoute::Native, levels);
        report.steps.push(result);
        let values = match report.steps[0].as_ref() {
            Ok(result) if priority_completed(result) => {
                Some(self.level_values(levels, result, cancel).await?)
            }
            _ => None,
        };
        for (level, value) in report.levels.iter_mut().zip(values.iter().flatten()) {
            level.value = Some(*value);
        }
        report.completed = values.is_some();
        report.failure =
            failure(&report.steps[0]).or_else(|| (!report.completed).then(priority_failure));
        Ok(report)
    }
    async fn staged_solve(
        &self,
        staged: &mut Staged,
        analysis: &ModelingAnalysis,
        levels: &Levels,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingLevelsReport, WorkflowError> {
        let last = levels.levels.len() - 1;
        // A declared zero degradation is certainly zero at every optimum.
        if let Some(level) = levels.levels[..last]
            .iter()
            .position(|l| l.absolute_tolerance == Some(0.0) && l.relative_tolerance == Some(0.0))
        {
            return Err(levels.zero_tolerance(level));
        }
        // The bound parameter β of every earlier level, as `objective_bounds` generated it
        // for the last level's structure.
        let parameters = if last == 0 {
            Vec::new()
        } else {
            let model = self
                .prepare(
                    analysis.root,
                    analysis.instance,
                    analysis.bindings.clone().with_objective_level(last),
                    analysis.limits,
                    cancel,
                )
                .await?;
            model.compiled().model.objectives.levels[..last]
                .iter()
                .map(|l| {
                    l.bound
                        .as_ref()
                        .map(|b| b.parameter)
                        .ok_or_else(|| contract("objective bound parameter"))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        // Admit every priority's original bound view before the first numerical attempt.
        // Earlier beta values are finite RHS placeholders here; only optimization completion supplies actual bounds.
        for position in 0..=last {
            let bindings = if last == 0 {
                analysis.bindings.clone()
            } else {
                analysis.bindings.clone().with_objective_level(position)
            };
            let resolution = self
                .resolve_case(
                    analysis.root,
                    analysis.instance,
                    bindings,
                    analysis.limits,
                    analysis.case.clone(),
                    analysis.order,
                    analysis.compiler,
                    analysis.solver.clone(),
                    analysis.numerical.clone(),
                    CaseOverrides {
                        parameters: parameters[..position].iter().map(|id| (*id, 0.0)).collect(),
                        ..Default::default()
                    },
                    false,
                    cancel,
                )
                .await?;
            let decision = resolution.route_decision()?;
            decision
                .route()
                .map_err(crate::math::MathRuntimeError::from)?;
        }
        let mut report = self.report(ModelingObjectiveRoute::Staged, levels);
        let mut bounds = BTreeMap::new();
        let mut accepted = None;
        for position in 0..=last {
            let overlay = Overlay {
                facts: if last == 0 {
                    BTreeMap::new()
                } else {
                    BTreeMap::from([(
                        pse_modeling::analysis::Fact::ObjectiveLevel,
                        Value::Integer(
                            i64::try_from(position).map_err(|_| contract("objective level"))?,
                        ),
                    )])
                },
                parameters: bounds.clone(),
                ..Overlay::default()
            };
            let record = staged
                .step(
                    self,
                    analysis,
                    &overlay,
                    accepted.map_or(Start::Specification, Start::Accepted),
                    Obligations::Final,
                    None,
                    "objective-levels",
                    cancel,
                )
                .await;
            let step = report.steps.len();
            report.levels[position].attempt = step;
            let result = match &record.result {
                Ok(result) if record.interruption.is_none() && priority_completed(result) => {
                    result.clone()
                }
                _ => {
                    report.failure = record
                        .interruption
                        .clone()
                        .or_else(|| failure(&record.result))
                        .or_else(|| Some(priority_failure()));
                    report.steps.push(record.result);
                    return Ok(report);
                }
            };
            report.steps.push(record.result);
            accepted = Some(step);
            let values = self.level_values(levels, &result, cancel).await?;
            let optimum = values[position];
            report.levels[position].optimum = Some(optimum);
            if position == last {
                for (level, value) in report.levels.iter_mut().zip(values) {
                    level.value = Some(value);
                }
                break;
            }
            let level = &levels.levels[position];
            let degradation = pse_math::binding::Degradation {
                absolute: level.absolute_tolerance.unwrap_or(0.0),
                relative: level.relative_tolerance.unwrap_or(0.0),
            }
            .at(optimum);
            // A degradation within the accuracy the optimum is known to is an exactly
            // active bound (ADR-0111 item 3).
            // An incomparable (NaN) degradation is refused like a zero one.
            if degradation.partial_cmp(&result.prepared.solve.objective_accuracy())
                != Some(std::cmp::Ordering::Greater)
            {
                return Err(levels.zero_tolerance(position));
            }
            let beta = match level.sense {
                ObjectiveSense::Minimize => optimum + degradation,
                ObjectiveSense::Maximize => optimum - degradation,
            };
            report.levels[position].bound = Some(beta);
            bounds.insert(parameters[position], beta);
        }
        report.completed = true;
        Ok(report)
    }
    fn report(&self, route: ModelingObjectiveRoute, levels: &Levels) -> ModelingLevelsReport {
        ModelingLevelsReport {
            run_id: pse_operations::mint_id(),
            runtime: self.runtime.clone(),
            route,
            levels: levels
                .levels
                .iter()
                .enumerate()
                .map(|(level, l)| ModelingObjectiveLevel {
                    level,
                    priority: l.priority,
                    sense: l.sense,
                    quantity: l.quantity,
                    attempt: 0,
                    optimum: None,
                    bound: None,
                    value: None,
                })
                .collect(),
            steps: Vec::new(),
            completed: false,
            failure: None,
        }
    }
    /// Every level's value at `result`, observed through the model at its solved values.
    async fn level_values(
        &self,
        levels: &Levels,
        result: &ModelingResult,
        cancel: &crate::CancelSource,
    ) -> Result<Vec<f64>, WorkflowError> {
        let observed = self
            .observe(
                result.prepared.model.model.clone(),
                levels.rows(),
                result.values.clone(),
                result.prepared.compiler,
                cancel,
            )
            .await?;
        levels.values(&observed)
    }
}
/// Why a step's candidate is not a result.
fn failure(result: &Result<ModelingResult, Arc<WorkflowError>>) -> Option<BoundaryDiagnostic> {
    match result {
        Ok(result) => result.diagnostic(),
        Err(error) => Some(error.boundary_diagnostic()),
    }
}

#[cfg(test)]
#[cfg(all(feature = "solver-highs", feature = "solver-ipopt"))]
mod tests {
    use super::*;
    use crate::math::solves::{NumericalInputs, Outcome};
    use crate::workflow::tests as fixture;
    use pse_backend_native::solve::{Backend, SolveIntent, SolverSelection};
    use pse_compiler::workspace::ModelingCaseBindings;
    use pse_modeling::{Bindings, Limits};
    use pse_relations::columnar::RelationRow;

    fn package(text: &str) -> (ModelingPackage, ModelingAnalysis) {
        let physical = fixture::physical();
        let rows = pse_authoring::language::parse(
            text,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = fixture::runtime().modeling_package(rows, physical).unwrap();
        let mut solver = fixture::profile();
        solver.intent = SolveIntent::Optimize;
        solver.selection = SolverSelection::Auto;
        let analysis = ModelingAnalysis {
            root,
            instance: pse_modeling::specialize::root_instance(root),
            bindings: Bindings::default(),
            limits: Limits::default(),
            case: ModelingCaseBindings::default(),
            order: pse_kernels::DerivativeOrder::Second,
            compiler: fixture::compiler_profile(),
            solver,
            numerical: NumericalInputs::default(),
        };
        (package, analysis)
    }
    fn symbol(result: &ModelingResult, name: &str) -> SemanticId {
        result
            .prepared
            .model
            .model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path.rsplit('.').next() == Some(name))
            .unwrap()
            .id
    }
    fn value(result: &ModelingResult, name: &str) -> f64 {
        result.values.scalars[&symbol(result, name)]
    }
    fn backend(result: &ModelingResult) -> Backend {
        match &result.outcome {
            Outcome::Native(native) => native.backend,
            other => panic!("{other:?}"),
        }
    }
    async fn solve(package: &ModelingPackage, analysis: &ModelingAnalysis) -> ModelingLevelsReport {
        let report = package
            .optimize_levels(analysis, &crate::CancelSource::new())
            .await
            .unwrap();
        assert!(report.completed, "{:?}", report.failure);
        report
    }
    fn levels(report: &ModelingLevelsReport) -> Vec<f64> {
        report.levels.iter().map(|l| l.value.unwrap()).collect()
    }

    /// `max x + y` then `min x` over integer x, y ∈ [0, 10] with x + y ≤ 4 W and y ≤ 3 W:
    /// lexicographically x = 1, y = 3. A degradation of 1 W on the first level admits
    /// x + y = 3, so the second level reaches x = 0.
    fn milp(absolute: &str) -> String {
        format!(
            "package p {{ def Root {{
            param unit: Power = 1{{W}};
            var x: Count in integer; var y: Count in integer;
            eq total: unit*x + unit*y <= 4{{W}};
            eq cap: unit*y <= 3{{W}};
            let sum: Power = unit*x + unit*y;
            let left: Power = unit*x;
            annotation objective sum(maximize, priority = 0, absolute_tolerance = {absolute}, relative_tolerance = 0);
            annotation objective left(minimize, priority = 1);
            annotation bounds x(0{{1}}, 10{{1}}); annotation bounds y(0{{1}}, 10{{1}});
            annotation start x(0{{1}}); annotation start y(0{{1}}); }} }}"
        )
    }

    /// LP and MILP levels are one native HiGHS solve over every level's objective, with the
    /// declared degradation, a zero one included.
    #[tokio::test]
    async fn lexicographic_milp_native() {
        for (absolute, x, y, values) in [
            ("0{W}", 1.0, 3.0, [4.0, 1.0]),
            ("1{W}", 0.0, 3.0, [3.0, 0.0]),
        ] {
            let (package, analysis) = package(&milp(absolute));
            let report = solve(&package, &analysis).await;
            assert_eq!(report.route, ModelingObjectiveRoute::Native);
            assert_eq!(report.steps.len(), 1);
            let result = report.result().unwrap();
            assert_eq!(backend(result), Backend::Highs);
            assert!((value(result, "x") - x).abs() < 1e-9, "{absolute}");
            assert!((value(result, "y") - y).abs() < 1e-9, "{absolute}");
            for (actual, expected) in levels(&report).iter().zip(values) {
                assert!((actual - expected).abs() < 1e-9, "{absolute}: {actual}");
            }
            // The native route states no per-level optimum or bound of its own.
            assert!(
                report
                    .levels
                    .iter()
                    .all(|l| l.optimum.is_none() && l.bound.is_none())
            );
            let rows = pse_relations::generated::runtime::objective_levels::Row::rows(
                &report.table().unwrap(),
            )
            .unwrap();
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].sense, NativeObjectiveSense::Maximize);
            assert_eq!(rows[1].result_id, Some(result.run_id));
        }
    }

    /// `min (x + y − 2)²` then `min x² + 2y²`, each level optimizing within the previous
    /// one's degradation.
    fn nlp(first: &str, absolute: &str, relative: &str) -> String {
        format!(
            "package p {{ def Root {{
            var x: Scalar; var y: Scalar;
            let miss: Scalar = {first};
            let effort: Scalar = x*x + 2*y*y;
            annotation objective miss(minimize, priority = 0, absolute_tolerance = {absolute}, relative_tolerance = {relative});
            annotation objective effort(minimize, priority = 1);
            annotation bounds x(-10, 10); annotation bounds y(-10, 10);
            annotation start x(0); annotation start y(0); }} }}"
        )
    }
    const MISS: &str = "(x + y - 2)*(x + y - 2)";

    /// A quadratic level is staged. As the degradation vanishes the staged optimum tends to
    /// the limit of the weighted sum w·miss + effort as w grows: x + y = 2 at the least
    /// effort, x = 4/3 and y = 2/3.
    #[tokio::test]
    async fn lexicographic_nlp_staged_matches_weighted_limit() {
        let (package, analysis) = package(&nlp(MISS, "1e-6", "0"));
        let report = solve(&package, &analysis).await;
        assert_eq!(report.route, ModelingObjectiveRoute::Staged);
        assert_eq!(report.steps.len(), 2);
        let result = report.result().unwrap();
        let (x, y) = (value(result, "x"), value(result, "y"));
        assert!(
            (x - 4.0 / 3.0).abs() < 2e-3 && (y - 2.0 / 3.0).abs() < 2e-3,
            "{x} {y}"
        );
        // A single weighted objective with a large weight on the first level agrees.
        let weighted = "package p { def Root {
            var x: Scalar; var y: Scalar;
            let blend: Scalar = 1e5*(x + y - 2)*(x + y - 2) + x*x + 2*y*y;
            annotation objective blend(minimize);
            annotation bounds x(-10, 10); annotation bounds y(-10, 10);
            annotation start x(0); annotation start y(0); } }";
        let (blended, mut analysis) = self::package(weighted);
        analysis.solver.selection = SolverSelection::Explicit(Backend::Ipopt);
        let limit = solve(&blended, &analysis).await;
        let limit = limit.result().unwrap();
        assert!((value(limit, "x") - x).abs() < 2e-3 && (value(limit, "y") - y).abs() < 2e-3);
    }

    /// T12: every later level holds an earlier one within f* + max(abs, rel·|f*|); the
    /// second level pulls the first to exactly that bound. An absolute degradation governs
    /// at f* = 0 and a relative one at f* = 1.
    #[tokio::test]
    async fn lexicographic_degradation_tolerance_respected() {
        let shifted = format!("{MISS} + 1");
        for (first, absolute, relative, optimum, bound) in [
            (MISS, "0.01", "0", 0.0, 0.01),
            (shifted.as_str(), "0.001", "0.05", 1.0, 1.05),
        ] {
            let (package, analysis) = package(&nlp(first, absolute, relative));
            let report = solve(&package, &analysis).await;
            assert_eq!(report.route, ModelingObjectiveRoute::Staged);
            let level = &report.levels[0];
            assert!((level.optimum.unwrap() - optimum).abs() < 1e-6, "{level:?}");
            assert!((level.bound.unwrap() - bound).abs() < 1e-6, "{level:?}");
            let value = level.value.unwrap();
            assert!(
                value <= bound + 1e-6 && value >= bound - 1e-4,
                "{value} {bound}"
            );
            assert_eq!(
                (report.levels[1].bound, report.levels[1].attempt),
                (None, 1)
            );
        }
    }

    /// A staged level with a zero degradation is refused: declared zero before any step,
    /// or zero at the optimum of its step, within the accuracy that optimum is known to (a
    /// relative tolerance at f* = 0).
    #[tokio::test]
    async fn zero_tolerance_refused_on_staged_level() {
        let refused = |error: WorkflowError| {
            let diagnostic = error.boundary_diagnostic();
            assert_eq!(
                diagnostic.class,
                pse_model::diagnostic::BoundaryClass::Unsupported,
                "{error}"
            );
            assert!(error.to_string().contains("tolerance"), "{error}");
        };
        let cancel = crate::CancelSource::new();
        for (absolute, relative) in [("0", "0"), ("0", "0.1")] {
            let (package, analysis) = package(&nlp(MISS, absolute, relative));
            refused(
                package
                    .optimize_levels(&analysis, &cancel)
                    .await
                    .unwrap_err(),
            );
        }
        // The native route admits a zero degradation (lexicographic_milp_native).
    }

    /// Each level's structure is prepared once: bounds are structure and β values, so a
    /// repeated solve with other values rebinds every step without a new preparation.
    #[tokio::test]
    async fn objective_levels_prepare_once_per_level() {
        let (package, mut analysis) = package(
            &nlp(MISS, "0.01", "0")
                .replace("let miss", "param target: Scalar = 2; let miss")
                .replace("(x + y - 2)", "(x + y - target)"),
        );
        analysis.bindings.demand = vec!["target".into()];
        let native = package.runtime.native();
        let before = native.preparations();
        let first = solve(&package, &analysis).await;
        let after_first = native.preparations();
        // The two level structures and the lexicographic probe.
        assert!(
            after_first.views - before.views <= 3,
            "{:?}",
            after_first.views - before.views
        );
        let keys = first
            .steps
            .iter()
            .map(|s| {
                s.as_ref()
                    .unwrap()
                    .prepared
                    .model
                    .case
                    .compiled()
                    .plan
                    .structure()
                    .key()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(keys.len(), 2);
        analysis.case.values.insert("target".into(), 3.0);
        let second = solve(&package, &analysis).await;
        let after_second = native.preparations();
        assert_eq!(after_second.views, after_first.views);
        assert!(after_second.rebuilt > after_first.rebuilt);
        // x + y = 3 − 0.1 at the least effort, x = 2y.
        let result = second.result().unwrap();
        assert!((value(result, "x") - 2.0 * value(result, "y")).abs() < 1e-4);
        assert!((value(result, "x") + value(result, "y") - 2.9).abs() < 1e-4);
    }
}
