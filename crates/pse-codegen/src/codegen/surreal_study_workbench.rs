// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded complete cohort capture shares the selector's one native snapshot.
pub(super) fn append(ddl: &mut String) {
    ddl.push_str(&r#"
// Conservative global publication scope includes future matching studies and all
// selector metadata. Expiry is never a writer-side bypass of the checked guard.
DEFINE FUNCTION fn::pse_workbench_v1::publication($rpc_expiry:int) -> bool {
    LET $gate=SELECT * FROM ONLY canonical_study_capture_gate:current FOR UPDATE;
    IF $gate!=NONE AND $gate.capture!=NONE { THROW 'PSE_REFUSAL:study_capture_publication_busy'; };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
};
// A physical tuple key is injectively framed, including opaque IDs containing
// delimiters. This is operational encoding, not a new semantic hash identity.
DEFINE FUNCTION fn::pse_workbench_v1::protection_key($capture:string,$study:string,$attempt:string) -> string {
    RETURN encoding::base64::encode(encoding::cbor::encode([$capture,$study,$attempt]));
};
DEFINE FUNCTION fn::pse_workbench_v1::open_capture($rpc_expiry:int,$key:string,$owner:string,$request:bytes) -> object {
    LET $old=SELECT * FROM ONLY type::record('canonical_study_captures',$key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.owner!=$owner OR $old.request!=$request { THROW 'PSE_REFUSAL:study_capture_identity_reused'; };
        RETURN $old;
    };
    LET $process=SELECT * FROM ONLY type::record('canonical_file_owners',$owner) FOR UPDATE;
    IF $process=NONE OR $process.drained OR $process.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:study_capture_actual_owner_unavailable'; };
    fn::pse_workbench_v1::publication($rpc_expiry);
    LET $incarnation=<string>rand::uuid::v4();
    UPSERT ONLY canonical_study_capture_gate:current SET key='current',incarnation=$incarnation,capture=$key,owner=$owner,request=$request,issued_expires_at=<decimal>$rpc_expiry;
    LET $saved=CREATE ONLY type::record('canonical_study_captures',$key) SET key=$key,owner=$owner,request=$request,incarnation=$incarnation,issued_expires_at=<decimal>$rpc_expiry,state='pending',header_count=0dec,member_count=0dec;
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $saved;
};
// Selection, exact facts, all selected manifest protections and gate closure
// commit together. Only compact receipt leaves this transaction.
DEFINE FUNCTION fn::pse_workbench_v1::commit_capture($rpc_expiry:int,$key:string,$owner:string,$request:bytes,$rows:array<object>,$maximum:int,$members:int) -> object {
    LET $capture=SELECT * FROM ONLY type::record('canonical_study_captures',$key) FOR UPDATE;
    IF $capture=NONE OR $capture.owner!=$owner OR $capture.request!=$request { THROW 'PSE_REFUSAL:study_capture_identity_reused'; };
    IF $capture.state='frozen' { RETURN $capture; };
    LET $gate=SELECT * FROM ONLY canonical_study_capture_gate:current FOR UPDATE;
    IF $capture.state!='pending' OR $gate=NONE OR $gate.capture!=$key OR $gate.owner!=$owner OR $gate.request!=$request OR $gate.incarnation!=$capture.incarnation OR $gate.issued_expires_at!=$capture.issued_expires_at OR $capture.issued_expires_at<=time::micros() { THROW 'PSE_REFUSAL:study_capture_original_grant_closed'; };
    LET $process=SELECT * FROM ONLY type::record('canonical_file_owners',$owner) FOR UPDATE;
    IF $process=NONE OR $process.drained OR $process.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:study_capture_actual_owner_unavailable'; };
    IF array::len($rows)>$maximum OR $maximum<1 OR $maximum>64 OR $members<1 OR $members>__COHORT_MEMBERS__ { THROW 'PSE_REFUSAL:study_workbench_narrow_selection'; };
    FOR $s IN $rows { fn::pse_history_v1::check_owner($rpc_expiry,'study',$s.key); };
    LET $total=math::sum($rows.point_count) ?? 0dec;
    IF array::len($rows.filter(|$s| !$s.active))!=0 OR $total>$members { THROW 'PSE_REFUSAL:study_workbench_narrow_selection'; };
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
    IF array::len($points)!=$total { THROW 'PSE_REFUSAL:study_workbench_narrow_selection'; };
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
    IF array::len($headers)!=0 { INSERT INTO canonical_study_capture_headers $headers RETURN NONE; };
    IF array::len($facts)!=0 { INSERT INTO canonical_study_capture_members $facts RETURN NONE; };
    IF array::len($protections)!=0 { INSERT INTO canonical_study_capture_protections $protections RETURN NONE; };
    UPDATE ONLY canonical_study_capture_gate:current SET incarnation=<string>rand::uuid::v4(),capture=NONE,owner=NONE,request=NONE,issued_expires_at=0dec;
    LET $saved=UPDATE ONLY type::record('canonical_study_captures',$key) SET state='frozen',header_count=<decimal>array::len($headers),member_count=<decimal>array::len($facts);
    IF $capture.issued_expires_at<=time::micros() { THROW 'PSE_REFUSAL:study_capture_original_grant_closed'; };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $saved;
};
DEFINE FUNCTION fn::pse_workbench_v1::capture_page($rpc_expiry:int,$key:string,$owner:string,$request:bytes,$kind:string,$after:decimal) -> array<object> {
    LET $capture=SELECT * FROM ONLY type::record('canonical_study_captures',$key) FOR UPDATE;
    IF $capture=NONE OR $capture.owner!=$owner OR $capture.request!=$request OR $capture.state!='frozen' { THROW 'PSE_REFUSAL:study_capture_original_owner_closed'; };
    LET $process=SELECT * FROM ONLY type::record('canonical_file_owners',$owner) FOR UPDATE;
    IF $process=NONE OR $process.drained OR $process.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:study_capture_actual_owner_unavailable'; };
    LET $rows=IF $kind='headers' {SELECT ordinal,body FROM canonical_study_capture_headers WHERE capture=$key AND ordinal>$after ORDER BY ordinal LIMIT 64} ELSE IF $kind='members' {SELECT ordinal,body FROM canonical_study_capture_members WHERE capture=$key AND ordinal>$after ORDER BY ordinal LIMIT __CAPTURE_ROWS__} ELSE {THROW 'PSE_REFUSAL:study_capture_page_kind';};
    // Byte-bound each page as well as its row count; keep an exact prefix.
    LET $page=$rows.fold({items:[],bytes:0,stopped:false},|$acc,$row| {
        LET $bytes=$acc.bytes+bytes::len($row.body)+128;
        IF $acc.stopped OR ($bytes>__STUDY_BYTES__ AND array::len($acc.items)!=0) {RETURN object::extend($acc,{stopped:true});};
        RETURN {items:array::append($acc.items,{ordinal:$row.ordinal,body:encoding::cbor::decode($row.body)}),bytes:$bytes,stopped:false};
    });
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $page.items;
};
// Every lazy native read consumes the original unforgeable capture witness and
// checks the current process/restore identity before exact immutable references.
DEFINE FUNCTION fn::pse_workbench_v1::captured_result($rpc_expiry:int,$key:string,$owner:string,$request:bytes,$position:decimal) -> object {
    LET $capture=SELECT * FROM ONLY type::record('canonical_study_captures',$key) FOR UPDATE;
    IF $capture=NONE OR $capture.owner!=$owner OR $capture.request!=$request OR $capture.state!='frozen' {THROW 'PSE_REFUSAL:study_capture_original_owner_closed';};
    LET $process=SELECT * FROM ONLY type::record('canonical_file_owners',$owner) FOR UPDATE;
    IF $process=NONE OR $process.drained OR $process.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store {THROW 'PSE_REFUSAL:study_capture_actual_owner_unavailable';};
    LET $member=SELECT * FROM ONLY type::record('canonical_study_capture_members',$key+':'+<string>$position);
    IF $member=NONE OR $member.capture!=$key OR $member.ordinal!=$position OR $member.attempt=NONE OR $member.manifest=NONE {THROW 'PSE_REFUSAL:terminal_result_selection_unavailable';};
    LET $protection=SELECT * FROM ONLY type::record('canonical_study_capture_protections',fn::pse_workbench_v1::protection_key($key,$member.study,$member.attempt));
    LET $run=SELECT key,problem,revision,interpretation FROM ONLY type::record('canonical_runs',$member.run);
    LET $attempt=SELECT key,run,phase,closed_manifest,generation FROM ONLY type::record('canonical_attempts',$member.attempt);
    LET $manifest=SELECT * FROM ONLY type::record('canonical_result_manifests',$member.manifest);
    IF $protection=NONE OR $protection.capture!=$key OR $protection.study!=$member.study OR $protection.run!=$member.run OR $protection.attempt!=$member.attempt OR $protection.manifest!=$member.manifest OR $run=NONE OR $run.problem!=$protection.problem OR $run.revision!=$protection.revision OR $attempt=NONE OR $attempt.run!=$member.run OR $attempt.phase NOT IN ['terminal','terminal_compacted'] OR $attempt.closed_manifest!=$member.manifest OR $manifest=NONE OR $manifest.attempt!=$member.attempt OR $manifest.generation!=$attempt.generation {THROW 'PSE_REFUSAL:terminal_result_selection_unavailable';};
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {member:$member,run:$run,attempt:$attempt,manifest:$manifest};
};
// One useful bounded locator page may cover many original members. No per-member
// result ticket, full snapshot or open transaction survives this native call.
DEFINE FUNCTION fn::pse_workbench_v1::result_page($rpc_expiry:int,$key:string,$owner:string,$request:bytes,$selected:array<decimal>,$relation:string,$output:string,$partition:string,$dense:bool,$start:decimal,$end:decimal,$member:option<decimal>,$after:option<decimal>) -> object {
    IF array::len($selected)>__CAPTURE_ROWS__ {THROW 'PSE_REFUSAL:study_workbench_narrow_selection';};
    LET $page=$selected.filter(|$position|$member=NONE OR $position>=$member).fold({items:[],bytes:512,blocks:0,next_member:$member,next_after:$after,stopped:false},|$acc,$position| {
        IF $acc.stopped {RETURN $acc;};
        LET $source=fn::pse_workbench_v1::captured_result($rpc_expiry,$key,$owner,$request,$position);
        LET $sets=SELECT key,next_ordinal FROM canonical_result_sets WHERE attempt=$source.member.attempt AND name=$relation LIMIT 1;
        LET $set=IF array::len($sets)=0 {NONE} ELSE {$sets[0]};
        LET $resume=IF $member=$position {$after} ELSE {NONE};
        LET $locators=IF $set=NONE {NONE} ELSE IF $start=$end {IF $dense {{indexes:[],blocks:[],batches:[],next:NULL,exhausted:true}} ELSE {{blocks:[],batches:[],next:NULL,exhausted:true}}} ELSE IF $dense {fn::pse_execution_v1::result_output_page($rpc_expiry,$set.key,$output,$partition,$start,$end,$resume,NONE,NONE)} ELSE {fn::pse_execution_v1::result_block_page($rpc_expiry,$set.key,$relation,'0',$start,$end,$resume,$set.next_ordinal)};
        LET $item={position:$position,attempt:$source.member.attempt,generation:$source.attempt.generation,interpretation:$source.run.interpretation,manifest:$source.manifest,set:$set,page:$locators};
        LET $blocks=$acc.blocks+IF $locators=NONE {0} ELSE {array::len($locators.blocks)};
        LET $bytes=$acc.bytes+bytes::len(encoding::cbor::encode($item));
        IF array::len($acc.items)>=64 OR $blocks>64 OR $bytes>__STUDY_BYTES__ {
            IF array::len($acc.items)=0 {THROW 'PSE_REFUSAL:results_page_member_extent_exceeded';};
            RETURN object::extend($acc,{stopped:true});
        };
        LET $done=$locators=NONE OR $locators.exhausted;
        RETURN {items:array::append($acc.items,$item),bytes:$bytes,blocks:$blocks,next_member:IF $done {$position+1dec} ELSE {$position},next_after:IF $done {NONE} ELSE {$locators.next},stopped:!$done};
    });
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {items:$page.items,member:$page.next_member,after:$page.next_after,exhausted:!$page.stopped};
};
// Descriptor equality is checked natively before a physical handle or body is
// exposed. The original capture reference stays held through physical drain.
DEFINE FUNCTION fn::pse_workbench_v1::result_descriptor($rpc_expiry:int,$key:string,$owner:string,$request:bytes,$position:decimal,$expected:object) -> object {
    LET $source=fn::pse_workbench_v1::captured_result($rpc_expiry,$key,$owner,$request,$position);
    LET $batch=SELECT * FROM ONLY type::record('canonical_result_batches',$expected.key);
    LET $sets=SELECT key,next_ordinal FROM canonical_result_sets WHERE attempt=$source.member.attempt AND key=$expected.result_set LIMIT 1;
    IF $batch=NONE OR object::remove($batch,'id')!=$expected OR $batch.attempt!=$source.member.attempt OR array::len($sets)!=1 OR $batch.ordinal>=$sets[0].next_ordinal {THROW 'PSE_REFUSAL:execution_batch_coordinate_mismatch';};
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $batch;
};
DEFINE FUNCTION fn::pse_workbench_v1::inline_payload($rpc_expiry:int,$key:string,$owner:string,$request:bytes,$position:decimal,$expected:object,$inline:string) -> object {
    fn::pse_workbench_v1::result_descriptor($rpc_expiry,$key,$owner,$request,$position,$expected);
    LET $body=SELECT * FROM ONLY type::record('canonical_inline_payloads',$inline);
    IF $body=NONE {THROW 'PSE_REFUSAL:execution_batch_coordinate_mismatch';};
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $body;
};
// Exact pending-gate recovery changes incarnation even after expiry, so a stale
// in-flight capture cannot commit against the reopened publication universe.
DEFINE FUNCTION fn::pse_workbench_v1::recover_capture($rpc_expiry:int,$key:string,$owner:string,$request:bytes) -> object {
    LET $capture=SELECT * FROM ONLY type::record('canonical_study_captures',$key) FOR UPDATE;
    IF $capture=NONE OR $capture.owner!=$owner OR $capture.request!=$request { THROW 'PSE_REFUSAL:study_capture_identity_reused'; };
    IF $capture.state!='pending' {RETURN $capture;};
    // The authenticated original holder can abandon early; every recovery fences
    // incarnation before reopening, including expiry and uncertain cancellation.
    LET $gate=SELECT * FROM ONLY canonical_study_capture_gate:current FOR UPDATE;
    IF $gate.capture=$key {
        IF $gate.incarnation!=$capture.incarnation OR $gate.request!=$request OR $gate.owner!=$owner { THROW 'PSE_REFUSAL:study_capture_identity_reused'; };
        UPDATE ONLY canonical_study_capture_gate:current SET incarnation=<string>rand::uuid::v4(),capture=NONE,owner=NONE,request=NONE,issued_expires_at=0dec;
    };
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN UPDATE ONLY type::record('canonical_study_captures',$key) SET state='abandoned';
};
// Only the last local capability/scan releases. A different process may clean an
// abandoned owner only after its actual authenticated native owner is drained.
DEFINE FUNCTION fn::pse_workbench_v1::release_capture($rpc_expiry:int,$key:string,$owner:string,$request:bytes,$custodian:string) -> bool {
    LET $actual=SELECT * FROM ONLY type::record('canonical_file_owners',$custodian) FOR UPDATE;
    IF $actual=NONE OR $actual.drained OR $actual.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:study_capture_actual_owner_unavailable'; };
    LET $capture=SELECT * FROM ONLY type::record('canonical_study_captures',$key) FOR UPDATE;
    IF $capture=NONE {
        IF $custodian!=$owner { THROW 'PSE_REFUSAL:study_capture_identity_reused'; };
        LET $process=SELECT * FROM ONLY type::record('canonical_file_owners',$owner) FOR UPDATE;
        IF $process=NONE OR $process.drained { THROW 'PSE_REFUSAL:study_capture_actual_owner_unavailable'; };
        // An exact tombstone fences a delayed gate acquisition after lost ACK.
        CREATE ONLY type::record('canonical_study_captures',$key) SET key=$key,owner=$owner,request=$request,incarnation='unissued',issued_expires_at=0dec,state='released',header_count=0dec,member_count=0dec;
        fn::pse_execution_v1::deadline($rpc_expiry); RETURN true;
    };
    IF $capture.owner!=$owner OR $capture.request!=$request { THROW 'PSE_REFUSAL:study_capture_identity_reused'; };
    IF $custodian!=$owner {
        LET $process=SELECT * FROM ONLY type::record('canonical_file_owners',$owner) FOR UPDATE;
        IF $process=NONE OR !$process.drained { THROW 'PSE_REFUSAL:study_capture_original_owner_not_drained'; };
    };
    IF $capture.state='pending' {
        fn::pse_workbench_v1::recover_capture($rpc_expiry,$key,$owner,$request);
    };
    IF $capture.state='released' {RETURN true;};
    UPDATE ONLY type::record('canonical_study_captures',$key) SET state='releasing';
    LET $references=SELECT key FROM canonical_study_capture_protections WHERE capture=$key LIMIT __CAPTURE_ROWS__;
    LET $members=SELECT key FROM canonical_study_capture_members WHERE capture=$key LIMIT __CAPTURE_ROWS__;
    LET $headers=SELECT key FROM canonical_study_capture_headers WHERE capture=$key LIMIT 64;
    FOR $row IN $references { DELETE ONLY type::record('canonical_study_capture_protections',$row.key); };
    FOR $row IN $members { DELETE ONLY type::record('canonical_study_capture_members',$row.key); };
    FOR $row IN $headers { DELETE ONLY type::record('canonical_study_capture_headers',$row.key); };
    LET $done=array::len($references)<__CAPTURE_ROWS__ AND array::len($members)<__CAPTURE_ROWS__ AND array::len($headers)<64;
    IF $done { UPDATE ONLY type::record('canonical_study_captures',$key) SET state='released'; };
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $done;
};

DEFINE FUNCTION fn::pse_workbench_v1::browse($rpc_expiry:int,$rows:array<object>,$maximum:int) -> object {
    LET $selected=array::slice($rows,0,$maximum);
    FOR $s IN $selected {fn::pse_history_v1::check_owner($rpc_expiry,'study',$s.key);};
    LET $headers=$selected.map(|$s| {
        LET $run=SELECT terminal_attempt,terminal_class FROM ONLY type::record('canonical_runs',$s.run);
        RETURN object::extend($s,{result_attempt:$run.terminal_attempt,outcome:$run.terminal_class});
    });
    LET $result={headers:$headers,members:[],has_more:array::len($rows)>$maximum};
    IF bytes::len(encoding::cbor::encode($result))>1048576 {THROW 'PSE_REFUSAL:study_workbench_narrow_selection';};
    fn::pse_execution_v1::deadline($rpc_expiry); RETURN $result;
};
"#.replace("__CAPTURE_ROWS__",&pse_schema::catalog::study_workbench::STUDY_CAPTURE_PAGE_ROWS.to_string()).replace("__COHORT_MEMBERS__",&pse_schema::catalog::study_workbench::STUDY_COHORT_MAX_MEMBERS.to_string()).replace("__COHORT_SENTINEL__",&(pse_schema::catalog::study_workbench::STUDY_COHORT_MAX_MEMBERS+1).to_string()).replace("__STUDY_BYTES__",&pse_schema::catalog::study_workbench::STUDY_PAGE_BYTES.to_string()));
}
