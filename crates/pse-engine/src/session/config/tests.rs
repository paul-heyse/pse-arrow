// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_schema::model::provider::{ProviderPolicy, ProviderScope};

#[test]
fn deployment_limits_do_not_change_semantic_identity() {
    let original = BTreeMap::from([
        (
            "datafusion.execution.time_zone".to_owned(),
            Some("UTC".to_owned()),
        ),
        (
            "datafusion.runtime.metadata_cache_limit".to_owned(),
            Some("0".to_owned()),
        ),
        (
            "datafusion.execution.sort_spill_reservation_bytes".to_owned(),
            Some("1024".to_owned()),
        ),
    ]);
    let mut changed = original.clone();
    changed.insert(
        "datafusion.runtime.metadata_cache_limit".to_owned(),
        Some("4096".to_owned()),
    );
    changed.insert(
        "datafusion.execution.sort_spill_reservation_bytes".to_owned(),
        Some("2048".to_owned()),
    );
    assert_eq!(settings_hash(&original), settings_hash(&changed));
    assert_eq!(
        semantic_settings(&original).unwrap(),
        semantic_settings(&changed).unwrap()
    );
    changed.insert(
        "datafusion.execution.time_zone".to_owned(),
        Some("America/New_York".to_owned()),
    );
    assert_ne!(settings_hash(&original), settings_hash(&changed));
}

#[test]
fn unknown_settings_and_explicit_absence_remain_semantic() {
    let absent = BTreeMap::from([("datafusion.runtime.future_extension".to_owned(), None)]);
    let text = BTreeMap::from([(
        "datafusion.runtime.future_extension".to_owned(),
        Some("None".to_owned()),
    )]);
    assert_eq!(semantic_settings(&absent).unwrap(), absent);
    assert_ne!(settings_hash(&absent), settings_hash(&text));
    assert_ne!(settings_hash(&absent), settings_hash(&BTreeMap::new()));
    assert!(semantic_settings(&BTreeMap::new()).is_err());
}

#[test]
fn policy_admission_ceiling_is_separate_from_policy_meaning() {
    let mut actual = ProviderPolicy::new(pse_ids::SemanticId::NIL, ProviderScope::Root);
    actual.max_bytes = Some(4096);
    actual.required_settings.insert(
        "datafusion.execution.spill_compression".to_owned(),
        "lz4_frame".to_owned(),
    );
    actual.defaults.insert(
        "datafusion.execution.time_zone".to_owned(),
        "UTC".to_owned(),
    );
    let semantic = semantic_policy(&actual);
    assert_eq!(semantic.max_bytes, None);
    assert!(semantic.required_settings.is_empty());
    assert!(policy_semantics_equal(&actual, &semantic));
    actual.max_bytes = Some(1024);
    assert!(policy_semantics_equal(&actual, &semantic));
    actual.defaults.insert(
        "datafusion.execution.time_zone".to_owned(),
        "Europe/Paris".to_owned(),
    );
    assert!(!policy_semantics_equal(&actual, &semantic));
}
