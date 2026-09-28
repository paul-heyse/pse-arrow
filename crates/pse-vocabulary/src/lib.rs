// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The platform vocabularies the registry is written in (ADR-0117).
//!
//! These closed enumerations decide what a registry declaration may say — which namespace
//! a relation lives in, who may write it, what a column is for — so `pse-schema` builds its
//! model from them. The registry also declares each one as a `pse.enum` from its `ALL` and
//! `as_str`, and the generator re-exports the type into `pse-model` instead of emitting a
//! second enum (ADR-0115 Outcome 3). This crate sits beneath both, so neither dependency
//! ceiling is weakened: `pse-model` stays free of arrow, and `pse-codegen` stays free of
//! generated crates.
//!
//! Every vocabulary has one spelling per member. `as_str` is that spelling; `parse`,
//! `FromStr`, `Display` and serde all use it.

pub use pse_diagnostics::VocabularyError;

/// Declares a closed vocabulary with one authoritative spelling per member: the enum,
/// `ALL` in registry order, `as_str`, `parse`, `Display`, `FromStr` and serde, and behind
/// the `postgres` feature the value mapping to the store ENUM type named `$sql`.
macro_rules! vocabulary {
    (
        $(#[$meta:meta])*
        $name:ident as $sql:literal {
            $(
                $(#[$vmeta:meta])*
                $variant:ident => $text:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize,
            serde::Deserialize,
        )]
        #[cfg_attr(
            feature = "postgres",
            derive(postgres_types::ToSql, postgres_types::FromSql),
            postgres(name = $sql)
        )]
        pub enum $name {
            $(
                $(#[$vmeta])*
                #[serde(rename = $text)]
                #[cfg_attr(feature = "postgres", postgres(name = $text))]
                $variant,
            )+
        }

        impl $name {
            #[doc = concat!("Every [`", stringify!($name), "`], in registry order.")]
            pub const ALL: [Self; [$($text),+].len()] = [$(Self::$variant),+];

            /// The registry spelling, the only textual form of the member.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text,)+
                }
            }

            /// The member with this spelling; `None` for any other text, since a closed
            /// vocabulary has no fallback member.
            pub fn parse(text: &str) -> Option<Self> {
                match text {
                    $($text => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl core::str::FromStr for $name {
            type Err = VocabularyError;
            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Self::parse(text).ok_or_else(|| VocabularyError::UnknownMember {
                    vocabulary: stringify!($name),
                    value: text.to_owned(),
                })
            }
        }
    };
}

vocabulary! {
    /// The seven namespaces a relation can live in (blueprint §4.1, §6).
    Namespace as "namespace" {
        /// Shipped contracts and reference data; named-policy identity (blueprint §5.1).
        Reference => "reference",
        /// Facts a human or an agent authored. The only writable namespace (decision D2).
        Authored => "authored",
        /// P3's canonical rewriting of authored facts.
        Normalized => "normalized",
        /// Facts the rule engine inferred (P4–P6).
        Inferred => "inferred",
        /// Compiler output: symbols, mathematics, plans.
        Compiled => "compiled",
        /// Execution records and their results.
        Runtime => "runtime",
        /// Evidence: derivations, pass records, assertions.
        Provenance => "provenance",
    }
}

vocabulary! {
    /// Who is allowed to write a relation (blueprint §4.1, decision D2).
    Authority as "authority" {
        /// Written by the authoring path only, through a change set (blueprint §22.2).
        Authored => "authored",
        /// Shipped by a reference package.
        Reference => "reference",
        /// Produced by a pass. Requires a [`DerivationGranularity`].
        Derived => "derived",
    }
}

vocabulary! {
    /// Which snapshot a relation is a member of (blueprint §5.3 step 7).
    ///
    /// Membership is generated from this field and from nothing else: there is no namespace
    /// wildcard and no implicit omission, because an omission that reads as "not relevant"
    /// and an omission that reads as "forgotten" are indistinguishable in a hash.
    SnapshotClass as "snapshot_class" {
        /// A structural authored or reference contract.
        Model => "model",
        /// A §6.10 case, specification or observation relation.
        Case => "case",
        /// Compiler or runtime output.
        Derived => "derived",
        /// A revision catalog, change log, mutable ref, stage catalog or pass record. A
        /// sidecar never enters its own membership.
        Sidecar => "sidecar",
    }
}

vocabulary! {
    /// How much provenance a derived relation stores per head row (blueprint §14.2 rule 4).
    DerivationGranularity as "derivation_granularity" {
        /// One `provenance.derivations` row per head row: inference, law expansion and
        /// method resolution, where negative completeness is the deliverable.
        Row => "row",
        /// The derivation is the rule plus the §5.1 identity formula, reconstructed on
        /// demand: index expansion, discretization, connection equations.
        Rule => "rule",
    }
}

vocabulary! {
    /// How much a consumer may rely on a relation's shape (blueprint §4.1).
    Stability as "stability" {
        /// Public and versioned; a change needs a migration.
        Stable => "stable",
        /// Public but still moving; a change needs a migration and a note.
        Evolving => "evolving",
        /// Platform-internal; no external consumer may depend on it.
        Internal => "internal",
    }
}

vocabulary! {
    /// What a column is for (blueprint §4.1).
    ///
    /// The role is not decoration: [`ColumnRole::Key`] and [`ColumnRole::Reference`] are
    /// the roles a rule plan may key on (§14.2 rule 7), and [`ColumnRole::Measure`] is the
    /// role that carries a quantity contract.
    ColumnRole as "column_role" {
        /// Part of the primary key.
        Key => "key",
        /// A reference to another relation's key.
        Reference => "reference",
        /// A numerical value, normally under a quantity contract.
        Measure => "measure",
        /// A human-facing name or label. Never identity.
        Label => "label",
        /// Structured content that is neither key nor measure.
        Payload => "payload",
        /// Evidence: a derivation, a source span, a producing pass.
        Provenance => "provenance",
    }
}

vocabulary! {
    /// What kind of statement an invariant makes (blueprint §4.1).
    InvariantKind as "invariant_kind" {
        /// The named columns are unique.
        Unique => "unique",
        /// Every value resolves in the referenced relation.
        ForeignKey => "foreign_key",
        /// A row-local predicate holds.
        Check => "check",
        /// A group has a declared number of members.
        Cardinality => "cardinality",
        /// A value lies in a declared domain.
        Domain => "domain",
        /// A set is closed under a declared relation.
        Closure => "closure",
        /// A declared edge relation has no cycle.
        Acyclic => "acyclic",
    }
}

vocabulary! {
    /// Whether a finding stops a commit (blueprint §4.1, §22.2).
    Severity as "severity" {
        /// The commit contract is not met; the change set is rejected.
        Error => "error",
        /// Reported, and the commit proceeds.
        Warning => "warning",
    }
}

vocabulary! {
    /// How reproducible an algorithm is (blueprint §14.1).
    Determinism as "determinism" {
        /// The same inputs give the same outputs, byte for byte.
        Deterministic => "deterministic",
        /// Deterministic as the least fixed point of its rules (P4, P6).
        DeterministicFixedPoint => "deterministic_fixed_point",
        /// Deterministic once a backend is chosen; different backends differ (P16).
        DeterministicPerBackend => "deterministic_per_backend",
    }
}

vocabulary! {
    /// Effects of an invocation, independent of the native operator or function family
    /// (§5.4). It is the canonical effect vocabulary; it does not enumerate eligible engine
    /// features.
    OperationEffect as "operation_effect" {
        /// Read captured relation inputs.
        Read => "read",
        /// Read external state whose capture belongs to the invocation.
        Observe => "observe",
        /// Use time, randomness or another explicitly varying input.
        Nondeterministic => "nondeterministic",
        /// Construct or change private candidate data.
        Write => "write",
        /// Change an attempt's namespace/configuration generation.
        Namespace => "namespace",
        /// Write external artifacts or conditionally publish authoritative visibility.
        Publish => "publish",
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "tests report failures by panicking"
    )]

    use super::*;

    /// Every member's `as_str`, `parse`, `FromStr`, `Display` and serde agree, and the
    /// spellings are unique within their vocabulary.
    macro_rules! one_spelling {
        ($($name:ident),+) => {$({
            let mut seen = std::collections::BTreeSet::new();
            for member in $name::ALL {
                let text = member.as_str();
                assert!(seen.insert(text), "{} repeats {text}", stringify!($name));
                assert_eq!($name::parse(text), Some(member));
                assert_eq!(text.parse::<$name>().unwrap(), member);
                assert_eq!(member.to_string(), text);
                assert_eq!(serde_json::to_value(member).unwrap(), serde_json::json!(text));
                assert_eq!(
                    serde_json::from_value::<$name>(serde_json::json!(text)).unwrap(),
                    member
                );
            }
            assert_eq!($name::parse("no such member"), None);
            let refused = "no such member".parse::<$name>().unwrap_err();
            assert_eq!(
                refused,
                VocabularyError::UnknownMember {
                    vocabulary: stringify!($name),
                    value: "no such member".into(),
                }
            );
        })+};
    }

    #[test]
    fn vocabularies_have_one_spelling_per_member() {
        one_spelling!(
            Namespace,
            Authority,
            SnapshotClass,
            DerivationGranularity,
            Stability,
            ColumnRole,
            InvariantKind,
            Severity,
            Determinism,
            OperationEffect
        );
    }

    /// The spellings are the registry's stored values; changing one is a data migration.
    #[test]
    fn spellings_unchanged() {
        assert_eq!(
            Namespace::ALL.map(Namespace::as_str),
            [
                "reference",
                "authored",
                "normalized",
                "inferred",
                "compiled",
                "runtime",
                "provenance"
            ]
        );
        assert_eq!(
            InvariantKind::ALL.map(InvariantKind::as_str),
            [
                "unique",
                "foreign_key",
                "check",
                "cardinality",
                "domain",
                "closure",
                "acyclic"
            ]
        );
        assert_eq!(
            Determinism::ALL.map(Determinism::as_str),
            [
                "deterministic",
                "deterministic_fixed_point",
                "deterministic_per_backend"
            ]
        );
        assert_eq!(
            OperationEffect::ALL.map(OperationEffect::as_str),
            [
                "read",
                "observe",
                "nondeterministic",
                "write",
                "namespace",
                "publish"
            ]
        );
    }
}
