// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual native selected mathematics and original protected readback across service retirement.
use super::*;
use crate::workflow::{AnalyticalOperationRequest, MathematicalRequest, RunOutcome};
#[tokio::test]
async fn native_analytical_original_run_retains_printer_and_refuses_rebound_submission() {
    let store = pse_operations::native_fixture::canonical_fixture_store().unwrap();
    let (service, _spill) = super::super::tests::service_on(Some(store.clone()));
    let mut source = super::super::tests::source_input();
    source.modeling[0].files.insert(
        "models/root.pse".into(),
        b"package local {def Root {var x:Scalar;let f:Scalar=x*x+2*x;}}".to_vec(),
    );
    service.admit_source(source.clone()).await.unwrap();
    let Outcome::Source { package, selection } = service
        .operation(&source.operation, Kind::Source)
        .unwrap()
        .completed()
        .await
        .unwrap()
    else {
        panic!("source")
    };
    let headers = service
        .inspect_source(
            EngineeringInspectSource {
                operation: source.operation.clone(),
                offset: 0,
                limit: 128,
            },
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let root = pse_model::generated::identities::DeclarationId::from_id(
        headers
            .declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration,
    );
    let instance = pse_modeling::specialize::root_instance(root);
    let bindings = pse_modeling::Bindings {
        demand: vec!["x".into(), "f".into()],
        ..Default::default()
    };
    let model = package
        .prepare(
            &service.runtime.preparation_services(),
            root,
            instance,
            bindings.clone(),
            Default::default(),
            &CancelSource::new(),
        )
        .await
        .unwrap();
    let x = model.compiled().model.paths["x"];
    let f = model.compiled().model.paths["f"];
    let request = EngineeringExecuteMathematical {
        operation: "original-analytical".into(),
        run: SemanticId::from_bytes([171; 16]),
        attempt: SemanticId::from_bytes([172; 16]),
        source_operation: source.operation.clone(),
        expected_local_revision: selection.local_revision,
        request: MathematicalRequest::Analytical(AnalyticalOperationRequest {
            root,
            instance,
            bindings,
            member: f,
            operation: pse_compiler::workspace::AnalyticalOperation::Derivative {
                coordinates: vec![x],
            },
            limits: Default::default(),
        }),
        preparation: Default::default(),
    };
    service.execute_mathematical(request.clone()).await.unwrap();
    let original = service.operation(&request.operation, Kind::Run).unwrap();
    let Outcome::Run { handle, .. } = original.completed().await.unwrap() else {
        panic!("mathematical run")
    };
    let result = handle.wait().await.unwrap();
    let RunOutcome::Mathematical {
        result: Ok(product),
        ..
    } = result.outcome()
    else {
        panic!("mathematical result: {:?}", result.report())
    };
    assert!(product.succeeded && product.supported);
    assert_eq!(product.claim, "exact_derivative");
    assert!(!result.usable());
    let formula = product
        .native_views
        .iter()
        .find(|view| view.path == "result.result")
        .unwrap()
        .latex
        .clone();
    assert!(!formula.is_empty());
    let read = EngineeringReadResult {
        operation: request.operation.clone(),
        run: request.run,
        attempt: request.attempt,
        relation: "runtime.mathematical_products".into(),
        start: 0,
        end: 1,
        maximum_bytes: 1 << 20,
    };
    let page = service
        .read_result(read.clone(), &CancelSource::new())
        .await
        .unwrap();
    assert_eq!(page.page.rows, 1);
    let bytes = page.page.ipc.clone();
    assert!(
        bytes
            .windows(formula.len())
            .any(|window| window == formula.as_bytes())
    );
    drop(page);
    drop(result);
    drop(model);
    drop(package);
    service.shutdown().await.unwrap();
    drop(original);
    drop(handle);
    drop(service);
    let (reader, _reader_spill) = super::super::tests::service_on(Some(store.clone()));
    let retained = reader.execute_mathematical(request.clone()).await.unwrap();
    assert!(retained.settled);
    assert_eq!(retained.usable, Some(false));
    assert!(
        reader.operations.lock().unwrap().is_empty(),
        "canonical repetition cannot construct science or another native operation"
    );
    let mut changed = source;
    changed.operation = "changed-source".into();
    changed.modeling[0].files.insert(
        "models/root.pse".into(),
        b"package local {def Root {var x:Scalar;let f:Scalar=x*x+3*x;}}".to_vec(),
    );
    reader.admit_source(changed.clone()).await.unwrap();
    let Outcome::Source { selection, .. } = reader
        .operation(&changed.operation, Kind::Source)
        .unwrap()
        .completed()
        .await
        .unwrap()
    else {
        panic!("evolved source")
    };
    let mut rebound = request;
    rebound.source_operation = changed.operation;
    rebound.expected_local_revision = selection.local_revision;
    assert!(reader.execute_mathematical(rebound).await.is_err());
    let reopened = reader
        .read_result(read.clone(), &CancelSource::new())
        .await
        .unwrap();
    assert_eq!(reopened.page.ipc, bytes);
    let mut wrong = read;
    wrong.attempt = SemanticId::from_bytes([173; 16]);
    assert!(
        reader
            .read_result(wrong, &CancelSource::new())
            .await
            .is_err()
    );
    drop(reopened);
    reader.shutdown().await.unwrap();
    drop(reader);
    store.remove_isolated_fixture().await.unwrap();
}
