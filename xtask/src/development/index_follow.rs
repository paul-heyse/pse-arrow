// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Current imported-index metadata, never a source-file watcher or transition journal.
use super::*;
use crate::generated::store::knowledge as knowledge_wire;
use pse_operations::transport::client::{
    Credentials as NativeCredentials, Database, Subscription, SubscriptionNotice,
};

/// Independently owned imported index selected for observation.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IndexFollowFamily {
    /// Original selected evidence population.
    Evidence,
    /// Owner-attributed development knowledge.
    Knowledge,
}
/// A stable current pointer or exact generation, with no arbitrary live predicate.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IndexFollowSelection {
    /// Selected independently disposable index.
    pub family: IndexFollowFamily,
    /// Exact generation; absence follows that family's current pointer.
    pub generation: Option<String>,
}
/// Bounded current-state metadata with explicit temporal/continuity interpretation.
#[derive(Debug, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum IndexFollowEvent {
    /// One fully received readonly as-of snapshot.
    Snapshot {
        /// Exact service/index issuer.
        issuing_store: String,
        /// Original followed root.
        selection: IndexFollowSelection,
        /// Unix observation time in microseconds; not scientific effective time.
        observed_at_micros: u64,
        /// Maximum scheduled compact metadata recheck time, not an authority lease.
        next_revalidation_at_micros: u64,
        /// An invalidation arrived during this read; another refresh is pending.
        refresh_pending: bool,
        /// Notifications attest imported index metadata only; original files are not reopened.
        freshness: &'static str,
        /// Compact pointer and generation metadata, never original scientific payloads.
        metadata: serde_json::Value,
    },
    /// Registration or permission continuity needs a fresh snapshot.
    Resynchronizing,
    /// Currentness is withdrawn; neither old metadata nor a quiet socket proves authority.
    Unavailable,
    /// Original observation window ended and the physical subscription settled.
    Closed,
}
/// Actual caller-held fixed-Viewer observation and its original finite window.
pub struct IndexFollow {
    store: EvidenceStore,
    selection: IndexFollowSelection,
    subscription: Option<Subscription>,
    deadline: std::time::Instant,
    refresh: bool,
    recheck: tokio::time::Instant,
    ended: bool,
}
impl std::fmt::Debug for IndexFollow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexFollow")
            .field("selection", &self.selection)
            .field("ended", &self.ended)
            .finish_non_exhaustive()
    }
}
impl EvidenceStore {
    /// Observe imported metadata through a dedicated fixed Viewer; no database is started or initialized.
    pub async fn follow_index(
        &self,
        selection: IndexFollowSelection,
        deadline: std::time::Instant,
    ) -> Result<IndexFollow, EvidenceError> {
        if selection.generation.as_ref().is_some_and(|generation| {
            generation.len() != 64 || !generation.bytes().all(|byte| byte.is_ascii_hexdigit())
        }) {
            return Err(config("invalid exact followed generation"));
        }
        let mut follow = IndexFollow {
            store: self.clone(),
            selection,
            subscription: None,
            deadline,
            refresh: true,
            recheck: tokio::time::Instant::now(),
            ended: false,
        };
        follow.register().await?;
        Ok(follow)
    }
}
impl IndexFollow {
    fn tables(&self) -> (&'static str, &'static str) {
        match self.selection.family {
            IndexFollowFamily::Evidence => ("evidence_interpretations", "evidence_generations"),
            IndexFollowFamily::Knowledge => ("knowledge_state", "knowledge_generations"),
        }
    }
    fn context_at(&self, deadline: std::time::Instant) -> Result<RequestContext, EvidenceError> {
        Ok(RequestContext::new(deadline.min(self.deadline))
            .with_lifetime_owner(self.store.access.admit()?)
            .with_response_limits(canonical::MetadataResponse::Header.limits()))
    }
    async fn register(&mut self) -> Result<(), EvidenceError> {
        let (pointer, generations) = self.tables();
        let (table, key) = match &self.selection.generation {
            Some(generation) => (generations, generation.clone()),
            None => (pointer, "current".into()),
        };
        let mut variables = Object::new();
        variables.insert("key", Value::String(key.into()));
        let options = &self.store.options;
        let credentials = NativeCredentials::Database(Database {
            namespace: options.namespace.clone(),
            database: options.database.clone(),
            username: options.credentials.reader_username.clone(),
            password: options.credentials.reader_password.clone(),
        });
        let principal_check = (tokio::time::Instant::now() + std::time::Duration::from_secs(30))
            .min(tokio::time::Instant::from_std(self.deadline));
        self.subscription = Some(
            self.store
                .db
                .subscribe_fixed_viewer(
                    credentials,
                    options.namespace.clone(),
                    options.database.clone(),
                    format!("LIVE SELECT VALUE id FROM {table} WHERE key=$key"),
                    variables,
                    self.context_at(self.deadline)?,
                )
                .await
                .map_err(CanonicalError::from)?,
        );
        self.refresh = true;
        // A fresh native signin is the principal check: cached Actor roles in an old
        // Session cannot establish current credentials. Notifications never extend it.
        self.recheck = principal_check;
        Ok(())
    }
    /// Next identified snapshot or explicit continuity state; all refreshes use the original window.
    pub async fn next(&mut self) -> Result<IndexFollowEvent, EvidenceError> {
        if self.ended {
            return Ok(IndexFollowEvent::Closed);
        }
        if std::time::Instant::now() >= self.deadline {
            self.close().await?;
            return Ok(IndexFollowEvent::Closed);
        }
        if self.subscription.is_none() {
            if self.register().await.is_err() {
                return Ok(IndexFollowEvent::Unavailable);
            }
            return Ok(IndexFollowEvent::Resynchronizing);
        }
        if !self.refresh {
            let wake = self
                .recheck
                .min(tokio::time::Instant::from_std(self.deadline));
            let sub = self
                .subscription
                .as_mut()
                .ok_or_else(|| config("follow subscription disappeared"))?;
            tokio::select! {_=tokio::time::sleep_until(wake)=>{},_notice=sub.changed()=>{}}
        }
        if std::time::Instant::now() >= self.deadline {
            self.close().await?;
            return Ok(IndexFollowEvent::Closed);
        }
        if tokio::time::Instant::now() >= self.recheck {
            self.withdraw().await?;
            if self.register().await.is_err() {
                return Ok(IndexFollowEvent::Unavailable);
            }
            return Ok(IndexFollowEvent::Resynchronizing);
        }
        let before = self
            .subscription
            .as_mut()
            .ok_or_else(|| config("follow subscription disappeared"))?
            .acknowledge();
        let sub = self
            .subscription
            .as_ref()
            .ok_or_else(|| config("follow subscription disappeared"))?;
        if !matches!(before, SubscriptionNotice::Active { .. }) {
            self.withdraw().await?;
            return Ok(IndexFollowEvent::Unavailable);
        }
        let (pointer, generations) = self.tables();
        let body = format!(
            "LET $marker=SELECT * FROM ONLY evidence_interpretations:current; LET $pointer=SELECT * FROM ONLY {pointer}:current; LET $selected=IF $generation=NONE {{$pointer.active_generation}} ELSE {{$generation}}; LET $generation_row=IF $selected=NONE {{NONE}} ELSE {{SELECT * FROM ONLY type::record('{generations}',$selected)}};"
        );
        let mut vars = Object::new();
        vars.insert(
            "generation",
            self.selection
                .generation
                .clone()
                .map(|value| Value::String(value.into()))
                .unwrap_or(Value::None),
        );
        let result = sub
            .read_snapshot(
                body,
                "{marker:$marker,pointer:$pointer,generation:$generation_row}".into(),
                vars,
                self.context_at(self.recheck.into_std())?,
            )
            .await;
        let Ok(mut result) = result else {
            self.withdraw().await?;
            return Ok(IndexFollowEvent::Unavailable);
        };
        let metadata: Value = result.take(0).map_err(CanonicalError::from)?;
        let Value::Object(ref object) = metadata else {
            self.withdraw().await?;
            return Ok(IndexFollowEvent::Unavailable);
        };
        if self.validate_snapshot(object).is_err() {
            self.withdraw().await?;
            return Ok(IndexFollowEvent::Unavailable);
        }
        let after = sub.notice();
        if !matches!(after, SubscriptionNotice::Active { .. }) {
            self.withdraw().await?;
            return Ok(IndexFollowEvent::Unavailable);
        }
        let pending = after != before;
        if std::time::Instant::now() >= self.deadline {
            self.close().await?;
            return Ok(IndexFollowEvent::Closed);
        }
        if tokio::time::Instant::now() >= self.recheck {
            self.withdraw().await?;
            if self.register().await.is_err() {
                return Ok(IndexFollowEvent::Unavailable);
            }
            return Ok(IndexFollowEvent::Resynchronizing);
        }
        self.refresh = pending;
        let interval = self
            .recheck
            .saturating_duration_since(tokio::time::Instant::now());
        let micros = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| config(error.to_string()))?
            .as_micros();
        let observed =
            u64::try_from(micros).map_err(|_| config("observation timestamp overflow"))?;
        Ok(IndexFollowEvent::Snapshot {
            issuing_store: self.store.issuing_store.clone(),
            selection: self.selection.clone(),
            observed_at_micros: observed,
            next_revalidation_at_micros: observed
                .saturating_add(u64::try_from(interval.as_micros()).unwrap_or(u64::MAX)),
            refresh_pending: self.refresh,
            freshness: "imported-cut-only",
            metadata: metadata.into_json_value(),
        })
    }
    fn validate_snapshot(&self, object: &Object) -> Result<(), EvidenceError> {
        let Some(Value::Object(marker)) = object.get("marker") else {
            return Err(config("follow marker absent"));
        };
        self.store.decode_marker(marker.clone(), true)?;
        let Some(Value::Object(pointer)) = object.get("pointer") else {
            return Err(config("follow pointer absent"));
        };
        let selected = match self.selection.family {
            IndexFollowFamily::Evidence => {
                self.store
                    .decode_marker(pointer.clone(), true)?
                    .active_generation
            }
            IndexFollowFamily::Knowledge => {
                knowledge_wire::decode_knowledge_state(pointer.clone())?.active_generation
            }
        };
        let expected = self.selection.generation.as_ref().or(selected.as_ref());
        match (expected, object.get("generation")) {
            (None, Some(Value::None)) => Ok(()),
            (Some(expected), Some(Value::Object(generation))) => {
                let key = match self.selection.family {
                    IndexFollowFamily::Evidence => {
                        wire::decode_evidence_generations(generation.clone())?.key
                    }
                    IndexFollowFamily::Knowledge => {
                        knowledge_wire::decode_knowledge_generations(generation.clone())?.key
                    }
                };
                if key != *expected {
                    return Err(config("follow generation identity differs"));
                }
                Ok(())
            }
            _ => Err(config("follow selected generation absent")),
        }
    }
    async fn withdraw(&mut self) -> Result<(), EvidenceError> {
        let outcome = match self.subscription.as_mut() {
            Some(subscription) => subscription.close().await.map_err(CanonicalError::from),
            None => Ok(()),
        };
        // Await while retaining the physical owner (including cancellation), then clear
        // the settled failed generation even when its original continuity result was Err.
        self.subscription = None;
        self.refresh = true;
        outcome.map_err(Into::into)
    }
    #[cfg(test)]
    pub(super) fn revalidate_principal_now(&mut self) {
        self.recheck = tokio::time::Instant::now();
    }
    /// Kill the original registration and await physical generation settlement.
    pub async fn close(&mut self) -> Result<(), EvidenceError> {
        self.withdraw().await?;
        self.ended = true;
        Ok(())
    }
}
