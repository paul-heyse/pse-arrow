// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Optional native custody enrichment; unavailable association never hides product evidence.
use crate::{
    envelope::{CliInvocationId, ErrorRecord, RedactionPolicy},
    environment::Environment,
    managed::supervisor,
};
use pse_diagnostics::process_facts::ManagedInvocationCandidate;
use std::{path::Path, time::Duration};

/// Native readback is separate from product interpretation and does not confer control.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Association {
    /// The selected product source supplied no bounded matching process candidate.
    Absent,
    /// Candidate syntax, original custody, actual kernel identity or membership refused.
    Rejected {
        /// Original producer routing candidate, kept separate from established native facts.
        candidate: ManagedInvocationCandidate,
        /// Original native custody, identity or source refusal.
        diagnostic: ErrorRecord,
    },
    /// Complete native identity/membership checks fenced original unit aggregate observations.
    Observed {
        /// Publisher identity whose current native membership was checked.
        candidate: ManagedInvocationCandidate,
        /// Independent original unit and aggregate accounting observations.
        facts: supervisor::PassiveRecordedObservation,
    },
}
/// Observe exactly the original candidate; this acquires no native or scientific stop capability.
pub async fn inspect(
    root: &Path,
    environment: &Environment,
    candidate: Option<&ManagedInvocationCandidate>,
) -> Association {
    let Some(candidate) = candidate else {
        return Association::Absent;
    };
    let checked = async {
        let id = CliInvocationId::try_from(candidate.invocation.clone())?;
        supervisor::observe_candidate_member(root, &id, &candidate.process, environment).await
    };
    let result = tokio::time::timeout(Duration::from_secs(5), checked)
        .await
        .map_err(|_| crate::HostError::Deadline("selected product native association".into()))
        .and_then(|result| result);
    match result {
        Ok(facts) => Association::Observed {
            candidate: candidate.clone(),
            facts,
        },
        Err(error) => Association::Rejected {
            candidate: candidate.clone(),
            diagnostic: ErrorRecord::from_typed(&error, &RedactionPolicy::from_inputs(environment)),
        },
    }
}
