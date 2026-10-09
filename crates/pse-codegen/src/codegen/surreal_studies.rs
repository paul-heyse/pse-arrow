// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Structural study transitions; scientific decisions remain at the shared policy owner.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(FUNCTIONS);
}

const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_study_v1::create($rpc_expiry: int, $row: object) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $row.key);
    LET $old = SELECT * FROM ONLY type::record('canonical_studies', $row.key);
    IF $old != NONE {
        IF $old.run != $row.run OR $old.revision != $row.revision OR $old.problem != $row.problem OR $old.metadata != $row.metadata OR $old.point_count != $row.point_count OR $old.interpretation != $row.interpretation { THROW 'study identity reused'; };
        LET $rpc_result = $old;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    LET $run = SELECT * FROM ONLY type::record('canonical_runs', $row.run);
    IF $run = NONE OR $run.problem != $row.problem OR $run.revision != $row.revision { THROW 'study source run unavailable'; };
    LET $rpc_result = CREATE ONLY type::record('canonical_studies', $row.key) CONTENT $row;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::append($rpc_expiry: int, $study: string, $points: array<object>, $edges: array<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study);
    IF $s = NONE OR array::len($points) = 0 OR array::len($points) > 64 { THROW 'study occurrence batch unavailable'; };
    LET $existing = $points.map(|$point| SELECT key FROM ONLY type::record('canonical_study_points', $point.key)).filter(|$row| $row != NONE);
    IF array::len($existing) != 0 AND array::len($existing) != array::len($points) { THROW 'study occurrence batch mixed settlement'; };
    FOR $point IN $points {
        IF $point.study != $study { THROW 'study occurrence belongs elsewhere'; };
        LET $old = SELECT * FROM ONLY type::record('canonical_study_points', $point.key);
        IF $old != NONE {
            IF $old.ordinal != $point.ordinal OR $old.occurrence != $point.occurrence OR $old.policy != $point.policy OR $old.descriptor != $point.descriptor OR $old.run != $point.run { THROW 'study immutable occurrence changed'; };
        } ELSE {
            LET $run = SELECT * FROM ONLY type::record('canonical_runs', $point.run);
            IF $run = NONE OR $run.problem != $s.problem OR $run.revision != $s.revision { THROW 'study occurrence source run unavailable'; };
        };
    };
    IF array::len($existing) = 0 {
        IF $s.active OR $s.cancelled OR array::len(array::distinct($points.ordinal)) != array::len($points) OR array::min($points.ordinal) != $s.next_ordinal OR array::max($points.ordinal) + 1dec != $s.next_ordinal + array::len($points) OR $s.next_ordinal + array::len($points) > $s.point_count { THROW 'study membership closed or sequence gap'; };
        LET $new = $points.map(|$point| object::extend($point, {id:type::record('canonical_study_points', $point.key)}));
        INSERT INTO canonical_study_points $new RETURN NONE;
        FOR $edge IN $edges {
            LET $old = SELECT * FROM ONLY type::record('canonical_study_dependencies', $edge.key);
            IF $old != NONE {
                IF object::remove($old, ['id', 'in', 'out']) != $edge { THROW 'study immutable dependency changed'; };
            } ELSE {
                INSERT RELATION INTO canonical_study_dependencies object::extend($edge, {id:type::record('canonical_study_dependencies', $edge.key), in:type::record('canonical_study_points', $edge.dependent), out:type::record('canonical_study_points', $edge.predecessor)}) RETURN NONE;
            };
        };
        UPDATE ONLY type::record('canonical_studies', $study) SET next_ordinal += <decimal>array::len($points);
    };
    LET $rpc_result = SELECT * FROM ONLY type::record('canonical_studies', $study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::activate($rpc_expiry: int, $study: string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study);
    IF $s = NONE OR $s.cancelled OR $s.point_count = 0dec OR $s.next_ordinal != $s.point_count { THROW 'study membership incomplete'; };
    LET $rpc_result = UPDATE ONLY type::record('canonical_studies', $study) SET active = true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::scope($rpc_expiry: int, $point: string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $p = SELECT key, study, ordinal, occurrence, policy, facts, outcome, run, revision, assigned, settled, attempt, start FROM ONLY type::record('canonical_study_points', $point);
    IF $p = NONE { THROW 'study candidate unavailable'; };
    LET $s = SELECT key, generation, cancelled, active, terminal FROM ONLY type::record('canonical_studies', $p.study);
    IF $s = NONE OR $s.active = false { THROW 'study not active'; };
    LET $edges = SELECT predecessor FROM canonical_study_dependencies WHERE dependent = $point LIMIT 193;
    IF array::len(array::distinct($edges.predecessor)) > 64 { THROW 'study candidate immediate dependency bound'; };
    LET $keys = array::distinct($edges.predecessor);
    LET $predecessors = $keys.map(|$key| SELECT key, study, ordinal, occurrence, policy, facts, outcome, revision, assigned, settled, attempt, run, start FROM ONLY type::record('canonical_study_points', $key));
    LET $rpc_result = {study:$s, point:$p, predecessors:$predecessors};
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::guard($rpc_expiry: int, $study: string, $generation: decimal, $cancelled: bool, $point: string, $expected: array<object>) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study);
    IF $s = NONE OR $s.active = false OR $s.terminal OR $s.generation != $generation OR $s.cancelled != $cancelled { THROW 'study decision generation changed'; };
    LET $p = SELECT * FROM ONLY type::record('canonical_study_points', $point);
    IF $p = NONE OR $p.study != $study OR $p.assigned OR $p.settled { THROW 'study candidate not available'; };
    LET $edges = SELECT predecessor FROM canonical_study_dependencies WHERE dependent = $point LIMIT 193;
    IF array::len(array::distinct($edges.predecessor)) > 64 { THROW 'study dependency bound'; };
    LET $required = array::sort(array::distinct(array::concat([$point], $edges.predecessor)));
    IF array::sort($expected.key) != $required { THROW 'study decision omitted or added a premise'; };
    FOR $item IN $expected {
        fn::pse_execution_v1::touch($rpc_expiry, 'study-point:' + $item.key);
        LET $current = SELECT * FROM ONLY type::record('canonical_study_points', $item.key);
        IF $current = NONE OR $current.study != $study OR $current.revision != $item.revision { THROW 'study decision premise changed'; };
    };
    LET $rpc_result = $p;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::settle($rpc_expiry: int, $study: string, $generation: decimal, $cancelled: bool, $point: string, $expected: array<object>, $facts: bytes, $outcome: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $p = fn::pse_study_v1::guard($rpc_expiry, $study,$generation,$cancelled,$point,$expected);
    LET $rpc_result = UPDATE ONLY type::record('canonical_study_points',$point) SET revision += 1dec, facts=$facts,outcome=$outcome,settled=true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::started($rpc_expiry: int, $study: string, $point: string, $revision: decimal, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes, $facts: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies',$study);
    IF $s = NONE OR $s.cancelled OR $s.terminal { THROW 'study cancelled before native dispatch'; };
    LET $a = fn::pse_execution_v1::fence($rpc_expiry, $run,$attempt,$generation);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation,'study-start',$request);
    IF $op != NONE {
        IF $op.run != $run OR $op.attempt != $attempt { THROW 'study start operation reused'; };
        LET $rpc_result = SELECT * FROM ONLY type::record('canonical_study_points',$point);
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    LET $started = SELECT key FROM canonical_execution_operations WHERE run=$run AND kind='study-start' AND attempt=$attempt LIMIT 1;
    IF array::len($started) != 0 { THROW 'study native start already admitted'; };
    fn::pse_execution_v1::touch($rpc_expiry, 'study-point:' + $point);
    LET $p = SELECT * FROM ONLY type::record('canonical_study_points',$point);
    IF $p = NONE OR $p.study != $study OR $p.run != $run OR $p.attempt != $attempt OR $p.revision != $revision OR $p.assigned = false OR $p.settled { THROW 'study native start premise changed'; };
    LET $saved = UPDATE ONLY type::record('canonical_study_points',$point) SET revision += 1dec,facts=$facts;
    CREATE ONLY type::record('canonical_execution_operations',$operation) SET key=$operation,run=$run,attempt=$attempt,kind='study-start',request=$request,result=$request;
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::finalize($rpc_expiry: int, $study: string, $operation: string, $request: bytes, $attempt: string, $worker: string, $lifetime: int) -> option<object> {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies',$study);
    IF $s = NONE OR $s.active = false { THROW 'study unavailable'; };
    IF $s.terminal { LET $rpc_result = NONE; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $pending = SELECT key FROM canonical_study_points WHERE study=$study AND settled=false LIMIT 1;
    IF array::len($pending) != 0 { LET $rpc_result = NONE; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $run = SELECT * FROM ONLY type::record('canonical_runs',$s.run);
    IF $run.terminal_class = 'succeeded' { LET $rpc_result = NONE; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    IF $run.current_attempt != NONE {
        LET $current = SELECT * FROM ONLY type::record('canonical_attempts',$run.current_attempt);
        IF $current != NONE AND $current.terminal = false AND $current.expires_at > time::micros() { LET $rpc_result = NONE; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    };
    LET $existing_operation = SELECT key FROM ONLY type::record('canonical_execution_operations', $operation);
    LET $rpc_result = fn::pse_execution_v1::claim($rpc_expiry, $s.run,$operation,$request,$attempt,$worker,$lifetime,$s.interpretation);
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $existing_operation = NONE { fn::pse_execution_v1::lease($rpc_expiry, $rpc_result.expires_at); };
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::seal($rpc_expiry: int, $study: string, $run: string, $attempt: string, $authority: decimal, $operation: string, $request: bytes, $manifest: string, $digest: string, $outcome: string, $completion: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies',$study);
    IF $s = NONE OR $s.active = false OR $s.run != $run { THROW 'study summary association unavailable'; };
    LET $pending = SELECT key FROM canonical_study_points WHERE study=$study AND settled=false LIMIT 1;
    IF array::len($pending) != 0 { THROW 'study summary has unsettled scientific occurrences'; };
    IF $s.cancelled AND $outcome != 'cancelled' { THROW 'study summary cancellation wins admission'; };
    LET $existing_operation = SELECT key FROM ONLY type::record('canonical_execution_operations', $operation);
    LET $saved = fn::pse_execution_v1::seal($rpc_expiry, $run,$attempt,$authority,$operation,$request,$manifest,$digest,$outcome,$completion);
    UPDATE ONLY type::record('canonical_studies',$study) SET terminal=true;
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $existing_operation = NONE AND $authority = $saved.generation { fn::pse_execution_v1::lease($rpc_expiry, $saved.expires_at); };
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::claim($rpc_expiry: int, $study: string, $generation: decimal, $point: string, $expected: array<object>, $start: bytes, $operation: string, $request: bytes, $claim_request: bytes, $attempt: string, $worker: string, $lifetime: int, $facts: bytes) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'study-claim', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_attempts', $op.attempt); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study);
    IF $s = NONE OR $s.active = false OR $s.cancelled OR $s.terminal OR $s.generation != $generation { THROW 'study claim cancellation or generation changed'; };
    LET $p = SELECT * FROM ONLY type::record('canonical_study_points', $point);
    IF $p = NONE OR $p.study != $study OR $p.assigned OR $p.settled { THROW 'study candidate not claimable'; };
    LET $edges = SELECT predecessor FROM canonical_study_dependencies WHERE dependent = $point LIMIT 193;
    IF array::len(array::distinct($edges.predecessor)) > 64 { THROW 'study dependency bound'; };
    LET $required = array::sort(array::distinct(array::concat([$point], $edges.predecessor)));
    IF array::sort($expected.key) != $required { THROW 'study claim omitted or added a premise'; };
    FOR $item IN $expected {
        fn::pse_execution_v1::touch($rpc_expiry, 'study-point:' + $item.key);
        LET $current = SELECT * FROM ONLY type::record('canonical_study_points', $item.key);
        IF $current = NONE OR $current.study != $study OR $current.revision != $item.revision { THROW 'study point premise changed'; };
    };
    LET $claimed = fn::pse_execution_v1::claim($rpc_expiry, $p.run, $operation + ':attempt', $claim_request, $attempt, $worker, $lifetime, $s.interpretation);
    UPDATE ONLY type::record('canonical_study_points', $point) SET assigned = true, revision += 1dec, facts = $facts, start = $start, attempt = $attempt;
    CREATE ONLY type::record('canonical_execution_operations', $operation) SET key = $operation, run = $p.run, attempt = $attempt, kind = 'study-claim', request = $request, result = $request;
    LET $rpc_result = $claimed;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $claimed.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::observe($rpc_expiry: int, $point: string, $revision: decimal, $attempt: option<string>, $facts: bytes, $outcome: bytes, $settled: bool) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study-point:' + $point);
    LET $p = SELECT * FROM ONLY type::record('canonical_study_points', $point);
    IF $p = NONE { THROW 'study observation unavailable'; };
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $p.study);
    IF $p.revision != $revision {
        IF $p.outcome = $outcome AND $p.facts = $facts AND $p.settled = $settled { LET $rpc_result = $p; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
        THROW 'study observation revision changed';
    };
    IF $p.settled OR $p.attempt != $attempt { THROW 'study observation attempt changed or already settled'; };
    IF $attempt != NONE {
        LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $attempt);
        IF $a = NONE OR $a.run != $p.run OR $a.terminal = false { THROW 'study result attempt not admitted'; };
    };
    LET $rpc_result = UPDATE ONLY type::record('canonical_study_points', $point) SET revision += 1dec, assigned = false, settled = $settled, facts = $facts, outcome = $outcome;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::cancel($rpc_expiry: int, $study: string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study);
    IF $s = NONE OR $s.terminal { THROW 'study unavailable or already concluded'; };
    IF $s.cancelled { LET $rpc_result = $s; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $rpc_result = UPDATE ONLY type::record('canonical_studies', $study) SET cancelled = true, generation += 1dec;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::conclude($rpc_expiry: int, $study: string) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study);
    IF $s = NONE OR $s.active = false { THROW 'study unavailable'; };
    LET $pending = SELECT key FROM canonical_study_points WHERE study = $study AND settled = false LIMIT 1;
    IF array::len($pending) != 0 { THROW 'study occurrences not all settled'; };
    LET $run = SELECT * FROM ONLY type::record('canonical_runs',$s.run);
    IF $run = NONE OR $run.terminal_attempt = NONE { THROW 'study result manifest not admitted'; };
    LET $rpc_result = UPDATE ONLY type::record('canonical_studies', $study) SET terminal = true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};
"#;
