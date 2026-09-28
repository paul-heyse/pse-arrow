// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The catalog read scope of a session (Plan 22 X10).
//!
//! A reader holds a catalog lease on the publication it reads, granted in the
//! workspace's current maintenance epoch. The epoch advances before every maintenance
//! effect, so the pair (workspace, epoch) names a period in which the files a reader
//! selected are not removed: it is the lookup input of every cached Delta snapshot,
//! resident selection and file-metadata namespace a reader shares. A session without a
//! scope bypasses those caches. The scope is protection only together with its lease;
//! the catalog, not this value, keeps the files.
use datafusion::execution::{config::SessionConfig, session_state::SessionState};
use pse_model::generated::identities::WorkspaceId;
use std::sync::Arc;

/// The workspace and maintenance epoch a reader's catalog lease was granted in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReadScope {
    /// The workspace whose maintenance is serialized with the reader.
    pub workspace: WorkspaceId,
    /// Its maintenance epoch when the lease was granted.
    pub epoch: i64,
}

impl ReadScope {
    /// Install this scope on a session configuration; readers of the session key their
    /// caches on it.
    pub fn install(self, config: &mut SessionConfig) {
        config.set_extension(Arc::new(self));
    }

    /// The scope a session configuration carries, if any.
    pub fn of_config(config: &SessionConfig) -> Option<Self> {
        config.get_extension::<Self>().map(|scope| *scope)
    }

    /// The scope a session carries, if any.
    pub fn of(state: &SessionState) -> Option<Self> {
        Self::of_config(state.config())
    }

    /// The cache namespace of this scope, for file-metadata caches keyed by text.
    pub(crate) fn namespace(&self) -> String {
        format!("pse.scope:{}:{}", self.workspace, self.epoch)
    }
}
