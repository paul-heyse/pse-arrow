// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Field facet meanings, declared once and projected into the lower columnar layer.
//! Physical observation retains every key. Undeclared keys remain significant.
//! Use `just codegen-bootstrap` after changing these policies: the first stage emits
//! the columnar table and the second rebuilds its consumers before registry generation.

/// Whether target value admission may attach a declaration to untyped storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Admission {
    /// Presentation and occurrence usage do not establish a value's meaning.
    Ignore,
    /// Source and target must both carry exactly the same meaning.
    Exact,
    /// An untyped value may be checked, but established meaning cannot be changed.
    Preserve,
}

/// A metadata spelling and the independent questions its consumers ask.
#[derive(Clone, Copy, Debug)]
pub struct Facet {
    /// Rust constant exported for this spelling.
    pub constant: &'static str,
    /// Native metadata key.
    pub key: &'static str,
    /// The unbound domain declaration spelling, as opposed to a materialized spelling.
    pub domain: bool,
    /// Retain this key in execution identity at every depth.
    pub execution: bool,
    /// Retain this key in value identity at every depth.
    pub value: bool,
    /// Retain this key at the root of the logical/storage projection.
    pub storage_root: bool,
    /// Retain this key in logical type identity at every depth.
    pub type_identity: bool,
    /// Directional target admission, independent of identity projection.
    pub admission: Admission,
}

macro_rules! facets {
    ($(($constant:ident, $key:literal, $domain:literal, $execution:literal, $value:literal,
        $storage_root:literal, $type_identity:literal, $admission:ident)),* $(,)?) => {
        $(#[doc = concat!("Declared field facet `", $key, "`.")]
        pub const $constant: &str = $key;)*
        /// Complete facet inventory. Restoration always preserves every present annotation.
        pub const FACETS: &[Facet] = &[$(Facet {
            constant: stringify!($constant), key: $constant, domain: $domain,
            execution: $execution, value: $value, storage_root: $storage_root,
            type_identity: $type_identity, admission: Admission::$admission,
        }),*];
    };
}

facets! {
    (EXTENSION, "pse.domain.extension", true, true, true, true, true, Preserve),
    (PARAMETER, "pse.domain.parameter", true, true, true, true, true, Preserve),
    (ROLE, "pse.domain.role", true, true, false, false, true, Ignore),
    (DOC, "pse.domain.doc", true, false, false, false, true, Ignore),
    (STRUCTURE, "pse.domain.structure", true, false, false, true, false, Ignore),
    (QUANTITY, "pse.domain.quantity", true, true, true, false, true, Exact),
    (TRANSFER_CONTEXT, "pse.domain.transfer_context", true, true, true, true, true, Exact),
    (FK_RELATION, "pse.domain.fk.relation", true, true, false, false, true, Ignore),
    (FK_COLUMN, "pse.domain.fk.column", true, true, false, false, true, Ignore),
    (IDENTITY, "pse.domain.identity", true, true, true, false, true, Preserve),
    (IDENTITY_OWNER, "pse.domain.identity.owner", true, true, true, false, true, Ignore),
    (DOCUMENT, "pse.domain.document", true, true, true, true, true, Preserve),
    (KEY_LOGICAL_TYPE, "pse.semantic.logical_type", false, true, true, true, true, Ignore),
    (KEY_QUANTITY_TYPE, "pse.semantic.quantity_type", false, true, true, true, true, Exact),
    (KEY_TRANSFER_CONTEXT, "pse.semantic.transfer_context", false, true, true, true, true, Exact),
    (KEY_ROLE, "pse.semantic.role", false, true, false, true, true, Ignore),
    (KEY_FK, "pse.semantic.fk", false, true, false, true, true, Ignore),
    (KEY_ENUM, "pse.semantic.enum", false, true, true, true, true, Preserve),
    (KEY_IDENTITY, "pse.semantic.identity", false, true, true, true, true, Preserve),
    (KEY_DOCUMENT, "pse.semantic.document", false, true, true, true, true, Preserve),
    (KEY_ROW_KEY_ENCODING, "pse.semantic.key_encoding", false, true, true, true, true, Preserve),
    (KEY_EXTENSION_NAME, "ARROW:extension:name", false, true, true, true, true, Preserve),
    (KEY_EXTENSION_METADATA, "ARROW:extension:metadata", false, true, true, true, true, Preserve),
    (KEY_COLLECTION, "pse.semantic.collection", false, true, true, true, true, Preserve),
    (KEY_INTEGER_RANGE, "pse.semantic.integer_range", false, true, true, true, true, Preserve),
    (KEY_TAGGED_ALTERNATIVE, "pse.semantic.tagged_alternative", false, true, true, true, true, Preserve),
    (KEY_REFERENCE, "pse.semantic.reference", false, true, false, false, true, Ignore),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_transfer_context_is_a_declared_semantic_structure() {
        let registry = crate::registry().unwrap();
        let report = registry.relation("runtime.modeling_reports").unwrap();
        let field = report
            .columns
            .iter()
            .find(|field| field.name() == "transfer_context")
            .unwrap();
        assert_eq!(field.transfer_context(), Some("1"));
        let bound = crate::arrow::field_for(registry, field).unwrap();
        assert_eq!(
            bound
                .metadata()
                .get(KEY_TRANSFER_CONTEXT)
                .map(String::as_str),
            Some("1")
        );
        for purpose in [
            pse_columnar::native_field::MetadataPurpose::ExecutionIdentity,
            pse_columnar::native_field::MetadataPurpose::ValueIdentity,
            pse_columnar::native_field::MetadataPurpose::LogicalTypeIdentity,
        ] {
            assert!(
                pse_columnar::native_field::project(&bound, purpose)
                    .unwrap()
                    .metadata()
                    .contains_key(KEY_TRANSFER_CONTEXT)
            );
        }
        let invalid = super::super::FieldContract::native(arrow_schema::DataType::Float64)
            .with_transfer_context();
        assert!(crate::arrow::field_for(registry, &invalid).is_err());
    }

    #[test]
    fn declared_field_facets_have_unique_spellings_and_constants() {
        let keys = FACETS
            .iter()
            .map(|facet| facet.key)
            .collect::<std::collections::BTreeSet<_>>();
        let names = FACETS
            .iter()
            .map(|facet| facet.constant)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(keys.len(), FACETS.len());
        assert_eq!(names.len(), FACETS.len());
        assert!(
            FACETS
                .iter()
                .all(|facet| facet.domain == facet.key.starts_with("pse.domain."))
        );
    }

    #[test]
    fn logical_value_type_preserves_nested_meaning_and_type_identity_omits_names() {
        use super::super::{ColumnRole, FieldContract};
        use arrow_schema::DataType;
        let child = FieldContract::native(DataType::Float64)
            .with_name("enthalpy")
            .with_quantity("MolarEnthalpy")
            .with_doc("nested meaning");
        let plain = FieldContract::structure(vec![child.clone()]);
        let named = plain.clone().named("PhysicalState");
        assert_eq!(plain.type_name().unwrap(), named.type_name().unwrap());
        let declared = named
            .clone()
            .with_name("state")
            .optional()
            .with_role(ColumnRole::Payload)
            .with_doc("root prose")
            .with_identity("State");
        let storage = declared.value_type();
        assert_eq!(storage.name(), "item");
        assert!(!storage.nullable());
        assert_eq!(storage.structure_name(), Some("PhysicalState"));
        assert_eq!(storage.children(), vec![child]);
        assert!(storage.identity().is_none());
        assert_eq!(declared.type_name().unwrap(), plain.type_name().unwrap());
    }
}
