// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned selected scientific closure and attributed lifetime operations.

pub(super) fn append(ddl: &mut String) {
    let holders = pse_schema::catalog::substrate::ARTIFACT_HOLDERS;
    let guards = |release: bool| {
        holders
            .iter()
            .filter_map(|(kind, attach, detach)| {
                let guard = if release { detach } else { attach };
                (!guard.is_empty())
                    .then(|| format!("IF $obligation.owner_kind='{kind}' {{ {guard} }};"))
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    ddl.push_str(
        &FUNCTIONS
            .replace(
                "__ARTIFACT_APPEND_ROWS__",
                &pse_schema::catalog::substrate::ARTIFACT_APPEND_ROWS.to_string(),
            )
            .replace(
                "__ARTIFACT_HOLDERS__",
                &holders
                    .iter()
                    .map(|(kind, _, _)| format!("'{kind}'"))
                    .collect::<Vec<_>>()
                    .join(","),
            )
            .replace("__HOLDER_ATTACH__", &guards(false))
            .replace("__HOLDER_RELEASE__", &guards(true)),
    );
}

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_artifact_v1::abandon_study_inputs($rpc_expiry:int,$study:string) -> bool {
    fn::pse_execution_v1::touch($rpc_expiry,'study:'+$study);
    LET $header=SELECT * FROM ONLY type::record('canonical_studies',$study) FOR UPDATE;
    IF $header!=NONE { THROW 'PSE_REFUSAL:study_input_abandonment_requires_absent_header'; };
    UPSERT type::record('canonical_study_retirements',$study) SET key=$study,study=$study;
    LET $owners=SELECT * FROM canonical_artifact_obligations WHERE owner_kind='study' AND owner=$study ORDER BY key LIMIT 64;
    FOR $owner IN $owners { fn::pse_artifact_v1::release($rpc_expiry,object::remove($owner,['id'])); };
    LET $inputs=SELECT id FROM canonical_study_inputs WHERE study=$study ORDER BY key LIMIT 64;
    DELETE $inputs.id;
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN array::len($owners)<64 AND array::len($inputs)<64;
};
DEFINE FUNCTION fn::pse_artifact_v1::authority($rpc_expiry:int,$row:object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$row.primary_problem) FOR UPDATE;
    IF $guard=NONE OR $guard.incarnation!=$row.primary_authority OR $row.key!='pse.artifact.v1:'+$row.primary_authority+':'+$row.creation_nonce+':'+<string><int>$row.creation_expires_at { THROW 'PSE_REFUSAL:artifact_creation_authority_differs'; };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $guard;
};
// Recovery validates the original immutable intent without granting content access.
DEFINE FUNCTION fn::pse_artifact_v1::original($rpc_expiry:int,$intent:object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$intent.primary_problem) FOR UPDATE;
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$intent.key) FOR UPDATE;
    IF $guard=NONE OR $intent.key!='pse.artifact.v1:'+$intent.primary_authority+':'+$intent.creation_nonce+':'+<string><int>$intent.creation_expires_at { THROW 'PSE_REFUSAL:artifact_original_settlement_authority_unavailable'; };
    IF $row!=NONE AND object::remove($row,['id','active','retiring'])!=object::remove($intent,['id','active','retiring']) { THROW 'PSE_REFUSAL:artifact_recovery_requires_exact_original_intent'; };
    IF $guard.incarnation!=$intent.primary_authority AND $row=NONE { THROW 'PSE_REFUSAL:historical_artifact_settlement_receipt_unavailable'; };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $guard;
};
DEFINE FUNCTION fn::pse_artifact_v1::available($rpc_expiry:int,$key:string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$key) FOR UPDATE;
    IF $row=NONE OR !$row.active OR $row.retiring { THROW 'PSE_REFUSAL:artifact_unavailable'; };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $row;
};
DEFINE FUNCTION fn::pse_artifact_v1::producer($rpc_expiry:int,$input:object) -> bool {
    fn::pse_execution_v1::result_creator($rpc_expiry,$input.run);
    fn::pse_execution_v1::available($rpc_expiry,$input.run);
    LET $attempt=SELECT * FROM ONLY type::record('canonical_attempts',$input.attempt);
    LET $manifest=SELECT * FROM ONLY type::record('canonical_result_manifests',$input.manifest);
    IF $attempt=NONE OR $manifest=NONE OR $attempt.run!=$input.run OR ($attempt.phase NOT IN ['terminal','terminal_compacted']) OR $attempt.closed_manifest!=$input.manifest OR $manifest.attempt!=$input.attempt OR $manifest.generation!=$attempt.generation { THROW 'PSE_REFUSAL:artifact_producer_inventory_unavailable'; };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_artifact_v1::begin($rpc_expiry:int,$row:object,$sources:array<object>,$inputs:array<object>) -> object {
    LET $guard=fn::pse_artifact_v1::authority($rpc_expiry,$row);
    LET $now=time::micros();
    IF $row.creation_expires_at<=$now OR $row.creation_expires_at>$now+60000000 OR $row.creation_expires_at<=($guard.artifact_creation_closed_through ?? 0dec) { THROW 'PSE_REFUSAL:artifact_creation_window_closed'; };
    IF $row.active OR $row.retiring OR string::len($row.creation_nonce)!=36 OR string::len($row.creation_request_digest)!=64 OR array::len($sources)=0 OR array::len($sources)>65 OR array::len($inputs)>64 OR $row.block_count=0dec OR bytes::len($row.provenance)>131072 { THROW 'PSE_REFUSAL:artifact_closure_admission_bound'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_artifacts',$row.key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.retiring OR object::remove($old,['id','active'])!=object::remove($row,['active']) { THROW 'PSE_REFUSAL:immutable_artifact_request_changed'; };
        fn::pse_execution_v1::deadline($rpc_expiry); RETURN $old;
    };
    FOR $source IN $sources {
        fn::pse_execution_v1::source_creator($rpc_expiry,$source.problem,$source.key);
        LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$source.revision);
        LET $pruned=SELECT key FROM canonical_reclaimed_ranges WHERE problem=$source.problem AND from_sequence<=$source.sequence AND to_sequence>$source.sequence LIMIT 1;
        IF $revision=NONE OR $revision.problem!=$source.problem OR $revision.sequence!=$source.sequence OR $revision.interpretation!=$row.interpretation OR array::len($pruned)!=0 { THROW 'PSE_REFUSAL:artifact_source_unavailable'; };
    };
    IF array::len($sources[WHERE revision=$row.revision AND problem=$row.primary_problem])!=1 { THROW 'PSE_REFUSAL:artifact_primary_source_omitted'; };
    FOR $input IN $inputs {
        fn::pse_artifact_v1::producer($rpc_expiry,$input);
        LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$input.protection) FOR UPDATE;
        LET $selected=SELECT * FROM ONLY type::record('canonical_result_protections',$input.protection);
        IF $pin=NONE OR $pin.released OR ($pin.expires_at<=time::micros() AND !fn::pse_files_v1::ticket_open($rpc_expiry,$input.protection)) OR $selected=NONE OR $selected.run!=$input.run OR $selected.attempt!=$input.attempt OR $selected.manifest!=$input.manifest { THROW 'PSE_REFUSAL:artifact_exact_producer_protection_absent'; };
        LET $actual=SELECT revision FROM canonical_run_sources WHERE run=$input.run LIMIT 66;
        IF array::len($actual)=0 OR array::len($actual)>65 OR array::len(array::complement($actual.revision,$sources.revision))!=0 { THROW 'PSE_REFUSAL:artifact_actual_source_omitted'; };
    };
    CREATE ONLY type::record('canonical_artifacts',$row.key) CONTENT $row;
    FOR $source IN $sources { CREATE ONLY type::record('canonical_roots',$source.key) SET key=$source.key,problem=$source.problem,revision=$source.revision,sequence=$source.sequence,owner_kind='artifact',owner=$row.key; };
    FOR $input IN $inputs { CREATE ONLY type::record('canonical_artifact_staging_inputs',$input.key) CONTENT object::remove($input,['protection']); };
    FOR $input IN $inputs {
        LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$input.protection) FOR UPDATE;
        IF $pin=NONE OR $pin.released { THROW 'PSE_REFUSAL:artifact_transfer_protection_unavailable'; };
        IF !fn::pse_files_v1::ticket_open($rpc_expiry,$input.protection) { fn::pse_execution_v1::lease($rpc_expiry,$pin.expires_at); };
    };
    fn::pse_execution_v1::lease($rpc_expiry,<int>$row.creation_expires_at);
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $row;
};
DEFINE FUNCTION fn::pse_artifact_v1::append($rpc_expiry:int,$key:string,$block:object) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$key) FOR UPDATE;
    IF $row=NONE OR $row.retiring OR $block.artifact!=$key OR $block.ordinal>=$row.block_count OR $block.payload_bytes>33554432dec OR string::len($block.digest)!=64 { THROW 'PSE_REFUSAL:artifact_block_admission'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_artifact_blocks',$block.key);
    IF $old!=NONE { IF object::remove($old,['id'])!=$block { THROW 'PSE_REFUSAL:immutable_artifact_block_changed'; }; }
    ELSE { IF $row.active { THROW 'PSE_REFUSAL:artifact_content_sealed'; }; fn::pse_files_v1::attach($rpc_expiry,$block,'artifact',$key,'artifact:'+$block.key); CREATE ONLY type::record('canonical_artifact_blocks',$block.key) CONTENT $block; };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};
// All rows attach under the caller's single metadata-only transaction. File
// reservation/sealing has already completed outside this transaction.
DEFINE FUNCTION fn::pse_artifact_v1::append_many($rpc_expiry:int,$key:string,$blocks:array) -> array {
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF array::len($blocks)=0 OR array::len($blocks)>__ARTIFACT_APPEND_ROWS__ OR array::len(array::distinct($blocks.key))!=array::len($blocks) { THROW 'PSE_REFUSAL:artifact_metadata_group_extent_or_identity'; };
    FOR $block IN $blocks { fn::pse_artifact_v1::append($rpc_expiry,$key,$block); };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $blocks.key;
};
DEFINE FUNCTION fn::pse_artifact_v1::activate($rpc_expiry:int,$key:string,$obligation:object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$key) FOR UPDATE;
    IF $row=NONE OR $row.retiring { THROW 'PSE_REFUSAL:artifact_staging_unavailable'; };
    fn::pse_artifact_v1::authority($rpc_expiry,$row);
    IF !$row.active AND $row.creation_expires_at<=time::micros() { THROW 'PSE_REFUSAL:artifact_creation_expired_before_activation'; };
    LET $inputs=SELECT * FROM canonical_artifact_staging_inputs WHERE artifact=$key ORDER BY key LIMIT 65;
    IF array::len($inputs)>64 { THROW 'PSE_REFUSAL:artifact_input_bound'; };
    FOR $input IN $inputs { fn::pse_artifact_v1::producer($rpc_expiry,$input); };
    LET $coverage=SELECT count() AS n, math::sum(payload_bytes) AS bytes FROM canonical_artifact_blocks WHERE artifact=$key GROUP ALL;
    IF $coverage[0].n!=$row.block_count OR $coverage[0].bytes!=$row.payload_bytes { THROW 'PSE_REFUSAL:artifact_content_incomplete'; };
    fn::pse_artifact_v1::attach($rpc_expiry,$key,$obligation,false);
    DELETE $inputs.id;
    LET $saved=UPDATE ONLY type::record('canonical_artifacts',$key) SET active=true;
    IF !$row.active { fn::pse_execution_v1::lease($rpc_expiry,<int>$row.creation_expires_at); };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $saved;
};
DEFINE FUNCTION fn::pse_artifact_v1::attach($rpc_expiry:int,$key:string,$obligation:object,$require_active:bool) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$key) FOR UPDATE;
    IF $row=NONE OR $row.retiring OR ($require_active AND !$row.active) OR $obligation.artifact!=$key OR $obligation.owner_kind NOT IN [__ARTIFACT_HOLDERS__] OR string::len($obligation.owner)=0 { THROW 'PSE_REFUSAL:artifact_obligation_unavailable'; };
    __HOLDER_ATTACH__
    IF (SELECT * FROM ONLY type::record('canonical_artifact_releases',$obligation.key) FOR UPDATE)!=NONE { THROW 'PSE_REFUSAL:artifact_obligation_permanently_released'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_artifact_obligations',$obligation.key) FOR UPDATE;
    IF $old!=NONE { IF object::remove($old,['id'])!=$obligation { THROW 'PSE_REFUSAL:artifact_obligation_identity_changed'; }; }
    ELSE { CREATE ONLY type::record('canonical_artifact_obligations',$obligation.key) CONTENT $obligation; };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_artifact_v1::release($rpc_expiry:int,$obligation:object) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    SELECT * FROM ONLY type::record('canonical_artifacts',$obligation.artifact) FOR UPDATE;
    LET $old=SELECT * FROM ONLY type::record('canonical_artifact_obligations',$obligation.key) FOR UPDATE;
    IF $old!=NONE AND object::remove($old,['id'])!=$obligation { THROW 'PSE_REFUSAL:artifact_release_owner_differs'; };
    LET $released=SELECT * FROM ONLY type::record('canonical_artifact_releases',$obligation.key) FOR UPDATE;
    IF $released!=NONE {
        IF object::remove($released,['id'])!=$obligation { THROW 'PSE_REFUSAL:artifact_release_identity_changed'; };
        // Exact settled replay remains available after its retiring header is removed.
        // An inconsistent live obligation fails closed instead of being silently released.
        IF $old!=NONE { THROW 'PSE_REFUSAL:artifact_release_owner_differs'; };
        fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
    };
    __HOLDER_RELEASE__
    CREATE ONLY type::record('canonical_artifact_releases',$obligation.key) CONTENT $obligation;
    DELETE ONLY type::record('canonical_artifact_obligations',$obligation.key);
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_artifact_v1::acquire($rpc_expiry:int,$key:string,$protection:string) -> object {
    LET $row=fn::pse_artifact_v1::available($rpc_expiry,$key);
    LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$protection) FOR UPDATE;
    IF $pin=NONE OR $pin.released OR $pin.expires_at<=time::micros() OR $pin.revision!=$row.revision { THROW 'PSE_REFUSAL:artifact_read_protection_absent'; };
    UPSERT type::record('canonical_artifact_reads',$protection) SET key=$protection,artifact=$key;
    fn::pse_execution_v1::lease($rpc_expiry,$pin.expires_at);
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $row;
};
DEFINE FUNCTION fn::pse_artifact_v1::read($rpc_expiry:int,$key:string,$protection:string,$ordinal:decimal) -> object {
    LET $row=fn::pse_artifact_v1::available($rpc_expiry,$key);
    LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$protection) FOR UPDATE;
    LET $read=SELECT * FROM ONLY type::record('canonical_artifact_reads',$protection);
    IF $pin=NONE OR $pin.released OR ($pin.expires_at<=time::micros() AND !fn::pse_files_v1::ticket_open($rpc_expiry,$protection)) OR $pin.revision!=$row.revision OR $read=NONE OR $read.artifact!=$key { THROW 'PSE_REFUSAL:artifact_read_grant_unavailable'; };
    LET $blocks=SELECT * FROM canonical_artifact_blocks WHERE artifact=$key AND ordinal=$ordinal LIMIT 1;
    IF array::len($blocks)!=1 { THROW 'PSE_REFUSAL:artifact_block_unavailable'; };
    IF !fn::pse_files_v1::ticket_open($rpc_expiry,$protection) { fn::pse_execution_v1::lease($rpc_expiry,$pin.expires_at); };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $blocks[0];
};
DEFINE FUNCTION fn::pse_artifact_v1::collect($rpc_expiry:int,$intent:object) -> object {
    LET $guard=fn::pse_artifact_v1::original($rpc_expiry,$intent);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$intent.key) FOR UPDATE;
    LET $owners=SELECT key FROM canonical_artifact_obligations WHERE artifact=$intent.key LIMIT 1;
    IF array::len($owners)!=0 { THROW 'PSE_REFUSAL:artifact_has_retained_obligations'; };
    IF $row!=NONE AND ($row.creation_request_digest!=$intent.creation_request_digest OR $row.creation_expires_at!=$intent.creation_expires_at) { THROW 'PSE_REFUSAL:artifact_settlement_requires_original_intent'; };
    LET $reads=SELECT key FROM canonical_artifact_reads WHERE artifact=$intent.key ORDER BY key LIMIT 64;
    FOR $read IN $reads {
        LET $settlement=fn::pse_retention_v1::settle_protection($rpc_expiry,$intent.primary_problem,$read.key);
        IF !$settlement.complete { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:true}; };
        DELETE ONLY type::record('canonical_artifact_reads',$read.key);
    };
    IF array::len($reads)=64 { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    IF $row!=NONE { UPDATE ONLY type::record('canonical_artifacts',$intent.key) SET retiring=true; };
    IF $guard.incarnation=$intent.primary_authority AND time::micros()<$intent.creation_expires_at { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:true}; };
    LET $blocks=SELECT id,key FROM canonical_artifact_blocks WHERE artifact=$intent.key ORDER BY ordinal LIMIT 64;
    IF array::len($blocks)!=0 { FOR $block IN $blocks { fn::pse_files_v1::release_borrower($rpc_expiry,'artifact:'+$block.key); }; DELETE $blocks.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $inputs=SELECT id,run FROM canonical_artifact_staging_inputs WHERE artifact=$intent.key ORDER BY key LIMIT 64;
    FOR $input IN $inputs { fn::pse_execution_v1::result_creator($rpc_expiry,$input.run); };
    IF array::len($inputs)!=0 { DELETE $inputs.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $roots=SELECT id,problem FROM canonical_roots WHERE owner_kind='artifact' AND owner=$intent.key ORDER BY key LIMIT 64;
    FOR $root IN $roots { SELECT * FROM ONLY type::record('canonical_guards','retention:'+$root.problem) FOR UPDATE; };
    IF array::len($roots)!=0 { DELETE $roots.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    IF $guard.incarnation=$intent.primary_authority {
        UPDATE ONLY type::record('canonical_guards','retention:'+$intent.primary_problem) SET artifact_creation_closed_through=math::max([artifact_creation_closed_through ?? 0dec,$intent.creation_expires_at]);
        DELETE ONLY type::record('canonical_artifacts',$intent.key);
    } ELSE {
        // Original historical authority is already fenced. Retain its exact
        // terminal receipt for lost-reply settlement, without touching a new floor.
        UPDATE ONLY type::record('canonical_artifacts',$intent.key) SET active=false,retiring=true;
    };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:true,waiting:false};
};
"#;
