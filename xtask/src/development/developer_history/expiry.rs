// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact owner expiry for one positively selected developer history publication.

use super::*;

/// Compact proof of removing one exact published row.
#[derive(Clone, Debug, PartialEq, Eq, SurrealValue)]
pub struct RetirementReceipt {
    /// Original native invocation key.
    pub invocation: String,
    /// Exact removed content and expiry commitment.
    pub content_sha256: String,
    /// Original selected expiry.
    pub expires_unix_ms: i64,
    /// Database time at retirement in Unix microseconds.
    pub retired_at_unix_us: i64,
    /// Latest permissible expiry of any publication of this same original invocation.
    pub publication_closes_unix_ms: i64,
}

impl RetirementReceipt {
    pub(super) fn verify(&self, invocation: &str) -> Result<(), HistoryError> {
        if self.invocation != invocation
            || !lower_hex(&self.content_sha256, 64)
            || self.expires_unix_ms <= 0
            || self.retired_at_unix_us / 1_000 < self.expires_unix_ms
            || self.publication_closes_unix_ms < self.expires_unix_ms
        {
            return Err(HistoryError::Invalid("stored retirement receipt"));
        }
        Ok(())
    }
}

#[derive(SurrealValue)]
struct ActiveExpiry {
    invocation: String,
    content_sha256: String,
    expires_unix_ms: i64,
    hold: Option<surrealdb_types::Uuid>,
}

#[derive(SurrealValue)]
struct ExpiryState {
    deployment: Option<Deployment>,
    active: Option<ActiveExpiry>,
    retirement: Option<RetirementReceipt>,
    now_unix_us: i64,
}

impl ExpiryState {
    fn receipt(&self, candidate: &ExpiryCandidate) -> Result<Option<&RetirementReceipt>, HistoryError> {
        let Some(receipt) = &self.retirement else {
            return Ok(None);
        };
        if receipt.content_sha256 != candidate.content_sha256
            || receipt.expires_unix_ms != candidate.expires_unix_ms
        {
            return Err(HistoryError::StaleExpiry);
        }
        Ok(Some(receipt))
    }

    fn require_due(&self, candidate: &ExpiryCandidate) -> Result<(), HistoryError> {
        let live = self.active.as_ref().ok_or(HistoryError::StaleExpiry)?;
        if live.invocation != candidate.invocation
            || live.content_sha256 != candidate.content_sha256
            || live.expires_unix_ms != candidate.expires_unix_ms
        {
            return Err(HistoryError::StaleExpiry);
        }
        if live.hold.is_some() {
            return Err(HistoryError::ExpiryHeld);
        }
        if live.expires_unix_ms > self.now_unix_us / 1_000 {
            return Err(HistoryError::ExpiryNotDue);
        }
        Ok(())
    }
}

const OBSERVE_EXPIRY: &str = "
    RETURN {
        deployment: (SELECT issuer, schema_sha256 FROM ONLY developer_history_deployment:current),
        active: (SELECT fact.invocation AS invocation, content_sha256, expires_unix_ms, hold
            FROM ONLY type::record('developer_test_history', $invocation)),
        retirement: (SELECT invocation, content_sha256, expires_unix_ms,
            retired_at_unix_us, publication_closes_unix_ms
            FROM ONLY type::record('developer_test_history_retired', $invocation)),
        now_unix_us: time::micros()
    };";

const RETIRE_EXACT: &str = "
    BEGIN;
    LET $marker = SELECT issuer, schema_sha256 FROM ONLY developer_history_deployment:current;
    IF $marker = NONE OR $marker.issuer != $issuer OR $marker.schema_sha256 != $schema_sha256 {
        THROW 'developer history deployment changed';
    };
    LET $active_id = type::record('developer_test_history', $expected.invocation);
    LET $retired_id = type::record('developer_test_history_retired', $expected.invocation);
    LET $prior = SELECT invocation, content_sha256, expires_unix_ms FROM ONLY $retired_id;
    IF $prior = NONE {
        LET $current = SELECT fact, content_sha256, expires_unix_ms, hold FROM ONLY $active_id FOR UPDATE;
        IF $current = NONE OR $current.fact.invocation != $expected.invocation
            OR $current.content_sha256 != $expected.content_sha256
            OR $current.expires_unix_ms != $expected.expires_unix_ms {
            THROW 'developer history expiry selection changed';
        };
        IF $current.hold != NONE {
            THROW 'developer history expiry is held';
        };
        LET $now = time::micros();
        IF $current.expires_unix_ms > $now / 1000 {
            THROW 'developer history expiry is not due';
        };
        CREATE ONLY $retired_id SET
            invocation = $expected.invocation,
            content_sha256 = $expected.content_sha256,
            expires_unix_ms = $expected.expires_unix_ms,
            retired_at_unix_us = $now,
            publication_closes_unix_ms = $current.fact.created_unix_ms + $maximum_lifetime_ms
            RETURN NONE;
        DELETE ONLY $active_id RETURN NONE;
    } ELSE {
        IF $prior.invocation != $expected.invocation
            OR $prior.content_sha256 != $expected.content_sha256
            OR $prior.expires_unix_ms != $expected.expires_unix_ms
            OR (SELECT id FROM ONLY $active_id) != NONE {
            THROW 'developer history retirement differs';
        };
    };
    COMMIT;";

impl HistoryStore {
    async fn expiry_state(&self, invocation: &str) -> Result<ExpiryState, HistoryError> {
        let mut response = transport::bounded_query(
            self.query(OBSERVE_EXPIRY)?.bind(("invocation", invocation)),
        )
        .await?;
        let state = response
            .take::<Option<ExpiryState>>(0)?
            .ok_or(CanonicalError::IncompleteResponse)?;
        self.scope.verify(state.deployment.as_ref())?;
        if let Some(receipt) = &state.retirement {
            receipt.verify(invocation)?;
            if state.active.is_some() {
                return Err(HistoryError::Invalid("live history and retirement both exist"));
            }
        }
        Ok(state)
    }

    /// Retire one exact, due and unheld publication through an independent reader.
    ///
    /// An uncertain acknowledgement settles only through a matching receipt.
    /// This operation never releases raw files, build state or another owner's holds.
    pub async fn retire(
        &self,
        reader: &Self,
        candidate: &ExpiryCandidate,
        deadline: Instant,
    ) -> Result<RetirementReceipt, HistoryError> {
        self.require_writer_reader(reader)?;
        let invocation = candidate.invocation.as_str();
        if !uuid(invocation)
            || !lower_hex(&candidate.content_sha256, 64)
            || candidate.expires_unix_ms <= 0
        {
            return Err(HistoryError::Invalid("expiry selection"));
        }
        transport::operation_scope(deadline, async {
            let before = reader.expiry_state(invocation).await?;
            if let Some(receipt) = before.receipt(candidate)? {
                return Ok(receipt.clone());
            }
            before.require_due(candidate)?;
            let acknowledgement = transport::protected_query("developer_history::retire", || {
                Ok(self
                    .query(RETIRE_EXACT)?
                    .bind(("issuer", self.scope.issuer.as_str()))
                    .bind(("schema_sha256", self.scope.schema_sha256.as_str()))
                    .bind(("maximum_lifetime_ms", PILOT_MAX_LIFETIME_MS))
                    .bind(("expected", candidate.clone())))
            })
            .await;
            #[cfg(test)]
            let acknowledgement = self.fault_ack(reader, acknowledgement).await;
            let after = match reader.expiry_state(invocation).await {
                Ok(after) => after,
                Err(readback) => {
                    return Err(unsettled("expiry", invocation, acknowledgement, Some(readback)));
                }
            };
            if let Some(receipt) = after.receipt(candidate)? {
                return Ok(receipt.clone());
            }
            match acknowledgement {
                Err(error) if !error.effect_is_unknown() => {
                    after.require_due(candidate)?;
                    Err(error.into())
                }
                uncertain => Err(unsettled("expiry", invocation, uncertain, None)),
            }
        })
        .await
    }
}
