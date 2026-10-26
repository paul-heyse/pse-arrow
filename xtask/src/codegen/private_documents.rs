// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual compiled private document contributions, separate from the product catalogue.
use pse_codegen::codegen::documents::CatalogueDocument;
use schemars::JsonSchema;

fn schema<T: JsonSchema>(name: &str, slug: &str, owner: &str, output: bool) -> CatalogueDocument {
    let settings = schemars::generate::SchemaSettings::draft2020_12();
    let settings = if output {
        settings.for_serialize()
    } else {
        settings.for_deserialize()
    };
    let mut schema = settings
        .into_generator()
        .into_root_schema_for::<T>()
        .to_value();
    schema["title"] = name.into();
    schema.sort_all_objects();
    let version = schema["properties"]["version"]["const"]
        .as_u64()
        .and_then(|version| u32::try_from(version).ok());
    CatalogueDocument {
        name: name.into(),
        slug: slug.into(),
        rust_type: owner.into(),
        registration_source: "xtask/src/codegen/private_documents.rs".into(),
        output,
        version,
        wire: "Native".into(),
        schema,
    }
}

pub(super) fn documents() -> Vec<CatalogueDocument> {
    let mut documents = Vec::new();
    macro_rules! add {
        ($ty:ty, $name:literal, $slug:literal, $output:expr) => {
            documents.push(schema::<$ty>($name, $slug, stringify!($ty), $output));
        };
    }
    add!(
        xtask::development::knowledge::KnowledgeQuery,
        "KnowledgeQuery",
        "knowledge-query",
        false
    );
    add!(
        xtask::development::knowledge::KnowledgePage,
        "KnowledgePage",
        "knowledge-page",
        true
    );
    add!(
        xtask::development::knowledge::KnowledgeExplain,
        "KnowledgeExplain",
        "knowledge-explain",
        true
    );
    add!(
        xtask::development::evidence::EvidenceSnapshot,
        "EvidenceSnapshot",
        "evidence-snapshot",
        false
    );
    add!(
        xtask::development::evidence::EvidenceSnapshot,
        "EvidenceSnapshot",
        "evidence-snapshot",
        true
    );
    add!(
        xtask::development::evidence_query::EvidenceComparisonRequest,
        "EvidenceComparisonRequest",
        "evidence-comparison-request",
        false
    );
    add!(
        xtask::development::evidence_query::EvidenceComparisonRequest,
        "EvidenceComparisonRequest",
        "evidence-comparison-request",
        true
    );
    add!(
        xtask::development::evidence_query::EvidenceComparison,
        "EvidenceComparison",
        "evidence-comparison",
        false
    );
    add!(
        xtask::development::evidence_query::EvidenceComparison,
        "EvidenceComparison",
        "evidence-comparison",
        true
    );
    documents.sort_by(|left, right| (&left.slug, left.output).cmp(&(&right.slug, right.output)));
    documents
}
