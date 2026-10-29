// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit guarded envelopes own transaction framing, authority binding and extraction slots.
use crate::canonical::{CanonicalError, ProtectedSelection};
use crate::canonical_codec;
use pse_model::generated::canonical::{OperationKey, ProblemKey, RevisionKey};
use std::{marker::PhantomData, time::Duration};
use surrealdb_types::{Object, SurrealValue, Value};

/// Read postlude must retain its original pin-versus-ticket meaning.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Recheck {
    Pin,
    PinOrTicket,
}
/// Stage mutations, close and cleanup retain distinct native guards and renewal policies.
#[derive(Clone, Copy, Debug)]
pub(crate) enum StagePolicy {
    Mutation,
    Close,
}
/// Explicit actual stage identity/fence consumed by the existing server guards.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StageAuthority<'a> {
    pub(crate) problem: &'a ProblemKey,
    pub(crate) operation: &'a OperationKey,
    pub(crate) generation: u64,
    pub(crate) expected: Option<&'a RevisionKey>,
    pub(crate) request_digest: &'a str,
    pub(crate) lifetime: Duration,
    pub(crate) policy: StagePolicy,
}
/// Original acquisition intent before a stage generation has been issued.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StageAcquisition<'a> {
    pub(crate) problem: &'a ProblemKey,
    pub(crate) operation: &'a OperationKey,
    pub(crate) expected: Option<&'a RevisionKey>,
    pub(crate) request_digest: &'a str,
    pub(crate) lifetime: Duration,
}
/// Existing cleanup phases place the generation fence on different sides of their body.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CleanupFence {
    BeforeBody,
    AfterBody,
}
/// Cleanup identifies an existing stage without fabricating a current lease generation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StageCleanup<'a> {
    pub(crate) problem: &'a ProblemKey,
    pub(crate) operation: &'a OperationKey,
    pub(crate) fence: CleanupFence,
}
/// No default authority. Attribution never constructs one of these guards.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Envelope<'a> {
    ProtectedRead {
        selection: &'a ProtectedSelection,
        recheck: Recheck,
    },
    ProtectedAdmission {
        selection: &'a ProtectedSelection,
    },
    ProjectAuthority {
        issuer: &'a str,
    },
    StageGuard(StageAuthority<'a>),
    StageAcquisition(StageAcquisition<'a>),
    StageCleanup(StageCleanup<'a>),
    /// Bootstrap cannot call a native deadline function before installing it.
    Installation,
    DeadlineOnly,
    /// Original custody release only; no new work may be created during migration drain.
    DrainCleanup,
}
/// One result expression, emitted after the owning postlude and before COMMIT.
#[derive(Clone, Debug)]
pub(crate) struct GuardedScript<'a> {
    envelope: Envelope<'a>,
}
/// Complete script plus typed authority bindings and independently retained pin custody.
#[derive(Clone, Debug)]
pub(crate) struct PreparedScript<T> {
    sql: String,
    bindings: Object,
    selection: Option<ProtectedSelection>,
    access: crate::canonical::migration::SchemaAdmission,
    installation: bool,
    result: ResultSlot<T>,
}
/// Own the result position rather than repeating tail arithmetic at every consumer.
#[derive(Debug)]
pub(crate) struct ResultSlot<T>(PhantomData<fn() -> T>);
impl<T> Clone for ResultSlot<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ResultSlot<T> {}
impl<T> PreparedScript<T> {
    pub(crate) fn result(&self) -> ResultSlot<T> {
        self.result
    }
    pub(crate) fn schema_policy(&self) -> (crate::canonical::migration::SchemaAdmission, bool) { (self.access, self.installation) }
    pub(crate) fn into_parts(self) -> (String, Object, Option<ProtectedSelection>) {
        (self.sql, self.bindings, self.selection)
    }
    /// Pace only an already typed installation fixture, preserving every authority binding.
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) fn test_delay_installation(mut self, delay: Duration) -> Result<Self, CanonicalError> {
        if !self.installation { return Err(CanonicalError::Configuration("only installation fixtures may delay their original transaction".into())); }
        self.sql.insert_str("BEGIN;".len(), &format!(" SLEEP {}ms;",delay.as_millis()));
        Ok(self)
    }
}
/// The adapter changes with the client realization; envelope consumers keep typed extraction.
pub(crate) trait ScriptResponse {
    fn take_script_result<T: SurrealValue>(&mut self) -> Result<T, CanonicalError>
    where
        usize: crate::transport::client::QueryResult<T>;
}
impl ScriptResponse for crate::transport::client::IndexedResults {
    fn take_script_result<T: SurrealValue>(&mut self) -> Result<T, CanonicalError>
    where
        usize: crate::transport::client::QueryResult<T>,
    {
        let slot = self
            .num_statements()
            .checked_sub(2)
            .ok_or(CanonicalError::IncompleteResponse)?;
        self.take(slot).map_err(Into::into)
    }
}
impl<T> ResultSlot<T> {
    /// Marker for a central decoder of this closed envelope's final RETURN. Grants no
    /// authority and requires the original complete response before any slot is consumed.
    pub(crate) const fn returned() -> Self {
        Self(PhantomData)
    }
}
impl<T: SurrealValue> ResultSlot<T>
where
    usize: crate::transport::client::QueryResult<T>,
{
    pub(crate) fn take(self, response: &mut impl ScriptResponse) -> Result<T, CanonicalError> {
        response.take_script_result()
    }
}
/// Result of a source-owned unframed metadata query; it carries no execution authority.
#[derive(Debug)]
pub(crate) struct UnframedResult<T>(PhantomData<fn() -> T>);
impl<T> UnframedResult<T> {
    pub(crate) const fn returned() -> Self {
        Self(PhantomData)
    }
}
impl<T: SurrealValue> UnframedResult<T>
where
    usize: crate::transport::client::QueryResult<T>,
{
    pub(crate) fn take(
        self,
        response: &mut crate::transport::client::IndexedResults,
    ) -> Result<T, CanonicalError> {
        let slot = response
            .num_statements()
            .checked_sub(1)
            .ok_or(CanonicalError::IncompleteResponse)?;
        response.take(slot).map_err(Into::into)
    }
}
impl<'a> GuardedScript<'a> {
    pub(crate) fn new(envelope: Envelope<'a>) -> Self {
        Self { envelope }
    }
    /// Bodies are owner-authored SurrealQL without transaction framing. Dynamic heads remain
    /// ordinary SQL body text; no extra RPC or catalog state is introduced.
    pub(crate) fn returning<T>(
        self,
        body: &str,
        expression: &str,
    ) -> Result<PreparedScript<T>, CanonicalError> {
        let mut bindings = Object::default();
        let mut selection = None;
        let mut sql = String::new();
        let mut postlude = String::new();
        let mut after_body = String::new();
        match self.envelope {
            Envelope::ProtectedRead {
                selection: pin,
                recheck,
            } => {
                sql.push_str(PROTECTED_BEGIN);
                bind_selection(&mut bindings, pin)?;
                selection = Some(pin.clone());
                postlude.push_str("fn::pse_execution_v1::deadline($pse_rpc_expires_at);\n");
                postlude.push_str(match recheck {
                    Recheck::Pin=>"IF $pin.expires_at <= time::micros() { THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; };\n",
                    Recheck::PinOrTicket=>"IF $pin.expires_at <= time::micros() AND !fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$protection) { THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; };\n",
                });
            }
            Envelope::ProtectedAdmission { selection: pin } => {
                sql.push_str(PROTECTED_BEGIN);
                bind_selection(&mut bindings, pin)?;
                selection = Some(pin.clone());
            }
            Envelope::ProjectAuthority { issuer } => {
                sql.push_str("BEGIN;\nfn::pse_project_v1::authority($pse_rpc_expires_at,$project_issuing_store);\n");
                bindings.insert("project_issuing_store", (issuer.to_owned()).into_value());
            }
            Envelope::StageGuard(authority) => {
                let (begin, end) = match authority.policy {
                    StagePolicy::Mutation => (STAGE_BEGIN, STAGE_END),
                    StagePolicy::Close => (CLOSE_STAGE_BEGIN, CLOSE_STAGE_END),
                };
                sql.push_str(begin);
                // These owner-issued constant endings contain exactly their outer COMMIT.
                postlude.push_str(end.strip_suffix("COMMIT;").unwrap_or(end));
                bindings.insert(
                    "problem",
                    (authority.problem.as_str().to_owned()).into_value(),
                );
                bindings.insert(
                    "operation",
                    (authority.operation.as_str().to_owned()).into_value(),
                );
                bindings.insert(
                    "generation",
                    canonical_codec::encode_uint(authority.generation)?,
                );
                bindings.insert(
                    "expected",
                    authority
                        .expected
                        .map_or(Value::None, |key| (key.as_str().to_owned()).into_value()),
                );
                bindings.insert(
                    "request_digest",
                    (authority.request_digest.to_owned()).into_value(),
                );
                bindings.insert(
                    "lifetime",
                    (i64::try_from(authority.lifetime.as_micros())
                        .map_err(|_| CanonicalError::PayloadLimit)?)
                    .into_value(),
                );
            }
            Envelope::StageAcquisition(authority) => {
                sql.push_str("BEGIN;\n");
                postlude.push_str("fn::pse_execution_v1::deadline($pse_rpc_expires_at);\n");
                bindings.insert(
                    "problem",
                    (authority.problem.as_str().to_owned()).into_value(),
                );
                bindings.insert(
                    "operation",
                    (authority.operation.as_str().to_owned()).into_value(),
                );
                bindings.insert(
                    "expected",
                    authority
                        .expected
                        .map_or(Value::None, |key| (key.as_str().to_owned()).into_value()),
                );
                bindings.insert(
                    "request_digest",
                    (authority.request_digest.to_owned()).into_value(),
                );
                bindings.insert(
                    "lifetime",
                    (i64::try_from(authority.lifetime.as_micros())
                        .map_err(|_| CanonicalError::PayloadLimit)?)
                    .into_value(),
                );
            }
            Envelope::StageCleanup(authority) => {
                sql.push_str(CLEANUP_STAGE_BEGIN);
                match authority.fence {
                    CleanupFence::BeforeBody => {
                        sql.push('\n');
                        sql.push_str(CLEANUP_STAGE_END);
                    }
                    CleanupFence::AfterBody => after_body.push_str(CLEANUP_STAGE_END),
                };
                postlude.push_str("\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\n");
                bindings.insert(
                    "problem",
                    (authority.problem.as_str().to_owned()).into_value(),
                );
                bindings.insert(
                    "operation",
                    (authority.operation.as_str().to_owned()).into_value(),
                );
            }
            Envelope::DeadlineOnly | Envelope::DrainCleanup => {
                sql.push_str("BEGIN;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\n");
                postlude.push_str("fn::pse_execution_v1::deadline($pse_rpc_expires_at);\n");
            }
            Envelope::Installation => {
                const DEADLINE: &str = "IF time::micros() >= $pse_rpc_expires_at { THROW 'PSE_REFUSAL:original_schema_installation_deadline_expired'; };\n";
                sql.push_str("BEGIN;\n");
                sql.push_str(DEADLINE);
                postlude.push_str(DEADLINE);
            }
        }
        sql.push('\n');
        sql.push_str(body);
        sql.push('\n');
        sql.push_str(&after_body);
        // Evaluate reads/functions before their owning recheck. RETURN only publishes
        // the retained value, so an expensive expression cannot escape its postlude.
        sql.push_str("\nLET $pse_guarded_script_result = (");
        sql.push_str(expression);
        sql.push_str(");\n");
        sql.push_str(&postlude);
        sql.push_str("RETURN $pse_guarded_script_result;\nCOMMIT;");
        let installation = matches!(self.envelope, Envelope::Installation);
        let access = match self.envelope {
            Envelope::ProtectedRead { .. } => crate::canonical::migration::SchemaAdmission::Read,
            Envelope::StageCleanup(_) | Envelope::DrainCleanup => crate::canonical::migration::SchemaAdmission::Cleanup,
            _ => crate::canonical::migration::SchemaAdmission::Mutation,
        };
        if !installation {
            sql.insert_str("BEGIN;".len(), crate::canonical::migration::TRANSACTION_ADMISSION);
        }
        Ok(PreparedScript {
            sql,
            bindings,
            selection,
            access,
            installation,
            result: ResultSlot(PhantomData),
        })
    }
}
fn bind_selection(
    bindings: &mut Object,
    selection: &ProtectedSelection,
) -> Result<(), CanonicalError> {
    let revision = selection.revision();
    bindings.insert("problem", (revision.problem.clone()).into_value());
    bindings.insert("revision", (revision.key.clone()).into_value());
    bindings.insert("sequence", canonical_codec::encode_uint(revision.sequence)?);
    bindings.insert("protection", (selection.key().to_owned()).into_value());
    Ok(())
}

/// Render source-owned bodies through the actual envelope owner for deliberate
/// peer/race fixtures that operate on an already held native transaction.
#[cfg(all(test, feature = "canonical-tests"))]
pub(crate) fn test_script_sql(body: &str, expression: &str) -> Result<String, CanonicalError> {
    Ok(test_script::<Value>(body, expression)?
        .into_parts()
        .0)
}
/// Fixture bodies retain the same typed framing, bindings and result slot as production.
#[cfg(any(feature = "test-support", all(test, feature = "canonical-tests")))]
pub(crate) fn test_script<T>(body: &str, expression: &str) -> Result<PreparedScript<T>, CanonicalError> {
    GuardedScript::new(Envelope::DeadlineOnly).returning(body,expression)
}

pub(crate) const PROTECTED_BEGIN: &str = r#"BEGIN;
SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $pin = SELECT * FROM ONLY type::record('canonical_protections', $protection) FOR UPDATE;
IF $pin = NONE OR $pin.problem != $problem OR $pin.revision != $revision OR $pin.sequence != $sequence OR $pin.released OR ($pin.expires_at <= time::micros() AND !fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$protection)) { THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; };
LET $grant=fn::pse_execution_v1::protection_grant($pse_rpc_expires_at,$problem,$protection,false);
IF $pin.grant_deadline!=$grant.grant_deadline OR $pin.grant_nonce!=$grant.grant_nonce OR $pin.authority_incarnation!=$grant.authority_incarnation { THROW 'PSE_REFUSAL:immutable_selection_grant_differs'; };
LET $result_pin=SELECT * FROM ONLY type::record('canonical_result_protections',$protection) FOR UPDATE;
IF $result_pin!=NONE {
    LET $retirement=SELECT * FROM ONLY type::record('canonical_result_retirements',$result_pin.run) FOR UPDATE;
    LET $selected_manifest=SELECT * FROM ONLY type::record('canonical_result_manifests',$result_pin.manifest) FOR UPDATE;
    IF $retirement!=NONE OR $selected_manifest=NONE OR $selected_manifest.attempt!=$result_pin.attempt { THROW 'PSE_REFUSAL:result_protection_retired_or_manifest_unavailable'; };
};"#;

pub(crate) const STAGE_BEGIN: &str = r#"BEGIN;
LET $retention_guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $retention_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $lease = SELECT * FROM ONLY type::record('canonical_stages', $operation);
IF $lease = NONE OR $lease.problem != $problem OR $lease.generation != $generation OR ($lease.expected_head ?? NONE) != $expected OR $lease.request_digest != $request_digest OR $lease.activated OR $lease.abandoned OR $lease.closed OR $lease.expires_at <= time::micros() { THROW 'PSE_REFUSAL:staging_lease_fenced_or_expired'; };"#;

pub(crate) const STAGE_END: &str = r#"IF $lease.expires_at <= time::micros() { THROW 'PSE_REFUSAL:staging_lease_expired_before_renewal'; };
UPDATE type::record('canonical_stages', $operation) SET expires_at = time::micros() + $lifetime;
UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
fn::pse_execution_v1::touch($pse_rpc_expires_at,'source-creators:'+$problem);
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
IF $lease.expires_at <= time::micros() { THROW 'PSE_REFUSAL:staging_lease_expired_before_commit'; };
COMMIT;"#;

pub(crate) const CLOSE_STAGE_BEGIN: &str = r#"BEGIN;
LET $retention_guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $retention_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $lease = SELECT * FROM ONLY type::record('canonical_stages', $operation);
IF $lease = NONE OR $lease.problem != $problem OR $lease.generation != $generation OR ($lease.expected_head ?? NONE) != $expected OR $lease.request_digest != $request_digest OR $lease.activated OR $lease.abandoned OR ($lease.closed = false AND $lease.expires_at <= time::micros()) { THROW 'PSE_REFUSAL:staging_close_fenced_or_expired'; };"#;

pub(crate) const CLOSE_STAGE_END: &str = r#"IF $lease.closed = false AND $lease.expires_at <= time::micros() { THROW 'PSE_REFUSAL:staging_close_lease_expired'; };
IF $lease.closed = false {
    UPDATE type::record('canonical_stages', $operation) SET expires_at = time::micros() + $lifetime;
    UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
    fn::pse_execution_v1::touch($pse_rpc_expires_at,'source-creators:'+$problem);
};
fn::pse_execution_v1::deadline($pse_rpc_expires_at);
IF $lease.closed = false AND $lease.expires_at <= time::micros() { THROW 'PSE_REFUSAL:staging_close_lease_expired_before_commit'; };
COMMIT;"#;

pub(crate) const CLEANUP_STAGE_BEGIN: &str = r#"BEGIN;
fn::pse_execution_v1::register($pse_rpc_expires_at,'source-creators:'+$problem);
LET $retention_guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $retention_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $stage_guard = type::record('canonical_guards', 'stage:' + $operation);
SELECT * FROM $stage_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $lease = SELECT * FROM ONLY type::record('canonical_stages', $operation);
LET $eligible = $lease != NONE AND $lease.problem = $problem AND ($lease.activated OR $lease.abandoned OR $lease.expires_at <= time::micros());
IF $eligible AND $lease.activated = false { UPDATE type::record('canonical_stages', $operation) SET abandoned = true, closed = false, generation = generation + 1dec; };"#;

pub(crate) const CLEANUP_STAGE_END: &str = r#"UPSERT $stage_guard SET key = 'stage:' + $operation, generation = (generation ?? 0dec) + 1dec;
UPSERT $retention_guard SET key='retention:'+$problem,generation=(generation ?? 0dec)+1dec;"#;

#[cfg(test)]
mod tests {
    use super::*;
    fn selection() -> ProtectedSelection {
        ProtectedSelection::test_only(
            crate::canonical::Revision {
                key: "revision".into(),
                problem: "problem".into(),
                sequence: u64::MAX,
                parent: None,
                operation: "operation".into(),
                request: vec![].into(),
                interpretation: crate::generated::surreal::INTERPRETATION.into(),
            },
            "protection",
        )
    }
    #[test]
    fn protected_reads_capture_actual_result_before_exact_postlude_and_retain_selection() {
        let pin = selection();
        for recheck in [Recheck::Pin, Recheck::PinOrTicket] {
            let script = GuardedScript::new(Envelope::ProtectedRead {
                selection: &pin,
                recheck,
            })
            .returning::<Option<Object>>(
                "LET $body=true;",
                "SELECT * FROM ONLY canonical_revisions:revision",
            )
            .unwrap();
            let (sql, bindings, retained) = script.into_parts();
            let evaluated = sql.find("LET $pse_guarded_script_result").unwrap();
            let rechecked = sql.rfind("IF $pin.expires_at").unwrap();
            let returned = sql.rfind("RETURN $pse_guarded_script_result").unwrap();
            assert!(evaluated < rechecked && rechecked < returned);
            let postlude = &sql[evaluated..];
            assert_eq!(
                postlude.contains("AND !fn::pse_files_v1::ticket_open"),
                matches!(recheck, Recheck::PinOrTicket)
            );
            assert_eq!(
                bindings.get("sequence"),
                Some(&canonical_codec::encode_uint(u64::MAX).unwrap())
            );
            assert_eq!(retained.as_ref().unwrap().key(), pin.key());
            assert_eq!(retained.as_ref().unwrap().revision(), pin.revision());
        }
        let script = GuardedScript::new(Envelope::ProtectedAdmission { selection: &pin })
            .returning::<Value>("LET $issued=true;", "$issued")
            .unwrap();
        let (sql, _, retained) = script.into_parts();
        let result = &sql[sql.find("LET $pse_guarded_script_result").unwrap()..];
        assert!(!result.contains("IF $pin.expires_at"));
        assert!(!result.contains("fn::pse_execution_v1::deadline"));
        assert!(retained.is_some());
    }
    #[cfg(feature = "canonical-maintenance")]
    #[test]
    fn every_guarded_envelope_parses_with_the_pinned_native_parser() {
        let revision = crate::canonical::Revision {
            key: "revision".into(),
            problem: "problem".into(),
            sequence: 1,
            parent: None,
            operation: "operation".into(),
            request: Vec::new().into(),
            interpretation: "test".into(),
        };
        let selection = ProtectedSelection::test_only(revision, "issued-protection");
        let problem = ProblemKey::from("problem");
        let operation = OperationKey::from("operation");
        let expected = RevisionKey::from("revision");
        let stage = StageAuthority {
            problem: &problem,
            operation: &operation,
            generation: 1,
            expected: Some(&expected),
            request_digest: "request",
            lifetime: Duration::from_secs(300),
            policy: StagePolicy::Mutation,
        };
        let envelopes = [
            Envelope::ProtectedRead {
                selection: &selection,
                recheck: Recheck::Pin,
            },
            Envelope::ProtectedRead {
                selection: &selection,
                recheck: Recheck::PinOrTicket,
            },
            Envelope::ProtectedAdmission {
                selection: &selection,
            },
            Envelope::ProjectAuthority {
                issuer: "issued-store",
            },
            Envelope::StageGuard(stage),
            Envelope::StageGuard(StageAuthority {
                policy: StagePolicy::Close,
                ..stage
            }),
            Envelope::StageAcquisition(StageAcquisition {
                problem: &problem,
                operation: &operation,
                expected: Some(&expected),
                request_digest: "request",
                lifetime: Duration::from_secs(300),
            }),
            Envelope::StageCleanup(StageCleanup {
                problem: &problem,
                operation: &operation,
                fence: CleanupFence::BeforeBody,
            }),
            Envelope::StageCleanup(StageCleanup {
                problem: &problem,
                operation: &operation,
                fence: CleanupFence::AfterBody,
            }),
            Envelope::Installation,
            Envelope::DeadlineOnly,
        ];
        for envelope in envelopes {
            let (sql, _, _) = GuardedScript::new(envelope)
                .returning::<Value>("LET $selected = {ready:true};", "$selected")
                .unwrap()
                .into_parts();
            assert!(
                surrealdb_syn::parse(&sql).is_ok(),
                "native parser refused {envelope:?}: {sql}"
            );
            assert!(
                sql.find("LET $pse_guarded_script_result").unwrap()
                    < sql.rfind("RETURN $pse_guarded_script_result").unwrap()
            );
        }
    }
    #[cfg(all(feature = "canonical-tests", feature = "canonical-maintenance"))]
    #[test]
    fn delayed_installation_keeps_typed_policy_bindings_and_native_syntax() {
        let mut script=GuardedScript::new(Envelope::Installation).returning::<Value>("LET $selected=$fixture_original;","$selected").unwrap();
        script.bindings.insert("fixture_original",Value::String("original".into()));
        let delayed=script.test_delay_installation(Duration::from_secs(1)).unwrap();
        assert!(delayed.schema_policy().1);
        let (sql,bindings,selection)=delayed.into_parts();
        assert!(surrealdb_syn::parse(&sql).is_ok(),"{sql}");
        assert_eq!(bindings.get("fixture_original"),Some(&Value::String("original".into())));
        assert!(selection.is_none());
        assert!(sql.starts_with("BEGIN; SLEEP 1000ms;"));
        assert!(!sql.contains(crate::canonical::migration::TRANSACTION_ADMISSION));
        assert!(GuardedScript::new(Envelope::DeadlineOnly).returning::<Value>("","NONE").unwrap().test_delay_installation(Duration::from_secs(1)).is_err());
    }

    #[test]
    fn cleanup_fences_preserve_both_original_native_statement_orders() {
        let problem = ProblemKey::from("problem");
        let operation = OperationKey::from("operation");
        for fence in [CleanupFence::BeforeBody, CleanupFence::AfterBody] {
            let (sql, _, _) = GuardedScript::new(Envelope::StageCleanup(StageCleanup {
                problem: &problem,
                operation: &operation,
                fence,
            }))
            .returning::<Value>("LET $remaining_child_read=true;", "$remaining_child_read")
            .unwrap()
            .into_parts();
            let body = sql.find("LET $remaining_child_read").unwrap();
            let generation = sql.find("UPSERT $stage_guard SET").unwrap();
            match fence {
                CleanupFence::BeforeBody => assert!(generation < body),
                CleanupFence::AfterBody => assert!(generation > body),
            }
            assert!(generation < sql.find("LET $pse_guarded_script_result").unwrap());
        }
    }
    #[test]
    fn project_authority_binds_issuer_without_interpreting_it_as_sql() {
        let script = GuardedScript::new(Envelope::ProjectAuthority {
            issuer: "native-'issuer",
        })
        .returning::<Option<Object>>(
            "LET $selected=SELECT * FROM ONLY canonical_projects:example;",
            "$selected",
        )
        .unwrap();
        let (sql, bindings, selection) = script.into_parts();
        assert!(sql.starts_with("BEGIN;"));
        assert!(sql.ends_with("COMMIT;"));
        assert_eq!(sql.matches("BEGIN;").count(), 1);
        assert_eq!(sql.matches("COMMIT;").count(), 1);
        assert!(!sql.contains("native-'issuer"));
        assert!(selection.is_none());
        assert_eq!(
            bindings.get("project_issuing_store"),
            Some(&("native-'issuer").into_value())
        );
    }
    #[test]
    fn stage_envelopes_preserve_distinct_mutation_close_and_cleanup_meanings() {
        let problem = ProblemKey::from("problem");
        let operation = OperationKey::from("operation");
        let expected = RevisionKey::from("revision");
        for policy in [StagePolicy::Mutation, StagePolicy::Close] {
            let authority = StageAuthority {
                problem: &problem,
                operation: &operation,
                generation: u64::MAX,
                expected: Some(&expected),
                request_digest: "digest",
                lifetime: Duration::from_secs(300),
                policy,
            };
            let script = GuardedScript::new(Envelope::StageGuard(authority))
                .returning::<Value>("LET $selected=true;", "$selected")
                .unwrap();
            let (sql, bindings, _) = script.into_parts();
            assert_eq!(sql.matches("BEGIN;").count(), 1);
            assert_eq!(sql.matches("COMMIT;").count(), 1);
            assert_eq!(
                bindings.get("generation"),
                Some(&canonical_codec::encode_uint(u64::MAX).unwrap())
            );
            match policy {
                StagePolicy::Mutation => assert!(sql.contains("staging_lease_fenced_or_expired")),
                StagePolicy::Close => assert!(sql.contains("staging_close_fenced_or_expired")),
            }
        }
        let cleanup = GuardedScript::new(Envelope::StageCleanup(StageCleanup {
            problem: &problem,
            operation: &operation,
            fence: CleanupFence::AfterBody,
        }))
        .returning::<Value>("LET $selected=true;", "$selected")
        .unwrap();
        let (sql, bindings, _) = cleanup.into_parts();
        assert!(sql.contains("LET $eligible ="));
        assert!(!bindings.contains_key("generation"));
        let acquisition = GuardedScript::new(Envelope::StageAcquisition(StageAcquisition {
            problem: &problem,
            operation: &operation,
            expected: None,
            request_digest: "digest",
            lifetime: Duration::from_secs(300),
        }))
        .returning::<Value>("LET $selected=true;", "$selected")
        .unwrap();
        let (sql, bindings, _) = acquisition.into_parts();
        assert!(!sql.contains("LET $lease ="));
        assert!(!bindings.contains_key("generation"));
        assert_eq!(bindings.get("expected"), Some(&Value::None));
    }
    #[test]
    fn deadline_only_has_explicit_atomic_return_without_protection_authority() {
        let script = GuardedScript::new(Envelope::DeadlineOnly)
            .returning::<bool>("LET $ok=true;", "$ok")
            .unwrap();
        let (sql, bindings, selection) = script.into_parts();
        assert!(sql.contains("LET $pse_guarded_script_result = ($ok);"));
        assert!(sql.contains("RETURN $pse_guarded_script_result;\nCOMMIT;"));
        assert!(bindings.is_empty());
        assert!(selection.is_none());
        assert!(!sql.contains("canonical_protections"));
        let evaluate = sql.find("LET $pse_guarded_script_result").unwrap();
        let recheck = sql.rfind("fn::pse_execution_v1::deadline").unwrap();
        assert!(evaluate < recheck);
    }
    #[test]
    fn installation_uses_original_clock_without_calling_uninstalled_module() {
        let script = GuardedScript::new(Envelope::Installation)
            .returning::<Value>("DEFINE TABLE canonical_interpretations SCHEMAFULL;", "NONE")
            .unwrap();
        let (sql, bindings, selection) = script.into_parts();
        assert!(!sql.contains("fn::pse_execution_v1::deadline"));
        assert_eq!(
            sql.matches("original_schema_installation_deadline_expired")
                .count(),
            2
        );
        let declaration = sql.find("DEFINE TABLE").unwrap();
        assert!(sql.find("IF time::micros()").unwrap() < declaration);
        assert!(declaration < sql.rfind("IF time::micros()").unwrap());
        assert!(bindings.is_empty() && selection.is_none());
    }
}
