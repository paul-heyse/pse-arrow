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
    (GRAPH_ENDPOINT, "pse.domain.graph_endpoint", true, false, false, false, false, Ignore),
    (IDENTITY, "pse.domain.identity", true, true, true, false, true, Preserve),
    (IDENTITY_OWNER, "pse.domain.identity.owner", true, true, true, false, true, Ignore),
    (DOCUMENT, "pse.domain.document", true, true, true, true, true, Preserve),
    (EXPRESSION_ROLE, "pse.domain.expression.role", true, true, false, false, true, Ignore),
    (EXPRESSION_ORDINAL, "pse.domain.expression.ordinal", true, true, false, false, true, Ignore),
    (EXPRESSION_BINDER, "pse.domain.expression.binder", true, true, false, false, true, Ignore),
    (EXPRESSION_CASES, "pse.domain.expression.cases", true, true, true, true, true, Preserve),
    (RESULT_SELECTION, "pse.domain.result.selection", true, true, false, false, true, Ignore),
    (RESULT_SORT, "pse.domain.result.sort", true, true, false, false, true, Ignore),
    (KEY_LOGICAL_TYPE, "pse.semantic.logical_type", false, true, true, true, true, Ignore),
    (KEY_QUANTITY_TYPE, "pse.semantic.quantity_type", false, true, true, true, true, Exact),
    (KEY_TRANSFER_CONTEXT, "pse.semantic.transfer_context", false, true, true, true, true, Exact),
    (KEY_EXPRESSION_ROLE, "pse.semantic.expression.role", false, true, false, true, true, Ignore),
    (KEY_EXPRESSION_ORDINAL, "pse.semantic.expression.ordinal", false, true, false, true, true, Ignore),
    (KEY_EXPRESSION_BINDER, "pse.semantic.expression.binder", false, true, false, true, true, Ignore),
    (KEY_EXPRESSION_CASES, "pse.semantic.expression.cases", false, true, true, true, true, Preserve),
    (KEY_RESULT_SELECTION, "pse.semantic.result.selection", false, true, false, true, true, Ignore),
    (KEY_RESULT_SORT, "pse.semantic.result.sort", false, true, false, true, true, Ignore),
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
    fn result_access_roles_do_not_change_the_referenced_value_domain() {
        use crate::model::{FieldContract, ResultSelectionRole};
        let value = FieldContract::native(arrow_schema::DataType::Float64);
        let selected = value
            .clone()
            .with_result_selection(ResultSelectionRole::Value)
            .with_result_sort(0);
        assert_eq!(selected.value_type(), value.value_type());
        assert_ne!(
            selected, value,
            "the occurrence still owns its access declaration"
        );
        let registry = crate::catalog::assemble().unwrap();
        let field = registry
            .relation("runtime.fit_parameters")
            .unwrap()
            .column("value")
            .unwrap();
        let bound = crate::arrow::field_for(&registry, field).unwrap();
        assert_eq!(bound.metadata()[KEY_RESULT_SELECTION], "value");
    }

    #[test]
    fn result_selection_fields_keep_distinct_scientific_keys_and_partitions() {
        use crate::model::ResultSelectionRole as R;
        let registry = &crate::catalog::assemble().unwrap();
        for (name, owner, partition, expected_sort) in [
            (
                "solve_variables",
                "symbol_id",
                Some(("step", R::Step)),
                vec!["run_id", "step", "symbol_id"],
            ),
            (
                "solve_constraints",
                "row_id",
                Some(("step", R::Step)),
                vec!["run_id", "step", "row_id"],
            ),
            (
                "fit_parameters",
                "parameter_id",
                None,
                vec!["run_id", "parameter_id"],
            ),
            (
                "fit_observations",
                "observation_id",
                Some(("experiment_id", R::Experiment)),
                vec!["run_id", "experiment_id", "observation_id"],
            ),
            ("simulation_samples", "symbol_id", None, vec![]),
        ] {
            let relation = registry.relation(&format!("runtime.{name}")).unwrap();
            assert_eq!(
                relation.column("run_id").unwrap().result_selection_role(),
                Some(R::Run)
            );
            assert_eq!(
                relation.column(owner).unwrap().result_selection_role(),
                Some(R::Owner)
            );
            if let Some((field, role)) = partition {
                assert_eq!(
                    relation.column(field).unwrap().result_selection_role(),
                    Some(role)
                );
            }
            let mut order = relation
                .columns
                .iter()
                .filter_map(|field| {
                    field
                        .result_sort_ordinal()
                        .map(|ordinal| (ordinal, field.name()))
                })
                .collect::<Vec<_>>();
            order.sort_unstable();
            assert_eq!(
                order.into_iter().map(|(_, name)| name).collect::<Vec<_>>(),
                expected_sort
            );
            for field in relation
                .columns
                .iter()
                .filter(|field| field.result_selection_role().is_some())
            {
                let bound = crate::arrow::field_for(registry, field).unwrap();
                assert_eq!(
                    bound.metadata()[KEY_RESULT_SELECTION],
                    field.result_selection_role().unwrap().as_str()
                );
            }
        }
        let fit = registry.relation("runtime.fit_parameters").unwrap();
        assert!(
            !fit.columns
                .iter()
                .any(|field| field.result_selection_role() == Some(R::Step))
        );
    }

    #[test]
    fn source_syntax_contracts_survive_native_binding_with_grammar_and_role() {
        let registry = crate::registry().unwrap();
        let declarations = registry.relation("authored.modeling_declarations").unwrap();
        assert_eq!(declarations.key.version, 26);
        let value = declarations.column("value").unwrap();
        let function = value
            .children()
            .into_iter()
            .find(|field| field.name() == "function")
            .unwrap();
        let body = function
            .children()
            .into_iter()
            .find(|field| field.name() == "body")
            .unwrap();
        assert_eq!(
            body.extension(),
            Some(crate::model::ExtensionUse::ExprDsl(
                crate::model::ExpressionGrammar::Expression
            ))
        );
        let bound = crate::arrow::field_for(registry, &body).unwrap();
        assert_eq!(bound.metadata()[KEY_EXPRESSION_ROLE], "function.body");
        assert_eq!(bound.metadata()[KEY_EXPRESSION_ORDINAL], "list");
        let metadata: serde_json::Value =
            serde_json::from_str(&bound.metadata()[KEY_EXTENSION_METADATA]).unwrap();
        assert_eq!(metadata["grammar"], "expression");
        assert_eq!(
            metadata["grammar_version"],
            crate::model::expression::GRAMMAR_VERSION
        );
        let annotation = value
            .children()
            .into_iter()
            .find(|field| field.name() == "annotation")
            .unwrap();
        let bound = crate::arrow::field_for(registry, &annotation).unwrap();
        let cases: Vec<crate::model::expression::AnnotationSyntax> =
            serde_json::from_str(&bound.metadata()[KEY_EXPRESSION_CASES]).unwrap();
        assert!(cases.iter().any(|case| case.kind == "check"
            && case.grammar.as_deref() == Some("predicate")
            && case.count == 1));
        assert!(
            cases.iter().any(|case| case.kind == "engineering_rule"
                && !case.target
                && case.grammar.is_none())
        );
    }

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
