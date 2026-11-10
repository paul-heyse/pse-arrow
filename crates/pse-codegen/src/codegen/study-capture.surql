// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
DEFINE FUNCTION fn::pse_workbench_v1::capture($capture:object) -> object {
    // The explicit serializable caller owns this complete bounded read/write
    // decision. Point/range and absent membership reads below, including those
    // inside capture_facts, are the conflict footprint. Frozen paging starts
    // only after this receipt commits; it does not reacquire live membership.
    IF array::sort(object::keys($capture))!=['expiry','filter','key','maximum','members','owner','request'] OR !type::is_int($capture.expiry) OR !type::is_int($capture.maximum) OR !type::is_int($capture.members) OR $capture.maximum<1 OR $capture.members<1 OR !type::is_string($capture.key) OR bytes::len(<bytes>$capture.key)<1 OR bytes::len(<bytes>$capture.key)>4096 OR !type::is_string($capture.owner) OR bytes::len(<bytes>$capture.owner)<1 OR bytes::len(<bytes>$capture.owner)>4096 OR !type::is_bytes($capture.request) OR bytes::len($capture.request)<1 OR bytes::len($capture.request)>262144 OR !type::is_object($capture.filter) { RETURN {receipt:NONE,refusal:'control_request_invalid'}; };
    fn::pse_workbench_v1::validate_filter($capture.filter);
    IF time::micros()>=$capture.expiry { RETURN {receipt:NONE,refusal:'original_canonical_rpc_deadline_expired'}; };
    LET $old=SELECT * FROM ONLY type::record('canonical_study_captures',$capture.key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.owner!=$capture.owner OR $old.request!=$capture.request { RETURN {receipt:NONE,refusal:'study_capture_identity_reused'}; };
        IF $old.state='frozen' { RETURN {receipt:$old,refusal:NONE}; };
        RETURN {receipt:NONE,refusal:'study_capture_original_owner_closed'};
    };
    LET $owner=SELECT * FROM ONLY type::record('canonical_file_owners',$capture.owner) FOR UPDATE;
    IF $owner=NONE OR $owner.drained OR $owner.issuing_store!=(SELECT issuing_store FROM ONLY canonical_interpretations:current).issuing_store { RETURN {receipt:NONE,refusal:'study_capture_actual_owner_unavailable'}; };
    LET $facts=fn::pse_workbench_v1::capture_facts($capture.expiry,$capture.key,$capture.filter,$capture.maximum,$capture.members);
    IF $facts.refusal!=NONE { RETURN {receipt:NONE,refusal:$facts.refusal}; };
    IF array::len($facts.headers)!=0 { INSERT INTO canonical_study_capture_headers $facts.headers RETURN NONE; };
    IF array::len($facts.members)!=0 { INSERT INTO canonical_study_capture_members $facts.members RETURN NONE; };
    IF array::len($facts.protections)!=0 { INSERT INTO canonical_study_capture_protections $facts.protections RETURN NONE; };
    LET $receipt={key:$capture.key,owner:$capture.owner,request:$capture.request,state:'frozen',header_count:<decimal>array::len($facts.headers),member_count:<decimal>array::len($facts.members)};
    LET $saved=CREATE ONLY type::record('canonical_study_captures',$capture.key) CONTENT $receipt;
    IF time::micros()>=$capture.expiry { RETURN {receipt:NONE,refusal:'original_canonical_rpc_deadline_expired'}; };
    RETURN {receipt:$saved,refusal:NONE};
} PERMISSIONS NONE;
