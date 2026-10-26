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
    fn::pse_workbench_v1::publication($rpc_expiry);
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
    fn::pse_workbench_v1::publication($rpc_expiry);
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

DEFINE FUNCTION fn::pse_study_v1::readiness_notify_advance($rpc_expiry:int,$study:string,$point:string,$revision:decimal,$before:string,$after:string,$done:bool) -> bool {
    fn::pse_workbench_v1::publication($rpc_expiry);
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
    fn::pse_workbench_v1::publication($rpc_expiry);
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

// Scope pages retain one native row per distinct premise. A candidate is admitted
// whole or left before the continuation; dependency lists are never truncated.
DEFINE FUNCTION fn::pse_study_v1::readiness_page_item($rpc_expiry:int,$study:string,$acc:object,$entry:object,$maximum:int) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    IF $acc.stopped OR $acc.refused { RETURN $acc; };
    IF $entry.position>=$maximum { RETURN object::extend($acc,{stopped:true}); };
    IF $entry.point IN $acc.items.point { RETURN object::extend($acc,{next_edge:$entry.edge,next_ordinal:$entry.ordinal}); };
    LET $point=$entry.point;
    LET $edges=type::record('canonical_study_points',$point)->(SELECT predecessor FROM canonical_study_dependencies LIMIT 193);
    LET $keys=array::distinct($edges.predecessor);
    IF array::len($keys)>64 { RETURN object::extend($acc,{refused:true}); };
    LET $required=array::distinct(array::concat([$point],$keys));
    LET $missing=$required.filter(|$key| $key NOT IN $acc.rows.key);
    LET $new=$missing.map(|$key| SELECT key,study,ordinal,occurrence,policy,facts,revision,assigned,settled FROM ONLY type::record('canonical_study_points',$key));
    IF array::len($new.filter(|$row| $row=NONE OR $row.study!=$study))!=0 { RETURN object::extend($acc,{refused:true}); };
    LET $rows=array::concat($acc.rows,$new);
    LET $items=array::append($acc.items,{point:$point,predecessors:$keys,ordinal:$entry.ordinal,edge:$entry.edge});
    LET $next={rows:$rows,items:$items,stopped:false,refused:false,next_edge:$entry.edge,next_ordinal:$entry.ordinal};
    // CBOR is used as a conservative application extent, independently of the
    // 4 MiB transport guard. 128 native rows also cap maps, keys and nesting.
    IF array::len($rows)>128 OR bytes::len(encoding::cbor::encode($next))>1048576 {
        IF array::len($acc.items)=0 { RETURN object::extend($acc,{refused:true}); };
        RETURN object::extend($acc,{stopped:true});
    };
    RETURN $next;
};

DEFINE FUNCTION fn::pse_study_v1::readiness_scope_page($rpc_expiry:int,$study:string,$cursor:decimal,$mode:string,$after:option<decimal>,$source:option<object>,$maximum:int) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $maximum<1 OR $maximum>64 { THROW 'PSE_REFUSAL:study_readiness_page_extent'; };
    LET $s=SELECT key,run,generation,cancelled,active,terminal FROM ONLY type::record('canonical_studies',$study);
    IF $s=NONE OR $s.active=false { THROW 'PSE_REFUSAL:study_not_active'; };
    LET $entries=IF $mode='notify' {
        LET $hint=SELECT * FROM ONLY type::record('canonical_study_readiness',$source.point);
        IF $hint=NONE OR $hint.study!=$study OR $hint.notify=false OR $hint.notify_revision!=$source.revision OR $hint.notify_after!=$source.before { THROW 'PSE_REFUSAL:study_readiness_notification_changed'; };
        LET $edges=type::record('canonical_study_points',$source.point)<-(SELECT key,dependent FROM canonical_study_dependencies WHERE key>$source.before ORDER BY key LIMIT 65);
        LET $children=$edges.map(|$edge,$index| {point:$edge.dependent,edge:$edge.key,ordinal:NONE,position:$index});
        IF $hint.dirty {array::prepend($children,{point:$source.point,edge:NONE,ordinal:NONE,position:-1})} ELSE {$children}
    } ELSE IF $mode IN ['rebuild','verify'] {
        LET $points=SELECT key,ordinal FROM canonical_study_points WHERE study=$study AND ordinal>($after ?? -1dec) ORDER BY ordinal LIMIT 65;
        $points.map(|$point,$index| {point:$point.key,ordinal:$point.ordinal,edge:NONE,position:$index})
    } ELSE { THROW 'PSE_REFUSAL:study_readiness_cursor_mode'; };
    // Position bounds outgoing edges even when relation kinds repeat a child.
    LET $page=$entries.fold({rows:[],items:[],stopped:false,refused:false,next_edge:$source.before,next_ordinal:$after},|$acc,$entry| fn::pse_study_v1::readiness_page_item($rpc_expiry,$study,$acc,$entry,$maximum));
    IF $page.refused { THROW 'PSE_REFUSAL:study_readiness_single_scope_extent'; };
    LET $exhausted=$page.stopped=false;
    LET $fence={cursor:$cursor,mode:$mode,after:$after,source:$source,next_ordinal:$page.next_ordinal,next_edge:$page.next_edge,exhausted:$exhausted};
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {study:$s,rows:$page.rows,items:$page.items,fence:$fence};
};

// One pure guest invocation per deduplicated native page. Scientific policy is
// the same Rust owner used by claims; no row/fact snapshot crosses the RPC.
DEFINE FUNCTION fn::pse_study_v1::readiness_policy($page:object) -> object {
    RETURN mod::pse::study::readiness({generation:$page.study.generation,cancelled:$page.study.cancelled,
        rows:$page.rows.map(|$row| {key:$row.key,occurrence:$row.occurrence,revision:$row.revision,policy:$row.policy,facts:$row.facts,assigned:$row.assigned,settled:$row.settled}),
        items:$page.items.map(|$item| {point:$item.point,predecessors:$item.predecessors})});
};

// Page acquisition and policy evaluation already share this exact native SSI
// snapshot. Only the destination hint needs a write lock; scientific rows and
// dependency edges are neither fetched nor decoded again per candidate.
DEFINE FUNCTION fn::pse_study_v1::readiness_apply_hint($rpc_expiry:int,$point:object,$item:object,$mutating:bool) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $hint=SELECT * FROM ONLY type::record('canonical_study_readiness',$point.key) FOR UPDATE;
    LET $eligible=$item.eligible AND $point.assigned=false AND $point.settled=false;
    LET $changed=$hint=NONE OR $hint.source_revision!=$point.revision OR $hint.pending!=$item.pending OR $hint.eligible!=$eligible OR $hint.premises!=$item.premises OR $hint.dirty;
    IF $changed {
        IF $mutating=false { RETURN {point:$point.key,status:'stale',changed:false}; };
        IF $hint=NONE OR $hint.source_revision!=$point.revision {
            UPSERT ONLY type::record('canonical_study_readiness',$point.key) CONTENT {
                key:$point.key,point:$point.key,study:$point.study,
                ordinal:$point.ordinal,
                source_revision:$point.revision,pending:$item.pending,eligible:$eligible,premises:$item.premises,
                dirty:false,notify:true,notify_revision:$point.revision,notify_after:''
            };
        } ELSE {
            UPDATE ONLY type::record('canonical_study_readiness',$point.key) SET pending=$item.pending,eligible=$eligible,premises=$item.premises,dirty=false;
        };
    };
    RETURN {point:$point.key,status:'accepted',changed:$changed};
};

DEFINE FUNCTION fn::pse_study_v1::readiness_settlement($rpc_expiry:int,$operation:string,$study:string,$run:string) -> option<object> {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $old=fn::pse_history_v1::study_repair_operation($rpc_expiry,$operation,$study,$run,NONE);
    RETURN IF $old=NONE OR $old.disposition NOT IN ['settled','no-effect','refused'] {NONE} ELSE IF $old.disposition='no-effect' {{no_effect:true}} ELSE IF $old.disposition='refused' {{refusal:true}} ELSE {encoding::cbor::decode($old.result)};
};

// The pending locator, exact original request and history obligation are born
// in one transaction. Losing preparation's acknowledgment cannot orphan an ID.
DEFINE FUNCTION fn::pse_study_v1::readiness_pending_page($rpc_expiry:int,$study:string) -> option<object> {
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    LET $cursor=SELECT * FROM ONLY type::record('canonical_study_readiness_cursors',$study);
    IF $cursor.pending_operation=NONE { RETURN NONE; };
    LET $s=SELECT run FROM ONLY type::record('canonical_studies',$study);
    LET $old=fn::pse_history_v1::study_repair_operation($rpc_expiry,$cursor.pending_operation,$study,$s.run,NONE);
    IF $old=NONE OR $old.disposition IN ['settled','no-effect','refused'] { THROW 'PSE_REFUSAL:study_readiness_pending_operation_unavailable'; };
    LET $request=encoding::cbor::decode($old.request);
    RETURN {operation:$old.key,study:$study,run:$old.run,epoch:$old.disposition,keys:$request.keys,created:false};
};

DEFINE FUNCTION fn::pse_study_v1::readiness_prepare_page($rpc_expiry:int,$operation:string,$coordinates:object) -> option<object> {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $study=$coordinates.study;
    LET $module=(SELECT study_module_sha256 FROM ONLY canonical_interpretations:current).study_module_sha256;
    IF $coordinates.module_sha256!=$module {THROW 'PSE_REFUSAL:study_readiness_module_identity_changed';};
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    LET $cursor=SELECT * FROM ONLY type::record('canonical_study_readiness_cursors',$study) FOR UPDATE;
    IF $cursor=NONE { THROW 'PSE_REFUSAL:study_readiness_owner_unavailable'; };
    IF $cursor.pending_operation!=NONE { RETURN fn::pse_study_v1::readiness_pending_page($rpc_expiry,$study); };
    IF $cursor.revision!=$coordinates.cursor { RETURN NONE; };
    LET $s=SELECT key,run,generation,cancelled,active,terminal FROM ONLY type::record('canonical_studies',$study);
    IF $s!=NONE AND ($s.active=false OR $s.terminal) { RETURN NONE; };
    IF $s=NONE { THROW 'PSE_REFUSAL:study_readiness_owner_unavailable'; };
    LET $page=fn::pse_study_v1::readiness_scope_page($rpc_expiry,$study,$coordinates.cursor,$coordinates.mode,$coordinates.after,$coordinates.source,64);
    // Verification stays entirely nonmutating when the authoritative guest
    // result already matches every hint. It creates no durable operation ID.
    IF $coordinates.mode='verify' {
        LET $reply=fn::pse_study_v1::readiness_policy($page);
        IF $reply.accepted=false { THROW 'PSE_REFUSAL:study_readiness_policy_refused'; };
        LET $check={study:$study,generation:$s.generation,cancelled:$s.cancelled,fence:$page.fence,mutating:false};
        LET $result=fn::pse_study_v1::readiness_apply_decisions($rpc_expiry,$check,$page,$reply.decisions,true);
        IF array::len($result.outcomes.filter(|$item| $item.status!='accepted'))=0 {
            RETURN {settled:true,result:$result,keys:$page.items.point};
        };
    };
    // Persist only traversal coordinates and immutable candidate identities.
    // Facts, premises and policy decisions are reacquired inside each apply.
    LET $request={study:$study,run:$s.run,module_sha256:$module,fence:$page.fence,keys:$page.items.point};
    LET $old=fn::pse_history_v1::study_repair_operation($rpc_expiry,$operation,$study,$s.run,encoding::cbor::encode($request));
    IF $old!=NONE { THROW 'PSE_REFUSAL:study_readiness_pending_operation_unavailable'; };
    fn::pse_history_v1::retain($rpc_expiry,$operation,'run',$s.run,'unsettled','study-readiness-repair-page');
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$s.run,NONE,'study-readiness-repair-page',encoding::cbor::encode($request),encoding::cbor::encode({outcomes:[]}),$study,NONE);
    LET $epoch='pending:'+$operation;
    UPDATE ONLY type::record('canonical_execution_operations',$operation) SET disposition=$epoch;
    UPDATE ONLY type::record('canonical_study_readiness_cursors',$study) SET pending_operation=$operation;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {operation:$operation,study:$study,run:$s.run,epoch:$epoch,keys:$request.keys,created:true};
};

// Writing the SAME row that every apply must read and write is an authenticated
// transaction fence, not an inference from a missing receipt or a dead client.
// A delayed old dispatch mismatches its epoch; an in-flight old transaction
// cannot commit its receipt after this write wins under native SSI.
DEFINE FUNCTION fn::pse_study_v1::readiness_fence_page($rpc_expiry:int,$operation:string,$study:string,$run:string,$before:string,$after:string) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $old=fn::pse_history_v1::study_repair_operation($rpc_expiry,$operation,$study,$run,NONE);
    IF $old=NONE { THROW 'PSE_REFUSAL:study_readiness_pending_operation_unavailable'; };
    IF $old.disposition='no-effect' { RETURN {settled:true,result:{no_effect:true}}; };
    IF $old.disposition='refused' { RETURN {settled:true,result:{refusal:true}}; };
    IF $old.disposition='settled' { RETURN {settled:true,result:encoding::cbor::decode($old.result)}; };
    IF $old.disposition=$before { UPDATE ONLY type::record('canonical_execution_operations',$operation) SET disposition=$after; };
    LET $current=SELECT disposition FROM ONLY type::record('canonical_execution_operations',$operation);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {settled:false,epoch:$current.disposition};
};

DEFINE FUNCTION fn::pse_study_v1::readiness_apply_decisions($rpc_expiry:int,$request:object,$page:object,$items:array<object>,$finish:bool) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $study=$request.study; LET $fence=$request.fence;
    fn::pse_execution_v1::register($rpc_expiry,'study-cancel:'+$study);
    // Validate the entire guest result before any hint/cursor write. An empty
    // or malformed later item cannot certify a prefix or commit earlier hints.
    LET $coverage=array::sort($items.point)=array::sort($page.items.point) AND array::len(array::distinct($items.point))=array::len($items);
    IF $coverage!=true { THROW 'PSE_REFUSAL:study_readiness_repair_page_extent'; };
    LET $invalid=$items.filter(|$item| {
        LET $members=($page.items.filter(|$member|$member.point=$item.point))[0];
        LET $point=($page.rows.filter(|$row|$row.key=$item.point))[0];
        LET $keys=array::concat([$item.point],$members.predecessors);
        LET $expected=$item.expected;
        LET $pending=<decimal>array::len($page.rows.filter(|$row| $row.key IN $members.predecessors AND $row.settled=false));
        LET $valid=$point!=NONE AND $point.study=$study AND array::sort($expected.key)=array::sort($keys)
            AND array::len($expected)=array::len($keys) AND type::is_decimal($item.pending) AND $item.pending=$pending
            AND type::is_bool($item.eligible) AND type::is_bytes($item.premises) AND bytes::len($item.premises)<=16384
            AND array::len($expected.filter(|$premise| type::is_decimal($premise.revision)=false OR array::len($page.rows.filter(|$row| $row.key=$premise.key AND $row.revision=$premise.revision AND $row.study=$study))!=1))=0;
        RETURN $valid!=true;
    });
    IF array::len($invalid)!=0 { THROW 'PSE_REFUSAL:study_readiness_repair_page_extent'; };
    LET $available=$page.study!=NONE AND $page.study.active AND $page.study.terminal=false;
    LET $outcomes=$items.map(|$item| {
        LET $point=($page.rows.filter(|$row|$row.key=$item.point))[0];
        IF $available=false {{point:$item.point,status:'stale',changed:false}}
        ELSE {fn::pse_study_v1::readiness_apply_hint($rpc_expiry,$point,$item,$request.mutating)}
    });
    LET $safe=$available AND $finish AND array::len($outcomes.filter(|$item|$item.status='refused' OR ($request.mutating=false AND $item.status='stale')))=0;
    LET $notified=IF $safe AND $fence.mode='notify' {
        fn::pse_study_v1::readiness_notify_advance($rpc_expiry,$study,$fence.source.point,$fence.source.revision,$fence.source.before,$fence.next_edge ?? $fence.source.before,$fence.exhausted)
    } ELSE {false};
    LET $advanced=IF $safe AND ($fence.mode!='notify' OR $notified) {
        fn::pse_study_v1::readiness_scan_advance($rpc_expiry,$study,$fence.cursor,$fence.mode,IF $fence.mode='notify' {$fence.source.ordinal} ELSE IF $fence.exhausted AND $fence.mode='verify' {NONE} ELSE {$fence.next_ordinal},$fence.exhausted)
    } ELSE {false};
    LET $cursor=SELECT * FROM ONLY type::record('canonical_study_readiness_cursors',$study);
    RETURN {outcomes:$outcomes,advanced:$advanced,notified:$notified,cursor:$cursor};
};

// Partial application shrinks a repeatedly conflicting transaction while keeping
// the same original operation, exact inputs and cursor fence. Committed prefixes
// are retained in its pending body; only a complete result becomes immutable.
DEFINE FUNCTION fn::pse_study_v1::readiness_apply_page($rpc_expiry:int,$operation:string,$study:string,$run:string,$epoch:string,$maximum:int) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $old=fn::pse_history_v1::study_repair_operation($rpc_expiry,$operation,$study,$run,NONE);
    IF $old=NONE OR $maximum<1 OR $maximum>65 { THROW 'PSE_REFUSAL:study_readiness_pending_operation_unavailable'; };
    IF $old.disposition='no-effect' { RETURN {settled:true,result:{no_effect:true}}; };
    IF $old.disposition='refused' { RETURN {settled:true,result:{refusal:true}}; };
    IF $old.disposition='settled' { RETURN {settled:true,result:encoding::cbor::decode($old.result)}; };
    IF $old.disposition!=$epoch { RETURN {settled:false,fenced:true}; };
    LET $cursor=SELECT pending_operation FROM ONLY type::record('canonical_study_readiness_cursors',$study) FOR UPDATE;
    IF $cursor.pending_operation!=$operation { THROW 'PSE_REFUSAL:study_readiness_pending_operation_unavailable'; };
    LET $request=encoding::cbor::decode($old.request);
    IF $request.module_sha256!=(SELECT study_module_sha256 FROM ONLY canonical_interpretations:current).study_module_sha256 {THROW 'PSE_REFUSAL:study_readiness_module_identity_changed';};
    LET $previous=encoding::cbor::decode($old.result);
    LET $offset=array::len($previous.outcomes);
    LET $keys=array::slice($request.keys,$offset,$offset+$maximum);
    LET $entries=$keys.map(|$key,$index| {point:$key,ordinal:NONE,edge:NONE,position:$index});
    LET $page=$entries.fold({rows:[],items:[],stopped:false,refused:false,next_edge:NONE,next_ordinal:NONE},|$acc,$entry| fn::pse_study_v1::readiness_page_item($rpc_expiry,$study,$acc,$entry,65));
    LET $s=SELECT key,generation,cancelled,active,terminal FROM ONLY type::record('canonical_studies',$study);
    LET $reply=IF $page.refused OR $page.stopped {{accepted:false,decisions:[]}} ELSE {fn::pse_study_v1::readiness_policy(object::extend($page,{study:$s}))};
    LET $finish=$reply.accepted=false OR $offset+array::len($keys)=array::len($request.keys);
    // A prior logical refusal also prevents cursor certification at the end.
    LET $safe=array::len($previous.outcomes.filter(|$item|$item.status='refused'))=0;
    LET $inputs=object::extend($request,{generation:$s.generation,cancelled:$s.cancelled,mutating:true});
    LET $next=IF $reply.accepted {
        fn::pse_study_v1::readiness_apply_decisions($rpc_expiry,$inputs,object::extend($page,{study:$s}),$reply.decisions,$finish AND $safe)
    } ELSE {{outcomes:array::slice($request.keys,$offset,array::len($request.keys)).map(|$key| {point:$key,status:'refused',changed:false}),advanced:false,notified:false,refusal:true}};
    IF $finish {
        UPDATE ONLY type::record('canonical_study_readiness_cursors',$study) SET pending_operation=NONE,revision=math::max([(SELECT revision FROM ONLY type::record('canonical_study_readiness_cursors',$study)).revision,$request.fence.cursor+1dec]);
        fn::pse_history_v1::drop_obligation($rpc_expiry,$operation,'run',$run,'unsettled','study-readiness-repair-page');
    };
    LET $cursor=SELECT * FROM ONLY type::record('canonical_study_readiness_cursors',$study);
    LET $result=object::extend($next,{outcomes:array::concat($previous.outcomes,$next.outcomes),cursor:$cursor});
    LET $effect=$result.notified OR ($result.advanced AND $request.fence.mode='rebuild') OR array::len($result.outcomes.filter(|$item|$item.changed))>0;
    // An acknowledged transaction with no obligation effect still consumes the
    // exact epoch. Its tiny permanent attribution is not a full history receipt.
    IF $finish AND $effect=false {
        UPDATE ONLY type::record('canonical_execution_operations',$operation) SET request=type::bytes(''),result=type::bytes(''),minimal=true,disposition=IF $result.refusal=true {'refused'} ELSE {'no-effect'};
    } ELSE {
        UPDATE ONLY type::record('canonical_execution_operations',$operation) SET result=encoding::cbor::encode($result),disposition=IF $finish {'settled'} ELSE {$epoch};
    };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {settled:$finish,result:$result};
};

DEFINE FUNCTION fn::pse_study_v1::create($rpc_expiry: int, $row: object, $digest:string) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$row.key);
    IF (SELECT * FROM ONLY type::record('canonical_study_retirements',$row.key) FOR UPDATE)!=NONE { THROW 'PSE_REFUSAL:study_identity_explicitly_retired'; };
    fn::pse_history_v1::run($rpc_expiry,$row.run);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::touch($rpc_expiry, 'study:' + $row.key);
    LET $installation=SELECT * FROM ONLY type::record('canonical_study_installations',$row.key) FOR UPDATE;
    IF $installation!=NONE AND $installation.digest!=$digest { THROW 'PSE_REFUSAL:study_immutable_population_changed'; };
    IF $installation=NONE { CREATE ONLY type::record('canonical_study_installations',$row.key) SET key=$row.key,digest=$digest,generation=0dec,phase='nodes',node_cursor=0dec,edge_cursor=0dec; };
    LET $old = SELECT * FROM ONLY type::record('canonical_studies', $row.key);
    IF $old != NONE {
        IF $old.run != $row.run OR $old.revision != $row.revision OR $old.problem != $row.problem OR $old.metadata != $row.metadata OR $old.physical_revision != $row.physical_revision OR $old.physical_identity != $row.physical_identity OR $old.context != $row.context OR $old.point_count != $row.point_count OR $old.dependency_count != $row.dependency_count OR $old.interpretation != $row.interpretation { THROW 'PSE_REFUSAL:study_identity_reused'; };
        LET $rpc_result = $old;
        fn::pse_execution_v1::deadline($rpc_expiry);
        RETURN $rpc_result;
    };
    fn::pse_execution_v1::result_creator($rpc_expiry,$row.run);
    fn::pse_execution_v1::touch($rpc_expiry,'study-association:'+$row.run);
    LET $run = SELECT * FROM ONLY type::record('canonical_runs', $row.run);
    IF $run = NONE OR $run.problem != $row.problem OR $run.revision != $row.revision { THROW 'PSE_REFUSAL:study_source_run_unavailable'; };
    IF $row.physical_revision!=NONE AND array::len(SELECT key FROM canonical_run_sources WHERE run=$row.run AND revision=$row.physical_revision LIMIT 1)!=1 { THROW 'PSE_REFUSAL:study_source_run_unavailable'; };
    IF $row.context!=NONE {
        LET $context=SELECT revision,physical_revision,physical_source_identity FROM ONLY type::record('canonical_project_contexts',$row.context);
        IF $context=NONE OR $context.revision!=$row.revision OR $context.physical_revision!=$row.physical_revision OR $context.physical_source_identity!=$row.physical_identity { THROW 'PSE_REFUSAL:study_source_run_unavailable'; };
    };
    fn::pse_history_v1::associate($rpc_expiry,$row.key,$row.run,NONE);
    LET $rpc_result = CREATE ONLY type::record('canonical_studies', $row.key) CONTENT $row;
    fn::pse_study_v1::readiness_cursor($rpc_expiry,$row.key,true);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

// Issue the exact page once, before its effect transaction. A retry reads this
// original grant; it cannot replace an expired issued operation with a new clock.
DEFINE FUNCTION fn::pse_study_v1::issue_page($rpc_expiry:int,$study:string,$digest:string,$phase:string,$cursor:decimal,$request:bytes,$count:int) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $state=SELECT * FROM ONLY type::record('canonical_study_installations',$study) FOR UPDATE;
    IF $state=NONE OR $state.digest!=$digest OR $phase NOT IN ['nodes','edges'] { THROW 'PSE_REFUSAL:study_installation_identity_differs'; };
    LET $key=$study+':'+$phase+':'+<string>$cursor;
    LET $old=SELECT * FROM ONLY type::record('canonical_study_installation_pages',$key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.request!=$request OR $old.digest!=$digest OR $old.generation!=$state.generation OR $old.next_cursor!=$cursor+<decimal>$count { THROW 'PSE_REFUSAL:study_installation_page_reused'; };
        RETURN $old;
    };
    LET $frontier=IF $phase='nodes' {$state.node_cursor} ELSE {$state.edge_cursor};
    IF $cursor!=$frontier OR $count<1 OR $count>64 OR bytes::len($request)>4194304 { THROW 'PSE_REFUSAL:study_installation_page_extent'; };
    LET $receipt=CREATE ONLY type::record('canonical_study_installation_pages',$key) SET key=$key,study=$study,digest=$digest,generation=$state.generation,phase=$phase,cursor=$cursor,next_cursor=$cursor+<decimal>$count,request=$request,issued_expires_at=<decimal>$rpc_expiry,state='issued';
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $receipt;
};

// A page is one immutable native transaction and one exact receipt, independent
// from the invocation's total population. Replaying never recreates runs/roots.
DEFINE FUNCTION fn::pse_study_v1::install_page($rpc_expiry:int,$study:string,$digest:string,$phase:string,$cursor:decimal,$request:bytes,$points:array<object>,$runs:array<object>,$edges:array<object>) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $state=SELECT * FROM ONLY type::record('canonical_study_installations',$study) FOR UPDATE;
    IF $state=NONE OR $state.digest!=$digest OR $phase NOT IN ['nodes','edges'] { THROW 'PSE_REFUSAL:study_installation_identity_differs'; };
    LET $key=$study+':'+$phase+':'+<string>$cursor;
    LET $old=SELECT * FROM ONLY type::record('canonical_study_installation_pages',$key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.request!=$request OR $old.digest!=$digest OR $old.generation!=$state.generation { THROW 'PSE_REFUSAL:study_installation_page_reused'; };
        IF $old.state='committed' { RETURN $old; };
    };
    IF $old=NONE OR $old.state!='issued' OR $old.issued_expires_at<=time::micros() { THROW 'PSE_REFUSAL:study_installation_original_page_expired'; };
    LET $frontier=IF $phase='nodes' {$state.node_cursor} ELSE {$state.edge_cursor};
    IF $cursor!=$frontier OR ($phase='nodes' AND $state.phase!='nodes') OR ($phase='edges' AND $state.phase NOT IN ['nodes','edges']) { THROW 'PSE_REFUSAL:study_installation_frontier_changed'; };
    LET $count=IF $phase='nodes' {array::len($points)} ELSE {array::len($edges)};
    IF $count=0 OR $count>64 OR bytes::len($request)>4194304 { THROW 'PSE_REFUSAL:study_installation_page_extent'; };
    IF $phase='nodes' {
        IF array::len($runs)!=array::len($points) OR array::len($edges)!=0 { THROW 'PSE_REFUSAL:study_installation_page_extent'; };
        fn::pse_execution_v1::install_runs($rpc_expiry,$runs);
        LET $projections=$runs.filter(|$item| $item.projection!=NONE).map(|$item| {id:type::record('canonical_study_admission_outcomes',$item.point),key:$item.point,projection:$item.projection});
        IF array::len($projections)!=0 { INSERT INTO canonical_study_admission_outcomes $projections RETURN NONE; };
        fn::pse_study_v1::append($rpc_expiry,$study,$points);
        UPDATE ONLY type::record('canonical_study_installations',$study) SET node_cursor+=$count;
    } ELSE {
        IF array::len($runs)!=0 OR array::len($points)!=0 { THROW 'PSE_REFUSAL:study_installation_page_extent'; };
        fn::pse_study_v1::append_dependencies($rpc_expiry,$study,$edges);
        UPDATE ONLY type::record('canonical_study_installations',$study) SET phase='edges',edge_cursor+=$count;
    };
    IF $old.issued_expires_at<=time::micros() { THROW 'PSE_REFUSAL:study_installation_original_page_expired'; };
    LET $receipt=UPDATE ONLY type::record('canonical_study_installation_pages',$key) SET state='committed';
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $receipt;
};

DEFINE FUNCTION fn::pse_study_v1::append($rpc_expiry: int, $study: string, $points: array<object>) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
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
            IF $old.admission_run!=$point.admission_run OR $old.variant_problem!=$point.variant_problem OR $old.variant_operation!=$point.variant_operation OR $old.seed_solution != $point.seed_solution OR $old.ordinal != $point.ordinal OR $old.occurrence != $point.occurrence OR $old.policy != $point.policy OR $old.descriptor != $point.descriptor OR $old.run != $point.run { THROW 'PSE_REFUSAL:study_immutable_occurrence_changed'; };
        } ELSE {
            fn::pse_execution_v1::result_creator($rpc_expiry,$point.run);
            fn::pse_execution_v1::touch($rpc_expiry,'study-association:'+$point.run);
            SELECT * FROM ONLY type::record('canonical_study_points',$point.key) FOR UPDATE;
            LET $run = SELECT * FROM ONLY type::record('canonical_runs', $point.run);
            LET $admission_run = IF $point.admission_run=NONE {NONE} ELSE {SELECT * FROM ONLY type::record('canonical_runs', $point.admission_run)};
            IF $run=NONE {
                IF $admission_run=NONE OR $admission_run.purpose!='study_admission' OR $admission_run.admission_study!=$study OR $admission_run.admission_point!=$point.key OR $admission_run.problem!=$s.problem OR $admission_run.revision!=$s.revision { THROW 'PSE_REFUSAL:study_lazy_admission_source_run_unavailable'; };
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
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::register($rpc_expiry,'study-cancel:'+$study);
    LET $s=SELECT * FROM ONLY type::record('canonical_studies',$study) FOR UPDATE;
    IF $s=NONE OR $s.next_ordinal!=$s.point_count OR array::len($edges)=0 OR array::len($edges)>64 OR array::len(array::distinct($edges.key))!=array::len($edges) { THROW 'PSE_REFUSAL:study_dependency_installation_unavailable'; };
    LET $checked=$edges.map(|$edge| {
        IF $edge.study!=$study { THROW 'PSE_REFUSAL:study_dependency_belongs_elsewhere'; };
        LET $old=SELECT * FROM ONLY type::record('canonical_study_dependencies',$edge.key);
        IF $old!=NONE {
            IF object::remove($old,['id'])!=$edge { THROW 'PSE_REFUSAL:study_immutable_dependency_changed'; };
        } ELSE {
            IF $s.active OR $s.cancelled { THROW 'PSE_REFUSAL:study_dependency_installation_closed'; };
            LET $dependent=SELECT study FROM ONLY type::record('canonical_study_points',$edge.dependent);
            LET $predecessor=SELECT study FROM ONLY type::record('canonical_study_points',$edge.predecessor);
            IF $dependent=NONE OR $predecessor=NONE OR $dependent.study!=$study OR $predecessor.study!=$study { THROW 'PSE_REFUSAL:study_dependency_endpoint_unavailable'; };
        };
        RETURN object::extend($edge,{present:$old!=NONE});
    });
    LET $new=$checked.filter(|$edge| !$edge.present).map(|$edge| object::extend(object::remove($edge,'present'),{id:type::record('canonical_study_dependencies',$edge.key),in:type::record('canonical_study_points',$edge.dependent),out:type::record('canonical_study_points',$edge.predecessor)}));
    IF array::len($new)!=0 {
        INSERT RELATION INTO canonical_study_dependencies $new RETURN NONE;
        UPDATE ONLY type::record('canonical_studies',$study) SET installed_dependencies+=<decimal>array::len($new);
        FOR $point IN array::distinct($new.dependent) { fn::pse_study_v1::readiness_invalidate($rpc_expiry,(SELECT * FROM ONLY type::record('canonical_study_points',$point)),true); };
    };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN true;
};

DEFINE FUNCTION fn::pse_study_v1::activate($rpc_expiry: int, $study: string) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::register($rpc_expiry, 'study-cancel:' + $study);
    LET $s = SELECT * FROM ONLY type::record('canonical_studies', $study) FOR UPDATE;
    IF $s = NONE OR $s.cancelled OR $s.point_count = 0dec OR $s.next_ordinal != $s.point_count OR $s.installed_dependencies!=$s.dependency_count { THROW 'PSE_REFUSAL:study_membership_incomplete'; };
    UPDATE ONLY type::record('canonical_study_installations',$study) SET phase='active';
    LET $rpc_result = UPDATE ONLY type::record('canonical_studies', $study) SET active = true;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::scope($rpc_expiry: int, $point: string) -> object {
    fn::pse_history_v1::point($rpc_expiry,$point);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $p = SELECT key, study, ordinal, occurrence, policy, facts, outcome, run, admission_run, variant_problem, variant_operation, revision, assigned, settled, attempt, start FROM ONLY type::record('canonical_study_points', $point);
    IF $p = NONE { THROW 'PSE_REFUSAL:study_candidate_unavailable'; };
    LET $s = SELECT key, generation, cancelled, active, terminal FROM ONLY type::record('canonical_studies', $p.study);
    IF $s = NONE OR $s.active = false { THROW 'PSE_REFUSAL:study_not_active'; };
    LET $edges = type::record('canonical_study_points',$point)->(SELECT predecessor FROM canonical_study_dependencies LIMIT 193);
    IF array::len(array::distinct($edges.predecessor)) > 64 { THROW 'PSE_REFUSAL:study_candidate_immediate_dependency_bound'; };
    LET $keys = array::distinct($edges.predecessor);
    LET $predecessors = $keys.map(|$key| SELECT key, study, ordinal, occurrence, policy, facts, outcome, revision, assigned, settled, attempt, run, admission_run, variant_problem, variant_operation, start FROM ONLY type::record('canonical_study_points', $key));
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
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_history_v1::check_owner($rpc_expiry,'study',$study);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $p = fn::pse_study_v1::guard($rpc_expiry, $study,$generation,$cancelled,$point,$expected);
    IF (SELECT key FROM ONLY type::record('canonical_study_admission_outcomes',$point))=NONE { CREATE ONLY type::record('canonical_study_admission_outcomes',$point) SET key=$point,projection=type::bytes('[null,null]'); };
    LET $rpc_result = UPDATE ONLY type::record('canonical_study_points',$point) SET revision += 1dec, facts=$facts,outcome=$outcome,settled=true;
    IF $rpc_result.settled AND !$rpc_result.assigned AND $rpc_result.attempt=NONE {
        LET $run=SELECT key FROM ONLY type::record('canonical_runs',$rpc_result.run);
        IF $run=NONE {
            LET $input=SELECT key FROM ONLY type::record('canonical_runs',$rpc_result.admission_run);
            IF $input=NONE { THROW 'PSE_REFUSAL:unadmitted_occurrence_input_owner_unavailable'; };
            fn::pse_history_v1::identity($rpc_expiry,'effect-free-run',$rpc_result.run,$rpc_result.run);
        } ELSE { fn::pse_history_v1::effect_free($rpc_expiry,$rpc_result); };
    };
    fn::pse_study_v1::readiness_invalidate($rpc_expiry,$rpc_result,false);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::started($rpc_expiry: int, $study: string, $point: string, $revision: decimal, $run: string, $attempt: string, $generation: decimal, $operation: string, $request: bytes, $facts: bytes) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
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
    fn::pse_workbench_v1::publication($rpc_expiry);
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
    LET $rpc_result = fn::pse_execution_v1::claim($rpc_expiry, $s.run,$operation,$request,$attempt,$worker,$lifetime,$s.interpretation,$study,NONE);
    fn::pse_execution_v1::deadline($rpc_expiry);
    IF $existing_operation = NONE { fn::pse_execution_v1::lease($rpc_expiry, $rpc_result.expires_at); };
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::seal($rpc_expiry: int, $study: string, $run: string, $attempt: string, $authority: decimal, $operation: string, $request: bytes, $manifest: string, $digest: string, $outcome: string, $completion: bytes) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
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
    LET $p=fn::pse_study_v1::guard($rpc_expiry,$authority.study,$authority.generation,false,$authority.point,$authority.expected);
    IF $authority.run!=$p.admission_run { THROW 'PSE_REFUSAL:scientific_admission_capability_belongs_elsewhere'; };
    IF $p.admission!=NONE { THROW 'PSE_REFUSAL:scientific_admission_already_settled'; };
    fn::pse_execution_v1::current_writer($rpc_expiry,$authority.run);
    RETURN fn::pse_execution_v1::fence($rpc_expiry,$authority.run,$authority.attempt,$authority.authority);
};

// Input admission uses a separate run. Its lease never increments native point attempts.
DEFINE FUNCTION fn::pse_study_v1::admission_claim($rpc_expiry:int,$study:string,$generation:decimal,$point:string,$expected:array<object>,$operation:string,$request:bytes,$attempt:string,$worker:string,$lifetime:int) -> option<object> {
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $p=fn::pse_study_v1::guard($rpc_expiry,$study,$generation,false,$point,$expected);
    IF $p.admission!=NONE { RETURN NONE; };
    LET $run=$p.admission_run;
    fn::pse_execution_v1::current_writer($rpc_expiry,$run);
    LET $r=SELECT * FROM ONLY type::record('canonical_runs',$run) FOR UPDATE;
    IF $r=NONE OR $r.purpose!='study_admission' OR $r.admission_study!=$study OR $r.admission_point!=$point { THROW 'PSE_REFUSAL:scientific_admission_run_unavailable'; };
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
    LET $claimed=fn::pse_execution_v1::claim($rpc_expiry,$run,$operation,$request,$attempt,$worker,$lifetime,$s.interpretation,NONE,{study:$study,generation:$generation,point:$point,expected:$expected});
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry,$claimed.expires_at);
    RETURN $claimed;
};

DEFINE FUNCTION fn::pse_study_v1::admission_complete($rpc_expiry:int,$study:string,$generation:decimal,$point:string,$expected:array<object>,$run:string,$attempt:string,$authority:decimal,$operation:string,$request:bytes,$admission:bytes,$projection:option<bytes>,$facts:bytes,$outcome:option<bytes>) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
    fn::pse_history_v1::point($rpc_expiry,$point);
    fn::pse_execution_v1::deadline($rpc_expiry);
    LET $op=fn::pse_execution_v1::operation($rpc_expiry,$operation,'study-admission',$request);
    LET $saved=SELECT * FROM ONLY type::record('canonical_study_points',$point);
    IF $saved.admission!=NONE {
        IF $saved.admission!=$admission OR $op=NONE { THROW 'PSE_REFUSAL:immutable_scientific_admission_differs'; };
        RETURN $saved;
    };
    LET $p=fn::pse_study_v1::guard($rpc_expiry,$study,$generation,false,$point,$expected);
    IF $run!=$p.admission_run { THROW 'PSE_REFUSAL:scientific_admission_owner_differs'; };
    LET $a=fn::pse_execution_v1::fence($rpc_expiry,$run,$attempt,$authority);
    IF $outcome=NONE {
        LET $science=SELECT key FROM ONLY type::record('canonical_runs',$p.run);
        IF $science=NONE { THROW 'PSE_REFUSAL:selected_scientific_source_run_unavailable'; };
    };
    IF $projection!=NONE { CREATE ONLY type::record('canonical_study_admission_outcomes',$point) SET key=$point,projection=$projection; };
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
    fn::pse_workbench_v1::publication($rpc_expiry);
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
    LET $claimed = fn::pse_execution_v1::claim($rpc_expiry, $p.run, $operation + ':attempt', $claim_request, $attempt, $worker, $lifetime, $s.interpretation,NONE,NONE);
    UPDATE ONLY type::record('canonical_study_points', $point) SET assigned = true, revision += 1dec, facts = $facts, start = $start, attempt = $attempt;
    fn::pse_study_v1::readiness_invalidate($rpc_expiry,(SELECT * FROM ONLY type::record('canonical_study_points',$point)),false);
    fn::pse_history_v1::record_operation($rpc_expiry,$operation,$p.run,$attempt,'study-claim',$request,$request,$study,$point);
    LET $rpc_result = $claimed;
    fn::pse_execution_v1::deadline($rpc_expiry);
    fn::pse_execution_v1::lease($rpc_expiry, $claimed.expires_at);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::observe($rpc_expiry: int, $point: string, $revision: decimal, $attempt: option<string>, $facts: bytes, $outcome: bytes, $settled: bool) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
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
    IF $settled AND (SELECT key FROM ONLY type::record('canonical_study_admission_outcomes',$point))=NONE { CREATE ONLY type::record('canonical_study_admission_outcomes',$point) SET key=$point,projection=type::bytes('[null,null]'); };
    LET $rpc_result = UPDATE ONLY type::record('canonical_study_points', $point) SET revision += 1dec, assigned = false, settled = $settled, facts = $facts, outcome = $outcome;
    IF $rpc_result.settled AND !$rpc_result.assigned AND $rpc_result.attempt=NONE {
        LET $run=SELECT key FROM ONLY type::record('canonical_runs',$rpc_result.run);
        IF $run=NONE {
            LET $input=SELECT key FROM ONLY type::record('canonical_runs',$rpc_result.admission_run);
            IF $input=NONE { THROW 'PSE_REFUSAL:unadmitted_occurrence_input_owner_unavailable'; };
            fn::pse_history_v1::identity($rpc_expiry,'effect-free-run',$rpc_result.run,$rpc_result.run);
        } ELSE { fn::pse_history_v1::effect_free($rpc_expiry,$rpc_result); };
    };
    fn::pse_study_v1::readiness_invalidate($rpc_expiry,$rpc_result,false);
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN $rpc_result;
};

DEFINE FUNCTION fn::pse_study_v1::cancel($rpc_expiry: int, $study: string) -> object {
    fn::pse_workbench_v1::publication($rpc_expiry);
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
    fn::pse_workbench_v1::publication($rpc_expiry);
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
