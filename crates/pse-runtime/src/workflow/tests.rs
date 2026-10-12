// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::math::solves::SolverProfile;
use pse_backend_native::execution::BackendSettings;
use pse_backend_native::solve::*;
use pse_ids::SemanticId;
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
pub(super) fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
/// Read the engineering allowance frozen by production admission for this target.
/// Tests must not substitute a coordinate nominal or a private solver tolerance.
pub(crate) fn engineering_target(
    numerics: &pse_model::numerics::ResolvedNumericalPolicy,
    kind: pse_relations::generated::enums::NumericalTarget,
    id: SemanticId,
) -> &pse_model::numerics::ResolvedTarget {
    let target = numerics
        .targets
        .iter()
        .find(|target| target.kind == kind && target.id == id)
        .expect("production-resolved numerical target");
    let context = target
        .engineering
        .as_ref()
        .expect("engineering interpretation for ordinary production fixture");
    assert!(context.budget.is_finite() && context.budget > 0.);
    assert!(target.coordinate_scale.is_finite() && target.coordinate_scale > 0.);
    target
}
/// The typed ADR-0103 refusal of a free discrete variable: its instance path's last
/// segment and the refusing analysis. The class is always `unsupported`.
pub(super) fn free_discrete_refusal(error: &WorkflowError) -> (String, String) {
    use pse_model::diagnostic::{BoundaryClass, Observation};
    let diagnostic = error.boundary_diagnostic();
    assert_eq!(diagnostic.class, BoundaryClass::Unsupported, "{error}");
    assert_eq!(
        diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingDomain,
        "{error}"
    );
    let text = |name: &str| match diagnostic.observations.get(name) {
        Some(Observation::Text(value)) => value.clone(),
        other => panic!("{name}: {other:?}"),
    };
    assert_eq!(text("reason"), pse_modeling::DomainRefusal::Free.as_str());
    let path = text("variable");
    (
        path.rsplit('.').next().unwrap_or_default().to_owned(),
        text("analysis"),
    )
}
pub(crate) fn runtime() -> Runtime {
    runtime_with_workspace(16 << 20)
}
pub(super) fn runtime_with_workspace(workspace_bytes: usize) -> Runtime {
    runtime_with(workspace_bytes, 1 << 20, 512 << 20)
}
/// A runtime whose native jobs admit `foreign_bytes` of library-owned memory within a
/// `pool_bytes` memory pool; SCIP takes its memory limit from the foreign allowance.
pub(crate) fn runtime_with(
    workspace_bytes: usize,
    foreign_bytes: usize,
    pool_bytes: usize,
) -> Runtime {
    // Ordinary fixtures need enough capacity for opaque construction bounds as
    // well as known native demand. Deliberate capacity controls use runtime_on.
    let workspace_bytes = workspace_bytes.max(1 << 30);
    let pool_bytes = pool_bytes.max(
        workspace_bytes
            .checked_mul(4)
            .expect("ordinary fixture memory pool extent"),
    );
    runtime_on(
        pool_bytes,
        crate::math::MathPolicy {
            worker_bytes: workspace_bytes,
            workspace_bytes,
            foreign_bytes,
            ..Default::default()
        },
    )
}
pub(crate) fn runtime_on(memory: usize, math: crate::math::MathPolicy) -> Runtime {
    runtime_on_threads(memory, math, 2)
}
pub(crate) fn runtime_on_threads(
    memory: usize,
    math: crate::math::MathPolicy,
    pool_threads: usize,
) -> Runtime {
    let shared = shared_runtime_on_threads(memory, math, pool_threads);
    let registry = pse_schema::shared_registry().unwrap();
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
    );
    Runtime::from_shared(
        shared,
        sessions.bind_model(registry).unwrap(),
        canonical_deployment(),
    )
    .unwrap()
    .with_durability(DurabilitySelection::Ephemeral)
    .unwrap()
}
fn shared_runtime_on(memory: usize, math: crate::math::MathPolicy) -> Arc<SharedRuntime> {
    shared_runtime_on_threads(memory, math, 2)
}
fn shared_runtime_on_threads(
    memory: usize,
    math: crate::math::MathPolicy,
    pool_threads: usize,
) -> Arc<SharedRuntime> {
    let n = |v| NonZeroUsize::new(v).unwrap();
    SharedRuntime::build(crate::ResourceBudget {
        memory_limit_bytes: n(memory),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(pool_threads),
            target_partitions: n(1),
        },
        execution: Default::default(),
        cache: crate::CacheBudget::disabled(1024),
        math,
        hashing_may_use_pool: false,
    })
    .unwrap()
}
pub(crate) fn canonical_deployment() -> CanonicalDeployment {
    let store = match std::env::var("PSE_TEST_EXECUTION_PROFILE") {
        Ok(profile) => pse_operations::testing::canonical_managed_fixture_store(&profile),
        Err(_) => pse_operations::testing::canonical_fixture_store(),
    }
    .unwrap();
    CanonicalDeployment::new(
        store,
        OuterAttestation {
            source: Some(pse_ids::ContentHash::from_bytes([0; 32])),
            build: pse_ids::ContentHash::from_bytes([1; 32]),
        },
        None,
    )
}

#[test]
fn deployment_check_preserves_custom_same_owner_factory_and_partition_demand() {
    #[derive(Debug)]
    struct Custom;
    let shared = shared_runtime_on(16 << 30, Default::default());
    let sessions = shared
        .session_factory(pse_engine::session::native_engine_profile())
        .unwrap();
    let env = shared.runtime_env();
    // A distinct outer wrapper is valid when its consumed services are unchanged.
    let wrapper = Arc::new(datafusion::execution::runtime_env::RuntimeEnv {
        memory_pool: env.memory_pool.clone(),
        disk_manager: env.disk_manager.clone(),
        cache_manager: env.cache_manager.clone(),
        object_store_registry: env.object_store_registry.clone(),
    });
    let builder = datafusion::execution::session_state::SessionStateBuilder::new_from_existing(
        sessions.native_state().clone(),
    );
    let factory = EngineFactory::from_builder(wrapper, shared.pool(), "custom", builder)
        .with_extension(Arc::new(Custom))
        .with_target_partitions(NonZeroUsize::new(3).unwrap());
    assert!(!Arc::ptr_eq(factory.native_state().runtime_env(), &env));
    assert!(Arc::ptr_eq(factory.pool(), &shared.pool()));
    Runtime::check_deployment(&shared, &factory).unwrap();
    assert!(
        factory
            .native_state()
            .config()
            .get_extension::<Custom>()
            .is_some()
    );
    assert_eq!(
        factory
            .native_state()
            .config()
            .options()
            .execution
            .target_partitions,
        3
    );
}

#[test]
fn deployment_check_refuses_foreign_consumed_owners_and_width() {
    let foreign = shared_runtime_on(16 << 30, Default::default());
    let shared = shared_runtime_on(16 << 30, Default::default());
    let check = |factory: &EngineFactory| Runtime::check_deployment(&shared, factory);
    let refuses_owner = |factory: &EngineFactory, owner: &str| match check(factory) {
        Err(WorkflowError::Input(message)) => assert_eq!(
            message,
            format!("workflow factory {owner} belongs to another deployment")
        ),
        other => panic!("expected foreign {owner} refusal, got {other:?}"),
    };
    refuses_owner(
        &foreign
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
        "allocation pool",
    );
    let factory = shared
        .session_factory(pse_engine::session::native_engine_profile())
        .unwrap();
    refuses_owner(
        &factory.clone().with_extension(foreign.caches().clone()),
        "cache service",
    );
    let cpu = |permits, workers| Arc::new(pse_engine::resources::CpuAdmission { permits, workers });
    let width =
        std::num::NonZeroU32::new(shared.budget().threads.pool_threads.get() as u32).unwrap();
    refuses_owner(
        &factory
            .clone()
            .with_extension(cpu(foreign.compiler_cpu(), width)),
        "CPU admission",
    );
    for configured in [width.get() - 1, width.get() + 1] {
        refuses_owner(
            &factory.clone().with_extension(cpu(
                shared.compiler_cpu(),
                std::num::NonZeroU32::new(configured).unwrap(),
            )),
            "CPU deployment width",
        );
    }
    let original = shared.runtime_env();
    let other = foreign.runtime_env();
    for changed in 0..4 {
        let env = Arc::new(datafusion::execution::runtime_env::RuntimeEnv {
            memory_pool: if changed == 0 {
                other.memory_pool.clone()
            } else {
                original.memory_pool.clone()
            },
            disk_manager: if changed == 1 {
                other.disk_manager.clone()
            } else {
                original.disk_manager.clone()
            },
            cache_manager: if changed == 2 {
                other.cache_manager.clone()
            } else {
                original.cache_manager.clone()
            },
            object_store_registry: if changed == 3 {
                other.object_store_registry.clone()
            } else {
                original.object_store_registry.clone()
            },
        });
        let builder = datafusion::execution::session_state::SessionStateBuilder::new_from_existing(
            factory.native_state().clone(),
        );
        // The explicit factory pool is consumed by native operators and application
        // allocations alike. A foreign RuntimeEnv pool alone is normalized away by
        // from_builder, so it cannot establish a foreign consumed owner.
        let pool = if changed == 0 {
            foreign.pool()
        } else {
            shared.pool()
        };
        let candidate = EngineFactory::from_builder(env, pool.clone(), "foreign-service", builder);
        assert!(Arc::ptr_eq(candidate.pool(), &pool));
        assert!(Arc::ptr_eq(
            &candidate.native_state().runtime_env().memory_pool,
            &pool
        ));
        refuses_owner(
            &candidate,
            [
                "allocation pool",
                "spill manager",
                "native cache manager",
                "object store registry",
            ][changed],
        );
    }
    // An unused incoming pool is replaced during factory assembly. Judge the
    // resulting consumed owners rather than the discarded wrapper's services.
    let incoming = Arc::new(datafusion::execution::runtime_env::RuntimeEnv {
        memory_pool: other.memory_pool.clone(),
        disk_manager: original.disk_manager.clone(),
        cache_manager: original.cache_manager.clone(),
        object_store_registry: original.object_store_registry.clone(),
    });
    let normalized = EngineFactory::from_builder(
        incoming.clone(),
        shared.pool(),
        "normalized-unused-pool",
        datafusion::execution::session_state::SessionStateBuilder::new_from_existing(
            factory.native_state().clone(),
        ),
    );
    assert!(!Arc::ptr_eq(&incoming.memory_pool, normalized.pool()));
    assert!(Arc::ptr_eq(
        &normalized.native_state().runtime_env().memory_pool,
        normalized.pool()
    ));
    check(&normalized).unwrap();
    // In-flight permits do not change configured deployment width.
    let permits = shared.compiler_cpu();
    let held = permits.try_acquire_many(width.get()).unwrap();
    assert_eq!(permits.available_permits(), 0);
    check(&factory).unwrap();
    drop(held);
    assert_eq!(permits.available_permits(), width.get() as usize);
}

#[test]
fn workflow_composition_binds_same_owner_factory_to_canonical_deployment() {
    let runtime = runtime();
    let sessions = runtime.sessions().clone();
    let checked = Runtime::from_shared(
        runtime.shared.clone(),
        sessions.bind_model(runtime.registry().clone()).unwrap(),
        runtime.canonical.clone(),
    )
    .unwrap();
    assert!(Arc::ptr_eq(checked.sessions(), &sessions));
    assert!(Arc::ptr_eq(&checked.shared, &runtime.shared));
    assert!(Arc::ptr_eq(
        &checked.validation_context(),
        &runtime.validation_context()
    ));
    assert_eq!(
        checked.canonical.store().endpoint(),
        runtime.canonical.store().endpoint()
    );
    assert_eq!(
        checked.canonical.store().namespace(),
        runtime.canonical.store().namespace()
    );
    assert_eq!(
        checked.canonical.store().database(),
        runtime.canonical.store().database()
    );
}

#[test]
fn workflow_durability_selection_uses_this_deployment() {
    let runtime = runtime();
    let policy = LeasePolicy {
        lease: std::time::Duration::from_secs(17),
        heartbeat: std::time::Duration::from_secs(3),
        batch: 11,
        flush: std::time::Duration::from_millis(71),
    };
    let durable = runtime
        .clone()
        .with_durability(DurabilitySelection::Durable {
            worker: "checked-owner".into(),
            policy,
        })
        .unwrap();
    let Durability::Durable(operations) = durable.durability() else {
        panic!("durable selection")
    };
    assert!(Arc::ptr_eq(&operations.pool, &runtime.shared.pool()));
    assert_eq!(
        operations.store().endpoint(),
        runtime.canonical.store().endpoint()
    );
    assert_eq!(
        operations.store().namespace(),
        runtime.canonical.store().namespace()
    );
    assert_eq!(
        operations.store().database(),
        runtime.canonical.store().database()
    );
    assert_eq!(operations.worker(), "checked-owner");
    assert_eq!(operations.policy(), policy);
    let ephemeral = durable
        .with_durability(DurabilitySelection::Ephemeral)
        .unwrap();
    assert!(matches!(ephemeral.durability(), Durability::Ephemeral));
    assert!(Arc::ptr_eq(ephemeral.sessions(), runtime.sessions()));
    assert!(Arc::ptr_eq(&ephemeral.shared, &runtime.shared));
    assert_eq!(
        ephemeral.canonical.store().database(),
        runtime.canonical.store().database()
    );
}

#[test]
fn durability_selection_validates_positive_lease_and_heartbeat_intervals() {
    assert!(DurabilitySelection::Ephemeral.validate().is_ok());
    assert!(
        DurabilitySelection::Durable {
            worker: "valid".into(),
            policy: LeasePolicy::default(),
        }
        .validate()
        .is_ok()
    );
    // Positivity is the production contract; validation must not round a positive
    // sub-millisecond duration to zero or impose an unstated lease/heartbeat order.
    for (lease, heartbeat) in [(1, 1), (1, 2), (2, 1), (0, 1), (1, 0), (0, 0)] {
        let selection = DurabilitySelection::Durable {
            worker: "interval-policy".into(),
            policy: LeasePolicy {
                lease: std::time::Duration::from_nanos(lease),
                heartbeat: std::time::Duration::from_nanos(heartbeat),
                ..Default::default()
            },
        };
        match selection.validate() {
            Ok(()) => assert!(lease > 0 && heartbeat > 0),
            Err(WorkflowError::Input(message)) => {
                assert!(lease == 0 || heartbeat == 0);
                assert_eq!(
                    message,
                    "durable heartbeat and lease intervals must be positive"
                );
            }
            other => panic!("unexpected interval validation: {other:?}"),
        }
    }
}
pub(crate) fn physical() -> PhysicalContext {
    // Fixture only: production requires source-backed PhysicalInventory admission.
    let quantities = Arc::new(pse_quantity::standard::standard_registry().unwrap());
    let preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .unwrap(),
    );
    PhysicalContext {
        key: pse_compiler::workspace::physical_identity(&quantities, &preconditions),
        quantities,
        preconditions,
        sources: BTreeMap::new(),
        package: None,
        _inventory: None,
        _source_owner: None,
    }
}
pub(crate) fn profile() -> SolverProfile {
    SolverProfile {
        presolve: Default::default(),
        numerics: Default::default(),
        convexity: Default::default(),
        intent: SolveIntent::Root,
        selection: SolverSelection::Auto,
        controls: Controls::default(),
        backend: BackendSettings::Default,
        sensitivity: None,
        composition: Default::default(),
        reconstruction: None,
    }
}
pub(crate) fn compiler_profile() -> pse_compiler::workspace::Profile {
    pse_compiler::workspace::Profile {
        evaluation: pse_math::jets::EvaluationLimits {
            scratch_bytes: 1 << 20,
            ..Default::default()
        },
        ..Default::default()
    }
}
