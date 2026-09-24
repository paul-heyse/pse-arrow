// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Identity policy checks preserve explicit bytes and verify named-policy correspondence.

#![allow(
    clippy::unwrap_used,
    reason = "fixture failures retain the typed authoring diagnostic"
)]

use pse_authoring::{
    AuthoringError, SourceSpan,
    ids::{IdPolicy, entity_id, parse_id, uuid_v7},
};
use pse_ids::{SemanticId, named_id};

#[test]
fn explicit_and_named_identity_policies_have_distinct_creation_semantics() {
    let package = SemanticId::from_bytes([1; 16]);
    let at = SourceSpan::head(package);
    assert!(matches!(
        entity_id(IdPolicy::Explicit, package, None, "pkg.object", at),
        Err(AuthoringError::MissingId { .. })
    ));
    let explicit = uuid_v7();
    assert_eq!(explicit.as_bytes()[6] >> 4, 7);
    for name in ["pkg.before", "pkg.after"] {
        assert_eq!(
            entity_id(
                IdPolicy::Explicit,
                package,
                Some(&explicit.to_hex()),
                name,
                at
            )
            .unwrap(),
            explicit
        );
    }
    assert_eq!(
        entity_id(IdPolicy::Named, package, None, "pkg.object", at).unwrap(),
        named_id(package, "pkg.object")
    );
    assert!(
        entity_id(
            IdPolicy::Named,
            package,
            Some(&explicit.to_hex()),
            "pkg.object",
            at
        )
        .is_err()
    );
}

#[test]
fn compact_and_hyphenated_id_spellings_preserve_the_same_identity() {
    let at = SourceSpan::head(SemanticId::NIL);
    let compact = parse_id("01900000000070008000000000000001", at).unwrap();
    let uuid = parse_id("01900000-0000-7000-8000-000000000001", at).unwrap();
    assert_eq!(compact, uuid);
    for malformed in ["", "xyz", "0190000000007000800000000000000"] {
        assert!(parse_id(malformed, at).is_err());
    }
}
