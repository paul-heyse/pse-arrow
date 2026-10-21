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
    IF $row=NONE OR $row.retiring { THROW 'PSE_REFUSAL:analysis_unavailable'; };
    LET $rpc_result = $row;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_analysis_v1::begin($rpc_expiry: int, $row: object,$sources: array<object>,$inputs: array<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $authority=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$row.primary_problem) FOR UPDATE;
    LET $now=time::micros();
    IF $authority=NONE OR $authority.incarnation=NONE OR $authority.incarnation!=$row.primary_authority { THROW 'PSE_REFUSAL:analysis_primary_authority_unavailable'; };
    IF $row.key!='pse.analysis.v2:'+$row.primary_authority+':'+$row.creation_nonce+':'+<string><int>$row.creation_expires_at OR string::len($row.creation_nonce)!=36 { THROW 'PSE_REFUSAL:analysis_immutable_creation_binding'; };
    IF $row.creation_expires_at<=$now OR $row.creation_expires_at>$now+60000000 OR $row.creation_expires_at<=($authority.analysis_creation_closed_through ?? 0dec) { THROW 'PSE_REFUSAL:analysis_creation_window_closed'; };
    IF array::len($sources)=0 OR array::len($sources)>65 OR array::len($inputs)>64 OR $row.node_count>4096dec OR $row.edge_count>8192dec OR string::len($row.creation_request_digest)!=64 OR $row.active OR $row.retiring { THROW 'PSE_REFUSAL:analysis_admission_bound'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_analyses',$row.key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.retiring OR object::remove($old,['id','active'])!=object::remove($row,['active']) { THROW 'PSE_REFUSAL:immutable_analysis_identity_changed'; };
        LET $retained_sources=SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$row.key LIMIT 66;
        LET $retained_inputs=SELECT key FROM canonical_analysis_inputs WHERE analysis=$row.key LIMIT 65;
        IF array::len($retained_sources)!=array::len($sources) OR array::len($retained_inputs)!=array::len($inputs) { THROW 'PSE_REFUSAL:immutable_analysis_lineage_coverage_changed'; };
        FOR $source IN $sources {
            LET $root=SELECT * FROM ONLY type::record('canonical_roots',$source.key);
            IF $root=NONE OR $root.owner_kind!='analysis' OR $root.owner!=$row.key OR $root.problem!=$source.problem OR $root.revision!=$source.revision OR $root.sequence!=$source.sequence { THROW 'PSE_REFUSAL:immutable_analysis_source_changed'; };
        };
        FOR $input IN $inputs {
            LET $selected=SELECT * FROM ONLY type::record('canonical_analysis_inputs',$input.key);
            IF $selected=NONE OR $selected.analysis!=$row.key OR $selected.artifact!=$input.artifact { THROW 'PSE_REFUSAL:immutable_analysis_input_changed'; };
        };
        LET $rpc_result = $old;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    FOR $source IN $sources {
        fn::pse_execution_v1::source_creator($rpc_expiry,$source.problem,$source.key);
        LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$source.problem) FOR UPDATE;
        IF $guard=NONE { THROW 'PSE_REFUSAL:analysis_source_authority_unavailable'; };
        LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$source.revision);
        IF $revision=NONE OR $revision.problem!=$source.problem OR $revision.sequence!=$source.sequence OR $revision.interpretation!=$row.interpretation { THROW 'PSE_REFUSAL:analysis_source_unavailable'; };
        LET $pruned=SELECT key FROM canonical_reclaimed_ranges WHERE problem=$source.problem AND from_sequence<=$source.sequence AND to_sequence>$source.sequence LIMIT 1;
        IF array::len($pruned)!=0 { THROW 'PSE_REFUSAL:analysis_source_reclaimed'; };
    };
    IF array::len($sources[WHERE revision=$row.revision AND problem=$row.primary_problem])!=1 OR $row.revision NOT IN $sources.revision { THROW 'PSE_REFUSAL:analysis_primary_source_omitted'; };
    FOR $input IN $inputs {
        LET $artifact=fn::pse_artifact_v1::available($rpc_expiry,$input.artifact);
        LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$input.protection) FOR UPDATE;
        LET $read=SELECT * FROM ONLY type::record('canonical_artifact_reads',$input.protection);
        IF $pin=NONE OR $pin.released OR ($pin.expires_at<=time::micros() AND !fn::pse_files_v1::ticket_open($rpc_expiry,$input.protection)) OR $pin.revision!=$artifact.revision OR $read=NONE OR $read.artifact!=$input.artifact OR $input.analysis!=$row.key { THROW 'PSE_REFUSAL:analysis_artifact_input_not_admitted'; };
        LET $selected=SELECT revision FROM canonical_roots WHERE owner_kind='artifact' AND owner=$input.artifact LIMIT 66;
        IF array::len($selected)=0 OR array::len($selected)>65 OR array::len(array::complement($selected.revision,$sources.revision))!=0 { THROW 'PSE_REFUSAL:analysis_omitted_artifact_source'; };
    };
    LET $saved=CREATE ONLY type::record('canonical_analyses',$row.key) CONTENT $row;
    FOR $source IN $sources {
        CREATE ONLY type::record('canonical_roots',$source.key) SET key=$source.key,problem=$source.problem,revision=$source.revision,sequence=$source.sequence,owner_kind='analysis',owner=$row.key;
    };
    FOR $input IN $inputs {
        fn::pse_artifact_v1::attach($rpc_expiry,$input.artifact,$input.obligation,true);
        INSERT RELATION INTO canonical_analysis_inputs object::extend(object::remove($input,['protection','obligation']),{id:type::record('canonical_analysis_inputs',$input.key),in:type::record('canonical_analyses',$row.key),out:type::record('canonical_artifacts',$input.artifact)}) RETURN NONE;
    };
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_analysis_v1::append($rpc_expiry: int, $key: string,$nodes: array<object>,$edges: array<object>) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=fn::pse_analysis_v1::available($rpc_expiry, $key);
    IF array::len($nodes)>64 OR array::len($edges)>64 { THROW 'PSE_REFUSAL:analysis_stage_batch_bound'; };
    FOR $node IN $nodes {
        IF $node.analysis!=$key { THROW 'PSE_REFUSAL:analysis_node_membership'; };
        LET $old=SELECT * FROM ONLY type::record('canonical_analysis_nodes',$node.key);
        IF $old!=NONE { IF object::remove($old,['id'])!=$node { THROW 'PSE_REFUSAL:immutable_analysis_node_changed'; }; }
        ELSE { IF $row.active { THROW 'PSE_REFUSAL:analysis_graph_closed'; }; CREATE ONLY type::record('canonical_analysis_nodes',$node.key) CONTENT $node; };
    };
    LET $endpoint_keys=array::distinct(array::concat($edges.source,$edges.target));
    LET $endpoints=$endpoint_keys.map(|$endpoint| SELECT key,analysis FROM ONLY type::record('canonical_analysis_nodes',$endpoint));
    LET $evidenced=$edges.filter(|$edge| $edge.evidence_artifact!=NONE);
    LET $artifacts=array::distinct($evidenced.evidence_artifact);
    LET $inputs=$artifacts.map(|$artifact| {artifact:$artifact,matches:(SELECT key FROM canonical_analysis_inputs WHERE analysis=$key AND artifact=$artifact LIMIT 1)});
    LET $coordinates=array::distinct($evidenced.map(|$edge| {artifact:$edge.evidence_artifact,ordinal:$edge.evidence_block}));
    LET $blocks=$coordinates.map(|$coordinate| {artifact:$coordinate.artifact,ordinal:$coordinate.ordinal,matches:(SELECT rows FROM canonical_artifact_blocks WHERE artifact=$coordinate.artifact AND ordinal=$coordinate.ordinal LIMIT 1)});
    FOR $edge IN $edges {
        IF $edge.analysis!=$key OR array::len($endpoints.filter(|$endpoint| $endpoint.key=$edge.source AND $endpoint.analysis=$key))!=1 OR array::len($endpoints.filter(|$endpoint| $endpoint.key=$edge.target AND $endpoint.analysis=$key))!=1 { THROW 'PSE_REFUSAL:analysis_edge_membership'; };
        IF ($edge.evidence_artifact=NONE)!=($edge.evidence_block=NONE) OR ($edge.evidence_artifact=NONE AND ($edge.evidence_row!=NONE OR $edge.evidence_field!=NONE)) { THROW 'PSE_REFUSAL:analysis_typed_evidence_incomplete'; };
        IF $edge.evidence_artifact!=NONE {
            LET $input=$inputs.filter(|$input| $input.artifact=$edge.evidence_artifact);
            LET $block=$blocks.filter(|$block| $block.artifact=$edge.evidence_artifact AND $block.ordinal=$edge.evidence_block);
            IF array::len($input)!=1 OR array::len($input[0].matches)!=1 OR array::len($block)!=1 OR array::len($block[0].matches)!=1 OR ($edge.evidence_row!=NONE AND $edge.evidence_row>=$block[0].matches[0].rows) { THROW 'PSE_REFUSAL:analysis_evidence_outside_retained_artifact'; };
        };
        LET $old=SELECT * FROM ONLY type::record('canonical_analysis_edges',$edge.key);
        IF $old!=NONE { IF $old.key!=$edge.key OR $old.analysis!=$edge.analysis OR $old.source!=$edge.source OR $old.target!=$edge.target OR $old.kind!=$edge.kind OR $old.evidence_artifact!=$edge.evidence_artifact OR $old.evidence_block!=$edge.evidence_block OR $old.evidence_row!=$edge.evidence_row OR $old.evidence_field!=$edge.evidence_field { THROW 'PSE_REFUSAL:immutable_analysis_edge_changed'; }; }
        ELSE { IF $row.active { THROW 'PSE_REFUSAL:analysis_graph_closed'; }; INSERT RELATION INTO canonical_analysis_edges object::extend($edge,{id:type::record('canonical_analysis_edges',$edge.key),in:type::record('canonical_analysis_nodes',$edge.source),out:type::record('canonical_analysis_nodes',$edge.target)}) RETURN NONE; };
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
    IF array::len($nodes)!=$row.node_count OR array::len($edges)!=$row.edge_count { THROW 'PSE_REFUSAL:analysis_membership_incomplete'; };
    LET $rpc_result = UPDATE ONLY type::record('canonical_analyses',$key) SET active=true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_analysis_v1::read($rpc_expiry: int, $key: string,$after: string,$kind: string) -> array<object> {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $row=fn::pse_analysis_v1::available($rpc_expiry, $key);
    IF !$row.active { THROW 'PSE_REFUSAL:analysis_graph_incomplete'; };
    IF $kind='nodes' { LET $rpc_result = SELECT * FROM canonical_analysis_nodes WHERE analysis=$key AND key>$after ORDER BY key LIMIT 64; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    IF $kind='edges' { LET $rpc_result = SELECT * FROM canonical_analysis_edges WHERE analysis=$key AND key>$after ORDER BY key LIMIT 64; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    THROW 'PSE_REFUSAL:analysis_page_kind';
};
"#;
