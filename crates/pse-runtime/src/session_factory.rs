// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Session factories retain the deployment's one engine pool and platform reserver.

use crate::{RuntimeError, SharedRuntime};
use pse_catalog::session::{EngineProfile, SessionFactory};

impl SharedRuntime {
    /// Construct session handles without creating a second runtime or allocation budget.
    /// # Errors
    /// The explicit engine profile or bound execution configuration is invalid.
    pub fn session_factory(&self, profile: EngineProfile) -> Result<SessionFactory, RuntimeError> {
        Ok(SessionFactory::new(
            self.runtime_env(),
            self.reserver(),
            self.budget().execution.clone(),
            self.budget().threads,
            profile,
        )?)
    }
}
