// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Versioned native execution decisions. Scientific admission belongs to Rust.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(
        &FUNCTIONS
            .replace(
                "__INTERPRETATION__",
                &format!("'{}'", pse_schema::catalog::substrate::INTERPRETATION),
            )
            .replace(
                "__INDEX_BYTES__",
                &pse_schema::catalog::substrate::RESULT_INDEX_BYTES.to_string(),
            )
            .replace(
                "__INDEX_RECORDS__",
                &pse_schema::catalog::substrate::RESULT_INDEX_RECORDS.to_string(),
            )
            .replace(
                "__INDEX_REPLAY__",
                &(pse_schema::catalog::substrate::RESULT_INDEX_RECORDS + 1).to_string(),
            )
            .replace(
                "__MESSAGE_BYTES__",
                &pse_schema::catalog::substrate::RESULT_MESSAGE_BYTES.to_string(),
            ),
    );
}

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_execution_v1::lease($rpc_expiry: int, $expires_at: int) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $expires_at <= time::micros() { THROW 'PSE_REFUSAL:execution_attempt_lease_expired_before_commit'; };
    RETURN true;
};

DEFINE FUNCTION fn::pse_execution_v1::touch($rpc_expiry: int, $key: string) -> decimal {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $id = type::record('canonical_guards', $key);
    LET $old = SELECT * FROM ONLY $id;
    IF $old = NONE {
        CREATE ONLY $id SET key = $key, generation = 1dec, incarnation = <string>rand::uuid::v4();
        LET $rpc_result = 1dec;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    } ELSE {
        UPDATE ONLY $id SET generation += 1dec;
        LET $rpc_result = $old.generation + 1dec;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
};

// Register exact keys, including absence. Registration does not write or block.
DEFINE FUNCTION fn::pse_execution_v1::register($rpc_expiry:int,$key:string) -> bool {
    SELECT * FROM ONLY type::record('canonical_guards',$key) FOR UPDATE;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN true;
};

DEFINE FUNCTION fn::pse_execution_v1::source_creator($rpc_expiry:int,$problem:string,$root:string) -> bool {
    fn::pse_execution_v1::register($rpc_expiry,'retention:'+$problem);
    SELECT * FROM ONLY type::record('canonical_roots',$root) FOR UPDATE;
    fn::pse_execution_v1::touch($rpc_expiry,'source-creators:'+$problem);
    RETURN true;
};

DEFINE FUNCTION fn::pse_execution_v1::result_creator($rpc_expiry:int,$run:string) -> bool {
    fn::pse_history_v1::run($rpc_expiry,$run);
    SELECT * FROM ONLY type::record('canonical_result_retirements',$run) FOR UPDATE;
    IF (SELECT * FROM ONLY type::record('canonical_result_retirements',$run))!=NONE { THROW 'PSE_REFUSAL:execution_results_explicitly_retired'; };
    fn::pse_execution_v1::touch($rpc_expiry,'result-creators:'+$run);
    RETURN true;
};

// The key itself carries the original acquisition window, including before
// a delayed acquisition has created its record. Release never invents a window.
DEFINE FUNCTION fn::pse_execution_v1::protection_grant($rpc_expiry:int,$problem:string,$key:string,$acquiring:bool) -> object {
    LET $parts=string::split($key,':');
    IF array::len($parts)!=4 OR $parts[0]!='pse.protection.v2' OR string::len($parts[2])!=36 { THROW 'PSE_REFUSAL:protection_immutable_grant_binding'; };
    LET $authority=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$problem) FOR UPDATE;
    LET $deadline=<int>$parts[3];
    IF $authority=NONE OR $authority.incarnation!=$parts[1] OR $deadline<=0 { THROW 'PSE_REFUSAL:protection_authority_incarnation_differs'; };
    IF $acquiring AND ($deadline<=time::micros() OR $deadline<=($authority.protection_closed_through ?? 0dec)) { THROW 'PSE_REFUSAL:protection_acquisition_window_closed'; };
    SELECT * FROM ONLY type::record('canonical_protections',$key) FOR UPDATE;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {grant_deadline:$deadline,grant_nonce:$parts[2],authority_incarnation:$parts[1],closed:$deadline<=($authority.protection_closed_through ?? 0dec)};
};

DEFINE FUNCTION fn::pse_execution_v1::operation($rpc_expiry: int, $key: string, $kind: string, $request: bytes) -> option<object> {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-operation:' + $key);
    LET $old = fn::pse_history_v1::operation($rpc_expiry,$key,NONE);
    IF $old != NONE AND ($old.kind != $kind OR $old.request != $request) { THROW 'PSE_REFUSAL:execution_operation_identity_reused'; };
    LET $rpc_result = $old;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::available($rpc_expiry: int, $run: string) -> bool {
    fn::pse_history_v1::run($rpc_expiry,$run);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry, 'run-cancel:' + $run);
    IF (SELECT * FROM ONLY type::record('canonical_result_retirements',$run) FOR UPDATE) != NONE { THROW 'PSE_REFUSAL:execution_results_explicitly_retired'; };
    LET $rpc_result = true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::study_cancelled($rpc_expiry: int, $run: string) -> bool {
    fn::pse_history_v1::run($rpc_expiry,$run);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry,'study-association:'+$run);
    LET $points = SELECT study FROM canonical_history_locators WHERE run=$run AND point!=NONE LIMIT 1;
    IF array::len($points) = 0 { LET $rpc_result = false; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $points[0].study);
    LET $study = SELECT * FROM ONLY type::record('canonical_studies', $points[0].study);
    IF $study = NONE { THROW 'PSE_REFUSAL:execution_study_cancellation_authority_absent'; };
    LET $rpc_result = $study.cancelled;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::summary_cancelled($rpc_expiry: int, $run: string) -> bool {
    fn::pse_history_v1::run($rpc_expiry,$run);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry,'study-association:'+$run);
    LET $parents = SELECT study AS key FROM canonical_history_locators WHERE run=$run AND point=NONE LIMIT 1;
    IF array::len($parents) = 0 { LET $rpc_result = false; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $parents[0].key);
    LET $study = SELECT * FROM ONLY type::record('canonical_studies',$parents[0].key);
    IF $study = NONE { THROW 'PSE_REFUSAL:execution_summary_cancellation_authority_absent'; };
    LET $rpc_result = $study.cancelled;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::writer($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $operation: string, $kind: string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $r = SELECT * FROM ONLY type::record('canonical_runs', $run);
    LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $attempt);
    IF $r = NONE OR $a = NONE OR $a.run != $run OR $r.current_attempt != $attempt OR $r.current_generation != $generation OR $a.generation != $generation OR $r.cancelled OR $r.terminal_attempt != NONE OR ($a.phase IN ['terminal','terminal_compacted']) OR $a.expires_at <= time::micros() { THROW 'PSE_REFUSAL:execution_attempt_fence_revoked'; };
    LET $summary_cancelled=fn::pse_execution_v1::summary_cancelled($rpc_expiry,$run);
    LET $finalizing=IF $a.finalization_study!=NONE {
        LET $summary=SELECT * FROM ONLY type::record('canonical_studies',$a.finalization_study) FOR UPDATE;
        IF $summary=NONE OR $summary.run!=$run OR !$summary.active OR $summary.next_ordinal!=$summary.point_count OR $summary.terminal { THROW 'PSE_REFUSAL:study_summary_finalization_association_changed'; };
        IF $kind NOT IN ['append','empty','close','renew'] { THROW 'PSE_REFUSAL:study_summary_capability_only_permits_publication'; };
        true
    } ELSE { false };
    IF fn::pse_execution_v1::study_cancelled($rpc_expiry, $run) OR ($summary_cancelled AND !$finalizing) {
        fn::pse_execution_v1::deadline($rpc_expiry);
        fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at);
        RETURN {pse_writer:'study-cancelled',run:$run,attempt:$attempt,generation:$generation,operation:$operation};
    };
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at);
    RETURN {pse_writer:'admitted',attempt:$a};
};

DEFINE FUNCTION fn::pse_execution_v1::fence($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal) -> object {
    LET $writer = fn::pse_execution_v1::writer($rpc_expiry, $run, $attempt, $generation, 'dispatch', 'dispatch');
    IF $writer.pse_writer != 'admitted' { THROW 'PSE_REFUSAL:execution_study_cancellation_fence_revoked'; };
    RETURN $writer.attempt;
};

DEFINE FUNCTION fn::pse_execution_v1::begin_run($rpc_expiry: int, $row: object, $sources: array<object>, $admission: option<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $row.interpretation!=__INTERPRETATION__ OR (SELECT * FROM ONLY canonical_interpretations:current).interpretation!=__INTERPRETATION__ { THROW 'PSE_REFUSAL:stale_execution_publication_interpretation'; };
    fn::pse_execution_v1::available($rpc_expiry, $row.key);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $row.key);
    LET $old = SELECT * FROM ONLY type::record('canonical_runs', $row.key);
    IF $old != NONE {
        IF $old.request != $row.request OR $old.problem != $row.problem OR $old.revision != $row.revision OR $old.source_sequence != $row.source_sequence OR $old.source_selection != $row.source_selection OR $old.attestation != $row.attestation OR $old.interpretation != $row.interpretation { THROW 'PSE_REFUSAL:execution_run_identity_reused'; };
        LET $rpc_result = $old;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    LET $admission_owner=IF $admission=NONE {NONE} ELSE {fn::pse_study_v1::admission_authority($rpc_expiry,$admission)};
    IF $admission!=NONE AND $row.key!=(SELECT run FROM ONLY type::record('canonical_study_points',$admission.point)).run { THROW 'PSE_REFUSAL:selected_scientific_run_identity_differs_from_its_occurrence'; };
    fn::pse_execution_v1::source_creator($rpc_expiry,$row.problem,'run:'+$row.key);
    LET $revision = SELECT * FROM ONLY type::record('canonical_revisions', $row.revision);
    IF $revision = NONE OR $revision.problem != $row.problem OR $revision.sequence != $row.source_sequence OR $revision.interpretation != $row.interpretation { THROW 'PSE_REFUSAL:execution_revision_unavailable'; };
    LET $pruned = SELECT key FROM canonical_reclaimed_ranges WHERE problem = $row.problem AND from_sequence <= $revision.sequence AND to_sequence > $revision.sequence LIMIT 1;
    IF array::len($pruned) != 0 { THROW 'PSE_REFUSAL:execution_revision_source_reclaimed'; };
    IF array::len($sources)>64 { THROW 'PSE_REFUSAL:execution_sources_limit_exceeded'; };
    FOR $source IN $sources {
        fn::pse_execution_v1::source_creator($rpc_expiry,$source.problem,$source.key);
        LET $selected = SELECT * FROM ONLY type::record('canonical_revisions',$source.revision);
        IF $selected=NONE OR $selected.problem!=$source.problem OR $selected.sequence!=$source.sequence OR $selected.interpretation!=$source.interpretation { THROW 'PSE_REFUSAL:execution_selected_source_unavailable'; };
        LET $reclaimed=SELECT key FROM canonical_reclaimed_ranges WHERE problem=$source.problem AND from_sequence<=$source.sequence AND to_sequence>$source.sequence LIMIT 1;
        IF array::len($reclaimed)!=0 { THROW 'PSE_REFUSAL:execution_selected_source_reclaimed'; };
    };
    LET $sequence = fn::pse_execution_v1::touch($rpc_expiry, 'execution-sequence:' + $row.problem);
    LET $saved = CREATE ONLY type::record('canonical_runs', $row.key) CONTENT object::extend($row, {sequence:$sequence});
    CREATE ONLY type::record('canonical_roots', 'run:' + $row.key) SET key = 'run:' + $row.key, problem = $row.problem, revision = $revision.key, sequence = $revision.sequence, owner_kind = 'run', owner = $row.key;
    INSERT RELATION INTO canonical_problem_runs {id:type::record('canonical_problem_runs',$row.key),in:type::record('canonical_problems',$row.problem),out:type::record('canonical_runs',$row.key),key:$row.key,problem:$row.problem,run:$row.key} RETURN NONE;
    INSERT RELATION INTO canonical_run_sources {id:type::record('canonical_run_sources',$row.key),in:type::record('canonical_runs',$row.key),out:type::record('canonical_revisions',$row.revision),key:$row.key,run:$row.key,revision:$row.revision} RETURN NONE;
    FOR $source IN $sources {
        CREATE ONLY type::record('canonical_roots',$source.key) SET key=$source.key,problem=$source.problem,revision=$source.revision,sequence=$source.sequence,owner_kind='run',owner=$row.key;
        INSERT RELATION INTO canonical_run_sources {id:type::record('canonical_run_sources',$source.key),in:type::record('canonical_runs',$row.key),out:type::record('canonical_revisions',$source.revision),key:$source.key,run:$row.key,revision:$source.revision} RETURN NONE;
    };
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $admission_owner!=NONE { fn::pse_execution_v1::lease($rpc_expiry,$admission_owner.expires_at); };
    RETURN $rpc_result;
};

// Installed writer meaning is server authority, independent of a stale client's argument.
// This fence is separate from worker freshness: current immutable replays still work
// after cancellation, closure or a later attempt.
DEFINE FUNCTION fn::pse_execution_v1::current_writer($rpc_expiry:int,$run:string) -> bool {
    fn::pse_history_v1::run($rpc_expiry,$run);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $marker=SELECT * FROM ONLY canonical_interpretations:current;
    LET $row=SELECT * FROM ONLY type::record('canonical_runs',$run);
    IF $marker=NONE OR $marker.interpretation!=__INTERPRETATION__ OR $row=NONE OR $row.interpretation!=__INTERPRETATION__ { THROW 'PSE_REFUSAL:historical_execution_is_read_only'; };
    RETURN true;
};

DEFINE FUNCTION fn::pse_execution_v1::claim($rpc_expiry: int, $run: string, $operation: string, $request: bytes, $attempt: string, $worker: string, $lifetime: int, $interpretation: string, $finalization_study: option<string>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    IF $interpretation!=__INTERPRETATION__ { THROW 'PSE_REFUSAL:stale_execution_claim_interpretation'; };
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'claim', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_attempts', $op.attempt); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $r = SELECT * FROM ONLY type::record('canonical_runs', $run);
    LET $summary_cancelled=fn::pse_execution_v1::summary_cancelled($rpc_expiry,$run);
    IF $finalization_study!=NONE {
        LET $summary=SELECT * FROM ONLY type::record('canonical_studies',$finalization_study) FOR UPDATE;
        fn::pse_execution_v1::register($rpc_expiry,'study-cancel:'+$finalization_study);
        IF $summary=NONE OR $summary.run!=$run OR !$summary.active OR $summary.terminal OR $summary.point_count=0dec OR $summary.next_ordinal!=$summary.point_count { THROW 'PSE_REFUSAL:study_summary_finalization_association_unavailable'; };
        LET $pending=SELECT key FROM canonical_study_points WHERE study=$finalization_study AND settled=false LIMIT 1;
        IF array::len($pending)!=0 { THROW 'PSE_REFUSAL:study_summary_finalization_requires_settled_membership'; };
    };
    IF fn::pse_execution_v1::study_cancelled($rpc_expiry, $run) OR ($summary_cancelled AND $finalization_study=NONE) { THROW 'PSE_REFUSAL:execution_cancelled_study_run_not_claimable'; };
    IF $r = NONE OR $r.cancelled OR $r.interpretation != $interpretation { THROW 'PSE_REFUSAL:execution_run_not_claimable'; };
    IF (SELECT * FROM ONLY type::record('canonical_history_identities','effect-free-run:'+$run) FOR UPDATE)!=NONE { THROW 'PSE_REFUSAL:effect_free_execution_already_settled'; };
    IF $r.current_attempt != NONE {
        LET $previous = SELECT * FROM ONLY type::record('canonical_attempts', $r.current_attempt);
        IF $previous != NONE AND (($previous.phase NOT IN ['terminal','terminal_compacted']) OR $previous.outcome = 'succeeded') { THROW 'PSE_REFUSAL:execution_attempt_is_live_or_already_succeeded'; };
    };
    LET $generation = $r.current_generation + 1dec;
    LET $saved = CREATE ONLY type::record('canonical_attempts', $attempt) SET key = $attempt, run = $run, generation = $generation, claim_operation = $operation, request = $request, worker = $worker, expires_at = time::micros() + $lifetime, phase = 'ingesting', finalization_study=$finalization_study;
    UPDATE ONLY type::record('canonical_runs', $run) SET current_generation = $generation, current_attempt = $attempt, terminal_attempt = NONE,terminal_class=NONE;
    fn::pse_execution_v1::source_creator($rpc_expiry,$r.problem,'active_attempt:'+$attempt);
    LET $source_root = SELECT * FROM ONLY type::record('canonical_roots', 'run:' + $run);
    IF $source_root = NONE OR $source_root.problem != $r.problem OR $source_root.revision != $r.revision OR $source_root.sequence != $r.source_sequence OR $source_root.owner_kind != 'run' OR $source_root.owner != $run { THROW 'PSE_REFUSAL:execution_retained_source_selection_unavailable'; };
    CREATE ONLY type::record('canonical_roots', 'active_attempt:' + $attempt) SET key = 'active_attempt:' + $attempt, problem = $r.problem, revision = $r.revision, sequence = $r.source_sequence, owner_kind = 'active_attempt', owner = $attempt;
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$attempt,'claim',$request,$request,NONE,NONE);
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $saved.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::append($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes, $set: object, $batch: object, $block: option<object>, $row_indexes: array<object>, $outputs: array<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    IF $set.interpretation!=__INTERPRETATION__ OR ($block!=NONE AND $block.interpretation!=__INTERPRETATION__) OR array::len($row_indexes[WHERE interpretation!=__INTERPRETATION__])!=0 OR array::len($outputs[WHERE interpretation!=__INTERPRETATION__])!=0 { THROW 'PSE_REFUSAL:stale_execution_append_interpretation'; };
    IF array::len($row_indexes) > __INDEX_RECORDS__ OR array::len($outputs) > __INDEX_RECORDS__ OR bytes::len(encoding::cbor::encode({block:$block,row_indexes:$row_indexes,outputs:$outputs})) > __INDEX_BYTES__ { THROW 'PSE_REFUSAL:execution_index_metadata_extent_exceeded'; };
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'append', $request);
    LET $writer = IF $op = NONE { fn::pse_execution_v1::writer($rpc_expiry, $run, $attempt, $generation, $operation, 'append') } ELSE { NONE };
    IF $writer != NONE AND $writer.pse_writer != 'admitted' { RETURN $writer; };
    LET $a = $writer.attempt;
    IF $op = NONE AND (($a.phase!='ingesting')) { THROW 'PSE_REFUSAL:execution_result_ingestion_closed'; };
    IF $batch.attempt != $attempt OR $set.attempt != $attempt OR $batch.result_set != $set.key { THROW 'PSE_REFUSAL:execution_batch_coordinate_mismatch'; };
    LET $existing = SELECT * FROM ONLY type::record('canonical_result_batches', $batch.key);
    LET $existing_block = SELECT * FROM ONLY type::record('canonical_result_blocks', $batch.key);
    IF $existing != NONE {
        IF object::remove($existing,'id')!=$batch OR $existing.digest != $batch.digest OR $existing.ordinal != $batch.ordinal OR $existing.row_count != $batch.row_count OR $existing.result_set != $batch.result_set OR $existing.attempt != $attempt { THROW 'PSE_REFUSAL:execution_immutable_batch_changed'; };
        IF ($block = NONE AND $existing_block != NONE) OR ($block != NONE AND ($existing_block = NONE OR object::remove($existing_block, 'id') != $block)) { THROW 'PSE_REFUSAL:execution_immutable_block_changed'; };
        LET $old_row_indexes = SELECT * FROM canonical_result_rows WHERE batch = $batch.key ORDER BY key LIMIT __INDEX_REPLAY__;
        IF array::len($old_row_indexes) != array::len($row_indexes) OR $old_row_indexes.map(|$row_index| object::remove($row_index, 'id')) != $row_indexes { THROW 'PSE_REFUSAL:execution_immutable_scalar_row_locators_changed'; };
        LET $old_outputs = SELECT * FROM canonical_result_block_outputs WHERE batch = $batch.key ORDER BY key LIMIT __INDEX_REPLAY__;
        IF array::len($old_outputs) != array::len($outputs) OR $old_outputs.map(|$output| object::remove($output, 'id')) != $outputs { THROW 'PSE_REFUSAL:execution_immutable_output_block_indexes_changed'; };
        IF $op != NONE { LET $rpc_result = {key:$batch.key,attempt:$attempt,result_set:$set.key,ordinal:$batch.ordinal,digest:$batch.digest,row_count:$batch.row_count,file_binding:$batch.file_binding,request:$request}; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    } ELSE {
        IF $op != NONE { THROW 'PSE_REFUSAL:execution_acknowledged_batch_unavailable'; };
        LET $s = SELECT * FROM ONLY type::record('canonical_result_sets', $set.key);
        IF $s = NONE {
            LET $sets = SELECT key FROM canonical_result_sets WHERE attempt = $attempt LIMIT 256;
            IF array::len($sets) >= 256 { THROW 'PSE_REFUSAL:execution_result_set_bound_exceeded'; };
            IF $batch.ordinal != 0dec { THROW 'PSE_REFUSAL:execution_batch_sequence_gap'; };
            CREATE ONLY type::record('canonical_result_sets', $set.key) CONTENT $set;
        } ELSE {
            IF $s.attempt != $attempt OR $s.name != $set.name OR $s.interpretation != $set.interpretation OR $s.next_ordinal != $batch.ordinal { THROW 'PSE_REFUSAL:execution_batch_sequence_gap'; };
        };
        fn::pse_files_v1::attach($rpc_expiry,$batch,'result',$attempt,'result:'+$batch.key);
        CREATE ONLY type::record('canonical_result_batches', $batch.key) CONTENT $batch;
        IF $block != NONE { CREATE ONLY type::record('canonical_result_blocks', $batch.key) CONTENT $block; };
        IF array::len($row_indexes) != 0 {
            LET $rows = $row_indexes.map(|$row_index| object::extend($row_index, {id:type::record('canonical_result_rows', $row_index.key)}));
            INSERT INTO canonical_result_rows $rows RETURN NONE;
        };
        IF array::len($outputs) != 0 {
            LET $rows = $outputs.map(|$output| object::extend($output, {id:type::record('canonical_result_block_outputs', $output.key)}));
            INSERT INTO canonical_result_block_outputs $rows RETURN NONE;
        };
        UPDATE ONLY type::record('canonical_result_sets', $set.key) SET next_ordinal += 1dec, row_count += $batch.row_count;
    };
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$attempt,'append',$request,$request,NONE,NONE);
    LET $rpc_result = {key:$batch.key,attempt:$attempt,result_set:$set.key,ordinal:$batch.ordinal,digest:$batch.digest,row_count:$batch.row_count,file_binding:$batch.file_binding,request:$request};
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $op = NONE { fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at); };
    RETURN $rpc_result;
};

// One statement/transaction, preserving every existing per-block operation receipt.
DEFINE FUNCTION fn::pse_execution_v1::append_many($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $items: array<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    IF array::len($items)=0 OR array::len($items)>__INDEX_RECORDS__ OR bytes::len(encoding::cbor::encode($items))>__MESSAGE_BYTES__ { THROW 'PSE_REFUSAL:execution_append_group_extent_exceeded'; };
    fn::pse_execution_v1::available($rpc_expiry,$run);
    // Preflight BEFORE any append mutation. An entirely replayed group needs no
    // fresh writer admission after cancellation, closure or attempt replacement.
    FOR $item IN $items {
        LET $old=fn::pse_execution_v1::operation($rpc_expiry,$item.operation,'append',$item.request);
        IF $old=NONE {
            LET $writer=fn::pse_execution_v1::writer($rpc_expiry,$run,$attempt,$generation,$item.operation,'append');
            IF $writer.pse_writer!='admitted' { RETURN $writer; };
            IF ($writer.attempt.phase!='ingesting') { THROW 'PSE_REFUSAL:execution_result_ingestion_closed'; };
        };
    };
    // FOR executes each original append serially. A later gap, conflicting
    // immutable receipt or deadline THROW rolls the entire new group back.
    FOR $item IN $items {
        LET $saved=fn::pse_execution_v1::append($rpc_expiry,$run,$attempt,$generation,$item.operation,$item.request,$item.set,$item.batch,$item.block,$item.row_indexes,$item.outputs);
        IF $saved.pse_writer!=NONE { THROW 'PSE_REFUSAL:execution_append_group_admission_changed'; };
    };
    LET $receipts=$items.map(|$item| {key:$item.batch.key,attempt:$attempt,result_set:$item.set.key,ordinal:$item.batch.ordinal,digest:$item.batch.digest,row_count:$item.batch.row_count,file_binding:$item.batch.file_binding,request:$item.request});
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {acknowledgments:$receipts};
};

DEFINE FUNCTION fn::pse_execution_v1::empty($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes, $set: object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    fn::pse_execution_v1::available($rpc_expiry,$run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry,$operation,'empty',$request);
    LET $writer=IF $op=NONE { fn::pse_execution_v1::writer($rpc_expiry,$run,$attempt,$generation,$operation,'empty') } ELSE { NONE };
    IF $writer!=NONE AND $writer.pse_writer != 'admitted' { RETURN $writer; };
    IF $op=NONE AND (($writer.attempt.phase!='ingesting')) { THROW 'PSE_REFUSAL:execution_empty_declaration_ingestion_closed'; };
    IF $set.attempt != $attempt OR $set.interpretation != __INTERPRETATION__ OR $set.next_ordinal != 0dec OR $set.row_count != 0dec { THROW 'PSE_REFUSAL:execution_empty_declaration_mismatch'; };
    LET $old = SELECT * FROM ONLY type::record('canonical_result_sets',$set.key);
    IF $old != NONE {
        IF object::remove($old,'id') != $set { THROW 'PSE_REFUSAL:execution_demanded_empty_set_changed'; };
    } ELSE {
        IF $op != NONE { THROW 'PSE_REFUSAL:execution_acknowledged_empty_set_unavailable'; };
        LET $sets = SELECT key FROM canonical_result_sets WHERE attempt=$attempt LIMIT 256;
        IF array::len($sets) >= 256 { THROW 'PSE_REFUSAL:execution_result_set_bound_exceeded'; };
        CREATE ONLY type::record('canonical_result_sets',$set.key) CONTENT $set;
    };
    IF $op = NONE { fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$attempt,'empty',$request,$request,NONE,NONE); };
    LET $saved = SELECT * FROM ONLY type::record('canonical_result_sets',$set.key);
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $op = NONE { fn::pse_execution_v1::lease($rpc_expiry,$writer.attempt.expires_at); };
    RETURN $saved;
};

DEFINE FUNCTION fn::pse_execution_v1::verify_acknowledged($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $proofs: array<object>, $seeds: object, $progress_queued: option<decimal>) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    LET $sets=SELECT * FROM canonical_result_sets WHERE attempt=$attempt ORDER BY key LIMIT 257;
    IF array::len($sets)>256 OR array::len($proofs)>256 OR array::len(array::distinct($proofs.map(|$p| $p.set))) != array::len($proofs) { THROW 'PSE_REFUSAL:execution_acknowledged_set_membership_differs'; };
    FOR $p IN $proofs {
        IF array::len($sets.filter(|$s| $s.key=$p.set))!=1 { THROW 'PSE_REFUSAL:execution_advertised_set_missing'; };
    };
    FOR $s IN $sets {
        LET $matching=$proofs.filter(|$p| $p.set=$s.key);
        IF array::len($matching)=0 {
            IF $progress_queued=NONE OR $s.name!='__progress' OR $s.attempt!=$attempt OR $s.interpretation!=__INTERPRETATION__ OR $s.row_count>$progress_queued { THROW 'PSE_REFUSAL:execution_unacknowledged_set_membership_differs'; };
        } ELSE {
        IF array::len($matching)!=1 { THROW 'PSE_REFUSAL:execution_advertised_set_ambiguous'; };
        LET $p=$matching[0];
        LET $uncertain=$s.name='__progress' AND $progress_queued!=NONE;
        IF $s.attempt != $attempt OR $s.name != $p.name OR $s.interpretation != __INTERPRETATION__ OR (!$uncertain AND ($s.next_ordinal != $p.count OR $s.row_count != $p.row_count)) OR ($uncertain AND ($s.next_ordinal<$p.count OR $s.row_count<$p.row_count OR $s.row_count>$progress_queued)) { THROW 'PSE_REFUSAL:execution_acknowledged_set_coverage_differs'; };
        LET $receipt=SELECT * FROM ONLY type::record('canonical_execution_operations',$p.operation);
        IF $receipt=NONE OR $receipt.run != $run OR $receipt.attempt != $attempt OR $receipt.kind != $p.kind OR $receipt.request != $p.request OR $receipt.result != $p.request { THROW 'PSE_REFUSAL:execution_actual_final_receipt_differs'; };
        IF $p.count=0dec {
            IF $p.kind != 'empty' OR $p.row_count != 0dec { THROW 'PSE_REFUSAL:execution_empty_membership_lacks_declaration'; };
        } ELSE {
            LET $last=SELECT key,attempt,result_set,ordinal,digest,row_count,file_binding FROM ONLY type::record('canonical_result_batches',$p.key);
            IF $p.kind != 'append' OR $last=NONE OR $last.attempt != $attempt OR $last.result_set != $s.key OR $last.ordinal+1dec != $p.count OR $last.ordinal != $p.ordinal OR $last.digest != $p.digest OR $last.row_count != $p.rows OR $last.file_binding!=$p.file_binding { THROW 'PSE_REFUSAL:execution_final_batch_receipt_differs'; };
        };
        };
    };
    LET $completion=$sets.filter(|$s| $s.name='__completion');
    IF array::len($completion)!=1 OR $completion[0].next_ordinal=0dec { THROW 'PSE_REFUSAL:execution_completion_coverage_missing'; };
    LET $inventory=SELECT count() AS n,math::sum(batch_count) AS batches,math::sum(payload_bytes) AS bytes FROM canonical_result_seeds WHERE attempt=$attempt GROUP ALL;
    LET $n=IF array::len($inventory)=0 { 0dec } ELSE { $inventory[0].n };
    LET $seed_sets=$sets.filter(|$s| $s.name='__seeds');
    IF $n != $seeds.count { THROW 'PSE_REFUSAL:execution_seed_inventory_differs'; };
    IF $n=0dec {
        IF array::len($seed_sets)!=0 OR $seeds.batches != 0dec OR $seeds.bytes != 0dec { THROW 'PSE_REFUSAL:execution_orphaned_seed_chunks'; };
    } ELSE {
        IF array::len($seed_sets)!=1 OR $seed_sets[0].next_ordinal != $seeds.batches OR $seed_sets[0].row_count != $seeds.bytes OR $inventory[0].batches != $seeds.batches OR $inventory[0].bytes != $seeds.bytes { THROW 'PSE_REFUSAL:execution_seed_coverage_differs'; };
        LET $last=SELECT * FROM ONLY type::record('canonical_result_seeds',$seeds.key);
        LET $receipt=SELECT * FROM ONLY type::record('canonical_execution_operations',$seeds.operation);
        IF $last=NONE OR $last.run != $run OR $last.attempt != $attempt OR $last.attempt_generation != $generation OR $last.result_set != $seed_sets[0].key OR $last.first_ordinal+$last.batch_count != $seeds.batches OR $receipt=NONE OR $receipt.run != $run OR $receipt.attempt != $attempt OR $receipt.kind != 'seed' OR $receipt.request != $seeds.request OR $receipt.result != $seeds.request { THROW 'PSE_REFUSAL:execution_actual_seed_receipt_differs'; };
    };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN true;
};
DEFINE FUNCTION fn::pse_execution_v1::check_closed_acknowledgments($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $proofs: array<object>, $seeds: object, $progress_queued: decimal) -> object {
    LET $writer=fn::pse_execution_v1::writer($rpc_expiry,$run,$attempt,$generation,'ack-readback','close');
    IF $writer.pse_writer!='admitted' { RETURN $writer; };
    IF ($writer.attempt.phase='ingesting') OR $writer.attempt.close_generation!=$generation { THROW 'PSE_REFUSAL:execution_acknowledgment_readback_fenced'; };
    fn::pse_execution_v1::verify_acknowledged($rpc_expiry,$run,$attempt,$generation,$proofs,$seeds,$progress_queued);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    fn::pse_execution_v1::lease($rpc_expiry,$writer.attempt.expires_at);
    RETURN {valid:true};
};

DEFINE FUNCTION fn::pse_execution_v1::close_acknowledged($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes, $manifest: object, $proofs: array<object>, $seeds: object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    fn::pse_execution_v1::available($rpc_expiry,$run);
    LET $op=fn::pse_execution_v1::operation($rpc_expiry,$operation,'close',$request);
    IF $op != NONE {
        LET $m=SELECT * FROM ONLY type::record('canonical_result_manifests',$manifest.key);
        IF $m=NONE OR object::remove($m,'id') != $manifest { THROW 'PSE_REFUSAL:execution_acknowledged_close_descriptor_changed'; };
        LET $saved=SELECT * FROM ONLY type::record('canonical_attempts',$attempt);
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $saved;
    };
    LET $writer=fn::pse_execution_v1::writer($rpc_expiry,$run,$attempt,$generation,$operation,'close');
    IF $writer.pse_writer != 'admitted' { RETURN $writer; };
    LET $a=$writer.attempt;
    IF ($a.phase!='ingesting') OR $manifest.key != $attempt OR $manifest.attempt != $attempt OR $manifest.generation != $generation OR bytes::len($manifest.descriptors) > __INDEX_BYTES__ { THROW 'PSE_REFUSAL:execution_acknowledged_close_fenced'; };
    fn::pse_execution_v1::verify_acknowledged($rpc_expiry,$run,$attempt,$generation,$proofs,$seeds,NONE);
    fn::pse_history_v1::identity($rpc_expiry,'manifest',$manifest.key,$run); CREATE ONLY type::record('canonical_result_manifests',$manifest.key) CONTENT $manifest;
    LET $saved=UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='closed_with_manifest',close_generation=$generation,closed_manifest=$manifest.key;
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$attempt,'close',$request,type::bytes(<string>$generation),NONE,NONE);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry,$a.expires_at);
    RETURN $saved;
};

DEFINE FUNCTION fn::pse_execution_v1::seal_ack($rpc_expiry: int, $run: string, $attempt: string, $authority: decimal, $operation: string, $request: bytes, $manifest: string, $digest: string, $outcome: string, $completion: bytes, $seed_eligible: bool, $coverage: option<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    IF $coverage != NONE {
        LET $sets=SELECT * FROM canonical_result_sets WHERE attempt=$attempt AND name='__completion' LIMIT 2;
        LET $progress=SELECT * FROM canonical_result_sets WHERE attempt=$attempt AND name='__progress' LIMIT 2;
        LET $rows=IF array::len($progress)=0 { 0dec } ELSE { $progress[0].row_count };
        IF array::len($sets)!=1 OR $sets[0].next_ordinal != $coverage.count OR $sets[0].row_count != $coverage.bytes OR $rows != $coverage.progress { THROW 'PSE_REFUSAL:execution_original_completion_coverage_differs'; };
    };
    LET $a=fn::pse_execution_v1::seal($rpc_expiry,$run,$attempt,$authority,$operation,$request,$manifest,$digest,$outcome,$completion,$seed_eligible);
    IF $a.pse_writer != NONE { RETURN $a; };
    LET $r=SELECT * FROM ONLY type::record('canonical_runs',$run);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {attempt:$a,run:$r};
};
DEFINE FUNCTION fn::pse_execution_v1::study_seal_ack($rpc_expiry: int, $study: string, $run: string, $attempt: string, $authority: decimal, $operation: string, $request: bytes, $manifest: string, $digest: string, $outcome: string, $completion: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    LET $a=fn::pse_study_v1::seal($rpc_expiry,$study,$run,$attempt,$authority,$operation,$request,$manifest,$digest,$outcome,$completion);
    IF $a.pse_writer != NONE { RETURN $a; };
    LET $r=SELECT * FROM ONLY type::record('canonical_runs',$run);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {attempt:$a,run:$r};
};

DEFINE FUNCTION fn::pse_execution_v1::close($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'close', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_attempts', $op.attempt); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $writer = fn::pse_execution_v1::writer($rpc_expiry, $run, $attempt, $generation, $operation, 'close');
    IF $writer.pse_writer != 'admitted' { RETURN $writer; };
    LET $a = $writer.attempt;
    IF ($a.phase!='ingesting') { THROW 'PSE_REFUSAL:execution_ingestion_already_closed'; };
    LET $saved = UPDATE ONLY type::record('canonical_attempts', $attempt) SET phase = IF closed_manifest=NONE { 'closed_awaiting_manifest' } ELSE { 'closed_with_manifest' }, close_generation = $generation;
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$attempt,'close',$request,type::bytes(<string>$generation),NONE,NONE);
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::cancel($rpc_expiry: int, $run: string, $operation: string, $request: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    fn::pse_execution_v1::touch($rpc_expiry,'run-cancel:'+$run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'cancel', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_runs', $run); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $r = SELECT * FROM ONLY type::record('canonical_runs', $run);
    IF $r = NONE { THROW 'PSE_REFUSAL:execution_run_unavailable'; };
    IF $r.terminal_attempt != NONE { THROW 'PSE_REFUSAL:execution_run_already_sealed'; };
    LET $generation = $r.current_generation + 1dec;
    LET $saved = UPDATE ONLY type::record('canonical_runs', $run) SET cancelled = true, current_generation = $generation;
    IF $r.current_attempt != NONE { UPDATE ONLY type::record('canonical_attempts', $r.current_attempt) SET phase = IF closed_manifest=NONE { 'closed_awaiting_manifest' } ELSE { 'closed_with_manifest' }, close_generation = $generation; };
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,NONE,'cancel',$request,$request,NONE,NONE);
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::recover($rpc_expiry: int, $run: string, $operation: string, $request: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'recover', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_attempts', $op.attempt); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $r = SELECT * FROM ONLY type::record('canonical_runs', $run);
    IF $r = NONE OR $r.current_attempt = NONE OR $r.terminal_attempt != NONE { THROW 'PSE_REFUSAL:execution_recovery_unavailable'; };
    LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $r.current_attempt);
    LET $study_cancelled = fn::pse_execution_v1::study_cancelled($rpc_expiry, $run);
    LET $summary_cancelled = fn::pse_execution_v1::summary_cancelled($rpc_expiry, $run);
    IF $a = NONE OR ($a.phase IN ['terminal','terminal_compacted']) OR ($r.cancelled = false AND $study_cancelled = false AND $a.expires_at > time::micros()) { THROW 'PSE_REFUSAL:execution_recovery_still_owned_by_live_worker'; };
    LET $generation = $r.current_generation + 1dec;
    UPDATE ONLY type::record('canonical_runs', $run) SET current_generation = $generation, cancelled = $r.cancelled OR $study_cancelled;
    LET $saved = UPDATE ONLY type::record('canonical_attempts', $a.key) SET phase = IF closed_manifest=NONE { 'closed_awaiting_manifest' } ELSE { 'closed_with_manifest' }, close_generation = $generation;
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$a.key,'recover',$request,type::bytes(<string>$generation),NONE,NONE);
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::manifest($rpc_expiry: int, $run: string, $manifest: object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $study_cancelled=fn::pse_execution_v1::study_cancelled($rpc_expiry,$run);
    LET $summary_cancelled=fn::pse_execution_v1::summary_cancelled($rpc_expiry,$run);
    LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $manifest.attempt);
    IF $a = NONE OR $a.run != $run OR ($a.phase='ingesting') OR $a.generation != $manifest.generation { THROW 'PSE_REFUSAL:execution_closed_descriptor_fenced'; };
    LET $old = SELECT * FROM ONLY type::record('canonical_result_manifests', $manifest.key);
    IF $old != NONE {
        IF $old.attempt != $manifest.attempt OR $old.generation != $manifest.generation OR $old.digest != $manifest.digest OR $old.descriptors != $manifest.descriptors { THROW 'PSE_REFUSAL:execution_immutable_descriptor_changed'; };
        LET $rpc_result = $old;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    IF ($a.phase IN ['terminal','terminal_compacted']) { THROW 'PSE_REFUSAL:execution_terminal_manifest_unavailable'; };
    fn::pse_history_v1::identity($rpc_expiry,'manifest',$manifest.key,$run);
    LET $saved = CREATE ONLY type::record('canonical_result_manifests', $manifest.key) CONTENT $manifest;
    UPDATE ONLY type::record('canonical_attempts', $a.key) SET phase = 'closed_with_manifest', closed_manifest = $manifest.key;
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::seal($rpc_expiry: int, $run: string, $attempt: string, $authority: decimal, $operation: string, $request: bytes, $manifest: string, $digest: string, $outcome: string, $completion: bytes, $seed_eligible: bool) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'seal', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_attempts', $op.attempt); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $r = SELECT * FROM ONLY type::record('canonical_runs', $run);
    LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $attempt);
    LET $m = SELECT * FROM ONLY type::record('canonical_result_manifests', $manifest);
    IF $r = NONE OR $a = NONE OR $m = NONE OR $r.current_attempt != $attempt OR $r.current_generation != $authority OR $a.close_generation != $authority OR $a.run != $run OR ($a.phase IN ['terminal','terminal_compacted']) OR $r.terminal_attempt != NONE OR ($a.phase='ingesting') OR $a.closed_manifest != $manifest OR $m.attempt != $attempt OR $m.generation != $a.generation OR $m.digest != $digest { THROW 'PSE_REFUSAL:execution_terminal_seal_fenced'; };
    IF $outcome NOT IN ['succeeded','failed','partial','cancelled'] { THROW 'PSE_REFUSAL:execution_unknown_terminal_outcome'; };
    LET $study_cancelled = fn::pse_execution_v1::study_cancelled($rpc_expiry, $run);
    IF $study_cancelled AND ($r.cancelled = false OR $authority = $a.generation OR $outcome != 'cancelled') { fn::pse_execution_v1::deadline($rpc_expiry); RETURN {pse_writer:'study-cancelled',run:$run,attempt:$attempt,generation:$a.generation,operation:$operation}; };
    IF fn::pse_execution_v1::summary_cancelled($rpc_expiry, $run) AND $outcome != 'cancelled' { THROW 'PSE_REFUSAL:execution_summary_cancellation_wins_terminal_seal'; };
    IF ($r.cancelled OR $study_cancelled) AND $outcome != 'cancelled' { THROW 'PSE_REFUSAL:execution_cancellation_wins_terminal_seal'; };
    IF $outcome = 'succeeded' AND ($authority != $a.generation OR $a.expires_at <= time::micros()) { THROW 'PSE_REFUSAL:execution_expired_worker_cannot_seal_success'; };
    IF $authority = $a.generation AND $a.expires_at <= time::micros() { THROW 'PSE_REFUSAL:execution_expired_worker_must_recover_before_sealing'; };
    LET $saved = UPDATE ONLY type::record('canonical_attempts', $attempt) SET phase = 'terminal', outcome = $outcome, completion = $completion, seed_manifest = IF $seed_eligible { $manifest } ELSE { NONE };
    UPDATE ONLY type::record('canonical_runs', $run) SET terminal_attempt = $attempt,terminal_class=$outcome;
    // Removing this exact owned root adds no retention dependency. Keep the
    // reclaim/exact-root fences without writing the shared creator generation.
    // The run root remains until its separate terminal retirement owner releases it.
    fn::pse_execution_v1::register($rpc_expiry,'retention:'+$r.problem);
    SELECT * FROM ONLY type::record('canonical_roots','active_attempt:'+$attempt) FOR UPDATE;
    DELETE ONLY type::record('canonical_roots', 'active_attempt:' + $attempt);
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$attempt,'seal',$request,$completion,NONE,NONE);
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $authority = $a.generation { fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at); };
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::renew($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $lifetime: int) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    LET $writer = fn::pse_execution_v1::writer($rpc_expiry, $run, $attempt, $generation, 'renew', 'renew');
    IF $writer.pse_writer != 'admitted' { RETURN $writer; };
    LET $a = $writer.attempt;
    LET $original_expiry = $a.expires_at;
    LET $rpc_result = UPDATE ONLY type::record('canonical_attempts', $attempt) SET expires_at = time::micros() + $lifetime;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $original_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::seed_eligible($seed: object) -> bool {
    LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $seed.attempt);
    IF $a = NONE OR ($a.phase NOT IN ['terminal','terminal_compacted']) OR $a.seed_manifest = NONE OR $a.seed_manifest != $a.closed_manifest OR $a.run != $seed.run OR $a.generation != $seed.attempt_generation { RETURN false; };
    LET $r = SELECT sequence,interpretation FROM ONLY type::record('canonical_runs', $seed.run);
    LET $installed=SELECT interpretation FROM ONLY canonical_interpretations:current;
    IF $r=NONE OR $r.interpretation!=__INTERPRETATION__ OR $installed=NONE OR $installed.interpretation!=__INTERPRETATION__ { RETURN false; };
    LET $m = SELECT attempt,generation FROM ONLY type::record('canonical_result_manifests', $a.seed_manifest);
    RETURN $r != NONE AND $r.sequence = $seed.run_sequence AND $m != NONE AND $m.attempt = $seed.attempt AND $m.generation = $seed.attempt_generation;
};

DEFINE FUNCTION fn::pse_execution_v1::seed($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes, $seed: object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'seed', $request);
    LET $old = SELECT * FROM ONLY type::record('canonical_result_seeds', $seed.key);
    IF $old != NONE {
        IF object::remove($old, 'id') != $seed { THROW 'PSE_REFUSAL:execution_immutable_seed_descriptor_changed'; };
        IF $op != NONE { LET $rpc_result = $old; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    };
    LET $writer = fn::pse_execution_v1::writer($rpc_expiry, $run, $attempt, $generation, $operation, 'seed');
    IF $writer.pse_writer != 'admitted' { RETURN $writer; };
    LET $a = $writer.attempt;
    IF ($a.phase!='ingesting') { THROW 'PSE_REFUSAL:execution_seed_ingestion_closed'; };
    IF $seed.run != $run OR $seed.attempt != $attempt OR $seed.batch_count <= 0dec { THROW 'PSE_REFUSAL:execution_seed_coordinate_mismatch'; };
    LET $set = SELECT * FROM ONLY type::record('canonical_result_sets', $seed.result_set);
    LET $batch = SELECT * FROM ONLY type::record('canonical_result_batches', $seed.batch);
    IF $set = NONE OR $batch = NONE OR $set.attempt != $attempt OR $set.next_ordinal < $seed.first_ordinal + $seed.batch_count OR $batch.attempt != $attempt OR $batch.result_set != $seed.result_set OR $batch.ordinal != $seed.first_ordinal { THROW 'PSE_REFUSAL:execution_seed_chunks_incomplete'; };
    fn::pse_history_v1::identity($rpc_expiry,'seed',$seed.key,$run);
    IF $old = NONE { CREATE ONLY type::record('canonical_result_seeds', $seed.key) CONTENT $seed; };
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$attempt,'seed',$request,$request,NONE,NONE);
    LET $rpc_result = SELECT * FROM ONLY type::record('canonical_result_seeds', $seed.key);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at);
    RETURN $rpc_result;
};
"#;
