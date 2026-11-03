// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Relational acquisition/projection for the typed atomic capture operation.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(
        &FACTS
            .replace(
                "__COHORT_MEMBERS__",
                &pse_schema::catalog::study_workbench::STUDY_COHORT_MAX_MEMBERS.to_string(),
            )
            .replace(
                "__COHORT_SENTINEL__",
                &(pse_schema::catalog::study_workbench::STUDY_COHORT_MAX_MEMBERS + 1).to_string(),
            ),
    );
}

// This helper acquires one bounded native relational product. Identity, actual
// owner, witness registration, publication and receipt are owned by typed Rust.
const FACTS: &str = r#"
DEFINE FUNCTION fn::pse_workbench_v1::capture_facts($rpc_expiry:int,$key:string,$filter:object,$maximum:int,$members:int) -> object {
    IF $maximum<1 OR $maximum>64 OR $members<1 OR $members>__COHORT_MEMBERS__ { RETURN {refusal:'study_workbench_narrow_selection',headers:[],members:[],protections:[]}; };
    LET $rows=fn::pse_workbench_v1::select_rows($rpc_expiry,$filter,'',$maximum+1);
    IF array::len($rows)>$maximum { RETURN {refusal:'study_workbench_narrow_selection',headers:[],members:[],protections:[]}; };
    FOR $s IN $rows { fn::pse_execution_v1::register($rpc_expiry,'history:study:'+$s.key); };
    LET $retired=$rows.filter(|$s| (SELECT * FROM ONLY type::record('canonical_history_retirements','study:'+$s.key) FOR UPDATE)!=NONE);
    IF array::len($retired)!=0 { RETURN {refusal:'history_explicitly_retired_detailed_response_unavailable',headers:[],members:[],protections:[]}; };
    LET $total=math::sum($rows.point_count) ?? 0dec;
    IF array::len($rows.filter(|$s| !$s.active))!=0 OR $total>$members { RETURN {refusal:'study_workbench_narrow_selection',headers:[],members:[],protections:[]}; };
    LET $selected_headers=$rows.map(|$s,$index| {
        LET $run=SELECT problem,revision,terminal_attempt,terminal_class FROM ONLY type::record('canonical_runs',$s.run);
        LET $attempt=IF $run.terminal_attempt=NONE {NONE} ELSE {SELECT key,run,phase,closed_manifest FROM ONLY type::record('canonical_attempts',$run.terminal_attempt)};
        LET $manifest=IF $attempt=NONE OR $attempt.run!=$s.run OR $attempt.phase NOT IN ['terminal','terminal_compacted'] OR $attempt.closed_manifest=NONE {NONE} ELSE {SELECT key,attempt FROM ONLY type::record('canonical_result_manifests',$attempt.closed_manifest)};
        LET $available=$manifest!=NONE AND $manifest.attempt=$run.terminal_attempt AND (SELECT key FROM ONLY type::record('canonical_result_retirements',$s.run))=NONE;
        LET $header=object::extend($s,{result_attempt:$run.terminal_attempt,outcome:$run.terminal_class});
        LET $protection=IF !$available {NONE} ELSE {{id:type::record('canonical_study_capture_protections',fn::pse_workbench_v1::protection_key($key,$s.key,$attempt.key)),key:fn::pse_workbench_v1::protection_key($key,$s.key,$attempt.key),capture:$key,study:$s.key,run:$s.run,attempt:$attempt.key,manifest:$manifest.key,problem:$run.problem,revision:$run.revision}};
        RETURN {row:{id:type::record('canonical_study_capture_headers',$key+':'+<string>$index),key:$key+':'+<string>$index,capture:$key,ordinal:<decimal>$index,body:encoding::cbor::encode($header)},protection:$protection};
    });
    LET $headers=$selected_headers.row;
    LET $selected=$rows.key;
    LET $points=SELECT key,study,ordinal,occurrence,run,attempt,revision,settled,facts FROM canonical_study_points WHERE study IN $selected ORDER BY study,ordinal LIMIT __COHORT_SENTINEL__;
    IF array::len($points)!=$total { RETURN {refusal:'study_workbench_narrow_selection',headers:[],members:[],protections:[]}; };
    LET $facts=$points.map(|$point,$index| {
        LET $attempt=IF $point.attempt=NONE {NONE} ELSE {SELECT key,run,phase,closed_manifest FROM ONLY type::record('canonical_attempts',$point.attempt)};
        LET $manifest=IF $attempt=NONE OR $attempt.run!=$point.run OR $attempt.phase NOT IN ['terminal','terminal_compacted'] OR $attempt.closed_manifest=NONE {NONE} ELSE {SELECT key,attempt FROM ONLY type::record('canonical_result_manifests',$attempt.closed_manifest)};
        LET $available=$manifest!=NONE AND $manifest.attempt=$point.attempt AND (SELECT key FROM ONLY type::record('canonical_result_retirements',$point.run))=NONE;
        LET $member=object::extend($point,{manifest:IF $available {$manifest.key} ELSE {NONE}});
        RETURN {id:type::record('canonical_study_capture_members',$key+':'+<string>$index),key:$key+':'+<string>$index,capture:$key,ordinal:<decimal>$index,body:encoding::cbor::encode($member),study:$point.study,run:$point.run,attempt:$point.attempt,manifest:IF $available {$manifest.key} ELSE {NONE}};
    });
    LET $member_protections=$facts.filter(|$fact| $fact.manifest!=NONE).map(|$fact| {
        LET $run=SELECT problem,revision FROM ONLY type::record('canonical_runs',$fact.run);
        RETURN {id:type::record('canonical_study_capture_protections',fn::pse_workbench_v1::protection_key($key,$fact.study,$fact.attempt)),key:fn::pse_workbench_v1::protection_key($key,$fact.study,$fact.attempt),capture:$key,study:$fact.study,run:$fact.run,attempt:$fact.attempt,manifest:$fact.manifest,problem:$run.problem,revision:$run.revision};
    });
    LET $header_protections=$selected_headers.map(|$header| $header.protection).filter(|$protection| $protection!=NONE);
    LET $protections=array::distinct(array::concat($member_protections,$header_protections));
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {refusal:NONE,headers:$headers,members:$facts,protections:$protections};
} PERMISSIONS NONE;
"#;
