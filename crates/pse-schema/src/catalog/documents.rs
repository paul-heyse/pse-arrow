// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Authoring sections map to exact declared rows (blueprint §22.1, ADR-0051).

use crate::builder::RegistryBuilder;
use crate::model::{DocumentKind, DocumentSection, DocumentSpec};

/// Declare each package surface once. Generated DTOs and the loader share these mappings.
pub fn declare(builder: &mut RegistryBuilder) {
    builder.declare_document(DocumentSpec {
        name: "package_header",
        kind: DocumentKind::PackageHeader,
        path_glob: "package.toml",
        sections: vec![
            DocumentSection {
                key: "package",
                relation: "authored.packages",
                repeated: false,
                identity_column: Some("package_id"),
                entity_kind: None,
                name_column: None,
                naming_scope_column: None,

                doc: "The package header; content_hash is supplied by the loader.",
            },
            DocumentSection {
                key: "unit_sets",
                relation: "authored.package_unit_sets",
                repeated: true,
                identity_column: None,
                entity_kind: None,
                name_column: None,
                naming_scope_column: None,

                doc: "Optional explicit representation-unit selector, at most one per package.",
            },
        ],
        doc: "Package identity, exact dependencies and authored identity policy.",
    });
    builder.declare_document(DocumentSpec{
        name:"modeling",kind:DocumentKind::Modeling,path_glob:"models/*.pse",
        sections:vec![DocumentSection{
            key:"modeling_declarations",relation:"authored.modeling_declarations",repeated:true,
            identity_column:None,entity_kind:None,name_column:None,naming_scope_column:None,

            doc:"Generic modeling declarations and their exact source ranges; emitted by the modeling parser.",
        }],
        doc:"Generic modeling source uses the shared expression language and registry-generated IR.",
    });
    // ADR-0125: package data documents are Parquet bytes decoded by Arrow type; a dataset
    // declaration names one by its path and admits its rows through its relation.
    builder.declare_document(DocumentSpec {
        name: "data",
        kind: DocumentKind::Data,
        path_glob: "data/*.parquet",
        sections: Vec::new(),
        doc: "Typed bulk rows a dataset declaration admits through its declared relation; units and references come only from that declaration.",
    });
    declare_materials(builder);
    declare_cases(builder);
    entities(
        builder,
        "assertions",
        "assertions/*.yaml",
        &["provenance.assertions"],
    );
}

fn declare_materials(builder: &mut RegistryBuilder) {
    entities(
        builder,
        "materials",
        "materials/*.yaml",
        &[
            "reference.dimensions",
            "reference.units",
            "reference.unit_sets",
            "reference.quantity_kinds",
            "reference.bases",
            "reference.reference_states",
            "reference.quantity_types",
            "reference.conversion_rules",
            "reference.quantity_operations",
            "reference.quantity_operation_reductions",
            "reference.quantity_preconditions",
            "reference.math_context",
        ],
    );
}

fn declare_cases(builder: &mut RegistryBuilder) {
    entities(builder, "cases", "cases/*.yaml", &["authored.fit_cases"]);
}

fn entities(
    builder: &mut RegistryBuilder,
    name: &'static str,
    path_glob: &'static str,
    relations: &[&'static str],
) {
    builder.declare_document(DocumentSpec {
        name,
        kind:DocumentKind::Entities,
        path_glob,
        sections:relations.iter().map(|relation|DocumentSection {
            key:relation.rsplit_once('.').map_or(*relation,|(_,name)|name),
            relation,
            repeated:true,
            identity_column: projection(relation).0,
            entity_kind: projection(relation).1,
            name_column: projection(relation).2,
            naming_scope_column: projection(relation).3,

            doc:"Typed rows under the declared relation contract.",
        }).collect(),
        doc:"Package sections use declared relation fields; identity and provenance are resolved before typed decoding.",
    });
}

// Explicit projections are shared by every document exposing a relation. A missing
// entry means a relationship row, never permission to guess identity from its key.
type Projection = (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
);
fn projection(relation: &str) -> Projection {
    match relation {
        "reference.units" => (Some("unit_id"), Some("unit"), Some("symbol"), None),
        "reference.unit_sets" => (Some("unit_set_id"), Some("unit_set"), Some("name"), None),
        "reference.quantity_kinds" => (
            Some("quantity_kind_id"),
            Some("quantity_kind"),
            Some("name"),
            None,
        ),
        "reference.bases" => (Some("basis_id"), None, None, None),
        "reference.reference_states" => (Some("reference_state_id"), None, None, None),
        "reference.quantity_types" => (Some("quantity_type_id"), None, None, None),
        "reference.conversion_rules" => (Some("conversion_id"), None, None, None),
        "reference.quantity_operations" => (Some("operation_id"), None, None, None),
        "provenance.assertions" => (Some("assertion_id"), None, None, None),
        _ => (None, None, None, None),
    }
}
