// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public native diagnostics, physical samples, authored checks and knowledge projection.
//! Requires solver-highs, solver-ipopt, solver-pounce and correctness force-validation.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "public scientific fixtures retain independent expectations and original failures"
)]

#[path = "reference_support/mod.rs"]
mod reference_support;
#[path = "native_diagnostics_support/mod.rs"]
mod support;

use datafusion::arrow::record_batch::{RecordBatch, RecordBatchIterator};
use futures_util::FutureExt;
use pse_backend_native::{
    presolve::PolicyKind,
    solve::{Backend, SolveIntent},
};
use pse_diagnostics::{DiagnosticRule, DiagnosticStage};
use pse_ids::SemanticId;
use pse_model::{
    diagnostic::{BoundaryClass, BoundaryDiagnostic},
    generated::{
        enums::{
            ModelingCheckKind, ModelingConformanceKind, ModelingDiagnosticSampleStop,
            ModelingElasticObservation, ModelingRealValueKind, ModelingUncertaintyKind,
            ModelingValidityLayer, NativeTermination, PhysicalInputKind,
        },
        extension_values::QuantityValue,
        identities::DeclarationId,
        structures::ScientificCoordinateValue,
    },
};
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::runtime::{
        diagnostic_samples, modeling_checks, modeling_conformance, modeling_diagnostic_samples,
        modeling_diagnostics, modeling_findings, modeling_initializations,
        modeling_jacobian_optimization, modeling_knowledge, modeling_linear_diagnostics,
        modeling_nonlinear_explanations, modeling_reports,
    },
};
use pse_runtime::{
    CancelSource,
    workflow::{
        ConformanceControls, DiagnosticOrigin, DiagnosticSamples, DiagnosticSamplesControls,
        InitializationOverrides, JacobianDiagnosticControls, LinearDiagnosticControls,
        ModelingDiagnosticPolicy, ModelingNonlinearPolicy, ModelingPackage,
        ModelingSourceMapDocument, PreparationSettings,
    },
};
use reference_support::{NativeContext, Result, root};
use std::{
    collections::{BTreeMap, BTreeSet},
    panic::AssertUnwindSafe,
    time::Duration,
};

fn id(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}
fn settings(backend: Option<Backend>, intent: SolveIntent) -> SolveSettings {
    SolveSettings {
        backend,
        intent,
        presolve: PolicyKind::Off,
        ..Default::default()
    }
}
use pse_runtime::math::settings::SolveSettings;

fn policy() -> Result<ModelingDiagnosticPolicy> {
    let mut document: serde_json::Value = serde_json::from_slice(&std::fs::read(
        root().join("packages/reference/diagnostics/idaes-2.13.json"),
    )?)?;
    document.as_object_mut().unwrap().remove("document_version");
    Ok(serde_json::from_value(document)?)
}
async fn declaration(
    context: &NativeContext,
    package: &ModelingPackage,
    name: &str,
) -> Result<DeclarationId> {
    Ok(package
        .declarations(&context.runtime.preparation_services())
        .await?
        .iter()
        .find(|row| row.name == name)
        .unwrap()
        .declaration_id)
}
fn origin() -> DiagnosticOrigin {
    DiagnosticOrigin {
        run_id: id(240).into(),
        attempt_id: None,
        step: Some(1),
        native_attempt: None,
        operation: None,
        failure: Some(BoundaryDiagnostic::new(
            BoundaryClass::Numerical,
            DiagnosticStage::Native,
            [id(245)],
            DiagnosticRule::NativeNumerical,
        )),
    }
}
fn finish<T>(outcome: std::thread::Result<Result<T>>, closed: Result<()>) -> Result<T> {
    match outcome {
        Ok(result) => {
            closed?;
            result
        }
        Err(panic) => {
            let _ = closed;
            std::panic::resume_unwind(panic)
        }
    }
}
fn samples(
    context: &NativeContext,
    rows: Vec<diagnostic_samples::Row>,
) -> Result<DiagnosticSamples> {
    let mut builder = diagnostic_samples::Builder::new(&context.runtime.validation_context())?;
    for row in rows {
        builder.push(row)?;
    }
    let batch = builder.finish()?.batch().clone();
    Ok(context
        .runtime
        .snapshot_diagnostic_samples(RecordBatchIterator::new(
            [Ok(batch.clone())],
            batch.schema(),
        ))?)
}
fn physical_value(
    coordinate: SemanticId,
    value: f64,
    quantity: SemanticId,
    unit: SemanticId,
) -> ScientificCoordinateValue {
    ScientificCoordinateValue {
        coordinate_id: coordinate,
        value: QuantityValue {
            value,
            quantity_type_id: quantity,
            unit_id: unit,
        },
    }
}

#[tokio::test]
async fn singular_diagnostics_preserve_missing_values_source_map_and_owned_modes() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let source = "package diagnostic { def D { var x:Scalar; var y:Scalar;
            annotation nominal x(2); annotation nominal y(4);
            eq a:x+y==0; eq b:2*x+2*y==0; }
            test sample fixture { dof 0; value root.x=1; value root.y=-1; }
                { child root:D=D(); expect root.x==1 tolerance 1e-6; }
            case unstarted { child root:D=D(); } }";
        let package = context.package(source).await?;
        let services = context.runtime.execution_services();
        let sample = declaration(&context, &package, "sample").await?;
        let unstarted = declaration(&context, &package, "unstarted").await?;
        let cancel = CancelSource::new();
        let prep = PreparationSettings::default();
        let report = Box::pin(package.diagnose_case_request(
            &services,
            sample,
            &prep,
            settings(None, SolveIntent::Root),
            policy()?,
            None,
            &cancel,
        ))
        .await?;
        assert!(report.complete && report.origin.is_none());
        assert_eq!(report.statistics["variables"], 2);
        assert_eq!((report.rows.len(), report.columns.len()), (2, 2));
        assert!(!report.findings.is_empty());
        let missing = Box::pin(package.diagnose_case_request(
            &services,
            unstarted,
            &prep,
            settings(None, SolveIntent::Root),
            policy()?,
            Some(origin()),
            &cancel,
        ))
        .await?;
        assert!(!missing.complete);
        assert_eq!(missing.statistics["missing_values"], 2);
        let missing_table =
            modeling_diagnostics::Row::rows(&missing.table("runtime.modeling_diagnostics")?)?;
        assert!(missing_table[0].point.is_empty());
        let retained_origin: DiagnosticOrigin =
            serde_json::from_str(missing_table[0].origin_json.as_ref().unwrap())?;
        assert_eq!(
            serde_json::to_value(retained_origin)?,
            serde_json::to_value(origin())?
        );
        let quantities = package.physical_context().quantities();
        let scalar = quantities.quantity_type(quantities.neutral_dimensionless().unwrap())?;
        let supplied = samples(
            &context,
            vec![diagnostic_samples::Row {
                sample_id: id(233),
                values: missing
                    .columns
                    .iter()
                    .zip([1., -1.])
                    .map(|(coordinate, value)| {
                        physical_value(
                            *coordinate,
                            value,
                            scalar.id.as_id(),
                            scalar.canonical_unit.as_id(),
                        )
                    })
                    .collect(),
            }],
        )?;
        let supplied = Box::pin(package.diagnose_samples_request(
            &services,
            unstarted,
            &prep,
            settings(None, SolveIntent::Root),
            supplied,
            policy()?,
            Default::default(),
            Some(origin()),
            &cancel,
        ))
        .await?;
        let supplied_report = supplied.outcomes[0].1.as_ref().unwrap();
        assert!(supplied_report.complete);
        assert_eq!(
            serde_json::to_value(&supplied_report.origin)?,
            serde_json::to_value(Some(origin()))?
        );
        let exported = modeling_diagnostics::Row::rows(
            &supplied_report.table("runtime.modeling_diagnostics")?,
        )?;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(exported[0].origin_json.as_ref().unwrap())?,
            serde_json::to_value(origin())?
        );
        assert!(
            Box::pin(package.execute_case_request(
                &services,
                unstarted,
                &prep,
                settings(None, SolveIntent::Root),
                &cancel
            ))
            .await
            .is_err(),
            "diagnostic preparation may inspect missing starts without granting solve admission"
        );

        for (bytes, time) in [(0, 30.), (4 << 20, 0.), (4 << 20, f64::NAN)] {
            assert!(
                Box::pin(package.source_map_request(
                    services.preparation(),
                    sample,
                    &prep,
                    settings(None, SolveIntent::Root),
                    bytes,
                    time,
                    &cancel
                ))
                .await
                .is_err()
            );
        }
        let map = Box::pin(package.source_map_request(
            services.preparation(),
            sample,
            &prep,
            settings(None, SolveIntent::Root),
            4 << 20,
            30.,
            &cancel,
        ))
        .await?;
        let source_map = map.document();
        source_map.validate()?;
        assert_eq!(
            serde_json::to_value(&source_map.scope)?,
            "static_eligible_alternatives"
        );
        assert_eq!(source_map.rows, report.rows);
        assert_eq!(source_map.columns, report.columns);
        assert!(
            source_map
                .native_pullback_unavailable
                .contains("no transformed native attempt")
        );
        let equations = [
            declaration(&context, &package, "a").await?.as_id(),
            declaration(&context, &package, "b").await?.as_id(),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        assert_eq!(
            source_map
                .members
                .iter()
                .filter(|member| report.rows.contains(&member.member))
                .map(|member| member.declaration)
                .collect::<BTreeSet<_>>(),
            equations
        );
        let equation_fields = source_map
            .fields
            .iter()
            .filter(|field| equations.contains(&field.declaration))
            .collect::<Vec<_>>();
        assert_eq!(
            equation_fields
                .iter()
                .map(|field| field.declaration)
                .collect::<BTreeSet<_>>(),
            equations,
            "both original equations must retain source fields before checking their spans"
        );
        for field in equation_fields {
            assert!(field.document.is_some() && !field.role.is_empty());
            let (start, end) = (field.start.unwrap(), field.end.unwrap());
            assert!(start < end && end as usize <= source.len());
        }
        assert!(!source_map.templates.is_empty() && !source_map.instances.is_empty());
        for instance in &source_map.instances {
            assert!(!instance.demands.is_empty());
            assert!(instance.checked_members.iter().all(|(_, original)| {
                source_map
                    .members
                    .iter()
                    .any(|member| member.member == *original)
            }));
            for demand in &instance.demands {
                let template = source_map
                    .templates
                    .iter()
                    .find(|template| template.template == demand.template)
                    .unwrap();
                assert_eq!(template.body, instance.body);
                assert_eq!(template.outputs, demand.outputs);
                assert_eq!(template.coordinates, demand.coordinates);
                assert_eq!(serde_json::to_value(template.product)?, "selected_partials");
                assert!(
                    template
                        .arithmetic
                        .iter()
                        .all(|origin| origin.start_output < origin.end_output
                            && (origin.source.is_none() == origin.source_unavailable.is_some()))
                );
            }
        }
        let table = report.table("runtime.modeling_diagnostics")?;
        let findings = report.table("runtime.modeling_findings")?;
        let document = serde_json::to_vec(source_map)?;
        // Exercise the public request and its generated export, in addition to
        // the lower-level optimization science in native_scientific_modes.
        let native = Box::pin(package.diagnose_jacobian_request(
            &services,
            sample,
            &prep,
            settings(None, SolveIntent::Root),
            JacobianDiagnosticControls {
                maximum_attempts: 4,
                ..Default::default()
            },
            Some(origin()),
            &cancel,
        ))
        .await?;
        assert_eq!(native.relation, "runtime.modeling_jacobian_optimization");
        assert_eq!(native.attempts.len(), 4);
        assert_eq!(
            serde_json::to_value(&native.origin)?,
            serde_json::to_value(Some(origin()))?
        );
        let limited = Box::pin(package.diagnose_jacobian_request(
            &services,
            sample,
            &prep,
            settings(None, SolveIntent::Root),
            JacobianDiagnosticControls {
                maximum_attempts: 1,
                ..Default::default()
            },
            None,
            &cancel,
        ))
        .await?;
        let mut invalid = policy()?;
        invalid.terms.maximum_terms = 1;
        assert!(invalid.validate().is_err());
        drop((report, missing, map, package));
        Ok((
            table,
            findings,
            document,
            native.table.clone(),
            limited.table.clone(),
        ))
    })
    .catch_unwind()
    .await;
    let (table, findings, document, native, limited) = finish(outcome, context.close().await)?;
    let rows = modeling_diagnostics::Row::rows(&table)?;
    let row = &rows[0];
    let matrix = row.matrix.as_ref().unwrap();
    assert_eq!(matrix.rank, 1);
    assert!(matrix.cutoff > 0. && matrix.modes.len() == 2);
    assert_eq!(
        row.row_nominals
            .iter()
            .map(|item| item.source_id)
            .collect::<Vec<_>>(),
        row.rows
    );
    assert_eq!(
        row.variable_nominals
            .iter()
            .map(|item| item.source_id)
            .collect::<Vec<_>>(),
        row.columns
    );
    let mut nominals = row
        .variable_nominals
        .iter()
        .map(|item| item.value)
        .collect::<Vec<_>>();
    nominals.sort_by(f64::total_cmp);
    assert_eq!(nominals, [2., 4.]);
    for mode in &matrix.modes {
        assert!(mode.left_residual.unwrap() <= mode.residual_bound);
        assert!(mode.right_residual.unwrap() <= mode.residual_bound);
        assert!(mode.normalization_error >= 0. && mode.residual_bound > 0.);
        for norm in [
            mode.left
                .iter()
                .map(|value| value.value * value.value)
                .sum::<f64>(),
            mode.right
                .iter()
                .map(|value| value.value * value.value)
                .sum::<f64>(),
        ] {
            assert!((norm - 1.).abs() <= 1e-10);
        }
    }
    assert!(!modeling_findings::Row::rows(&findings)?.is_empty());
    let map: ModelingSourceMapDocument = serde_json::from_slice(&document)?;
    map.validate()?;
    assert_eq!(map.rows, row.rows);
    let native_rows = modeling_jacobian_optimization::Row::rows(&native)?;
    let [native] = native_rows.as_slice() else {
        panic!("one exported original Jacobian analysis")
    };
    assert!(native.complete && native.unavailable.is_empty());
    assert_eq!(native.attempts.len(), 4);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(native.origin_json.as_ref().unwrap())?,
        serde_json::to_value(origin())?
    );
    assert_eq!(native.source_identity, row.source_identity);
    assert_eq!(native.numerical_identity, row.numerical_identity);
    assert_eq!(
        native
            .row_nominals
            .iter()
            .map(|item| item.source_id)
            .collect::<Vec<_>>(),
        row.rows
    );
    assert_eq!(
        native
            .variable_nominals
            .iter()
            .map(|item| item.source_id)
            .collect::<Vec<_>>(),
        row.columns
    );
    assert_eq!(
        native
            .row_nominals
            .iter()
            .map(|item| (item.source_id, item.value))
            .collect::<Vec<_>>(),
        row.row_nominals
            .iter()
            .map(|item| (item.source_id, item.value))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        native
            .variable_nominals
            .iter()
            .map(|item| (item.source_id, item.value))
            .collect::<Vec<_>>(),
        row.variable_nominals
            .iter()
            .map(|item| (item.source_id, item.value))
            .collect::<Vec<_>>()
    );
    assert!(!native.conditioning.is_empty());
    let original_rows = row.rows.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        native.conditioning[0]
            .weights
            .iter()
            .map(|weight| weight.source_id)
            .collect::<BTreeSet<_>>(),
        original_rows
    );
    assert!(native.conditioning.iter().all(|certificate| {
        original_rows.contains(&certificate.pivot)
            && certificate
                .weights
                .iter()
                .all(|weight| original_rows.contains(&weight.source_id))
    }));
    let [degenerate] = native.degenerate.as_slice() else {
        panic!("one original two-row degenerate set")
    };
    assert!(degenerate.irreducible_at_tolerance);
    assert_eq!(
        degenerate.rows.iter().copied().collect::<BTreeSet<_>>(),
        original_rows
    );
    assert_eq!(
        degenerate
            .certificate
            .weights
            .iter()
            .map(|weight| weight.source_id)
            .collect::<BTreeSet<_>>(),
        original_rows
    );
    let limited = modeling_jacobian_optimization::Row::rows(&limited)?;
    assert_eq!(limited.len(), 1);
    assert!(!limited[0].complete);
    assert_eq!(limited[0].attempts.len(), 1);
    assert!(limited[0].degenerate.is_empty());
    assert!(
        limited[0]
            .unavailable
            .iter()
            .any(|reason| reason.contains("attempt budget"))
    );
    Ok(())
}

#[tokio::test]
async fn diagnostic_samples_keep_converted_sparse_points_origin_and_bounded_partial_population()
-> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = support::length_package(
            &context,
            "package custody { def Root {
            var x:Length; var y:Length; eq balance:x+y==0{m}; eq contrast:x-y==6{m};
            annotation start x(1{m}); annotation start y(-1{m}); }}",
        )
        .await?;
        let services = context.runtime.execution_services();
        let case = declaration(&context, &package, "Root").await?;
        let cancel = CancelSource::new();
        let execution = Box::pin(package.admit_declared_request(
            services.preparation(),
            case,
            &PreparationSettings::default(),
            settings(None, SolveIntent::Root),
            &cancel,
        ))
        .await?;
        let prepared = Box::pin(package.prepare_diagnostics(
            services.preparation(),
            &execution.analysis,
            &cancel,
        ))
        .await?;
        let model = &prepared.model.model.compiled().model;
        let mut roots = model
            .instances
            .values()
            .filter(|instance| instance.parent.is_none());
        let root = roots.next().expect("one actual source root instance");
        assert!(roots.next().is_none());
        let coordinate = |name: &str| {
            let lineage = format!("{}.{name}", root.path);
            let mut symbols = model
                .symbols
                .values()
                .filter(|symbol| symbol.lineage.path == lineage);
            let id = symbols
                .next()
                .expect("authored variable in original root lineage")
                .id;
            assert!(
                symbols.next().is_none(),
                "authored coordinate resolves uniquely"
            );
            assert!(
                prepared.model.case.compiled().plan.columns().contains(&id),
                "physical samples target actual diagnostic columns"
            );
            id
        };
        let (x, y) = (coordinate("x"), coordinate("y"));
        let quantities = package.physical_context().quantities();
        let length = quantities
            .quantity_types()
            .find(|quantity| quantity.name.as_deref() == Some("Length"))
            .unwrap();
        let points = vec![
            diagnostic_samples::Row {
                sample_id: id(241),
                values: vec![
                    physical_value(y, -300., length.id.as_id(), support::CENTIMETRE),
                    physical_value(x, 300., length.id.as_id(), support::CENTIMETRE),
                ],
            },
            diagnostic_samples::Row {
                sample_id: id(242),
                values: vec![],
            },
        ];
        let input = samples(&context, points)?;
        // The native snapshot owns decoded physical values after the producer's
        // Arrow batches have been dropped. Rust Arrow buffers are immutable;
        // mutating Python's borrowed producer is a separate frontend concern.
        let report = Box::pin(package.diagnose_samples_request(
            &services,
            case,
            &PreparationSettings::default(),
            settings(None, SolveIntent::Root),
            input.clone(),
            policy()?,
            DiagnosticSamplesControls::default(),
            Some(origin()),
            &cancel,
        ))
        .await?;
        assert_eq!(report.stop, ModelingDiagnosticSampleStop::Completed);
        assert_eq!(report.unattempted, 0);
        assert_eq!(
            report
                .outcomes
                .iter()
                .map(|outcome| outcome.0)
                .collect::<Vec<_>>(),
            [id(241), id(242)]
        );
        for ((_, outcome), (want_x, want_y)) in report.outcomes.iter().zip([(3., -3.), (1., -1.)]) {
            let outcome = outcome.as_ref().unwrap();
            assert!(outcome.complete);
            assert_eq!(
                serde_json::to_value(&outcome.origin)?,
                serde_json::to_value(Some(origin()))?
            );
            let table =
                modeling_diagnostics::Row::rows(&outcome.table("runtime.modeling_diagnostics")?)?;
            let point = table[0]
                .point
                .iter()
                .map(|cell| (cell.source_id, cell.value))
                .collect::<BTreeMap<_, _>>();
            assert_eq!((point[&x], point[&y]), (Some(want_x), Some(want_y)));
        }
        let limited = Box::pin(package.diagnose_samples_request(
            &services,
            case,
            &PreparationSettings::default(),
            settings(None, SolveIntent::Root),
            input,
            policy()?,
            DiagnosticSamplesControls {
                maximum_samples: 1,
                ..Default::default()
            },
            None,
            &cancel,
        ))
        .await?;
        assert_eq!(limited.stop, ModelingDiagnosticSampleStop::SampleLimit);
        assert_eq!(limited.unattempted, 1);
        let wrong = samples(
            &context,
            vec![diagnostic_samples::Row {
                sample_id: id(243),
                values: vec![physical_value(
                    id(249),
                    1.,
                    length.id.as_id(),
                    length.canonical_unit.as_id(),
                )],
            }],
        )?;
        let error = Box::pin(package.diagnose_samples_request(
            &services,
            case,
            &PreparationSettings::default(),
            settings(None, SolveIntent::Root),
            wrong,
            policy()?,
            Default::default(),
            None,
            &cancel,
        ))
        .await
        .unwrap_err();
        assert!(
            error.to_string().contains("unknown physical coordinate"),
            "{error}"
        );
        let scalar = quantities.quantity_type(quantities.neutral_dimensionless().unwrap())?;
        let wrong = samples(
            &context,
            vec![diagnostic_samples::Row {
                sample_id: id(243),
                values: vec![physical_value(
                    x,
                    1.,
                    scalar.id.as_id(),
                    scalar.canonical_unit.as_id(),
                )],
            }],
        )?;
        let error = Box::pin(package.diagnose_samples_request(
            &services,
            case,
            &PreparationSettings::default(),
            settings(None, SolveIntent::Root),
            wrong,
            policy()?,
            Default::default(),
            None,
            &cancel,
        ))
        .await
        .unwrap_err();
        assert!(error.to_string().contains("quantity differs"), "{error}");
        // Exercise the receiving field admission, bypassing the checked Builder's
        // earlier nonfinite refusal exactly as an external Arrow producer can.
        let schema = context
            .runtime
            .physical_input_schema(PhysicalInputKind::DiagnosticSamples)?;
        let mut columns = schema
            .fields()
            .iter()
            .map(|field| datafusion::arrow::array::builder::make_builder(field.data_type(), 1))
            .collect::<Vec<_>>();
        diagnostic_samples::Row {
            sample_id: id(244),
            values: vec![physical_value(
                x,
                f64::NAN,
                length.id.as_id(),
                length.canonical_unit.as_id(),
            )],
        }
        .append_columns(&mut columns)?;
        let nonfinite = RecordBatch::try_new(
            schema,
            columns.iter_mut().map(|column| column.finish()).collect(),
        )?;
        assert!(
            context
                .runtime
                .snapshot_diagnostic_samples(RecordBatchIterator::new(
                    [Ok(nonfinite.clone())],
                    nonfinite.schema()
                ))
                .is_err()
        );
        let table = report.table()?;
        let limited = limited.table()?;
        let findings = report.findings_table()?;
        drop((report, package));
        Ok((table, limited, findings))
    })
    .catch_unwind()
    .await;
    let (table, limited, findings) = finish(outcome, context.close().await)?;
    assert!(
        modeling_diagnostic_samples::Row::rows(&table)?[0]
            .outcomes
            .iter()
            .all(|row| row.failure_ordinal.is_none())
    );
    assert_eq!(
        modeling_diagnostic_samples::Row::rows(&limited)?[0].unattempted,
        1
    );
    assert!(modeling_findings::Row::rows(&findings)?.is_empty());
    Ok(())
}

#[tokio::test]
async fn affine_native_diagnostics_preserve_iis_relaxation_ranging_and_refusals() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = context
            .package(
                "package linear {
            def Bad {var x:Scalar; eq lo:x>=2; eq hi:x<=1;}
            def Good {var x:Scalar; eq low:x>=0; eq high:x<=2;}
            def Curved {var x:Scalar; eq curve:x*x==1;}
            case bad {child root:Bad=Bad();} case good {child root:Good=Good();}
            case curved {child root:Curved=Curved();}}",
            )
            .await?;
        let services = context.runtime.execution_services();
        let (bad, good, curved) = (
            declaration(&context, &package, "bad").await?,
            declaration(&context, &package, "good").await?,
            declaration(&context, &package, "curved").await?,
        );
        let cancel = CancelSource::new();
        let solver = settings(Some(Backend::Highs), SolveIntent::FeasiblePoint);
        let bad = Box::pin(package.diagnose_linear_request(
            &services,
            bad,
            &PreparationSettings::default(),
            solver.clone(),
            LinearDiagnosticControls {
                iis: true,
                rays: true,
                relaxation: Some((-1., -1., 1.)),
                ..Default::default()
            },
            Some(origin()),
            &cancel,
        ))
        .await?;
        assert_eq!(bad.relation, "runtime.modeling_linear_diagnostics");
        let table = bad.table.clone();
        let rows = modeling_linear_diagnostics::Row::rows(&table)?;
        let row = &rows[0];
        assert_eq!(
            row.attempt.termination.category,
            NativeTermination::Infeasible
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(row.origin_json.as_ref().unwrap())?,
            serde_json::to_value(origin())?
        );
        let iis = row.iis.as_ref().unwrap();
        assert!(!iis.relaxation_only);
        assert_eq!(
            iis.rows
                .iter()
                .map(|item| item.source_id)
                .collect::<BTreeSet<_>>(),
            row.rows.iter().copied().collect()
        );
        let relaxation = row.relaxation.as_ref().unwrap();
        assert_eq!(relaxation.operation_status, 0);
        assert!(
            (relaxation.penalty.unwrap() - 1.).abs()
                <= solver.numerics.engineering_relative_fraction
        );
        assert_eq!(
            relaxation.restored_status.category,
            NativeTermination::Inconclusive
        );
        assert_eq!(
            relaxation.primal.as_ref().unwrap()[0].source_id,
            row.columns[0]
        );
        let bad_case = declaration(&context, &package, "bad").await?;
        let error = Box::pin(package.diagnose_linear_request(
            &services,
            bad_case,
            &PreparationSettings::default(),
            solver.clone(),
            LinearDiagnosticControls {
                relaxation: Some((-1., -1., 1.)),
                row_penalties: Some(BTreeMap::new()),
                ..Default::default()
            },
            None,
            &cancel,
        ))
        .await
        .unwrap_err();
        assert!(
            error.to_string().contains("every source coordinate"),
            "{error}"
        );
        let error = Box::pin(package.diagnose_linear_request(
            &services,
            curved,
            &PreparationSettings::default(),
            solver.clone(),
            LinearDiagnosticControls {
                iis: true,
                ..Default::default()
            },
            None,
            &cancel,
        ))
        .await
        .unwrap_err();
        assert!(error.to_string().contains("affine coefficient"), "{error}");
        let ranged = Box::pin(package.diagnose_linear_request(
            &services,
            good,
            &PreparationSettings::default(),
            solver,
            LinearDiagnosticControls {
                ranging: true,
                ..Default::default()
            },
            None,
            &cancel,
        ))
        .await?;
        Ok((table, ranged.table.clone()))
    })
    .catch_unwind()
    .await;
    let (bad, ranged) = finish(outcome, context.close().await)?;
    assert_eq!(modeling_linear_diagnostics::Row::rows(&bad)?.len(), 1);
    let rows = modeling_linear_diagnostics::Row::rows(&ranged)?;
    let row = &rows[0];
    assert!(!row.ranging.is_empty(), "{:?}", row.unavailable);
    let coordinates = row
        .columns
        .iter()
        .chain(&row.rows)
        .copied()
        .collect::<BTreeSet<_>>();
    for range in &row.ranging {
        assert!(coordinates.contains(&range.source_id));
        assert_eq!(
            range.value.kind == ModelingRealValueKind::Finite,
            range.value.value.is_some()
        );
        assert_eq!(
            range.objective.kind == ModelingRealValueKind::Finite,
            range.objective.value.is_some()
        );
        assert_ne!(
            range
                .entering
                .source_id
                .is_some_and(|id| coordinates.contains(&id)),
            range.entering.sentinel.is_some()
        );
        assert_ne!(
            range
                .leaving
                .source_id
                .is_some_and(|id| coordinates.contains(&id)),
            range.leaving.sentinel.is_some()
        );
    }
    assert!(row.ranging.iter().any(|range| matches!(
        range.value.kind,
        ModelingRealValueKind::PositiveInfinity | ModelingRealValueKind::NegativeInfinity
    )));
    Ok(())
}

#[tokio::test]
async fn authored_fixed_fixture_shares_initialization_conformance_reports_and_closure_checks()
-> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = context
            .package(
                "package fixed { def D {var x:Scalar; eq e:x==2;
            annotation report x(\"value\"); annotation valid x(0,3); }
            test sample fixture {dof -1; route simultaneous; procedure initialize;
                policy {presolve off; derivatives step(1e-7) tolerance(1e-4) cells(100);}
                fix root.x=2; } {child root:D=D(); expect root.x==2 tolerance 1e-6;} }",
            )
            .await?;
        let services = context.runtime.execution_services();
        let sample = declaration(&context, &package, "sample").await?;
        let cancel = CancelSource::new();
        let result = Box::pin(package.execute_case_request(
            &services,
            sample,
            &PreparationSettings::default(),
            settings(None, SolveIntent::Root),
            &cancel,
        ))
        .await?;
        assert!(result.accepted);
        assert!(matches!(
            &result.outcome,
            pse_runtime::math::solves::Outcome::Constant(_)
        ));
        let conformance = Box::pin(package.conform_request(
            &services,
            &PreparationSettings::default(),
            settings(None, SolveIntent::Root),
            ConformanceControls {
                maximum_checks: 64,
                ..Default::default()
            },
            &cancel,
        ))
        .await?;
        assert!(conformance.complete && conformance.passed());
        assert_eq!(conformance.fixtures(), BTreeSet::from([sample]));
        let extracted = &conformance.results[&sample];
        let reports = extracted.table("runtime.modeling_reports")?;
        let checks = result.table("runtime.modeling_checks")?;
        let initialized = Box::pin(package.initialize_case_request(
            &services,
            sample,
            &PreparationSettings::default(),
            settings(None, SolveIntent::Root),
            InitializationOverrides {
                maximum_attempts: Some(2),
                ..Default::default()
            },
            &cancel,
        ))
        .await?;
        assert!(
            initialized.completed
                && initialized.failure.is_none()
                && initialized.committed.is_some()
        );
        assert_eq!(initialized.attempts.len(), 1);
        assert!(initialized.attempts[0].accepted());
        Ok((reports, checks, initialized.table()?, conformance.table()?))
    })
    .catch_unwind()
    .await;
    let (reports, checks, initialized, conformance) = finish(outcome, context.close().await)?;
    assert_eq!(
        modeling_reports::Row::rows(&reports)?
            .iter()
            .map(|row| row.value)
            .collect::<Vec<_>>(),
        [2.]
    );
    let checks = modeling_checks::Row::rows(&checks)?;
    assert!(checks.iter().any(|row| row.within_validity == Some(true)
        && row.extrapolation_allowed.is_none()
        && row.satisfied
        && row.layer == Some(ModelingValidityLayer::Closure)));
    assert!(
        checks
            .iter()
            .all(|row| row.layer.is_none() == (row.kind != ModelingCheckKind::Validity))
    );
    assert!(modeling_initializations::Row::rows(&initialized)?[0].complete);
    assert!(
        modeling_conformance::Row::rows(&conformance)?
            .iter()
            .any(|row| row.kind == ModelingConformanceKind::Coverage)
    );
    Ok(())
}

#[tokio::test]
async fn nonlinear_explanation_retains_local_obstruction_original_candidates_and_attempt_limit()
-> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = context
            .package(
                "package nonlinear {def D {var x:Scalar;
            eq lo:x*x>=4; eq hi:x*x<=1; eq spare:x*x<=100; annotation start x(1.5);}
            case run {child root:D=D();}}",
            )
            .await?;
        let services = context.runtime.execution_services();
        let case = declaration(&context, &package, "run").await?;
        let cancel = CancelSource::new();
        let solver = settings(Some(Backend::Ipopt), SolveIntent::Optimize);
        let diagnostic = Box::pin(package.diagnose_case_request(
            &services,
            case,
            &PreparationSettings::default(),
            solver.clone(),
            policy()?,
            None,
            &cancel,
        ))
        .await?;
        let policy = ModelingNonlinearPolicy {
            nominals: diagnostic.rows.iter().map(|row| (*row, 1.)).collect(),
            penalty_tolerance: 1e-6,
            maximum_attempts: 4,
            time_limit: Duration::from_secs(30),
        };
        let report = Box::pin(package.explain_case_request(
            &services,
            case,
            &PreparationSettings::default(),
            solver.clone(),
            policy.clone(),
            &cancel,
        ))
        .await?;
        assert!(report.complete && report.stop.is_none());
        assert_eq!(report.candidate_rows.len(), 2);
        assert!(report.background_variables.is_empty());
        assert_eq!(report.attempts.len(), 4);
        assert_eq!(
            report.attempts[0].observation,
            ModelingElasticObservation::LocalObstruction
        );
        assert!(
            (report.attempts[0].penalty.unwrap() - 3.).abs()
                <= 3. * solver.numerics.engineering_relative_fraction
        );
        assert_eq!(
                report
                    .attempts
                    .iter()
                    .filter(|attempt| attempt.observation
                        == ModelingElasticObservation::FeasibleWitness)
                    .count(),
                2
            );
        let original = report.attempts[0].result.as_ref().unwrap();
        assert!(!original.accepted);
        let pse_runtime::math::solves::Outcome::Native(attempt) = &original.outcome else {
            panic!("original local native attempt")
        };
        assert_eq!(attempt.backend, Backend::Pounce);
        let limited = Box::pin(package.explain_case_request(
            &services,
            case,
            &PreparationSettings::default(),
            solver,
            ModelingNonlinearPolicy {
                maximum_attempts: 1,
                ..policy
            },
            &cancel,
        ))
        .await?;
        assert!(!limited.complete && limited.stop.is_some());
        assert_eq!(
            limited.stop.as_ref().unwrap().rule,
            DiagnosticRule::ModelingNonlinearAttemptLimit
        );
        Ok((
            report.table()?,
            report.findings_table()?,
            limited.findings_table()?,
        ))
    })
    .catch_unwind()
    .await;
    let (table, findings, limited) = finish(outcome, context.close().await)?;
    let rows = modeling_nonlinear_explanations::Row::rows(&table)?;
    let row = &rows[0];
    assert!(row.complete && row.candidate_rows.len() == 2);
    assert_eq!(row.attempts[0].failure_ordinal, Some(1));
    let findings = modeling_findings::Row::rows(&findings)?;
    let failure = findings
        .iter()
        .find(|finding| finding.ordinal == 1)
        .unwrap();
    assert_eq!(failure.rule, DiagnosticRule::ModelingQualificationRejected);
    assert!(!failure.sources.is_empty() && !failure.observations.is_empty());
    assert!(
        failure
            .sources
            .iter()
            .all(|source| row.candidate_rows.contains(source))
    );
    assert!(
        failure
            .observations
            .iter()
            .any(|observation| observation.name == "candidate_use"
                && observation.text.as_deref() == Some("diagnostic_only"))
    );
    let limited = modeling_findings::Row::rows(&limited)?;
    assert_eq!(limited[0].ordinal, 0);
    assert_eq!(
        limited[0].rule,
        DiagnosticRule::ModelingNonlinearAttemptLimit
    );
    Ok(())
}

#[tokio::test]
async fn knowledge_preserves_admitted_quantity_uncertainty_provenance_and_byte_refusal()
-> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = context
            .package(
                "package knowledge {entity kind source provenance {attribute title:Text;}
            enum role {measured} entity source origin {title=\"synthetic inspection test\"}
            entity kind sample {attribute value:Scalar;}
            entity sample a provenance(origin,role.measured) {value=2.5 ± relative(0.1)}}",
            )
            .await?;
        let sample = declaration(&context, &package, "a").await?;
        let origin = declaration(&context, &package, "origin").await?;
        let services = context.runtime.preparation_services();
        let cancel = pse_columnar::CancellationToken::new();
        let knowledge =
            Box::pin(package.knowledge(&services, Some(sample), 1, 64 << 20, &cancel)).await?;
        let row = modeling_knowledge::Row::rows(knowledge.table())?
            .pop()
            .unwrap();
        assert_eq!(row.owner_id, sample);
        assert_eq!(row.source_id, Some(origin));
        assert_eq!(row.source_revision, knowledge.source_revision().as_id());
        assert_eq!(row.slot, "value");
        assert!(!row.test_only);
        let value = row.value.last().unwrap();
        assert_eq!(value.magnitude, Some(2.5));
        let quantities = package.physical_context().quantities();
        let scalar = quantities.quantity_type(quantities.neutral_dimensionless().unwrap())?;
        assert_eq!(value.quantity_type_id, Some(scalar.id.as_id()));
        assert_eq!(value.canonical_unit_id, Some(scalar.canonical_unit.as_id()));
        let uncertainty = row.uncertainty.as_ref().unwrap();
        assert_eq!(uncertainty.kind, ModelingUncertaintyKind::Relative);
        assert_eq!(uncertainty.magnitude, 0.1);
        let error = Box::pin(package.knowledge(&services, Some(sample), 1, 128, &cancel))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("maximum_bytes"), "{error}");
        Ok(knowledge.table().clone())
    })
    .catch_unwind()
    .await;
    let table: FieldCheckedBatch = finish(outcome, context.close().await)?;
    assert_eq!(modeling_knowledge::Row::rows(&table)?.len(), 1);
    Ok(())
}
