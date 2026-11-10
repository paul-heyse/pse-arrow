// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Identified compact observations, independent of scientific completion and capture authority.
use super::{CanonicalError, CanonicalStore, StudyWorkbenchHeader, codec, header};
use crate::{
    canonical_script::{Envelope, GuardedScript, PreparedScript},
    transport::client::{Subscription, SubscriptionNotice},
};
use std::time::{Duration, Instant};
use surrealdb_types::{Object, Value};

/// One native grouped count. Operational settlement and scientific usability remain separate.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct StudySummaryGroup {
    /// Original source-owned lifecycle classification.
    pub lifecycle: pse_model::generated::enums::StudyPointState,
    /// A worker currently owns the occurrence.
    pub assigned: bool,
    /// Pure policy has settled this occurrence.
    pub settled: bool,
    /// The owning scientific operation grants usable-result permission.
    pub usable: bool,
    /// Number of currently visible rows in this group.
    pub count: u64,
}
/// Current display summary, never an immutable scientific result or capture fence.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct StudySummary {
    /// Original study/source identity and observed root state.
    pub header: StudyWorkbenchHeader,
    /// Native grouped visible membership, without decoding point payloads.
    pub groups: Vec<StudySummaryGroup>,
    /// UTC microseconds supplied by the same native snapshot.
    pub observed_as_of: i64,
}
/// Current run header, separate from protected result acquisition.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RunSummary {
    /// Original run identity.
    pub run: String,
    /// Current retry/authority generation.
    pub generation: u64,
    /// Original terminal attempt, when published.
    pub terminal_attempt: Option<String>,
    /// Operational terminal class, not scientific usability.
    pub terminal_class: Option<pse_model::generated::enums::CanonicalTerminalOutcome>,
    /// UTC microseconds supplied by the same native snapshot.
    pub observed_as_of: i64,
}
/// Exact stable root identities; mutable display predicates do not narrow notifications.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FollowSelection {
    /// One exact study and its member state.
    Study {
        /// Original study key.
        study: String,
    },
    /// One exact canonical run.
    Run {
        /// Original run key.
        run: String,
    },
}
/// An identified authoritative read, not a notification payload.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", content = "snapshot", rename_all = "snake_case")]
pub enum CurrentSnapshot {
    /// Compact study summary.
    Study(StudySummary),
    /// Compact run header.
    Run(RunSummary),
}
/// Current-state delivery is explicitly different from a durable transition history.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FollowUpdate {
    /// An as-of snapshot, with continuing authority checked at the next bounded deadline.
    Snapshot {
        /// Identified current read.
        snapshot: CurrentSnapshot,
        /// A notification arrived during this read.
        refresh_pending: bool,
        /// Advisory next check; not a revocation-proof authority lease.
        next_revalidation_at: i64,
    },
    /// Continuity/authority must be re-established before another current snapshot.
    Unavailable,
    /// The caller's original observation window ended or was explicitly closed.
    Ended,
}
/// A follow retains one original store/activity and dedicated fixed-principal channels.
pub struct CurrentFollow {
    store: CanonicalStore,
    selection: FollowSelection,
    subscriptions: Vec<Subscription>,
    linked_run: Option<String>,
    deadline: Instant,
    next_check: Instant,
    initial: bool,
    ended: bool,
    #[cfg(test)]
    snapshot_pause_until: Option<Instant>,
    #[cfg(test)]
    close_dependency_after_read: Option<usize>,
}
impl std::fmt::Debug for CurrentFollow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CurrentFollow")
            .field("selection", &self.selection)
            .field("deadline", &self.deadline)
            .field("ended", &self.ended)
            .finish_non_exhaustive()
    }
}
fn validate(key: &str) -> Result<(), CanonicalError> {
    if key.is_empty() || key.len() > 4096 {
        Err(CanonicalError::PayloadLimit)
    } else {
        Ok(())
    }
}
fn script(selection: &FollowSelection) -> Result<PreparedScript<Option<Object>>, CanonicalError> {
    let (body, value) = match selection {
        FollowSelection::Study { .. } => (
            "LET $root=SELECT key,run,problem,revision,physical_revision,physical_identity,context,point_count,generation,active,cancelled,terminal FROM ONLY type::record('canonical_studies',$root); LET $run=IF $root=NONE {NONE} ELSE {SELECT terminal_attempt,terminal_class FROM ONLY type::record('canonical_runs',$root.run)}; LET $groups=SELECT facts.lifecycle AS lifecycle,assigned,settled,facts.scientific.usable AS usable,count() AS count FROM canonical_study_points WHERE study=$root.key GROUP BY facts.lifecycle,assigned,settled,facts.scientific.usable;",
            "IF $root=NONE {NONE} ELSE {{header:object::extend($root,{result_attempt:$run.terminal_attempt,outcome:$run.terminal_class}),groups:$groups,observed_as_of:time::micros()}}",
        ),
        FollowSelection::Run { .. } => (
            "LET $root=SELECT key,current_generation,terminal_attempt,terminal_class FROM ONLY type::record('canonical_runs',$root);",
            "IF $root=NONE {NONE} ELSE {{run:$root.key,generation:$root.current_generation,terminal_attempt:$root.terminal_attempt,terminal_class:$root.terminal_class,observed_as_of:time::micros()}}",
        ),
    };
    GuardedScript::new(Envelope::ReadOnly).returning(body, value)
}
fn key(selection: &FollowSelection) -> &str {
    match selection {
        FollowSelection::Study { study } => study,
        FollowSelection::Run { run } => run,
    }
}
fn optional(row: &mut Object, key: &str) -> Result<Option<String>, CanonicalError> {
    match row.remove(key) {
        None | Some(Value::None | Value::Null) => Ok(None),
        Some(value) => codec::decode_string(value).map(Some).map_err(Into::into),
    }
}
fn decode(selection: &FollowSelection, mut row: Object) -> Result<CurrentSnapshot, CanonicalError> {
    let observed_as_of = match codec::required(&mut row, "observed_as_of")? {
        Value::Number(surrealdb_types::Number::Int(value)) => value,
        _ => return Err(CanonicalError::IncompleteResponse),
    };
    Ok(match selection {
        FollowSelection::Study { .. } => {
            let Value::Object(root) = codec::required(&mut row, "header")? else {
                return Err(CanonicalError::IncompleteResponse);
            };
            let Value::Array(groups) = codec::required(&mut row, "groups")? else {
                return Err(CanonicalError::IncompleteResponse);
            };
            if groups.len() > pse_model::generated::enums::StudyPointState::ALL.len() * 8 {
                return Err(CanonicalError::PayloadLimit);
            }
            let mut decoded = Vec::with_capacity(groups.len());
            for group in groups {
                let Value::Object(mut group) = group else {
                    return Err(CanonicalError::IncompleteResponse);
                };
                decoded.push(StudySummaryGroup {
                    lifecycle: codec::decode_string(codec::required(&mut group, "lifecycle")?)?
                        .parse()
                        .map_err(|_| CanonicalError::IncompleteResponse)?,
                    assigned: codec::decode_boolean(codec::required(&mut group, "assigned")?)?,
                    settled: codec::decode_boolean(codec::required(&mut group, "settled")?)?,
                    usable: codec::decode_boolean(codec::required(&mut group, "usable")?)?,
                    count: codec::decode_uint(codec::required(&mut group, "count")?)?,
                });
            }
            CurrentSnapshot::Study(StudySummary {
                header: header(root)?,
                groups: decoded,
                observed_as_of,
            })
        }
        FollowSelection::Run { .. } => CurrentSnapshot::Run(RunSummary {
            run: codec::decode_string(codec::required(&mut row, "run")?)?,
            generation: codec::decode_uint(codec::required(&mut row, "generation")?)?,
            terminal_attempt: optional(&mut row, "terminal_attempt")?,
            terminal_class: optional(&mut row, "terminal_class")?
                .map(|value| value.parse())
                .transpose()
                .map_err(|_| CanonicalError::IncompleteResponse)?,
            observed_as_of,
        }),
    })
}
impl CanonicalStore {
    /// Read one compact current study summary in one native readonly snapshot.
    pub async fn study_summary(&self, study: &str) -> Result<Option<StudySummary>, CanonicalError> {
        let selection = FollowSelection::Study {
            study: study.to_owned(),
        };
        match self.current_snapshot(&selection).await? {
            Some(CurrentSnapshot::Study(summary)) => Ok(Some(summary)),
            None => Ok(None),
            _ => Err(CanonicalError::IncompleteResponse),
        }
    }
    /// Read one compact current run header without acquiring its result payload.
    pub async fn run_summary(&self, run: &str) -> Result<Option<RunSummary>, CanonicalError> {
        let selection = FollowSelection::Run {
            run: run.to_owned(),
        };
        match self.current_snapshot(&selection).await? {
            Some(CurrentSnapshot::Run(summary)) => Ok(Some(summary)),
            None => Ok(None),
            _ => Err(CanonicalError::IncompleteResponse),
        }
    }
    async fn current_snapshot(
        &self,
        selection: &FollowSelection,
    ) -> Result<Option<CurrentSnapshot>, CanonicalError> {
        validate(key(selection))?;
        let activity = self.admit_activity()?;
        activity
            .scope(crate::transport::within_clock(
                crate::transport::original_deadline(crate::canonical::REQUEST_TIMEOUT),
                async {
                    let prepared = script(selection)?;
                    let result = prepared.result();
                    let mut response = crate::canonical::bounded_query(
                        self.db
                            .metadata_script(
                                crate::canonical::MetadataResponse::StudySummary,
                                prepared,
                            )
                            .bind(("root", key(selection).to_owned())),
                    )
                    .await?;
                    result
                        .take(&mut response)?
                        .map(|row| decode(selection, row))
                        .transpose()
                },
            ))
            .await
    }
    /// Register before the initial read. The original deadline bounds every refresh/reconnect.
    pub async fn follow_current(
        &self,
        selection: FollowSelection,
        deadline: Instant,
    ) -> Result<CurrentFollow, CanonicalError> {
        validate(key(&selection))?;
        let mut follow = CurrentFollow {
            store: self.activity_store()?,
            selection,
            subscriptions: Vec::new(),
            linked_run: None,
            deadline,
            next_check: Instant::now(),
            initial: true,
            ended: false,
            #[cfg(test)]
            snapshot_pause_until: None,
            #[cfg(test)]
            close_dependency_after_read: None,
        };
        if let Err(error) = follow.register().await {
            if let Err(cleanup) = follow.withdraw().await {
                return Err(surrealdb_types::Error::connection(format!("Follow registration failed: {error}; attached registration cleanup failed: {cleanup}"),None).into());
            }
            return Err(error);
        }
        Ok(follow)
    }
}
impl CurrentFollow {
    #[cfg(test)]
    pub(crate) fn lose_linked_run_after_next_read(&mut self) {
        self.close_dependency_after_read = Some(2);
    }
    #[cfg(test)]
    pub(crate) fn cross_principal_boundary_after_next_read(
        &mut self,
        principal: Instant,
        pause: Instant,
    ) {
        self.next_check = principal;
        self.snapshot_pause_until = Some(pause);
    }
    async fn register(&mut self) -> Result<(), CanonicalError> {
        let started = Instant::now();
        let queries = match &self.selection {
            FollowSelection::Study { .. } => vec![
                "LIVE SELECT VALUE id FROM canonical_studies WHERE key=$root;",
                "LIVE SELECT VALUE id FROM canonical_study_points WHERE study=$root;",
            ],
            FollowSelection::Run { .. } => vec![
                "LIVE SELECT VALUE id FROM canonical_runs WHERE key=$root;",
                "LIVE SELECT VALUE id FROM canonical_attempts WHERE run=$root;",
            ],
        };
        for query in queries {
            let mut variables = Object::new();
            variables.insert("root", key(&self.selection).to_owned());
            match self
                .store
                .db
                .subscribe(query.to_owned(), variables, self.deadline)
                .await
            {
                Ok(subscription) => self.subscriptions.push(subscription),
                Err(error) => return Err(error.into()),
            }
        }
        // Revalidation is anchored to fresh authentication, never to a noisy read.
        self.next_check = (started + Duration::from_secs(30)).min(self.deadline);
        Ok(())
    }
    async fn synchronize_run_subscription(&mut self) -> Result<(), CanonicalError> {
        let FollowSelection::Study { study } = &self.selection else {
            return Ok(());
        };
        let prepared = GuardedScript::new(Envelope::ReadOnly).returning::<Option<String>>(
            "LET $row=SELECT run FROM ONLY type::record('canonical_studies',$root);",
            "$row.run",
        )?;
        let slot = prepared.result();
        let read_deadline = self.deadline.min(self.next_check);
        let mut response = crate::transport::within_clock(read_deadline.into(), async {
            let query = self
                .store
                .db
                .observation_script(
                    &self.subscriptions[0],
                    crate::canonical::MetadataResponse::Header,
                    prepared,
                )
                .bind(("root", study.clone()));
            crate::canonical::bounded_query(query).await
        })
        .await?;
        if Instant::now() >= read_deadline {
            return Err(CanonicalError::Timeout);
        }
        let run = slot.take(&mut response)?;
        if run != self.linked_run {
            if self.linked_run.is_some() {
                return Err(CanonicalError::IncompleteResponse);
            }
            self.linked_run = None;
            if let Some(run) = run {
                let mut variables = Object::new();
                variables.insert("root", run.clone());
                self.subscriptions.push(
                    self.store
                        .db
                        .subscribe(
                            "LIVE SELECT VALUE id FROM canonical_runs WHERE key=$root;".to_owned(),
                            variables,
                            self.deadline,
                        )
                        .await?,
                );
                self.linked_run = Some(run);
            }
        }
        Ok(())
    }
    /// Obtain the initial snapshot or the next coalesced invalidation/authority recheck.
    /// # Errors
    /// Native shape/admission failures preserve an unavailable state, never a stale success.
    pub async fn next(&mut self) -> Result<FollowUpdate, CanonicalError> {
        if self.ended {
            return Ok(FollowUpdate::Ended);
        }
        if Instant::now() >= self.deadline {
            self.close().await?;
            return Ok(FollowUpdate::Ended);
        }
        if !self.initial && self.subscriptions.is_empty() {
            tokio::time::sleep_until(self.next_check.into()).await;
        }
        if !self.initial && !self.subscriptions.is_empty() {
            let wait = futures_util::future::select_all(
                self.subscriptions
                    .iter_mut()
                    .map(|subscription| Box::pin(subscription.changed())),
            );
            tokio::select! {_=tokio::time::sleep_until(self.next_check.into())=>{},_ =wait=>{}};
        }
        self.initial = false;
        if Instant::now() >= self.deadline {
            self.close().await?;
            return Ok(FollowUpdate::Ended);
        }
        if Instant::now() >= self.next_check
            || self.subscriptions.is_empty()
            || self.subscriptions.iter().any(|subscription| {
                !matches!(subscription.notice(), SubscriptionNotice::Active { .. })
            })
        {
            self.withdraw().await?;
            if self.register().await.is_err() {
                self.withdraw().await?;
                self.next_check = (Instant::now() + Duration::from_secs(30)).min(self.deadline);
                return Ok(FollowUpdate::Unavailable);
            }
        }
        if self.synchronize_run_subscription().await.is_err() {
            self.withdraw().await?;
            self.next_check = if Instant::now() >= self.next_check {
                Instant::now()
            } else {
                (Instant::now() + Duration::from_secs(30)).min(self.deadline)
            };
            return Ok(FollowUpdate::Unavailable);
        }
        let before = self
            .subscriptions
            .iter_mut()
            .map(Subscription::acknowledge)
            .collect::<Vec<_>>();
        let prepared = script(&self.selection)?;
        let result = prepared.result();
        let read_deadline = self.deadline.min(self.next_check);
        let snapshot = match crate::transport::within_clock(read_deadline.into(), async {
            let query = self
                .store
                .db
                .observation_script(
                    &self.subscriptions[0],
                    crate::canonical::MetadataResponse::StudySummary,
                    prepared,
                )
                .bind(("root", key(&self.selection).to_owned()));
            crate::canonical::bounded_query(query).await
        })
        .await
        {
            Ok(mut response) => result
                .take(&mut response)?
                .map(|row| decode(&self.selection, row))
                .transpose()?,
            Err(_) => {
                self.withdraw().await?;
                None
            }
        };
        // A source link changed while its exact dependency was being registered.
        // Withdraw currentness; the next registration/read reconstructs the cut.
        if matches!(&snapshot,Some(CurrentSnapshot::Study(summary)) if Some(&summary.header.run)!=self.linked_run.as_ref())
        {
            self.withdraw().await?;
            self.next_check = Instant::now();
            return Ok(FollowUpdate::Unavailable);
        }
        let next_revalidation_at = i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .saturating_add(self.next_check.saturating_duration_since(Instant::now()))
                .as_micros(),
        )
        .map_err(|_| CanonicalError::PayloadLimit)?;
        #[cfg(test)]
        if let Some(until) = self.snapshot_pause_until.take() {
            tokio::time::sleep_until(until.min(self.deadline).into()).await;
        }
        #[cfg(test)]
        if let Some(index) = self.close_dependency_after_read.take() {
            self.subscriptions
                .get_mut(index)
                .ok_or(CanonicalError::IncompleteResponse)?
                .close()
                .await?;
        }
        // A completed read is still tentative across the principal boundary.
        // Check after typed decoding and attribution, immediately before publication.
        if Instant::now() >= read_deadline {
            self.withdraw().await?;
            self.next_check = Instant::now();
            return Ok(FollowUpdate::Unavailable);
        }
        let after = self
            .subscriptions
            .iter()
            .map(Subscription::notice)
            .collect::<Vec<_>>();
        if after
            .iter()
            .any(|notice| !matches!(notice, SubscriptionNotice::Active { .. }))
        {
            self.withdraw().await?;
            self.next_check = Instant::now();
            return Ok(FollowUpdate::Unavailable);
        }
        Ok(match snapshot {
            Some(snapshot) => FollowUpdate::Snapshot {
                snapshot,
                refresh_pending: before != after,
                next_revalidation_at,
            },
            None => FollowUpdate::Unavailable,
        })
    }
    async fn withdraw(&mut self) -> Result<(), CanonicalError> {
        let mut failures = Vec::new();
        for (index, subscription) in self.subscriptions.iter_mut().enumerate() {
            if let Err(error) = subscription.close().await {
                failures.push((index, error));
            }
        }
        self.subscriptions.clear();
        self.linked_run = None;
        if failures.len() == 1 {
            return Err(failures.remove(0).1.into());
        }
        if !failures.is_empty() {
            return Err(surrealdb_types::Error::connection(
                format!(
                    "Follow cleanup failed: {}",
                    failures
                        .into_iter()
                        .map(|(index, error)| format!("registration {index}: {error}"))
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
                None,
            )
            .into());
        }
        Ok(())
    }
    /// Explicitly kill every registration and join all dedicated native generations.
    pub async fn close(&mut self) -> Result<(), CanonicalError> {
        self.ended = true;
        self.withdraw().await
    }
}
