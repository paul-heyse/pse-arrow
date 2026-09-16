// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Actual conditional-write outcomes, retained independently of result delivery.

use crate::CatalogError;
use pse_ids::ContentHash;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// What the backend established about one attempted conditional replacement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    /// The attempted replacement did not change the target.
    Unchanged,
    /// The request may have taken effect; reconcile exact intended bytes before retry.
    Indeterminate,
    /// The backend confirmed that the intended bytes became visible.
    Visible,
}

/// Evidence for one control object's exact expected and intended content.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublicationOutcome {
    /// Exact backend object path, including the ref or auxiliary-index namespace.
    pub path: String,
    /// Observed bytes, or absence for a conditional create.
    pub expected: Option<ContentHash>,
    /// Intended complete content, usable for response-loss reconciliation.
    pub intended: ContentHash,
    /// Actual visibility evidence; never inferred from a native row count.
    pub visibility: Visibility,
    /// The backend's durability contract completed for this replacement.
    pub durability_confirmed: bool,
}

/// One operation's actual write history. Clones retain the same attempts and outcomes.
#[derive(Clone, Debug, Default)]
pub struct PublicationJournal(Arc<Mutex<JournalState>>);
#[derive(Debug, Default)]
struct JournalState {
    outcomes: Vec<PublicationOutcome>,
    reservation: Option<Box<dyn pse_ids::Reservation>>,
}
impl PublicationJournal {
    /// Inspect current evidence, including an interrupted in-flight request.
    /// # Errors
    /// A poisoned journal indicates an internal execution failure.
    pub fn outcomes(&self) -> Result<Vec<PublicationOutcome>, CatalogError> {
        self.0
            .lock()
            .map(|items| items.outcomes.clone())
            .map_err(|_| poisoned())
    }
    pub(super) fn begin(
        &self,
        outcome: PublicationOutcome,
        reserver: &dyn pse_ids::MemoryReserver,
    ) -> Result<usize, CatalogError> {
        let mut items = self.0.lock().map_err(|_| poisoned())?;
        let bytes = super::control::add(
            super::control::mul(size_of::<PublicationOutcome>(), 4)?,
            outcome.path.capacity(),
        )?;
        items
            .reservation
            .get_or_insert_with(|| reserver.open("store:publication-journal"))
            .try_grow(bytes)?;
        let index = items.outcomes.len();
        items.outcomes.push(outcome);
        Ok(index)
    }
    pub(super) fn finish(
        &self,
        index: usize,
        visibility: Visibility,
        durable: bool,
    ) -> Result<(), CatalogError> {
        let mut items = self.0.lock().map_err(|_| poisoned())?;
        let outcome = items.outcomes.get_mut(index).ok_or_else(poisoned)?;
        outcome.visibility = visibility;
        outcome.durability_confirmed = durable;
        Ok(())
    }
    pub(crate) fn has_visible(&self) -> bool {
        self.0.lock().is_ok_and(|items| {
            items
                .outcomes
                .iter()
                .any(|item| item.visibility == Visibility::Visible)
        })
    }
    pub(crate) fn failure(&self, source: CatalogError) -> CatalogError {
        match self.outcomes() {
            Ok(outcomes)
                if outcomes
                    .iter()
                    .any(|outcome| outcome.visibility != Visibility::Unchanged) =>
            {
                // Nested operations share this journal. Keep every preceding write
                // when an inner backend error already carries a partial outcome.
                let source = match source {
                    CatalogError::Publication { source, .. } => source,
                    source => Box::new(source),
                };
                CatalogError::Publication { outcomes, source }
            }
            _ => source,
        }
    }
}
fn poisoned() -> CatalogError {
    CatalogError::Internal {
        message: "publication journal is unavailable".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_failure_keeps_prior_visible_writes_and_releases_journal_lease() {
        let budget = pse_ids::FixedBudget::new(4096);
        let journal = PublicationJournal::default();
        let outcome = |path: &str, visibility| PublicationOutcome {
            path: path.into(),
            expected: None,
            intended: pse_ids::encoding_checksum(path.as_bytes()).content_hash(),
            visibility,
            durability_confirmed: visibility == Visibility::Visible,
        };
        let first = outcome("refs/model", Visibility::Visible);
        let second = outcome("refs/case", Visibility::Indeterminate);
        journal.begin(first.clone(), budget.as_ref()).unwrap();
        journal.begin(second.clone(), budget.as_ref()).unwrap();
        let error = journal.failure(CatalogError::Publication {
            outcomes: vec![second.clone()],
            source: Box::new(poisoned()),
        });
        let CatalogError::Publication { outcomes, source } = error else {
            panic!("conditional-write evidence must survive nested failures");
        };
        assert_eq!(outcomes, vec![first, second]);
        assert!(matches!(*source, CatalogError::Internal { .. }));
        assert!(budget.reserved() > 0);
        drop(journal);
        assert_eq!(budget.reserved(), 0);
    }
}
