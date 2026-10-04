// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The Rust-owned boundary documents whose JSON Schemas and Python types are published
//! (ADR-0116 Outcomes 6 and 7). Each schema is derived by schemars from the document's
//! serde type in its owning crate; the pure generator renders them.

use pse_codegen::codegen::documents::Document;

fn document<T: schemars::JsonSchema>(name: &'static str) -> Document {
    Document {
        name,
        schema: schemars::schema_for!(T).to_value(),
    }
}

/// Isolated values serialized by the same owners as exported operation products.
/// These fixtures exercise transport only; no solver or operational store is invoked.
pub(super) fn boundary_fixtures() -> anyhow::Result<pse_codegen::codegen::GeneratedTree> {
    use pse_model::generated::{
        enums::*,
        runtime::{computation_runs, solve_runs},
    };
    use pse_model::study::{Conclusion, EffectState, PointAttemptOutcome, ScientificFacts};
    use pse_runtime::workflow::{
        Completion, ProgressEventDocument, TerminationCause, TerminationDetail, WorkflowError,
    };
    use std::collections::BTreeMap;

    fn encoded<T: serde::Serialize + serde::de::DeserializeOwned>(
        value: &T,
    ) -> anyhow::Result<serde_json::Value> {
        let bytes = serde_json::to_vec(value)?;
        let retained: T = serde_json::from_slice(&bytes)?;
        let original = serde_json::to_value(value)?;
        anyhow::ensure!(
            serde_json::to_value(retained)? == original,
            "owner document changed on round trip"
        );
        Ok(original)
    }
    let refusal = WorkflowError::Input("selected binding refused before dispatch".into())
        .boundary_diagnostic();
    let attempt =
        |lifecycle, diagnostic: pse_model::diagnostic::BoundaryDiagnostic| TerminationDetail {
            version: pse_model::document::Version,
            cause: TerminationCause::Error {
                diagnostic: diagnostic.clone(),
            },
            point: Some(PointAttemptOutcome {
                attempt_id: None,
                lifecycle: Some(lifecycle),
                diagnostic: Some(diagnostic),
                scientific: ScientificFacts::default(),
                start: None,
                effect: EffectState::Absent,
            }),
            retry_failure: None,
            effect: EffectState::Absent,
        };
    let cancelled =
        WorkflowError::Math(pse_runtime::math::MathRuntimeError::Cancelled).boundary_diagnostic();
    let incumbent = ProgressEventDocument::from(pse_backend_native::solve::Event {
        phase: "incumbent".into(),
        elapsed: std::time::Duration::from_millis(125),
        values: BTreeMap::new(),
        incumbent: Some(pse_backend_native::solve::IncumbentEvent {
            objective: 12.0,
            dual_bound: Some(11.5),
            gap: Some(0.04),
            nodes: (1_i64 << 53) + 1,
            seconds: 0.125,
            primal: None,
        }),
    });
    let row = computation_runs::Row {
        run_id: pse_model::generated::identities::RunId::from_bytes([7; 16]),
        kind: ComputationKind::Simulation,
        source_identity: pse_ids::ContentHash::from_bytes([8; 32]),
        profile_identity: pse_ids::ContentHash::from_bytes([9; 32]),
        state: NativeRunState::Native,
        termination: None,
        trajectory_termination: Some(TrajectoryTermination::Event),
        backend: Some(NativeBackend::Diffsol),
        native_code: None,
        native_status: None,
        qualification: NativeQualification::Feasible,
        candidate_kind: None,
        candidate_available: true,
        feasible: Some(true),
        completed_time: Some(0.125),
        completed_samples: Some(1),
        estimate_qualified: None,
        response_available: None,
        response_rank: None,
        response_condition: None,
        validation_error: None,
        error: None,
    };
    let qualified = solve_runs::Row {
        run_id: row.run_id,
        step: 0,
        model_id: None,
        revision: None,
        case_id: None,
        instance_id: None,
        backend: Some(NativeBackend::Scip),
        native_code: None,
        native_status: None,
        state: NativeRunState::Native,
        termination: Some(NativeTermination::NodeLimit),
        assurance: NativeAssurance::None,
        qualification: NativeQualification::GapQualified,
        candidate_kind: Some(NativeCandidateKind::FeasiblePoint),
        feasible: Some(true),
        objective: Some(12.0),
        objective_sense: Some(NativeObjectiveSense::Minimize),
        objective_quantity_id: None,
        validation_error: None,
        error: None,
        transformation: None,
        commitment: None,
    };
    let products = BTreeMap::from([
        (
            "partial",
            encoded(&Conclusion {
                availability: StudyAvailability::Partial,
                lifecycle: StudyLifecycle::Terminal,
            })?,
        ),
        (
            "cancelled_partial",
            encoded(&Conclusion {
                availability: StudyAvailability::Partial,
                lifecycle: StudyLifecycle::Cancelled,
            })?,
        ),
        ("pre_result_refusal", encoded(&refusal)?),
        (
            "failed_attempt",
            encoded(&attempt(AttemptState::Failed, refusal))?,
        ),
        (
            "cancelled_attempt",
            encoded(&attempt(AttemptState::Cancelled, cancelled))?,
        ),
        ("incumbent", encoded(&incumbent)?),
        (
            "qualified_incumbent",
            encoded(&Completion {
                solves: vec![qualified],
                ..Default::default()
            })?,
        ),
        (
            "event_ended_trajectory",
            encoded(&Completion {
                computation: Some(row),
                ..Default::default()
            })?,
        ),
    ]);
    let root = std::path::PathBuf::from("python/pse/tests/fixtures/generated-native-boundaries");
    let mut tree = pse_codegen::codegen::GeneratedTree::empty(vec![root.clone()]);
    tree.files.insert(
        root.join("products.json"),
        serde_json::to_vec_pretty(&products)?,
    );
    tree.files.insert(root.join("README.md"), b"<!-- @generated by xtask/src/codegen/schemas.rs; do not edit -->\n\nThese operation-owned Rust values exercise serialization and generated Python decoding. They do not qualify solver or store behavior.\n".to_vec());
    Ok(tree)
}

/// Every published document, in publication order.
pub(super) fn documents() -> Vec<Document> {
    vec![
        document::<pse_runtime::workflow::SimulationProfile>("simulation-profile"),
        document::<pse_runtime::workflow::InitializationOverrides>("initialization-overrides"),
        document::<pse_runtime::workflow::ModelingNonlinearPolicy>("nonlinear-explanation-policy"),
        document::<pse_runtime::workflow::ModelingDiagnosticPolicy>("modeling-diagnostic-policy"),
        document::<pse_backend_native::solve::WarmStartSnapshot>("warm-start-snapshot"),
        document::<pse_runtime::workflow::DiagnosticCauseDocument>("diagnostic-cause"),
        document::<pse_runtime::workflow::DiagnosticSpanDocument>("diagnostic-span"),
        document::<pse_runtime::workflow::DiagnosticNoteDocument>("diagnostic-note"),
        document::<pse_runtime::workflow::DiagnosticAnnotationDocument>("diagnostic-annotation"),
        document::<pse_runtime::workflow::DiagnosticContextDocument>("diagnostic-context"),
        document::<pse_engine::cache_service::CacheReport>("cache-report"),
        document::<pse_runtime::ResourceReport>("resource-report"),
        document::<pse_runtime::TableName>("table-name"),
        document::<pse_buildinfo::BuildInfo>("build-info"),
        document::<pse_runtime::workflow::ProgressEventDocument>("progress-event"),
        document::<pse_runtime::workflow::RouteDocument>("route"),
        document::<pse_runtime::workflow::EligibilityDocument>("eligibility"),
        document::<pse_runtime::workflow::Workspace>("workspace"),
        document::<pse_runtime::workflow::Published>("published"),
        document::<pse_runtime::workflow::PublicationSettlement>("publication-settlement"),
        document::<pse_runtime::workflow::ExportReceipt>("export-receipt"),
        document::<pse_runtime::workflow::Completion>("completion"),
        document::<pse_runtime::workflow::FitPreparationDocument>("fit-preparation"),
        document::<pse_runtime::workflow::FitProfileDocument>("fit-profiles"),
        document::<pse_runtime::workflow::FitDeclarations>("fit-declarations"),
        document::<pse_runtime::workflow::InventoryControls>("inventory-controls"),
        document::<pse_runtime::workflow::ProgressControls>("progress-controls"),
        document::<pse_runtime::workflow::RunControls>("run-controls"),
        document::<pse_runtime::workflow::StudyRunControls>("study-run-controls"),
        document::<pse_runtime::workflow::StudySubmitControls>("study-submit-controls"),
        document::<pse_runtime::workflow::StudyWaitControls>("study-wait-controls"),
        document::<pse_runtime::workflow::KnowledgeControls>("knowledge-controls"),
        document::<pse_runtime::workflow::DiagnosticSamplesControls>("diagnostic-samples-controls"),
        document::<pse_runtime::workflow::JacobianDiagnosticControls>(
            "jacobian-diagnostic-controls",
        ),
        document::<pse_runtime::workflow::LinearDiagnosticControls>("linear-diagnostic-controls"),
        document::<pse_runtime::workflow::PureConformanceControls>("pure-conformance-controls"),
        document::<pse_runtime::workflow::ConformanceControls>("conformance-controls"),
        document::<pse_runtime::workflow::DeclarationEdit>("declaration-edit"),
        document::<pse_runtime::workflow::DeclarationInventory>("declaration-inventory"),
        document::<pse_runtime::workflow::ModelingInspection>("modeling-inspection"),
        document::<pse_runtime::workflow::ConicRequest>("conic-request"),
        document::<pse_runtime::workflow::RecycleRequest>("recycle-request"),
        document::<pse_runtime::math::flows::FlowSelectionDocument>("flow-selection"),
        document::<pse_runtime::math::flows::FlowGraphDocument>("flow-graph"),
        document::<pse_runtime::math::flows::TearSelectionDocument>("tear-selection"),
        document::<pse_runtime::workflow::InitializationDocument>("initialization"),
        document::<pse_runtime::math::settings::SolveSettings>("solve-settings"),
        document::<pse_model::strategy::NumericalStrategyDocument>("numerical-strategy"),
        document::<pse_model::strategy::CompositionRequest>("composition-request"),
        document::<pse_backend_native::execution::BackendSettings>("backend-settings"),
        document::<pse_backend_native::dynamics::DiffsolSettings>("diffsol-settings"),
        document::<pse_backend_native::dynamics::IdasSettings>("idas-settings"),
        document::<pse_backend_native::dynamics::AdjointSettings>("adjoint-settings"),
        document::<pse_runtime::workflow::JobPayload>("job-payload"),
        document::<pse_runtime::workflow::StudyDefinition>("study-definition"),
        document::<pse_runtime::workflow::StudyRequest>("study-request"),
        document::<pse_runtime::workflow::StudyStatus>("study-status"),
        document::<pse_runtime::workflow::StudyCancel>("study-cancel"),
        document::<pse_runtime::workflow::PointOutcome>("study-point-outcome"),
        document::<pse_runtime::workflow::StudyConclusion>("study-conclusion"),
        document::<pse_runtime::workflow::BoundaryDiagnostic>("boundary-diagnostic"),
        document::<pse_runtime::workflow::TerminationDetail>("termination-detail"),
        document::<pse_runtime::workflow::SourceManifest>("source-manifest"),
        document::<pse_runtime::workflow::FitUncertainty>("fit-uncertainty"),
        document::<pse_runtime::math::PreparationCounts>("preparation-counts"),
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use pse_authoring::generated::documents::{
        AssertionsDocument, CasesDocument, MaterialsDocument, ModelingDocument,
        PackageHeaderDocument,
    };
    use serde_json::Value;

    /// A schema with its local `$ref` followed.
    fn resolve<'a>(root: &'a Value, schema: &'a Value) -> &'a Value {
        match schema
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|target| target.strip_prefix("#/$defs/"))
        {
            Some(name) => resolve(root, &root["$defs"][name]),
            None => schema,
        }
    }

    /// The string members a (possibly nullable) enumeration schema admits.
    fn members(root: &Value, schema: &Value) -> Option<BTreeSet<String>> {
        let schema = resolve(root, schema);
        if let Some(values) = schema.get("enum").and_then(Value::as_array) {
            return Some(
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
            );
        }
        schema
            .get("anyOf")
            .or_else(|| schema.get("oneOf"))
            .and_then(Value::as_array)
            .and_then(|alternatives| alternatives.iter().find_map(|a| members(root, a)))
    }

    /// B5.6 (ADR-0116 Outcome 10): the authoring schema stays with its registry emitter.
    ///
    /// schemars derives the schema of the generated authoring document structs, which are
    /// the documents after parsing and identity hydration. On everything the two contracts
    /// share it is equivalent to the registry emitter's schema: every source field is a
    /// hydrated field and every enumeration states the same registry members. But the
    /// published schema is the one an author's editor validates *sources* against, before
    /// hydration: the identity alias `id` and its UUID form, package-context columns an
    /// author omits, and no parser-span or package-integrity columns. The hydrated structs
    /// require fields no source states, so a schema derived from them would refuse every
    /// valid source document; the registry emitter is the more precise one and stays.
    #[test]
    fn authoring_schema_equivalent_under_schemars() {
        let registry = pse_schema::registry().unwrap();
        let emitted: Value =
            serde_json::from_str(&pse_codegen::codegen::jsonschema::generate(registry).unwrap())
                .unwrap();
        let derived = [
            ("assertions", schemars::schema_for!(AssertionsDocument)),
            ("cases", schemars::schema_for!(CasesDocument)),
            ("materials", schemars::schema_for!(MaterialsDocument)),
            ("modeling", schemars::schema_for!(ModelingDocument)),
            (
                "package_header",
                schemars::schema_for!(PackageHeaderDocument),
            ),
        ];
        let mut sections = 0;
        let mut enumerations = 0;
        let mut hydration_only = BTreeSet::new();
        for (name, schema) in derived {
            let root = schema.to_value();
            let declaration = registry
                .documents()
                .iter()
                .find(|document| document.name == name)
                .unwrap();
            for section in &declaration.sections {
                let source = &emitted["$defs"][format!("source:{name}:{}", section.key)];
                let property = &root["properties"][section.key];
                let row = resolve(&root, property.get("items").unwrap_or(property));
                let hydrated = row["properties"].as_object().unwrap();
                let written = source["properties"].as_object().unwrap();
                for (field, value) in written.iter().filter(|(field, _)| *field != "id") {
                    assert!(
                        hydrated.contains_key(field),
                        "{name}.{}.{field}",
                        section.key
                    );
                    if let Some(expected) = members(&emitted, value) {
                        assert_eq!(
                            members(&root, &hydrated[field]),
                            Some(expected),
                            "{name}.{}.{field}",
                            section.key
                        );
                        enumerations += 1;
                    }
                }
                for field in row["required"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(Value::as_str)
                {
                    if !written.contains_key(field) {
                        hydration_only.insert(format!("{name}.{}.{field}", section.key));
                    }
                }
                sections += 1;
            }
        }
        assert_eq!(
            sections,
            registry
                .documents()
                .iter()
                .map(|d| d.sections.len())
                .sum::<usize>()
        );
        assert!(enumerations > 0);
        // The package's computed content hash is required only after hydration.
        assert_eq!(
            hydration_only,
            BTreeSet::from(["package_header.package.content_hash".to_owned()])
        );
    }
}
