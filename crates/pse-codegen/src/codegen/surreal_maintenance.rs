// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Privileged phase-owned domain mutations. Installation and authentication stay external.
pub(super) fn append(ddl: &mut String) {
    ddl.push_str(r#"
DEFINE FUNCTION fn::pse_maintenance_v1::current($expiry:int) -> option<object> {
    fn::pse_execution_v1::deadline($expiry);
    LET $owner=SELECT * FROM ONLY canonical_maintenance_operations:current;
    RETURN IF $owner=NONE { NONE } ELSE { {operation:$owner.operation,generation:$owner.generation,phase:$owner.phase} };
};
DEFINE FUNCTION fn::pse_maintenance_v1::claim($expiry:int,$operation:string,$generation:string,$phase:string,$source:string,$target:string,$previous:option<object>) -> object {
    fn::pse_execution_v1::deadline($expiry);
    LET $old=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
    IF $old!=NONE AND ($previous=NONE OR $old.generation!=$previous.generation OR $old.operation!=$previous.operation OR $old.phase!=$previous.phase) { THROW 'PSE_REFUSAL:maintenance_generation_changed'; };
    IF $old=NONE AND $previous!=NONE { THROW 'PSE_REFUSAL:maintenance_generation_changed'; };
    IF string::len($operation)=0 OR string::len($generation)=0 OR string::len($phase)=0 OR string::len($source)=0 OR string::len($target)=0 { THROW 'PSE_REFUSAL:maintenance_original_inputs_differ'; };
    LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE;
    IF $gate=NONE OR !$gate.frozen OR !$gate.gc_frozen OR $gate.storage_incarnation NOT IN [$source,$target] { THROW 'PSE_REFUSAL:maintenance_frozen_placement_required'; };
    LET $resume=$old!=NONE AND $old.operation=$operation AND $old.phase=$phase AND $old.source=$source AND $old.target=$target;
    UPSERT canonical_maintenance_operations:current SET key='current',operation=$operation,generation=$generation,phase=$phase,source=$source,target=$target,kind='generation',after='',next=IF $resume { $old.next } ELSE { '' },count=0dec,complete=false;
    fn::pse_execution_v1::deadline($expiry);
    RETURN {operation:$operation,generation:$generation,phase:$phase};
};
DEFINE FUNCTION fn::pse_maintenance_v1::guard($expiry:int,$operation:string,$generation:string,$phase:string) -> object {
    fn::pse_execution_v1::deadline($expiry);
    LET $owner=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
    IF $owner=NONE OR $owner.operation!=$operation OR $owner.generation!=$generation OR $owner.phase!=$phase { THROW 'PSE_REFUSAL:maintenance_generation_changed'; };
    RETURN $owner;
};
DEFINE FUNCTION fn::pse_maintenance_v1::frozen($owner:object) -> object {
    LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE;
    IF $gate=NONE OR !$gate.frozen OR !$gate.gc_frozen OR $gate.storage_incarnation NOT IN [$owner.source,$owner.target] { THROW 'PSE_REFUSAL:maintenance_frozen_placement_required'; };
    RETURN $gate;
};
DEFINE FUNCTION fn::pse_maintenance_v1::unprotected($expiry:int) -> bool {
    LET $pins=SELECT key FROM canonical_protections WHERE !released AND (expires_at>time::micros() OR fn::pse_files_v1::ticket_open($expiry,key)) LIMIT 1;
    IF array::len($pins)!=0 { THROW 'PSE_REFUSAL:maintenance_protected_source'; };
    RETURN true;
};
DEFINE FUNCTION fn::pse_maintenance_v1::page($expiry:int,$operation:string,$generation:string,$phase:string,$kind:string,$after:string) -> object {
    LET $owner=fn::pse_maintenance_v1::guard($expiry,$operation,$generation,$phase);
    fn::pse_maintenance_v1::frozen($owner);
    IF $kind NOT IN ['rotate','rehome'] { THROW 'PSE_REFUSAL:maintenance_original_inputs_differ'; };
    LET $key=$operation+':'+$kind+':'+$after;
    LET $prior=SELECT * FROM ONLY type::record('canonical_maintenance_operations',$key) FOR UPDATE;
    IF $prior!=NONE {
        IF $prior.operation!=$operation OR $prior.phase!=$phase OR $prior.source!=$owner.source OR $prior.target!=$owner.target OR $prior.kind!=$kind OR $prior.after!=$after { THROW 'PSE_REFUSAL:maintenance_original_inputs_differ'; };
        RETURN {count:<int>$prior.count,after:$prior.next,complete:$prior.complete};
    };
    IF $owner.next!=$after { THROW 'PSE_REFUSAL:maintenance_original_inputs_differ'; };
    LET $rows=IF $kind='rotate' {
        fn::pse_maintenance_v1::unprotected($expiry);
        SELECT * FROM canonical_guards WHERE string::starts_with(key,'retention:') AND key>$after ORDER BY key LIMIT 64;
    } ELSE {
        LET $owners=SELECT key FROM canonical_file_owners WHERE !drained LIMIT 1;
        IF array::len($owners)!=0 { THROW 'PSE_REFUSAL:maintenance_protected_source'; };
        SELECT * FROM canonical_file_instances WHERE key>$after ORDER BY key LIMIT 64;
    };
    FOR $row IN $rows {
        IF $kind='rotate' { UPDATE ONLY $row.id SET incarnation=<string>rand::uuid::v4(),generation=generation+1dec RETURN NONE; }
        ELSE {
            IF $row.storage_incarnation NOT IN [$owner.source,$owner.target] { THROW 'PSE_REFUSAL:maintenance_original_inputs_differ'; };
            UPDATE ONLY $row.id SET storage_incarnation=$owner.target RETURN NONE;
        };
    };
    LET $next=array::last($rows.key) ?? $after;
    LET $count=array::len($rows);
    CREATE ONLY type::record('canonical_maintenance_operations',$key) SET key=$key,operation=$operation,generation=$generation,phase=$phase,source=$owner.source,target=$owner.target,kind=$kind,after=$after,next=$next,count=<decimal>$count,complete=$count=0;
    UPDATE ONLY canonical_maintenance_operations:current SET next=$next,kind=$kind,complete=$count=0 RETURN NONE;
    fn::pse_execution_v1::deadline($expiry);
    RETURN {count:$count,after:$next,complete:$count=0};
};
DEFINE FUNCTION fn::pse_maintenance_v1::rehome_complete($expiry:int,$operation:string,$generation:string,$phase:string) -> bool {
    LET $owner=fn::pse_maintenance_v1::guard($expiry,$operation,$generation,$phase);
    fn::pse_maintenance_v1::frozen($owner);
    LET $owners=SELECT key FROM canonical_file_owners WHERE !drained LIMIT 1;
    IF array::len($owners)!=0 { THROW 'PSE_REFUSAL:maintenance_protected_source'; };
    LET $remaining=SELECT key FROM canonical_file_instances WHERE storage_incarnation!=$owner.target LIMIT 1;
    IF array::len($remaining)!=0 { THROW 'PSE_REFUSAL:maintenance_placement_incomplete'; };
    UPDATE ONLY canonical_interpretations:current SET payload_incarnation=$owner.target RETURN NONE;
    UPDATE ONLY canonical_file_admission:current SET storage_incarnation=$owner.target RETURN NONE;
    UPDATE ONLY canonical_maintenance_operations:current SET complete=true RETURN NONE;
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_maintenance_v1::validation_reads($expiry:int,$operation:string,$generation:string,$phase:string,$interpretation:string) -> bool {
    LET $owner=fn::pse_maintenance_v1::guard($expiry,$operation,$generation,$phase);
    fn::pse_maintenance_v1::frozen($owner);
    LET $marker=SELECT * FROM ONLY canonical_interpretations:current;
    IF $marker=NONE OR $marker.payload_incarnation!=$owner.target OR $marker.interpretation!=$interpretation OR $owner.source!=$owner.target { THROW 'PSE_REFUSAL:maintenance_original_inputs_differ'; };
    UPDATE ONLY canonical_file_admission:current SET frozen=false,gc_frozen=true RETURN NONE;
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_maintenance_v1::activate($expiry:int,$predecessor:option<object>,$interpretation:string,$schema_digest:string,$issuing:string,$payload_incarnation:string,$nonce:string,$input_sha256:string,$predecessor_bytes:bytes) -> bool {
    fn::pse_execution_v1::deadline($expiry); LET $old=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE; LET $receipt=SELECT * FROM ONLY type::record('canonical_maintenance_activations',$nonce) FOR UPDATE; IF $receipt!=NONE { THROW 'PSE_REFUSAL:activation_operation_already_exists_settle_exact_receipt'; }; IF ($predecessor=NONE AND $old!=NONE) OR ($predecessor!=NONE AND ($old.interpretation!=$predecessor.interpretation OR $old.schema_digest!=$predecessor.schema_digest OR $old.issuing_store!=$predecessor.issuing_store)) { THROW 'PSE_REFUSAL:activation_predecessor_changed'; }; IF $old!=NONE AND $old.payload_incarnation!=$payload_incarnation { THROW 'PSE_REFUSAL:activation_payload_incarnation_differs'; }; LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE; IF $gate!=NONE AND $gate.storage_incarnation!=$payload_incarnation { THROW 'PSE_REFUSAL:activation_file_placement_differs'; }; IF $gate=NONE { CREATE canonical_file_admission:current SET key='current',storage_incarnation=$payload_incarnation,frozen=false,gc_frozen=false; }; UPSERT canonical_interpretations:current SET key='current', interpretation=$interpretation, schema_digest=$schema_digest, issuing_store=$issuing,payload_incarnation=$payload_incarnation; CREATE ONLY type::record('canonical_maintenance_activations',$nonce) SET key=$nonce,input_sha256=$input_sha256,predecessor=$predecessor_bytes,interpretation=$interpretation,schema_digest=$schema_digest,issuing_store=$issuing; fn::pse_execution_v1::deadline($expiry);
    RETURN true;
};
"#);
}
