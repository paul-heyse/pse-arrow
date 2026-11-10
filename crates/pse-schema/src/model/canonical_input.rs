// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Operation input policy derived from the existing declared field contract.

use super::FieldContract;

/// Creation supply policy; an initial-state constraint is not a default expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalCreatePresence {
    /// The caller must supply the declared value.
    Required,
    /// Omission denotes native absence.
    Optional,
    /// Omission invokes the declared database default.
    Defaulted,
}

/// Input obligations of one field occurrence, without execution authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalInputPolicy {
    /// Creation supply, independent of later immutability.
    pub create: CanonicalCreatePresence,
    /// Whether this occurrence may be changed after creation.
    pub mutable: bool,
    /// Whether an admitted mutation may remove this field.
    pub removable: bool,
}

impl CanonicalInputPolicy {
    /// Derive from checked source facets, never from field spelling or current values.
    /// # Errors
    /// Malformed or incoherent canonical policy refuses.
    pub fn for_field(field: &FieldContract) -> Result<Self, crate::SchemaError> {
        let policy = field.canonical_policy()?;
        let mutable = !policy
            .as_ref()
            .and_then(|p| p.contract)
            .is_some_and(|p| p.readonly == Some(true));
        let create = if policy.as_ref().is_some_and(|p| p.default.is_some()) {
            CanonicalCreatePresence::Defaulted
        } else if field.nullable() {
            CanonicalCreatePresence::Optional
        } else {
            CanonicalCreatePresence::Required
        };
        Ok(Self {
            create,
            mutable,
            removable: mutable && field.nullable(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_readonly_and_default_have_distinct_creation_meaning() {
        let initial = FieldContract::enumeration("State")
            .with_canonical_initial("new")
            .canonical_readonly();
        let policy = CanonicalInputPolicy::for_field(&initial).unwrap();
        assert_eq!(policy.create, CanonicalCreatePresence::Required);
        assert!(!policy.mutable);
        let defaulted = initial.with_canonical_default("'new'");
        assert_eq!(
            CanonicalInputPolicy::for_field(&defaulted).unwrap().create,
            CanonicalCreatePresence::Defaulted
        );
    }

    #[test]
    fn optional_mutation_removal_is_independent_of_create_supply() {
        let field = FieldContract::native(arrow_schema::DataType::Utf8).optional();
        let policy = CanonicalInputPolicy::for_field(&field).unwrap();
        assert_eq!(policy.create, CanonicalCreatePresence::Optional);
        assert!(policy.mutable && policy.removable);
        let immutable = CanonicalInputPolicy::for_field(&field.canonical_readonly()).unwrap();
        assert_eq!(immutable.create, CanonicalCreatePresence::Optional);
        assert!(!immutable.mutable && !immutable.removable);
    }
}
