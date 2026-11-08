// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared operation admission; hosts supply transport and their original cancellation owner.
use super::{
    AdmittedBinding, AdmittedHorizonValues, DeclaredExecution, DeclaredProcedure,
    ExecutionServices, ModelingPackage, ModelingSolvePreparation, OperationRequest,
    OperationSource, PreparationServices, PreparationSettings, PreparedStudyOperation, RunHandle,
    WorkflowError, contract,
};
use crate::{CancelSource, math::settings::SolveSettings};
use pse_model::generated::{enums::ModelingAnalysisRoute, identities::DeclarationId};

/// Admitted initialization retains the original specification before any numerical start exists.
#[derive(Clone, Debug)]
pub struct PreparedInitializationOperation {
    pub(in crate::workflow) source: ModelingPackage,
    pub(in crate::workflow) execution: DeclaredExecution,
    identity: pse_ids::ContentHash,
}
impl PreparedInitializationOperation {
    pub(in crate::workflow) fn new(
        source: ModelingPackage,
        execution: DeclaredExecution,
    ) -> Result<Self, WorkflowError> {
        let DeclaredProcedure::Initialize(policy) = &execution.procedure else {
            return Err(contract(
                "initialization operation requires its admitted procedure",
            ));
        };
        execution.prepared_model(&source)?;
        let constructor = super::ReplayCaseConstructor::from_analysis(0, &execution.analysis)?;
        let providers = source
            .registrations()
            .into_iter()
            .map(|(key, provider)| (key.0, provider.configuration_key()))
            .collect::<Vec<_>>();
        let identity = pse_ids::document::of(
            pse_ids::Frame::DurableInitializationRequestV1,
            &(
                source.revision.identity(),
                source.physical.identity(),
                constructor,
                policy,
                &execution.requested_start,
                providers,
            ),
        )
        .map_err(|error| contract(error.to_string()))?;
        Ok(Self {
            source,
            execution,
            identity,
        })
    }
    /// Exact original operation constructor, not the semantic question or preparation key.
    pub fn identity(&self) -> pse_ids::ContentHash {
        self.identity
    }
    pub(in crate::workflow) fn policy(
        &self,
    ) -> Result<&super::ModelingInitialization, WorkflowError> {
        match &self.execution.procedure {
            DeclaredProcedure::Initialize(policy) => Ok(policy),
            _ => Err(contract("initialization procedure absent")),
        }
    }
}

/// Host intent. Prepared shooting handles have no serialized study representation.
#[derive(Clone, Debug)]
pub enum RuntimeOperationRequest {
    /// A declaration whose route is resolved by runtime admission.
    DeclaredCase {
        /// Selected authored case.
        case: DeclarationId,
        /// Complete native controls.
        settings: Box<SolveSettings>,
    },
    /// The unchanged study-admitted operation vocabulary and its contextual horizon values.
    Study {
        /// Scientific request, independent of occurrence identity and output selection.
        operation: Box<OperationRequest>,
        /// Explicit requested result retention, independent of occurrence identity.
        output: super::StudyOutputDemand,
        /// Already admitted physical horizon assignments, when present.
        admitted_horizon: Option<AdmittedHorizonValues>,
    },
    /// Existing host-only shooting capability.
    #[cfg(feature = "solver-diffsol")]
    Shooting {
        /// Admitted immutable problem.
        problem: std::sync::Arc<super::ShootingProblem>,
        /// Explicit original coordinates, or the problem's admitted defaults.
        initial: Option<Vec<f64>>,
    },
}
fn projection_refusal(reason: &str) -> WorkflowError {
    projection_refusal_at(reason, None)
}
fn projection_refusal_at(reason: &str, case: Option<DeclarationId>) -> WorkflowError {
    let mut diagnostic = pse_model::diagnostic::BoundaryDiagnostic::new(
        pse_model::diagnostic::BoundaryClass::Unsupported,
        pse_diagnostics::DiagnosticStage::StudyAdmission,
        case.map(|id| id.as_id()),
        pse_diagnostics::DiagnosticRule::StudyOperationUnsupported,
    );
    diagnostic.observations.insert(
        "reason".into(),
        pse_model::diagnostic::Observation::Text(reason.into()),
    );
    diagnostic.into()
}
impl RuntimeOperationRequest {
    /// Project only the admitted persisted vocabulary, before any durable effect.
    /// # Errors
    /// Host-only work and an unresolved declared route have no study descriptor.
    pub fn into_study(self) -> Result<OperationRequest, WorkflowError> {
        match self {
            Self::Study { operation, .. } => Ok(*operation),
            Self::DeclaredCase { .. } => Err(projection_refusal(
                "study projection requires the runtime-admitted declared route",
            )),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { .. } => Err(projection_refusal(
                "shooting is a host capability, outside StudyOperation Version<8>",
            )),
        }
    }
}

/// One prepared host operation, composed from the shared study subset and host-only work.
#[derive(Clone, Debug)]
pub enum PreparedOperation {
    /// Original authored initialization; native recorded runs retain its full report.
    Initialization(Box<PreparedInitializationOperation>),
    /// Shared declared-case, simulation, fitting or horizon owner.
    Study(PreparedStudyOperation),
    /// A shooting problem retains its exact requested initial vector.
    #[cfg(feature = "solver-diffsol")]
    Shooting {
        /// Existing native problem owner.
        problem: std::sync::Arc<super::ShootingProblem>,
        /// Original coordinates selected before dispatch.
        initial: Option<Vec<f64>>,
    },
}
impl PreparedOperation {
    /// Submit one explicit original native algebraic identity, resolving persisted repetitions.
    /// # Errors
    /// Other prepared families require their own issued persisted constructor; no fallback runs.
    pub async fn start_issued(
        &self,
        services: &ExecutionServices,
        identity: super::IssuedRunIdentity,
        cancel: &CancelSource,
    ) -> Result<super::IssuedRunStart, WorkflowError> {
        match self {
            Self::Study(PreparedStudyOperation::DeclaredCase(case)) => {
                services
                    .start_modeling_issued(vec![case.as_ref().clone()], identity, cancel)
                    .await
            }
            _ => Err(projection_refusal(
                "issued execution requires the prepared algebraic operation owner",
            )),
        }
    }
    /// Original scientific input identity, established by its actual owner.
    /// # Errors
    /// Horizon identity requires the contextual horizon admission.
    pub fn identity(&self) -> Result<pse_ids::ContentHash, WorkflowError> {
        match self {
            Self::Initialization(prepared) => Ok(prepared.identity()),
            Self::Study(PreparedStudyOperation::DeclaredCase(prepared)) => prepared
                .solve
                .request_identity()
                .map(|key| key.as_id())
                .map_err(crate::math::MathRuntimeError::from)
                .map_err(Into::into),
            Self::Study(PreparedStudyOperation::Simulation(prepared)) => Ok(prepared.identity()),
            Self::Study(PreparedStudyOperation::Fit(prepared)) => Ok(prepared.identity()),
            Self::Study(PreparedStudyOperation::Horizon(_)) => Err(contract(
                "horizon identity requires its contextual admission",
            )),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { problem, initial } => {
                Ok(problem.request_identity(initial.as_deref()).as_id())
            }
        }
    }
    /// Keep study admission a typed subset of actual prepared capabilities.
    /// # Errors
    /// Shooting cannot enter the persisted Version<8> operation set.
    pub fn into_study(self) -> Result<PreparedStudyOperation, WorkflowError> {
        match self {
            Self::Initialization(_) => Err(projection_refusal(
                "initialization requires an explicit persisted operation contract, outside StudyOperation Version<8>",
            )),
            Self::Study(prepared) => Ok(prepared),
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { .. } => Err(projection_refusal(
                "shooting has no StudyOperation Version<8> projection",
            )),
        }
    }
    /// Start through the same joined supervisor used by durable workers.
    /// # Errors
    /// Original operation admission and execution refusals retain their causes.
    pub async fn start(
        &self,
        services: &ExecutionServices,
        cancel: &CancelSource,
    ) -> Result<RunHandle, WorkflowError> {
        match self {
            Self::Initialization(initialization) => {
                services.start_initialization(initialization.as_ref().clone(), cancel)
            }
            Self::Study(prepared) => prepared.start(services, cancel).await,
            #[cfg(feature = "solver-diffsol")]
            Self::Shooting { problem, initial } => {
                problem.start_replay_with_cancel(services, initial.clone(), cancel)
            }
        }
    }
}

impl ModelingPackage {
    /// Prepare one host request under explicit source/context, controls and admitted values.
    /// # Errors
    /// Invalid context, unsupported bindings or the selected scientific owner's refusal.
    pub async fn prepare_operation(
        &self,
        services: &PreparationServices,
        source: &OperationSource,
        preparation: &PreparationSettings,
        request: RuntimeOperationRequest,
        binding: Option<&AdmittedBinding>,
        cancel: &CancelSource,
    ) -> Result<PreparedOperation, WorkflowError> {
        self.check_service_bindings(services)?;
        source.check(self)?;
        let driver = services.math().preparation_driver(cancel)?;
        match request {
            RuntimeOperationRequest::DeclaredCase { case, settings } => {
                self.prepare_case_operation(
                    services,
                    case,
                    &settings,
                    preparation,
                    None,
                    binding,
                    &driver,
                )
                .await
            }
            RuntimeOperationRequest::Study {
                operation,
                output,
                admitted_horizon,
            } => {
                let admitted_horizon = match admitted_horizon {
                    Some(values) => Some(values),
                    None => {
                        super::study_operations::admit_horizon_values(
                            services,
                            self,
                            source,
                            preparation,
                            &operation,
                            &driver,
                        )
                        .await?
                    }
                };
                super::study_operations::OperationPreparation {
                    source,
                    preparation,
                    operation: &operation,
                    output: &output,
                    admitted_horizon: admitted_horizon.as_ref(),
                    binding,
                }
                .prepare(services, self, &driver)
                .await
                .map(PreparedOperation::Study)
            }
            #[cfg(feature = "solver-diffsol")]
            RuntimeOperationRequest::Shooting { problem, initial } => {
                OperationSource::of(&problem.simulation.source).check(self)?;
                if binding.is_some_and(|binding| !binding.entries.is_empty()) {
                    return Err(contract("shooting request has unsupported bindings"));
                }
                Ok(PreparedOperation::Shooting { problem, initial })
            }
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "original request coordinates and controls remain explicit alongside the receiving effect capability"
    )]
    pub(in crate::workflow) async fn prepare_case_operation(
        &self,
        services: &PreparationServices,
        case: DeclarationId,
        settings: &SolveSettings,
        preparation: &PreparationSettings,
        expected_route: Option<ModelingAnalysisRoute>,
        binding: Option<&AdmittedBinding>,
        cancel: &CancelSource,
    ) -> Result<PreparedOperation, WorkflowError> {
        self.check_service_bindings(services)?;
        #[cfg(test)]
        super::study_operations::count_declared_admission();
        let execution = self
            .admit_declared_request(services, case, preparation, settings.clone(), cancel)
            .await?;
        if expected_route.is_some_and(|route| execution.route != route) {
            return Err(pse_model::diagnostic::BoundaryDiagnostic::new(
                pse_model::diagnostic::BoundaryClass::Unsupported,
                pse_diagnostics::DiagnosticStage::StudyAdmission,
                [case.as_id()],
                pse_diagnostics::DiagnosticRule::StudyOperationUnsupported,
            )
            .into());
        }
        #[cfg(test)]
        super::study_operations::observe_declared_admission();
        match execution.procedure {
            DeclaredProcedure::Solve => self
                .prepare_operation_execution(services, execution, binding, None, cancel)
                .await
                .map(|prepared| {
                    PreparedOperation::Study(PreparedStudyOperation::DeclaredCase(Box::new(
                        prepared,
                    )))
                }),
            DeclaredProcedure::Initialize(_) => {
                if let Some(binding) = binding {
                    self.validate_binding(execution.model.compiled(), binding)?;
                    if !binding.values().is_empty() {
                        return Err(projection_refusal(
                            "initialization requires its original specification, without a study input overlay",
                        ));
                    }
                }
                Ok(PreparedOperation::Initialization(Box::new(
                    PreparedInitializationOperation::new(self.clone(), execution)?,
                )))
            }
            DeclaredProcedure::Integrate(_) => Err(projection_refusal_at(
                "authored integration requires a Simulation operation and its integration controls",
                Some(case),
            )),
            _ => Err(projection_refusal_at(
                "selected authored procedure is unsupported by a Case operation",
                Some(case),
            )),
        }
    }

    /// Existing lane admissions and immediate requests share the same binding lowering.
    pub(in crate::workflow) async fn prepare_operation_execution(
        &self,
        services: &PreparationServices,
        mut execution: DeclaredExecution,
        binding: Option<&AdmittedBinding>,
        current_view: Option<&mut super::study_operations::CurrentNumericalView>,
        cancel: &CancelSource,
    ) -> Result<ModelingSolvePreparation, WorkflowError> {
        self.check_service_bindings(services)?;
        let Some(binding) = binding else {
            return self.prepare_declared(services, &execution, cancel).await;
        };
        execution.prepared_model(self)?;
        self.validate_binding(execution.model.compiled(), binding)?;
        let bound_source = self.retain_bound_source(services, binding)?;
        execution.analysis.case.members = binding.values();
        let analysis = execution.analysis;
        let resolution = self
            .resolve_prepared_case_with_current_view(
                services,
                execution.model,
                analysis.case,
                analysis.order,
                analysis.compiler,
                analysis.solver,
                analysis.numerical,
                super::modeling::cases::CaseOverrides::default(),
                false,
                current_view,
                cancel,
            )
            .await?;
        let mut prepared = self.finish_case(services, resolution, cancel).await?;
        prepared.bound_source = Some(bound_source);
        Ok(prepared)
    }
}

#[cfg(test)]
mod initialization_tests {
    use super::*;
    use pse_relations::columnar::RelationRow;

    #[tokio::test]
    async fn initialization_operation_retains_success_and_failed_stage_projection() {
        initialization_round_trip(crate::workflow::tests::runtime(), false).await;
    }
    #[tokio::test]
    async fn canonical_initialization_operation_reopens_complete_success_and_failure_history() {
        initialization_round_trip(crate::workflow::durable_tests::durable_runtime(), true).await;
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn initialization_root_response_retains_original_point_permission_and_invocation() {
        initialization_root_response(crate::workflow::tests::runtime(), false).await;
    }
    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn canonical_initialization_root_response_reopens_scoped_permission() {
        initialization_root_response(crate::workflow::durable_tests::durable_runtime(), true).await;
    }
    #[cfg(feature = "solver-kinsol")]
    async fn initialization_root_response(runtime: super::super::Runtime, recorded: bool) {
        use pse_model::generated::enums::DerivedQuantity;
        use pse_relations::generated::runtime::parametric_sensitivities;
        let rows = pse_authoring::language::parse(
            "package p {def D {param a:Scalar=2; var x:Scalar; annotation start x(1); eq e:x==a; annotation report x(\"state\"); stage \"warm\" {override eq e:x==a;}} case selected fixture {dof 0; route simultaneous; procedure initialize; stages(\"warm\"); value root.a=2;} {child root:D=D();}}",
            pse_ids::SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, Default::default(),
        ).unwrap();
        let case = rows
            .iter()
            .find(|row| row.name == "selected")
            .unwrap()
            .declaration_id;
        let parameter_declaration = rows
            .iter()
            .find(|row| row.name == "a")
            .unwrap()
            .declaration_id;
        let target_declaration = rows
            .iter()
            .find(|row| row.name == "x")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let services = runtime.execution_services();
        let compiler = crate::workflow::tests::compiler_profile();
        let cancel = CancelSource::new();
        let declared = package
            .declared_execution(
                services.preparation(),
                case,
                compiler,
                crate::workflow::tests::profile(),
                Default::default(),
                Default::default(),
                &cancel,
            )
            .await
            .unwrap();
        let model = package
            .prepare(
                services.preparation(),
                declared.analysis.root,
                declared.analysis.instance,
                declared.analysis.bindings.clone(),
                declared.analysis.limits,
                &cancel,
            )
            .await
            .unwrap();
        let symbol = |declaration| {
            let mut symbols = model
                .compiled()
                .model
                .symbols
                .values()
                .filter(|symbol| symbol.lineage.declaration == declaration);
            let id = symbols
                .next()
                .expect("selected declaration has its actual instantiated symbol")
                .id;
            assert!(
                symbols.next().is_none(),
                "this case has exactly one instance of each selected declaration"
            );
            id
        };
        let parameter = symbol(parameter_declaration);
        let target = symbol(target_declaration);
        let operation = package
            .prepare_operation(
                services.preparation(),
                &OperationSource::of(&package),
                &PreparationSettings {
                    compiler,
                    limits: Default::default(),
                },
                RuntimeOperationRequest::DeclaredCase {
                    case,
                    settings: Box::new(SolveSettings {
                        intent: pse_backend_native::solve::SolveIntent::Root,
                        backend: Some(pse_backend_native::solve::Backend::Kinsol),
                        presolve: pse_backend_native::presolve::PolicyKind::Off,
                        sensitivity: Some(crate::math::settings::SensitivityRequest {
                            parameters: vec![parameter],
                            reduced_hessian: false,
                            propagation: None,
                        }),
                        ..Default::default()
                    }),
                },
                None,
                &cancel,
            )
            .await
            .unwrap();
        let handle = operation.start(&services, &cancel).await.unwrap();
        let keys = recorded.then(|| {
            (
                handle.canonical_run_key().unwrap().to_owned(),
                handle.canonical_attempt_key().unwrap().to_owned(),
            )
        });
        let run = handle.wait().await.unwrap();
        let super::super::RunReport::Initialization(report) = run.report().unwrap() else {
            panic!("full initialization report required");
        };
        assert!(report.completed, "{:?}", report.failure);
        assert_eq!(report.attempts.len(), 2);
        assert_eq!(
            report.attempts[0].step,
            super::super::ModelingInitializationStep::Stage("warm".into())
        );
        assert_eq!(
            report.attempts[1].step,
            super::super::ModelingInitializationStep::Original
        );
        let completion = run.completion().unwrap();
        assert!(!completion.step_permission(0).unwrap().permits_result());
        assert!(completion.step_permission(1).unwrap().permits_result());
        for (ordinal, attempt) in report.attempts.iter().enumerate() {
            let result = attempt.result.as_ref().unwrap();
            assert!(result.accepted);
            let scope = super::super::ProductScope::Point {
                step: Some(ordinal as i64),
                member: None,
                point: result.run_id.as_id(),
            };
            assert_eq!(
                completion
                    .product_permission(&scope, DerivedQuantity::ParametricSensitivity)
                    .unwrap()
                    .permits_result(),
                ordinal == 1,
                "identical valid derivatives at a stage and original have distinct scientific scopes"
            );
        }
        let table = run.table("runtime.parametric_sensitivities").unwrap();
        let responses = parametric_sensitivities::Row::rows(&table).unwrap();
        assert_eq!(responses.len(), 2);
        assert_eq!(
            responses
                .iter()
                .map(|row| row.step)
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from([0, 1])
        );
        for response in &responses {
            assert_eq!(response.run_id, run.run_id);
            assert_eq!(
                (response.parameter_id, response.target_id),
                (parameter, target)
            );
            assert!(
                (response.primal.unwrap() - 1.).abs() < 1e-12,
                "x=a has the exact independent physical response dx/da=1"
            );
        }
        let invocation = run.invocation_evidence().unwrap();
        assert_eq!(invocation.family, "initialization");
        for (step, expected) in [
            (0, crate::invocation::ProductAvailability::Unavailable),
            (1, crate::invocation::ProductAvailability::Available),
        ] {
            let product = invocation
                .products
                .iter()
                .find(|product| {
                    product.role == "parametric_sensitivity" && product.step == Some(step)
                })
                .unwrap();
            assert_eq!(product.availability, expected);
        }
        if let Some((run_key, attempt_key)) = keys {
            let expected = table.batch().clone();
            drop(invocation);
            drop((run, handle, operation));
            let mut reader = runtime
                .results(
                    &run_key,
                    &attempt_key,
                    "runtime.parametric_sensitivities",
                    0,
                    2,
                    cancel.token().clone(),
                )
                .await
                .unwrap();
            assert_eq!(
                reader.next_relation_batch().await.unwrap().unwrap().batch(),
                &expected
            );
            assert!(reader.next_relation_batch().await.unwrap().is_none());
            for (start, end, usability) in [
                (0, 1, super::super::PackScientificState::Unusable),
                (1, 2, super::super::PackScientificState::Usable),
            ] {
                let reader = runtime
                    .results(
                        &run_key,
                        &attempt_key,
                        "runtime.parametric_sensitivities",
                        start,
                        end,
                        cancel.token().clone(),
                    )
                    .await
                    .unwrap();
                assert_eq!(reader.pack_selection().await.unwrap().usability, usability);
            }
        }
    }
    async fn initialization_round_trip(runtime: super::super::Runtime, recorded: bool) {
        let rows = pse_authoring::language::parse(
            "package p {def D {var x:Scalar; eq e:x==2; annotation report x(\"state\"); annotation check x(x==2); stage \"warm\" {override eq e:x==2;} stage \"bad\" {override eq e:x==3;}} test successful fixture {dof -1; route simultaneous; procedure initialize; stages(\"warm\"); fix root.x=2;} {child root:D=D();} test failed fixture {dof -1; route simultaneous; procedure initialize; stages(\"warm\",\"bad\"); fix root.x=2;} {child root:D=D();}}",
            pse_ids::SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, Default::default(),
        ).unwrap();
        let cases = ["successful", "failed"].map(|name| {
            rows.iter()
                .find(|row| row.name == name)
                .unwrap()
                .declaration_id
        });
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let services = runtime.execution_services();
        let cancel = CancelSource::new();
        for (case, completed) in cases.into_iter().zip([true, false]) {
            let operation = package
                .prepare_operation(
                    services.preparation(),
                    &OperationSource::of(&package),
                    &PreparationSettings {
                        compiler: crate::workflow::tests::compiler_profile(),
                        limits: Default::default(),
                    },
                    RuntimeOperationRequest::DeclaredCase {
                        case,
                        settings: Box::new(SolveSettings {
                            intent: pse_backend_native::solve::SolveIntent::Root,
                            presolve: pse_backend_native::presolve::PolicyKind::Off,
                            ..Default::default()
                        }),
                    },
                    None,
                    &cancel,
                )
                .await
                .unwrap();
            assert!(matches!(&operation, PreparedOperation::Initialization(_)));
            if !recorded {
                assert!(operation.clone().into_study().is_err());
                let PreparedOperation::Initialization(original) = &operation else {
                    panic!("prepared initialization operation required");
                };
                let mut changed = original.execution.clone();
                let DeclaredProcedure::Initialize(policy) = &mut changed.procedure else {
                    panic!("declared initialization policy required");
                };
                policy.maximum_attempts += 1;
                let changed =
                    PreparedInitializationOperation::new(package.clone(), changed).unwrap();
                assert_ne!(changed.identity(), original.identity());
                let facts = super::super::replay::initialization_request_facts(
                    original,
                    pse_operations::mint_id(),
                )
                .unwrap();
                let mut missing_policy = serde_json::to_value(facts.constructor).unwrap();
                missing_policy.as_object_mut().unwrap().remove("policy");
                assert!(
                    serde_json::from_value::<super::super::ReplayConstructor>(missing_policy)
                        .is_err()
                );
                let archived = OperationRequest::DeclaredCase(super::super::CaseOperation {
                    case,
                    route: ModelingAnalysisRoute::Simultaneous,
                    settings: SolveSettings {
                        intent: pse_backend_native::solve::SolveIntent::Root,
                        presolve: pse_backend_native::presolve::PolicyKind::Off,
                        ..Default::default()
                    },
                });
                let archived: OperationRequest =
                    serde_json::from_slice(&serde_json::to_vec(&archived).unwrap()).unwrap();
                assert!(
                    package
                        .prepare_operation(
                            services.preparation(),
                            &OperationSource::of(&package),
                            &PreparationSettings {
                                compiler: crate::workflow::tests::compiler_profile(),
                                limits: Default::default()
                            },
                            RuntimeOperationRequest::Study {
                                operation: Box::new(archived),
                                output: super::super::StudyOutputDemand::Full {},
                                admitted_horizon: None
                            },
                            None,
                            &cancel
                        )
                        .await
                        .is_err(),
                    "historical solve-only descriptors cannot acquire initialization semantics"
                );
            }
            let handle = operation.start(&services, &cancel).await.unwrap();
            let keys = if recorded {
                Some((
                    handle.canonical_run_key().unwrap().to_owned(),
                    handle.canonical_attempt_key().unwrap().to_owned(),
                ))
            } else {
                None
            };
            let run = handle.wait().await.unwrap();
            let super::super::RunReport::Initialization(report) = run.report().unwrap() else {
                panic!("full initialization report required");
            };
            assert_eq!(report.completed, completed);
            assert_eq!(report.run_id, run.run_id);
            assert_eq!(report.attempts.len(), 2);
            assert!(report.attempts[0].accepted());
            assert!(
                report.attempts[1].result.is_ok(),
                "the intended stage must execute its original assessment"
            );
            assert_eq!(report.attempts[1].accepted(), completed);
            assert_eq!(run.usable(), completed);
            let contexts = run.table("runtime.scientific_contexts").unwrap();
            let contexts =
                pse_relations::generated::runtime::scientific_contexts::Row::rows(&contexts)
                    .unwrap();
            assert_eq!(
                contexts
                    .iter()
                    .map(|row| row.step)
                    .collect::<std::collections::BTreeSet<_>>(),
                std::collections::BTreeSet::from([0, 1])
            );
            let replay = run.replay().document_with_limit(128 << 20).unwrap();
            assert_eq!(
                replay.contexts, contexts,
                "replay must preserve every actual attempt's source and parent ordinal"
            );
            assert_eq!(replay.runtime_premises.len(), report.attempts.len());
            assert_eq!(replay.solver_settings.len(), report.attempts.len());
            let super::super::ReplayConstructor::Initialization { policy, .. } =
                &replay.constructor
            else {
                panic!("initialization replay constructor required");
            };
            let PreparedOperation::Initialization(request) = &operation else {
                panic!("prepared initialization operation required");
            };
            assert_eq!(
                serde_json::to_value(policy).unwrap(),
                serde_json::to_value(request.policy().unwrap()).unwrap()
            );
            for (ordinal, attempt) in report.attempts.iter().enumerate() {
                let original = attempt.result.as_ref().unwrap();
                assert_eq!(replay.contexts[ordinal].step, ordinal as i64);
                assert_eq!(
                    replay.contexts[ordinal].source_identity,
                    original.prepared.source.revision.identity().as_id()
                );
                assert_eq!(
                    serde_json::to_value(&replay.runtime_premises[ordinal]).unwrap(),
                    serde_json::to_value(original.prepared.solve.execution_snapshot()).unwrap()
                );
            }
            drop(replay);
            let table = run.table("runtime.modeling_initializations").unwrap();
            let stored =
                pse_relations::generated::runtime::modeling_initializations::Row::rows(&table)
                    .unwrap();
            assert_eq!(stored.len(), 1);
            assert_eq!(stored[0].complete, completed);
            assert_eq!(stored[0].attempts.len(), report.attempts.len());
            assert_eq!(stored[0].run_id, report.run_id);
            let findings_table = run.table("runtime.modeling_findings").unwrap();
            let findings =
                pse_relations::generated::runtime::modeling_findings::Row::rows(&findings_table)
                    .unwrap();
            let reports_table = run.table("runtime.modeling_reports").unwrap();
            let reports =
                pse_relations::generated::runtime::modeling_reports::Row::rows(&reports_table)
                    .unwrap();
            let parent_run_id = run.run_id;
            let expected_reports = report
                .attempts
                .iter()
                .enumerate()
                .flat_map(|(ordinal, attempt)| {
                    attempt
                        .result
                        .as_ref()
                        .unwrap()
                        .reports
                        .iter()
                        .cloned()
                        .map(move |mut row| {
                            row.run_id = parent_run_id;
                            row.step = ordinal as i64;
                            row
                        })
                })
                .collect::<Vec<_>>();
            assert_eq!(reports.len(), expected_reports.len());
            for expected in &expected_reports {
                assert_eq!(
                    reports.iter().filter(|row| *row == expected).count(),
                    1,
                    "every actual retained child report projects exactly once at its parent coordinate"
                );
            }
            assert!(reports.iter().all(|row| row.run_id == run.run_id));
            assert_eq!(
                reports
                    .iter()
                    .map(|row| row.step)
                    .collect::<std::collections::BTreeSet<_>>(),
                std::collections::BTreeSet::from([0, 1]),
                "projection coordinates name the actual parent attempts"
            );
            let checks_table = run.table("runtime.modeling_checks").unwrap();
            let checks =
                pse_relations::generated::runtime::modeling_checks::Row::rows(&checks_table)
                    .unwrap();
            let expected_checks =
                report
                    .attempts
                    .iter()
                    .enumerate()
                    .flat_map(|(ordinal, attempt)| {
                        attempt.result.as_ref().unwrap().checks.iter().cloned().map(
                            move |mut row| {
                                row.run_id = parent_run_id;
                                row.step = ordinal as i64;
                                row
                            },
                        )
                    })
                    .collect::<Vec<_>>();
            assert_eq!(checks.len(), expected_checks.len());
            for expected in &expected_checks {
                assert_eq!(
                    checks.iter().filter(|row| *row == expected).count(),
                    1,
                    "stage omission policy cannot drop or reassign an actual retained original check"
                );
            }
            assert!(checks.iter().all(|row| row.run_id == run.run_id));
            if completed {
                assert!(!checks.is_empty());
                assert!(
                    checks.iter().any(|row| row.step == 1),
                    "the restored original retains its requested native check"
                );
            }
            if !completed {
                assert!(stored[0].failure_ordinal.is_some());
                assert!(!findings.is_empty());
            }
            if let Some((run_key, attempt_key)) = keys {
                let expected = table.batch().clone();
                let expected_findings = findings_table.batch().clone();
                let expected_reports = reports_table.batch().clone();
                let expected_checks = checks_table.batch().clone();
                let report_permissions = reports
                    .iter()
                    .map(|row| {
                        let attempt = &report.attempts[row.step as usize];
                        if attempt.step == super::super::ModelingInitializationStep::Original
                            && attempt.accepted()
                        {
                            super::super::PackScientificState::Usable
                        } else {
                            super::super::PackScientificState::Unusable
                        }
                    })
                    .collect::<Vec<_>>();
                drop(handle);
                drop(run);
                let mut reader = runtime
                    .results(
                        &run_key,
                        &attempt_key,
                        "runtime.modeling_initializations",
                        0,
                        1,
                        cancel.token().clone(),
                    )
                    .await
                    .unwrap();
                let batch = reader.next_relation_batch().await.unwrap().unwrap();
                assert_eq!(batch.batch(), &expected);
                assert!(reader.next_relation_batch().await.unwrap().is_none());
                let mut reports_reader = runtime
                    .results(
                        &run_key,
                        &attempt_key,
                        "runtime.modeling_reports",
                        0,
                        expected_reports.num_rows() as u64,
                        cancel.token().clone(),
                    )
                    .await
                    .unwrap();
                let reports_batch = reports_reader.next_relation_batch().await.unwrap().unwrap();
                assert_eq!(reports_batch.batch(), &expected_reports);
                assert!(
                    reports_reader
                        .next_relation_batch()
                        .await
                        .unwrap()
                        .is_none()
                );
                if expected_checks.num_rows() > 0 {
                    let mut checks_reader = runtime
                        .results(
                            &run_key,
                            &attempt_key,
                            "runtime.modeling_checks",
                            0,
                            expected_checks.num_rows() as u64,
                            cancel.token().clone(),
                        )
                        .await
                        .unwrap();
                    assert_eq!(
                        checks_reader
                            .next_relation_batch()
                            .await
                            .unwrap()
                            .unwrap()
                            .batch(),
                        &expected_checks
                    );
                    assert!(checks_reader.next_relation_batch().await.unwrap().is_none());
                }
                for (index, usability) in report_permissions.into_iter().enumerate() {
                    let reader = runtime
                        .results(
                            &run_key,
                            &attempt_key,
                            "runtime.modeling_reports",
                            index as u64,
                            index as u64 + 1,
                            cancel.token().clone(),
                        )
                        .await
                        .unwrap();
                    assert_eq!(
                        reader.pack_selection().await.unwrap().usability,
                        usability,
                        "an accepted auxiliary stage is diagnostic; only an accepted restored original supplies scientific use"
                    );
                }
                if !findings.is_empty() {
                    let mut reader = runtime
                        .results(
                            &run_key,
                            &attempt_key,
                            "runtime.modeling_findings",
                            0,
                            findings.len() as u64,
                            cancel.token().clone(),
                        )
                        .await
                        .unwrap();
                    let batch = reader.next_relation_batch().await.unwrap().unwrap();
                    assert_eq!(batch.batch(), &expected_findings);
                    assert!(reader.next_relation_batch().await.unwrap().is_none());
                }
            }
        }
    }
}
