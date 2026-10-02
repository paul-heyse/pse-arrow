// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Production-owned admission of the authored route, procedure and execution defaults.
use super::*;
use crate::math::solves::{NumericalInputs, SolverProfile};
use pse_compiler::workspace::{ModelingCaseBindings, Profile};
use pse_model::generated::enums::{ModelingAnalysisRoute as Route, ModelingProcedure as Procedure};
use pse_modeling::specialize::{Fixture, IntegrationFixture, ShootingFixture};
fn authored_fixture(row: &Declaration) -> Option<&pse_model::generated::authored::modeling_declarations::AuthoredModelingDeclarationsFieldValueScopeFixture>{
    row.value
        .scope
        .as_ref()
        .and_then(|scope| scope.fixture.as_ref())
}
/// The specialization limits one fixture runs under: the run's, with each allowance its
/// declared execution policy states (ADR-0119); admission refused a nonpositive one.
pub(super) fn declared_limits(row: &Declaration, run: Limits) -> Result<Limits, WorkflowError> {
    let Some(declared) = authored_fixture(row).and_then(|f| f.policy.as_ref()) else {
        return Ok(run);
    };
    let fixture = row.declaration_id;
    let allowance = |name: &str, value: i64| {
        usize::try_from(value)
            .map_err(|_| contract(format!("fixture {fixture} policy {name} allowance")))
    };
    Ok(Limits {
        items: declared
            .items
            .map_or(Ok(run.items), |v| allowance("items", v))?,
        body_occurrences: declared
            .body_occurrences
            .map(|v| allowance("body_occurrences", v))
            .transpose()?
            .or(run.body_occurrences),
        body_slots: declared
            .body_slots
            .map(|v| allowance("body_slots", v))
            .transpose()?
            .or(run.body_slots),
        ..run
    })
}
/// Resolve a declaration's solver policy once for both conformance and declared analyses.
/// Authored settings override caller defaults only for this fixture.
pub(super) fn declared_solver(
    row: &Declaration,
    run: &SolverProfile,
) -> Result<SolverProfile, WorkflowError> {
    use pse_backend_native::presolve::{Policy as Presolve, PolicyKind};
    let fixture = row.declaration_id;
    let authored = authored_fixture(row);
    let mut solver = run.clone();
    if let Some(intent) = authored.and_then(|f| f.intent) {
        solver.intent = intent;
    }
    if let Some(declared) = authored.and_then(|f| f.policy.as_ref()) {
        if let Some(backend) = declared.backend {
            solver.selection = pse_backend_native::solve::SolverSelection::Explicit(backend);
        }
        match declared.presolve {
            Some(PolicyKind::Off) => solver.presolve = Presolve::Off,
            Some(PolicyKind::Auto) => solver.presolve = Presolve::Auto,
            Some(PolicyKind::Explicit) => {
                return Err(contract(format!(
                    "fixture {fixture} policy presolve is auto or off"
                )));
            }
            None => {}
        }
        for option in &declared.native_options {
            use pse_authoring::language::CellSelected;
            use pse_backend_native::solve::OptionValue;
            let value = match option
                .value
                .value
                .selected()
                .map_err(|error| contract(error.to_string()))?
            {
                CellSelected::Boolean(value) => OptionValue::Bool(value.value),
                CellSelected::Integer(value) => OptionValue::Integer(
                    i32::try_from(value.value)
                        .map_err(|_| contract("native option integer extent"))?,
                ),
                CellSelected::Quantity(value)
                    if value.unit.as_ref().is_none_or(Vec::is_empty)
                        && value.magnitude.is_finite() =>
                {
                    OptionValue::Real(value.magnitude)
                }
                CellSelected::Text(value) => OptionValue::Text(value.value.clone()),
                _ => return Err(contract("native fixture option is not a primitive")),
            };
            if option.value.uncertainty.is_some() {
                return Err(contract("native fixture options have no uncertainty"));
            }
            solver.controls.options.insert(option.name.clone(), value);
        }
        if let Some(bytes) = declared.foreign_bytes {
            solver.controls.foreign_bytes =
                Some(usize::try_from(bytes).map_err(|_| {
                    contract(format!("fixture {fixture} policy foreign allowance"))
                })?);
        }
    }
    Ok(solver)
}
/// One admitted procedure, with its required authored inputs retained.
#[derive(Clone, Debug)]
pub enum DeclaredProcedure {
    /// Evaluate declared observations without solving.
    Check,
    /// Solve the algebraic case on its temporal specialization route.
    Solve,
    /// Run stages/homotopy followed by the unchanged original specification.
    Initialize(ModelingInitialization),
    /// Integrate the authored trajectory and its sample obligations.
    Integrate(IntegrationFixture),
    /// Optimize the authored integration experiment and shooting mesh.
    Shooting {
        /// Authored integration controls.
        integration: IntegrationFixture,
        /// Authored optimization mesh and controls.
        shooting: ShootingFixture,
    },
}
impl DeclaredProcedure {
    /// Registry procedure, derived from the admitted payload.
    pub fn kind(&self) -> Procedure {
        match self {
            Self::Check => Procedure::Check,
            Self::Solve => Procedure::Solve,
            Self::Initialize(_) => Procedure::Initialize,
            Self::Integrate(_) => Procedure::Integrate,
            Self::Shooting { .. } => Procedure::Shooting,
        }
    }
}
/// Rust-owned declaration consumed by execution, initialization, studies and inspection.
#[derive(Clone, Debug)]
pub struct DeclaredExecution {
    /// Temporal route, independent of the procedure.
    pub route: Route,
    /// The admitted authored procedure and its required inputs.
    pub procedure: DeclaredProcedure,
    /// The operation owner's explicit analysis view; it cannot silently change procedure.
    pub analysis: ModelingAnalysis,
    /// Original specialized model retained by nonexecuting inspection.
    pub model: ModelingPreparation,
    /// Requested start choice; applied choices are native StartReceipt/StartSource facts.
    pub requested_start: pse_backend_native::solve::StartPolicy,
}
/// Optional explicit initialization choices. Rust owns every omitted value.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct InitializationOverrides {
    /// Optional stage order, compatible with the authored order.
    pub stages: Option<Vec<String>>,
    /// Optional homotopy enablement.
    pub homotopy: Option<bool>,
    /// Optional initial fraction.
    pub initial_step: Option<f64>,
    /// Optional minimum fraction.
    pub minimum_step: Option<f64>,
    /// Optional growth factor.
    pub growth: Option<f64>,
    /// Optional attempt bound.
    pub maximum_attempts: Option<usize>,
    /// Optional wall-clock bound.
    #[schemars(with = "Option<pse_model::document::ClosedDuration>")]
    pub time_limit: Option<std::time::Duration>,
    /// Optional discrete treatment.
    pub discrete: Option<DiscreteInitialization>,
}
/// Resolve initialization defaults in one production owner for all consumers.
pub(super) fn initialization(
    fixture: Option<&Fixture>,
) -> Result<ModelingInitialization, WorkflowError> {
    let mut plan = ModelingInitialization::default();
    if let Some(fixture) = fixture {
        plan.stages = fixture.stages.clone();
        if let Some(authored) = &fixture.initialization {
            plan.homotopy = authored.homotopy;
            plan.initial_step = authored.initial_step;
            plan.minimum_step = authored.minimum_step;
            plan.growth = authored.growth;
            plan.maximum_attempts = usize::try_from(authored.maximum_attempts)
                .map_err(|_| contract("initialization attempt extent"))?;
            plan.time_limit = std::time::Duration::try_from_secs_f64(authored.time_limit_seconds)
                .map_err(|_| contract("initialization time extent"))?;
        }
    }
    plan.validate()?;
    Ok(plan)
}
impl DeclaredExecution {
    /// Admit a compatible explicit initialization override, preserving authored choices.
    pub fn initialization(
        &self,
        overrides: InitializationOverrides,
    ) -> Result<ModelingInitialization, WorkflowError> {
        let DeclaredProcedure::Initialize(authored) = &self.procedure else {
            return Err(contract(
                "initialization requires the authored initialization procedure",
            ));
        };
        let mut plan = authored.clone();
        let fixture = self
            .model
            .compiled()
            .model
            .fixtures
            .get(&self.analysis.instance);
        let declared = fixture.and_then(|f| f.initialization.as_ref());
        macro_rules! field {
            ($name:ident) => {
                if let Some(value) = overrides.$name {
                    if declared.is_some() && value != authored.$name {
                        return Err(contract(concat!(
                            "initialization override contradicts authored ",
                            stringify!($name)
                        )));
                    }
                    plan.$name = value;
                }
            };
        }
        if let Some(stages) = overrides.stages {
            if fixture.is_some_and(|f| !f.stages.is_empty()) && stages != authored.stages {
                return Err(contract(
                    "initialization override contradicts authored stages",
                ));
            }
            plan.stages = stages;
        }
        field!(homotopy);
        field!(initial_step);
        field!(minimum_step);
        field!(growth);
        field!(maximum_attempts);
        field!(time_limit);
        if let Some(discrete) = overrides.discrete {
            plan.discrete = discrete;
        }
        plan.validate()?;
        Ok(plan)
    }
}
impl ModelingPackage {
    /// Admit the authored route/procedure before any operation executes it.
    pub async fn declared_execution(
        &self,
        root: DeclarationId,
        compiler: Profile,
        solver: SolverProfile,
        numerical: NumericalInputs,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<DeclaredExecution, WorkflowError> {
        let row = self
            .declarations()
            .iter()
            .find(|row| row.declaration_id == root)
            .ok_or_else(|| contract(format!("missing declared execution root {root}")))?;
        let choice = pse_modeling::analysis::declared_policy(root, authored_fixture(row))
            .map_err(super::super::modeling_error)?;
        let limits = declared_limits(row, limits)?;
        let solver = declared_solver(row, &solver)?;
        let requested_start = solver.controls.start;
        let bindings = Bindings::default().with_analysis(choice.route);
        let instance = pse_modeling::specialize::root_instance(root);
        let model = self
            .prepare(root, instance, bindings.clone(), limits, cancel)
            .await?;
        let fixture = model.compiled().model.fixtures.get(&instance);
        let case = fixture.map(ModelingCaseBindings::from).unwrap_or_default();
        let procedure = match choice.procedure {
            Procedure::Check => DeclaredProcedure::Check,
            Procedure::Solve => DeclaredProcedure::Solve,
            Procedure::Initialize => DeclaredProcedure::Initialize(initialization(fixture)?),
            Procedure::Integrate => DeclaredProcedure::Integrate(
                fixture.and_then(|f| f.integration.clone()).ok_or_else(|| {
                    contract("integration procedure requires its authored inputs")
                })?,
            ),
            Procedure::Shooting => DeclaredProcedure::Shooting {
                integration: fixture.and_then(|f| f.integration.clone()).ok_or_else(|| {
                    contract("shooting procedure requires its authored integration")
                })?,
                shooting: fixture
                    .and_then(|f| f.shooting.clone())
                    .ok_or_else(|| contract("shooting procedure requires its authored mesh"))?,
            },
        };
        let analysis = ModelingAnalysis {
            root,
            instance,
            bindings,
            limits,
            case,
            order: solver.derivative_order(),
            compiler,
            solver,
            numerical,
        };
        Ok(DeclaredExecution {
            route: choice.route,
            procedure,
            analysis,
            model,
            requested_start,
        })
    }
    /// Prepare only an admitted algebraic solve procedure; inspection never executes it.
    pub async fn prepare_declared(
        &self,
        execution: &DeclaredExecution,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        if !matches!(execution.procedure, DeclaredProcedure::Solve) {
            return Err(contract(
                "prepared algebraic solve requires the authored solve procedure",
            ));
        }
        self.prepare_analysis(&execution.analysis, cancel).await
    }
    /// Execute the admitted algebraic procedure, including its authored initialization.
    pub async fn execute_declared(
        &self,
        execution: &DeclaredExecution,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingResult, WorkflowError> {
        match &execution.procedure {
            DeclaredProcedure::Solve => {
                self.solve_case(
                    self.prepare_analysis(&execution.analysis, cancel).await?,
                    execution.analysis.compiler,
                    cancel,
                )
                .await
            }
            DeclaredProcedure::Initialize(plan) => {
                let report = self
                    .initialize_model(&execution.analysis, plan.clone(), cancel)
                    .await?;
                if !report.completed {
                    return Err(contract(format!(
                        "declared initialization failed: {:?}",
                        report.failure
                    )));
                }
                report
                    .attempts
                    .last()
                    .and_then(|a| a.result.as_ref().ok())
                    .cloned()
                    .ok_or_else(|| contract("declared initialization has no original result"))
            }
            _ => Err(contract(
                "algebraic execution requires the authored solve or initialization procedure",
            )),
        }
    }
    /// Execute an explicitly admitted compatible initialization policy.
    pub async fn initialize_declared(
        &self,
        execution: &DeclaredExecution,
        overrides: InitializationOverrides,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingInitializationReport, WorkflowError> {
        let policy = execution.initialization(overrides)?;
        self.initialize_model(&execution.analysis, policy, cancel)
            .await
    }
    /// Fitting may retain an explicit route only when it agrees with the authored case.
    pub(in crate::workflow) async fn declared_case(
        &self,
        root: DeclarationId,
        route: Route,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<(Bindings, ModelingCaseBindings), WorkflowError> {
        let execution = self
            .declared_execution(
                root,
                Default::default(),
                Default::default(),
                Default::default(),
                limits,
                cancel,
            )
            .await?;
        if route != execution.route {
            return Err(contract(
                "explicit route override contradicts authored temporal route",
            ));
        }
        Ok((execution.analysis.bindings, execution.analysis.case))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn declared_simultaneous_initialization_retains_procedure_and_admits_overrides_once() {
        let source = "package p { def D { var x:Scalar; eq e:x==1; } test initialized fixture { dof 0; route simultaneous; procedure initialize; initialize homotopy(false) step(0.25) minimum(1e-6) growth(1.5) attempts(128) seconds(60); } { child root:D=D(); } test solving fixture { dof 0; route simultaneous; procedure solve; } { child root:D=D(); } }";
        let rows = pse_authoring::language::parse(
            source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|r| r.name == "initialized")
            .unwrap()
            .declaration_id;
        let solving = rows
            .iter()
            .find(|r| r.name == "solving")
            .unwrap()
            .declaration_id;
        let runtime = crate::workflow::tests::runtime();
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .unwrap();
        let cancel = crate::CancelSource::new();
        let compiler = crate::workflow::tests::compiler_profile();
        let execution = package
            .declared_execution(
                root,
                compiler,
                Default::default(),
                Default::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(execution.route, Route::Simultaneous);
        assert_eq!(execution.procedure.kind(), Procedure::Initialize);
        let omitted = execution.initialization(Default::default()).unwrap();
        let explicit = execution
            .initialization(InitializationOverrides {
                homotopy: Some(omitted.homotopy),
                initial_step: Some(omitted.initial_step),
                maximum_attempts: Some(omitted.maximum_attempts),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(omitted.initial_step, explicit.initial_step);
        assert_eq!(omitted.maximum_attempts, explicit.maximum_attempts);
        assert!(
            execution
                .initialization(InitializationOverrides {
                    initial_step: Some(0.5),
                    ..Default::default()
                })
                .is_err()
        );
        assert!(package.prepare_declared(&execution, &cancel).await.is_err());
        let solving = package
            .declared_execution(
                solving,
                compiler,
                Default::default(),
                Default::default(),
                Limits::default(),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(solving.procedure.kind(), Procedure::Solve);
        assert!(solving.initialization(Default::default()).is_err());
    }
}
