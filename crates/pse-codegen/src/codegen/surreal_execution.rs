// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Versioned native execution decisions. Scientific admission belongs to Rust.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(
        &FUNCTIONS
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
            ),
    );
}

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_execution_v1::lease($rpc_expiry: int, $expires_at: int) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $expires_at <= time::micros() { THROW 'execution attempt lease expired before commit'; };
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

DEFINE FUNCTION fn::pse_execution_v1::operation($rpc_expiry: int, $key: string, $kind: string, $request: bytes) -> option<object> {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-operation:' + $key);
    LET $old = SELECT * FROM ONLY type::record('canonical_execution_operations', $key);
    IF $old != NONE AND ($old.kind != $kind OR $old.request != $request) { THROW 'execution operation identity reused'; };
    LET $rpc_result = $old;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::available($rpc_expiry: int, $run: string) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    IF (SELECT * FROM ONLY type::record('canonical_result_retirements',$run)) != NONE { THROW 'execution results explicitly retired'; };
    LET $rpc_result = true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::study_cancelled($rpc_expiry: int, $run: string) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $points = SELECT study FROM canonical_study_points WHERE run = $run LIMIT 1;
    IF array::len($points) = 0 { LET $rpc_result = false; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $points[0].study);
    LET $study = SELECT * FROM ONLY type::record('canonical_studies', $points[0].study);
    IF $study = NONE { THROW 'execution study cancellation authority absent'; };
    LET $rpc_result = $study.cancelled;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::summary_cancelled($rpc_expiry: int, $run: string) -> bool {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $parents = SELECT key FROM canonical_studies WHERE run=$run LIMIT 1;
    IF array::len($parents) = 0 { LET $rpc_result = false; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $parents[0].key);
    LET $study = SELECT * FROM ONLY type::record('canonical_studies',$parents[0].key);
    IF $study = NONE { THROW 'execution summary cancellation authority absent'; };
    LET $rpc_result = $study.cancelled;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::fence($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $r = SELECT * FROM ONLY type::record('canonical_runs', $run);
    LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $attempt);
    IF fn::pse_execution_v1::study_cancelled($rpc_expiry, $run) { THROW 'execution study cancellation fence revoked'; };
    IF $r = NONE OR $a = NONE OR $a.run != $run OR $r.current_attempt != $attempt OR $r.current_generation != $generation OR $a.generation != $generation OR $r.cancelled OR $r.terminal_attempt != NONE OR $a.terminal OR $a.expires_at <= time::micros() { THROW 'execution attempt fence revoked'; };
    LET $rpc_result = $a;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::begin_run($rpc_expiry: int, $row: object, $sources: array<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $row.key);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $row.key);
    LET $old = SELECT * FROM ONLY type::record('canonical_runs', $row.key);
    IF $old != NONE {
        IF $old.request != $row.request OR $old.problem != $row.problem OR $old.revision != $row.revision OR $old.source_sequence != $row.source_sequence OR $old.source_selection != $row.source_selection OR $old.attestation != $row.attestation OR $old.interpretation != $row.interpretation { THROW 'execution run identity reused'; };
        LET $rpc_result = $old;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    fn::pse_execution_v1::touch($rpc_expiry, 'retention:' + $row.problem);
    LET $revision = SELECT * FROM ONLY type::record('canonical_revisions', $row.revision);
    IF $revision = NONE OR $revision.problem != $row.problem OR $revision.sequence != $row.source_sequence OR $revision.interpretation != $row.interpretation { THROW 'execution revision unavailable'; };
    LET $pruned = SELECT key FROM canonical_reclaimed_ranges WHERE problem = $row.problem AND from_sequence <= $revision.sequence AND to_sequence > $revision.sequence LIMIT 1;
    IF array::len($pruned) != 0 { THROW 'execution revision source reclaimed'; };
    IF array::len($sources)>64 { THROW 'execution sources limit exceeded'; };
    FOR $source IN $sources {
        fn::pse_execution_v1::touch($rpc_expiry, 'retention:' + $source.problem);
        LET $selected = SELECT * FROM ONLY type::record('canonical_revisions',$source.revision);
        IF $selected=NONE OR $selected.problem!=$source.problem OR $selected.sequence!=$source.sequence OR $selected.interpretation!=$source.interpretation { THROW 'execution selected source unavailable'; };
        LET $reclaimed=SELECT key FROM canonical_reclaimed_ranges WHERE problem=$source.problem AND from_sequence<=$source.sequence AND to_sequence>$source.sequence LIMIT 1;
        IF array::len($reclaimed)!=0 { THROW 'execution selected source reclaimed'; };
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
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::claim($rpc_expiry: int, $run: string, $operation: string, $request: bytes, $attempt: string, $worker: string, $lifetime: int, $interpretation: string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'claim', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_attempts', $op.attempt); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $r = SELECT * FROM ONLY type::record('canonical_runs', $run);
    IF fn::pse_execution_v1::study_cancelled($rpc_expiry, $run) { THROW 'execution cancelled study run not claimable'; };
    IF $r = NONE OR $r.cancelled OR $r.interpretation != $interpretation { THROW 'execution run not claimable'; };
    IF $r.current_attempt != NONE {
        LET $previous = SELECT * FROM ONLY type::record('canonical_attempts', $r.current_attempt);
        IF $previous != NONE AND ($previous.terminal = false OR $previous.outcome = 'succeeded') { THROW 'execution attempt is live or already succeeded'; };
    };
    LET $generation = $r.current_generation + 1dec;
    LET $saved = CREATE ONLY type::record('canonical_attempts', $attempt) SET key = $attempt, run = $run, generation = $generation, claim_operation = $operation, request = $request, worker = $worker, expires_at = time::micros() + $lifetime, ingestion_open = true, closed = false, terminal = false;
    UPDATE ONLY type::record('canonical_runs', $run) SET current_generation = $generation, current_attempt = $attempt, terminal_attempt = NONE,terminal_class=NONE;
    fn::pse_execution_v1::touch($rpc_expiry, 'retention:' + $r.problem);
    LET $source_root = SELECT * FROM ONLY type::record('canonical_roots', 'run:' + $run);
    IF $source_root = NONE OR $source_root.problem != $r.problem OR $source_root.revision != $r.revision OR $source_root.sequence != $r.source_sequence OR $source_root.owner_kind != 'run' OR $source_root.owner != $run { THROW 'execution retained source selection unavailable'; };
    CREATE ONLY type::record('canonical_roots', 'active_attempt:' + $attempt) SET key = 'active_attempt:' + $attempt, problem = $r.problem, revision = $r.revision, sequence = $r.source_sequence, owner_kind = 'active_attempt', owner = $attempt;
    CREATE ONLY type::record('canonical_execution_operations', $operation) SET key = $operation, run = $run, attempt = $attempt, kind = 'claim', request = $request, result = $request;
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $saved.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::append($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes, $set: object, $batch: object, $block: option<object>, $cells: array<object>, $outputs: array<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF array::len($cells) > __INDEX_RECORDS__ OR array::len($outputs) > __INDEX_RECORDS__ OR bytes::len(encoding::cbor::encode({block:$block,cells:$cells,outputs:$outputs})) > __INDEX_BYTES__ { THROW 'execution index metadata extent exceeded'; };
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'append', $request);
    LET $a = IF $op = NONE { fn::pse_execution_v1::fence($rpc_expiry, $run, $attempt, $generation) } ELSE { NONE };
    IF $op = NONE AND ($a.ingestion_open = false OR $a.closed) { THROW 'execution result ingestion closed'; };
    IF $batch.attempt != $attempt OR $set.attempt != $attempt OR $batch.result_set != $set.key { THROW 'execution batch coordinate mismatch'; };
    LET $existing = SELECT * FROM ONLY type::record('canonical_result_batches', $batch.key);
    LET $existing_block = SELECT * FROM ONLY type::record('canonical_result_blocks', $batch.key);
    IF $existing != NONE {
        IF $existing.payload != $batch.payload OR $existing.digest != $batch.digest OR $existing.ordinal != $batch.ordinal OR $existing.row_count != $batch.row_count OR $existing.result_set != $batch.result_set OR $existing.attempt != $attempt { THROW 'execution immutable batch changed'; };
        IF ($block = NONE AND $existing_block != NONE) OR ($block != NONE AND ($existing_block = NONE OR object::remove($existing_block, 'id') != $block)) { THROW 'execution immutable block changed'; };
        LET $old_cells = SELECT * FROM canonical_result_cells WHERE batch = $batch.key ORDER BY key LIMIT __INDEX_REPLAY__;
        IF array::len($old_cells) != array::len($cells) OR $old_cells.map(|$cell| object::remove($cell, 'id')) != $cells { THROW 'execution immutable scalar cells changed'; };
        LET $old_outputs = SELECT * FROM canonical_result_block_outputs WHERE batch = $batch.key ORDER BY key LIMIT __INDEX_REPLAY__;
        IF array::len($old_outputs) != array::len($outputs) OR $old_outputs.map(|$output| object::remove($output, 'id')) != $outputs { THROW 'execution immutable output block indexes changed'; };
        IF $op != NONE { LET $rpc_result = {key:$batch.key,attempt:$attempt,result_set:$set.key,ordinal:$batch.ordinal,digest:$batch.digest,row_count:$batch.row_count,request:$request}; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    } ELSE {
        IF $op != NONE { THROW 'execution acknowledged batch unavailable'; };
        LET $s = SELECT * FROM ONLY type::record('canonical_result_sets', $set.key);
        IF $s = NONE {
            LET $sets = SELECT key FROM canonical_result_sets WHERE attempt = $attempt LIMIT 256;
            IF array::len($sets) >= 256 { THROW 'execution result-set bound exceeded'; };
            IF $batch.ordinal != 0dec { THROW 'execution batch sequence gap'; };
            CREATE ONLY type::record('canonical_result_sets', $set.key) CONTENT $set;
        } ELSE {
            IF $s.attempt != $attempt OR $s.name != $set.name OR $s.interpretation != $set.interpretation OR $s.next_ordinal != $batch.ordinal { THROW 'execution batch sequence gap'; };
        };
        CREATE ONLY type::record('canonical_result_batches', $batch.key) CONTENT $batch;
        IF $block != NONE { CREATE ONLY type::record('canonical_result_blocks', $batch.key) CONTENT $block; };
        IF array::len($cells) != 0 {
            LET $rows = $cells.map(|$cell| object::extend($cell, {id:type::record('canonical_result_cells', $cell.key)}));
            INSERT INTO canonical_result_cells $rows RETURN NONE;
        };
        IF array::len($outputs) != 0 {
            LET $rows = $outputs.map(|$output| object::extend($output, {id:type::record('canonical_result_block_outputs', $output.key)}));
            INSERT INTO canonical_result_block_outputs $rows RETURN NONE;
        };
        UPDATE ONLY type::record('canonical_result_sets', $set.key) SET next_ordinal += 1dec, row_count += $batch.row_count;
    };
    CREATE ONLY type::record('canonical_execution_operations', $operation) SET key = $operation, run = $run, attempt = $attempt, kind = 'append', request = $request, result = $request;
    LET $rpc_result = {key:$batch.key,attempt:$attempt,result_set:$set.key,ordinal:$batch.ordinal,digest:$batch.digest,row_count:$batch.row_count,request:$request};
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $op = NONE { fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at); };
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::close($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'close', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_attempts', $op.attempt); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $a = fn::pse_execution_v1::fence($rpc_expiry, $run, $attempt, $generation);
    IF $a.closed { THROW 'execution ingestion already closed'; };
    LET $saved = UPDATE ONLY type::record('canonical_attempts', $attempt) SET ingestion_open = false, closed = true, close_generation = $generation;
    CREATE ONLY type::record('canonical_execution_operations', $operation) SET key = $operation, run = $run, attempt = $attempt, kind = 'close', request = $request, result = type::bytes(<string>$generation);
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::cancel($rpc_expiry: int, $run: string, $operation: string, $request: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'cancel', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_runs', $run); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $r = SELECT * FROM ONLY type::record('canonical_runs', $run);
    IF $r = NONE { THROW 'execution run unavailable'; };
    IF $r.terminal_attempt != NONE { THROW 'execution run already sealed'; };
    LET $generation = $r.current_generation + 1dec;
    LET $saved = UPDATE ONLY type::record('canonical_runs', $run) SET cancelled = true, current_generation = $generation;
    IF $r.current_attempt != NONE { UPDATE ONLY type::record('canonical_attempts', $r.current_attempt) SET ingestion_open = false, closed = true, close_generation = $generation; };
    CREATE ONLY type::record('canonical_execution_operations', $operation) SET key = $operation, run = $run, kind = 'cancel', request = $request, result = $request;
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
    IF $r = NONE OR $r.current_attempt = NONE OR $r.terminal_attempt != NONE { THROW 'execution recovery unavailable'; };
    LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $r.current_attempt);
    LET $study_cancelled = fn::pse_execution_v1::study_cancelled($rpc_expiry, $run);
    IF $a = NONE OR $a.terminal OR ($r.cancelled = false AND $study_cancelled = false AND $a.expires_at > time::micros()) { THROW 'execution recovery still owned by live worker'; };
    LET $generation = $r.current_generation + 1dec;
    UPDATE ONLY type::record('canonical_runs', $run) SET current_generation = $generation, cancelled = $r.cancelled OR $study_cancelled;
    LET $saved = UPDATE ONLY type::record('canonical_attempts', $a.key) SET ingestion_open = false, closed = true, close_generation = $generation;
    CREATE ONLY type::record('canonical_execution_operations', $operation) SET key = $operation, run = $run, attempt = $a.key, kind = 'recover', request = $request, result = type::bytes(<string>$generation);
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::manifest($rpc_expiry: int, $run: string, $manifest: object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $manifest.attempt);
    IF $a = NONE OR $a.run != $run OR $a.closed = false OR $a.ingestion_open OR $a.generation != $manifest.generation { THROW 'execution closed descriptor fenced'; };
    LET $old = SELECT * FROM ONLY type::record('canonical_result_manifests', $manifest.key);
    IF $old != NONE {
        IF $old.attempt != $manifest.attempt OR $old.generation != $manifest.generation OR $old.digest != $manifest.digest OR $old.descriptors != $manifest.descriptors { THROW 'execution immutable descriptor changed'; };
        LET $rpc_result = $old;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    IF $a.terminal { THROW 'execution terminal manifest unavailable'; };
    LET $saved = CREATE ONLY type::record('canonical_result_manifests', $manifest.key) CONTENT $manifest;
    UPDATE ONLY type::record('canonical_attempts', $a.key) SET closed_manifest = $manifest.key;
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::seal($rpc_expiry: int, $run: string, $attempt: string, $authority: decimal, $operation: string, $request: bytes, $manifest: string, $digest: string, $outcome: string, $completion: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'seal', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_attempts', $op.attempt); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'execution-run:' + $run);
    LET $r = SELECT * FROM ONLY type::record('canonical_runs', $run);
    LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $attempt);
    LET $m = SELECT * FROM ONLY type::record('canonical_result_manifests', $manifest);
    IF $r = NONE OR $a = NONE OR $m = NONE OR $r.current_attempt != $attempt OR $r.current_generation != $authority OR $a.close_generation != $authority OR $a.run != $run OR $a.terminal OR $r.terminal_attempt != NONE OR $a.closed = false OR $a.ingestion_open OR $a.closed_manifest != $manifest OR $m.attempt != $attempt OR $m.generation != $a.generation OR $m.digest != $digest { THROW 'execution terminal seal fenced'; };
    IF $outcome NOT IN ['succeeded','failed','partial','cancelled'] { THROW 'execution unknown terminal outcome'; };
    LET $study_cancelled = fn::pse_execution_v1::study_cancelled($rpc_expiry, $run);
    IF $study_cancelled AND ($r.cancelled = false OR $authority = $a.generation) { THROW 'execution cancelled study requires current recovery authority'; };
    IF fn::pse_execution_v1::summary_cancelled($rpc_expiry, $run) AND $outcome != 'cancelled' { THROW 'execution summary cancellation wins terminal seal'; };
    IF ($r.cancelled OR $study_cancelled) AND $outcome != 'cancelled' { THROW 'execution cancellation wins terminal seal'; };
    IF $outcome = 'succeeded' AND ($authority != $a.generation OR $a.expires_at <= time::micros()) { THROW 'execution expired worker cannot seal success'; };
    IF $authority = $a.generation AND $a.expires_at <= time::micros() { THROW 'execution expired worker must recover before sealing'; };
    LET $saved = UPDATE ONLY type::record('canonical_attempts', $attempt) SET terminal = true, outcome = $outcome, completion = $completion;
    UPDATE ONLY type::record('canonical_runs', $run) SET terminal_attempt = $attempt,terminal_class=$outcome;
    fn::pse_execution_v1::touch($rpc_expiry, 'retention:' + $r.problem);
    DELETE ONLY type::record('canonical_roots', 'active_attempt:' + $attempt);
    CREATE ONLY type::record('canonical_execution_operations', $operation) SET key = $operation, run = $run, attempt = $attempt, kind = 'seal', request = $request, result = $completion;
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $authority = $a.generation { fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at); };
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::renew($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $lifetime: int) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $a = fn::pse_execution_v1::fence($rpc_expiry, $run, $attempt, $generation);
    LET $original_expiry = $a.expires_at;
    LET $rpc_result = UPDATE ONLY type::record('canonical_attempts', $attempt) SET expires_at = time::micros() + $lifetime;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $original_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_execution_v1::seed($rpc_expiry: int, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes, $seed: object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::available($rpc_expiry, $run);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'seed', $request);
    LET $old = SELECT * FROM ONLY type::record('canonical_result_seeds', $seed.key);
    IF $old != NONE {
        IF object::remove($old, 'id') != $seed { THROW 'execution immutable seed descriptor changed'; };
        IF $op != NONE { LET $rpc_result = $old; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    };
    LET $a = fn::pse_execution_v1::fence($rpc_expiry, $run, $attempt, $generation);
    IF $a.ingestion_open = false OR $a.closed { THROW 'execution seed ingestion closed'; };
    IF $seed.run != $run OR $seed.attempt != $attempt OR $seed.batch_count <= 0dec { THROW 'execution seed coordinate mismatch'; };
    LET $set = SELECT * FROM ONLY type::record('canonical_result_sets', $seed.result_set);
    LET $batch = SELECT * FROM ONLY type::record('canonical_result_batches', $seed.batch);
    IF $set = NONE OR $batch = NONE OR $set.attempt != $attempt OR $set.next_ordinal < $seed.first_ordinal + $seed.batch_count OR $batch.attempt != $attempt OR $batch.result_set != $seed.result_set OR $batch.ordinal != $seed.first_ordinal { THROW 'execution seed chunks incomplete'; };
    IF $old = NONE { CREATE ONLY type::record('canonical_result_seeds', $seed.key) CONTENT $seed; };
    CREATE ONLY type::record('canonical_execution_operations', $operation) SET key = $operation, run = $run, attempt = $attempt, kind = 'seed', request = $request, result = $request;
    LET $rpc_result = SELECT * FROM ONLY type::record('canonical_result_seeds', $seed.key);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at);
    RETURN $rpc_result;
};
"#;
