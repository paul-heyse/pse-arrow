// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded capture operation; every host SQL call inherits the invoking transaction.

use surrealdb_types::{Bytes, Decimal, Object, SurrealValue, Value};

#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
pub(super) struct CaptureRequest {
    expiry: i64,
    key: String,
    owner: String,
    request: Bytes,
    filter: Object,
    maximum: i64,
    members: i64,
}

#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
pub(super) struct CaptureReply {
    pub(super) receipt: Option<Object>,
    pub(super) refusal: Option<String>,
}

#[derive(Clone, Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
struct CaptureFacts {
    refusal: Option<String>,
    headers: Vec<Object>,
    members: Vec<Object>,
    protections: Vec<Object>,
}

impl CaptureRequest {
    fn query<T: SurrealValue>(&self, expression: &str) -> Result<T, String> {
        surrealism::imports::sql_with_vars(
            expression,
            [("capture".into(), self.clone().into_value())],
        )
        .map_err(|error| error.to_string())
    }
}

fn refused(code: &str) -> CaptureReply {
    CaptureReply {
        receipt: None,
        refusal: Some(code.into()),
    }
}

/// A checked refusal is returned to the invoking generated SQL envelope, which
/// throws the registry-owned refusal and rolls back this entire invocation.
/// Host failures propagate as guest errors and also abort the caller transaction.
pub(super) fn capture(input: CaptureRequest) -> Result<CaptureReply, String> {
    if input.key.is_empty()
        || input.key.len() > 4096
        || input.owner.is_empty()
        || input.owner.len() > 4096
        || input.request.is_empty()
        // Exact JSON intent can expand each admitted selector byte to six bytes.
        || input.request.len() > 256 * 1024
        || input.maximum < 1
        || input.members < 1
    {
        return Ok(refused("control_request_invalid"));
    }
    let keys = [
        "branch",
        "context",
        "outcome",
        "physical_revision",
        "project",
        "research_reference",
        "source",
        "study",
    ];
    if input.filter.keys().map(String::as_str).ne(keys) {
        return Ok(refused("control_request_invalid"));
    }
    for value in input.filter.values() {
        if !matches!(value, Value::None | Value::Null)
            && !matches!(value, Value::String(value) if !value.is_empty() && value.len()<=4096)
        {
            return Ok(refused("control_request_invalid"));
        }
    }
    if let Some(Value::String(outcome)) = input.filter.get("outcome")
        && serde_json::from_value::<pse_model::generated::enums::CanonicalTerminalOutcome>(
            serde_json::Value::String(outcome.clone()),
        )
        .is_err()
    {
        return Ok(refused("control_request_invalid"));
    }
    let now: i64 = input.query("time::micros()")?;
    if now >= input.expiry {
        return Ok(refused("original_canonical_rpc_deadline_expired"));
    }
    let old: Option<Object> = input.query(
        "SELECT * FROM ONLY type::record('canonical_study_captures',$capture.key) FOR UPDATE",
    )?;
    if let Some(old) = old {
        if old.get("owner") != Some(&Value::String(input.owner.clone()))
            || old.get("request") != Some(&Value::Bytes(input.request.clone()))
        {
            return Ok(refused("study_capture_identity_reused"));
        }
        return Ok(
            if old.get("state") == Some(&Value::String("frozen".into())) {
                CaptureReply {
                    receipt: Some(old),
                    refusal: None,
                }
            } else {
                refused("study_capture_original_owner_closed")
            },
        );
    }
    let actual: bool = input.query(
        "{ LET $owner=SELECT * FROM ONLY type::record('canonical_file_owners',$capture.owner) FOR UPDATE; RETURN $owner!=NONE AND !$owner.drained AND $owner.issuing_store=(SELECT issuing_store FROM ONLY canonical_interpretations:current).issuing_store; }",
    )?;
    if !actual {
        return Ok(refused("study_capture_actual_owner_unavailable"));
    }
    // Registration precedes all selector and captured-fact reads. Writers change
    // this preexisting record in their own transaction; they never wait here.
    let witness: Option<Object> = input.query(
        "SELECT * FROM ONLY type::record('canonical_guards','study-publication') FOR UPDATE",
    )?;
    if witness.is_none() {
        return Ok(refused("study_capture_publication_witness_unavailable"));
    }
    let projects: Vec<String> = input.query("SELECT VALUE problem FROM canonical_project_branches WHERE project=$capture.filter.project LIMIT 65")?;
    if projects.len() > 64 {
        return Ok(refused("study_workbench_narrow_selection"));
    }
    let facts: CaptureFacts = input.query(
        "fn::pse_workbench_v1::capture_facts($capture.expiry,$capture.key,$capture.filter,$capture.maximum,$capture.members)",
    )?;
    if let Some(refusal) = facts.refusal {
        return Ok(refused(&refusal));
    }
    let mut receipt = Object::new();
    receipt.insert("key", input.key.clone());
    receipt.insert("owner", input.owner.clone());
    receipt.insert("request", input.request.clone());
    receipt.insert("state", "frozen");
    receipt.insert("header_count", Decimal::from(facts.headers.len()));
    receipt.insert("member_count", Decimal::from(facts.members.len()));
    for (table, rows) in [
        ("canonical_study_capture_headers", facts.headers),
        ("canonical_study_capture_members", facts.members),
        ("canonical_study_capture_protections", facts.protections),
    ] {
        if !rows.is_empty() {
            let _: Value = surrealism::imports::sql_with_vars(
                format!("INSERT INTO {table} $rows RETURN NONE"),
                [("rows".into(), rows.into_value())],
            )
            .map_err(|error| error.to_string())?;
        }
    }
    let saved: Object = surrealism::imports::sql_with_vars(
        "CREATE ONLY type::record('canonical_study_captures',$receipt.key) CONTENT $receipt",
        [("receipt".into(), receipt.into_value())],
    )
    .map_err(|error| error.to_string())?;
    // Fence an older retention transaction's predicate read of absent capture
    // protections, including when this capture commits before that withdrawal.
    let _: Value = input.query("fn::pse_workbench_v1::changed()")?;
    let now: i64 = input.query("time::micros()")?;
    if now >= input.expiry {
        return Ok(refused("original_canonical_rpc_deadline_expired"));
    }
    Ok(CaptureReply {
        receipt: Some(saved),
        refusal: None,
    })
}

#[surrealism::surrealism]
mod study_capture {
    use super::{CaptureReply, CaptureRequest, capture as perform};

    /// Publish one exact protected cohort and receipt in the caller transaction.
    #[surrealism(writeable)]
    fn capture(input: CaptureRequest) -> Result<CaptureReply, String> {
        perform(input)
    }
}
