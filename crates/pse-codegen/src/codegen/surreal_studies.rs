// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Structural study transitions; scientific decisions remain at the shared policy owner.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(FUNCTIONS);
}

const FUNCTIONS: &str = r#"
// Every point writer queues its own derivation and outgoing propagation in the
// same transaction. These rows are evidence for discovery, never permission.
DEFINE FUNCTION fn::pse_study_v1::readiness_invalidate($rpc_expiry:int,$p: object, $initial: bool) -> bool {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$p.study);
    SELECT * FROM ONLY type::record('canonical_study_readiness',$p.key) FOR UPDATE;
    LET $edges = type::record('canonical_study_points',$p.key)->(SELECT predecessor FROM canonical_study_dependencies LIMIT 193);
    LET $keys = array::distinct($edges.predecessor);
    IF array::len($keys)>64 { THROW 'PSE_REFUSAL:study_readiness_immediate_dependency_bound'; };
    LET $predecessors = $keys.map(|$key| SELECT settled FROM ONLY type::record('canonical_study_points',$key));
    LET $pending = <decimal>array::len($predecessors.filter(|$row| $row=NONE OR $row.settled=false));
    UPSERT ONLY type::record('canonical_study_readiness',$p.key) CONTENT {
        key:$p.key,point:$p.key,study:$p.study,ordinal:$p.ordinal,source_revision:$p.revision,
        pending:$pending,eligible:($p.assigned=false AND $p.settled=false AND ($initial=false OR $pending=0dec)),
        premises:NONE,dirty:($initial=false),notify:($initial=false),notify_revision:$p.revision,notify_after:''
    };
    RETURN true;
};

DEFINE FUNCTION fn::pse_study_v1::readiness_cursor($rpc_expiry:int,$study:string,$fresh:bool) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    SELECT * FROM ONLY type::record('canonical_study_readiness_cursors',$study) FOR UPDATE;
    LET $old=SELECT * FROM ONLY type::record('canonical_study_readiness_cursors',$study);
    IF $old!=NONE { RETURN $old; };
    LET $s=SELECT key FROM ONLY type::record('canonical_studies',$study);
    IF $s=NONE { THROW 'PSE_REFUSAL:study_readiness_owner_unavailable'; };
    LET $result=CREATE ONLY type::record('canonical_study_readiness_cursors',$study) SET
        key=$study,study=$study,revision=0dec,rebuild_pending=($fresh=false),
        rebuild_after=NONE,notify_after=NONE,verify_after=NONE;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $result;
};

DEFINE FUNCTION fn::pse_study_v1::readiness_repair($rpc_expiry:int,$study:string,$generation:decimal,$point:string,$expected:array<object>,$pending:decimal,$eligible:bool,$premises:bytes) -> bool {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry,'study-cancel:'+$study);
    LET $s=SELECT active,cancelled,generation,terminal FROM ONLY type::record('canonical_studies',$study);
    IF $s=NONE OR $s.active=false OR $s.cancelled OR $s.terminal OR $s.generation!=$generation { RETURN false; };
    LET $p=SELECT * FROM ONLY type::record('canonical_study_points',$point);
    IF $p=NONE OR $p.study!=$study { THROW 'PSE_REFUSAL:study_readiness_point_unavailable'; };
    LET $edges=type::record('canonical_study_points',$point)->(SELECT predecessor FROM canonical_study_dependencies LIMIT 193);
    LET $keys=array::distinct($edges.predecessor);
    IF array::len($keys)>64 OR $pending<0dec OR $pending > <decimal>array::len($keys) { THROW 'PSE_REFUSAL:study_readiness_dependency_extent'; };
    IF array::sort($expected.key)!=array::sort(array::concat([$point],$keys)) { THROW 'PSE_REFUSAL:study_readiness_omitted_or_added_premise'; };
    FOR $item IN $expected {
        SELECT * FROM ONLY type::record('canonical_study_points',$item.key) FOR UPDATE;
        LET $current=SELECT study,revision FROM ONLY type::record('canonical_study_points',$item.key);
        IF $current=NONE OR $current.study!=$study OR $current.revision!=$item.revision { RETURN false; };
    };
    SELECT * FROM ONLY type::record('canonical_study_readiness',$point) FOR UPDATE;
    LET $hint=SELECT * FROM ONLY type::record('canonical_study_readiness',$point);
    IF $hint=NONE OR $hint.source_revision!=$p.revision {
        fn::pse_study_v1::readiness_invalidate($rpc_expiry,$p,false);
    };
    UPDATE ONLY type::record('canonical_study_readiness',$point) SET
        source_revision=$p.revision,pending=$pending,eligible=($eligible AND $p.assigned=false AND $p.settled=false),premises=$premises,dirty=false;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN true;
};

DEFINE FUNCTION fn::pse_study_v1::readiness_notify_advance($rpc_expiry:int,$study:string,$point:string,$revision:decimal,$before:string,$after:string,$done:bool) -> bool {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    SELECT * FROM ONLY type::record('canonical_study_readiness',$point) FOR UPDATE;
    LET $hint=SELECT * FROM ONLY type::record('canonical_study_readiness',$point);
    IF $hint=NONE OR $hint.study!=$study OR $hint.notify=false OR $hint.notify_revision!=$revision OR $hint.notify_after!=$before OR $hint.dirty { RETURN false; };
    UPDATE ONLY type::record('canonical_study_readiness',$point) SET notify=($done=false),notify_after=$after;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN true;
};

DEFINE FUNCTION fn::pse_study_v1::readiness_scan_advance($rpc_expiry:int,$study:string,$revision:decimal,$mode:string,$after:option<decimal>,$done:bool) -> bool {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    SELECT * FROM ONLY type::record('canonical_study_readiness_cursors',$study) FOR UPDATE;
    LET $old=SELECT * FROM ONLY type::record('canonical_study_readiness_cursors',$study);
    IF $old=NONE OR $old.revision!=$revision { RETURN false; };
    IF $mode='rebuild' {
        UPDATE ONLY type::record('canonical_study_readiness_cursors',$study) SET revision+=1dec,rebuild_after=$after,rebuild_pending=($done=false);
    } ELSE IF $mode='verify' {
        UPDATE ONLY type::record('canonical_study_readiness_cursors',$study) SET revision+=1dec,verify_after=$after;
    } ELSE IF $mode='notify' {
        UPDATE ONLY type::record('canonical_study_readiness_cursors',$study) SET revision+=1dec,notify_after=$after;
    } ELSE { THROW 'PSE_REFUSAL:study_readiness_cursor_mode'; };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN true;
};

DEFINE FUNCTION fn::pse_study_v1::create($rpc_expiry: int, $row: object) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$row.key);
    IF (SELECT * FROM ONLY type::record('canonical_study_retirements',$row.key) FOR UPDATE)!=NONE { THROW 'PSE_REFUSAL:study_identity_explicitly_retired'; };
    fn::pse_history_v1::run($rpc_expiry,$row.run);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $row.key);
    LET $old = SELECT * FROM ONLY type::record('canonical_studies', $row.key);
    IF $old != NONE {
        IF $old.run != $row.run OR $old.revision != $row.revision OR $old.problem != $row.problem OR $old.metadata != $row.metadata OR $old.point_count != $row.point_count OR $old.dependency_count != $row.dependency_count OR $old.interpretation != $row.interpretation { THROW 'PSE_REFUSAL:study_identity_reused'; };
        LET $rpc_result = $old;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    fn::pse_execution_v1::result_creator($rpc_expiry,$row.run);
    fn::pse_execution_v1::touch($rpc_expiry,'study-association:'+$row.run);
    LET $run = SELECT * FROM ONLY type::record('canonical_runs', $row.run);
    IF $run = NONE OR $run.problem != $row.problem OR $run.revision != $row.revision { THROW 'PSE_REFUSAL:study_source_run_unavailable'; };
    fn::pse_history_v1::associate($rpc_expiry,$row.key,$row.run,NONE);
    LET $rpc_result = CREATE ONLY type::record('canonical_studies', $row.key) CONTENT $row;
    fn::pse_study_v1::readiness_cursor($rpc_expiry,$row.key,true);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::append($rpc_expiry: int, $study: string, $points: array<object>) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study) FOR UPDATE;
    IF $s = NONE OR array::len($points) = 0 OR array::len($points) > 64 { THROW 'PSE_REFUSAL:study_occurrence_batch_unavailable'; };
    LET $existing = $points.map(|$point| SELECT key FROM ONLY type::record('canonical_study_points', $point.key)).filter(|$row| $row != NONE);
    IF array::len($existing) != 0 AND array::len($existing) != array::len($points) { THROW 'PSE_REFUSAL:study_occurrence_batch_mixed_settlement'; };
    FOR $point IN $points {
        fn::pse_history_v1::point($rpc_expiry,$point.key);
        fn::pse_history_v1::run($rpc_expiry,$point.run);
        IF $point.study != $study { THROW 'PSE_REFUSAL:study_occurrence_belongs_elsewhere'; };
        IF $point.seed_solution!=NONE {
            LET $input=SELECT * FROM ONLY type::record('canonical_study_inputs',$study+':'+$point.seed_solution);
            IF $input=NONE OR $input.study!=$study OR $input.solution!=$point.seed_solution { THROW 'PSE_REFUSAL:study_external_seed_was_not_retained'; };
            fn::pse_artifact_v1::available($rpc_expiry,$input.artifact);
            LET $owners=SELECT key FROM canonical_artifact_obligations WHERE artifact=$input.artifact AND owner_kind='study' AND owner=$study LIMIT 1;
            IF array::len($owners)!=1 { THROW 'PSE_REFUSAL:study_external_seed_obligation_absent'; };
        };
        LET $old = SELECT * FROM ONLY type::record('canonical_study_points', $point.key);
        IF $old != NONE {
            IF $old.seed_solution != $point.seed_solution OR $old.ordinal != $point.ordinal OR $old.occurrence != $point.occurrence OR $old.policy != $point.policy OR $old.descriptor != $point.descriptor OR $old.run != $point.run { THROW 'PSE_REFUSAL:study_immutable_occurrence_changed'; };
        } ELSE {
            fn::pse_execution_v1::result_creator($rpc_expiry,$point.run);
            fn::pse_execution_v1::touch($rpc_expiry,'study-association:'+$point.run);
            SELECT * FROM ONLY type::record('canonical_study_points',$point.key) FOR UPDATE;
            LET $run = SELECT * FROM ONLY type::record('canonical_runs', $point.run);
            LET $admission_run = SELECT * FROM ONLY type::record('canonical_runs', 'study-admission:'+$point.key);
            IF $run=NONE {
                IF $admission_run=NONE OR $admission_run.problem!=$s.problem OR $admission_run.revision!=$s.revision { THROW 'PSE_REFUSAL:study_lazy_admission_source_run_unavailable'; };
                LET $locator='admission:'+$point.key;
                LET $issuing=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store;
                CREATE ONLY type::record('canonical_history_locators',$locator) SET key=$locator,study=$study,run=$admission_run.key,point=$point.key,issuing_store=$issuing;
            } ELSE IF $run.problem != $s.problem OR $run.revision != $s.revision { THROW 'PSE_REFUSAL:study_occurrence_source_run_unavailable'; };
            fn::pse_history_v1::associate($rpc_expiry,$study,$point.run,$point.key);
        };
    };
    IF array::len($existing) = 0 {
        IF $s.active OR $s.cancelled OR array::len(array::distinct($points.ordinal)) != array::len($points) OR array::min($points.ordinal) != $s.next_ordinal OR array::max($points.ordinal) + 1dec != $s.next_ordinal + array::len($points) OR $s.next_ordinal + array::len($points) > $s.point_count { THROW 'PSE_REFUSAL:study_membership_closed_or_sequence_gap'; };
        LET $new = $points.map(|$point| object::extend($point, {id:type::record('canonical_study_points', $point.key)}));
        INSERT INTO canonical_study_points $new RETURN NONE;
        FOR $point IN $points { fn::pse_study_v1::readiness_invalidate($rpc_expiry,$point,true); };
        UPDATE ONLY type::record('canonical_studies', $study) SET next_ordinal += <decimal>array::len($points);
    };
    LET $rpc_result = SELECT * FROM ONLY type::record('canonical_studies', $study) FOR UPDATE;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::append_dependencies($rpc_expiry:int,$study:string,$edges:array<object>) -> bool {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::register($rpc_expiry,'study-cancel:'+$study);
    LET $s=SELECT * FROM ONLY type::record('canonical_studies',$study) FOR UPDATE;
    IF $s=NONE OR $s.next_ordinal!=$s.point_count OR array::len($edges)=0 OR array::len($edges)>64 OR array::len(array::distinct($edges.key))!=array::len($edges) { THROW 'PSE_REFUSAL:study_dependency_installation_unavailable'; };
    FOR $edge IN $edges {
        IF $edge.study!=$study { THROW 'PSE_REFUSAL:study_dependency_belongs_elsewhere'; };
        LET $old=SELECT * FROM ONLY type::record('canonical_study_dependencies',$edge.key);
        IF $old!=NONE {
            IF object::remove($old,['id'])!=$edge { THROW 'PSE_REFUSAL:study_immutable_dependency_changed'; };
        } ELSE {
            IF $s.active OR $s.cancelled { THROW 'PSE_REFUSAL:study_dependency_installation_closed'; };
            LET $dependent=SELECT * FROM ONLY type::record('canonical_study_points',$edge.dependent);
            LET $predecessor=SELECT study FROM ONLY type::record('canonical_study_points',$edge.predecessor);
            IF $dependent=NONE OR $predecessor=NONE OR $dependent.study!=$study OR $predecessor.study!=$study { THROW 'PSE_REFUSAL:study_dependency_endpoint_unavailable'; };
            INSERT RELATION INTO canonical_study_dependencies object::extend($edge,{id:type::record('canonical_study_dependencies',$edge.key),in:type::record('canonical_study_points',$edge.dependent),out:type::record('canonical_study_points',$edge.predecessor)}) RETURN NONE;
            UPDATE ONLY type::record('canonical_studies',$study) SET installed_dependencies+=1dec;
            fn::pse_study_v1::readiness_invalidate($rpc_expiry,$dependent,true);
        };
    };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN true;
};

DEFINE FUNCTION fn::pse_study_v1::activate($rpc_expiry: int, $study: string) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study) FOR UPDATE;
    IF $s = NONE OR $s.cancelled OR $s.point_count = 0dec OR $s.next_ordinal != $s.point_count OR $s.installed_dependencies!=$s.dependency_count { THROW 'PSE_REFUSAL:study_membership_incomplete'; };
    LET $rpc_result = UPDATE ONLY type::record('canonical_studies', $study) SET active = true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::scope($rpc_expiry: int, $point: string) -> object {
    fn::pse_history_v1::point($rpc_expiry,$point);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $p = SELECT key, study, ordinal, occurrence, policy, facts, outcome, run, revision, assigned, settled, attempt, start FROM ONLY type::record('canonical_study_points', $point);
    IF $p = NONE { THROW 'PSE_REFUSAL:study_candidate_unavailable'; };
    LET $s = SELECT key, generation, cancelled, active, terminal FROM ONLY type::record('canonical_studies', $p.study);
    IF $s = NONE OR $s.active = false { THROW 'PSE_REFUSAL:study_not_active'; };
    LET $edges = type::record('canonical_study_points',$point)->(SELECT predecessor FROM canonical_study_dependencies LIMIT 193);
    IF array::len(array::distinct($edges.predecessor)) > 64 { THROW 'PSE_REFUSAL:study_candidate_immediate_dependency_bound'; };
    LET $keys = array::distinct($edges.predecessor);
    LET $predecessors = $keys.map(|$key| SELECT key, study, ordinal, occurrence, policy, facts, outcome, revision, assigned, settled, attempt, run, start FROM ONLY type::record('canonical_study_points', $key));
    LET $rpc_result = {study:$s, point:$p, predecessors:$predecessors};
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::guard($rpc_expiry: int, $study: string, $generation: decimal, $cancelled: bool, $point: string, $expected: array<object>) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study) FOR UPDATE;
    IF $s = NONE OR $s.active = false OR $s.terminal OR $s.generation != $generation OR $s.cancelled != $cancelled { THROW 'PSE_REFUSAL:study_decision_generation_changed'; };
    LET $p = SELECT * FROM ONLY type::record('canonical_study_points', $point);
    IF $p = NONE OR $p.study != $study OR $p.assigned OR $p.settled { THROW 'PSE_REFUSAL:study_candidate_not_available'; };
    LET $edges = type::record('canonical_study_points',$point)->(SELECT predecessor FROM canonical_study_dependencies LIMIT 193);
    IF array::len(array::distinct($edges.predecessor)) > 64 { THROW 'PSE_REFUSAL:study_dependency_bound'; };
    LET $required = array::sort(array::distinct(array::concat([$point], $edges.predecessor)));
    IF array::sort($expected.key) != $required { THROW 'PSE_REFUSAL:study_decision_omitted_or_added_a_premise'; };
    FOR $item IN $expected {
        fn::pse_execution_v1::register($rpc_expiry, 'study-point:' + $item.key);
        SELECT * FROM ONLY type::record('canonical_study_points',$item.key) FOR UPDATE;
        LET $current = SELECT * FROM ONLY type::record('canonical_study_points', $item.key);
        IF $current = NONE OR $current.study != $study OR $current.revision != $item.revision { THROW 'PSE_REFUSAL:study_decision_premise_changed'; };
    };
    LET $rpc_result = $p;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::settle($rpc_expiry: int, $study: string, $generation: decimal, $cancelled: bool, $point: string, $expected: array<object>, $facts: bytes, $outcome: bytes) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $p = fn::pse_study_v1::guard($rpc_expiry, $study,$generation,$cancelled,$point,$expected);
    LET $rpc_result = UPDATE ONLY type::record('canonical_study_points',$point) SET revision += 1dec, facts=$facts,outcome=$outcome,settled=true;
    IF $rpc_result.settled AND !$rpc_result.assigned AND $rpc_result.attempt=NONE {
        LET $run=SELECT key FROM ONLY type::record('canonical_runs',$rpc_result.run);
        IF $run=NONE {
            LET $input=SELECT key FROM ONLY type::record('canonical_runs','study-admission:'+$point);
            IF $input=NONE { THROW 'PSE_REFUSAL:unadmitted_occurrence_input_owner_unavailable'; };
            fn::pse_history_v1::identity($rpc_expiry,'effect-free-run',$rpc_result.run,$rpc_result.run);
        } ELSE { fn::pse_history_v1::effect_free($rpc_expiry,$rpc_result); };
    };
    fn::pse_study_v1::readiness_invalidate($rpc_expiry,$rpc_result,false);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::started($rpc_expiry: int, $study: string, $point: string, $revision: decimal, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes, $facts: bytes) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies',$study) FOR UPDATE;
    IF $s = NONE OR $s.cancelled OR $s.terminal { THROW 'PSE_REFUSAL:study_cancelled_before_native_dispatch'; };
    LET $a = fn::pse_execution_v1::fence($rpc_expiry, $run,$attempt,$generation);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation,'study-start',$request);
    IF $op != NONE {
        IF $op.run != $run OR $op.attempt != $attempt { THROW 'PSE_REFUSAL:study_start_operation_reused'; };
        LET $rpc_result = SELECT * FROM ONLY type::record('canonical_study_points',$point);
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    LET $started = SELECT key FROM canonical_execution_operations WHERE run=$run AND kind='study-start' AND attempt=$attempt LIMIT 1;
    IF array::len($started) != 0 { THROW 'PSE_REFUSAL:study_native_start_already_admitted'; };
    fn::pse_execution_v1::touch($rpc_expiry, 'study-point:' + $point);
    LET $p = SELECT * FROM ONLY type::record('canonical_study_points',$point);
    IF $p = NONE OR $p.study != $study OR $p.run != $run OR $p.attempt != $attempt OR $p.revision != $revision OR $p.assigned = false OR $p.settled { THROW 'PSE_REFUSAL:study_native_start_premise_changed'; };
    LET $saved = UPDATE ONLY type::record('canonical_study_points',$point) SET revision += 1dec,facts=$facts;
    fn::pse_study_v1::readiness_invalidate($rpc_expiry,$saved,false);
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$attempt,'study-start',$request,$request,$study,$point);
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $a.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::finalize($rpc_expiry: int, $study: string, $operation: string, $request: bytes, $attempt: string, $worker: string, $lifetime: int) -> option<object> {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies',$study) FOR UPDATE;
    IF $s = NONE OR $s.active = false { THROW 'PSE_REFUSAL:study_unavailable'; };
    IF $s.terminal { LET $rpc_result = NONE; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $pending = SELECT key FROM canonical_study_points WHERE study=$study AND settled=false LIMIT 1;
    IF array::len($pending) != 0 { LET $rpc_result = NONE; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $run = SELECT * FROM ONLY type::record('canonical_runs',$s.run);
    IF $run.terminal_class = 'succeeded' { LET $rpc_result = NONE; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    IF $run.current_attempt != NONE {
        LET $current = SELECT * FROM ONLY type::record('canonical_attempts',$run.current_attempt);
        IF $current != NONE AND ($current.phase NOT IN ['terminal','terminal_compacted']) AND $current.expires_at > time::micros() { LET $rpc_result = NONE; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    };
    LET $existing_operation = SELECT key FROM ONLY type::record('canonical_execution_operations', $operation);
    LET $rpc_result = fn::pse_execution_v1::claim($rpc_expiry, $s.run,$operation,$request,$attempt,$worker,$lifetime,$s.interpretation,$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $existing_operation = NONE { fn::pse_execution_v1::lease($rpc_expiry, $rpc_result.expires_at); };
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::seal($rpc_expiry: int, $study: string, $run: string, $attempt: string, $authority: decimal, $operation: string, $request: bytes, $manifest: string, $digest: string, $outcome: string, $completion: bytes) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies',$study) FOR UPDATE;
    IF $s = NONE OR $s.active = false OR $s.run != $run { THROW 'PSE_REFUSAL:study_summary_association_unavailable'; };
    LET $pending = SELECT key FROM canonical_study_points WHERE study=$study AND settled=false LIMIT 1;
    IF array::len($pending) != 0 { THROW 'PSE_REFUSAL:study_summary_has_unsettled_scientific_occurrences'; };
    IF $s.cancelled AND $outcome != 'cancelled' { THROW 'PSE_REFUSAL:study_summary_cancellation_wins_admission'; };
    LET $existing_operation = SELECT key FROM ONLY type::record('canonical_execution_operations', $operation);
    LET $saved = fn::pse_execution_v1::seal($rpc_expiry, $run,$attempt,$authority,$operation,$request,$manifest,$digest,$outcome,$completion,false);
    UPDATE ONLY type::record('canonical_studies',$study) SET terminal=true;
    LET $rpc_result = $saved;
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $existing_operation = NONE AND $authority = $saved.generation { fn::pse_execution_v1::lease($rpc_expiry, $saved.expires_at); };
    RETURN $rpc_result;
};

// Shared mutation admission: exact original point/premises, cancellation, owner and lease.
DEFINE FUNCTION fn::pse_study_v1::admission_authority($rpc_expiry:int,$authority:object) -> object {
    IF $authority.run!='study-admission:'+$authority.point { THROW 'PSE_REFUSAL:scientific_admission_capability_belongs_elsewhere'; };
    LET $p=fn::pse_study_v1::guard($rpc_expiry,$authority.study,$authority.generation,false,$authority.point,$authority.expected);
    IF $p.admission!=NONE { THROW 'PSE_REFUSAL:scientific_admission_already_settled'; };
    fn::pse_execution_v1::current_writer($rpc_expiry,$authority.run);
    RETURN fn::pse_execution_v1::fence($rpc_expiry,$authority.run,$authority.attempt,$authority.authority);
};

// Input admission uses a separate run. Its lease never increments native point attempts.
DEFINE FUNCTION fn::pse_study_v1::admission_claim($rpc_expiry:int,$study:string,$generation:decimal,$point:string,$expected:array<object>,$operation:string,$request:bytes,$attempt:string,$worker:string,$lifetime:int) -> option<object> {
    LET $p=fn::pse_study_v1::guard($rpc_expiry,$study,$generation,false,$point,$expected);
    IF $p.admission!=NONE { RETURN NONE; };
    LET $run='study-admission:'+$point;
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    LET $r=SELECT * FROM ONLY type::record('canonical_runs',$run) FOR UPDATE;
    IF $r=NONE { THROW 'PSE_REFUSAL:scientific_admission_run_unavailable'; };
    IF $r.current_attempt!=NONE {
        LET $old=SELECT * FROM ONLY type::record('canonical_attempts',$r.current_attempt) FOR UPDATE;
        IF $old!=NONE AND ($old.phase NOT IN ['terminal','terminal_compacted']) {
            IF $old.expires_at>time::micros() { RETURN NONE; };
            // This phase owns no native execution, callbacks, result ingestion or solver history.
            // Expiry fences the old source admission; actual published source receipts remain immutable.
            UPDATE ONLY type::record('canonical_attempts',$old.key) SET phase='terminal',outcome='failed';
            fn::pse_execution_v1::register($rpc_expiry,'retention:'+$r.problem);
            SELECT * FROM ONLY type::record('canonical_roots','active_attempt:'+$old.key) FOR UPDATE;
            DELETE ONLY type::record('canonical_roots','active_attempt:'+$old.key);
        };
    };
    LET $s=SELECT interpretation FROM ONLY type::record('canonical_studies',$study);
    LET $claimed=fn::pse_execution_v1::claim($rpc_expiry,$run,$operation,$request,$attempt,$worker,$lifetime,$s.interpretation,NONE);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry,$claimed.expires_at);
    RETURN $claimed;
};

DEFINE FUNCTION fn::pse_study_v1::admission_complete($rpc_expiry:int,$study:string,$generation:decimal,$point:string,$expected:array<object>,$run:string,$attempt:string,$authority:decimal,$operation:string,$request:bytes,$admission:bytes,$facts:bytes,$outcome:option<bytes>) -> object {
    fn::pse_history_v1::point($rpc_expiry,$point);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $op=fn::pse_execution_v1::operation($rpc_expiry,$operation,'study-admission',$request);
    LET $saved=SELECT * FROM ONLY type::record('canonical_study_points',$point);
    IF $saved.admission!=NONE {
        IF $saved.admission!=$admission OR $op=NONE { THROW 'PSE_REFUSAL:immutable_scientific_admission_differs'; };
        RETURN $saved;
    };
    IF $run!='study-admission:'+$point { THROW 'PSE_REFUSAL:scientific_admission_owner_differs'; };
    LET $p=fn::pse_study_v1::guard($rpc_expiry,$study,$generation,false,$point,$expected);
    LET $a=fn::pse_execution_v1::fence($rpc_expiry,$run,$attempt,$authority);
    IF $outcome=NONE {
        LET $science=SELECT key FROM ONLY type::record('canonical_runs',$p.run);
        IF $science=NONE { THROW 'PSE_REFUSAL:selected_scientific_source_run_unavailable'; };
    };
    LET $result=UPDATE ONLY type::record('canonical_study_points',$point) SET admission=$admission,facts=$facts,outcome=$outcome,settled=($outcome!=NONE),revision+=1dec;
    UPDATE ONLY type::record('canonical_attempts',$attempt) SET phase='terminal',outcome='succeeded',completion=$admission;
    UPDATE ONLY type::record('canonical_runs',$run) SET terminal_attempt=$attempt,terminal_class='succeeded';
    fn::pse_execution_v1::register($rpc_expiry,'retention:'+(SELECT problem FROM ONLY type::record('canonical_runs',$run)).problem);
    SELECT * FROM ONLY type::record('canonical_roots','active_attempt:'+$attempt) FOR UPDATE;
    DELETE ONLY type::record('canonical_roots','active_attempt:'+$attempt);
    IF $result.settled { fn::pse_history_v1::identity($rpc_expiry,'effect-free-run',$p.run,$p.run); };
    fn::pse_study_v1::readiness_invalidate($rpc_expiry,$result,false);
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$run,$attempt,'study-admission',$request,$admission,$study,$point);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry,$a.expires_at);
    RETURN $result;
};

DEFINE FUNCTION fn::pse_study_v1::claim($rpc_expiry: int, $study: string, $generation: decimal, $point: string, $expected: array<object>, $start: bytes, $operation: string, $request: bytes, $claim_request: bytes, $attempt: string, $worker: string, $lifetime: int, $facts: bytes) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $op = fn::pse_execution_v1::operation($rpc_expiry, $operation, 'study-claim', $request);
    IF $op != NONE { LET $rpc_result = SELECT * FROM ONLY type::record('canonical_attempts', $op.attempt); fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study) FOR UPDATE;
    IF $s = NONE OR $s.active = false OR $s.cancelled OR $s.terminal OR $s.generation != $generation { THROW 'PSE_REFUSAL:study_claim_cancellation_or_generation_changed'; };
    LET $p = SELECT * FROM ONLY type::record('canonical_study_points', $point);
    IF $p = NONE OR $p.study != $study OR $p.assigned OR $p.settled { THROW 'PSE_REFUSAL:study_candidate_not_claimable'; };
    LET $edges = type::record('canonical_study_points',$point)->(SELECT predecessor FROM canonical_study_dependencies LIMIT 193);
    IF array::len(array::distinct($edges.predecessor)) > 64 { THROW 'PSE_REFUSAL:study_dependency_bound'; };
    LET $required = array::sort(array::distinct(array::concat([$point], $edges.predecessor)));
    IF array::sort($expected.key) != $required { THROW 'PSE_REFUSAL:study_claim_omitted_or_added_a_premise'; };
    FOR $item IN $expected {
        fn::pse_execution_v1::register($rpc_expiry, 'study-point:' + $item.key);
        SELECT * FROM ONLY type::record('canonical_study_points',$item.key) FOR UPDATE;
        LET $current = SELECT * FROM ONLY type::record('canonical_study_points', $item.key);
        IF $current = NONE OR $current.study != $study OR $current.revision != $item.revision { THROW 'PSE_REFUSAL:study_point_premise_changed'; };
    };
    LET $claimed = fn::pse_execution_v1::claim($rpc_expiry, $p.run, $operation + ':attempt', $claim_request, $attempt, $worker, $lifetime, $s.interpretation,NONE);
    UPDATE ONLY type::record('canonical_study_points', $point) SET assigned = true, revision += 1dec, facts = $facts, start = $start, attempt = $attempt;
    fn::pse_study_v1::readiness_invalidate($rpc_expiry,(SELECT * FROM ONLY type::record('canonical_study_points',$point)),false);
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$p.run,$attempt,'study-claim',$request,$request,$study,$point);
    LET $rpc_result = $claimed;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $claimed.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::observe($rpc_expiry: int, $point: string, $revision: decimal, $attempt: option<string>, $facts: bytes, $outcome: bytes, $settled: bool) -> object {
    fn::pse_history_v1::point($rpc_expiry,$point);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study-point:' + $point);
    LET $p = SELECT * FROM ONLY type::record('canonical_study_points', $point);
    IF $p = NONE { THROW 'PSE_REFUSAL:study_observation_unavailable'; };
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $p.study);
    IF $p.revision != $revision {
        IF $p.outcome = $outcome AND $p.facts = $facts AND $p.settled = $settled { LET $rpc_result = $p; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
        THROW 'PSE_REFUSAL:study_observation_revision_changed';
    };
    IF $p.settled OR $p.attempt != $attempt { THROW 'PSE_REFUSAL:study_observation_attempt_changed_or_already_settled'; };
    IF $attempt != NONE {
        LET $a = SELECT * FROM ONLY type::record('canonical_attempts', $attempt);
        IF $a = NONE OR $a.run != $p.run OR ($a.phase NOT IN ['terminal','terminal_compacted']) { THROW 'PSE_REFUSAL:study_result_attempt_not_admitted'; };
    };
    LET $rpc_result = UPDATE ONLY type::record('canonical_study_points', $point) SET revision += 1dec, assigned = false, settled = $settled, facts = $facts, outcome = $outcome;
    IF $rpc_result.settled AND !$rpc_result.assigned AND $rpc_result.attempt=NONE {
        LET $run=SELECT key FROM ONLY type::record('canonical_runs',$rpc_result.run);
        IF $run=NONE {
            LET $input=SELECT key FROM ONLY type::record('canonical_runs','study-admission:'+$point);
            IF $input=NONE { THROW 'PSE_REFUSAL:unadmitted_occurrence_input_owner_unavailable'; };
            fn::pse_history_v1::identity($rpc_expiry,'effect-free-run',$rpc_result.run,$rpc_result.run);
        } ELSE { fn::pse_history_v1::effect_free($rpc_expiry,$rpc_result); };
    };
    fn::pse_study_v1::readiness_invalidate($rpc_expiry,$rpc_result,false);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::cancel($rpc_expiry: int, $study: string) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study) FOR UPDATE;
    IF $s = NONE OR $s.terminal { THROW 'PSE_REFUSAL:study_unavailable_or_already_concluded'; };
    IF $s.cancelled { LET $rpc_result = $s; fn::pse_execution_v1::deadline($rpc_expiry); RETURN $rpc_result; };
    LET $rpc_result = UPDATE ONLY type::record('canonical_studies', $study) SET cancelled = true, generation += 1dec;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::conclude($rpc_expiry: int, $study: string) -> object {
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study) FOR UPDATE;
    IF $s = NONE OR $s.active = false { THROW 'PSE_REFUSAL:study_unavailable'; };
    LET $pending = SELECT key FROM canonical_study_points WHERE study = $study AND settled = false LIMIT 1;
    IF array::len($pending) != 0 { THROW 'PSE_REFUSAL:study_occurrences_not_all_settled'; };
    LET $run = SELECT * FROM ONLY type::record('canonical_runs',$s.run);
    IF $run = NONE OR $run.terminal_attempt = NONE { THROW 'PSE_REFUSAL:study_result_manifest_not_admitted'; };
    LET $attempt=SELECT * FROM ONLY type::record('canonical_attempts',$run.terminal_attempt);
    IF $attempt=NONE OR ($attempt.phase NOT IN ['terminal','terminal_compacted']) OR $attempt.run!=$s.run OR $attempt.finalization_study!=$study OR $attempt.closed_manifest=NONE OR ($s.cancelled AND $attempt.outcome!='cancelled') { THROW 'PSE_REFUSAL:study_conclusion_requires_its_exact_admitted_summary'; };
    LET $rpc_result = UPDATE ONLY type::record('canonical_studies', $study) SET terminal = true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};
"#;
