// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A connected client of the exact ordinary exchanger source closure.

use super::{endpoint, pb, selected};
use pse_admission::authoring_driver::document::capture_package;
use pse_ids::SemanticId;
use pse_runtime::{
    engineering_projection as codec,
    host_operations::engineering::EngineeringClient,
    workflow::{ConformanceManifest, OriginalSourceFileRequest},
};
use std::{collections::BTreeSet, path::Path};

const CASE: &str = "ordinary_exchanger";
const SOURCE_PATH: &str = "models/delivery.pse";

fn source(operation: String) -> (pb::EngineeringAdmitSource, SemanticId, Vec<u8>) {
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/reference")
        .canonicalize()
        .unwrap();
    let manifest = ConformanceManifest::load(&reference.join("conformance.toml")).unwrap();
    let seed = manifest
        .runs
        .into_iter()
        .find(|run| run.name == "seed")
        .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let capture = |name: &str| {
        let directory = reference.join(name);
        assert_eq!(directory.canonicalize().unwrap(), directory);
        capture_package(&directory, &registry, Default::default()).unwrap()
    };
    let delivery = capture("ordinary-delivery");
    let package = delivery.header.package_id.as_id();
    let exact_source = delivery.files[SOURCE_PATH].clone();
    let wrap = |files: std::collections::BTreeMap<String, Vec<u8>>| pb::EngineeringPackage {
        files: files.into_iter().collect(),
        ..Default::default()
    };
    let physical = wrap(capture(&seed.physical).files);
    let mut modeling = vec![wrap(delivery.files)];
    let mut admitted_paths = BTreeSet::new();
    // Select the same package closure as the native reference. The physical
    // package is also the separately admitted physical context.
    for name in seed
        .dependencies
        .into_iter()
        .chain([seed.package, "data/ordinary-bt".into()])
    {
        if admitted_paths.insert(name.clone()) {
            modeling.push(if name == seed.physical {
                physical.clone()
            } else {
                wrap(capture(&name).files)
            });
        }
    }
    assert!(
        modeling.len() > 16,
        "the actual original closure exceeds the former wire shape"
    );
    (
        pb::EngineeringAdmitSource {
            operation,
            physical: vec![physical],
            modeling,
            ..Default::default()
        },
        package,
        exact_source,
    )
}

async fn reopen_original_source(
    endpoint: &super::EngineeringEndpoint,
    prefix: &str,
) -> (EngineeringClient, pb::EngineeringSourceSelection, Vec<u8>) {
    let mut client = EngineeringClient::connect(endpoint.clone()).await.unwrap();
    let source_operation = format!("{prefix}-source");
    let (request, package, exact_source) = source(source_operation.clone());
    client.generated().admit_source(request).await.unwrap();
    let original = selected(&mut client, &source_operation).await;
    assert_eq!(original.operation, source_operation);
    let mut offset = 0;
    let case = loop {
        let page = client
            .generated()
            .inspect_source(pb::EngineeringInspectSource {
                operation: source_operation.clone(),
                offset,
                limit: 128,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_owned();
        assert_eq!(page.source.as_option(), Some(&original));
        if let Some(row) = page.declarations.iter().find(|row| row.name == CASE) {
            let declaration =
                SemanticId::from_bytes(row.declaration.as_slice().try_into().unwrap());
            assert_ne!(declaration, SemanticId::NIL);
            break row.declaration.clone();
        }
        offset = page
            .next_offset
            .expect("original exchanger declaration absent");
    };

    drop(client);
    let client = EngineeringClient::connect(endpoint.clone()).await.unwrap();
    let file = client
        .read_source_file(pb::EngineeringReadSourceFile {
            source_operation,
            request: codec::encode_original_source_file_request(OriginalSourceFileRequest {
                parent: original.local_revision.clone(),
                package,
                path: SOURCE_PATH.into(),
                maximum_bytes: 1 << 20,
            })
            .unwrap()
            .into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let file = codec::decode_original_source_file(file).unwrap();
    assert_eq!(file.local_revision, original.local_revision);
    assert_eq!(file.body, exact_source);
    assert_eq!(
        file.original_hash,
        pse_ids::encoding_checksum(&exact_source).content_hash()
    );
    (client, original, case)
}

#[tokio::test(flavor = "multi_thread")]
async fn prepared_engineering_ordinary_exchanger_reopens_original_source() {
    let id: SemanticId = pse_operations::mint_id();
    reopen_original_source(&endpoint(), &format!("ordinary-exchanger-{id}")).await;
}

#[cfg(all(feature = "solver-ipopt", feature = "solver-root-isolation"))]
async fn result_rows<T: pse_relations::columnar::RelationRow>(
    client: &EngineeringClient,
    key: &pb::EngineeringRunKey,
    relation: &str,
) -> Vec<T> {
    use datafusion::arrow::ipc::reader::StreamReader;
    use pse_relations::{columnar::FieldCheckedBatch, validate::ValidationContext};
    let page = client
        .read_result(pb::EngineeringReadResult {
            operation: key.operation.clone(),
            run: key.run.clone(),
            attempt: key.attempt.clone(),
            relation: relation.into(),
            start: 0,
            end: 4096,
            maximum_bytes: 16 << 20,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    assert_eq!(page.run, key.run);
    assert_eq!(page.relation, relation);
    let registry = pse_schema::shared_registry().unwrap();
    let spec = T::relation(&registry).unwrap();
    let validation = ValidationContext::local(&registry).unwrap();
    let cancel = pse_columnar::CancellationToken::new();
    let rows = StreamReader::try_new(std::io::Cursor::new(page.ipc), None)
        .unwrap()
        .flat_map(|batch| {
            let checked =
                FieldCheckedBatch::admit(&registry, spec, batch.unwrap(), &validation, &cancel)
                    .unwrap();
            T::rows(&checked).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(rows.len() as u64, page.rows);
    assert!(
        rows.len() < 4096,
        "the original relation must fit the selected range"
    );
    rows
}

#[cfg(all(feature = "solver-ipopt", feature = "solver-root-isolation"))]
#[tokio::test(flavor = "multi_thread")]
async fn prepared_engineering_ordinary_exchanger_reopens_original_physics_and_cost_scope() {
    use pse_model::generated::{
        enums::{
            AccuracyGoalStatus, AccuracyGoalUse, ModelingApplicabilityOutcome as Applicability,
            ModelingPermissionTarget, NativeBackend, NativeSolveIntent, NumericalAccuracyClass,
        },
        identities::{AttemptId, DeclarationId, RunId},
    };
    use pse_relations::generated::runtime::{
        accuracy_goal_assessments, modeling_checks, modeling_reports, solve_runs,
    };

    let endpoint = endpoint();
    let run: RunId = pse_operations::mint_id();
    let attempt: AttemptId = pse_operations::mint_id();
    let prefix = format!("ordinary-exchanger-{}", run.as_id());
    let (mut client, original, case) = reopen_original_source(&endpoint, &prefix).await;
    let case_id = DeclarationId::from_bytes(case.as_slice().try_into().unwrap());
    let key = pb::EngineeringRunKey {
        operation: format!("{prefix}-run"),
        run: run.as_id().as_bytes().to_vec(),
        attempt: attempt.as_id().as_bytes().to_vec(),
        ..Default::default()
    };
    let submitted = client
        .generated()
        .submit_run(pb::EngineeringSubmitRun {
            operation: key.operation.clone(),
            run: key.run.clone(),
            attempt: key.attempt.clone(),
            source_operation: original.operation,
            expected_local_revision: original.local_revision,
            case,
            intent: pb::NativeSolveIntent {
                value: NativeSolveIntent::Root.as_str().into(),
                ..Default::default()
            }
            .into(),
            backend: pb::NativeBackend {
                value: NativeBackend::Ipopt.as_str().into(),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    assert_eq!(
        submitted.run.as_deref(),
        Some(key.run.as_slice()),
        "{submitted:?}"
    );
    drop(client);
    let mut client = EngineeringClient::connect(endpoint).await.unwrap();
    let finished = super::settled(&mut client, &key).await;
    assert_eq!(finished.run, submitted.run);
    assert!(finished.canonical_run.is_some() && finished.canonical_attempt.is_some());
    assert_eq!(finished.canonical_run, submitted.canonical_run);
    assert_eq!(finished.canonical_attempt, submitted.canonical_attempt);
    assert_eq!(finished.preparation, submitted.preparation);
    assert_eq!(finished.preparation.as_deref().map(<[u8]>::len), Some(32));
    let goals = result_rows::<accuracy_goal_assessments::Row>(
        &client,
        &key,
        "runtime.accuracy_goal_assessments",
    )
    .await;
    assert_eq!(
        finished.usable,
        Some(true),
        "{finished:?}; assessments: {goals:?}"
    );
    assert_eq!(goals.len(), 4);
    assert_eq!(
        goals
            .iter()
            .map(|row| row.target_id)
            .collect::<BTreeSet<_>>()
            .len(),
        4
    );
    for goal in goals {
        assert_eq!(goal.run_id, run);
        assert_eq!(goal.case_id, Some(case_id));
        assert_eq!(goal.use_policy, AccuracyGoalUse::Assess);
        assert!(
            matches!(
                goal.status,
                AccuracyGoalStatus::Satisfied | AccuracyGoalStatus::Violated
            ),
            "{goal:?}"
        );
        assert!(
            matches!(
                goal.accuracy_class,
                Some(NumericalAccuracyClass::Estimated | NumericalAccuracyClass::Certified)
            ),
            "{goal:?}"
        );
        assert!(goal.error.is_some_and(f64::is_finite));
    }
    let foreign: AttemptId = pse_operations::mint_id();
    let foreign_attempt = pb::EngineeringRunKey {
        attempt: foreign.as_id().as_bytes().to_vec(),
        ..key.clone()
    };
    let refusal = client
        .generated()
        .run_status(foreign_attempt)
        .await
        .unwrap_err();
    assert_eq!(refusal.code, connectrpc::ErrorCode::InvalidArgument);
    assert_eq!(
        super::failure(refusal).code,
        pse_diagnostics::DiagnosticCode::WorkflowInput.as_str()
    );

    let native = result_rows::<solve_runs::Row>(&client, &key, "runtime.solve_runs").await;
    assert_eq!(native.len(), 1);
    assert_eq!(native[0].run_id, run);
    assert_eq!(native[0].case_id, Some(case_id));
    assert_eq!(native[0].backend, Some(NativeBackend::Ipopt));
    assert_eq!(native[0].feasible, Some(true));

    let reports =
        result_rows::<modeling_reports::Row>(&client, &key, "runtime.modeling_reports").await;
    assert!(reports.iter().all(|row| row.run_id == run));
    let value = |matches: &dyn Fn(&modeling_reports::Row) -> bool| {
        let mut rows = reports.iter().filter(|row| matches(row));
        let value = rows.next().unwrap().value;
        assert!(rows.next().is_none() && value.is_finite());
        value
    };
    let path = |suffix: &str| value(&|row| row.path.ends_with(suffix));
    let label = |label: &str| value(&|row| row.label == label);
    let hot_gain =
        path("exchanger.hot.outlet.energy_flow") - path("exchanger.hot.inlet.energy_flow");
    let cold_gain =
        path("exchanger.cold.outlet.energy_flow") - path("exchanger.cold.inlet.energy_flow");
    assert!(hot_gain < 0.0 && cold_gain > 0.0);
    assert!((hot_gain - label("ordinary hot duty")).abs() < 1.0);
    assert!((cold_gain - label("ordinary cold duty")).abs() < 1.0);
    assert!((hot_gain + cold_gain).abs() < 3.0);
    assert!(label("solved heat-transfer area") > 0.0);
    assert!(label("capital cost at CEPCI 500") > 0.0);

    let checks =
        result_rows::<modeling_checks::Row>(&client, &key, "runtime.modeling_checks").await;
    assert!(checks.iter().all(|row| row.run_id == run && row.satisfied));
    let id = |hex| SemanticId::parse_hex(hex).unwrap();
    let area = [
        id("baeeb286c60d5a0486aacfdc2cf94199"),
        id("41ec0a0739f25f5c88a6a863d352f0c5"),
    ];
    let pressure = [id("60e57bb7b1cb549b9ef8639a300c2e03")];
    let mut permission_ids = BTreeSet::new();
    let area_families = [
        id("e65466ca94df46418736981dbf0d735f"),
        id("58b11994b5255027a11b2b689d0be3dd"),
    ];
    let pressure_families = [id("7fbbf5dd13f65da3a6b5895ddd89f89f")];
    for (claims, families, outcome, unknown, extrapolation) in [
        (
            area.as_slice(),
            area_families.as_slice(),
            Applicability::UnknownEvidence,
            true,
            false,
        ),
        (
            pressure.as_slice(),
            pressure_families.as_slice(),
            Applicability::OutsideRegion,
            false,
            true,
        ),
    ] {
        for claim in claims {
            let rows = checks
                .iter()
                .filter(|row| row.claim_id == Some(*claim))
                .collect::<Vec<_>>();
            assert!(
                !rows.is_empty(),
                "missing original applicability claim {claim}"
            );
            for row in rows {
                assert_eq!(row.applicability_outcome, Some(outcome));
                let permission = row
                    .applicability_permissions
                    .iter()
                    .find(|permission| {
                        permission.scope == case_id.as_id()
                            && permission.target_kind == ModelingPermissionTarget::Families
                            && permission.allow_unknown == unknown
                            && permission.allow_extrapolation == extrapolation
                    })
                    .unwrap();
                assert_eq!(
                    permission.targets.iter().copied().collect::<BTreeSet<_>>(),
                    families.iter().copied().collect::<BTreeSet<_>>()
                );
                permission_ids.insert(permission.permission_id);
            }
        }
    }
    assert_eq!(permission_ids.len(), 2);
}
