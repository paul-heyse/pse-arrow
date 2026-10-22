// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The Rust-owned boundary documents whose JSON Schemas and Python types are published
//! (ADR-0116 Outcomes 6 and 7). Each schema is derived by schemars from the document's
//! serde type in its owning crate; the pure generator renders them.


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
        source: None,
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
    // Fixed clock/locator keep generated codec controls reproducible. These are actual
    // serialized owner values, not a second handwritten boundary shape.
    struct FixtureClock;
    impl pse_diagnostics::observation::Clock for FixtureClock {
        fn now(&self) -> std::time::Duration {
            std::time::Duration::from_secs(3)
        }
    }
    let hub = pse_diagnostics::observation::ObservationHub::with_clock(
        pse_diagnostics::observation::Policy::default(),
        std::sync::Arc::new(FixtureClock),
    );
    let operation = hub.start("codec.control");
    operation
        .context()
        .count("callback.calls", "calls", (1_u64 << 53) + 1);
    operation
        .context()
        .gauge("ipopt.native_residual", "scaled", f64::INFINITY);
    operation.context().unavailable(
        "sensitivity.withheld_cause",
        "original assessment unavailable",
    );
    let reader = operation.reader();
    operation.finish(pse_diagnostics::observation::Outcome::Cancelled);
    let mut snapshot = reader.snapshot(std::time::Duration::from_secs(60));
    snapshot.id = pse_diagnostics::observation::OperationId {
        process_instance: [1, 2],
        sequence: 3,
    };
    let mut request = pse_runtime::tooling::inspection::InspectionRequest::new(snapshot.id);
    request.include_related = false;
    let diagnostic = pse_runtime::tooling::inspection::inspect_snapshot(snapshot, request);
    let products = BTreeMap::from([
        ("diagnostic_inspection", encoded(&diagnostic)?),
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

/// The actual compiled catalogue supplies both publication and native registrations.
pub(super) fn documents() -> Vec<pse_codegen::codegen::documents::CatalogueDocument> {
    pse_runtime::boundary_documents::schemas().into_iter().map(|entry| pse_codegen::codegen::documents::CatalogueDocument {
        name: entry.name.into(), slug: entry.slug.into(), rust_type: entry.rust_type.into(),
        output: entry.direction == pse_runtime::boundary_documents::Direction::Output,
        version: entry.version, schema: entry.schema,
    }).collect()
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
