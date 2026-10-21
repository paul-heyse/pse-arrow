// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Transactional coordination of immutable local file effects and exact readers.

pub(super) fn append(ddl: &mut String) {
    ddl.push_str(r#"
DEFINE FUNCTION fn::pse_files_v1::admission($expiry:int,$incarnation:string,$collecting:bool) -> object {
    fn::pse_execution_v1::deadline($expiry);
    LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE;
    IF $gate=NONE OR $gate.storage_incarnation!=$incarnation OR ($collecting AND $gate.gc_frozen) OR (!$collecting AND $gate.frozen) { THROW 'PSE_REFUSAL:file_admission_closed_or_incarnation_differs'; };
    RETURN $gate;
};
DEFINE FUNCTION fn::pse_files_v1::owner($expiry:int,$key:string,$incarnation:string) -> object {
    fn::pse_files_v1::admission($expiry,$incarnation,false);
    LET $owner=SELECT * FROM ONLY type::record('canonical_file_owners',$key) FOR UPDATE;
    IF $owner=NONE OR $owner.drained OR $owner.storage_incarnation!=$incarnation OR $owner.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:file_actual_owner_unavailable'; };
    RETURN $owner;
};
DEFINE FUNCTION fn::pse_files_v1::register_owner($expiry:int,$row:object) -> object {
    fn::pse_files_v1::admission($expiry,$row.storage_incarnation,false);
    IF $row.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store OR $row.drained OR $row.pid=0dec OR string::len($row.boot)=0 OR string::len($row.start)=0 OR string::len($row.pid_namespace)=0 { THROW 'PSE_REFUSAL:file_process_identity_incomplete'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_file_owners',$row.key) FOR UPDATE;
    IF $old!=NONE { IF object::remove($old,'id')!=$row { THROW 'PSE_REFUSAL:file_owner_identity_reused'; }; RETURN $old; };
    RETURN CREATE ONLY type::record('canonical_file_owners',$row.key) CONTENT $row;
};
DEFINE FUNCTION fn::pse_files_v1::reserve($expiry:int,$row:object,$run:option<string>,$attempt:option<string>,$generation:option<decimal>) -> object {
    fn::pse_files_v1::owner($expiry,$row.owner,$row.storage_incarnation);
    LET $same=SELECT * FROM canonical_file_instances WHERE intent=$row.intent LIMIT 1;
    IF array::len($same)!=0 {
        LET $old=SELECT * FROM ONLY type::record('canonical_file_instances',$same[0].key) FOR UPDATE;
        IF $old.request_digest!=$row.request_digest OR $old.owner_kind!=$row.owner_kind OR $old.selection!=$row.selection OR $old.storage_incarnation!=$row.storage_incarnation OR $old.state IN ['collecting','collected','abandoned'] { THROW 'PSE_REFUSAL:original_file_intent_differs_or_settled'; };
        RETURN $old;
    };
    IF $row.owner_kind='result' {
        IF $run=NONE OR $attempt=NONE OR $generation=NONE OR $row.selection!=$attempt { THROW 'PSE_REFUSAL:result_file_original_fence_missing'; };
        LET $writer=fn::pse_execution_v1::writer($expiry,$run,$attempt,$generation,$row.intent,'append');
        IF $writer.pse_writer!='admitted' { RETURN $writer; };
        IF ($writer.attempt.phase!='ingesting') { THROW 'PSE_REFUSAL:result_file_writer_refused'; };
    } ELSE IF $row.owner_kind='artifact' {
        LET $artifact=SELECT * FROM ONLY type::record('canonical_artifacts',$row.selection) FOR UPDATE;
        IF $artifact=NONE OR $artifact.retiring OR $artifact.active OR $artifact.creation_expires_at<=time::micros() { THROW 'PSE_REFUSAL:artifact_file_original_creation_unavailable'; };
    } ELSE { THROW 'PSE_REFUSAL:file_intent_owner_kind'; };
    IF $row.state!='pending' OR $row.digest!='' OR $row.payload_bytes!=0dec OR $row.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:file_original_intent_malformed'; };
    RETURN CREATE ONLY type::record('canonical_file_instances',$row.key) CONTENT $row;
};
// Custody recovery changes no original occurrence, writer generation, deadline,
// request or physical instance. Only authenticated actual owner drain permits it.
DEFINE FUNCTION fn::pse_files_v1::recover_intent($expiry:int,$expected:object,$new_owner:string,$incarnation:string) -> object {
    fn::pse_files_v1::owner($expiry,$new_owner,$incarnation);
    LET $file=SELECT * FROM ONLY type::record('canonical_file_instances',$expected.instance) FOR UPDATE;
    IF $file=NONE OR $file.intent!=$expected.intent OR $file.request_digest!=$expected.request_digest OR $file.storage_incarnation!=$incarnation OR $file.state IN ['abandoned','collecting','collected'] { THROW 'PSE_REFUSAL:original_file_recovery_intent_differs_or_settled'; };
    IF $file.owner=$new_owner { RETURN $file; };
    IF $file.owner!=$expected.owner { THROW 'PSE_REFUSAL:original_file_recovery_custody_differs'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_file_owners',$file.owner) FOR UPDATE;
    IF $old=NONE OR !$old.drained { THROW 'PSE_REFUSAL:original_file_owner_has_no_authenticated_drain_run_recover_file_owners_state_path'; };
    RETURN UPDATE ONLY type::record('canonical_file_instances',$file.key) SET owner=$new_owner;
};
// Called while the unique OS instance lock is held, before any physical
// mutation. This fences late original callers after irreversible collection.
DEFINE FUNCTION fn::pse_files_v1::intent_open($expiry:int,$expected:object,$owner:string,$incarnation:string) -> object {
    fn::pse_files_v1::owner($expiry,$owner,$incarnation);
    LET $file=SELECT * FROM ONLY type::record('canonical_file_instances',$expected.instance) FOR UPDATE;
    IF $file=NONE OR $file.intent!=$expected.intent OR $file.request_digest!=$expected.request_digest OR $file.owner!=$owner OR $file.storage_incarnation!=$incarnation OR $file.state NOT IN ['pending','sealed','attached'] { THROW 'PSE_REFUSAL:original_physical_intent_is_settled_or_differs'; };
    RETURN $file;
};
DEFINE FUNCTION fn::pse_files_v1::seal($expiry:int,$key:string,$owner:string,$incarnation:string,$descriptor:object) -> object {
    fn::pse_files_v1::owner($expiry,$owner,$incarnation);
    LET $row=SELECT * FROM ONLY type::record('canonical_file_instances',$key) FOR UPDATE;
    IF $row=NONE OR $row.owner!=$owner OR $row.storage_incarnation!=$incarnation OR $descriptor.instance!=$key OR $row.state NOT IN ['pending','sealed','attached'] { THROW 'PSE_REFUSAL:file_sealing_original_intent_unavailable'; };
    IF string::len($descriptor.digest)!=64 OR $descriptor.bytes=0dec OR $descriptor.bytes>33554432dec OR $descriptor.encoding NOT IN ['arrow-ipc-file','opaque-json','opaque-bytes'] OR ($descriptor.encoding='arrow-ipc-file' AND $descriptor.batch_count=0dec) OR ($descriptor.encoding!='arrow-ipc-file' AND $descriptor.batch_count!=0dec) { THROW 'PSE_REFUSAL:sealed_file_extent_or_encoding_invalid'; };
    IF $row.state!='pending' {
        IF $row.digest!=$descriptor.digest OR $row.payload_bytes!=$descriptor.bytes OR $row.encoding!=$descriptor.encoding OR $row.batch_count!=$descriptor.batch_count OR $row.schema!=$descriptor.schema OR $row.interpretation!=$descriptor.interpretation { THROW 'PSE_REFUSAL:sealed_file_immutable_descriptor_differs'; };
        RETURN $row;
    };
    RETURN UPDATE ONLY type::record('canonical_file_instances',$key) SET digest=$descriptor.digest,payload_bytes=$descriptor.bytes,encoding=$descriptor.encoding,batch_count=$descriptor.batch_count,schema=$descriptor.schema,interpretation=$descriptor.interpretation,state='sealed';
};
DEFINE FUNCTION fn::pse_files_v1::attach($expiry:int,$batch:object,$kind:string,$selection:string,$borrower:string) -> bool {
    LET $file=SELECT * FROM ONLY type::record('canonical_file_instances',$batch.file_instance) FOR UPDATE;
    IF $file=NONE { THROW 'PSE_REFUSAL:sealed_file_instance_unavailable'; };
    fn::pse_files_v1::admission($expiry,$file.storage_incarnation,false);
    IF $file.state NOT IN ['sealed','attached'] OR $file.digest!=$batch.file_digest OR $file.payload_bytes!=$batch.payload_bytes OR $file.encoding!=$batch.encoding OR $file.schema!=$batch.schema OR ($file.encoding='arrow-ipc-file' AND $batch.file_batch_index>=$file.batch_count) OR ($file.encoding!='arrow-ipc-file' AND $batch.file_batch_index!=0dec) { THROW 'PSE_REFUSAL:logical_batch_sealed_file_binding_differs'; };
    IF $file.owner_kind!=$kind OR $file.selection!=$selection { THROW 'PSE_REFUSAL:file_original_intent_selected_another_owner'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_file_borrowers',$borrower) FOR UPDATE;
    IF $old!=NONE AND ($old.released OR $old.instance!=$file.key OR $old.owner_kind!=$kind OR $old.owner!=$selection) { THROW 'PSE_REFUSAL:file_borrower_identity_released_or_reused'; };
    IF $old=NONE { CREATE ONLY type::record('canonical_file_borrowers',$borrower) SET key=$borrower,instance=$file.key,owner_kind=$kind,owner=$selection,released=false; };
    UPDATE ONLY type::record('canonical_file_instances',$file.key) SET state='attached';
    RETURN true;
};
DEFINE FUNCTION fn::pse_files_v1::release_borrower($expiry:int,$key:string) -> bool {
    LET $borrower=SELECT * FROM ONLY type::record('canonical_file_borrowers',$key) FOR UPDATE;
    IF $borrower=NONE { THROW 'PSE_REFUSAL:original_file_borrower_unavailable'; };
    SELECT * FROM ONLY type::record('canonical_file_instances',$borrower.instance) FOR UPDATE;
    UPDATE ONLY type::record('canonical_file_borrowers',$key) SET released=true;
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_files_v1::ticket($expiry:int,$row:object) -> object {
    fn::pse_files_v1::owner($expiry,$row.owner,$row.storage_incarnation);
    LET $pin=SELECT * FROM ONLY type::record('canonical_protections',$row.key) FOR UPDATE;
    IF $pin=NONE OR $pin.released OR $pin.expires_at<=time::micros() OR $row.released { THROW 'PSE_REFUSAL:read_ticket_original_admission_unavailable'; };
    IF $row.kind='result' {
        fn::pse_execution_v1::result_creator($expiry,$row.run);
        LET $selection=SELECT * FROM ONLY type::record('canonical_result_protections',$row.key) FOR UPDATE;
        IF $selection=NONE OR $selection.run!=$row.run OR $selection.attempt!=$row.attempt OR $selection.manifest!=$row.manifest OR $row.selection!=$row.manifest { THROW 'PSE_REFUSAL:read_ticket_exact_manifest_differs'; };
    } ELSE IF $row.kind='artifact' {
        LET $artifact=fn::pse_artifact_v1::available($expiry,$row.selection);
        LET $read=SELECT * FROM ONLY type::record('canonical_artifact_reads',$row.key) FOR UPDATE;
        IF $read=NONE OR $read.artifact!=$row.selection OR $artifact.revision!=$pin.revision { THROW 'PSE_REFUSAL:artifact_ticket_selection_differs'; };
    } ELSE { THROW 'PSE_REFUSAL:read_ticket_kind'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_manifest_tickets',$row.key) FOR UPDATE;
    IF $old!=NONE { IF object::remove($old,'id')!=$row { THROW 'PSE_REFUSAL:read_ticket_identity_reused'; }; RETURN $old; };
    RETURN CREATE ONLY type::record('canonical_manifest_tickets',$row.key) CONTENT $row;
};
DEFINE FUNCTION fn::pse_files_v1::ticket_open($expiry:int,$key:string) -> bool {
    LET $ticket=SELECT * FROM ONLY type::record('canonical_manifest_tickets',$key) FOR UPDATE;
    IF $ticket=NONE OR $ticket.released { RETURN false; };
    LET $owner=SELECT * FROM ONLY type::record('canonical_file_owners',$ticket.owner) FOR UPDATE;
    LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE;
    IF $owner=NONE OR $gate=NONE { THROW 'PSE_REFUSAL:read_ticket_actual_process_owner_unavailable'; };
    IF $owner.drained { RETURN false; };
    IF $ticket.storage_incarnation!=$gate.storage_incarnation OR $owner.storage_incarnation!=$gate.storage_incarnation { THROW 'PSE_REFUSAL:read_ticket_actual_process_incarnation_unavailable'; };
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_files_v1::release_ticket($expiry:int,$key:string) -> bool {
    LET $ticket=SELECT * FROM ONLY type::record('canonical_manifest_tickets',$key) FOR UPDATE;
    IF $ticket!=NONE { UPDATE ONLY type::record('canonical_manifest_tickets',$key) SET released=true; };
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_files_v1::settle_intent($expiry:int,$expected:object,$incarnation:string) -> bool {
    fn::pse_files_v1::admission($expiry,$incarnation,false);
    LET $file=SELECT * FROM ONLY type::record('canonical_file_instances',$expected.instance) FOR UPDATE;
    IF $file=NONE OR $file.intent!=$expected.intent OR $file.request_digest!=$expected.request_digest OR $file.owner!=$expected.owner OR $file.storage_incarnation!=$incarnation { THROW 'PSE_REFUSAL:file_settlement_requires_exact_original_intent'; };
    IF $file.state IN ['attached','abandoned','collecting','collected'] { RETURN $file.state!='attached'; };
    LET $owner=SELECT * FROM ONLY type::record('canonical_file_owners',$file.owner) FOR UPDATE;
    IF $owner=NONE OR !$owner.drained { THROW 'PSE_REFUSAL:file_settlement_requires_authenticated_actual_owner_drain'; };
    IF $file.owner_kind='result' {
        LET $attempt=SELECT * FROM ONLY type::record('canonical_attempts',$file.selection) FOR UPDATE;
        IF $attempt=NONE { THROW 'PSE_REFUSAL:original_result_file_attempt_settlement_unavailable'; };
        fn::pse_execution_v1::touch($expiry,'execution-run:'+$attempt.run);
        IF ($attempt.phase='ingesting') AND $attempt.expires_at>time::micros() { RETURN false; };
    } ELSE IF $file.owner_kind='artifact' {
        LET $artifact=SELECT * FROM ONLY type::record('canonical_artifacts',$file.selection) FOR UPDATE;
        IF $artifact!=NONE AND !$artifact.retiring AND $artifact.creation_expires_at>time::micros() { RETURN false; };
        IF $artifact=NONE { LET $parts=string::split($file.selection,':'); LET $guards=SELECT artifact_creation_closed_through FROM canonical_guards WHERE incarnation=$parts[1] LIMIT 1; IF array::len($guards)!=1 OR ($guards[0].artifact_creation_closed_through ?? 0dec)< <decimal>$parts[3] { THROW 'PSE_REFUSAL:original_artifact_file_settlement_floor_unavailable'; }; };
    } ELSE { THROW 'PSE_REFUSAL:original_file_settlement_owner_kind'; };
    UPDATE ONLY type::record('canonical_file_instances',$file.key) SET state='abandoned';
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_files_v1::collect($expiry:int,$key:string,$incarnation:string) -> object {
    fn::pse_files_v1::admission($expiry,$incarnation,true);
    LET $file=SELECT * FROM ONLY type::record('canonical_file_instances',$key) FOR UPDATE;
    IF $file=NONE OR $file.storage_incarnation!=$incarnation { THROW 'PSE_REFUSAL:file_collection_exact_placement_unavailable'; };
    IF $file.state='collected' { RETURN $file; };
    IF $file.state IN ['pending','sealed'] { THROW 'PSE_REFUSAL:file_publication_remains_unresolved'; };
    LET $borrowers=SELECT key FROM canonical_file_borrowers WHERE instance=$key AND released=false LIMIT 1;
    IF array::len($borrowers)!=0 { THROW 'PSE_REFUSAL:file_still_has_independent_borrowers'; };
    RETURN UPDATE ONLY type::record('canonical_file_instances',$key) SET state='collecting';
};
DEFINE FUNCTION fn::pse_files_v1::collected($expiry:int,$key:string,$incarnation:string) -> bool {
    fn::pse_files_v1::admission($expiry,$incarnation,true);
    LET $file=SELECT * FROM ONLY type::record('canonical_file_instances',$key) FOR UPDATE;
    IF $file=NONE OR $file.storage_incarnation!=$incarnation OR $file.state NOT IN ['collecting','collected'] { THROW 'PSE_REFUSAL:file_collection_original_intent_unavailable'; };
    UPDATE ONLY type::record('canonical_file_instances',$key) SET state='collected';
    RETURN true;
};
DEFINE FUNCTION fn::pse_files_v1::freeze($expiry:int,$incarnation:string) -> bool {
    LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE;
    IF $gate=NONE OR $gate.storage_incarnation!=$incarnation { THROW 'PSE_REFUSAL:file_freeze_incarnation_differs'; };
    UPDATE ONLY canonical_file_admission:current SET frozen=true,gc_frozen=true;
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_files_v1::thaw($expiry:int,$incarnation:string) -> bool {
    LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE;
    IF $gate=NONE OR $gate.storage_incarnation!=$incarnation { THROW 'PSE_REFUSAL:file_thaw_incarnation_differs'; };
    UPDATE ONLY canonical_file_admission:current SET frozen=false,gc_frozen=false;
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_files_v1::drain_owner($expiry:int,$expected:object) -> bool {
    LET $owner=SELECT * FROM ONLY type::record('canonical_file_owners',$expected.key) FOR UPDATE;
    IF $owner=NONE {
        // Local cancellation can precede registration dispatch. Fence that
        // original key permanently before any delayed create can acquire it.
        LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE;
        LET $marker=SELECT * FROM ONLY canonical_interpretations:current;
        IF $gate=NONE OR $marker=NONE OR $expected.storage_incarnation!=$gate.storage_incarnation OR $expected.storage_incarnation!=$marker.payload_incarnation OR $expected.issuing_store!=$marker.issuing_store OR $expected.drained OR $expected.pid=0dec OR string::len($expected.key)!=36 OR string::len($expected.boot)=0 OR string::len($expected.start)=0 OR string::len($expected.pid_namespace)=0 { THROW 'PSE_REFUSAL:file_pending_owner_drain_original_identity_unavailable'; };
        CREATE ONLY type::record('canonical_file_owners',$expected.key) CONTENT object::extend($expected,{drained:true});
    } ELSE {
        IF object::remove($owner,['id','drained'])!=object::remove($expected,['id','drained']) { THROW 'PSE_REFUSAL:file_owner_drain_exact_identity_differs'; };
        UPDATE ONLY type::record('canonical_file_owners',$expected.key) SET drained=true;
    };
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
"#);
}
