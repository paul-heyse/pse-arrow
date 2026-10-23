// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded complete cohort capture shares the selector's one native snapshot.
pub(super) fn append(ddl: &mut String) {
    ddl.push_str(r#"
DEFINE FUNCTION fn::pse_workbench_v1::capture($rpc_expiry:int,$rows:array<object>,$maximum:int,$frozen:bool,$members:int) -> object {
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $more=array::len($rows)>$maximum;
    IF $frozen AND $more { THROW 'PSE_REFUSAL:study_workbench_narrow_selection'; };
    LET $selected=array::slice($rows,0,$maximum);
    FOR $s IN $selected { fn::pse_history_v1::check_owner($rpc_expiry,'study',$s.key); };
    LET $headers=$selected.map(|$s| {
        LET $run=SELECT terminal_attempt,terminal_class FROM ONLY type::record('canonical_runs',$s.run);
        RETURN object::extend($s,{result_attempt:$run.terminal_attempt,outcome:$run.terminal_class});
    });
    LET $total=math::sum($selected.point_count) ?? 0;
    IF $frozen AND (array::len($selected.filter(|$s|$s.active=false))!=0 OR $total>$members) { THROW 'PSE_REFUSAL:study_workbench_narrow_selection'; };
    LET $points=IF $frozen {
        LET $selected_keys=$selected.key;
        SELECT key,study,ordinal,occurrence,run,attempt,revision,settled,facts FROM canonical_study_points WHERE study IN $selected_keys ORDER BY study,ordinal LIMIT 129
    } ELSE {[]};
    IF $frozen AND (array::len($points)!=$total OR array::len($points)>$members) { THROW 'PSE_REFUSAL:study_workbench_narrow_selection'; };
    LET $result={headers:$headers,members:$points,has_more:$more};
    IF bytes::len(encoding::cbor::encode($result))>1048576 { THROW 'PSE_REFUSAL:study_workbench_narrow_selection'; };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $result;
};
"#);
}
