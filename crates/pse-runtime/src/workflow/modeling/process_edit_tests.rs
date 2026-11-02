// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public typed process edits and topology retain original physical identities.
use super::*;
use pse_compiler::workspace::ModelingFlowSelection;
use pse_structural::flowsheet::{Decision, Policy};

const SOURCE: &str = "package p {
def Unit { var x:Scalar;
port in1:Scalar=x; annotation connectivity in1(1,0);
port in2:Scalar=x; annotation connectivity in2(1,0);
port out1:Scalar=x; annotation connectivity out1(0,1);
port out2:Scalar=x; annotation connectivity out2(0,1); }
def Root {child a=Unit(); child b=Unit(); child isolated=Unit();
connect first:a.out1 -> b.in1; connect second:a.out2 -> b.in2; }}";

async fn topology(
    runtime: &Runtime,
    package: &ModelingPackage,
) -> pse_structural::flowsheet::Declaration {
    let services = runtime.preparation_services();
    let root = package
        .declarations(&services)
        .await
        .unwrap()
        .iter()
        .find(|declaration| declaration.name == "Root")
        .unwrap()
        .declaration_id;
    let instance = pse_modeling::specialize::root_instance(root);
    let cancel = crate::CancelSource::new();
    let prepared = package
        .prepare(
            &services,
            root,
            instance,
            Bindings::default(),
            Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    let model = &prepared.compiled().model;
    let selection = ModelingFlowSelection {
        nodes: model
            .ports
            .values()
            .map(|port| port.lineage.instance)
            .collect(),
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.,
                        policy: Policy::Free,
                    },
                )
            })
            .collect(),
    };
    let analysis = ModelingAnalysis {
        root,
        instance,
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: Default::default(),
        order: pse_kernels::DerivativeOrder::Value,
        compiler: super::super::tests::compiler_profile(),
        solver: super::super::tests::profile(),
        numerical: Default::default(),
    };
    package
        .prepare_flow(&services, &analysis, selection, &cancel)
        .await
        .unwrap()
        .graph()
        .declaration()
        .clone()
}

#[tokio::test]
async fn process_typed_edits_preserve_parallel_occurrences_isolate_and_parent() {
    let runtime = super::super::tests::runtime();
    let declarations = pse_authoring::language::parse(
        SOURCE,
        SemanticId::from_bytes([81; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let package = runtime
        .modeling_package(declarations.clone(), super::super::tests::physical())
        .await
        .unwrap();
    let original_revision = package.revision_id().to_owned();
    let before = topology(&runtime, &package).await;
    assert_eq!(
        before.nodes.len(),
        3,
        "isolate is part of the selected physical inventory"
    );
    assert_eq!(before.connections.len(), 2);
    assert_ne!(before.connections[0].id, before.connections[1].id);
    assert_eq!(
        (before.connections[0].from, before.connections[0].to),
        (before.connections[1].from, before.connections[1].to)
    );

    let mut renamed = declarations.clone();
    let connection = renamed
        .iter_mut()
        .find(|declaration| declaration.name == "first")
        .unwrap();
    let original_connection_id = connection.declaration_id;
    connection.name = "display_name".into();
    let edited = package
        .with_declarations(&runtime.source_services(), renamed)
        .await
        .unwrap();
    assert_eq!(
        topology(&runtime, &edited).await,
        before,
        "a label does not change the port/connection occurrence identities"
    );
    assert_eq!(package.revision_id(), original_revision);
    assert!(
        edited
            .declarations(&runtime.preparation_services())
            .await
            .unwrap()
            .iter()
            .any(|row| row.name == "display_name" && row.declaration_id == original_connection_id)
    );

    let mut invalid = declarations;
    invalid
        .iter_mut()
        .find(|declaration| declaration.name == "first")
        .unwrap()
        .value
        .connection
        .as_mut()
        .unwrap()
        .to = "b.absent_port".into();
    let result = package
        .with_declarations(&runtime.source_services(), invalid)
        .await;
    // Contextual declaration admission may defer instance member checking until the
    // selected process is prepared. Either boundary must refuse the same invalid edit.
    let error = match result {
        Err(error) => error,
        Ok(invalid_package) => {
            let services = runtime.preparation_services();
            let root = invalid_package
                .declarations(&services)
                .await
                .unwrap()
                .iter()
                .find(|row| row.name == "Root")
                .unwrap()
                .declaration_id;
            invalid_package
                .prepare(
                    &services,
                    root,
                    pse_modeling::specialize::root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                    &crate::CancelSource::new(),
                )
                .await
                .unwrap_err()
        }
    };
    assert!(error.to_string().contains("absent_port"));
    assert!(
        error
            .boundary_diagnostic()
            .sources
            .contains(&original_connection_id.as_id()),
        "invalid connection remains attributable to its original declaration"
    );
    assert_eq!(package.revision_id(), original_revision);
    assert_eq!(topology(&runtime, &package).await, before);
}
