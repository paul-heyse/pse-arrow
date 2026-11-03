// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded legacy capture settlement inside the original quiescent schema owner.
//!
//! The migration caller must prepend its exact original owner/predecessor guard
//! and retain the original per-RPC migration clock. No ordinary caller can use
//! this helper; it runs before the original recovery function is overwritten.

use surrealdb_types::SurrealValue;

#[derive(Debug, SurrealValue)]
#[surreal(crate = "::surrealdb_types")]
pub(super) struct Page {
    pub(super) capture: Option<String>,
    pub(super) done: bool,
}

pub(super) const RESULT: &str = "$pse_legacy_capture_page";

pub(super) fn body() -> String {
    BODY.replace(
        "__PAGE__",
        &pse_schema::catalog::study_workbench::STUDY_CAPTURE_PAGE_ROWS.to_string(),
    )
    .replace(
        "__MEMBERS__",
        &pse_schema::catalog::study_workbench::STUDY_COHORT_MAX_MEMBERS.to_string(),
    )
}

const BODY: &str = r#"
LET $pse_legacy_rows=SELECT key FROM canonical_study_captures WHERE state!='released' ORDER BY key LIMIT 1;
LET $pse_legacy_capture_page=IF array::len($pse_legacy_rows)=0 {
    LET $gate=SELECT * FROM ONLY type::record('canonical_study_capture_gate','current') FOR UPDATE;
    IF $gate.capture!=NONE { THROW 'PSE_REFUSAL:study_capture_legacy_pending_not_drained'; };
    RETURN {capture:NONE,done:true};
} ELSE {
    LET $capture=SELECT * FROM ONLY type::record('canonical_study_captures',$pse_legacy_rows[0].key) FOR UPDATE;
    LET $owner=SELECT * FROM ONLY type::record('canonical_file_owners',$capture.owner) FOR UPDATE;
    IF $owner=NONE OR !$owner.drained OR $owner.issuing_store!=(SELECT issuing_store FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:study_capture_legacy_pending_not_drained'; };
    IF $capture.header_count>64dec OR $capture.member_count>__MEMBERS__dec OR $capture.state NOT IN ['pending','abandoned','frozen','releasing'] { THROW 'PSE_REFUSAL:study_capture_identity_reused'; };
    IF $capture.state='pending' {
        LET $gate=SELECT * FROM ONLY type::record('canonical_study_capture_gate','current') FOR UPDATE;
        IF $gate.capture!=$capture.key OR $gate.owner!=$capture.owner OR $gate.request!=$capture.request OR $gate.incarnation!=$capture.incarnation OR $gate.issued_expires_at!=$capture.issued_expires_at { THROW 'PSE_REFUSAL:study_capture_identity_reused'; };
        // Invoke the original installed recovery, including its incarnation fence.
        fn::pse_workbench_v1::recover_capture($pse_rpc_expires_at,$capture.key,$capture.owner,$capture.request);
    };
    UPDATE ONLY type::record('canonical_study_captures',$capture.key) SET state='releasing' RETURN NONE;
    LET $protections=SELECT key FROM canonical_study_capture_protections WHERE capture=$capture.key LIMIT __PAGE__;
    LET $members=SELECT key FROM canonical_study_capture_members WHERE capture=$capture.key LIMIT __PAGE__;
    LET $headers=SELECT key FROM canonical_study_capture_headers WHERE capture=$capture.key LIMIT 64;
    FOR $row IN $protections { DELETE ONLY type::record('canonical_study_capture_protections',$row.key) RETURN NONE; };
    FOR $row IN $members { DELETE ONLY type::record('canonical_study_capture_members',$row.key) RETURN NONE; };
    FOR $row IN $headers { DELETE ONLY type::record('canonical_study_capture_headers',$row.key) RETURN NONE; };
    LET $done=array::len($protections)<__PAGE__ AND array::len($members)<__PAGE__ AND array::len($headers)<64;
    IF $done { UPDATE ONLY type::record('canonical_study_captures',$capture.key) SET state='released' RETURN NONE; };
    RETURN {capture:$capture.key,done:$done};
};
"#;

#[cfg(all(test, feature = "canonical-tests", feature = "canonical-maintenance"))]
#[path = "canonical_capture_legacy_tests.rs"]
mod tests;
