// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Optional continuation effort target. Native work remains an observation, never
//! scientific permission or a replacement for the enclosing finite allowance.

/// Opt-in adaptation from comparable native nonlinear iteration counts.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct EffortPolicy {
    /// Desired iterations at the next qualified continuation endpoint.
    pub target_iterations: u32,
}

impl EffortPolicy {
    /// Whether this declaration names a positive finite work target.
    pub fn is_valid(self) -> bool {
        self.target_iterations > 0
    }
}
