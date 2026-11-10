// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable parent, original finite proposal and exactly-once expectation barriers.
use crate::{
    canonical::{
        CanonicalError, CanonicalStore, REQUEST_TIMEOUT, bounded_query, protected_query,
        transport::{original_deadline, within_clock},
    },
    canonical_script::{Envelope, GuardedScript, ResultSlot},
    generated::surreal as wire,
};
pub use pse_model::generated::runtime::canonical_expectation_finalizations::Row as Finalization;
pub use pse_model::generated::runtime::canonical_expectation_publications::Row as Publication;
pub use pse_model::generated::runtime::{
    canonical_expectation_boundaries::Row as Boundary,
    canonical_expectation_proposals::Row as Proposal, canonical_expectations::Row as Expectation,
};
use surrealdb_types::Object;

const DOCUMENT_BYTES: usize = 16 << 20;
fn key(value: &str) -> Result<(), CanonicalError> {
    if value.is_empty() || value.len() > 4096 {
        return Err(CanonicalError::PayloadLimit);
    }
    Ok(())
}
fn document(value: &[u8]) -> Result<(), CanonicalError> {
    if value.is_empty() || value.len() > DOCUMENT_BYTES {
        return Err(CanonicalError::PayloadLimit);
    }
    Ok(())
}
impl CanonicalStore {
    /// Bind terminal expectation data to its existing sealed ordinary result manifest.
    pub async fn publish_expectation(
        &self,
        row: Publication,
    ) -> Result<Publication, CanonicalError> {
        key(&row.key)?;
        key(&row.run)?;
        key(&row.attempt)?;
        key(&row.manifest)?;
        let object = wire::create_canonical_expectation_publications(&(&row).into())?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            let mut response = protected_query("canonical_expectations::publication", || Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>(
                "LET $final=SELECT key FROM ONLY type::record('canonical_expectation_finalizations',$row.key); LET $attempt=SELECT run,phase,closed_manifest FROM ONLY type::record('canonical_attempts',$row.attempt); IF $final=NONE OR $attempt=NONE OR $attempt.run!=$row.run OR $attempt.phase NOT IN ['terminal','terminal_compacted'] OR $attempt.closed_manifest!=$row.manifest {THROW 'expectation publication is not the original sealed terminal result';}; LET $old=SELECT * FROM ONLY type::record('canonical_expectation_publications',$row.key); IF $old=NONE {CREATE type::record('canonical_expectation_publications',$row.key) CONTENT $row;};",
                "SELECT * FROM ONLY type::record('canonical_expectation_publications',$row.key)")?).bind(("row", object.clone())))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectation_publications).transpose()?.ok_or(CanonicalError::IncompleteResponse)
        })).await
    }
    /// Read the original publication locator, without recreating unavailable scientific rows.
    pub async fn expectation_publication(
        &self,
        parent: &str,
    ) -> Result<Option<Publication>, CanonicalError> {
        key(parent)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut response = bounded_query(self.db.script(GuardedScript::new(Envelope::ReadOnly).returning::<Option<Object>>("", "SELECT * FROM ONLY type::record('canonical_expectation_publications',$key)")?).bind(("key", parent.to_owned()))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectation_publications).transpose().map_err(Into::into)
        })).await
    }
    /// Retain the original terminal parent decision independently of its last adaptive barrier.
    pub async fn finalize_expectation(
        &self,
        row: Finalization,
    ) -> Result<Finalization, CanonicalError> {
        key(&row.key)?;
        document(row.report.as_slice())?;
        let object = wire::create_canonical_expectation_finalizations(&(&row).into())?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            let mut response = protected_query("canonical_expectations::finalize", || Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>(
                "LET $parent=SELECT key FROM ONLY type::record('canonical_expectations',$row.key); IF $parent=NONE {THROW 'expectation parent absent';}; LET $last=(SELECT ordinal FROM canonical_expectation_boundaries WHERE parent=$row.key ORDER BY ordinal DESC LIMIT 1)[0]; IF $last.ordinal!=$row.latest_boundary {THROW 'expectation terminal boundary differs';}; LET $old=SELECT * FROM ONLY type::record('canonical_expectation_finalizations',$row.key); IF $old!=NONE AND ($old.latest_boundary!=$row.latest_boundary OR $old.report!=$row.report) {THROW 'expectation original terminal decision differs';}; IF $old=NONE {CREATE type::record('canonical_expectation_finalizations',$row.key) CONTENT $row;};",
                "SELECT * FROM ONLY type::record('canonical_expectation_finalizations',$row.key)")?).bind(("row", object.clone())))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectation_finalizations).transpose()?.ok_or(CanonicalError::IncompleteResponse)
        })).await
    }
    /// Read original terminal parent custody; a prior checkpoint never substitutes for it.
    pub async fn expectation_finalization(
        &self,
        parent: &str,
    ) -> Result<Option<Finalization>, CanonicalError> {
        key(parent)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut response = bounded_query(self.db.script(GuardedScript::new(Envelope::ReadOnly).returning::<Option<Object>>("", "SELECT * FROM ONLY type::record('canonical_expectation_finalizations',$key)")?).bind(("key", parent.to_owned()))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectation_finalizations).transpose().map_err(Into::into)
        })).await
    }
    /// Retain one exact immutable request before any random sample or child effect.
    pub async fn create_expectation(
        &self,
        row: Expectation,
    ) -> Result<Expectation, CanonicalError> {
        key(&row.key)?;
        key(&row.supplier)?;
        key(&row.revision)?;
        key(&row.physical)?;
        document(row.request.as_slice())?;
        let object = wire::create_canonical_expectations(&(&row).into())?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            let mut response = protected_query("canonical_expectations::create", || Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>(
                "LET $old=SELECT * FROM ONLY type::record('canonical_expectations',$row.key); IF $old!=NONE AND ($old.request!=$row.request OR $old.supplier!=$row.supplier OR $old.revision!=$row.revision OR $old.physical!=$row.physical) { THROW 'expectation original request differs'; }; IF $old=NONE { CREATE type::record('canonical_expectations',$row.key) CONTENT $row; };",
                "SELECT * FROM ONLY type::record('canonical_expectations',$row.key)")?).bind(("row", object.clone())))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectations).transpose()?.ok_or(CanonicalError::IncompleteResponse)
        })).await
    }
    /// Read original request custody without invoking a scientific producer.
    pub async fn expectation(&self, parent: &str) -> Result<Option<Expectation>, CanonicalError> {
        key(parent)?;
        let activity = self.admit_activity()?;
        activity
            .scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
                let mut response = bounded_query(
                    self.db
                        .script(
                            GuardedScript::new(Envelope::ReadOnly).returning::<Option<Object>>(
                                "",
                                "SELECT * FROM ONLY type::record('canonical_expectations',$key)",
                            )?,
                        )
                        .bind(("key", parent.to_owned())),
                )
                .await?;
                ResultSlot::<Option<Object>>::returned()
                    .take(&mut response)?
                    .map(wire::decode_canonical_expectations)
                    .transpose()
                    .map_err(Into::into)
            }))
            .await
    }
    /// Append an immutable original proposal before its ordinary finite child activates.
    pub async fn retain_expectation_proposal(
        &self,
        row: Proposal,
    ) -> Result<Proposal, CanonicalError> {
        key(&row.key)?;
        key(&row.parent)?;
        key(&row.child)?;
        document(row.samples.as_slice())?;
        document(row.definition.as_slice())?;
        if row.key != format!("{}/{}", row.parent, row.ordinal) {
            return Err(CanonicalError::PayloadLimit);
        }
        let object = wire::create_canonical_expectation_proposals(&(&row).into())?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            let mut response = protected_query("canonical_expectations::proposal", || Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>(
                "LET $parent=SELECT * FROM ONLY type::record('canonical_expectations',$row.parent); IF $parent=NONE {THROW 'expectation parent absent';}; LET $old=SELECT * FROM ONLY type::record('canonical_expectation_proposals',$row.key); IF $old!=NONE AND ($old.parent!=$row.parent OR $old.ordinal!=$row.ordinal OR $old.child!=$row.child OR $old.samples!=$row.samples OR $old.definition!=$row.definition) {THROW 'expectation original proposal differs';}; IF $old=NONE { IF (SELECT key FROM ONLY type::record('canonical_expectation_finalizations',$row.parent))!=NONE {THROW 'expectation parent is terminal';}; LET $prior=SELECT ordinal,checkpoint FROM canonical_expectation_boundaries WHERE parent=$row.parent ORDER BY ordinal DESC LIMIT 1; IF ($row.ordinal=0 AND array::len($prior)!=0) OR ($row.ordinal>0 AND (array::len($prior)!=1 OR $prior[0].ordinal+1!=$row.ordinal OR $prior[0].checkpoint=NONE)) {THROW 'expectation previous boundary unavailable';}; CREATE type::record('canonical_expectation_proposals',$row.key) CONTENT $row; };",
                "SELECT * FROM ONLY type::record('canonical_expectation_proposals',$row.key)")?).bind(("row", object.clone())))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectation_proposals).transpose()?.ok_or(CanonicalError::IncompleteResponse)
        })).await
    }
    /// Read one original frozen child definition without resampling its population.
    pub async fn expectation_proposal(
        &self,
        parent: &str,
        ordinal: u64,
    ) -> Result<Option<Proposal>, CanonicalError> {
        key(parent)?;
        let record = format!("{parent}/{ordinal}");
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut response = bounded_query(self.db.script(GuardedScript::new(Envelope::ReadOnly).returning::<Option<Object>>("", "SELECT * FROM ONLY type::record('canonical_expectation_proposals',$key)")?).bind(("key", record))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectation_proposals).transpose().map_err(Into::into)
        })).await
    }
    /// Seal exactly one original child's consumed deltas and its resulting report.
    pub async fn seal_expectation_boundary(
        &self,
        row: Boundary,
    ) -> Result<Boundary, CanonicalError> {
        key(&row.key)?;
        key(&row.parent)?;
        key(&row.child)?;
        document(row.report.as_slice())?;
        document(row.observations.as_slice())?;
        if let Some(checkpoint) = &row.checkpoint {
            document(checkpoint.as_slice())?;
        }
        if row.key != format!("{}/{}", row.parent, row.ordinal) {
            return Err(CanonicalError::PayloadLimit);
        }
        let object = wire::create_canonical_expectation_boundaries(&(&row).into())?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            self.ensure_writes()?;
            let mut response = protected_query("canonical_expectations::boundary", || Ok(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Option<Object>>(
                "LET $proposal=SELECT * FROM ONLY type::record('canonical_expectation_proposals',$row.key); LET $child=SELECT * FROM ONLY type::record('canonical_studies',$row.child); IF $proposal=NONE OR $proposal.parent!=$row.parent OR $proposal.child!=$row.child OR $child=NONE OR !$child.terminal {THROW 'expectation original child is not terminal';}; LET $old=SELECT * FROM ONLY type::record('canonical_expectation_boundaries',$row.key); IF $old!=NONE AND ($old.parent!=$row.parent OR $old.ordinal!=$row.ordinal OR $old.child!=$row.child OR $old.checkpoint!=$row.checkpoint OR $old.report!=$row.report OR $old.observations!=$row.observations) {THROW 'expectation original boundary differs';}; IF $old=NONE {LET $prior=SELECT ordinal FROM canonical_expectation_boundaries WHERE parent=$row.parent ORDER BY ordinal DESC LIMIT 1; IF ($row.ordinal=0 AND array::len($prior)!=0) OR ($row.ordinal>0 AND (array::len($prior)!=1 OR $prior[0].ordinal+1!=$row.ordinal)) {THROW 'expectation boundary order differs';}; CREATE type::record('canonical_expectation_boundaries',$row.key) CONTENT $row;};",
                "SELECT * FROM ONLY type::record('canonical_expectation_boundaries',$row.key)")?).bind(("row", object.clone())))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectation_boundaries).transpose()?.ok_or(CanonicalError::IncompleteResponse)
        })).await
    }
    /// Read the latest immutable sealed checkpoint/report; no library state is reconstructed.
    pub async fn expectation_boundary(
        &self,
        parent: &str,
    ) -> Result<Option<Boundary>, CanonicalError> {
        key(parent)?;
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut response = bounded_query(self.db.script(GuardedScript::new(Envelope::ReadOnly).returning::<Option<Object>>("", "(SELECT * FROM canonical_expectation_boundaries WHERE parent=$parent ORDER BY ordinal DESC LIMIT 1)[0]")?).bind(("parent", parent.to_owned()))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectation_boundaries).transpose().map_err(Into::into)
        })).await
    }
    /// Read one exact original sealed iteration without selecting a later boundary.
    pub async fn expectation_boundary_at(
        &self,
        parent: &str,
        ordinal: u64,
    ) -> Result<Option<Boundary>, CanonicalError> {
        key(parent)?;
        let record = format!("{parent}/{ordinal}");
        let activity = self.admit_activity()?;
        activity.scope(within_clock(original_deadline(REQUEST_TIMEOUT), async {
            let mut response = bounded_query(self.db.script(GuardedScript::new(Envelope::ReadOnly).returning::<Option<Object>>("", "SELECT * FROM ONLY type::record('canonical_expectation_boundaries',$key)")?).bind(("key", record))).await?;
            ResultSlot::<Option<Object>>::returned().take(&mut response)?.map(wire::decode_canonical_expectation_boundaries).transpose().map_err(Into::into)
        })).await
    }
}
