// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Closed terminal outcomes of an actual pass attempt (blueprint §14.3).

/// How an attempt ended; a cancelled attempt is never an empty successful output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PassStatus {
    /// Every output port and postcondition was admitted.
    Ok,
    /// Execution or admission failed.
    Failed,
    /// Cancellation stopped the attempt.
    Cancelled,
    /// Complete exact inputs permitted an admitted output to be reused.
    Reused,
}
impl PassStatus {
    /// The sole roster projected into the registry's closed dictionary.
    pub const ALL: [Self; 4] = [Self::Ok, Self::Failed, Self::Cancelled, Self::Reused];
    /// The declared wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Reused => "reused",
        }
    }
}
