// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! What the catalog knows about one relation: its canonical contract plus the column
//! roles the engine needs (blueprint §5.4, §20.1).
//!
//! [`pse_ids::CanonicalContract`] answers "what may be hashed, and how?". A
//! [`RelationContract`] answers the engine's questions on top of it: which columns are the
//! primary key (so a `TableProvider` can advertise `Constraints`), which are enums (so
//! pushdown knows the registered predicate subset), which sets are unique (so `Unique`
//! constraints are only claimed where P2 validates one), and which encodings this
//! relation is published in.
//!
//! The split is deliberate. `pse-ids` sits below `pse-schema` and must not know what a
//! DataFusion `Constraints` is; this crate does, and translates. Nothing here infers: a
//! key is a key because the registry said so, and this type only resolves the names the
//! registry declared into ordinals and refuses the ones that cannot mean what they claim.
//! Primary keys have exactly one declaration, in the canonical contract.

mod registry;

use std::sync::Arc;

use datafusion::arrow::datatypes::Schema;
use datafusion::common::{Constraint, Constraints, DFSchema, TableReference};
use pse_ids::{CanonError, CanonicalContract, FieldPath, Layout};

use crate::error::CatalogError;
use crate::failure::{PlanOrigin, classify};

/// Which physical encodings a relation is published in (blueprint §20.1).
///
/// Arrow IPC files are the hot path; Parquet is the durable form for authored, reference
/// and runtime history. Both carry the same `logical_hash` (ADR-0045), so this is a
/// storage policy and never an identity decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EncodingPolicy {
    /// One Arrow IPC file per relation.
    IpcFile,
    /// An Arrow IPC file and a Parquet file of the same logical content.
    IpcFileAndParquet,
}

/// One relation as the catalog serves it (blueprint §5.4).
#[derive(Clone, Debug)]
pub struct RelationContract {
    /// What `pse.canon.v2` may consume, including the declared schema.
    pub canonical: CanonicalContract,
    /// The relation's namespace: one of the seven schemas of §5.4.
    pub namespace: String,
    /// The relation's declared name.
    pub name: String,
    /// Ordinals of the dictionary-encoded enum columns, ascending.
    pub enum_columns: Vec<usize>,
    /// Column ordinals of each validated `unique` invariant, in declaration order.
    pub unique_sets: Vec<Vec<usize>>,
    /// Which encodings this relation is published in.
    pub encodings: EncodingPolicy,
}

impl RelationContract {
    /// Resolves column names against the declared schema, or says which one cannot mean
    /// what it claims.
    ///
    /// # Errors
    ///
    /// - [`CatalogError::Canon`] wrapping [`CanonError::InvalidKey`] for a key column that
    ///   is unknown, repeated or floating point. A float key is refused by ADR-0030
    ///   because DataFusion's grouping collapses signed zero and treats NaN as
    ///   self-equal, so a float key merges rows the reader considers distinct.
    /// - [`CatalogError::Canon`] wrapping [`CanonError::ContractMismatch`] for an unknown
    ///   enum or unique column, for an enum column whose declared layout is not an enum,
    ///   and for an empty unique set.
    pub fn try_new(
        canonical: CanonicalContract,
        namespace: &str,
        name: &str,
        enum_columns: &[&str],
        unique_sets: &[&[&str]],
        encodings: EncodingPolicy,
    ) -> Result<Self, CatalogError> {
        validate_primary_key(&canonical)?;

        let mut enums: Vec<usize> = Vec::with_capacity(enum_columns.len());
        for column in enum_columns {
            let ordinal = ordinal_of(&canonical, column)?;
            if !matches!(canonical.layouts.get(ordinal), Some(Layout::Enum { .. })) {
                return Err(mismatch(
                    ordinal,
                    "a dictionary-encoded enum column with a declared domain",
                    &describe_layout(canonical.layouts.get(ordinal)),
                ));
            }
            if !enums.contains(&ordinal) {
                enums.push(ordinal);
            }
        }
        enums.sort_unstable();

        let mut uniques: Vec<Vec<usize>> = Vec::with_capacity(unique_sets.len());
        for set in unique_sets {
            if set.is_empty() {
                return Err(mismatch(
                    0,
                    "a unique set of at least one column",
                    "an empty unique set",
                ));
            }
            let mut resolved: Vec<usize> = Vec::with_capacity(set.len());
            for column in *set {
                let ordinal = ordinal_of(&canonical, column)?;
                if !resolved.contains(&ordinal) {
                    resolved.push(ordinal);
                }
            }
            uniques.push(resolved);
        }

        Ok(Self {
            canonical,
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            enum_columns: enums,
            unique_sets: uniques,
            encodings,
        })
    }

    /// The constraints a `TableProvider` advertises (blueprint §5.4).
    ///
    /// `new_unverified` because DataFusion does not check these and does not claim to —
    /// P2 does (§14.2). Advertising a `Unique` that P2 has not validated would let the
    /// optimizer eliminate a distinct the data still needs.
    #[must_use]
    pub fn constraints(&self) -> Constraints {
        let mut declared: Vec<Constraint> = Vec::with_capacity(1 + self.unique_sets.len());
        declared.push(Constraint::PrimaryKey(self.canonical.primary_key.clone()));
        for set in &self.unique_sets {
            declared.push(Constraint::Unique(set.clone()));
        }
        Constraints::new_unverified(declared)
    }

    /// The declared schema as a qualified [`DFSchema`], for plan construction.
    ///
    /// # Errors
    ///
    /// [`CatalogError::Internal`]: a registry schema that cannot become a `DFSchema` has
    /// duplicate field names, which is a registry bug rather than a caller's input. The
    /// engine's own failure is mapped through [`classify`] so that even this path has one
    /// §23.2 mapping site.
    pub fn df_schema(&self) -> Result<DFSchema, CatalogError> {
        let reference = TableReference::partial(self.namespace.clone(), self.name.clone());
        DFSchema::try_from_qualified_schema(reference, self.canonical.schema.as_ref()).map_err(
            |error| {
                classify(error, PlanOrigin::RuleCompiler)
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| CatalogError::Internal {
                        message: "the engine refused a registry schema without saying why"
                            .to_owned(),
                    })
            },
        )
    }

    /// The key columns as declared names, in key order.
    #[must_use]
    pub fn key_column_names(&self) -> Vec<&str> {
        self.canonical.key_column_names()
    }

    /// The relation as `namespace.name`, which is how a message names it.
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.namespace, self.name)
    }
}

/// The ordinal of `column`, or a contract mismatch naming it.
fn ordinal_of(canonical: &CanonicalContract, column: &str) -> Result<usize, CatalogError> {
    canonical
        .schema
        .fields()
        .iter()
        .position(|field| field.name() == column)
        .ok_or_else(|| {
            CatalogError::Canon(CanonError::ContractMismatch {
                path: FieldPath::root(),
                expected: format!("a column named `{column}`"),
                actual: "no such column in the declared schema".to_owned(),
            })
        })
}

/// Check key declarations even when a caller constructed a canonical contract directly.
fn validate_primary_key(canonical: &CanonicalContract) -> Result<(), CatalogError> {
    if canonical.primary_key.is_empty() {
        return Err(invalid_key("", "a relation requires a primary key"));
    }
    let mut seen = Vec::with_capacity(canonical.primary_key.len());
    for &ordinal in &canonical.primary_key {
        let Some(field) = canonical.schema.fields().get(ordinal) else {
            return Err(invalid_key(
                &ordinal.to_string(),
                "no such column in the declared schema",
            ));
        };
        if seen.contains(&ordinal) {
            return Err(invalid_key(field.name(), "named twice in the primary key"));
        }
        if field.is_nullable() {
            return Err(invalid_key(
                field.name(),
                "a primary-key column cannot be nullable",
            ));
        }
        if canonical
            .layouts
            .get(ordinal)
            .is_some_and(Layout::is_floating)
        {
            return Err(invalid_key(
                field.name(),
                "a floating-point column is never a key (ADR-0030)",
            ));
        }
        seen.push(ordinal);
    }
    Ok(())
}

/// A key column that cannot order a relation.
fn invalid_key(column: &str, reason: &str) -> CatalogError {
    CatalogError::Canon(CanonError::InvalidKey {
        column: column.to_owned(),
        reason: reason.to_owned(),
    })
}

/// A column whose declared layout is not what the contract asked for.
fn mismatch(ordinal: usize, expected: &str, actual: &str) -> CatalogError {
    CatalogError::Canon(CanonError::ContractMismatch {
        path: FieldPath::root().child(ordinal),
        expected: expected.to_owned(),
        actual: actual.to_owned(),
    })
}

/// A terse description of a declared layout, for a message.
fn describe_layout(layout: Option<&Layout>) -> String {
    match layout {
        Some(Layout::Enum { .. }) => "an enum column".to_owned(),
        Some(other) => format!("{other:?}"),
        None => "no declared layout".to_owned(),
    }
}

/// Asserts at compile time that the contract's schema is the same Arrow type the engine
/// speaks. Two resolved `arrow` majors would make this stop compiling, which is a much
/// better failure than the runtime one they otherwise cause: `downcast_ref` returning
/// `None` with no compile error at all (blueprint §3.1).
const _: fn(&CanonicalContract) -> &Arc<Schema> = |contract| &contract.schema;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    use pse_ids::{ContentHash, SchemaVersion, SemanticId};

    use super::*;

    /// A schema with a text key, an integer, a float and a dictionary-encoded enum.
    fn contract() -> CanonicalContract {
        let schema = Arc::new(Schema::new(vec![
            Field::new("unit_id", DataType::Utf8, false),
            Field::new("stage", DataType::Int64, false),
            Field::new("duty", DataType::Float64, true),
            Field::new(
                "phase",
                DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
                false,
            ),
        ]));
        let mut domains: BTreeMap<FieldPath, Arc<[String]>> = BTreeMap::new();
        domains.insert(
            FieldPath::root().child(3),
            Arc::from(vec!["Liq".to_owned(), "Vap".to_owned()]),
        );
        let built = CanonicalContract::try_new(
            SemanticId::from_bytes([0x01; 16]),
            SchemaVersion(1),
            ContentHash::from_bytes([0x11; 32]),
            schema,
            &["unit_id", "stage"],
            &domains,
        );
        let Ok(built) = built else {
            panic!("the fixture schema is canonicalizable");
        };
        built
    }

    fn relation() -> RelationContract {
        let built = RelationContract::try_new(
            contract(),
            "authored",
            "unit_stages",
            &["phase"],
            &[&["unit_id"][..], &["stage", "phase"][..]],
            EncodingPolicy::IpcFileAndParquet,
        );
        let Ok(built) = built else {
            panic!("the fixture contract is admissible");
        };
        built
    }

    #[test]
    fn columns_resolve_to_declared_ordinals() {
        let contract = relation();
        assert_eq!(contract.canonical.primary_key, vec![0, 1]);
        assert_eq!(contract.enum_columns, vec![3]);
        assert_eq!(contract.unique_sets, vec![vec![0], vec![1, 3]]);
        assert_eq!(contract.key_column_names(), vec!["unit_id", "stage"]);
        assert_eq!(contract.qualified_name(), "authored.unit_stages");
        assert_eq!(contract.encodings, EncodingPolicy::IpcFileAndParquet);
    }

    #[test]
    fn an_unknown_key_column_is_refused() {
        let mut canonical = contract();
        canonical.primary_key = vec![99];
        let refused = RelationContract::try_new(
            canonical,
            "authored",
            "unit_stages",
            &[],
            &[],
            EncodingPolicy::IpcFile,
        );
        assert!(matches!(
            refused,
            Err(CatalogError::Canon(CanonError::InvalidKey { ref reason, .. }))
                if reason.contains("no such column")
        ));
    }

    #[test]
    fn a_nullable_key_is_refused() {
        let mut canonical = contract();
        canonical.primary_key = vec![2];
        let refused = RelationContract::try_new(
            canonical,
            "authored",
            "unit_stages",
            &[],
            &[],
            EncodingPolicy::IpcFile,
        );
        assert!(matches!(
            refused,
            Err(CatalogError::Canon(CanonError::InvalidKey { ref column, ref reason }))
                if column == "duty" && reason.contains("nullable")
        ));
    }

    #[test]
    fn a_repeated_key_column_is_refused() {
        let mut canonical = contract();
        canonical.primary_key = vec![0, 0];
        let refused = RelationContract::try_new(
            canonical,
            "authored",
            "unit_stages",
            &[],
            &[],
            EncodingPolicy::IpcFile,
        );
        assert!(matches!(
            refused,
            Err(CatalogError::Canon(CanonError::InvalidKey { ref reason, .. }))
                if reason.contains("named twice")
        ));
    }

    #[test]
    fn an_empty_primary_key_is_refused() {
        let mut canonical = contract();
        canonical.primary_key.clear();
        assert!(matches!(
            RelationContract::try_new(
                canonical,
                "authored",
                "unit_stages",
                &[],
                &[],
                EncodingPolicy::IpcFile
            ),
            Err(CatalogError::Canon(CanonError::InvalidKey { .. }))
        ));
    }

    #[test]
    fn a_non_enum_column_cannot_be_declared_an_enum() {
        let refused = RelationContract::try_new(
            contract(),
            "authored",
            "unit_stages",
            &["stage"],
            &[],
            EncodingPolicy::IpcFile,
        );
        assert!(matches!(
            refused,
            Err(CatalogError::Canon(CanonError::ContractMismatch { .. }))
        ));
    }

    #[test]
    fn an_unknown_unique_column_is_refused() {
        let refused = RelationContract::try_new(
            contract(),
            "authored",
            "unit_stages",
            &[],
            &[&["no_such_column"][..]],
            EncodingPolicy::IpcFile,
        );
        assert!(matches!(
            refused,
            Err(CatalogError::Canon(CanonError::ContractMismatch { .. }))
        ));
    }

    #[test]
    fn an_empty_unique_set_is_refused() {
        let refused = RelationContract::try_new(
            contract(),
            "authored",
            "unit_stages",
            &[],
            &[&[][..]],
            EncodingPolicy::IpcFile,
        );
        assert!(matches!(
            refused,
            Err(CatalogError::Canon(CanonError::ContractMismatch { .. }))
        ));
    }

    #[test]
    fn constraints_carry_the_primary_key_first_and_then_every_unique_set() {
        let contract = relation();
        let constraints = contract.constraints();
        let declared: Vec<&Constraint> = constraints.iter().collect();
        assert_eq!(declared.len(), 3);
        assert!(
            matches!(declared.first(), Some(Constraint::PrimaryKey(key)) if key == &vec![0, 1])
        );
        assert!(matches!(declared.get(1), Some(Constraint::Unique(set)) if set == &vec![0]));
        assert!(matches!(declared.get(2), Some(Constraint::Unique(set)) if set == &vec![1, 3]));
    }

    #[test]
    fn a_relation_without_unique_invariants_advertises_only_its_key() {
        let built = RelationContract::try_new(
            contract(),
            "authored",
            "unit_stages",
            &[],
            &[],
            EncodingPolicy::IpcFile,
        );
        let Ok(built) = built else {
            panic!("the contract is admissible");
        };
        assert_eq!(built.constraints().iter().count(), 1);
    }

    #[test]
    fn the_qualified_df_schema_carries_every_declared_column() {
        let contract = relation();
        let Ok(schema) = contract.df_schema() else {
            panic!("the fixture schema is a DFSchema");
        };
        assert_eq!(schema.fields().len(), 4);
        assert_eq!(
            schema.field_names(),
            vec![
                "authored.unit_stages.unit_id",
                "authored.unit_stages.stage",
                "authored.unit_stages.duty",
                "authored.unit_stages.phase",
            ]
        );
    }
}
