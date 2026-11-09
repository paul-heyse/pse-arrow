// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit prepared production registrations, actual authored steam equipment,
//! native root solves, and retained original-model conformance observations.
use super::*;
use crate::{math::solves::NumericalInputs, workflow::tests as fixture};
use pse_backend_native::coolprop::{self, WaterRegime};
use pse_model::generated::enums::ModelingDeclarationKind as DeclarationKind;
use std::collections::BTreeSet;

#[path = "steam_reference.rs"]
mod independent_reference;

fn steam_infrastructure(
    operation: &'static str,
    source: impl std::error::Error + Send + Sync + 'static,
) -> WorkflowError {
    pse_engine::EngineError::Infrastructure {
        op: operation.into(),
        source: Box::new(source),
    }
    .into()
}

async fn steam_physical(runtime: &Runtime) -> Result<PhysicalContext, WorkflowError> {
    use pse_admission::authoring_driver::document::{
        OwnedDocumentSet, load_package_documents_owned,
    };
    let token = pse_columnar::CancellationToken::new();
    let pool = runtime.shared.pool();
    let selected = crate::scientific_inputs::family("steam");
    let bundle = load_package_documents_owned(
        &selected
            .package("packages/reference/physical")
            .expect("steam's physical package")
            .files,
        runtime.registry(),
        Default::default(),
        &pool,
        &token,
        &runtime.validation_context(),
    )?;
    let documents = OwnedDocumentSet::try_from_bundles(vec![bundle], &pool, &token)?;
    // Provider units and caloric datum come from the same captured source closure
    // as the authored equipment, rather than the synthetic standard registry.
    let physical = runtime.physical_from_documents(&documents, &token).await?;
    assert!(physical.package.is_some());
    assert!(physical.documents.is_some());
    Ok(physical)
}

#[tokio::test]
async fn captured_flow_over_density_requires_the_actual_nominal_molar_contracts() {
    use pse_quantity::infer::{OpRequest, Operand, infer_with_evidence};
    let runtime = fixture::runtime();
    let physical = steam_physical(&runtime).await.unwrap();
    let registry = physical.quantities();
    let quantity = |name| match registry.physical_name(name) {
        Some(pse_quantity::PhysicalName::QuantityType(id)) => id,
        _ => panic!("captured physical quantity {name}"),
    };
    let indices = pse_quantity::IndexSet::default();
    let flow = quantity("Flow");
    let density = quantity("Density");
    let operands = [
        Operand {
            quantity_type: flow,
            indices: &indices,
        },
        Operand {
            quantity_type: density,
            indices: &indices,
        },
    ];
    let inferred = infer_with_evidence(
        &OpRequest::Div,
        &operands,
        registry,
        physical.preconditions(),
    )
    .unwrap();
    assert_eq!(inferred.result, quantity("VolumeFlow"));
    assert!(
        infer_with_evidence(
            &OpRequest::Div,
            &operands,
            registry,
            &pse_quantity::infer::NoInvariantFacts
        )
        .is_err()
    );

    // Preserve dimensions and nominal flow kind while changing only its actual material basis.
    // The selected nominal operation must refuse, without an algebraic fallback.
    let mass = registry
        .bases()
        .find(|basis| basis.kind == pse_quantity::BasisKind::Mass)
        .unwrap()
        .id;
    let mut wrong_flow = registry.quantity_type(flow).unwrap().clone();
    wrong_flow.id = pse_operations::mint_id();
    wrong_flow.name = Some("WrongBasisFlow".into());
    wrong_flow.key.basis = Some(mass);
    let wrong = wrong_flow.id;
    let mut builder = registry.to_builder();
    builder.quantity_type(wrong_flow);
    let changed = builder.build().unwrap();
    let wrong_operands = [
        Operand {
            quantity_type: wrong,
            indices: &indices,
        },
        operands[1],
    ];
    assert!(
        infer_with_evidence(
            &OpRequest::Div,
            &wrong_operands,
            &changed,
            physical.preconditions()
        )
        .is_err()
    );
    assert!(
        infer_with_evidence(
            &OpRequest::Div,
            &[operands[1], operands[0]],
            registry,
            physical.preconditions()
        )
        .is_err()
    );
    runtime.close();
}

async fn steam_package(
    runtime: &Runtime,
    physical: PhysicalContext,
    providers: BTreeMap<String, pse_kernels::Registration>,
    wrong_component: bool,
) -> Result<ModelingPackage, WorkflowError> {
    use pse_admission::authoring_driver::document::{
        OwnedDocumentSet, load_package_documents_owned,
    };
    let token = pse_columnar::CancellationToken::new();
    let pool = runtime.shared.pool();
    let selected = crate::scientific_inputs::family("steam");
    let bundles = selected
        .packages()
        .map(|(_, package)| -> Result<_, WorkflowError> {
            let mut files = std::borrow::Cow::Borrowed(&package.files);
            if wrong_component {
                for (path, bytes) in files.to_mut() {
                    if path.ends_with(".pse") {
                        let source = std::str::from_utf8(bytes).map_err(|error| {
                            steam_infrastructure("decode captured steam source", error)
                        })?;
                        *bytes = source
                            .replace(
                                "water_component=chem.water",
                                "water_component=chem.nitrogen",
                            )
                            .into_bytes();
                    }
                }
            }
            Ok(load_package_documents_owned(
                &files,
                runtime.registry(),
                Default::default(),
                &pool,
                &token,
                &runtime.validation_context(),
            )?)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let documents = OwnedDocumentSet::try_from_bundles(bundles, &pool, &token)?;
    runtime
        .modeling_from_documents_registered(
            &documents,
            physical,
            providers,
            &crate::CancelSource::new(),
        )
        .await
}

async fn conform(name: &str, worker: Option<&std::path::Path>) -> Result<(), WorkflowError> {
    conform_case(name, worker, false).await
}

fn assert_wall_inlet_ph_roots(package: &ModelingPackage, result: &ModelingResult) {
    use crate::math::solves::Outcome;
    use pse_backend_native::solve::Backend;
    use pse_model::generated::enums::ModelingCheckKind;

    // Two uncoupled PH states can use an original assessment of separately
    // solved native components. An all-fixed evaluation has no such evidence.
    let Outcome::Constant(original) = &result.outcome else {
        panic!("two independent PH roots require original component assessment: {:?}", result.outcome.state());
    };
    assert!(result.accepted && result.validation_error.is_none() && original.quality.feasible());
    let model = &result.prepared.model.model.compiled().model;
    let instance = result.prepared.model.model.solved().instance();
    let path = &model.instances[&instance].path;
    let quantities = package.physical_context().quantities();
    let coordinate = |side: &str, field: &str, quantity: &str| {
        let full = format!("{path}.{side}.state.{field}");
        let mut members = model.symbols.values().filter(|symbol| symbol.lineage.path == full);
        let member = members.next().unwrap_or_else(|| panic!("original PH coordinate {full} absent"));
        assert!(members.next().is_none(), "ambiguous original PH coordinate {full}");
        let physical = member.ty.quantity_scheme().unwrap().resolve(quantities, &BTreeMap::new()).unwrap();
        assert_eq!(quantities.quantity_type(physical).unwrap().name.as_deref(), Some(quantity));
        let value = original.coordinates.iter().find(|(id, _)| *id == member.id)
            .map(|(_, value)| *value)
            .or_else(|| result.values.scalars.get(&member.id).copied())
            .unwrap_or_else(|| panic!("original PH coordinate {full} has no value"));
        assert!(value.is_finite(), "nonfinite original PH coordinate {full}");
        (member.id, value)
    };
    let mut enthalpies = Vec::new();
    for (side, expectation) in [
        ("hot", "66e02000000000000000000000000019"),
        ("cold", "66e0200000000000000000000000001a"),
    ] {
        let (temperature, _) = coordinate(side, "T", "Temperature");
        let (density, molar_density) = coordinate(side, "rho", "Density");
        assert!((50_000. .. 56_000.).contains(&molar_density));
        assert!(original.component_reports().any(|component| {
            component.backend == Backend::Ipopt
                && component.candidate.is_some()
                && component.variables.contains(&temperature)
                && component.variables.contains(&density)
        }), "{side} PH state must retain an actual coupled liquid temperature/density root");
        let source = DeclarationId::from_id(SemanticId::parse_hex(expectation).unwrap());
        let mut observed = result.checks.iter().filter(|check| {
            check.source_id == source && check.kind == ModelingCheckKind::Expectation
        });
        let check = observed.next().expect("original independent PH temperature comparison");
        assert!(observed.next().is_none());
        assert_eq!(check.tolerance, Some(0.01));
        assert!(check.satisfied && check.value <= 0.01);
        assert!((coordinate(side, "pressure", "Pressure").1 - 101_325.).abs() <= 1.);
        enthalpies.push(coordinate(side, "h", "MolarEnthalpy").1);
    }
    assert!((enthalpies[0] - enthalpies[1] - 1000.).abs() <= 0.01);
}

fn assert_linear_valve_liquid_root(result: &ModelingResult) {
    use crate::math::solves::Outcome;
    use pse_backend_native::solve::{Backend, SolveReport};

    assert!(result.accepted && result.validation_error.is_none());
    let model = &result.prepared.model.model.compiled().model;
    let root = &model.instances[&result.prepared.model.model.solved().instance()];
    let outlet = ["T", "rho"].map(|name| {
        let path = format!("{}.root.outlet.{name}", root.path);
        let mut members = model.symbols.values().filter(|member| member.lineage.path == path);
        let id = members.next().unwrap_or_else(|| panic!("original valve state {path} absent")).id;
        assert!(members.next().is_none(), "ambiguous valve state {path}");
        id
    });
    let solves_outlet = |report: &SolveReport| {
        report.backend == Backend::Ipopt && report.candidate.is_some()
            && outlet.iter().all(|member| report.variables.contains(member))
    };
    assert!(match &result.outcome {
        Outcome::Native(report) => solves_outlet(report),
        Outcome::Constant(report) => report.quality.feasible() && report.component_reports().any(solves_outlet),
        Outcome::Rejected(_) => false,
    }, "selected valve must retain the actual coupled liquid outlet root");
}

async fn conform_case(
    name: &str,
    worker: Option<&std::path::Path>,
    compare_reference: bool,
) -> Result<(), WorkflowError> {
    use futures_util::FutureExt as _;
    use std::io::Write as _;
    let started = std::time::Instant::now();
    let child_observations = Arc::new(Mutex::new(Vec::new()));
    let runtime = fixture::runtime_with_workspace(128 << 20);
    let services = runtime.execution_services();
    let outcome = std::panic::AssertUnwindSafe(async {
        let physical = steam_physical(&runtime).await?;
        let mut registrations: BTreeMap<_, _> = [WaterRegime::Liquid, WaterRegime::Vapor]
            .into_iter()
            .map(|regime| -> Result<_, WorkflowError> {
                Ok((
                    regime.implementation().to_owned(),
                    match worker {
                        None => coolprop::registration(&physical.quantities, regime).map_err(
                            |error| {
                                WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(
                                    error,
                                ))
                            },
                        )?,
                        Some(path) => crate::scientific_worker::LocalWaterFactory::registration(
                            crate::scientific_worker::PreparedExecutable::observe(path, 64 << 20)
                                .map_err(|error| {
                                    WorkflowError::Typed(
                                        pse_model::diagnostic::DiagnosticCause::new(error),
                                    )
                                })?
                                .measured(child_observations.clone()),
                            &physical.quantities,
                            regime,
                            runtime.shared.math().clone(),
                        )
                        .map_err(|error| {
                            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
                        })?,
                    },
                ))
            })
            .collect::<Result<_, _>>()?;
        if name == "wet_water_turbine" {
            registrations.insert(
                coolprop::WATER_SATURATION_IMPLEMENTATION.to_owned(),
                coolprop::saturation_registration(&physical.quantities).map_err(|error| {
                    WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
                })?,
            );
        }
        let source = crate::scientific_inputs::family_sources("steam");
        let rows = crate::scientific_inputs::parse_sources(
            &source,
            pse_authoring::language::IdentityPolicy::Named,
        );
        let selected = rows
            .iter()
            .find(|row| row.name == name && row.value.kind == DeclarationKind::Test)
            .ok_or_else(|| {
                WorkflowError::Internal(format!(
                    "captured steam conformance fixture absent: {name}"
                ))
            })?
            .declaration_id;
        let mut solver = fixture::profile();
        // The authored two-sided physical bounds require the existing bounded-root route.
        solver.selection = pse_backend_native::solve::SolverSelection::Explicit(
            pse_backend_native::solve::Backend::Ipopt,
        );
        let policy = ModelingConformancePolicy {
            compiler: fixture::compiler_profile(),
            solver,
            numerical: NumericalInputs::default(),
            limits: Limits::default(),
            derivatives: pse_backend_native::derivative_diagnostics::Policy {
                perturbation: 1e-6,
                relative_tolerance: 1e-4,
                // Dense diagnostic extents are n*(m+1): pump 11*12, turbines 17*18.
                maximum_cells: 512,
            },
            maximum_fixtures: 1,
            maximum_checks: if name.starts_with("released_wall_stock_") {
                256
            } else {
                64
            },
            fixtures: ModelingFixtureSelection::Selected(BTreeSet::from([selected])),
            diagnostics: None,
        };
        if name == "liquid_pump" {
            // Change the actual constructor selection to an existing, identified
            // non-water species. Admission must refuse its CAS before any solve.
            let wrong = source.replace(
                "water_component=chem.water",
                "water_component=chem.nitrogen",
            );
            assert_ne!(wrong, source);
            let rejected =
                steam_package(&runtime, physical.clone(), registrations.clone(), true).await?;
            let refusal = rejected
                .conform(&services, policy.clone(), &crate::CancelSource::new())
                .await?;
            assert!(!refusal.passed());
            assert!(
                refusal.results.is_empty(),
                "wrong CAS must not reach a native solve"
            );
            assert!(
                refusal.checks.iter().any(|check| check.message.contains(
                    "HEOS Water state requires an explicitly identified water component"
                )),
                "{:?}; failures={:?}",
                refusal.checks,
                refusal.failures
            );
        }
        // This is the public prepared registration route used by downstream callers;
        // no source-name dispatch, test provider, or separate thermodynamic solver.
        let package = steam_package(&runtime, physical, registrations, false).await?;
        let prepare_ns = started.elapsed().as_nanos();
        let mut conform_ns = Vec::new();
        let mut phase_pool_high_water = Vec::new();
        let mut retained_after_conform = Vec::new();
        for _ in 0..2 {
            runtime.shared.reset_observation_peak();
            let solve_started = std::time::Instant::now();
            let report = package
                .conform(&services, policy.clone(), &crate::CancelSource::new())
                .await?;
            conform_ns.push(solve_started.elapsed().as_nanos());
            assert!(
                report.complete && report.passed(),
                "steam fixture {name}: complete={}; checks={:?}; failures={:?}",
                report.complete,
                report.checks,
                report.failures
            );
            if matches!(name, "linear_water_valve_zero_refuses" | "linear_water_valve_reverse_refuses") {
                assert!(report.results.is_empty(), "unsupported flow must not publish a root");
                assert_eq!(report.failures.len(), 1, "the declared forward-flow guard must refuse");
            } else {
                let result = report.results.get(&selected).expect("an actual authored result must be retained");
                match name {
                    "released_wall_inlet_ph" => assert_wall_inlet_ph_roots(&package, result),
                    "linear_water_valve_rated" | "linear_water_valve_off_design" => assert_linear_valve_liquid_root(result),
                    _ => {
                        let crate::math::solves::Outcome::Native(native) = &result.outcome else {
                            return Err(WorkflowError::Internal(format!(
                                "actual authored steam fixture {name} did not produce a native solve: {:?}",
                                result.outcome.state(),
                            )));
                        };
                        assert_eq!(native.backend, pse_backend_native::solve::Backend::Ipopt);
                    }
                }
                if compare_reference && conform_ns.len() == 1 {
                    independent_reference::compare_wet(&package, result, selected, &services)
                        .map_err(|error| WorkflowError::Infrastructure(format!("independent wet Water reference: {error}")))?;
                }
            }
            drop(report);
            phase_pool_high_water.push(runtime.shared.observation_peak_bytes());
            retained_after_conform.push(
                runtime
                    .shared
                    .report()
                    .map_err(|error| {
                        steam_infrastructure("observe steam conformance resources", error)
                    })?
                    .pool_reserved_now,
            );
        }
        // The original positive package stays retained through native drain and measurements.
        Ok((
            package,
            prepare_ns,
            conform_ns,
            phase_pool_high_water,
            retained_after_conform,
        ))
    })
    .catch_unwind()
    .await;
    let drain_started = std::time::Instant::now();
    runtime.close();
    runtime.clear_program_cache();
    let cleanup = runtime
        .shared
        .math()
        .shutdown_native_lanes()
        .await
        .map_err(WorkflowError::from);
    let drain_ns = drain_started.elapsed().as_nanos();
    let total_ns = started.elapsed().as_nanos();
    let (_package, prepare_ns, conform_ns, phase_pool_high_water, retained_after_conform) =
        match outcome {
            Ok(operation) => WorkflowError::settle(operation, cleanup)?,
            Err(original) => {
                if let Err(cleanup) = cleanup {
                    let _reported = writeln!(
                        std::io::stderr().lock(),
                        "steam conformance original assertion failure; native settlement failed: {cleanup:?}; child observations={:?}",
                        child_observations.lock(),
                    );
                }
                std::panic::resume_unwind(original);
            }
        };
    let resources = runtime
        .shared
        .report()
        .map_err(|error| steam_infrastructure("observe drained steam resources", error))?;
    let children = child_observations
        .lock()
        .map_err(|_| WorkflowError::Internal("steam worker observation lock poisoned".into()))?
        .clone();
    for child in &children {
        assert!(
            child.after_close_ack && child.error.is_none() && child.identity.is_some(),
            "{child:?}"
        );
        assert!(
            child
                .peak_resident_bytes_at_close
                .is_some_and(|bytes| bytes > 0)
        );
        assert_eq!(child.exit_code, Some(0));
        assert!(child.signal.is_none());
        assert!(!std::path::Path::new(&format!("/proc/{}", child.pid)).exists());
    }
    if worker.is_some() {
        assert!(
            !children.is_empty(),
            "equipment must execute through actual worker processes"
        );
    }
    writeln!(std::io::stderr().lock(), "PSE_SCIENTIFIC_WORKER_EQUIPMENT_MEASUREMENT={}", serde_json::json!({
        "case": name, "placement": if worker.is_some() { "local-worker" } else { "in-process" },
        "condition": "fresh runtime and admitted positive package; repeat the same package with reused preparation; pump also retains its prior wrong-CAS intake control",
        "prepare_ns": prepare_ns, "fresh_conform_ns": conform_ns[0], "repeated_conform_ns": conform_ns[1],
        "drain_ns": drain_ns, "total_including_prepare_repeated_case_and_drain_ns": total_ns,
        "accounted_conform_phase_high_water_bytes": phase_pool_high_water,
        "accounted_retained_after_conform_bytes": retained_after_conform,
        "accounted_retained_after_drain_bytes": resources.pool_reserved_now,
        "accounted_all_phases_high_water_bytes": resources.pool_peak_bytes,
        "parent_cumulative_vm_hwm_bytes": resources.process_peak_rss_bytes,
        "configured_opaque_allowance_bytes_per_child": if worker.is_some() { Some(64usize << 20) } else { None },
        "child_snapshots_after_native_close_before_transport_shutdown": children,
    })).map_err(|error| steam_infrastructure("write steam conformance measurement", error))?;
    Ok(())
}

#[tokio::test]
async fn authored_liquid_pump_matches_independent_hydraulic_power_and_iapws_pressure()
-> Result<(), WorkflowError> {
    conform("liquid_pump", None).await
}

#[tokio::test]
async fn authored_steam_turbine_matches_released_single_phase_reference()
-> Result<(), WorkflowError> {
    conform("vapor_turbine", None).await
}

#[tokio::test]
async fn authored_hot_steam_turbine_matches_second_released_single_phase_reference()
-> Result<(), WorkflowError> {
    conform("hot_vapor_turbine", None).await
}

#[tokio::test]
async fn authored_wet_water_turbine_matches_released_reference() -> Result<(), WorkflowError> {
    conform("wet_water_turbine", None).await
}

#[tokio::test]
async fn authored_wet_water_turbine_matches_independent_iapws_binary() -> Result<(), WorkflowError> {
    conform_case("wet_water_turbine", None, true).await
}

#[tokio::test]
async fn authored_wall_inlet_ph_rebases_released_iapws95_liquid_states() -> Result<(), WorkflowError> {
    conform("released_wall_inlet_ph", None).await
}

#[tokio::test]
async fn authored_wall_stock_preserves_released_zero_rate_initialization()
-> Result<(), WorkflowError> {
    conform("released_wall_stock_equilibrium", None).await
}

#[tokio::test]
async fn authored_wall_stock_perturbation_closes_both_fluid_energy_boundaries()
-> Result<(), WorkflowError> {
    conform("released_wall_stock_perturbation", None).await
}

#[tokio::test]
async fn authored_linear_water_valve_preserves_released_rating_and_enthalpy() -> Result<(), WorkflowError> {
    conform("linear_water_valve_rated", None).await
}

#[tokio::test]
async fn authored_linear_water_valve_changes_pressure_off_design() -> Result<(), WorkflowError> {
    conform("linear_water_valve_off_design", None).await
}

#[tokio::test]
async fn authored_linear_water_valve_refuses_zero_and_reverse_flow() -> Result<(), WorkflowError> {
    conform("linear_water_valve_zero_refuses", None).await?;
    conform("linear_water_valve_reverse_refuses", None).await
}

fn scientific_worker_executable() -> Result<std::path::PathBuf, WorkflowError> {
    std::env::var_os("PSE_SCIENTIFIC_WORKER_BINARY")
        .map(std::path::PathBuf::from)
        .ok_or_else(|| WorkflowError::Infrastructure("source-built scientific worker executable fixture must be explicitly supplied by the native correctness owner".into()))
}

#[tokio::test]
async fn authored_liquid_pump_through_prepared_scientific_worker_keeps_original_controls()
-> Result<(), WorkflowError> {
    conform("liquid_pump", Some(&scientific_worker_executable()?)).await
}

#[tokio::test]
async fn authored_steam_turbine_through_prepared_scientific_worker_keeps_original_controls()
-> Result<(), WorkflowError> {
    conform("vapor_turbine", Some(&scientific_worker_executable()?)).await
}

#[tokio::test]
async fn authored_hot_steam_turbine_through_prepared_scientific_worker_keeps_original_controls()
-> Result<(), WorkflowError> {
    conform("hot_vapor_turbine", Some(&scientific_worker_executable()?)).await
}

async fn protected_steam_case(name: &str) -> Result<(), WorkflowError> {
    use base64::Engine as _;
    use buffa_descriptor::{DescriptorPool, DynamicMessage, Value as WireValue};
    use datafusion::arrow::{
        array::{FixedSizeBinaryArray, Float64Array, Int64Array, StringArray},
        ipc::reader::StreamReader,
    };
    use futures_util::FutureExt as _;
    use pse_backend_native::solve::Backend;
    use pse_relations::{columnar::RelationRow as _, generated::runtime::modeling_reports};

    let runtime =
        super::super::durable_tests::durable_runtime_on(fixture::runtime_with_workspace(128 << 20));
    let outcome = std::panic::AssertUnwindSafe(async {
        let physical = steam_physical(&runtime).await.unwrap();
        // Explicit native preparation consumes this captured physical registry.
        // Durable reconstruction receives these registrations, never EOS-name dispatch.
        let mut providers = [WaterRegime::Liquid, WaterRegime::Vapor]
            .into_iter()
            .map(|regime| {
                (
                    regime.implementation().to_owned(),
                    coolprop::registration(physical.quantities(), regime).unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let wet = name == "wet_water_turbine";
        if wet {
            providers.insert(
                coolprop::WATER_SATURATION_IMPLEMENTATION.to_owned(),
                coolprop::saturation_registration(physical.quantities()).unwrap(),
            );
        }
        let package = steam_package(&runtime, physical.clone(), providers.clone(), false)
            .await
            .unwrap();
        let key = format!("protected-steam-{name}");
        let store = runtime.canonical_store().unwrap();
        store
            .create_project(
                &format!("{key}-create"),
                &key,
                "Original HEOS Water equipment",
            )
            .await
            .unwrap();
        let project = store.project(&key).await.unwrap().unwrap();
        let source = Box::pin(runtime.project_fork(
            &project,
            &format!("{key}-main"),
            &format!("{key}-problem"),
            &format!("{key}-fork"),
            &package,
        ))
        .await
        .unwrap();
        let revision = source.key.clone();
        let original = Box::pin(runtime.modeling_revision(source, physical, providers))
            .await
            .unwrap();
        let services = runtime.execution_services();
        let selected = original
            .declarations(services.preparation())
            .await
            .unwrap()
            .into_iter()
            .find(|row| row.name == name && row.value.kind == DeclarationKind::Test)
            .unwrap()
            .declaration_id;
        let cancel = crate::CancelSource::new();
        let operation = Box::pin(original.prepare_operation(
            services.preparation(),
            &crate::workflow::OperationSource::of(&original),
            &crate::workflow::PreparationSettings {
                compiler: fixture::compiler_profile(),
                limits: Limits::default(),
            },
            crate::workflow::RuntimeOperationRequest::DeclaredCase {
                case: selected,
                settings: Box::new(crate::math::settings::SolveSettings {
                    intent: pse_backend_native::solve::SolveIntent::Root,
                    backend: Some(Backend::Ipopt),
                    ..Default::default()
                }),
            },
            None,
            &cancel,
        ))
        .await
        .unwrap();
        let handle = operation.start(&services, &cancel).await.unwrap();
        let run_key = handle.canonical_run_key().unwrap().to_owned();
        let attempt_key = handle.canonical_attempt_key().unwrap().to_owned();
        let run = handle.wait().await.unwrap();
        let crate::workflow::RunReport::Modeling(results) = run.report().unwrap() else {
            panic!("the original authored Water case must produce a native modeling result");
        };
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert!(
            result.accepted && result.validation_error.is_none(),
            "{:?}",
            result.validation_error
        );
        assert!(result.checks.iter().all(|check| check.satisfied));
        let permission = run.completion().unwrap().step_permission(0).unwrap();
        assert!(permission.permits_result());
        assert_eq!(permission.reference_point, Some(result.run_id.as_id()));
        let crate::math::solves::Outcome::Native(native) = &result.outcome else {
            panic!("original Water execution must retain its native outcome");
        };
        assert_eq!(native.backend, Backend::Ipopt);
        let candidate = native.candidate.as_ref().unwrap();
        assert_eq!(native.variables.len(), candidate.primal.len());
        let model = &result.prepared.model.model.compiled().model;
        let root = &model.instances[&result.prepared.model.model.solved().instance()];
        let raw = |path: &str| {
            // Specialization's demand-path index is partial. Observe the original
            // scalar's complete published lineage, under its actual selected root.
            let original_path = format!("{}.{path}", root.path);
            let symbols = model
                .symbols
                .values()
                .filter(|symbol| symbol.lineage.path == original_path)
                .collect::<Vec<_>>();
            assert_eq!(symbols.len(), 1, "original Water scalar {original_path}");
            let id = symbols[0].id;
            if let Some(column) = native.variables.iter().position(|member| *member == id) {
                return candidate.primal[column];
            }
            if let Some(value) = result.values.scalars.get(&id) {
                return *value;
            }
            // A computed member has no independent primal or input value.
            // Observe it through its original run's authored report.
            let mut reports = result.reports.iter().filter(|row| row.target_id == id);
            let report = reports
                .next()
                .unwrap_or_else(|| panic!("original Water value unavailable: {original_path}"));
            assert!(reports.next().is_none(), "ambiguous Water report {original_path}");
            report.value
        };
        // These stored observations come from the published IAPWS/released-case
        // sources; the receiver does not run the independent reference binary.
        let checks = match name {
            "liquid_pump" => {
                assert!(
                    (raw("root.unit.inlet.flow") / raw("root.unit.inlet.rho") - 0.001).abs()
                        <= 1e-10
                );
                vec![
                    ("root.unit.inlet.pressure", 99241.8352, 1.),
                    ("root.unit.outlet.pressure", 199241.8352, 1.),
                    ("root.unit.work_fluid", 100., 1e-6),
                    ("root.unit.work", 125., 1e-6),
                ]
            }
            "vapor_turbine" => vec![
                ("root.unit.outlet.T", 463.435, 0.01),
                ("root.unit.reference.T", 460.149, 0.01),
                ("root.unit.outlet.pressure", 700000., 1.),
                ("root.unit.work", -1224640., 50.),
                ("root.unit.outlet.flow", 1000., 1e-5),
            ],
            "hot_vapor_turbine" => vec![
                ("root.unit.outlet.T", 742.992, 0.01),
                ("root.unit.reference.T", 738.224, 0.01),
                ("root.unit.outlet.pressure", 7000000., 10.),
                ("root.unit.work", -1911550., 50.),
            ],
            "wet_water_turbine" => {
                let actual_quality = raw("root.unit.outlet.quality");
                let isentropic_quality = raw("root.unit.reference.quality");
                assert!(0. < isentropic_quality && isentropic_quality < actual_quality);
                assert!(actual_quality < 1.);
                // Entropy is a computed let without its own report. The
                // original case already demands its isentropic residual.
                let entropy_source = DeclarationId::from_id(
                    SemanticId::parse_hex("64d03000000000000000000000000224").unwrap(),
                );
                let mut entropy = result.checks.iter().filter(|check| {
                    check.source_id == entropy_source
                        && check.kind == pse_model::generated::enums::ModelingCheckKind::Expectation
                });
                let check = entropy.next().expect("original isentropic entropy check");
                assert!(entropy.next().is_none(), "ambiguous original entropy check");
                assert_eq!(check.tolerance, Some(1e-6));
                assert!(check.satisfied && check.value <= 1e-6);
                assert!(
                    (raw("root.unit.outlet.energy_flow") - raw("root.unit.inlet.energy_flow")
                        - raw("root.unit.work"))
                    .abs()
                        <= 0.01
                );
                vec![
                    ("root.unit.outlet.T", 382.442, 0.01),
                    ("root.unit.reference.T", 382.442, 0.01),
                    ("root.unit.outlet.pressure", 140000., 1.),
                    ("root.unit.work", -1010640., 0.01 * 1010640.),
                    ("root.unit.outlet.quality", 0.9886, 0.0001),
                ]
            }
            _ => panic!("unadmitted Water consumer case"),
        };
        for (path, expected, tolerance) in checks {
            assert!(
                (raw(path) - expected).abs() <= tolerance,
                "{name} {path}: {}",
                raw(path)
            );
        }
        let expected = result.reports.clone();
        let quality_source = DeclarationId::from_id(
            SemanticId::parse_hex("64d030000000000000000000000000a6").unwrap(),
        );
        let quality_paths = expected
            .iter()
            .filter(|row| row.source_id == quality_source)
            .map(|row| {
                let path = row.path.strip_prefix(&format!("{}.", root.path)).unwrap();
                assert_eq!(row.value.to_bits(), raw(path).to_bits());
                path
            })
            .collect::<BTreeSet<_>>();
        if wet {
            assert_eq!(
                quality_paths,
                BTreeSet::from(["root.unit.outlet.quality", "root.unit.reference.quality"])
            );
        } else {
            assert!(quality_paths.is_empty());
        }
        let original_reports = model
            .annotations
            .iter()
            .filter(|annotation| {
                matches!(
                    annotation.value,
                    pse_modeling::annotation::AnnotationValue::Report(_)
                )
            })
            .map(|annotation| (annotation.target, annotation.lineage.declaration))
            .collect::<BTreeSet<_>>();
        let produced_reports = expected
            .iter()
            .map(|row| (row.target_id, row.source_id))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            produced_reports.len(),
            expected.len(),
            "original reports are unique"
        );
        assert_eq!(
            produced_reports, original_reports,
            "complete inherited and local authored report population"
        );
        let row_count = expected.len() as u64;
        assert!(row_count > 0);
        let expected_sources = [
            "cdb7d7d6fec34b908d2e7444d1d1e6cb",
            "15c6d2f3831745ed8f2ab22babc681a4",
        ]
        .map(|id| DeclarationId::from_id(SemanticId::parse_hex(id).unwrap()))
        .into_iter()
        .collect::<BTreeSet<_>>();
        let sources = expected
            .iter()
            .map(|row| row.source_id)
            .collect::<BTreeSet<_>>();
        assert!(
            expected_sources.is_subset(&sources),
            "both existing original work reports are required"
        );
        for row in &expected {
            let annotations = model
                .annotations
                .iter()
                .filter(|annotation| {
                    annotation.target == row.target_id
                        && annotation.lineage.declaration == row.source_id
                })
                .collect::<Vec<_>>();
            assert_eq!(annotations.len(), 1);
            assert_eq!(row.run_id, result.run_id);
            assert_eq!(row.step, 0);
            if expected_sources.contains(&row.source_id) {
                let column = native
                    .variables
                    .iter()
                    .position(|member| *member == row.target_id)
                    .unwrap();
                assert_eq!(row.value.to_bits(), candidate.primal[column].to_bits());
                assert!(
                    row.transfer_context.is_some(),
                    "work keeps its original directed energy boundary"
                );
            }
        }
        drop((run, handle, operation, original, package));
        runtime.clear_program_cache();

        let token = pse_columnar::CancellationToken::new();
        let mut reader = runtime
            .results(
                &run_key,
                &attempt_key,
                "runtime.modeling_reports",
                0,
                row_count,
                token.clone(),
            )
            .await
            .unwrap();
        let selection = reader.pack_selection().await.unwrap();
        assert_eq!(
            selection.usability,
            crate::workflow::PackScientificState::Usable
        );
        assert_eq!((selection.row_start, selection.row_end), (0, row_count));
        let context = selection.context.as_ref().unwrap();
        assert_eq!(context.problem, format!("{key}-problem"));
        assert_eq!(context.revision, revision);
        let mut typed = Vec::new();
        while let Some(batch) = reader.next_relation_batch().await.unwrap() {
            typed.extend(modeling_reports::Row::rows(&batch).unwrap());
        }
        assert_eq!(
            typed, expected,
            "typed protected read retains complete original reports"
        );
        drop(reader);
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("original-water.arrow");
        let mut export = runtime
            .results(
                &run_key,
                &attempt_key,
                "runtime.modeling_reports",
                0,
                row_count,
                token.clone(),
            )
            .await
            .unwrap();
        export
            .export_ipc_bounded(&path, 8 << 20, &token)
            .await
            .unwrap();
        drop(export);
        let file = std::fs::File::open(&path).unwrap();
        let bytes = file.metadata().unwrap().len();
        crate::workflow::portable_payload::validate_scientific_ipc(
            file,
            bytes,
            8 << 20,
            &selection,
        )
        .unwrap();
        let mut portable =
            StreamReader::try_new(std::fs::File::open(&path).unwrap(), None).unwrap();
        let schema = portable.schema();
        let encoded = &schema.metadata()[crate::workflow::result_selection::SELECTION_METADATA_KEY];
        let wire = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        let world = Arc::new(
            DescriptorPool::decode(crate::recorded_result_projection::DESCRIPTORS).unwrap(),
        );
        let message = world
            .message_index("pse.engineering.v1.RecordedResultSelection")
            .unwrap();
        let metadata = DynamicMessage::decode(world, message, &wire).unwrap();
        for (tag, text) in [
            (1, run_key.as_str()),
            (2, attempt_key.as_str()),
            (3, "runtime.modeling_reports"),
        ] {
            assert_eq!(
                metadata.field_by_number(tag),
                Some(&WireValue::String(text.into()))
            );
        }
        let Some(WireValue::Message(usability)) = metadata.field_by_number(10) else {
            panic!("original usability absent")
        };
        assert!(matches!(
            usability.field_by_number(2),
            Some(WireValue::Message(_))
        ));
        for tag in [1, 3, 4, 5] {
            assert!(usability.field_by_number(tag).is_none());
        }
        crate::workflow::portable_payload::validate_scientific_lineage(schema.as_ref(), &selection)
            .unwrap();
        let mut foreign = selection.clone();
        foreign.context.as_mut().unwrap().revision = "foreign-water-revision".into();
        assert!(
            crate::workflow::portable_payload::validate_scientific_lineage(
                schema.as_ref(),
                &foreign
            )
            .is_err()
        );
        let mut seen = BTreeSet::new();
        for batch in &mut portable {
            let batch = batch.unwrap();
            let ids = |name| {
                batch
                    .column_by_name(name)
                    .unwrap()
                    .as_any()
                    .downcast_ref::<FixedSizeBinaryArray>()
                    .unwrap()
            };
            let values = batch
                .column_by_name("value")
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            let steps = batch
                .column_by_name("step")
                .unwrap()
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap();
            let labels = batch
                .column_by_name("label")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let paths = batch
                .column_by_name("path")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            for index in 0..batch.num_rows() {
                let target =
                    SemanticId::from_bytes(ids("target_id").value(index).try_into().unwrap());
                let source =
                    SemanticId::from_bytes(ids("source_id").value(index).try_into().unwrap());
                assert!(
                    seen.insert((target, source)),
                    "portable duplicate original report"
                );
                let row = expected
                    .iter()
                    .find(|row| row.target_id == target && row.source_id.as_id() == source)
                    .unwrap();
                for (name, id) in [
                    ("run_id", row.run_id.as_id()),
                    ("quantity_id", row.quantity_id),
                    ("unit_id", row.unit_id),
                ] {
                    assert_eq!(ids(name).value(index), id.as_bytes());
                }
                assert_eq!(steps.value(index), row.step);
                assert_eq!(values.value(index).to_bits(), row.value.to_bits());
                assert_eq!(labels.value(index), row.label);
                assert_eq!(paths.value(index), row.path);
            }
        }
        assert_eq!(
            seen.len(),
            expected.len(),
            "independent portable reader consumes every original Water report"
        );
    })
    .catch_unwind()
    .await;
    runtime.close();
    runtime.clear_program_cache();
    let cleanup = runtime
        .shared
        .math()
        .shutdown_native_lanes()
        .await
        .map_err(WorkflowError::from);
    match outcome {
        Ok(()) => cleanup,
        Err(original) => {
            if let Err(error) = cleanup {
                eprintln!(
                    "Water consumer original assertion retained; native settlement failed: {error:?}"
                );
            }
            std::panic::resume_unwind(original)
        }
    }
}

#[tokio::test]
async fn authored_liquid_pump_original_native_outcome_survives_protected_export()
-> Result<(), WorkflowError> {
    protected_steam_case("liquid_pump").await
}

#[tokio::test]
async fn authored_vapor_turbine_original_native_outcome_survives_protected_export()
-> Result<(), WorkflowError> {
    protected_steam_case("vapor_turbine").await
}

#[tokio::test]
async fn authored_hot_vapor_turbine_original_native_outcome_survives_protected_export()
-> Result<(), WorkflowError> {
    protected_steam_case("hot_vapor_turbine").await
}

#[tokio::test]
async fn authored_wet_water_turbine_original_native_outcome_survives_protected_export()
-> Result<(), WorkflowError> {
    protected_steam_case("wet_water_turbine").await
}

#[tokio::test]
async fn document_ingress_uses_explicit_prepared_water_without_a_second_source_path() {
    use pse_admission::authoring_driver::document::{
        OwnedDocumentSet, load_package_documents_owned,
    };
    let runtime = fixture::runtime();
    let physical = steam_physical(&runtime).await.unwrap();
    let provider = coolprop::registration(physical.quantities(), WaterRegime::Vapor).unwrap();
    let source = format!(
        "package water_point {{ fn pressure(T:Temperature,rho:Density)->Pressure external \"coolprop8-water-vapor\" revision \"{}\" data \"{}\" output 0 derivatives 2 source analytic smoothness 2; def Point {{ var result:Pressure; eq binding:result==pressure(500{{K}},{}{{mol/m^3}}); annotation start result(100000{{Pa}}); annotation nominal result(100000{{Pa}}); }} test point fixture {{ dof 0; route steady; procedure solve; }} {{ child root:Point=Point(); expect root.result==99967.9423{{Pa}} tolerance 0.02{{Pa}}; }} }}",
        provider.spec().revision,
        provider.spec().data,
        0.435 / 0.018_015_268,
    );
    let manifest =
        include_str!("../../../../../tests/fixtures/packages/minimal_explicit/package.toml")
            .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
            .replace("dependencies = []", &format!(
                "dependencies = [{{ package_id = \"{}\", version_req = {{ operator = \"exact\", major = 1, minor = 0, patch = 0 }} }}]",
                physical.package.as_ref().unwrap().id,
            ));
    let pool = runtime.shared.pool();
    let token = pse_columnar::CancellationToken::new();
    let bundle = load_package_documents_owned(
        &BTreeMap::from([
            ("package.toml".into(), manifest.into_bytes()),
            ("models/water-point.pse".into(), source.into_bytes()),
        ]),
        runtime.registry(),
        Default::default(),
        &pool,
        &token,
        &runtime.validation_context(),
    )
    .unwrap();
    let documents = OwnedDocumentSet::try_from_bundles(vec![bundle], &pool, &token).unwrap();
    let package = runtime
        .modeling_from_documents_registered(
            &documents,
            physical,
            BTreeMap::from([(WaterRegime::Vapor.implementation().to_owned(), provider)]),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let services = runtime.execution_services();
    let selected = package
        .declarations(services.preparation())
        .await
        .unwrap()
        .iter()
        .find(|row| row.name == "point")
        .unwrap()
        .declaration_id;
    let report = package
        .conform(
            &services,
            ModelingConformancePolicy {
                compiler: fixture::compiler_profile(),
                solver: fixture::profile(),
                numerical: Default::default(),
                limits: Default::default(),
                derivatives: pse_backend_native::derivative_diagnostics::Policy {
                    perturbation: 1e-6,
                    relative_tolerance: 1e-4,
                    maximum_cells: 100,
                },
                maximum_fixtures: 1,
                maximum_checks: 16,
                fixtures: ModelingFixtureSelection::Selected(BTreeSet::from([selected])),
                diagnostics: None,
            },
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    assert!(
        report.passed(),
        "{:?}; failures={:?}",
        report.checks,
        report.failures
    );
    runtime.close();
    runtime.clear_program_cache();
    runtime.shared.math().shutdown_native_lanes().await.unwrap();
}
