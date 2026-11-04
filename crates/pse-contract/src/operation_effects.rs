// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owner-declared operation effects, separate from transport success or scientific permission.

use crate::boundary_projection::{Direction, LocalRule};
use crate::{projection_enum, projection_struct};

projection_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(rename_all = "snake_case")]
    /// Observable original operation scope, never inferred from transport or method naming.
    pub enum EffectScope {
        projection: { name: "EffectScope", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Read original supplied or protected facts without a requested mutation.
        Observation => 1;
        /// Admit original source and its actual providers into a local native owner.
        SourceAdmission => 2;
        /// Original caller-issued canonical command or publication.
        CanonicalCommand => 3;
        /// Explicit release of retained canonical custody.
        RetentionRelease => 4;
        /// Create a new bounded requested output file.
        FileExport => 5;
        /// Explicit finite runtime sampling/capture.
        Capture => 6;
        /// Start an original native computation, including its configured canonical publication.
        NativeExecution => 7;
    }
}
projection_struct! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    /// Source-declared effects and exact-request repeatability, without scientific permission.
    pub struct EffectFacts {
        projection: { name: "EffectFacts", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Actual operation scope, never inferred from a method spelling.
        pub scope: EffectScope => { tag: 1, direction: Direction::Both, rules: [LocalRule::Required] };
        /// Observation rather than mutation, admission or capture.
        pub read_only: bool => { tag: 2, direction: Direction::Both, rules: [] };
        /// Original owner's stated repeatability contract for the exact original request.
        pub idempotent: bool => { tag: 3, direction: Direction::Both, rules: [] };
        /// Explicit release of retained data/custody.
        pub destructive: bool => { tag: 4, direction: Direction::Both, rules: [] };
    }
}
impl EffectFacts {
    /// Ordinary observation, without granting scientific or source completeness.
    pub const OBSERVATION: Self = Self {
        scope: EffectScope::Observation,
        read_only: true,
        idempotent: true,
        destructive: false,
    };
    /// Native original source/context admission, without a numerical solve.
    pub const SOURCE_ADMISSION: Self = Self {
        scope: EffectScope::SourceAdmission,
        read_only: false,
        idempotent: true,
        destructive: false,
    };
    /// Canonical command under the original exact-operation owner.
    pub const CANONICAL_COMMAND: Self = Self {
        scope: EffectScope::CanonicalCommand,
        read_only: false,
        idempotent: true,
        destructive: false,
    };
    /// Explicit retained-custody release independently admitted by its original owner.
    pub const RETENTION_RELEASE: Self = Self {
        scope: EffectScope::RetentionRelease,
        read_only: false,
        idempotent: false,
        destructive: true,
    };
    /// Fresh bounded output, with no overwrite or silent retry.
    pub const FILE_EXPORT: Self = Self {
        scope: EffectScope::FileExport,
        read_only: false,
        idempotent: false,
        destructive: false,
    };
    /// An original capture cannot replay a previous time interval.
    pub const CAPTURE: Self = Self {
        scope: EffectScope::Capture,
        read_only: false,
        idempotent: false,
        destructive: false,
    };
    /// One caller-issued native execution; repetition resolves only that original identity.
    pub const NATIVE_EXECUTION: Self = Self {
        scope: EffectScope::NativeExecution,
        read_only: false,
        idempotent: true,
        destructive: false,
    };
}
