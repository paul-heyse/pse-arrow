// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Operation-owned omission defaults for runtime observation and study execution.

/// A bounded operational inventory page.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct InventoryControls {
    /// Maximum rows returned by the selected inventory operation.
    pub limit: i64,
}
impl Default for InventoryControls {
    fn default() -> Self {
        Self { limit: 100 }
    }
}
/// Durable progress observation controls.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ProgressControls {
    /// Continue observing until the selected attempt ends.
    pub follow: bool,
    /// Maximum rows retained from each progress stream at a time.
    pub page: usize,
}
impl Default for ProgressControls {
    fn default() -> Self {
        Self {
            follow: true,
            page: 256,
        }
    }
}
/// The failure policy for a finite sequence of admitted modeling operations.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct RunControls {
    /// Continue independent operations after one operation fails.
    pub continue_independent: bool,
}
/// Bounded in-process study execution.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct StudyRunControls {
    /// Maximum points the local execution may admit.
    pub maximum_points: usize,
}
impl Default for StudyRunControls {
    fn default() -> Self {
        Self {
            maximum_points: 1024,
        }
    }
}
/// Durable study queue submission.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct StudySubmitControls {
    /// Maximum job tries including the initial attempt.
    pub max_tries: u32,
    /// Queue priority passed to the operational store.
    pub priority: i32,
}
impl Default for StudySubmitControls {
    fn default() -> Self {
        Self {
            max_tries: super::RetryPolicy::ONCE.max_tries,
            priority: 0,
        }
    }
}
/// Observation timing while waiting for a study publication.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct StudyWaitControls {
    /// Seconds between study state observations.
    pub poll_seconds: f64,
    /// Optional seconds after which observation stops with a typed refusal.
    pub timeout_seconds: Option<f64>,
}
impl Default for StudyWaitControls {
    fn default() -> Self {
        Self {
            poll_seconds: 0.5,
            timeout_seconds: None,
        }
    }
}

#[cfg(test)]
mod boundary_unit {
    use super::*;
    #[test]
    fn omission_matches_explicit_runtime_defaults() {
        let omitted: ProgressControls = serde_json::from_str("{}").unwrap();
        let explicit = ProgressControls::default();
        assert_eq!(omitted.follow, explicit.follow);
        assert_eq!(omitted.page, explicit.page);
        let run: RunControls = serde_json::from_str("{}").unwrap();
        assert!(!run.continue_independent);
        let retry: StudySubmitControls = serde_json::from_str("{}").unwrap();
        assert_eq!(retry.max_tries, super::super::RetryPolicy::ONCE.max_tries);
    }
    #[test]
    fn controls_reject_unknown_fields() {
        assert!(serde_json::from_str::<InventoryControls>(r#"{"maximum":1}"#).is_err());
        assert!(serde_json::from_str::<StudyRunControls>(r#"{"priority":1}"#).is_err());
    }
}
