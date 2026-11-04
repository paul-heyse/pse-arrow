// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual source/type/domain inspection and complete finite producer controls.

use super::*;
use crate::workflow::{ModelingPackage, OperationSource, PreparationSettings, Runtime};
use pse_contract::boundary_projection::{ProjectionGraph, Shape, ValidateProjection as _};
use pse_modeling::{Type, resolved::CarrierKind, specialize::Value};

pub(super) const FINITE: &str = "package p {
    entity kind item { attribute below:Set<item>; }
    entity item a { below={} } entity item b { below={b,a} }
    set ordered:Set<item>={b,a}; set empty:Set<item>={};
    def Root { var x[i in ordered,j in i.below]:Scalar; var unused[j in empty]:Scalar;
        alias shared:Scalar=x[b,a]; eq selected[i in ordered,j in i.below]:x[i,j]==1;
    }
    test view fixture { dof 0; route steady; procedure solve; } { child root:Root=Root(); }
}";

pub(super) async fn admitted(
    runtime: &Runtime,
    source: &str,
) -> (
    ModelingPackage,
    crate::workflow::modeling::DeclaredExecution,
    PreparationSettings,
) {
    let rows = pse_authoring::language::parse(
        source,
        SemanticId::from_bytes([151; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        Default::default(),
    )
    .unwrap();
    let case = rows
        .iter()
        .find(|row| row.name == "view")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(rows, crate::workflow::tests::physical())
        .await
        .unwrap();
    let preparation = PreparationSettings {
        compiler: crate::workflow::tests::compiler_profile(),
        ..Default::default()
    };
    let execution = package
        .admit_declared_request(
            &runtime.preparation_services(),
            case,
            &preparation,
            Default::default(),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    (package, execution, preparation)
}

#[tokio::test]
async fn modeling_inspection_returns_original_types_and_finite_carriers_under_selected_custody() {
    let runtime = crate::workflow::tests::runtime();
    let (package, execution, preparation) = admitted(&runtime, FINITE).await;
    let services = runtime.preparation_services();
    let source = &execution.model.compiled().model;
    let original_domains = source.finite_facts.domains().collect::<Vec<_>>();
    assert!(
        original_domains
            .iter()
            .any(|(_, carrier)| carrier.members.is_empty())
    );
    assert!(
        original_domains
            .iter()
            .any(|(_, carrier)| carrier.members.len() == 2)
    );
    assert!(
        original_domains
            .iter()
            .any(|(key, _)| key.position == 1 && key.prefix.len() == 1),
        "actual dependent domains retain their preceding coordinate context"
    );
    let original_effects = runtime.native().test_admission_snapshot();
    let pool = services.pool();
    let before = pool.reserved();
    let document = bound_observation(
        execution
            .inspection(
                &services,
                OperationSource::of(&package),
                preparation.clone(),
                0,
                128,
            )
            .unwrap(),
    )
    .unwrap();
    document.validate_projection().unwrap();
    assert!(pool.reserved() > before);
    assert!(document.payload_owner().is_some());
    assert_eq!(document.source, OperationSource::of(&package));
    assert_eq!(document.population.domains, original_domains.len());
    assert_eq!(document.domains.len(), original_domains.len());
    assert_eq!(document.population.members, source.symbols.len());
    for (selected, (key, carrier)) in document.domains.iter().zip(&original_domains) {
        assert_eq!(&selected.key, *key);
        assert_eq!(&selected.carrier, *carrier);
        assert_eq!(selected.carrier.kind, CarrierKind::OrderedSet);
        assert!(matches!(selected.carrier.element, Type::Entity(_)));
        if !carrier.members.is_empty() {
            assert_eq!(carrier.members.len(), 2);
            assert!(matches!(&carrier.members[0], Value::Entity { .. }));
            assert_ne!(carrier.members[0], carrier.members[1]);
        }
    }
    for (selected, original) in document.members.iter().zip(source.symbols.values()) {
        assert_eq!(selected.id, original.id);
        assert_eq!(selected.ty, original.ty);
        assert_eq!(selected.lineage.path, original.lineage.path);
    }
    let mut offset = 0;
    let mut selected_domains = Vec::new();
    loop {
        let page = bound_observation(
            execution
                .inspection(
                    &services,
                    OperationSource::of(&package),
                    preparation.clone(),
                    offset,
                    1,
                )
                .unwrap(),
        )
        .unwrap();
        assert_eq!(page.population.domains, original_domains.len());
        assert!(page.domains.len() <= 1 && page.members.len() <= 1);
        selected_domains.extend(
            page.domains
                .iter()
                .map(|domain| (domain.key.clone(), domain.carrier.clone())),
        );
        let next = page.next_offset;
        drop(page);
        match next {
            Some(next) => {
                assert!(next > offset);
                offset = next;
            }
            None => break,
        }
    }
    assert_eq!(selected_domains.len(), original_domains.len());
    for ((selected_key, selected_carrier), (key, carrier)) in
        selected_domains.iter().zip(&original_domains)
    {
        assert_eq!(selected_key, *key);
        assert_eq!(selected_carrier, *carrier);
    }
    let empty = execution
        .inspection(
            &services,
            OperationSource::of(&package),
            preparation,
            usize::MAX,
            1,
        )
        .unwrap();
    assert!(empty.domains.is_empty() && empty.members.is_empty() && empty.next_offset.is_none());
    assert_eq!(empty.population.domains, original_domains.len());
    drop(empty);
    drop(document);
    assert_eq!(pool.reserved(), before);
    assert_eq!(
        runtime.native().test_admission_snapshot(),
        original_effects,
        "selected observation never dispatches or reserves a numerical worker"
    );
}

#[tokio::test]
async fn modeling_inspection_enforces_shared_limits_and_original_attribution_for_direct_native_calls()
 {
    let runtime = crate::workflow::tests::runtime();
    let (package, execution, preparation) = admitted(&runtime, FINITE).await;
    let services = runtime.preparation_services();
    for limit in [0, 129] {
        let error = execution
            .inspection(
                &services,
                OperationSource::of(&package),
                preparation.clone(),
                0,
                limit,
            )
            .unwrap_err();
        let crate::workflow::WorkflowError::Typed(cause) = error else {
            panic!("preserve original local rule")
        };
        assert!(
            matches!(cause.as_error().downcast_ref::<pse_contract::boundary_projection::ValidationError>(),
            Some(pse_contract::boundary_projection::ValidationError::Rule { path, rule: "unsigned.range", .. }) if path == "limit")
        );
    }
    let mut foreign = OperationSource::of(&package);
    foreign.physical_context = pse_ids::ContentHash::NIL;
    assert!(
        execution
            .inspection(&services, foreign, preparation.clone(), 0, 1)
            .is_err()
    );
    let mut changed = preparation;
    changed.limits.depth += 1;
    assert!(
        execution
            .inspection(&services, OperationSource::of(&package), changed, 0, 1)
            .is_err()
    );
    let graph = ProjectionGraph::of::<ModelingInspection>().unwrap();
    let Shape::Record(fields) = &graph.definitions()["ModelingInspection"].shape else {
        panic!("actual native output")
    };
    for name in ["instances", "members", "ports", "connections", "domains"] {
        assert_eq!(
            fields
                .iter()
                .find(|field| field.name == name)
                .unwrap()
                .rules,
            [INSPECTION_MEMBERS]
        );
    }
    let schema = schemars::schema_for!(ModelingInspection).to_value();
    for name in ["instances", "members", "ports", "connections", "domains"] {
        assert_eq!(
            schema["properties"][name]["maxItems"],
            MAXIMUM_INSPECTION_MEMBERS
        );
    }
    let before = services.pool().reserved();
    runtime.close();
    assert!(
        execution
            .inspection(
                &services,
                OperationSource::of(&package),
                PreparationSettings {
                    compiler: crate::workflow::tests::compiler_profile(),
                    ..Default::default()
                },
                0,
                1
            )
            .is_err(),
        "a closed original caller cannot admit a fresh selected read"
    );
    assert_eq!(services.pool().reserved(), before);
}

#[tokio::test]
async fn modeling_inspection_counts_selected_nested_carriers_before_cloning_and_releases_its_grant()
{
    let runtime = crate::workflow::tests::runtime();
    let label = "x".repeat(48 << 10);
    let source = format!("package p {{ set names:Set<Text>={{\"{label}\"}}; set empty:Set<Integer>={{}};
        def Root {{ var y:Scalar; eq balance:y==1;
            var x[j in names,k in empty]:Scalar; eq selected[j in names,k in empty]:x[j,k]==1; }}
        test view fixture {{ dof 0; route steady; procedure solve; }} {{ child root:Root=Root(); }} }}");
    let (package, execution, preparation) = admitted(&runtime, &source).await;
    assert!(
        execution
            .model
            .compiled()
            .model
            .finite_facts
            .domains()
            .count()
            >= 2
    );
    let services = runtime.preparation_services();
    let pool = services.pool();
    let before = pool.reserved();
    assert!(matches!(
        execution.inspection(
            &services,
            OperationSource::of(&package),
            preparation.clone(),
            0,
            128
        ),
        Err(crate::workflow::WorkflowError::Resource(_))
    ));
    assert_eq!(
        pool.reserved(),
        before,
        "an overlarge original cut refuses before selected metadata allocation"
    );
    let narrowed = bound_observation(
        execution
            .inspection(&services, OperationSource::of(&package), preparation, 0, 1)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(narrowed.domains.len(), 1);
    assert!(pool.reserved() > before);
    drop(narrowed);
    assert_eq!(pool.reserved(), before);
}

#[tokio::test]
async fn modeling_inspection_resolved_physical_metadata_is_readable_without_granting_native_adoption()
 {
    let runtime = crate::workflow::tests::runtime();
    let (package, execution, preparation) = admitted(&runtime, FINITE).await;
    let document = execution
        .inspection(
            &runtime.preparation_services(),
            OperationSource::of(&package),
            preparation,
            0,
            1,
        )
        .unwrap();
    let registry = &package.physical_context().quantities;
    let quantity = registry.neutral_dimensionless().unwrap();
    let actual = pse_quantity::ResolvedPhysicalContract::named(
        quantity,
        pse_quantity::IndexSet::new(),
        registry,
    )
    .unwrap();
    let mut metadata = document.domains[0].clone();
    metadata.carrier.element =
        Type::Quantity(pse_quantity::scheme::Scheme::Resolved(Box::new(actual)));
    let passive = serde_json::to_value(&metadata).unwrap();
    assert!(passive["carrier"]["element"].is_object());
    let error = serde_json::from_value::<InspectionDomain>(passive).unwrap_err();
    assert!(
        error.to_string().contains("qualified"),
        "unqualified receipt restoration must refuse: {error}"
    );
}
