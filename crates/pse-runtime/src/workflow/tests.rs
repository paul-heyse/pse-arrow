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
    let n = |v| NonZeroUsize::new(v).unwrap();
    let shared = SharedRuntime::build(crate::ResourceBudget {
        memory_limit_bytes: n(memory),
        spill_dir: std::env::temp_dir(),
        max_temp_dir_bytes: 1 << 30,
        top_consumers: n(5),
        threads: pse_engine::ThreadBudget {
            pool_threads: n(2),
            target_partitions: n(1),
        },
        execution: Default::default(),
        cache: crate::CacheBudget::disabled(1024),
        math,
        hashing_may_use_pool: false,
    })
    .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let sessions = Arc::new(
        shared
            .session_factory(pse_engine::session::native_engine_profile())
            .unwrap(),
    );
    Runtime::from_shared(shared, registry, sessions, canonical_deployment())
        .unwrap()
        .with_durability(DurabilitySelection::Ephemeral)
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
fn workflow_composition_preserves_custom_same_owner_factory_and_partition_demand() {
    #[derive(Debug)]
    struct Custom;
    let runtime = runtime();
    let shared = runtime.shared.clone();
    let env = shared.runtime_env();
    // A distinct outer wrapper is valid when its consumed services are unchanged.
    let wrapper = Arc::new(datafusion::execution::runtime_env::RuntimeEnv {
        memory_pool: env.memory_pool.clone(),
        disk_manager: env.disk_manager.clone(),
        cache_manager: env.cache_manager.clone(),
        object_store_registry: env.object_store_registry.clone(),
    });
    let builder = datafusion::execution::session_state::SessionStateBuilder::new_from_existing(
        runtime.sessions.native_state().clone(),
    );
    let factory = EngineFactory::from_builder(wrapper, shared.pool(), "custom", builder)
        .with_extension(Arc::new(Custom))
        .with_target_partitions(NonZeroUsize::new(3).unwrap());
    let checked = Runtime::from_shared(
        shared,
        runtime.registry.clone(),
        Arc::new(factory),
        runtime.canonical.clone(),
    )
    .unwrap();
    assert!(
        checked
            .sessions
            .native_state()
            .config()
            .get_extension::<Custom>()
            .is_some()
    );
    assert_eq!(
        checked
            .sessions
            .native_state()
            .config()
            .options()
            .execution
            .target_partitions,
        3
    );
}

#[test]
fn workflow_composition_refuses_foreign_consumed_owners_and_width() {
    let foreign = runtime();
    let runtime = runtime();
    let shared = &runtime.shared;
    let check = |factory| {
        Runtime::from_shared(
            shared.clone(),
            runtime.registry.clone(),
            Arc::new(factory),
            runtime.canonical.clone(),
        )
    };
    assert!(matches!(
        check(foreign.sessions.as_ref().clone()),
        Err(WorkflowError::Input(_))
    ));
    let factory = runtime.sessions.as_ref().clone();
    assert!(matches!(
        check(
            factory
                .clone()
                .with_extension(foreign.shared.caches().clone())
        ),
        Err(WorkflowError::Input(_))
    ));
    let cpu = |permits, workers| Arc::new(pse_engine::resources::CpuAdmission { permits, workers });
    let width =
        std::num::NonZeroU32::new(shared.budget().threads.pool_threads.get() as u32).unwrap();
    assert!(matches!(
        check(
            factory
                .clone()
                .with_extension(cpu(foreign.shared.compiler_cpu(), width))
        ),
        Err(WorkflowError::Input(_))
    ));
    assert!(matches!(
        check(factory.clone().with_extension(cpu(
            shared.compiler_cpu(),
            std::num::NonZeroU32::new(width.get() + 1).unwrap()
        ))),
        Err(WorkflowError::Input(_))
    ));
    let original = shared.runtime_env();
    let other = foreign.shared.runtime_env();
    for changed in 0..3 {
        let env = Arc::new(datafusion::execution::runtime_env::RuntimeEnv {
            memory_pool: original.memory_pool.clone(),
            disk_manager: if changed == 0 {
                other.disk_manager.clone()
            } else {
                original.disk_manager.clone()
            },
            cache_manager: if changed == 1 {
                other.cache_manager.clone()
            } else {
                original.cache_manager.clone()
            },
            object_store_registry: if changed == 2 {
                other.object_store_registry.clone()
            } else {
                original.object_store_registry.clone()
            },
        });
        let builder = datafusion::execution::session_state::SessionStateBuilder::new_from_existing(
            factory.native_state().clone(),
        );
        assert!(matches!(
            check(EngineFactory::from_builder(
                env,
                shared.pool(),
                "foreign-service",
                builder
            )),
            Err(WorkflowError::Input(_))
        ));
    }
    // In-flight permits do not change configured deployment width.
    let permits = shared.compiler_cpu();
    let _permit = permits.try_acquire().unwrap();
    assert!(check(factory).is_ok());
}

#[test]
fn workflow_durability_selection_uses_this_deployment_and_refuses_zero_intervals() {
    let runtime = runtime();
    let durable = runtime
        .clone()
        .with_durability(DurabilitySelection::Durable {
            worker: "checked-owner".into(),
            policy: LeasePolicy::default(),
        })
        .unwrap();
    let Durability::Durable(operations) = durable.durability() else {
        panic!("durable selection")
    };
    assert!(Arc::ptr_eq(&operations.pool, &runtime.shared.pool()));
    assert_eq!(
        operations.store().database(),
        runtime.canonical.store().database()
    );
    assert_eq!(operations.worker(), "checked-owner");
    for lease in [true, false] {
        let mut policy = LeasePolicy::default();
        if lease {
            policy.lease = std::time::Duration::ZERO;
        } else {
            policy.heartbeat = std::time::Duration::ZERO;
        }
        assert!(matches!(
            runtime
                .clone()
                .with_durability(DurabilitySelection::Durable {
                    worker: "invalid".into(),
                    policy
                }),
            Err(WorkflowError::Input(_))
        ));
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
