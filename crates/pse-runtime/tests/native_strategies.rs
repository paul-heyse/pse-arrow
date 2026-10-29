// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public authored initialization and causal recycle callers with analytic expectations.
//! Requires solver-kinsol and force-validate; no canonical service or Python fixtures.
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "public caller setup and independent scientific assertions fail directly"
)]

#[path = "reference_support/mod.rs"]
mod reference_support;

use datafusion::arrow::record_batch::RecordBatchIterator;
use futures_util::FutureExt;
use pse_backend_native::solve::{Backend, Qualification, SolveIntent, StartPolicy};
use pse_compiler::workspace::ModelingFlowSelection;
use pse_ids::SemanticId;
use pse_model::generated::{
    enums::CandidateUse, extension_values::QuantityValue, structures::ScientificCoordinateValue,
};
use pse_relations::{columnar::RowBuilder, generated::runtime::initialization_stages};
use pse_runtime::{
    CancelSource,
    math::{
        initialization::{DeclaredRootReport, InitializationReport},
        settings::SolveSettings,
    },
    workflow::{
        CausalUnitRealization, CausalUnitRequest, InitializationStages, ModelingPackage,
        PreparationSettings, RecycleRequest,
    },
};
use pse_structural::flowsheet::{Decision, Policy};
use reference_support::{NativeContext, Result};
use std::{collections::BTreeSet, panic::AssertUnwindSafe};

fn physical_stages(
    context: &NativeContext,
    package: &ModelingPackage,
    parameter: SemanticId,
    overlays: &[Option<f64>],
) -> Result<InitializationStages> {
    let quantities = package.physical_context().quantities();
    let quantity = quantities
        .neutral_dimensionless()
        .expect("the admitted physical package declares Scalar");
    let unit = quantities.quantity_type(quantity)?.canonical_unit;
    let validation = context.runtime.validation_context();
    let mut builder = RowBuilder::<initialization_stages::Row>::new(&validation)?;
    for (index, overlay) in overlays.iter().enumerate() {
        builder.push(initialization_stages::Row {
            stage: i64::try_from(index)?,
            values: overlay
                .iter()
                .map(|value| ScientificCoordinateValue {
                    coordinate_id: parameter,
                    value: QuantityValue {
                        value: *value,
                        quantity_type_id: quantity.as_id(),
                        unit_id: unit.as_id(),
                    },
                })
                .collect(),
        })?;
    }
    let batch = builder.finish()?.batch().clone();
    Ok(context.runtime.snapshot_initialization_stages(
        RecordBatchIterator::new([Ok(batch.clone())], batch.schema()),
    )?)
}

fn close_to(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-6,
        "{actual} differs from independent expectation {expected}"
    );
}

#[tokio::test]
async fn native_initialization_restores_original_inputs_and_retains_partial_stages() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        let package = context
            .package(
                "package strategies {def Root {
                 param a:Scalar=4; var x:Scalar; var y:Scalar;
                 eq square:x*x==a; eq downstream:y==x+1;
                 annotation start x(2); annotation start y(3); annotation nominal x(2);
                 }}",
            )
            .await?;
        let services = context.runtime.preparation_services();
        let root = package
            .declarations(&services)
            .await?
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let mut preparation = PreparationSettings::default();
        preparation.compiler.evaluation.scratch_bytes = 1 << 20;
        let mut settings = SolveSettings {
            backend: Some(Backend::Kinsol),
            intent: SolveIntent::Root,
            ..Default::default()
        };
        settings.controls.start = StartPolicy::PreviousAccepted;
        settings.controls.iterations = 50;
        let cancel = CancelSource::new();
        let execution = Box::pin(package.admit_declared_request(
            &services, root, &preparation, settings.clone(), &cancel,
        ))
        .await?;
        let model = &execution.model;
        let symbol = |name: &str| {
            model
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path.ends_with(&format!(".{name}")))
                .unwrap()
                .id
        };
        let (a, x, y) = (symbol("a"), symbol("x"), symbol("y"));
        let revision = package.revision_id().to_owned();

        // The positive branch is selected by the authored start: sqrt(9)=3,
        // followed by y=4. An empty final overlay restores a=4, x=2, y=3.
        // The second schedule instead requests x*x=-1, which has no real root.
        for (overlays, complete) in [
            ([Some(9.0), None], true),
            ([Some(9.0), Some(-1.0)], false),
        ] {
            let stages = physical_stages(&context, &package, a, &overlays)?;
            let prepared = Box::pin(package.prepare_initialization_request(
                &services,
                root,
                &preparation,
                settings.clone(),
                stages,
                &cancel,
            ))
            .await?;
            assert_eq!(prepared.blocks().boundaries().count(), 2);
            let report: InitializationReport = prepared.start(&services)?.finish().await?;
            assert!(!report.cancelled, "{report:?}");
            assert_eq!(report.completed_stages, if complete { 2 } else { 1 });
            assert_eq!(report.original_bindings_restored, complete);
            assert_eq!(report.stages.len(), 2);
            assert_eq!(report.original.scalars[&a], 4.0);
            assert_eq!(report.original.scalars[&x], 2.0);
            assert_eq!(report.original.scalars[&y], 3.0);
            assert_eq!(package.revision_id(), revision);
            assert_eq!(report.values.scalars.len(), 2);
            assert!(!report.values.scalars.contains_key(&a));
            close_to(report.stages[0].candidate.scalars[&x], 3.0);
            close_to(report.stages[0].candidate.scalars[&y], 4.0);
            assert!(report.stages[0].completed);
            assert_eq!(report.stages[1].completed, complete);
            let expected_x = if complete { 2.0 } else { 3.0 };
            close_to(report.values.scalars[&x], expected_x);
            close_to(report.values.scalars[&y], expected_x + 1.0);
            // Independently check both authored equations on the committed stage.
            let committed_a = if complete { 4.0 } else { 9.0 };
            close_to(report.values.scalars[&x].powi(2), committed_a);
            close_to(report.values.scalars[&y] - report.values.scalars[&x], 1.0);
            if complete {
                assert!(report.stages[1].overlay.is_empty());
                assert!(report.attempts.iter().all(|attempt| attempt.committed));
            } else {
                assert_eq!(report.stages[1].overlay[&a], -1.0);
                assert_eq!(report.stages[1].candidate.scalars[&a], -1.0);
                assert!(report.attempts.iter().any(|attempt| !attempt.committed));
            }
        }
        Ok(())
    })
    .catch_unwind()
    .await;
    context.close().await?;
    match outcome {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

#[tokio::test]
async fn native_explicit_and_conditional_recycles_conserve_and_refuse_unbroken_cycles() -> Result<()> {
    let context = NativeContext::new()?;
    let outcome = AssertUnwindSafe(async {
        for conditional in [false, true] {
            let equations = if conditional {
                "var y:Scalar; var hidden:Scalar;
                 eq local:y==x/2+2; eq internal:hidden==y+3;
                 let output:Scalar=y; annotation start y(1); annotation start hidden(1);"
            } else {
                "let output:Scalar=x/2+2;"
            };
            let source = format!(
                "package strategies {{def Root {{var x:Scalar; {equations}
                 port inlet:Scalar=x; port outlet:Scalar=output;
                 annotation connectivity inlet(1,0); annotation connectivity outlet(0,1);
                 connect outlet -> inlet; annotation start x(1); annotation nominal x(5);
                 }}}}"
            );
            let package = context.package(&source).await?;
            let services = context.runtime.preparation_services();
            let root = package
                .declarations(&services)
                .await?
                .iter()
                .find(|row| row.name == "Root")
                .unwrap()
                .declaration_id;
            let mut preparation = PreparationSettings::default();
            preparation.compiler.evaluation.scratch_bytes = 1 << 20;
            let settings = SolveSettings {
                backend: Some(Backend::Kinsol),
                intent: SolveIntent::Root,
                ..Default::default()
            };
            let cancel = CancelSource::new();
            let execution = Box::pin(package.admit_declared_request(
                &services, root, &preparation, settings.clone(), &cancel,
            ))
            .await?;
            let analysis = &execution.analysis;
            let product = &execution.model.compiled().model;
            let port = |name: &str| {
                product
                    .ports
                    .values()
                    .find(|port| port.lineage.path.ends_with(&format!(".{name}")))
                    .unwrap()
                    .id
            };
            let symbol = |name: &str| {
                product
                    .symbols
                    .values()
                    .find(|symbol| symbol.lineage.path.ends_with(&format!(".{name}")))
                    .unwrap()
                    .id
            };
            let x = symbol("x");
            let selection = ModelingFlowSelection {
                nodes: BTreeSet::from([analysis.instance]),
                connections: product
                    .connections
                    .keys()
                    .map(|id| (*id, Decision { id: *id, cost: 1.0, policy: Policy::Free }))
                    .collect(),
            };
            assert_eq!(selection.connections.len(), 1);
            let realization = if conditional {
                CausalUnitRealization::Conditional {
                    residuals: product
                        .equations
                        .iter()
                        .filter(|row| {
                            row.lineage.path.ends_with(".local")
                                || row.lineage.path.ends_with(".internal")
                        })
                        .map(|row| row.id)
                        .collect(),
                    unknowns: BTreeSet::from([symbol("y"), symbol("hidden")]),
                    solver: Box::new(settings.clone()),
                }
            } else {
                CausalUnitRealization::ExplicitMap
            };
            let request = RecycleRequest {
                tears: selection.connections.keys().copied().collect(),
                units: vec![CausalUnitRequest {
                    node: analysis.instance.as_id(),
                    inputs: BTreeSet::from([port("inlet")]),
                    outputs: BTreeSet::from([port("outlet")]),
                    realization,
                }],
                anderson: 1,
                anderson_delay: 0,
                orthogonalization: None,
                restarts: None,
                damping: pse_model::scalars::Fraction::try_new(1.0)?,
            };
            let prepared = Box::pin(package.prepare_recycle_request(
                &services,
                root,
                &preparation,
                settings.clone(),
                selection.clone(),
                request.clone(),
                &cancel,
            ))
            .await?;
            assert_eq!(prepared.order()?, vec![analysis.instance.as_id()]);
            assert_eq!(prepared.request().tears, request.tears);
            assert_eq!(prepared.conditional_units().count(), usize::from(conditional));
            let report: DeclaredRootReport = prepared.start(&services)?.finish().await?;
            assert_eq!(report.candidate_use(), CandidateUse::Usable, "{report:?}");
            assert_eq!(report.report.qualification, Qualification::Feasible, "{report:?}");
            let candidate = report.report.candidate.as_ref().unwrap();
            assert_eq!(candidate.primal.len(), 1);
            // Independent balance: x = x/2 + 2, so x=4. Check the
            // complete original candidate as well as the tear-space root.
            close_to(candidate.primal[0], 4.0);
            let original = report.original_candidate.as_ref().unwrap();
            close_to(original.scalars[&x], 4.0);
            close_to(original.scalars[&x] - original.scalars[&x] / 2.0, 2.0);
            if conditional {
                close_to(original.scalars[&symbol("y")], 4.0);
                close_to(original.scalars[&symbol("hidden")], 7.0);
                close_to(original.scalars[&symbol("hidden")] - original.scalars[&symbol("y")], 3.0);
            }

            // A physical recycle cannot be run as an acyclic causal schedule
            // without its selected tear; no fallback or partial graph is admitted.
            let mut unbroken = request;
            unbroken.tears.clear();
            let error = Box::pin(package.prepare_recycle_request(
                &services,
                root,
                &preparation,
                settings,
                selection,
                unbroken,
                &cancel,
            ))
            .await
            .unwrap_err();
            assert!(error.to_string().contains("directed cycle"), "{error}");
        }
        Ok(())
    })
    .catch_unwind()
    .await;
    context.close().await?;
    match outcome {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
