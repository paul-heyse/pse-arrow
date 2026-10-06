// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit withdrawal and bounded cleanup of connected scientific results.

pub(super) fn append(ddl: &mut String) { ddl.push_str(FUNCTIONS); }

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_retention_v1::forget_study($study:string) -> bool {
    fn::pse_execution_v1::touch('study:'+$study);
    LET $header=SELECT * FROM ONLY type::record('canonical_studies',$study);
    IF $header=NONE OR !$header.terminal { THROW 'only terminal studies may be retired'; };
    UPSERT type::record('canonical_study_retirements',$study) SET key=$study,study=$study;
    RETURN true;
};

DEFINE FUNCTION fn::pse_retention_v1::forget_analysis($analysis:string) -> bool {
    fn::pse_execution_v1::touch('analysis:'+$analysis);
    LET $header=SELECT * FROM ONLY type::record('canonical_analyses',$analysis);
    IF $header=NONE { THROW 'analysis unavailable'; };
    LET $roots=SELECT key,problem FROM canonical_roots WHERE owner_kind='analysis' AND owner=$analysis ORDER BY key LIMIT 66;
    IF array::len($roots)>65 { THROW 'analysis source retention limit'; };
    FOR $root IN $roots { fn::pse_execution_v1::touch('retention:'+$root.problem); };
    DELETE $roots.map(|$root|type::record('canonical_roots',$root.key));
    UPSERT type::record('canonical_analysis_retirements',$analysis) SET key=$analysis,analysis=$analysis;
    RETURN true;
};

DEFINE FUNCTION fn::pse_retention_v1::forget_run($run:string) -> object {
    fn::pse_execution_v1::touch('execution-run:'+$run);
    LET $old=SELECT * FROM ONLY type::record('canonical_result_retirements',$run);
    IF $old!=NONE { RETURN $old; };
    LET $header=SELECT * FROM ONLY type::record('canonical_runs',$run);
    IF $header=NONE { THROW 'retired run unavailable'; };
    fn::pse_execution_v1::touch('retention:'+$header.problem);
    LET $live=SELECT key FROM canonical_attempts WHERE run=$run AND terminal=false LIMIT 1;
    IF array::len($live)!=0 OR $header.terminal_attempt=NONE { THROW 'run recovery must finish before retirement'; };
    LET $readers=SELECT key FROM canonical_result_protections WHERE run=$run AND type::record('canonical_protections',key).released=false AND type::record('canonical_protections',key).expires_at>time::micros() LIMIT 1;
    IF array::len($readers)!=0 { THROW 'run results have protected readers'; };
    LET $point=SELECT study FROM canonical_study_points WHERE run=$run LIMIT 1;
    LET $parent=SELECT key FROM canonical_studies WHERE run=$run LIMIT 1;
    LET $study=IF array::len($point)!=0 {$point[0].study} ELSE IF array::len($parent)!=0 {$parent[0].key} ELSE {NONE};
    IF $study!=NONE {
        fn::pse_execution_v1::touch('study:'+$study);
        IF (SELECT * FROM ONLY type::record('canonical_study_retirements',$study))=NONE { THROW 'run results are retained by a study'; };
    };
    LET $analyses=SELECT analysis FROM canonical_analysis_inputs WHERE run=$run AND type::record('canonical_analysis_retirements',analysis).analysis=NONE LIMIT 1;
    IF array::len($analyses)!=0 { THROW 'run results are retained by an analysis'; };
    LET $roots=SELECT key,problem FROM canonical_roots WHERE owner_kind='run' AND owner=$run ORDER BY key LIMIT 66;
    IF array::len($roots)>65 { THROW 'run source retention limit'; };
    FOR $root IN $roots { fn::pse_execution_v1::touch('retention:'+$root.problem); };
    DELETE $roots.map(|$root|type::record('canonical_roots',$root.key));
    RETURN CREATE ONLY type::record('canonical_result_retirements',$run) SET key=$run,run=$run,after_generation=0dec,complete=false;
};

DEFINE FUNCTION fn::pse_retention_v1::collect_run($run:string) -> object {
    fn::pse_execution_v1::touch('execution-run:'+$run);
    LET $retirement=SELECT * FROM ONLY type::record('canonical_result_retirements',$run);
    IF $retirement=NONE { THROW 'explicit run retirement required'; };
    IF $retirement.complete { RETURN {complete:true,batches:0dec,indexes:0dec,sets:0dec}; };
    LET $attempts=SELECT key,generation FROM canonical_attempts WHERE run=$run AND generation>$retirement.after_generation ORDER BY generation LIMIT 1;
    IF array::len($attempts)=0 {
        UPDATE ONLY type::record('canonical_result_retirements',$run) SET complete=true;
        RETURN {complete:true,batches:0dec,indexes:0dec,sets:0dec};
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
            RETURN {complete:false,batches:IF $deleted {1dec} ELSE {0dec},indexes:<decimal>(array::len($cells)+array::len($outputs)),sets:0dec};
        };
        DELETE ONLY type::record('canonical_result_sets',$set);
        RETURN {complete:false,batches:0dec,indexes:0dec,sets:1dec};
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
    RETURN {complete:false,batches:0dec,indexes:<decimal>array::len($seeds),sets:0dec};
};
"#;
