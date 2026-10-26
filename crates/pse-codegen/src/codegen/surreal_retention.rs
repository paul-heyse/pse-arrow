// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit withdrawal and bounded cleanup of connected scientific results.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(FUNCTIONS);
}

const FUNCTIONS: &str = r#"

// One physical source-deletion decision shared by stage and membership cleanup.
DEFINE FUNCTION fn::pse_retention_v1::delete_source_version($pse_rpc_expires_at:int,$version:string,$excluded_membership:option<string>,$excluded_stage:option<string>) -> object {
    fn::pse_workbench_v1::publication($pse_rpc_expires_at);
LET $version_guard = type::record('canonical_guards', 'version:' + $version);
SELECT * FROM $version_guard FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $members = SELECT key FROM canonical_memberships WHERE version = $version AND ($excluded_membership = NONE OR key != $excluded_membership) LIMIT 1;
LET $stages = SELECT key FROM canonical_staged_edits WHERE version = $version AND ($excluded_stage = NONE OR stage != $excluded_stage)
    AND type::record('canonical_stages', stage).activated = false AND type::record('canonical_stages', stage).abandoned = false
    AND type::record('canonical_stages', stage).expires_at > time::micros() LIMIT 1;
LET $products = SELECT key FROM canonical_product_associations WHERE (content = $version OR receipt = $version) AND type::record('canonical_roots',key).owner_kind = 'product' AND type::record('canonical_roots',key).owner = key AND type::record('canonical_roots',key).problem = problem AND type::record('canonical_roots',key).revision = revision LIMIT 1;
LET $deleted = IF array::len($members) != 0 OR array::len($stages) != 0 OR array::len($products) != 0 {
    {version:false, blocks:0dec, edges:0dec, pending:false}
} ELSE {
    LET $manifest = SELECT * FROM ONLY type::record('canonical_version_manifests', $version);
    UPDATE type::record('canonical_version_manifests', $version) SET closed = false;
    LET $blocks = SELECT key FROM canonical_payload_blocks WHERE version = $version ORDER BY ordinal LIMIT 64;
    LET $edges = SELECT key FROM canonical_edges WHERE source_version = $version ORDER BY ordinal LIMIT 64;
    FOR $block IN $blocks { DELETE ONLY type::record('canonical_payload_blocks', $block.key) RETURN NONE; };
    FOR $edge IN $edges { DELETE ONLY type::record('canonical_edges', $edge.key); };
    LET $remaining_blocks = SELECT key FROM canonical_payload_blocks WHERE version = $version LIMIT 1;
    LET $remaining_edges = SELECT key FROM canonical_edges WHERE source_version = $version LIMIT 1;
    IF array::len($remaining_blocks) = 0 AND array::len($remaining_edges) = 0 {
        DELETE ONLY type::record('canonical_version_manifests', $version);
        {version:$manifest != NONE, blocks:<decimal>array::len($blocks), edges:<decimal>array::len($edges), pending:false}
    } ELSE {
        {version:false, blocks:<decimal>array::len($blocks), edges:<decimal>array::len($edges), pending:true}
    };
};
UPSERT $version_guard SET key = 'version:' + $version, generation = (generation ?? 0dec) + 1dec;
    RETURN $deleted;
};

// Cursor and the candidate's full fresh decision commit in the same native transaction.
DEFINE FUNCTION fn::pse_retention_v1::source_step($pse_rpc_expires_at:int,$problem:string) -> object {
    fn::pse_workbench_v1::publication($pse_rpc_expires_at);
    fn::pse_execution_v1::register($pse_rpc_expires_at,'source-creators:'+$problem);
    LET $guard = type::record('canonical_guards','retention:'+$problem);
    LET $position = SELECT * FROM ONLY $guard FOR UPDATE;
    fn::pse_execution_v1::deadline($pse_rpc_expires_at);
    IF $position=NONE { THROW 'PSE_REFUSAL:reclamation_problem_head_unavailable'; };
    LET $after = $position.source_reclaim_after ?? '';
    LET $candidates = SELECT key FROM canonical_memberships WHERE problem=$problem AND key>$after AND to_sequence!=NONE ORDER BY key LIMIT 1;
    IF array::len($candidates)=0 {
        UPDATE ONLY $guard SET source_reclaim_after=NONE,generation=generation+1dec;
        RETURN {done:true,after:NONE,membership:false,version:false,edges:0dec,blocks:0dec,pending:false};
    };
    LET $candidate = $candidates[0].key;
LET $member = SELECT * FROM ONLY type::record('canonical_memberships', $candidate) FOR UPDATE;
LET $result = IF $member = NONE OR $member.problem != $problem OR $member.to_sequence = NONE {
    {membership:false, version:false, edges:0dec, blocks:0dec, pending:false}
} ELSE {
    LET $head = SELECT * FROM ONLY type::record('canonical_problems', $problem);
    IF $head = NONE { THROW 'PSE_REFUSAL:reclamation_problem_head_unavailable'; };
    LET $roots = SELECT key FROM canonical_roots WHERE problem = $problem AND sequence >= $member.from_sequence AND sequence < $member.to_sequence LIMIT 1;
    LET $captures=SELECT key FROM canonical_study_capture_protections WHERE problem=$problem
        AND type::record('canonical_revisions',revision).sequence >= $member.from_sequence
        AND type::record('canonical_revisions',revision).sequence < $member.to_sequence LIMIT 1;
    LET $pin_page=SELECT key,expires_at FROM canonical_protections WHERE problem=$problem AND sequence >= $member.from_sequence AND sequence < $member.to_sequence AND released=false ORDER BY key LIMIT 64;
    FOR $pin_key IN $pin_page {
        SELECT * FROM ONLY type::record('canonical_protections',$pin_key.key) FOR UPDATE;
        IF $pin_key.expires_at<=time::micros() AND !fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$pin_key.key) { UPDATE ONLY type::record('canonical_protections',$pin_key.key) SET released=true; };
    };
    LET $pins=$pin_page.filter(|$pin|$pin.expires_at>time::micros() OR fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$pin.key));
    IF ($head.sequence >= $member.from_sequence AND $head.sequence < $member.to_sequence) OR array::len($roots) != 0 OR array::len($pins) != 0 OR array::len($captures) != 0 {
        {membership:false, version:false, edges:0dec, blocks:0dec, pending:false}
    } ELSE IF array::len($pin_page)=64 {
        {membership:false,version:false,edges:0dec,blocks:0dec,pending:true}
    } ELSE {
        UPSERT type::record('canonical_reclaimed_ranges', $candidate) SET key = $candidate, problem = $problem, from_sequence = $member.from_sequence, to_sequence = $member.to_sequence;
        LET $version = $member.version;
        LET $excluded_membership = $candidate;
        LET $excluded_stage = NONE;
        LET $deleted = fn::pse_retention_v1::delete_source_version($pse_rpc_expires_at,$version,$excluded_membership,$excluded_stage);
        IF $deleted.pending = false { DELETE ONLY type::record('canonical_memberships', $candidate); };
        {membership:!$deleted.pending, version:$deleted.version, edges:$deleted.edges, blocks:$deleted.blocks, pending:$deleted.pending}
    };
};

    LET $next = IF $result.pending { $after } ELSE { $candidate };
    UPDATE ONLY $guard SET source_reclaim_after=$next,generation=generation+1dec;
    fn::pse_execution_v1::deadline($pse_rpc_expires_at);
    RETURN object::extend($result,{done:false,after:$next});
};

// Settlement closes acquisition authority before deleting any per-grant row.
// Continued renewals register the exact pin and therefore invalidate a stale
// settlement decision. The monotone floor remains after physical cleanup.
DEFINE FUNCTION fn::pse_retention_v1::settle_protection($rpc_expiry:int,$problem:string,$key:string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $parts=string::split($key,':');
    IF array::len($parts)!=4 OR $parts[0]!='pse.protection.v2' OR string::len($parts[1])!=36 OR string::len($parts[2])!=36 OR <int>$parts[3]<=0 { THROW 'PSE_REFUSAL:protection_settlement_immutable_grant_binding'; };
    LET $authority=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$problem) FOR UPDATE;
    IF $authority=NONE { THROW 'PSE_REFUSAL:protection_settlement_authority_unavailable'; };
    LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$key) FOR UPDATE;
    IF $pin!=NONE AND ($pin.problem!=$problem OR $pin.grant_deadline!=<int>$parts[3] OR $pin.grant_nonce!=$parts[2] OR $pin.authority_incarnation!=$parts[1]) { THROW 'PSE_REFUSAL:protection_settlement_requires_original_grant'; };
    IF fn::pse_files_v1::ticket_open($rpc_expiry,$key) { RETURN {complete:false,waiting:true}; };
    IF $authority.incarnation!=$parts[1] {
        // A rotated authority closes old acquisition, but is not evidence that
        // an arbitrary absent grant ever belonged to this problem. Persisted
        // pins, or exact persisted read/header associations, supply that proof.
        IF $pin=NONE {
            LET $artifact_read=SELECT * FROM ONLY type::record('canonical_artifact_reads',$key) FOR UPDATE;
            LET $artifact=IF $artifact_read=NONE { NONE } ELSE { SELECT * FROM ONLY type::record('canonical_artifacts',$artifact_read.artifact) FOR UPDATE };
            LET $result_read=SELECT * FROM ONLY type::record('canonical_result_protections',$key) FOR UPDATE;
            LET $run=IF $result_read=NONE { NONE } ELSE { SELECT * FROM ONLY type::record('canonical_runs',$result_read.run) };
            IF ($artifact=NONE OR $artifact.primary_problem!=$problem) AND ($run=NONE OR $run.problem!=$problem) { THROW 'PSE_REFUSAL:historical_protection_original_receipt_unavailable'; };
            LET $revision_key=IF $artifact!=NONE AND $artifact.primary_problem=$problem { $artifact.revision } ELSE { $run.revision };
            LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$revision_key);
            IF $revision=NONE OR $revision.problem!=$problem { THROW 'PSE_REFUSAL:historical_protection_original_revision_unavailable'; };
            CREATE ONLY type::record('canonical_protections',$key) SET key=$key,problem=$problem,revision=$revision.key,sequence=$revision.sequence,expires_at=0,released=true,grant_deadline=<int>$parts[3],grant_nonce=$parts[2],authority_incarnation=$parts[1];
        } ELSE {
            // Retain the exact terminal grant as its lost-reply receipt. Released
            // pins retain no source/result bytes and cannot reacquire authority.
            UPDATE ONLY type::record('canonical_protections',$key) SET released=true,expires_at=0;
        };
        DELETE ONLY type::record('canonical_result_protections',$key);
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN {complete:true,waiting:false};
    };
    LET $grant=fn::pse_execution_v1::protection_grant($rpc_expiry,$problem,$key,false);
    IF time::micros()<$grant.grant_deadline { RETURN {complete:false,waiting:true}; };
    IF $pin!=NONE AND !$pin.released AND $pin.expires_at>time::micros() { RETURN {complete:false,waiting:true}; };
    UPDATE ONLY type::record('canonical_guards','retention:'+$problem) SET protection_closed_through=math::max([protection_closed_through ?? 0dec,<decimal>$grant.grant_deadline]);
    DELETE ONLY type::record('canonical_result_protections',$key);
    DELETE ONLY type::record('canonical_protections',$key);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {complete:true,waiting:false};
};

DEFINE FUNCTION fn::pse_retention_v1::forget_study($rpc_expiry: int, $study:string) -> bool {
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:'+$study);
    LET $header=SELECT * FROM ONLY type::record('canonical_studies',$study);
    IF $header=NONE OR !$header.terminal { THROW 'PSE_REFUSAL:only_terminal_studies_may_be_retired'; };
    UPSERT type::record('canonical_study_retirements',$study) SET key=$study,study=$study;
    LET $inputs=SELECT * FROM canonical_artifact_obligations WHERE owner_kind='study' AND owner=$study ORDER BY key LIMIT 64;
    FOR $input IN $inputs { fn::pse_artifact_v1::release($rpc_expiry,object::remove($input,['id'])); };
    LET $rpc_result = array::len($inputs)<64;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

// The occurrence key carries its immutable authority and expiry even after header deletion.
DEFINE FUNCTION fn::pse_retention_v1::forget_analysis($rpc_expiry: int, $analysis:string) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $header=SELECT * FROM ONLY type::record('canonical_analyses',$analysis) FOR UPDATE;
    IF $header=NONE {
        LET $parts=string::split($analysis,':');
        IF array::len($parts)!=4 OR $parts[0]!='pse.analysis.v2' OR string::len($parts[1])!=36 OR string::len($parts[2])!=36 { THROW 'PSE_REFUSAL:unknown_analysis_key_has_no_creation_authority'; };
        LET $authorities=SELECT key FROM canonical_guards WITH INDEX guard_incarnation WHERE incarnation=$parts[1] LIMIT 2;
        IF array::len($authorities)!=1 OR !string::starts_with($authorities[0].key,'retention:') { THROW 'PSE_REFUSAL:analysis_creation_authority_unavailable'; };
        LET $intent={key:$analysis,primary_problem:string::slice($authorities[0].key,10),primary_authority:$parts[1],creation_nonce:$parts[2],creation_expires_at:<decimal>$parts[3]};
        RETURN fn::pse_retention_v1::settle_analysis($rpc_expiry,$intent);
    };
    UPDATE ONLY type::record('canonical_analyses',$analysis) SET retiring=true;
    LET $edges=SELECT id FROM canonical_analysis_edges WHERE analysis=$analysis ORDER BY key LIMIT 64;
    IF array::len($edges)!=0 { DELETE $edges.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $nodes=SELECT id FROM canonical_analysis_nodes WHERE analysis=$analysis ORDER BY key LIMIT 64;
    IF array::len($nodes)!=0 { DELETE $nodes.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $inputs=SELECT id,artifact FROM canonical_analysis_inputs WHERE analysis=$analysis ORDER BY key LIMIT 64;
    IF array::len($inputs)!=0 { FOR $input IN $inputs { LET $owners=SELECT * FROM canonical_artifact_obligations WHERE artifact=$input.artifact AND owner_kind='analysis' AND owner=$analysis LIMIT 1; FOR $owner IN $owners { fn::pse_artifact_v1::release($rpc_expiry,object::remove($owner,['id'])); }; }; DELETE $inputs.id; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $roots=SELECT id,problem FROM canonical_roots WHERE owner_kind='analysis' AND owner=$analysis ORDER BY key LIMIT 64;
    IF array::len($roots)!=0 {
        FOR $root IN $roots {
            LET $source=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$root.problem) FOR UPDATE;
            IF $source=NONE { THROW 'PSE_REFUSAL:analysis_cleanup_source_authority_unavailable'; };
        };
        DELETE $roots.id;
        fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false};
    };
    LET $authority=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$header.primary_problem) FOR UPDATE;
    IF $authority=NONE { THROW 'PSE_REFUSAL:analysis_original_settlement_authority_unavailable'; };
    IF $authority.incarnation=$header.primary_authority AND time::micros()<$header.creation_expires_at { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:true}; };
    RETURN fn::pse_retention_v1::settle_analysis($rpc_expiry,$header);
};

DEFINE FUNCTION fn::pse_retention_v1::settle_analysis($rpc_expiry:int,$intent:object) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $header=SELECT * FROM ONLY type::record('canonical_analyses',$intent.key) FOR UPDATE;
    LET $guard=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$intent.primary_problem) FOR UPDATE;
    IF $guard=NONE OR ($guard.incarnation!=$intent.primary_authority AND $header=NONE) { THROW 'PSE_REFUSAL:analysis_settlement_authority_unavailable'; };
    IF $intent.key!='pse.analysis.v2:'+$intent.primary_authority+':'+$intent.creation_nonce+':'+<string><int>$intent.creation_expires_at { THROW 'PSE_REFUSAL:analysis_settlement_key_binding'; };
    IF $guard.incarnation=$intent.primary_authority AND time::micros()<$intent.creation_expires_at { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:true}; };
    IF $header!=NONE AND object::remove($header,['id','active','retiring'])!=object::remove($intent,['id','active','retiring']) { THROW 'PSE_REFUSAL:analysis_settlement_requires_exact_original_occurrence'; };
    IF $header!=NONE AND !$header.retiring { UPDATE ONLY type::record('canonical_analyses',$intent.key) SET retiring=true; fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    LET $edges=SELECT key FROM canonical_analysis_edges WHERE analysis=$intent.key LIMIT 1;
    LET $nodes=SELECT key FROM canonical_analysis_nodes WHERE analysis=$intent.key LIMIT 1;
    LET $inputs=SELECT key FROM canonical_analysis_inputs WHERE analysis=$intent.key LIMIT 1;
    LET $roots=SELECT key FROM canonical_roots WHERE owner_kind='analysis' AND owner=$intent.key LIMIT 1;
    IF array::len($edges)+array::len($nodes)+array::len($inputs)+array::len($roots)!=0 { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {complete:false,waiting:false}; };
    IF $guard.incarnation=$intent.primary_authority {
        UPDATE ONLY type::record('canonical_guards','retention:'+$intent.primary_problem) SET generation=generation+1dec, analysis_creation_closed_through=math::max([analysis_creation_closed_through ?? 0dec,$intent.creation_expires_at]);
        DELETE ONLY type::record('canonical_analyses',$intent.key);
    } ELSE {
        UPDATE ONLY type::record('canonical_analyses',$intent.key) SET active=false,retiring=true;
    };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {complete:true,waiting:false};
};

DEFINE FUNCTION fn::pse_retention_v1::forget_run($rpc_expiry: int, $run:string) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $captures=SELECT key FROM canonical_study_capture_protections WHERE run=$run LIMIT 1;
    IF array::len($captures)!=0 { THROW 'PSE_REFUSAL:run_results_have_protected_readers'; };
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:'+$run);
    fn::pse_execution_v1::register($rpc_expiry,'result-creators:'+$run);
    LET $old=SELECT * FROM ONLY type::record('canonical_result_retirements',$run) FOR UPDATE;
    IF $old!=NONE { LET $rpc_result = $old; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $header=SELECT * FROM ONLY type::record('canonical_runs',$run);
    IF $header=NONE { THROW 'PSE_REFUSAL:retired_run_unavailable'; };
    fn::pse_execution_v1::register($rpc_expiry, 'retention:'+$header.problem);
    LET $live=SELECT key FROM canonical_attempts WHERE run=$run AND phase NOT IN ['terminal','terminal_compacted'] LIMIT 1;
    IF array::len($live)!=0 OR ($header.terminal_attempt=NONE AND (SELECT * FROM ONLY type::record('canonical_history_identities','effect-free-run:'+$run))=NONE) { THROW 'PSE_REFUSAL:run_recovery_must_finish_before_retirement'; };
    LET $reader_page=SELECT key FROM canonical_result_protections WHERE run=$run AND type::record('canonical_protections',key).released=false ORDER BY key LIMIT 64;
    LET $readers=$reader_page.filter(|$reader|type::record('canonical_protections',$reader.key).expires_at>time::micros() OR fn::pse_files_v1::ticket_open($rpc_expiry,$reader.key));
    FOR $reader IN $reader_page {
        LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$reader.key) FOR UPDATE;
        IF $pin!=NONE AND !$pin.released AND $pin.expires_at<=time::micros() AND !fn::pse_files_v1::ticket_open($rpc_expiry,$reader.key) { UPDATE ONLY type::record('canonical_protections',$reader.key) SET released=true; };
    };
    IF array::len($readers)!=0 { THROW 'PSE_REFUSAL:run_results_have_protected_readers'; };
    IF array::len($reader_page)=64 { RETURN {pending:true}; };
    LET $point=SELECT study FROM canonical_study_points WHERE run=$run LIMIT 1;
    LET $parent=SELECT key FROM canonical_studies WHERE run=$run LIMIT 1;
    LET $studies=array::distinct(array::concat($point.study,$parent.key));
    FOR $study IN $studies {
        SELECT * FROM ONLY type::record('canonical_study_retirements',$study) FOR UPDATE;
        IF (SELECT * FROM ONLY type::record('canonical_study_retirements',$study))=NONE { THROW 'PSE_REFUSAL:run_results_are_retained_by_a_study'; };
    };
    LET $staging=SELECT key FROM canonical_artifact_staging_inputs WHERE run=$run LIMIT 1;
    IF array::len($staging)!=0 { THROW 'PSE_REFUSAL:run_results_have_unsettled_artifact_transfer'; };

    LET $project_refs=SELECT key FROM canonical_research_references WITH INDEX project_research_run WHERE run=$run LIMIT 1;
    IF array::len($project_refs)!=0 { THROW 'PSE_REFUSAL:project_research_retains_results'; };
    LET $roots=SELECT key,problem FROM canonical_roots WHERE owner_kind='run' AND owner=$run ORDER BY key LIMIT 66;
    IF array::len($roots)>65 { THROW 'PSE_REFUSAL:run_source_retention_limit'; };
    FOR $root IN $roots { fn::pse_execution_v1::source_creator($rpc_expiry,$root.problem,$root.key); };
    DELETE $roots.map(|$root|type::record('canonical_roots',$root.key));
    LET $rpc_result = CREATE ONLY type::record('canonical_result_retirements',$run) SET key=$run,run=$run,after_generation=0dec,complete=false;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_retention_v1::collect_run($rpc_expiry: int, $run:string) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $captures=SELECT key FROM canonical_study_capture_protections WHERE run=$run LIMIT 1;
    IF array::len($captures)!=0 { THROW 'PSE_REFUSAL:run_results_have_protected_readers'; };
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:'+$run);
    LET $retirement=SELECT * FROM ONLY type::record('canonical_result_retirements',$run);
    IF $retirement=NONE { THROW 'PSE_REFUSAL:explicit_run_retirement_required'; };
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
        LET $batches=SELECT key,placement,inline_key FROM canonical_result_batches WHERE result_set=$set ORDER BY ordinal LIMIT 1;
        IF array::len($batches)!=0 {
            LET $batch=$batches[0].key;
            LET $outputs=SELECT key FROM canonical_result_block_outputs WHERE batch=$batch ORDER BY key LIMIT 64;
            DELETE $outputs.map(|$row|type::record('canonical_result_block_outputs',$row.key));
            LET $remaining_outputs=SELECT key FROM canonical_result_block_outputs WHERE batch=$batch LIMIT 1;
            LET $deleted=array::len($remaining_outputs)=0;
            IF $deleted {
                IF $batches[0].placement='sealed' { fn::pse_files_v1::release_borrower($rpc_expiry,'result:'+$batch); };
                LET $inline=$batches[0].inline_key;
                IF $inline!=NONE { SELECT key FROM ONLY type::record('canonical_inline_payloads',$inline) FOR UPDATE; };
                DELETE ONLY type::record('canonical_result_blocks',$batch);
                DELETE ONLY type::record('canonical_result_batches',$batch);
                IF $inline!=NONE {
                    LET $remaining=SELECT key FROM canonical_result_batches WHERE inline_key=$inline LIMIT 1;
                    IF array::len($remaining)=0 { DELETE ONLY type::record('canonical_inline_payloads',$inline); };
                };
            };
            LET $rpc_result = {complete:false,batches:IF $deleted {1dec} ELSE {0dec},indexes:<decimal>array::len($outputs),sets:0dec};
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
    // Operation identities and full detail belong to explicit history release.
    // Payload collection never deletes a receipt that a delayed caller can replay.
    LET $manifest=SELECT closed_manifest FROM ONLY type::record('canonical_attempts',$attempt.key);
    IF $manifest.closed_manifest!=NONE { DELETE ONLY type::record('canonical_result_manifests',$manifest.closed_manifest); };
    // Inline observations and operation response bytes are scientific payloads,
    // while the effect identity/request and actual terminal class remain receipts.
    UPDATE ONLY type::record('canonical_attempts',$attempt.key) SET phase='terminal_compacted',completion=NONE;
    LET $operations=SELECT key FROM canonical_execution_operations WHERE run=$run AND kind='seal' AND attempt=$attempt.key LIMIT 1;
    FOR $operation IN $operations { UPDATE ONLY type::record('canonical_execution_operations',$operation.key) SET result=type::bytes(''); };
    UPDATE ONLY type::record('canonical_result_retirements',$run) SET after_generation=$attempt.generation;
    LET $rpc_result = {complete:false,batches:0dec,indexes:<decimal>array::len($seeds),sets:0dec};
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};
"#;
