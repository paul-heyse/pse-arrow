// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The one place a [`DataFusionError`] becomes a [`CatalogError`] (blueprint §23.2).
//!
//! §23.2 says the mapping happens "at one place, never per call site", and the reason is
//! concrete: twenty engine variants mapped by convention at each call site converge on
//! `internal::invariant`, because that is the arm a hurried author writes. The same
//! `Execution` string is a user's fault in an analytics query, a platform bug in a rule
//! plan, and an evaluation failure inside a generated kernel — [`PlanOrigin`] is what
//! tells the three apart, and it is an argument precisely so that a call site must state
//! which one it is.
//!
//! [`classify`] returns a `Vec` rather than one error because `DataFusionError::Collection`
//! and `Diagnostic` exist to carry more than one failure. §23.2's rule — "validators
//! return violating keys plural; the first error is never the only one reported" — is
//! only true if the unpacking is recursive, so a `Collection` of `Context`s of a
//! `Collection` yields every leaf.

use std::sync::Arc;

use datafusion::error::DataFusionError;

use crate::error::CatalogError;

/// Where the plan that failed came from (blueprint §23.2's `DataFusionError` table).
///
/// The same engine failure has a different platform class depending on who wrote the
/// plan, and nothing in the error itself records that. A caller that cannot say which of
/// these it is has not decided whose fault a failure would be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlanOrigin {
    /// A plan the rule compiler built from a registry declaration (blueprint §14.2).
    ///
    /// A rule plan that does not type is a platform bug, never a user error.
    RuleCompiler,
    /// A query a user wrote against `runtime`/`compiled` relations (blueprint §19.2).
    Analytics,
    /// Execution inside a generated kernel adapter (blueprint §9).
    KernelUdf,
}

impl PlanOrigin {
    /// The stable lowercase spelling used in messages.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RuleCompiler => "rule_compiler",
            Self::Analytics => "analytics",
            Self::KernelUdf => "kernel_udf",
        }
    }
}

impl std::fmt::Display for PlanOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The `op` a failure from Arrow carries.
const OP_ARROW: &str = "arrow";
/// The `op` a failure from Parquet carries.
const OP_PARQUET: &str = "parquet";
/// The `op` a failure from the object store carries.
const OP_OBJECT_STORE: &str = "object store";
/// The `op` a filesystem failure carries.
const OP_IO: &str = "io";
/// The `op` a query-execution failure carries when it is nobody's declared fault.
const OP_EXECUTION: &str = "query execution";

/// The consumer named when a `ResourcesExhausted` message does not identify one.
const UNATTRIBUTED: &str = "unattributed";

/// What a kernel writes into an `Execution` message to claim the `compile.property` class.
const COMPILE_PROPERTY_MARKER: &str = "compile.property";

/// The prefix of a DataFusion configuration key, as its messages spell it.
const CONFIG_KEY_PREFIX: &str = "datafusion.";

/// What a `config::invalid` names when the engine's message named no key at all: the
/// namespace, which is the most that can be said without inventing a setting.
const UNNAMED_CONFIG_KEY: &str = "datafusion";

/// Maps one engine failure into one or more platform failures (blueprint §23.2).
///
/// `Collection`, `Diagnostic`, `Context` and `Shared` are containers, so they are
/// unpacked recursively and contribute one entry per contained leaf. Every other variant
/// contributes exactly one entry. The result is empty only for an empty `Collection`,
/// which is an engine failure that reports nothing.
///
/// `External` carrying a [`CatalogError`] is passed through unchanged: the platform put
/// it there on the way *into* the engine (a `TableProvider` or a UDF returning a typed
/// failure), and re-classifying its own error would discard the class it already chose.
///
/// Shared failures keep their platform class and diagnostic fields regardless of the
/// number of owners. When the error cannot be moved, non-cloneable infrastructure
/// sources are retained as rendered diagnostics; ownership never changes the class.
///
/// ```
/// use datafusion::error::DataFusionError;
/// use pse_catalog::{CatalogError, PlanOrigin, classify};
///
/// let found = classify(
///     DataFusionError::Plan("no such column `x`".to_owned()),
///     PlanOrigin::Analytics,
/// );
/// assert!(matches!(found.as_slice(), [CatalogError::UserModel { .. }]));
/// ```
#[must_use]
pub fn classify(error: DataFusionError, origin: PlanOrigin) -> Vec<CatalogError> {
    match error {
        DataFusionError::Collection(errors) => errors
            .into_iter()
            .flat_map(|inner| classify(inner, origin))
            .collect(),
        DataFusionError::Diagnostic(_, inner) | DataFusionError::Context(_, inner) => {
            classify(*inner, origin)
        }
        DataFusionError::Shared(shared) => match Arc::try_unwrap(shared) {
            Ok(inner) => classify(inner, origin),
            Err(still_shared) => classify_borrowed(&still_shared, origin),
        },
        DataFusionError::External(source) => vec![match source.downcast::<CatalogError>() {
            Ok(platform) => *platform,
            Err(foreign) => CatalogError::Internal {
                message: foreign.to_string(),
            },
        }],
        DataFusionError::ArrowError(inner, _) => vec![CatalogError::Infrastructure {
            op: OP_ARROW.to_owned(),
            source: inner,
        }],
        DataFusionError::ParquetError(inner) => vec![CatalogError::Infrastructure {
            op: OP_PARQUET.to_owned(),
            source: inner,
        }],
        DataFusionError::ObjectStore(inner) => vec![CatalogError::Infrastructure {
            op: OP_OBJECT_STORE.to_owned(),
            source: inner,
        }],
        DataFusionError::IoError(inner) => vec![CatalogError::Infrastructure {
            op: OP_IO.to_owned(),
            source: Box::new(inner),
        }],
        leaf => classify_borrowed(&leaf, origin),
    }
}

/// The full mapping, decided without taking ownership.
///
/// This is where the policy lives; [`classify`] only adds the arms that ownership makes
/// better (a real `#[source]` chain, and the `External` pass-through).
fn classify_borrowed(error: &DataFusionError, origin: PlanOrigin) -> Vec<CatalogError> {
    match error {
        DataFusionError::Collection(errors) => errors
            .iter()
            .flat_map(|inner| classify_borrowed(inner, origin))
            .collect(),
        DataFusionError::Diagnostic(_, inner) | DataFusionError::Context(_, inner) => {
            classify_borrowed(inner, origin)
        }
        DataFusionError::Shared(shared) => classify_borrowed(shared, origin),
        DataFusionError::External(source) => {
            vec![source.downcast_ref::<CatalogError>().map_or_else(
                || CatalogError::Internal {
                    message: source.to_string(),
                },
                copy_platform_diagnostic,
            )]
        }
        DataFusionError::ResourcesExhausted(message) => vec![resource_limit(message)],
        DataFusionError::Configuration(message) => vec![CatalogError::ConfigInvalid {
            key: config_keys_in(message)
                .first()
                .cloned()
                .unwrap_or_else(|| UNNAMED_CONFIG_KEY.to_owned()),
            reason: message.clone(),
        }],
        DataFusionError::SchemaError(inner, _) => vec![plan_failure(origin, inner.to_string())],
        DataFusionError::Plan(message) => vec![plan_failure(origin, message.clone())],
        DataFusionError::Execution(message) => vec![execution_failure(origin, message.clone())],
        DataFusionError::ArrowError(..) => vec![stringified(OP_ARROW, error)],
        DataFusionError::ParquetError(_) => vec![stringified(OP_PARQUET, error)],
        DataFusionError::ObjectStore(_) => vec![stringified(OP_OBJECT_STORE, error)],
        DataFusionError::IoError(_) => vec![stringified(OP_IO, error)],
        other => vec![CatalogError::Internal {
            message: other.to_string(),
        }],
    }
}

/// Preserve the typed platform result when a shared engine wrapper cannot be consumed.
/// The only non-cloneable payloads are infrastructure source errors; their rendering is
/// retained while the platform variant and all its context remain unchanged.
fn copy_platform_diagnostic(error: &CatalogError) -> CatalogError {
    match error {
        CatalogError::Semantic(source) => CatalogError::Semantic(Arc::clone(source)),
        CatalogError::Multiple { errors } => CatalogError::Multiple {
            errors: errors.iter().map(copy_platform_diagnostic).collect(),
        },
        CatalogError::ResourceLimit {
            consumer,
            config_keys,
            detail,
        } => CatalogError::ResourceLimit {
            consumer: consumer.clone(),
            config_keys: config_keys.clone(),
            detail: detail.clone(),
        },
        CatalogError::Cancelled => CatalogError::Cancelled,
        CatalogError::Infrastructure { op, source } => CatalogError::Infrastructure {
            op: op.clone(),
            source: source.to_string().into(),
        },
        CatalogError::CorruptObject {
            path,
            expected,
            actual,
        } => CatalogError::CorruptObject {
            path: path.clone(),
            expected: expected.clone(),
            actual: actual.clone(),
        },
        CatalogError::RefConflict { name } => CatalogError::RefConflict { name: name.clone() },
        CatalogError::ManifestInvalid { reason } => CatalogError::ManifestInvalid {
            reason: reason.clone(),
        },
        CatalogError::UnknownRegistry { fingerprint } => CatalogError::UnknownRegistry {
            fingerprint: *fingerprint,
        },
        CatalogError::UnknownVersion { field, value } => CatalogError::UnknownVersion {
            field: field.clone(),
            value: value.clone(),
        },
        CatalogError::Admission { path, reason } => CatalogError::Admission {
            path: path.clone(),
            reason: reason.clone(),
        },
        CatalogError::ForeignSource { table } => CatalogError::ForeignSource {
            table: table.clone(),
        },
        CatalogError::Sealed => CatalogError::Sealed,
        CatalogError::LogicalHashMismatch {
            relation,
            expected,
            actual,
        } => CatalogError::LogicalHashMismatch {
            relation: relation.clone(),
            expected: *expected,
            actual: *actual,
        },
        CatalogError::Membership { reason } => CatalogError::Membership {
            reason: reason.clone(),
        },
        CatalogError::ConfigInvalid { key, reason } => CatalogError::ConfigInvalid {
            key: key.clone(),
            reason: reason.clone(),
        },
        CatalogError::UserModel { message } => CatalogError::UserModel {
            message: message.clone(),
        },
        CatalogError::EvaluationError { message } => CatalogError::EvaluationError {
            message: message.clone(),
        },
        CatalogError::CompileProperty { message } => CatalogError::CompileProperty {
            message: message.clone(),
        },
        CatalogError::Canon(error) => CatalogError::Canon(error.copy_for_reporting()),
        CatalogError::Snapshot(error) => CatalogError::Snapshot(error.clone()),
        CatalogError::Reserve(error) => CatalogError::Reserve(error.clone()),
        CatalogError::Internal { message } => CatalogError::Internal {
            message: message.clone(),
        },
    }
}

/// Collapse classified engine leaves into one return value without discarding any
/// diagnostic. One leaf stays transparent; several (or an empty engine collection) are
/// a related diagnostic group without an invented replacement class.
pub fn collapse_classified(mut errors: Vec<CatalogError>) -> CatalogError {
    if errors.len() == 1 {
        errors.remove(0)
    } else {
        CatalogError::Multiple { errors }
    }
}

/// An infrastructure failure whose source could only be rendered, not moved.
fn stringified(op: &str, error: &DataFusionError) -> CatalogError {
    CatalogError::Infrastructure {
        op: op.to_owned(),
        source: error.to_string().into(),
    }
}

/// `ResourcesExhausted` is the one engine failure that is expected, recoverable and
/// configuration-driven, so it keeps the consumer and the keys its message named.
fn resource_limit(message: &str) -> CatalogError {
    CatalogError::ResourceLimit {
        consumer: consumer_in(message).unwrap_or(UNATTRIBUTED).to_owned(),
        config_keys: config_keys_in(message),
        detail: message.to_owned(),
    }
}

/// A schema or plan failure, which is a platform bug or a user error depending on who
/// wrote the plan (blueprint §23.2).
fn plan_failure(origin: PlanOrigin, message: String) -> CatalogError {
    match origin {
        PlanOrigin::Analytics => CatalogError::UserModel { message },
        PlanOrigin::RuleCompiler | PlanOrigin::KernelUdf => CatalogError::Internal { message },
    }
}

/// An execution failure: the kernel's declared failure class inside a generated UDF,
/// infrastructure anywhere else.
fn execution_failure(origin: PlanOrigin, message: String) -> CatalogError {
    match origin {
        PlanOrigin::KernelUdf if message.contains(COMPILE_PROPERTY_MARKER) => {
            CatalogError::CompileProperty { message }
        }
        PlanOrigin::KernelUdf => CatalogError::EvaluationError { message },
        PlanOrigin::RuleCompiler | PlanOrigin::Analytics => CatalogError::Infrastructure {
            op: OP_EXECUTION.to_owned(),
            source: message.into(),
        },
    }
}

/// The consumer a memory-pool message names, for the two shapes DataFusion 55.1 writes.
///
/// `Failed to allocate additional 1.0 KB for ExternalSorter[0] with 0.0 B already …` and
/// `Additional allocation failed for ExternalSorter[0] with top memory consumers …` both
/// put the consumer between `" for "` and `" with "`; a message with neither (the sort
/// preflight message, for one) names no consumer and is not given an invented one.
fn consumer_in(message: &str) -> Option<&str> {
    let after_for = message.find(" for ")? + " for ".len();
    let tail = message.get(after_for..)?;
    let before_with = tail.find(" with ")?;
    let consumer = tail.get(..before_with)?.trim();
    (!consumer.is_empty()).then_some(consumer)
}

/// Every `datafusion.*` key the message names, in order of first appearance.
///
/// Two message shapes have to be read, because DataFusion writes both: the memory-pool
/// messages quote their keys (`'datafusion.runtime.memory_limit'`) and the configuration
/// messages do not (`Error setting config datafusion.explain.format`). Scanning for
/// maximal key-shaped runs reads both without a rule per message.
///
/// Order of appearance rather than sorted, because the engine writes the key to raise
/// before the key to lower, and that ordering is advice.
fn config_keys_in(message: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for token in message.split(|character: char| !is_key_character(character)) {
        let candidate = token.trim_end_matches('.');
        if candidate.starts_with(CONFIG_KEY_PREFIX)
            && candidate.len() > CONFIG_KEY_PREFIX.len()
            && !keys.iter().any(|seen| seen == candidate)
        {
            keys.push(candidate.to_owned());
        }
    }
    keys
}

/// Whether `character` may appear in a DataFusion configuration key.
fn is_key_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '.' || character == '_'
}

#[cfg(test)]
mod tests {
    use std::io;

    use datafusion::arrow::error::ArrowError;
    use datafusion::common::SchemaError;
    use datafusion::parquet::errors::ParquetError;

    use super::*;

    /// The PROBE H message from the DataFusion capability map (§D7), verbatim.
    const SPILL_MESSAGE: &str = "Not enough memory to continue external sort. Consider \
         increasing the memory limit config: 'datafusion.runtime.memory_limit', or \
         decreasing the config: 'datafusion.execution.sort_spill_reservation_bytes'";

    /// The shape `TrackConsumersPool` writes when a grow is refused.
    const TRACKED_MESSAGE: &str = "Additional allocation failed for ExternalSorter[0] with top \
         memory consumers (across reservations) as:\n  ExternalSorter[0] consumed 1.0 MB";

    fn one(error: DataFusionError, origin: PlanOrigin) -> CatalogError {
        let mut found = classify(error, origin);
        assert_eq!(found.len(), 1, "expected exactly one finding");
        found.remove(0)
    }

    #[test]
    fn resources_exhausted_names_the_consumer_and_the_configuration() {
        let found = one(
            DataFusionError::ResourcesExhausted(TRACKED_MESSAGE.to_owned()),
            PlanOrigin::Analytics,
        );
        let CatalogError::ResourceLimit {
            consumer,
            config_keys,
            detail,
        } = found
        else {
            panic!("expected a resource limit, found {found}");
        };
        assert_eq!(consumer, "ExternalSorter[0]");
        assert!(config_keys.is_empty());
        assert_eq!(detail, TRACKED_MESSAGE);
    }

    #[test]
    fn a_spill_message_yields_its_two_configuration_keys() {
        let found = one(
            DataFusionError::ResourcesExhausted(SPILL_MESSAGE.to_owned()),
            PlanOrigin::RuleCompiler,
        );
        let CatalogError::ResourceLimit {
            consumer,
            config_keys,
            ..
        } = found
        else {
            panic!("expected a resource limit, found {found}");
        };
        assert_eq!(consumer, UNATTRIBUTED);
        assert_eq!(
            config_keys,
            vec![
                "datafusion.runtime.memory_limit".to_owned(),
                "datafusion.execution.sort_spill_reservation_bytes".to_owned(),
            ]
        );
    }

    #[test]
    fn a_plan_failure_is_a_platform_bug_or_a_user_error_by_origin() {
        assert!(matches!(
            one(
                DataFusionError::Plan("x".to_owned()),
                PlanOrigin::RuleCompiler
            ),
            CatalogError::Internal { .. }
        ));
        assert!(matches!(
            one(DataFusionError::Plan("x".to_owned()), PlanOrigin::Analytics),
            CatalogError::UserModel { .. }
        ));
        assert!(matches!(
            one(DataFusionError::Plan("x".to_owned()), PlanOrigin::KernelUdf),
            CatalogError::Internal { .. }
        ));
    }

    #[test]
    fn a_schema_failure_follows_the_same_split() {
        let schema_error = || {
            DataFusionError::SchemaError(
                Box::new(SchemaError::DuplicateUnqualifiedField {
                    name: "x".to_owned(),
                }),
                Box::new(None),
            )
        };
        assert!(matches!(
            one(schema_error(), PlanOrigin::RuleCompiler),
            CatalogError::Internal { .. }
        ));
        assert!(matches!(
            one(schema_error(), PlanOrigin::Analytics),
            CatalogError::UserModel { .. }
        ));
    }

    #[test]
    fn execution_inside_a_kernel_carries_the_kernels_declared_class() {
        assert!(matches!(
            one(
                DataFusionError::Execution("log of a negative argument".to_owned()),
                PlanOrigin::KernelUdf
            ),
            CatalogError::EvaluationError { .. }
        ));
        assert!(matches!(
            one(
                DataFusionError::Execution("compile.property: unresolved method".to_owned()),
                PlanOrigin::KernelUdf
            ),
            CatalogError::CompileProperty { .. }
        ));
    }

    #[test]
    fn execution_anywhere_else_is_infrastructure() {
        for origin in [PlanOrigin::RuleCompiler, PlanOrigin::Analytics] {
            let found = one(DataFusionError::Execution("boom".to_owned()), origin);
            let CatalogError::Infrastructure { op, .. } = found else {
                panic!("expected infrastructure, found {found}");
            };
            assert_eq!(op, OP_EXECUTION);
        }
    }

    #[test]
    fn the_four_carrier_failures_are_infrastructure_with_their_source_intact() {
        let cases: Vec<(DataFusionError, &str)> = vec![
            (
                DataFusionError::ArrowError(
                    Box::new(ArrowError::InvalidArgumentError("bad".to_owned())),
                    None,
                ),
                OP_ARROW,
            ),
            (
                DataFusionError::ParquetError(Box::new(ParquetError::General("bad".to_owned()))),
                OP_PARQUET,
            ),
            (
                DataFusionError::ObjectStore(Box::new(object_store::Error::NotFound {
                    path: "x".to_owned(),
                    source: Box::new(io::Error::other("gone")),
                })),
                OP_OBJECT_STORE,
            ),
            (DataFusionError::IoError(io::Error::other("disk")), OP_IO),
        ];
        for (error, expected_op) in cases {
            let found = one(error, PlanOrigin::Analytics);
            let CatalogError::Infrastructure { op, source } = found else {
                panic!("expected infrastructure, found {found}");
            };
            assert_eq!(op, expected_op);
            assert!(!source.to_string().is_empty());
        }
    }

    #[test]
    fn a_configuration_failure_names_the_key_it_refused() {
        // PROBE X3 in the DataFusion capability map (§D8): `ConfigOptions::set` returns
        // this, unquoted, where `SessionConfig::set_str` panics instead.
        let found = one(
            DataFusionError::Configuration(
                "Error setting config datafusion.explain.format".to_owned(),
            ),
            PlanOrigin::Analytics,
        );
        let CatalogError::ConfigInvalid { key, .. } = found else {
            panic!("expected a configuration failure, found {found}");
        };
        assert_eq!(key, "datafusion.explain.format");
    }

    #[test]
    fn a_configuration_failure_that_names_no_key_says_only_the_namespace() {
        let found = one(
            DataFusionError::Configuration("something went wrong".to_owned()),
            PlanOrigin::Analytics,
        );
        let CatalogError::ConfigInvalid { key, reason } = found else {
            panic!("expected a configuration failure, found {found}");
        };
        assert_eq!(key, "datafusion");
        assert_eq!(reason, "something went wrong");
    }

    #[test]
    fn a_collection_reports_every_error_it_contains() {
        let found = classify(
            DataFusionError::Collection(vec![
                DataFusionError::Plan("one".to_owned()),
                DataFusionError::Plan("two".to_owned()),
                DataFusionError::Plan("three".to_owned()),
            ]),
            PlanOrigin::Analytics,
        );
        assert_eq!(found.len(), 3);
        assert!(
            found
                .iter()
                .all(|error| matches!(error, CatalogError::UserModel { .. }))
        );
    }

    #[test]
    fn containers_nest_and_still_report_every_leaf() {
        let nested = DataFusionError::Context(
            "while planning".to_owned(),
            Box::new(DataFusionError::Collection(vec![
                DataFusionError::Plan("one".to_owned()),
                DataFusionError::Shared(Arc::new(DataFusionError::Plan("two".to_owned()))),
            ])),
        );
        let found = classify(nested, PlanOrigin::Analytics);
        assert_eq!(found.len(), 2);
        assert!(
            found
                .iter()
                .all(|error| matches!(error, CatalogError::UserModel { .. }))
        );
    }

    #[test]
    fn a_diagnostic_is_unwrapped_to_the_error_it_decorates() {
        let inner = DataFusionError::Plan("no such column".to_owned());
        let decorated = inner.with_diagnostic(datafusion::common::Diagnostic::new_error(
            "no such column",
            None,
        ));
        assert!(matches!(
            one(decorated, PlanOrigin::Analytics),
            CatalogError::UserModel { .. }
        ));
    }

    #[test]
    fn a_shared_error_with_other_holders_keeps_its_class() {
        let shared = Arc::new(DataFusionError::Plan("no such column".to_owned()));
        let other_holder = Arc::clone(&shared);
        assert!(matches!(
            one(DataFusionError::Shared(shared), PlanOrigin::Analytics),
            CatalogError::UserModel { .. }
        ));
        drop(other_holder);
    }

    #[test]
    fn an_external_platform_error_is_passed_through_unchanged() {
        let platform = CatalogError::Sealed;
        let wrapped = DataFusionError::External(Box::new(platform));
        assert!(matches!(
            one(wrapped, PlanOrigin::KernelUdf),
            CatalogError::Sealed
        ));
    }

    #[test]
    fn shared_external_platform_errors_keep_class_and_fields_through_all_wrappers() {
        use miette::Diagnostic;

        let inputs = vec![
            CatalogError::Semantic(Arc::new(pse_ids::CanonError::Cancelled)),
            CatalogError::Cancelled,
            CatalogError::Sealed,
            CatalogError::Admission {
                path: "authored.units".to_owned(),
                reason: "duplicate key".to_owned(),
            },
            CatalogError::ConfigInvalid {
                key: "limit".to_owned(),
                reason: "zero".to_owned(),
            },
            CatalogError::Canon(pse_ids::CanonError::InvalidKey {
                column: "id".to_owned(),
                reason: "nullable".to_owned(),
            }),
            CatalogError::Canon(pse_ids::CanonError::Arrow(ArrowError::ParseError(
                "bad IPC".to_owned(),
            ))),
            CatalogError::Infrastructure {
                op: "read".to_owned(),
                source: io::Error::other("disk").into(),
            },
        ];
        for error in inputs {
            let expected_class = error.code().expect("class is declared").to_string();
            let expected_message = error.to_string();
            let shared = Arc::new(
                DataFusionError::Context(
                    "outer".to_owned(),
                    Box::new(DataFusionError::Collection(vec![
                        DataFusionError::External(Box::new(error)),
                    ])),
                )
                .with_diagnostic(datafusion::common::Diagnostic::new_error("wrapped", None)),
            );
            let other_holder = Arc::clone(&shared);
            let classified = one(DataFusionError::Shared(shared), PlanOrigin::Analytics);
            assert_eq!(
                classified.code().expect("class survives").to_string(),
                expected_class
            );
            // Arrow's owned reporting copy wraps the rendered foreign source once.
            if !matches!(
                classified,
                CatalogError::Canon(pse_ids::CanonError::Arrow(_))
            ) {
                assert_eq!(classified.to_string(), expected_message);
            }
            drop(other_holder);
        }
    }

    #[test]
    fn collapsed_collections_retain_every_typed_related_diagnostic() {
        use miette::Diagnostic;
        let error = collapse_classified(classify(
            DataFusionError::Collection(vec![
                DataFusionError::External(Box::new(CatalogError::Semantic(Arc::new(
                    pse_ids::CanonError::Cancelled,
                )))),
                DataFusionError::External(Box::new(CatalogError::Sealed)),
            ]),
            PlanOrigin::Analytics,
        ));
        assert!(error.code().is_none());
        let codes = error
            .related()
            .expect("related diagnostic group")
            .map(|error| error.code().expect("leaf code").to_string())
            .collect::<Vec<_>>();
        assert_eq!(codes, vec!["runtime::cancelled", "schema::sealed"]);
        let shared = Arc::new(DataFusionError::External(Box::new(error)));
        let retained = Arc::clone(&shared);
        let copied = one(DataFusionError::Shared(shared), PlanOrigin::Analytics);
        assert_eq!(copied.related().expect("group preserved").count(), 2);
        drop(retained);
    }

    #[test]
    fn an_external_foreign_error_is_a_platform_bug() {
        let wrapped = DataFusionError::External(Box::new(io::Error::other("foreign")));
        assert!(matches!(
            one(wrapped, PlanOrigin::Analytics),
            CatalogError::Internal { .. }
        ));
    }

    #[test]
    fn the_remaining_variants_are_internal_invariants() {
        let cases = vec![
            DataFusionError::NotImplemented("later".to_owned()),
            DataFusionError::Substrait("later".to_owned()),
            DataFusionError::Ffi("later".to_owned()),
            DataFusionError::Internal("bug".to_owned()),
            DataFusionError::SQL(
                Box::new(
                    datafusion::sql::sqlparser::parser::ParserError::ParserError("bad".to_owned()),
                ),
                None,
            ),
        ];
        for error in cases {
            assert!(matches!(
                one(error, PlanOrigin::Analytics),
                CatalogError::Internal { .. }
            ));
        }
    }

    #[test]
    fn a_cancelled_engine_task_is_an_internal_invariant_too() {
        // `ExecutionJoin` is the one variant that cannot be constructed from a literal:
        // its payload is a `tokio::task::JoinError`, which only the runtime produces.
        // Aborting a pending task yields one without a panic in the output.
        let runtime = tokio::runtime::Builder::new_current_thread().build();
        let Ok(runtime) = runtime else {
            panic!("a current-thread runtime is buildable");
        };
        let joined = runtime.block_on(async {
            let handle = tokio::spawn(std::future::pending::<()>());
            handle.abort();
            handle.await
        });
        let Err(join_error) = joined else {
            panic!("an aborted task does not complete");
        };
        assert!(matches!(
            one(
                DataFusionError::ExecutionJoin(Box::new(join_error)),
                PlanOrigin::Analytics
            ),
            CatalogError::Internal { .. }
        ));
    }

    #[test]
    fn every_classification_reports_at_least_one_finding() {
        let empty = classify(
            DataFusionError::Collection(Vec::new()),
            PlanOrigin::Analytics,
        );
        assert!(
            empty.is_empty(),
            "an empty collection is the one input with nothing to report"
        );
        let single = classify(
            DataFusionError::Internal("x".to_owned()),
            PlanOrigin::Analytics,
        );
        assert_eq!(single.len(), 1);
    }

    #[test]
    fn the_origins_spell_themselves() {
        assert_eq!(PlanOrigin::RuleCompiler.as_str(), "rule_compiler");
        assert_eq!(PlanOrigin::Analytics.to_string(), "analytics");
        assert_eq!(PlanOrigin::KernelUdf.as_str(), "kernel_udf");
    }
}
