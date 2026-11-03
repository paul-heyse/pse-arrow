// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded capture operation; every host SQL call inherits the invoking transaction.

use surrealdb_types::{Bytes, Object, SurrealValue, Value};

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
    input.query(CAPTURE)
}

// Fixed SQL and bound typed input only. Keep the bounded relational arrays in
// the host transaction instead of serializing every member through WASM twice.
// Only the exact receipt/refusal crosses the host boundary. Witness registration
// still precedes every selector/fact read, and publication advances it only after
// all protections exist. The invoking envelope rolls back every checked refusal.
const CAPTURE: &str = r#"{
    IF time::micros()>=$capture.expiry { RETURN {receipt:NONE,refusal:'original_canonical_rpc_deadline_expired'}; };
    LET $old=SELECT * FROM ONLY type::record('canonical_study_captures',$capture.key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.owner!=$capture.owner OR $old.request!=$capture.request { RETURN {receipt:NONE,refusal:'study_capture_identity_reused'}; };
        IF $old.state='frozen' { RETURN {receipt:$old,refusal:NONE}; };
        RETURN {receipt:NONE,refusal:'study_capture_original_owner_closed'};
    };
    LET $owner=SELECT * FROM ONLY type::record('canonical_file_owners',$capture.owner) FOR UPDATE;
    IF $owner=NONE OR $owner.drained OR $owner.issuing_store!=(SELECT issuing_store FROM ONLY canonical_interpretations:current).issuing_store { RETURN {receipt:NONE,refusal:'study_capture_actual_owner_unavailable'}; };
    LET $witness=SELECT * FROM ONLY type::record('canonical_guards','study-publication') FOR UPDATE;
    IF $witness=NONE { RETURN {receipt:NONE,refusal:'study_capture_publication_witness_unavailable'}; };
    LET $projects=SELECT VALUE problem FROM canonical_project_branches WHERE project=$capture.filter.project LIMIT 65;
    IF array::len($projects)>64 { RETURN {receipt:NONE,refusal:'study_workbench_narrow_selection'}; };
    LET $facts=fn::pse_workbench_v1::capture_facts($capture.expiry,$capture.key,$capture.filter,$capture.maximum,$capture.members);
    IF $facts.refusal!=NONE { RETURN {receipt:NONE,refusal:$facts.refusal}; };
    IF array::len($facts.headers)!=0 { INSERT INTO canonical_study_capture_headers $facts.headers RETURN NONE; };
    IF array::len($facts.members)!=0 { INSERT INTO canonical_study_capture_members $facts.members RETURN NONE; };
    IF array::len($facts.protections)!=0 { INSERT INTO canonical_study_capture_protections $facts.protections RETURN NONE; };
    LET $receipt={key:$capture.key,owner:$capture.owner,request:$capture.request,state:'frozen',header_count:<decimal>array::len($facts.headers),member_count:<decimal>array::len($facts.members)};
    LET $saved=CREATE ONLY type::record('canonical_study_captures',$capture.key) CONTENT $receipt;
    fn::pse_workbench_v1::changed();
    IF time::micros()>=$capture.expiry { RETURN {receipt:NONE,refusal:'original_canonical_rpc_deadline_expired'}; };
    RETURN {receipt:$saved,refusal:NONE};
}"#;

#[surrealism::surrealism]
mod study_capture {
    use super::{CaptureReply, CaptureRequest, capture as perform};

    /// Publish one exact protected cohort and receipt in the caller transaction.
    #[surrealism(writeable)]
    fn capture(input: CaptureRequest) -> Result<CaptureReply, String> {
        perform(input)
    }
}
