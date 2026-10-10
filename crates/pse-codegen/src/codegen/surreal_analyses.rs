// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable derived graph staging and exact result-input admission.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(FUNCTIONS);
}

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_analysis_v1::available($rpc_expiry: int, $key: string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_analyses',$key) FOR UPDATE;
    IF $row=NONE OR $row.retiring { THROW 'analysis unavailable'; };
    LET $rpc_result = $row;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_analysis_v1::begin($rpc_expiry: int, $row: object,$sources: array<object>,$inputs: array<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $authority=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$row.primary_problem) FOR UPDATE;
    LET $now=time::micros();
    IF $authority=NONE OR $authority.incarnation=NONE OR $authority.incarnation!=$row.primary_authority { THROW 'analysis primary authority unavailable'; };
    IF $row.key!='pse.analysis.v2:'+$row.primary_authority+':'+$row.creation_nonce+':'+<string><int>$row.creation_expires_at OR string::len($row.creation_nonce)!=36 { THROW 'analysis immutable creation binding'; };
    IF $row.creation_expires_at<=$now OR $row.creation_expires_at>$now+60000000 OR $row.creation_expires_at<=($authority.analysis_creation_closed_through ?? 0dec) { THROW 'analysis creation window closed'; };
    IF array::len($sources)=0 OR array::len($sources)>65 OR array::len($inputs)>64 OR $row.node_count>4096dec OR $row.edge_count>8192dec OR string::len($row.creation_request_digest)!=64 OR $row.active OR $row.retiring { THROW 'analysis admission bound'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_analyses',$row.key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.retiring OR object::remove($old,['id','active'])!=object::remove($row,['active']) { THROW 'immutable analysis identity changed'; };
        LET $retained_sources=SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$row.key LIMIT 66;
        LET $retained_inputs=SELECT key FROM canonical_analysis_inputs WHERE analysis=$row.key LIMIT 65;
        IF array::len($retained_sources)!=array::len($sources) OR array::len($retained_inputs)!=array::len($inputs) { THROW 'immutable analysis lineage coverage changed'; };
        FOR $source IN $sources {
            LET $root=SELECT * FROM ONLY type::record('canonical_roots',$source.key);
            IF $root=NONE OR $root.owner_kind!='analysis' OR $root.owner!=$row.key OR $root.problem!=$source.problem OR $root.revision!=$source.revision OR $root.sequence!=$source.sequence { THROW 'immutable analysis source changed'; };
        };
        FOR $input IN $inputs {
            LET $selected=SELECT * FROM ONLY type::record('canonical_analysis_inputs',$input.key);
            IF $selected=NONE OR $selected.analysis!=$row.key OR $selected.run!=$input.run OR $selected.attempt!=$input.attempt OR $selected.manifest!=$input.manifest { THROW 'immutable analysis input changed'; };
        };
        LET $rpc_result = $old;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    FOR $source IN $sources {
        LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$source.problem) FOR UPDATE;
        IF $guard=NONE { THROW 'analysis source authority unavailable'; };
        LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$source.revision);
        IF $revision=NONE OR $revision.problem!=$source.problem OR $revision.sequence!=$source.sequence OR $revision.interpretation!=$row.interpretation { THROW 'analysis source unavailable'; };
        LET $pruned=SELECT key FROM canonical_reclaimed_ranges WHERE problem=$source.problem AND from_sequence<=$source.sequence AND to_sequence>$source.sequence LIMIT 1;
        IF array::len($pruned)!=0 { THROW 'analysis source reclaimed'; };
    };
    IF array::len($sources[WHERE revision=$row.revision AND problem=$row.primary_problem])!=1 OR $row.revision NOT IN $sources.revision { THROW 'analysis primary source omitted'; };
    FOR $input IN $inputs {
        LET $guard=SELECT * FROM ONLY type::record('canonical_guards','execution-run:'+$input.run) FOR UPDATE;
        IF $guard=NONE { THROW 'analysis run authority unavailable'; };
        fn::pse_execution_v1::available($rpc_expiry, $input.run);
        LET $run=SELECT * FROM ONLY type::record('canonical_runs',$input.run);
        LET $attempt=SELECT * FROM ONLY type::record('canonical_attempts',$input.attempt);
        LET $manifest=SELECT * FROM ONLY type::record('canonical_result_manifests',$input.manifest);
        LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$input.protection);
        LET $selected_pin=SELECT * FROM ONLY type::record('canonical_result_protections',$input.protection);
        IF $selected_pin=NONE OR $selected_pin.run!=$input.run OR $selected_pin.attempt!=$input.attempt OR $selected_pin.manifest!=$input.manifest { THROW 'analysis result protection differs'; };
        IF $run=NONE OR $attempt=NONE OR $manifest=NONE OR $pin=NONE OR $pin.released OR $pin.expires_at<=time::micros() OR $pin.revision!=$run.revision OR $attempt.run!=$input.run OR !$attempt.terminal OR !$attempt.closed OR $attempt.ingestion_open OR $attempt.closed_manifest!=$input.manifest OR $manifest.attempt!=$input.attempt OR $manifest.generation!=$attempt.generation OR $input.analysis!=$row.key { THROW 'analysis input not admitted'; };
        LET $selected=SELECT revision FROM canonical_run_sources WHERE run=$input.run LIMIT 66;
        IF array::len($selected)>65 OR array::len($selected)=0 OR array::len(array::complement($selected.revision,$sources.revision))!=0 { THROW 'analysis omitted actual result source'; };
    };
    LET $saved=CREATE ONLY type::record('canonical_analyses',$row.key) CONTENT $row;
    FOR $source IN $sources {
        CREATE ONLY type::record('canonical_roots',$source.key) SET key=$source.key,problem=$source.problem,revision=$source.revision,sequence=$source.sequence,owner_kind='analysis',owner=$row.key;
    };
    FOR $input IN $inputs {
        INSERT RELATION INTO canonical_analysis_inputs object::extend(object::remove($input,['protection']),{id:type::record('canonical_analysis_inputs',$input.key),in:type::record('canonical_analyses',$row.key),out:type::record('canonical_attempts',$input.attempt)}) RETURN NONE;
    };
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_analysis_v1::append($rpc_expiry: int, $key: string,$nodes: array<object>,$edges: array<object>) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=fn::pse_analysis_v1::available($rpc_expiry, $key);
    IF array::len($nodes)>64 OR array::len($edges)>64 { THROW 'analysis stage batch bound'; };
    FOR $node IN $nodes {
        IF $node.analysis!=$key { THROW 'analysis node membership'; };
        LET $old=SELECT * FROM ONLY type::record('canonical_analysis_nodes',$node.key);
        IF $old!=NONE { IF object::remove($old,['id'])!=$node { THROW 'immutable analysis node changed'; }; }
        ELSE { IF $row.active { THROW 'analysis graph closed'; }; CREATE ONLY type::record('canonical_analysis_nodes',$node.key) CONTENT $node; };
    };
    FOR $edge IN $edges {
        IF $edge.analysis!=$key OR type::record('canonical_analysis_nodes',$edge.source).analysis!=$key OR type::record('canonical_analysis_nodes',$edge.target).analysis!=$key { THROW 'analysis edge membership'; };
        LET $old=SELECT * FROM ONLY type::record('canonical_analysis_edges',$edge.key);
        IF $old!=NONE { IF $old.key!=$edge.key OR $old.analysis!=$edge.analysis OR $old.source!=$edge.source OR $old.target!=$edge.target OR $old.kind!=$edge.kind OR $old.evidence!=$edge.evidence { THROW 'immutable analysis edge changed'; }; }
        ELSE { IF $row.active { THROW 'analysis graph closed'; }; INSERT RELATION INTO canonical_analysis_edges object::extend($edge,{id:type::record('canonical_analysis_edges',$edge.key),in:type::record('canonical_analysis_nodes',$edge.source),out:type::record('canonical_analysis_nodes',$edge.target)}) RETURN NONE; };
    };
    LET $rpc_result = true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_analysis_v1::activate($rpc_expiry: int, $key: string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=fn::pse_analysis_v1::available($rpc_expiry, $key);
    LET $nodes=SELECT key FROM canonical_analysis_nodes WHERE analysis=$key LIMIT 4097;
    LET $edges=SELECT key FROM canonical_analysis_edges WHERE analysis=$key LIMIT 8193;
    IF array::len($nodes)!=$row.node_count OR array::len($edges)!=$row.edge_count { THROW 'analysis membership incomplete'; };
    LET $rpc_result = UPDATE ONLY type::record('canonical_analyses',$key) SET active=true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_analysis_v1::read($rpc_expiry: int, $key: string,$after: string,$kind: string) -> array<object> {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=fn::pse_analysis_v1::available($rpc_expiry, $key);
    IF !$row.active { THROW 'analysis graph incomplete'; };
    IF $kind='nodes' { LET $rpc_result = SELECT * FROM canonical_analysis_nodes WHERE analysis=$key AND key>$after ORDER BY key LIMIT 64; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    IF $kind='edges' { LET $rpc_result = SELECT * FROM canonical_analysis_edges WHERE analysis=$key AND key>$after ORDER BY key LIMIT 64; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    THROW 'analysis page kind';
};
"#;
