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

/// Every published document, in publication order.
pub(super) fn documents() -> Vec<Document> {
    vec![
        document::<pse_runtime::math::settings::SolveSettings>("solve-settings"),
        document::<pse_backend_native::execution::BackendSettings>("backend-settings"),
        document::<pse_backend_native::dynamics::DiffsolSettings>("diffsol-settings"),
        document::<pse_backend_native::dynamics::IdasSettings>("idas-settings"),
        document::<pse_backend_native::dynamics::AdjointSettings>("adjoint-settings"),
        document::<pse_runtime::workflow::JobPayload>("job-payload"),
        document::<pse_runtime::workflow::StudyDefinition>("study-definition"),
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
        // Hydration-owned fields a source never states: the reason the emitter stays.
        assert!(
            hydration_only
                .iter()
                .any(|field| field.ends_with(".source_span")
                    || field.ends_with(".package_checksum")
                    || field.contains("span")),
            "{hydration_only:?}"
        );
    }
}
