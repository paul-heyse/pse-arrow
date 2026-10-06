// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable derived graph staging and exact result-input admission.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(FUNCTIONS);
}

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_analysis_v1::available($key: string) -> object {
    fn::pse_execution_v1::touch('analysis:'+$key);
    IF (SELECT * FROM ONLY type::record('canonical_analysis_retirements',$key))!=NONE { THROW 'analysis explicitly retired'; };
    LET $row=SELECT * FROM ONLY type::record('canonical_analyses',$key);
    IF $row=NONE { THROW 'analysis unavailable'; };
    RETURN $row;
};

DEFINE FUNCTION fn::pse_analysis_v1::begin($row: object,$sources: array<object>,$inputs: array<object>) -> object {
    fn::pse_execution_v1::touch('analysis:'+$row.key);
    IF (SELECT * FROM ONLY type::record('canonical_analysis_retirements',$row.key))!=NONE { THROW 'analysis explicitly retired'; };
    IF array::len($sources)=0 OR array::len($sources)>65 OR array::len($inputs)>64 OR $row.node_count>4096dec OR $row.edge_count>8192dec OR $row.active { THROW 'analysis admission bound'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_analyses',$row.key);
    IF $old!=NONE {
        IF object::remove($old,['id','active'])!=object::remove($row,['active']) { THROW 'immutable analysis identity changed'; };
        RETURN $old;
    };
    FOR $source IN $sources {
        fn::pse_execution_v1::touch('retention:'+$source.problem);
        LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$source.revision);
        IF $revision=NONE OR $revision.problem!=$source.problem OR $revision.sequence!=$source.sequence OR $revision.interpretation!=$row.interpretation { THROW 'analysis source unavailable'; };
        LET $pruned=SELECT key FROM canonical_reclaimed_ranges WHERE problem=$source.problem AND from_sequence<=$source.sequence AND to_sequence>$source.sequence LIMIT 1;
        IF array::len($pruned)!=0 { THROW 'analysis source reclaimed'; };
    };
    IF $row.revision NOT IN $sources.revision { THROW 'analysis primary source omitted'; };
    FOR $input IN $inputs {
        fn::pse_execution_v1::available($input.run);
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
    RETURN $saved;
};

DEFINE FUNCTION fn::pse_analysis_v1::append($key: string,$nodes: array<object>,$edges: array<object>) -> bool {
    LET $row=fn::pse_analysis_v1::available($key);
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
    RETURN true;
};

DEFINE FUNCTION fn::pse_analysis_v1::activate($key: string) -> object {
    LET $row=fn::pse_analysis_v1::available($key);
    LET $nodes=SELECT key FROM canonical_analysis_nodes WHERE analysis=$key LIMIT 4097;
    LET $edges=SELECT key FROM canonical_analysis_edges WHERE analysis=$key LIMIT 8193;
    IF array::len($nodes)!=$row.node_count OR array::len($edges)!=$row.edge_count { THROW 'analysis membership incomplete'; };
    RETURN UPDATE ONLY type::record('canonical_analyses',$key) SET active=true;
};

DEFINE FUNCTION fn::pse_analysis_v1::read($key: string,$after: string,$kind: string) -> array<object> {
    LET $row=fn::pse_analysis_v1::available($key);
    IF !$row.active { THROW 'analysis graph incomplete'; };
    IF $kind='nodes' { RETURN SELECT * FROM canonical_analysis_nodes WHERE analysis=$key AND key>$after ORDER BY key LIMIT 64; };
    IF $kind='edges' { RETURN SELECT * FROM canonical_analysis_edges WHERE analysis=$key AND key>$after ORDER BY key LIMIT 64; };
    THROW 'analysis page kind';
};
"#;
