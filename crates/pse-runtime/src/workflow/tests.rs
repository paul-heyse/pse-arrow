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
    runtime_on(
        pool_bytes,
        crate::math::MathPolicy {
            worker_bytes: 8 << 20,
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
        .with_durability(Durability::Ephemeral)
}
pub(crate) fn canonical_deployment() -> CanonicalDeployment {
    CanonicalDeployment::new(
        pse_operations::testing::canonical_fixture_store().unwrap(),
        OuterAttestation {
            source: Some(pse_ids::ContentHash::from_bytes([0; 32])),
            build: pse_ids::ContentHash::from_bytes([1; 32]),
        },
        None,
    )
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
