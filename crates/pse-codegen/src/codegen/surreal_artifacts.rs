// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned selected scientific closure and attributed lifetime operations.

pub(super) fn append(ddl: &mut String) { ddl.push_str(FUNCTIONS); }

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_artifact_v1::authority($rpc_expiry:int,$row:object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$row.primary_problem) FOR UPDATE;
    IF $guard=NONE OR $guard.incarnation!=$row.primary_authority OR $row.key!='pse.artifact.v1:'+$row.primary_authority+':'+$row.creation_nonce+':'+<string><int>$row.creation_expires_at { THROW 'artifact creation authority differs'; };
    RETURN $guard;
};
DEFINE FUNCTION fn::pse_artifact_v1::available($rpc_expiry:int,$key:string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$key) FOR UPDATE;
    IF $row=NONE OR !$row.active OR $row.retiring { THROW 'artifact unavailable'; };
    RETURN $row;
};
DEFINE FUNCTION fn::pse_artifact_v1::producer($rpc_expiry:int,$input:object) -> bool {
    fn::pse_execution_v1::result_creator($rpc_expiry,$input.run);
    fn::pse_execution_v1::available($rpc_expiry,$input.run);
    LET $attempt=SELECT * FROM ONLY type::record('canonical_attempts',$input.attempt);
    LET $manifest=SELECT * FROM ONLY type::record('canonical_result_manifests',$input.manifest);
    IF $attempt=NONE OR $manifest=NONE OR $attempt.run!=$input.run OR !$attempt.terminal OR !$attempt.closed OR $attempt.ingestion_open OR $attempt.closed_manifest!=$input.manifest OR $manifest.attempt!=$input.attempt OR $manifest.generation!=$attempt.generation { THROW 'artifact producer inventory unavailable'; };
    RETURN true;
};
DEFINE FUNCTION fn::pse_artifact_v1::begin($rpc_expiry:int,$row:object,$sources:array<object>,$inputs:array<object>) -> object {
    LET $guard=fn::pse_artifact_v1::authority($rpc_expiry,$row);
    LET $now=time::micros();
    IF $row.creation_expires_at<=$now OR $row.creation_expires_at>$now+60000000 OR $row.creation_expires_at<=($guard.artifact_creation_closed_through ?? 0dec) { THROW 'artifact creation window closed'; };
    IF $row.active OR $row.retiring OR string::len($row.creation_nonce)!=36 OR string::len($row.creation_request_digest)!=64 OR array::len($sources)=0 OR array::len($sources)>65 OR array::len($inputs)>64 OR $row.block_count=0dec OR bytes::len($row.provenance)>131072 { THROW 'artifact closure admission bound'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_artifacts',$row.key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.retiring OR object::remove($old,['id','active'])!=object::remove($row,['active']) { THROW 'immutable artifact request changed'; };
        RETURN $old;
    };
    FOR $source IN $sources {
        fn::pse_execution_v1::source_creator($rpc_expiry,$source.problem,$source.key);
        LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$source.revision);
        LET $pruned=SELECT key FROM canonical_reclaimed_ranges WHERE problem=$source.problem AND from_sequence<=$source.sequence AND to_sequence>$source.sequence LIMIT 1;
        IF $revision=NONE OR $revision.problem!=$source.problem OR $revision.sequence!=$source.sequence OR $revision.interpretation!=$row.interpretation OR array::len($pruned)!=0 { THROW 'artifact source unavailable'; };
    };
    IF array::len($sources[WHERE revision=$row.revision AND problem=$row.primary_problem])!=1 { THROW 'artifact primary source omitted'; };
    FOR $input IN $inputs {
        fn::pse_artifact_v1::producer($rpc_expiry,$input);
        LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$input.protection) FOR UPDATE;
        LET $selected=SELECT * FROM ONLY type::record('canonical_result_protections',$input.protection);
        IF $pin=NONE OR $pin.released OR $pin.expires_at<=time::micros() OR $selected=NONE OR $selected.run!=$input.run OR $selected.attempt!=$input.attempt OR $selected.manifest!=$input.manifest { THROW 'artifact exact producer protection absent'; };
        LET $actual=SELECT revision FROM canonical_run_sources WHERE run=$input.run LIMIT 66;
        IF array::len($actual)=0 OR array::len($actual)>65 OR array::len(array::complement($actual.revision,$sources.revision))!=0 { THROW 'artifact actual source omitted'; };
    };
    CREATE ONLY type::record('canonical_artifacts',$row.key) CONTENT $row;
    FOR $source IN $sources { CREATE ONLY type::record('canonical_roots',$source.key) SET key=$source.key,problem=$source.problem,revision=$source.revision,sequence=$source.sequence,owner_kind='artifact',owner=$row.key; };
    FOR $input IN $inputs { CREATE ONLY type::record('canonical_artifact_staging_inputs',$input.key) CONTENT object::remove($input,['protection']); };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $row;
};
DEFINE FUNCTION fn::pse_artifact_v1::append($rpc_expiry:int,$key:string,$block:object) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$key) FOR UPDATE;
    IF $row=NONE OR $row.retiring OR $block.artifact!=$key OR $block.ordinal>=$row.block_count OR bytes::len($block.payload)>524288 OR string::len($block.digest)!=64 { THROW 'artifact block admission'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_artifact_blocks',$block.key);
    IF $old!=NONE { IF object::remove($old,['id'])!=$block { THROW 'immutable artifact block changed'; }; }
    ELSE { IF $row.active { THROW 'artifact content sealed'; }; CREATE ONLY type::record('canonical_artifact_blocks',$block.key) CONTENT $block; };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN true;
};
DEFINE FUNCTION fn::pse_artifact_v1::activate($rpc_expiry:int,$key:string,$obligation:object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$key) FOR UPDATE;
    IF $row=NONE OR $row.retiring { THROW 'artifact staging unavailable'; };
    fn::pse_artifact_v1::authority($rpc_expiry,$row);
    IF !$row.active AND $row.creation_expires_at<=time::micros() { THROW 'artifact creation expired before activation'; };
    LET $inputs=SELECT * FROM canonical_artifact_staging_inputs WHERE artifact=$key ORDER BY key LIMIT 65;
    IF array::len($inputs)>64 { THROW 'artifact input bound'; };
    FOR $input IN $inputs { fn::pse_artifact_v1::producer($rpc_expiry,$input); };
    LET $coverage=SELECT count() AS n, math::sum(bytes::len(payload)) AS bytes FROM canonical_artifact_blocks WHERE artifact=$key GROUP ALL;
    IF $coverage[0].n!=$row.block_count OR $coverage[0].bytes!=$row.payload_bytes { THROW 'artifact content incomplete'; };
    fn::pse_artifact_v1::attach($rpc_expiry,$key,$obligation,false);
    DELETE $inputs.id;
    LET $saved=UPDATE ONLY type::record('canonical_artifacts',$key) SET active=true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $saved;
};
DEFINE FUNCTION fn::pse_artifact_v1::attach($rpc_expiry:int,$key:string,$obligation:object,$require_active:bool) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$key) FOR UPDATE;
    IF $row=NONE OR $row.retiring OR ($require_active AND !$row.active) OR $obligation.artifact!=$key OR $obligation.owner_kind NOT IN ['study','user','project','analysis','audit'] OR string::len($obligation.owner)=0 { THROW 'artifact obligation unavailable'; };
    IF $obligation.owner_kind='study' { fn::pse_execution_v1::register($rpc_expiry,'study:'+$obligation.owner); IF (SELECT * FROM ONLY type::record('canonical_study_retirements',$obligation.owner))!=NONE { THROW 'study artifact owner retired'; }; };
    IF $obligation.owner_kind='analysis' { LET $analysis=SELECT * FROM ONLY type::record('canonical_analyses',$obligation.owner) FOR UPDATE; IF $analysis=NONE OR $analysis.retiring { THROW 'analysis artifact owner retired'; }; };
    LET $old=SELECT * FROM ONLY type::record('canonical_artifact_obligations',$obligation.key) FOR UPDATE;
    IF $old!=NONE { IF object::remove($old,['id'])!=$obligation { THROW 'artifact obligation identity changed'; }; }
    ELSE { CREATE ONLY type::record('canonical_artifact_obligations',$obligation.key) CONTENT $obligation; };
    RETURN true;
};
DEFINE FUNCTION fn::pse_artifact_v1::release($rpc_expiry:int,$obligation:object) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    SELECT * FROM ONLY type::record('canonical_artifacts',$obligation.artifact) FOR UPDATE;
    LET $old=SELECT * FROM ONLY type::record('canonical_artifact_obligations',$obligation.key) FOR UPDATE;
    IF $old!=NONE AND object::remove($old,['id'])!=$obligation { THROW 'artifact release owner differs'; };
    IF $obligation.owner_kind='study' {
        fn::pse_execution_v1::register($rpc_expiry,'study:'+$obligation.owner);
        IF (SELECT * FROM ONLY type::record('canonical_study_retirements',$obligation.owner))=NONE { THROW 'study input release requires explicit study retirement'; };
    };
    DELETE ONLY type::record('canonical_artifact_obligations',$obligation.key);
    RETURN true;
};
DEFINE FUNCTION fn::pse_artifact_v1::acquire($rpc_expiry:int,$key:string,$protection:string) -> object {
    LET $row=fn::pse_artifact_v1::available($rpc_expiry,$key);
    LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$protection) FOR UPDATE;
    IF $pin=NONE OR $pin.released OR $pin.expires_at<=time::micros() OR $pin.revision!=$row.revision { THROW 'artifact read protection absent'; };
    UPSERT type::record('canonical_artifact_reads',$protection) SET key=$protection,artifact=$key;
    RETURN $row;
};
DEFINE FUNCTION fn::pse_artifact_v1::read($rpc_expiry:int,$key:string,$protection:string,$ordinal:decimal) -> object {
    LET $row=fn::pse_artifact_v1::available($rpc_expiry,$key);
    LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$protection) FOR UPDATE;
    LET $read=SELECT * FROM ONLY type::record('canonical_artifact_reads',$protection);
    IF $pin=NONE OR $pin.released OR $pin.expires_at<=time::micros() OR $pin.revision!=$row.revision OR $read=NONE OR $read.artifact!=$key { THROW 'artifact read grant unavailable'; };
    LET $blocks=SELECT * FROM canonical_artifact_blocks WHERE artifact=$key AND ordinal=$ordinal LIMIT 1;
    IF array::len($blocks)!=1 { THROW 'artifact block unavailable'; };
    RETURN $blocks[0];
};
DEFINE FUNCTION fn::pse_artifact_v1::collect($rpc_expiry:int,$intent:object) -> object {
    LET $guard=fn::pse_artifact_v1::authority($rpc_expiry,$intent);
    LET $row=SELECT * FROM ONLY type::record('canonical_artifacts',$intent.key) FOR UPDATE;
    LET $owners=SELECT key FROM canonical_artifact_obligations WHERE artifact=$intent.key LIMIT 1;
    IF array::len($owners)!=0 { THROW 'artifact has retained obligations'; };
    IF $row!=NONE AND ($row.creation_request_digest!=$intent.creation_request_digest OR $row.creation_expires_at!=$intent.creation_expires_at) { THROW 'artifact settlement requires original intent'; };
    LET $reads=SELECT key FROM canonical_artifact_reads WHERE artifact=$intent.key ORDER BY key LIMIT 64;
    FOR $read IN $reads {
        LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$read.key) FOR UPDATE;
        IF $pin!=NONE AND !$pin.released AND $pin.expires_at>time::micros() { RETURN {complete:false,waiting:true}; };
        LET $grant=fn::pse_execution_v1::protection_grant($rpc_expiry,$intent.primary_problem,$read.key,false);
        IF time::micros()<$grant.grant_deadline { RETURN {complete:false,waiting:true}; };
        fn::pse_retention_v1::settle_protection($rpc_expiry,$intent.primary_problem,$read.key);
        DELETE ONLY type::record('canonical_artifact_reads',$read.key);
    };
    IF array::len($reads)=64 { RETURN {complete:false,waiting:false}; };
    IF $row!=NONE { UPDATE ONLY type::record('canonical_artifacts',$intent.key) SET retiring=true; };
    IF time::micros()<$intent.creation_expires_at { RETURN {complete:false,waiting:true}; };
    LET $blocks=SELECT id FROM canonical_artifact_blocks WHERE artifact=$intent.key ORDER BY ordinal LIMIT 64;
    IF array::len($blocks)!=0 { DELETE $blocks.id; RETURN {complete:false,waiting:false}; };
    LET $inputs=SELECT id,run FROM canonical_artifact_staging_inputs WHERE artifact=$intent.key ORDER BY key LIMIT 64;
    FOR $input IN $inputs { fn::pse_execution_v1::result_creator($rpc_expiry,$input.run); };
    IF array::len($inputs)!=0 { DELETE $inputs.id; RETURN {complete:false,waiting:false}; };
    LET $roots=SELECT id,problem FROM canonical_roots WHERE owner_kind='artifact' AND owner=$intent.key ORDER BY key LIMIT 64;
    FOR $root IN $roots { SELECT * FROM ONLY type::record('canonical_guards','retention:'+$root.problem) FOR UPDATE; };
    IF array::len($roots)!=0 { DELETE $roots.id; RETURN {complete:false,waiting:false}; };
    UPDATE ONLY type::record('canonical_guards','retention:'+$intent.primary_problem) SET artifact_creation_closed_through=math::max([artifact_creation_closed_through ?? 0dec,$intent.creation_expires_at]);
    DELETE ONLY type::record('canonical_artifacts',$intent.key);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {complete:true,waiting:false};
};
"#;
