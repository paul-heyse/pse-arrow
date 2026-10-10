// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit withdrawal and bounded cleanup of connected scientific results.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(FUNCTIONS);
}

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_retention_v1::forget_study($rpc_expiry: int, $study:string) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:'+$study);
    LET $header=SELECT * FROM ONLY type::record('canonical_studies',$study);
    IF $header=NONE OR !$header.terminal { THROW 'only terminal studies may be retired'; };
    UPSERT type::record('canonical_study_retirements',$study) SET key=$study,study=$study;
    LET $rpc_result = true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

// The occurrence key carries its immutable authority and expiry even after header deletion.
DEFINE FUNCTION fn::pse_retention_v1::forget_analysis($rpc_expiry: int, $analysis:string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $header=SELECT * FROM ONLY type::record('canonical_analyses',$analysis) FOR UPDATE;
    IF $header=NONE {
        LET $parts=string::split($analysis,':');
        IF array::len($parts)!=4 OR $parts[0]!='pse.analysis.v2' OR string::len($parts[1])!=36 OR string::len($parts[2])!=36 { THROW 'unknown analysis key has no creation authority'; };
        LET $authorities=SELECT key FROM canonical_guards WITH INDEX guard_incarnation WHERE incarnation=$parts[1] LIMIT 2;
        IF array::len($authorities)!=1 OR !string::starts_with($authorities[0].key,'retention:') { THROW 'analysis creation authority unavailable'; };
        LET $intent={key:$analysis,primary_problem:string::slice($authorities[0].key,10),primary_authority:$parts[1],creation_nonce:$parts[2],creation_expires_at:<decimal>$parts[3]};
        RETURN fn::pse_retention_v1::settle_analysis($rpc_expiry,$intent);
    };
    UPDATE ONLY type::record('canonical_analyses',$analysis) SET retiring=true;
    LET $edges=SELECT id FROM canonical_analysis_edges WHERE analysis=$analysis ORDER BY key LIMIT 64;
    IF array::len($edges)!=0 { DELETE $edges.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $nodes=SELECT id FROM canonical_analysis_nodes WHERE analysis=$analysis ORDER BY key LIMIT 64;
    IF array::len($nodes)!=0 { DELETE $nodes.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $inputs=SELECT id FROM canonical_analysis_inputs WHERE analysis=$analysis ORDER BY key LIMIT 64;
    IF array::len($inputs)!=0 { DELETE $inputs.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $roots=SELECT id,problem FROM canonical_roots WHERE owner_kind='analysis' AND owner=$analysis ORDER BY key LIMIT 64;
    IF array::len($roots)!=0 {
        FOR $root IN $roots {
            LET $source=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$root.problem) FOR UPDATE;
            IF $source=NONE { THROW 'analysis cleanup source authority unavailable'; };
        };
        DELETE $roots.id;
        fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false};
    };
    IF time::micros()<$header.creation_expires_at { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:true}; };
    RETURN fn::pse_retention_v1::settle_analysis($rpc_expiry,$header);
};

DEFINE FUNCTION fn::pse_retention_v1::settle_analysis($rpc_expiry:int,$intent:object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $header=SELECT * FROM ONLY type::record('canonical_analyses',$intent.key) FOR UPDATE;
    LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$intent.primary_problem) FOR UPDATE;
    IF $guard=NONE OR $guard.incarnation!=$intent.primary_authority { THROW 'analysis settlement authority unavailable'; };
    IF $intent.key!='pse.analysis.v2:'+$intent.primary_authority+':'+$intent.creation_nonce+':'+<string><int>$intent.creation_expires_at { THROW 'analysis settlement key binding'; };
    IF time::micros()<$intent.creation_expires_at { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:true}; };
    IF $header!=NONE AND ($header.primary_authority!=$intent.primary_authority OR $header.creation_expires_at!=$intent.creation_expires_at) { THROW 'analysis settlement requires original occurrence'; };
    IF $header!=NONE AND !$header.retiring { UPDATE ONLY type::record('canonical_analyses',$intent.key) SET retiring=true; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $edges=SELECT key FROM canonical_analysis_edges WHERE analysis=$intent.key LIMIT 1;
    LET $nodes=SELECT key FROM canonical_analysis_nodes WHERE analysis=$intent.key LIMIT 1;
    LET $inputs=SELECT key FROM canonical_analysis_inputs WHERE analysis=$intent.key LIMIT 1;
    LET $roots=SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$intent.key LIMIT 1;
    IF array::len($edges)+array::len($nodes)+array::len($inputs)+array::len($roots)!=0 { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    UPDATE ONLY type::record('canonical_guards','retention:'+$intent.primary_problem) SET generation=generation+1dec, analysis_creation_closed_through=math::max([analysis_creation_closed_through ?? 0dec,$intent.creation_expires_at]);
    DELETE ONLY type::record('canonical_analyses',$intent.key);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {complete:true,waiting:false};
};

DEFINE FUNCTION fn::pse_retention_v1::forget_run($rpc_expiry: int, $run:string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:'+$run);
    LET $old=SELECT * FROM ONLY type::record('canonical_result_retirements',$run);
    IF $old!=NONE { LET $rpc_result = $old; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $header=SELECT * FROM ONLY type::record('canonical_runs',$run);
    IF $header=NONE { THROW 'retired run unavailable'; };
    fn::pse_execution_v1::touch($rpc_expiry, 'retention:'+$header.problem);
    LET $live=SELECT key FROM canonical_attempts WHERE run=$run AND terminal=false LIMIT 1;
    IF array::len($live)!=0 OR $header.terminal_attempt=NONE { THROW 'run recovery must finish before retirement'; };
    LET $readers=SELECT key FROM canonical_result_protections WHERE run=$run AND type::record('canonical_protections',key).released=false AND type::record('canonical_protections',key).expires_at>time::micros() LIMIT 1;
    IF array::len($readers)!=0 { THROW 'run results have protected readers'; };
    LET $point=SELECT study FROM canonical_study_points WHERE run=$run LIMIT 1;
    LET $parent=SELECT key FROM canonical_studies WHERE run=$run LIMIT 1;
    LET $study=IF array::len($point)!=0 {$point[0].study} ELSE IF array::len($parent)!=0 {$parent[0].key} ELSE {NONE};
    IF $study!=NONE {
        fn::pse_execution_v1::touch($rpc_expiry, 'study:'+$study);
        IF (SELECT * FROM ONLY type::record('canonical_study_retirements',$study))=NONE { THROW 'run results are retained by a study'; };
    };
    LET $analyses=SELECT analysis FROM canonical_analysis_inputs WHERE run=$run LIMIT 1;
    IF array::len($analyses)!=0 { THROW 'run results are retained by an analysis'; };
    LET $roots=SELECT key,problem FROM canonical_roots WHERE owner_kind='run' AND owner=$run ORDER BY key LIMIT 66;
    IF array::len($roots)>65 { THROW 'run source retention limit'; };
    FOR $root IN $roots { fn::pse_execution_v1::touch($rpc_expiry, 'retention:'+$root.problem); };
    DELETE $roots.map(|$root|type::record('canonical_roots',$root.key));
    LET $rpc_result = CREATE ONLY type::record('canonical_result_retirements',$run) SET key=$run,run=$run,after_generation=0dec,complete=false;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_retention_v1::collect_run($rpc_expiry: int, $run:string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:'+$run);
    LET $retirement=SELECT * FROM ONLY type::record('canonical_result_retirements',$run);
    IF $retirement=NONE { THROW 'explicit run retirement required'; };
    IF $retirement.complete { RETURN {complete:true,batches:0dec,indexes:0dec,sets:0dec}; };
    LET $attempts=SELECT key,generation FROM canonical_attempts WHERE run=$run AND generation>$retirement.after_generation ORDER BY generation LIMIT 1;
    IF array::len($attempts)=0 {
        UPDATE ONLY type::record('canonical_result_retirements',$run) SET complete=true;
        LET $rpc_result = {complete:true,batches:0dec,indexes:0dec,sets:0dec};
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    LET $attempt=$attempts[0];
    LET $sets=SELECT key FROM canonical_result_sets WHERE attempt=$attempt.key ORDER BY name LIMIT 1;
    IF array::len($sets)!=0 {
        LET $set=$sets[0].key;
        LET $batches=SELECT key FROM canonical_result_batches WHERE result_set=$set ORDER BY ordinal LIMIT 1;
        IF array::len($batches)!=0 {
            LET $batch=$batches[0].key;
            LET $cells=SELECT key FROM canonical_result_cells WHERE batch=$batch ORDER BY key LIMIT 64;
            LET $outputs=SELECT key FROM canonical_result_block_outputs WHERE batch=$batch ORDER BY key LIMIT 64;
            DELETE $cells.map(|$row|type::record('canonical_result_cells',$row.key));
            DELETE $outputs.map(|$row|type::record('canonical_result_block_outputs',$row.key));
            LET $remaining_cells=SELECT key FROM canonical_result_cells WHERE batch=$batch LIMIT 1;
            LET $remaining_outputs=SELECT key FROM canonical_result_block_outputs WHERE batch=$batch LIMIT 1;
            LET $deleted=array::len($remaining_cells)=0 AND array::len($remaining_outputs)=0;
            IF $deleted { DELETE ONLY type::record('canonical_result_blocks',$batch); DELETE ONLY type::record('canonical_result_batches',$batch); };
            LET $rpc_result = {complete:false,batches:IF $deleted {1dec} ELSE {0dec},indexes:<decimal>(array::len($cells)+array::len($outputs)),sets:0dec};
            fn::pse_execution_v1::deadline($rpc_expiry);
            RETURN $rpc_result;
        };
        DELETE ONLY type::record('canonical_result_sets',$set);
        LET $rpc_result = {complete:false,batches:0dec,indexes:0dec,sets:1dec};
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    LET $seeds=SELECT key FROM canonical_result_seeds WHERE attempt=$attempt.key ORDER BY step LIMIT 64;
    DELETE $seeds.map(|$row|type::record('canonical_result_seeds',$row.key));
    IF array::len($seeds)=64 { RETURN {complete:false,batches:0dec,indexes:<decimal>array::len($seeds),sets:0dec}; };
    LET $manifest=SELECT closed_manifest FROM ONLY type::record('canonical_attempts',$attempt.key);
    IF $manifest.closed_manifest!=NONE { DELETE ONLY type::record('canonical_result_manifests',$manifest.closed_manifest); };
    // Inline observations and operation response bytes are scientific payloads,
    // while the effect identity/request and actual terminal class remain receipts.
    UPDATE ONLY type::record('canonical_attempts',$attempt.key) SET completion=NONE;
    LET $operations=SELECT key FROM canonical_execution_operations WHERE run=$run AND kind='seal' AND attempt=$attempt.key LIMIT 1;
    FOR $operation IN $operations { UPDATE ONLY type::record('canonical_execution_operations',$operation.key) SET result=type::bytes(''); };
    UPDATE ONLY type::record('canonical_result_retirements',$run) SET after_generation=$attempt.generation;
    LET $rpc_result = {complete:false,batches:0dec,indexes:<decimal>array::len($seeds),sets:0dec};
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};
"#;
