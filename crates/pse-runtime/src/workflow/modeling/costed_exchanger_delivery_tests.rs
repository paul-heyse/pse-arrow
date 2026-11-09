// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original costed physics, conditional arithmetic and exact read-only period interpretation.
use super::*;
use pse_model::generated::enums::{
    ModelingApplicabilityOutcome as Applicability, ModelingContributionRole,
    ModelingPermissionTarget,
};
use std::collections::BTreeSet;

fn id(text: &str) -> SemanticId {
    SemanticId::parse_hex(text).unwrap()
}

fn cost_support(
    checks: &[ModelingCheck],
    case: SemanticId,
    area_permission: SemanticId,
    pressure_permission: SemanticId,
) {
    let area_functions = BTreeSet::from([
        id("e65466ca94df46418736981dbf0d735f"),
        id("58b11994b5255027a11b2b689d0be3dd"),
    ]);
    for claim in [
        id("baeeb286c60d5a0486aacfdc2cf94199"),
        id("41ec0a0739f25f5c88a6a863d352f0c5"),
    ] {
        let observations = checks
            .iter()
            .filter(|check| check.claim_id == Some(claim))
            .collect::<Vec<_>>();
        assert!(
            !observations.is_empty(),
            "both original area-support observations are required"
        );
        for check in observations {
            assert!(check.satisfied);
            assert_eq!(
                check.applicability_outcome,
                Some(Applicability::UnknownEvidence)
            );
            assert_eq!(check.unknown_allowed, Some(true));
            assert!(check.applicability_basis.is_none() && check.coverage_id.is_none());
            let permission = check
                .applicability_permissions
                .iter()
                .find(|permission| permission.permission_id == area_permission)
                .unwrap();
            assert_eq!(permission.scope, case);
            assert_eq!(permission.target_kind, ModelingPermissionTarget::Families);
            assert_eq!(
                permission.targets.iter().copied().collect::<BTreeSet<_>>(),
                area_functions
            );
            assert!(permission.allow_unknown && !permission.allow_extrapolation);
            assert!(
                check
                    .input_values
                    .iter()
                    .any(|value| value.name == "area" && value.value > 0.)
            );
        }
    }
    let pressure = checks
        .iter()
        .filter(|check| check.claim_id == Some(id("60e57bb7b1cb549b9ef8639a300c2e03")))
        .collect::<Vec<_>>();
    assert!(!pressure.is_empty());
    for check in pressure {
        assert!(check.satisfied);
        assert_eq!(
            check.applicability_outcome,
            Some(Applicability::OutsideRegion)
        );
        assert_eq!(check.extrapolation_allowed, Some(true));
        assert!(
            check
                .input_values
                .iter()
                .any(|value| value.name == "x" && value.value.abs() <= 1e-8)
        );
        let permission = check
            .applicability_permissions
            .iter()
            .find(|permission| permission.permission_id == pressure_permission)
            .unwrap();
        assert_eq!(permission.scope, case);
        assert_eq!(permission.target_kind, ModelingPermissionTarget::Families);
        assert_eq!(
            permission.targets,
            vec![id("7fbbf5dd13f65da3a6b5895ddd89f89f")]
        );
        assert!(!permission.allow_unknown && permission.allow_extrapolation);
    }
    // Permitted arithmetic is not a purchase-price qualification: the unknown
    // source intervals and outside-region pressure observations remain explicit.
}

#[tokio::test]
async fn ordinary_costed_exchanger_preserves_support_period_and_protected_view()
-> Result<(), WorkflowError> {
    use crate::workflow::{
        ResultInspectionCell, ResultInspectionDocument, ResultInspectionRequest,
        ResultInspectionView, ResultPhysicalContext,
    };
    use base64::Engine as _;
    use buffa_descriptor::{DescriptorPool, DynamicMessage, Value as WireValue};
    use datafusion::arrow::{
        array::{
            BooleanArray, FixedSizeBinaryArray, Float64Array, ListArray, StringArray, StructArray,
        },
        ipc::reader::StreamReader,
    };
    use futures_util::FutureExt as _;
    use pse_backend_native::solve::{Backend, Controls, SolveIntent};
    use pse_relations::{
        columnar::RelationRow as _,
        generated::runtime::{modeling_checks, modeling_reports},
    };

    let (runtime, package, _spill, preparation) =
        pr_jacobian_tests::ordinary_delivery_package(true)
            .await
            .unwrap();
    let physical = package.physical_context().clone();
    let runtime = super::super::durable_tests::durable_runtime_on(runtime);
    let outcome = std::panic::AssertUnwindSafe(async {
        let store = runtime.canonical_store().unwrap();
        store.create_project("costed-exchanger-create", "costed-exchanger-delivery", "Original conditional exchanger costing")
            .await.unwrap();
        let project = store.project("costed-exchanger-delivery").await.unwrap().unwrap();
        let source = Box::pin(runtime.project_fork(&project, "costed-exchanger-main", "costed-exchanger-problem",
            "costed-exchanger-fork", &package)).await.unwrap();
        let revision = source.key.clone();
        let original = Box::pin(runtime.modeling_revision(source, physical.clone(), BTreeMap::new())).await.unwrap();
        let services = runtime.execution_services();
        let declarations = original.declarations(services.preparation()).await.unwrap();
        let selected = declarations.iter().find(|row| row.name == "ordinary_exchanger").unwrap().declaration_id;
        let permission = |name: &str| declarations.iter().find(|row|
            row.name == name && row.parent_id == Some(selected)).unwrap().declaration_id.as_id();
        let area_permission = permission("area_use");
        let pressure_permission = permission("pressure_use");
        let cancel = crate::CancelSource::new();
        let operation = Box::pin(original.prepare_operation(services.preparation(),
            &crate::workflow::OperationSource::of(&original), &preparation,
            crate::workflow::RuntimeOperationRequest::DeclaredCase {
                case: selected,
                settings: Box::new(crate::math::settings::SolveSettings {
                    intent: SolveIntent::Root, backend: Some(Backend::Ipopt),
                    numerics: pse_model::numerics::NumericalPolicy {
                        strict_engineering_context: true, ..Default::default()
                    },
                    controls: Controls { time_limit: std::time::Duration::from_secs(600), ..Default::default() },
                    ..Default::default()
                }),
            }, None, &cancel)).await.unwrap();
        let handle = operation.start(&services, &cancel).await.unwrap();
        let run_key = handle.canonical_run_key().unwrap().to_owned();
        let attempt_key = handle.canonical_attempt_key().unwrap().to_owned();
        let run = handle.wait().await.unwrap();
        let crate::workflow::RunReport::Modeling(results) = run.report().unwrap() else {
            panic!("ordinary costed exchanger requires its actual original modeling result");
        };
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert!(result.accepted && result.validation_error.is_none(), "{:?}", result.diagnostic());
        assert!(!result.checks.is_empty() && result.checks.iter().all(|check| check.satisfied));
        ordinary_delivery_tests::check_exchanger_reports(&result.reports, false).unwrap();
        cost_support(&result.checks, selected.as_id(), area_permission, pressure_permission);
        let use_permission = run.completion().unwrap().step_permission(0).unwrap();
        assert!(use_permission.permits_result());
        assert_eq!(use_permission.reference_point, Some(result.run_id.as_id()));
        let model = &result.prepared.model.model.compiled().model;
        let capital = model.closures.values().filter(|closure|
            closure.lineage.declaration.as_id() == id("01a0f037426c718f919d2139a378952b"))
            .collect::<Vec<_>>();
        assert_eq!(capital.len(), 1);
        assert_eq!(capital[0].mode, pse_model::generated::enums::ModelingAccumulatorMode::Accounting);
        assert_eq!(capital[0].terms.len(), 1, "one original equipment contribution, never counted twice");
        let contributor = &capital[0].terms[0];
        assert_eq!(contributor.role, ModelingContributionRole::Positive);
        assert_eq!(contributor.lineage.declaration.as_id(), id("01a0f037426c718f919d213ad27eadda"));
        assert_eq!(contributor.lineage.instance, capital[0].lineage.instance);
        let expected = result.reports.clone();
        let expected_checks = result.checks.clone();
        let report_keys = expected.iter().map(|row| (row.target_id, row.source_id)).collect::<BTreeSet<_>>();
        let annotation_keys = model.annotations.iter().filter(|annotation|
            matches!(&annotation.value, pse_modeling::annotation::AnnotationValue::Report(_)))
            .map(|annotation| (annotation.target, annotation.lineage.declaration)).collect::<BTreeSet<_>>();
        assert_eq!(report_keys.len(), expected.len());
        assert_eq!(report_keys, annotation_keys, "complete physical and assessment reports");
        let capital_index = expected.iter().position(|row|
            row.source_id.as_id() == id("a2b764f0890043ac882d3c6fc676968d")).unwrap();
        let original_capital = expected[capital_index].clone();
        let registry = physical.quantities();
        let base = registry.units().find(|unit| unit.symbol == "USD_CE500").unwrap();
        let display = registry.units().find(|unit| unit.symbol == "USD_2018").unwrap();
        let quantity = registry.quantity_type(original_capital.quantity_id.into()).unwrap();
        assert_eq!(original_capital.unit_id, base.id.as_id());
        assert_eq!(base.id.as_id(), id("5eeba1c1f9b34e5ea3894a882557517d"));
        assert_eq!(display.id.as_id(), id("80ef33b1eb464d3baed688b08ca1ec04"));
        let conversion = pse_quantity::convert_spec_for_type(base, display, &quantity.key).unwrap();
        assert_eq!(conversion.offset, 0.);
        assert!((conversion.scale - 603.1 / 500.).abs() <= 1e-12);
        let kelvin = registry.units().find(|unit| unit.symbol == "K").unwrap();
        assert!(pse_quantity::convert_spec_for_type(base, kelvin, &quantity.key).is_err());
        let reports_count = expected.len() as u64;
        let checks_count = expected_checks.len() as u64;
        drop((run, handle, operation, original, package));
        runtime.clear_program_cache();

        let token = pse_columnar::CancellationToken::new();
        let mut reader = runtime.results(&run_key, &attempt_key, "runtime.modeling_reports", 0, reports_count, token.child_token())
            .await.unwrap();
        let selection = reader.pack_selection().await.unwrap();
        assert_eq!(selection.usability, crate::workflow::PackScientificState::Usable);
        let context = selection.context.as_ref().unwrap();
        assert_eq!(context.problem, "costed-exchanger-problem");
        assert_eq!(context.revision, revision);
        let mut recovered = Vec::new();
        while let Some(batch) = reader.next_relation_batch().await.unwrap() {
            recovered.extend(modeling_reports::Row::rows(&batch).unwrap());
        }
        assert_eq!(recovered, expected);
        drop(reader);
        let mut reader = runtime.results(&run_key, &attempt_key, "runtime.modeling_checks", 0, checks_count, token.child_token())
            .await.unwrap();
        let checks_selection = reader.pack_selection().await.unwrap();
        let mut recovered_checks = Vec::new();
        while let Some(batch) = reader.next_relation_batch().await.unwrap() {
            recovered_checks.extend(modeling_checks::Row::rows(&batch).unwrap());
        }
        assert_eq!(recovered_checks, expected_checks, "all original support and permissions survive protection");
        cost_support(&recovered_checks, selected.as_id(), area_permission, pressure_permission);
        drop(reader);

        let viewed = runtime.inspect_result(ResultInspectionRequest {
            run: run_key.clone(), attempt: attempt_key.clone(), relation: "runtime.modeling_reports".into(),
            start: capital_index as u64, end: capital_index as u64 + 1,
            expected_manifest: Some(context.manifest.clone()), cursor: None,
            view: ResultInspectionView::Values { fields: vec!["value".into(), "unit_id".into(), "label".into()], maximum_rows: 1 },
        }, token.child_token()).await.unwrap();
        assert!(!token.is_cancelled(), "a completed scalar view cannot cancel sibling reads");
        let ResultInspectionDocument::Values { rows, selected_fields, context_fields, selection: view_selection,
            original: view_original, readable_complete, .. } = viewed else {
            panic!("native original cost view required");
        };
        // Shallow composite logical metadata is explicitly deferred by this
        // existing view, even when that row's optional container is null.
        assert!(!readable_complete);
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].context_complete);
        assert_eq!(view_original.governing_permission, Some(true));
        assert!(matches!(view_original.physical, ResultPhysicalContext::Available { .. }));
        assert_eq!(view_selection.context, selection.context);
        assert_eq!(selected_fields.iter().map(|field| field.name.as_str()).collect::<Vec<_>>(), ["value", "unit_id", "label"]);
        let mut full_metadata = BTreeMap::new();
        for field in selected_fields.iter().chain(&context_fields).filter(|field| !field.metadata_omitted.is_empty()) {
            let mut request = ResultInspectionRequest {
                run: run_key.clone(), attempt: attempt_key.clone(), relation: "runtime.modeling_reports".into(),
                start: capital_index as u64, end: capital_index as u64 + 1,
                expected_manifest: Some(context.manifest.clone()), cursor: None,
                view: ResultInspectionView::Metadata { path: field.path.clone(), offset: 0, limit: 16 },
            };
            let mut complete = BTreeMap::new();
            loop {
                let page = runtime.inspect_result(request.clone(), token.child_token()).await.unwrap();
                assert!(!token.is_cancelled(), "each metadata page owns its child cancellation");
                let ResultInspectionDocument::Metadata { entries, entry_count, next_cursor, selection: original, .. } = page else {
                    panic!("deferred original logical metadata page required");
                };
                assert_eq!(original.context, selection.context);
                for (key, value) in entries { assert!(complete.insert(key, value).is_none()); }
                if let Some(cursor) = next_cursor { request.cursor = Some(cursor); } else {
                    assert_eq!(complete.len(), entry_count);
                    break;
                }
            }
            for omitted in &field.metadata_omitted {
                assert_eq!(complete[&omitted.key].len(), omitted.value_bytes);
            }
            assert!(full_metadata.insert(field.name.clone(), complete).is_none());
        }
        assert!(!full_metadata.is_empty(), "explicit partiality is followed to the original deferred metadata");
        let ResultInspectionCell::Float64 { bits, finite_value: Some(value), .. } = &rows[0].values[0] else {
            panic!("exact finite original currency cell");
        };
        assert_eq!(*bits, original_capital.value.to_bits());
        assert_eq!(*value, original_capital.value);
        assert!(matches!(&rows[0].values[1], ResultInspectionCell::Binary { hex } if *hex == base.id.to_string()));
        assert!(matches!(&rows[0].values[2], ResultInspectionCell::Text { value } if value == "capital cost at CEPCI 500"));
        // Read-only display conversion after all native producer handles were dropped.
        // Original CEPCI500 amount/unit/context remain intact; this is USD cost-index
        // interpretation, not FX, a new physical solve or purchase-price permission.
        let displayed_2018 = value * conversion.scale + conversion.offset;
        assert!((displayed_2018 - original_capital.value * (603.1 / 500.)).abs() <= 1e-6);
        assert_ne!(base.id, display.id);
        assert_eq!(recovered[capital_index].unit_id, base.id.as_id());

        let directory = tempfile::tempdir().unwrap();
        for (relation, count, selected_payload) in [
            ("runtime.modeling_reports", reports_count, &selection),
            ("runtime.modeling_checks", checks_count, &checks_selection),
        ] {
            let path = directory.path().join(format!("{relation}.arrow"));
            let mut reader = runtime.results(&run_key, &attempt_key, relation, 0, count, token.child_token()).await.unwrap();
            reader.export_ipc_bounded(&path, 16 << 20, &token).await.unwrap();
            drop(reader);
            let file = std::fs::File::open(&path).unwrap();
            let bytes = file.metadata().unwrap().len();
            crate::workflow::portable_payload::validate_scientific_ipc(file, bytes, 16 << 20, selected_payload).unwrap();
            let mut portable = StreamReader::try_new(std::fs::File::open(&path).unwrap(), None).unwrap();
            let schema = portable.schema();
            if relation == "runtime.modeling_reports" {
                for (name, metadata) in &full_metadata {
                    let field = schema.field_with_name(name).unwrap();
                    for (key, value) in metadata { assert_eq!(field.metadata().get(key), Some(value)); }
                }
            }
            let metadata_bytes = base64::engine::general_purpose::STANDARD.decode(
                &schema.metadata()[crate::workflow::result_selection::SELECTION_METADATA_KEY]).unwrap();
            let world = Arc::new(DescriptorPool::decode(crate::recorded_result_projection::DESCRIPTORS).unwrap());
            let message = world.message_index("pse.engineering.v1.RecordedResultSelection").unwrap();
            let metadata = DynamicMessage::decode(world, message, &metadata_bytes).unwrap();
            for (tag, text) in [(1, run_key.as_str()), (2, attempt_key.as_str()), (3, relation)] {
                assert_eq!(metadata.field_by_number(tag), Some(&WireValue::String(text.into())));
            }
            let Some(WireValue::Message(usability)) = metadata.field_by_number(10) else { panic!("original passive usability absent") };
            let original_alternative = match selected_payload.usability {
                crate::workflow::PackScientificState::Unknown => 1,
                crate::workflow::PackScientificState::Usable => 2,
                crate::workflow::PackScientificState::Partial => 3,
                crate::workflow::PackScientificState::Failed => 4,
                crate::workflow::PackScientificState::Unusable => 5,
            };
            for tag in 1..=5 {
                assert_eq!(matches!(usability.field_by_number(tag), Some(WireValue::Message(_))), tag == original_alternative);
            }
            let Some(WireValue::Message(wire_context)) = metadata.field_by_number(11) else { panic!("original interpretation context absent") };
            let original_context = selected_payload.context.as_ref().unwrap();
            for (tag, text) in [(1, original_context.problem.as_str()), (2, original_context.revision.as_str()),
                (3, original_context.manifest.as_str()), (4, original_context.interpretation.as_str())] {
                assert_eq!(wire_context.field_by_number(tag), Some(&WireValue::String(text.into())));
            }
            crate::workflow::portable_payload::validate_scientific_lineage(schema.as_ref(), selected_payload).unwrap();
            let mut foreign = selected_payload.clone();
            foreign.context.as_mut().unwrap().revision = "foreign-assessment-period-context".into();
            assert!(crate::workflow::portable_payload::validate_scientific_lineage(schema.as_ref(), &foreign).is_err());
            let mut seen = BTreeSet::new();
            for batch in &mut portable {
                let batch = batch.unwrap();
                let ids = |name| batch.column_by_name(name).unwrap().as_any().downcast_ref::<FixedSizeBinaryArray>().unwrap();
                let values = batch.column_by_name("value").unwrap().as_any().downcast_ref::<Float64Array>().unwrap();
                for index in 0..batch.num_rows() {
                    let target = SemanticId::from_bytes(ids("target_id").value(index).try_into().unwrap());
                    let source = SemanticId::from_bytes(ids("source_id").value(index).try_into().unwrap());
                    assert!(seen.insert((target, source)), "portable original key is unique");
                    if relation == "runtime.modeling_reports" {
                        let row = expected.iter().find(|row| row.target_id == target && row.source_id.as_id() == source).unwrap();
                        for (name, id) in [("run_id", row.run_id.as_id()), ("quantity_id", row.quantity_id), ("unit_id", row.unit_id)] {
                            assert_eq!(ids(name).value(index), id.as_bytes());
                        }
                        assert_eq!(values.value(index).to_bits(), row.value.to_bits());
                    } else {
                        let row = expected_checks.iter().find(|row| row.target_id == target && row.source_id.as_id() == source).unwrap();
                        assert_eq!(values.value(index).to_bits(), row.value.to_bits());
                        let outcome = batch.column_by_name("applicability_outcome").unwrap().as_any().downcast_ref::<StringArray>().unwrap();
                        use datafusion::arrow::array::Array as _;
                        assert_eq!(outcome.is_null(index), row.applicability_outcome.is_none());
                        if let Some(original) = row.applicability_outcome { assert_eq!(outcome.value(index), original.as_str()); }
                        for (name, expected) in [("unknown_allowed", row.unknown_allowed), ("extrapolation_allowed", row.extrapolation_allowed)] {
                            let values = batch.column_by_name(name).unwrap().as_any().downcast_ref::<BooleanArray>().unwrap();
                            assert_eq!(values.is_null(index), expected.is_none());
                            if let Some(expected) = expected { assert_eq!(values.value(index), expected); }
                        }
                        let satisfied = batch.column_by_name("satisfied").unwrap().as_any().downcast_ref::<BooleanArray>().unwrap();
                        assert_eq!(satisfied.value(index), row.satisfied);
                        let permission_ids = batch.column_by_name("permission_ids").unwrap().as_any().downcast_ref::<ListArray>().unwrap().value(index);
                        let permission_ids = permission_ids.as_any().downcast_ref::<FixedSizeBinaryArray>().unwrap();
                        assert_eq!(permission_ids.len(), row.permission_ids.len());
                        for (at, permission) in row.permission_ids.iter().enumerate() { assert_eq!(permission_ids.value(at), permission.as_bytes()); }
                        let permissions = batch.column_by_name("applicability_permissions").unwrap().as_any().downcast_ref::<ListArray>().unwrap().value(index);
                        let permissions = permissions.as_any().downcast_ref::<StructArray>().unwrap();
                        assert_eq!(permissions.len(), row.applicability_permissions.len());
                        for (at, permission) in row.applicability_permissions.iter().enumerate() {
                            for (name, expected) in [("permission_id", permission.permission_id), ("scope", permission.scope)] {
                                let ids = permissions.column_by_name(name).unwrap().as_any().downcast_ref::<FixedSizeBinaryArray>().unwrap();
                                assert_eq!(ids.value(at), expected.as_bytes());
                            }
                            let kind = permissions.column_by_name("target_kind").unwrap().as_any().downcast_ref::<StringArray>().unwrap();
                            assert_eq!(kind.value(at), permission.target_kind.as_str());
                            for (name, expected) in [("allow_unknown", permission.allow_unknown), ("allow_extrapolation", permission.allow_extrapolation)] {
                                let values = permissions.column_by_name(name).unwrap().as_any().downcast_ref::<BooleanArray>().unwrap();
                                assert_eq!(values.value(at), expected);
                            }
                            let targets = permissions.column_by_name("targets").unwrap().as_any().downcast_ref::<ListArray>().unwrap().value(at);
                            let targets = targets.as_any().downcast_ref::<FixedSizeBinaryArray>().unwrap();
                            assert_eq!(targets.len(), permission.targets.len());
                            for (at, target) in permission.targets.iter().enumerate() { assert_eq!(targets.value(at), target.as_bytes()); }
                        }
                    }
                }
            }
            assert_eq!(seen.len() as u64, count, "no original physical/support contributor observation may disappear");
            if relation == "runtime.modeling_reports" {
                assert!(seen.contains(&(original_capital.target_id, original_capital.source_id.as_id())));
            }
        }
    }).catch_unwind().await;
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
                    "costed exchanger original assertion retained; native settlement failed: {error:?}"
                );
            }
            std::panic::resume_unwind(original)
        }
    }
}
