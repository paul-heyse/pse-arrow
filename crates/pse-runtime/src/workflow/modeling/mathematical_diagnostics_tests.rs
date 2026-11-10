// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual source-aware diagnostics and protected original readback after source evolution.
use super::*;
use crate::workflow::{
    ArrowFieldPath, ArrowFieldStep, MathematicalRequest, PreparationSettings,
    PreparedMathematicalOperation, ResultInspectionCell, ResultInspectionDocument,
    ResultInspectionRefusal, ResultInspectionRequest, ResultInspectionView, RetainedDataServices,
    RunDurability, RunOutcome,
};
use pse_math::mathematical_diagnostics::{
    DiagnosticClaim, DiagnosticInput, DiagnosticLimits, ExactValue,
};

#[tokio::test]
async fn mathematical_diagnostic_original_reader_survives_source_evolution_without_math_services() {
    let runtime = crate::workflow::durable_tests::durable_runtime();
    let source = "package p {def Root {var x:Scalar;let f:Scalar=x*x+2*x;}}";
    let (package, analysis) = crate::workflow::durable_tests::package_on(&runtime, source).await;
    let services = runtime.preparation_services();
    let cancel = crate::CancelSource::new();
    let bindings = Bindings {
        demand: vec!["x".into(), "f".into()],
        ..Default::default()
    };
    let model = package
        .prepare(
            &services,
            analysis.root,
            analysis.instance,
            bindings.clone(),
            Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    let coordinate = model.compiled().model.paths["x"];
    let member = model.compiled().model.paths["f"];
    let row = pse_compiler::workspace::ModelingOutput::Member(member).row_id();
    let operation = MathematicalDiagnosticOperation {
        root: analysis.root,
        instance: analysis.instance,
        bindings,
        case_members: BTreeMap::new(),
        request: DiagnosticRequest {
            rows: vec![row],
            unknowns: vec![coordinate],
            parameters: vec![],
            inputs: vec![DiagnosticInput {
                coordinate,
                lower: ExactValue::integer(2),
                upper: None,
            }],
            derivatives: vec![],
            claim: DiagnosticClaim::ChosenPrecision,
            precision: 160,
            limits: DiagnosticLimits::default(),
        },
    };
    let prepared = PreparedMathematicalOperation {
        source: package.clone(),
        request: MathematicalRequest::Diagnostic(operation.clone()),
        preparation: PreparationSettings::default(),
    };
    let handle = prepared
        .start(&runtime.execution_services(), &cancel)
        .unwrap();
    let run = handle.canonical_run_key().unwrap().to_owned();
    let attempt = handle.canonical_attempt_key().unwrap().to_owned();
    let result = handle.wait().await.unwrap();
    let RunOutcome::Mathematical {
        result: Ok(product),
        ..
    } = result.outcome()
    else {
        panic!(
            "actual diagnostic run must return the mathematical product: {:?}",
            result.report()
        );
    };
    assert!(product.succeeded && product.supported);
    assert_eq!(product.claim, "chosen_precision");
    let expected_formula = product.native_views[0].latex.clone();
    let original_revision = product.source_revision;
    let document: serde_json::Value = serde_json::from_str(&product.result_context).unwrap();
    assert_eq!(
        document["numerical"][0]["value"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        8.0
    );
    let RunDurability::Durable(record) = result.durability() else {
        panic!("actual durable result")
    };
    let manifest = record.manifest.as_ref().unwrap().digest.clone();

    let (evolved, _) =
        crate::workflow::durable_tests::package_on(&runtime, &source.replace("2*x", "3*x")).await;
    let changed = evolved
        .diagnose_selected_mathematics(
            &services,
            &operation,
            &PreparationSettings::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert_eq!(
        changed.numerical[0]
            .value
            .as_ref()
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        10.0
    );
    assert_ne!(changed.mathematical_views[0].latex, expected_formula);

    // This independent capability contains a native model and protected store, and
    // has no MathService, compiler, source builder or execution capability.
    let reader = RetainedDataServices::new(
        runtime.canonical_store().unwrap().clone(),
        runtime.model.clone(),
    )
    .unwrap();
    let store = reader.canonical_store().unwrap().clone();
    let original_attempt = store.canonical_attempt(&attempt).await.unwrap().unwrap();
    drop(changed);
    drop(evolved);
    drop(result);
    drop(prepared);
    drop(model);
    drop(package);
    drop(services);
    runtime.clear_program_cache();
    drop(runtime);
    let request = ResultInspectionRequest {
        run: run.clone(),
        attempt: attempt.clone(),
        relation: "runtime.mathematical_products".into(),
        start: 0,
        end: 1,
        expected_manifest: Some(manifest.clone()),
        cursor: None,
        view: ResultInspectionView::Values {
            fields: vec![
                ArrowFieldPath::from("claim"),
                ArrowFieldPath {
                    field: "native_views".into(),
                    steps: vec![
                        ArrowFieldStep::ListElement { index: 0 },
                        ArrowFieldStep::StructField {
                            name: "latex".into(),
                        },
                    ],
                },
            ],
            maximum_rows: 1,
        },
    };
    let observed = reader
        .inspect_result(request.clone(), pse_columnar::CancellationToken::new())
        .await
        .unwrap();
    let ResultInspectionDocument::Values {
        rows,
        manifest: observed_manifest,
        ..
    } = observed
    else {
        panic!("original stored scalar view")
    };
    assert_eq!(observed_manifest, manifest);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].values,
        vec![
            ResultInspectionCell::Text {
                value: "chosen_precision".into()
            },
            ResultInspectionCell::Text {
                value: expected_formula
            }
        ]
    );
    // The stored source revision is an original-context scalar, independent of the
    // evolved science and any currently selected model.
    let mut revision_request = request.clone();
    revision_request.view = ResultInspectionView::Values {
        fields: vec![ArrowFieldPath::from("source_revision")],
        maximum_rows: 1,
    };
    let revision = reader
        .inspect_result(revision_request, pse_columnar::CancellationToken::new())
        .await
        .unwrap();
    let ResultInspectionDocument::Values { rows, .. } = revision else {
        panic!("original source revision")
    };
    assert_eq!(
        rows[0].values[0],
        ResultInspectionCell::Binary {
            hex: original_revision.to_hex()
        }
    );

    let mut incomplete = request.clone();
    incomplete.view = ResultInspectionView::Values {
        fields: vec![ArrowFieldPath::from("native_views")],
        maximum_rows: 1,
    };
    let partial = reader
        .inspect_result(incomplete, pse_columnar::CancellationToken::new())
        .await
        .unwrap();
    assert!(matches!(
        partial,
        ResultInspectionDocument::Values {
            readable_complete: false,
            ..
        }
    ));
    let mut mismatch = request;
    mismatch.expected_manifest = Some("another-manifest".into());
    let refused = reader
        .inspect_result(mismatch, pse_columnar::CancellationToken::new())
        .await
        .unwrap();
    assert!(matches!(
        refused,
        ResultInspectionDocument::Refused {
            reason: ResultInspectionRefusal::ManifestChanged,
            ..
        }
    ));
    assert_eq!(
        store.canonical_attempt(&attempt).await.unwrap().unwrap(),
        original_attempt
    );
}

#[tokio::test]
async fn mathematical_diagnostic_authored_exact_and_enclosure_producers_deliver_native_products() {
    let runtime = crate::workflow::durable_tests::durable_runtime();
    let (package, analysis) = crate::workflow::durable_tests::package_on(&runtime,
        "package p {def Root {var x:Scalar;var y:Scalar;var p:Scalar;var q:Scalar;eq f:x+y==1;eq g:x+p*y==q;let h:Scalar=x*x+2*x;}}")
        .await;
    let services = runtime.preparation_services();
    let cancel = crate::CancelSource::new();
    let bindings = Bindings {
        demand: vec!["x".into(), "y".into(), "p".into(), "q".into(), "h".into()],
        ..Default::default()
    };
    let model = package
        .prepare(
            &services,
            analysis.root,
            analysis.instance,
            bindings.clone(),
            Limits::default(),
            &cancel,
        )
        .await
        .unwrap();
    let paths = &model.compiled().model.paths;
    let x = paths["x"];
    let y = paths["y"];
    let p = paths["p"];
    let q = paths["q"];
    let row = |name: &str| {
        model
            .compiled()
            .model
            .equations
            .iter()
            .find(|row| row.lineage.path.ends_with(&format!(".{name}")))
            .map(|row| row.id)
            .unwrap_or_else(|| {
                pse_compiler::workspace::ModelingOutput::Member(paths[name]).row_id()
            })
    };
    let point = |coordinate, value| DiagnosticInput {
        coordinate,
        lower: ExactValue::integer(value),
        upper: None,
    };
    let mut operation = MathematicalDiagnosticOperation {
        root: analysis.root,
        instance: analysis.instance,
        bindings,
        case_members: BTreeMap::new(),
        request: DiagnosticRequest {
            rows: vec![row("f"), row("g")],
            unknowns: vec![x, y],
            parameters: vec![p, q],
            inputs: vec![point(p, 2), point(q, 3)],
            derivatives: vec![],
            claim: DiagnosticClaim::ExactIdentity,
            precision: 160,
            limits: DiagnosticLimits::default(),
        },
    };
    let exact = package
        .diagnose_selected_mathematics(
            &services,
            &operation,
            &PreparationSettings::default(),
            &cancel,
        )
        .await
        .unwrap();
    let block = exact.exact.as_ref().unwrap();
    assert_eq!(block.rank, 2);
    assert_eq!(block.recovered, vec![(x, "-1".into()), (y, "2".into())]);
    assert_eq!(block.original_domain_valid, Some(true));
    assert!(block.reduced_evaluation);
    assert_eq!(exact.mathematical_views.len(), 2);

    operation.request.rows = vec![row("h")];
    operation.request.unknowns = vec![x];
    operation.request.parameters.clear();
    operation.request.claim = DiagnosticClaim::ValidatedEnclosure;
    operation.request.inputs = vec![DiagnosticInput {
        coordinate: x,
        lower: ExactValue::integer(-2),
        upper: Some(ExactValue::integer(3)),
    }];
    operation.request.derivatives = vec![vec![x], vec![x, x]];
    let ball = package
        .diagnose_selected_mathematics(
            &services,
            &operation,
            &PreparationSettings::default(),
            &cancel,
        )
        .await
        .unwrap();
    assert!(ball.unsupported.is_none());
    assert_eq!(ball.numerical.len(), 3);
    assert!(ball.numerical.iter().all(|value| value.value.is_none()
        && value.lower.is_some()
        && value.upper.is_some()
        && !value.unbounded));
    assert_eq!(ball.numerical[1].lower.as_deref(), Some("-2"));
    assert_eq!(ball.numerical[1].upper.as_deref(), Some("8"));
    assert_eq!(ball.numerical[2].lower.as_deref(), Some("2"));
    assert_eq!(ball.numerical[2].upper.as_deref(), Some("2"));
}
