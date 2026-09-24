// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit creation identities and checked deterministic package identities (§5.1).

use pse_ids::{SemanticId, named_id};
pub use pse_model::generated::enums::IdPolicy;

use crate::{AuthoringError, SourceSpan};

/// Resolve an authored identity under its package's declared policy.
///
/// # Errors
/// Missing explicit identities are `MissingId`; malformed identities and mismatches
/// against a named-policy declaration are typed syntax/reference failures.
pub fn entity_id(
    policy: IdPolicy,
    package_id: SemanticId,
    explicit: Option<&str>,
    qualified_name: &str,
    at: SourceSpan,
) -> Result<SemanticId, AuthoringError> {
    let supplied = explicit.map(|value| parse_id(value, at)).transpose()?;
    match policy {
        IdPolicy::Explicit => supplied.ok_or_else(|| AuthoringError::MissingId {
            at,
            kind: "entity".to_owned(),
            name: qualified_name.to_owned(),
        }),
        IdPolicy::Named => {
            if qualified_name.is_empty() {
                return Err(invalid_id(at, "a nonempty qualified name", qualified_name));
            }
            let expected = named_id(package_id, qualified_name);
            if supplied.is_some_and(|supplied| supplied != expected) {
                return Err(invalid_id(
                    at,
                    &format!("named identity {expected}"),
                    explicit.unwrap_or(""),
                ));
            }
            Ok(expected)
        }
    }
}

/// Read a compact semantic ID or a hyphenated UUID without changing its bytes.
///
/// # Errors
/// Malformed identity syntax is reported at the supplied source span.
pub fn parse_id(text: &str, at: SourceSpan) -> Result<SemanticId, AuthoringError> {
    if let Ok(id) = SemanticId::parse_hex(text) {
        return Ok(id);
    }
    uuid::Uuid::parse_str(text)
        .map(|uuid| SemanticId::from_bytes(*uuid.as_bytes()))
        .map_err(|_| invalid_id(at, "a 128-bit hexadecimal identity or UUID", text))
}

/// Assign a new `UUIDv7` when an authoring tool creates an explicit-policy entity.
pub fn uuid_v7() -> SemanticId {
    SemanticId::from_bytes(*uuid::Uuid::now_v7().as_bytes())
}

fn invalid_id(at: SourceSpan, expected: &str, found: &str) -> AuthoringError {
    AuthoringError::Syntax {
        at,
        offset: at.start,
        expected: expected.to_owned(),
        found: found.to_owned(),
    }
}
