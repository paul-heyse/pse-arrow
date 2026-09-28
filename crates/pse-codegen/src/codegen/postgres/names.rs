// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! PostgreSQL names of registry declarations: one spelling rule per kind of object.

use crate::SchemaError;

/// PostgreSQL silently truncates identifiers to 63 bytes (`NAMEDATALEN - 1`); a generated
/// name that long would stop naming what it was generated from, so it is refused.
pub(super) const MAX_IDENTIFIER_BYTES: usize = 63;

/// The domain every content hash is stored as: 32 bytes, checked.
pub(super) const CONTENT_HASH_DOMAIN: &str = "content_hash";

/// `AttemptState` is `attempt_state`: registry names are PascalCase, PostgreSQL types
/// are snake_case.
pub(crate) fn snake(name: &str) -> String {
    let characters = name.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(name.len() + 4);
    for (index, character) in characters.iter().enumerate() {
        if character.is_ascii_uppercase() {
            let previous = index.checked_sub(1).and_then(|i| characters.get(i));
            let next = characters.get(index + 1);
            let boundary = previous.is_some_and(|p| p.is_ascii_lowercase() || p.is_ascii_digit())
                || (previous.is_some_and(char::is_ascii_uppercase)
                    && next.is_some_and(char::is_ascii_lowercase));
            if boundary {
                out.push('_');
            }
            out.push(character.to_ascii_lowercase());
        } else {
            out.push(*character);
        }
    }
    out
}

/// The ENUM type of a registry enumeration.
pub(crate) fn enum_type(name: &str) -> String {
    snake(name)
}

/// The domain of an entity identity: `attempt` is `attempt_id`.
pub(crate) fn identity_domain(identity: &str) -> String {
    format!("{identity}_id")
}

/// A quoted identifier; registry names are identifiers, so quoting only protects
/// spellings PostgreSQL reserves (`real`, `text`, ...).
pub(super) fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// A schema-qualified object name.
pub(super) fn qualified(name: &str) -> String {
    format!("{}.{name}", pse_schema::store::SCHEMA)
}

/// A name that PostgreSQL keeps whole.
pub(super) fn bounded(name: String) -> Result<String, SchemaError> {
    if name.len() > MAX_IDENTIFIER_BYTES {
        return Err(super::error(format!(
            "generated PostgreSQL name {name} exceeds {MAX_IDENTIFIER_BYTES} bytes"
        )));
    }
    Ok(name)
}

/// A constraint of `table`: `<table>_<name>_<suffix>` (PostgreSQL's own convention).
pub(super) fn constraint(table: &str, name: &str, suffix: &str) -> Result<String, SchemaError> {
    bounded(format!("{table}_{name}_{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_names_become_snake_case_types() {
        assert_eq!(snake("AttemptState"), "attempt_state");
        assert_eq!(snake("NativeRunState"), "native_run_state");
        assert_eq!(snake("EvidenceUnavailableReason"), "evidence_unavailable_reason");
        assert_eq!(snake("HTTPStatus"), "http_status");
        assert_eq!(snake("Stage2Kind"), "stage2_kind");
        assert_eq!(identity_domain("reader_lease"), "reader_lease_id");
        assert!(constraint(&"t".repeat(60), "x", "check").is_err());
        assert_eq!(
            constraint("jobs", "idempotency_key", "key").ok().as_deref(),
            Some("jobs_idempotency_key_key")
        );
    }
}
