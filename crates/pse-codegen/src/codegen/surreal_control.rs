// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authenticated application ingress; all database transitions retain their existing owners.
pub(super) fn append(ddl: &mut String) {
    ddl.push_str(FUNCTIONS);
}
const FUNCTIONS: &str = r#"
DEFINE ACCESS pse_control ON DATABASE TYPE RECORD
    SIGNIN (SELECT * FROM canonical_control_actors WHERE key=$actor AND enabled=true AND crypto::argon2::compare(password,$password))
    DURATION FOR TOKEN 1h, FOR SESSION 1h;

// These functions are callable by privileged internal owners and by the API's
// permission-limited handler. Ordinary record users cannot call them directly.
DEFINE FUNCTION fn::pse_control_v1::actor($grant:string) -> option<string> {
    IF $auth=NONE OR $auth.id=NONE OR record::tb($auth.id)!='canonical_control_actors' { RETURN NONE; };
    LET $actor=SELECT * FROM ONLY $auth.id;
    IF $actor=NONE OR !$actor.enabled OR ($grant='browse' AND !$actor.browse) OR ($grant='cancel' AND !$actor.cancel) OR $grant NOT IN ['browse','cancel'] { RETURN NONE; };
    RETURN <string>$actor.id;
} PERMISSIONS NONE;
// The legacy API JSON parser promotes UUID/date/record-looking JSON strings.
// Decode the declared JSON wire with the library's RFC 8259 parser instead;
// bytes-to-string casting rejects invalid UTF-8 before decoding.
DEFINE FUNCTION fn::pse_control_v1::json($request:object) -> object {
    IF $request.headers['content-type']!='application/json' OR !type::is_bytes($request.body) { THROW 'PSE_REFUSAL:control_request_invalid'; };
    RETURN encoding::json::decode(<string>$request.body);
} PERMISSIONS NONE;
DEFINE FUNCTION fn::pse_control_v1::expiry($body:object) -> int {
    IF array::sort(object::keys($body))!=['expires_at','issuing_store','request'] OR !type::is_int($body.expires_at) OR !type::is_object($body.request) { THROW 'PSE_REFUSAL:control_request_invalid'; };
    IF $body.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:control_store_differs'; };
    LET $expiry=math::min([$body.expires_at,time::micros()+30000000]);
    fn::pse_execution_v1::deadline($expiry); RETURN $expiry;
} PERMISSIONS NONE;
DEFINE FUNCTION fn::pse_control_v1::key($actor:string,$operation:string) -> string {
    LET $store=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store;
    RETURN encoding::base64::encode(encoding::cbor::encode([$store,$actor,$operation]));
} PERMISSIONS NONE;
DEFINE FUNCTION fn::pse_control_v1::cancel($expiry:int,$actor:string,$request:object) -> object {
    fn::pse_execution_v1::deadline($expiry);
    IF array::sort(object::keys($request))!=['document_version','issuing_store','operation_id','study'] OR $request.document_version!=1 OR !type::is_string($request.operation_id) OR bytes::len(<bytes>$request.operation_id)<1 OR bytes::len(<bytes>$request.operation_id)>128 OR !type::is_string($request.study) OR bytes::len(<bytes>$request.study)<1 OR bytes::len(<bytes>$request.study)>4096 { THROW 'PSE_REFUSAL:control_request_invalid'; };
    IF $request.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:control_store_differs'; };
    LET $key=fn::pse_control_v1::key($actor,$request.operation_id);
    LET $old=SELECT * FROM ONLY type::record('canonical_control_commands',$key) FOR UPDATE;
    IF $old!=NONE {
        IF $old.actor!=$actor OR $old.operation_id!=$request.operation_id OR $old.study!=$request.study { fn::pse_execution_v1::deadline($expiry); RETURN {document_version:1,issuing_store:$request.issuing_store,operation_id:$request.operation_id,reason:'control_command_identity_reused'}; };
        fn::pse_execution_v1::deadline($expiry); RETURN encoding::cbor::decode($old.receipt);
    };
    fn::pse_schema_v1::current_mutation($expiry);
    fn::pse_workbench_v1::publication($expiry);
    fn::pse_history_v1::check_owner($expiry,'study',$request.study);
    fn::pse_execution_v1::touch($expiry,'study-cancel:'+$request.study);
    LET $study=SELECT * FROM ONLY type::record('canonical_studies',$request.study) FOR UPDATE;
    LET $state=IF $study=NONE OR $study.terminal {$study} ELSE {fn::pse_study_v1::cancel($expiry,$request.study)};
    LET $receipt={document_version:1,issuing_store:$request.issuing_store,operation_id:$request.operation_id,study:$request.study,status:IF $state=NONE {'refused'} ELSE {'accepted'},generation:IF $state=NONE {NULL} ELSE {<string>$state.generation},already_concluded:IF $state=NONE {false} ELSE {$state.terminal},cancelled:IF $state=NONE {false} ELSE {$state.cancelled},reason:IF $state=NONE {'study_unavailable_or_already_concluded'} ELSE {NULL}};
    CREATE ONLY type::record('canonical_control_commands',$key) SET key=$key,issuing_store=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store,actor=$actor,operation_id=$request.operation_id,study=$request.study,receipt=encoding::cbor::encode($receipt);
    fn::pse_execution_v1::deadline($expiry); RETURN $receipt;
} PERMISSIONS NONE;
DEFINE FUNCTION fn::pse_control_v1::reconcile($expiry:int,$actor:string,$request:object) -> object {
    IF array::sort(object::keys($request))!=['document_version','issuing_store','operation_id'] OR $request.document_version!=1 OR !type::is_string($request.operation_id) OR bytes::len(<bytes>$request.operation_id)<1 OR bytes::len(<bytes>$request.operation_id)>128 { THROW 'PSE_REFUSAL:control_request_invalid'; };
    IF $request.issuing_store!=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store { THROW 'PSE_REFUSAL:control_store_differs'; };
    LET $key=fn::pse_control_v1::key($actor,$request.operation_id);
    LET $old=SELECT * FROM ONLY type::record('canonical_control_commands',$key);
    fn::pse_execution_v1::deadline($expiry); RETURN {document_version:1,receipt:IF $old=NONE {NULL} ELSE {encoding::cbor::decode($old.receipt)}};
} PERMISSIONS NONE;
DEFINE FUNCTION fn::pse_control_v1::browse($expiry:int,$request:object) -> object {
    IF array::sort(object::keys($request))!=['cursor','document_version','filter','maximum'] OR $request.document_version!=1 OR !type::is_int($request.maximum) OR $request.maximum<1 OR $request.maximum>64 { THROW 'PSE_REFUSAL:control_request_invalid'; };
    LET $store=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store;
    IF $request.cursor NOT IN [NULL,NONE] {
        IF !type::is_object($request.cursor) { THROW 'PSE_REFUSAL:control_request_invalid'; };
        IF array::sort(object::keys($request.cursor))!=['after','filter','issuing_store'] OR !type::is_string($request.cursor.after) { THROW 'PSE_REFUSAL:control_request_invalid'; };
        IF bytes::len(<bytes>$request.cursor.after)<1 OR bytes::len(<bytes>$request.cursor.after)>4096 OR $request.cursor.issuing_store!=$store OR $request.cursor.filter!=$request.filter { THROW 'PSE_REFUSAL:control_request_invalid'; };
    };
    LET $after=IF $request.cursor=NULL OR $request.cursor=NONE {''} ELSE {$request.cursor.after};
    LET $rows=fn::pse_workbench_v1::select_rows($expiry,$request.filter,$after,$request.maximum+1);
    LET $selected=fn::pse_workbench_v1::browse($expiry,$rows,$request.maximum);
    LET $headers=$selected.headers.map(|$s| {study:$s.key,run:$s.run,problem:$s.problem,source:$s.revision,physical_revision:$s.physical_revision ?? NULL,physical_identity:$s.physical_identity ?? NULL,context:$s.context ?? NULL,point_count:<string>$s.point_count,generation:<string>$s.generation,active:$s.active,cancelled:$s.cancelled,terminal:$s.terminal,result_attempt:$s.result_attempt ?? NULL,outcome:$s.outcome ?? NULL});
    LET $next=IF $selected.has_more {{issuing_store:$store,filter:$request.filter,after:array::last($headers).study}} ELSE {NULL};
    fn::pse_execution_v1::deadline($expiry); RETURN {document_version:1,headers:$headers,next:$next};
} PERMISSIONS NONE;
DEFINE API '/pse/control/v1/browse' FOR post
    PERMISSIONS WHERE $auth!=NONE
    MIDDLEWARE api::req::max_body(65536),api::res::body('json')
    THEN { LET $actor=fn::pse_control_v1::actor('browse'); IF $actor=NONE { RETURN {status:403,body:{error:'control_actor_refused'}}; }; LET $body=fn::pse_control_v1::json($request); LET $expiry=fn::pse_control_v1::expiry($body); RETURN {status:200,body:fn::pse_control_v1::browse($expiry,$body.request)}; };
DEFINE API '/pse/control/v1/cancel' FOR post
    PERMISSIONS WHERE $auth!=NONE
    MIDDLEWARE api::req::max_body(65536),api::res::body('json')
    THEN { LET $actor=fn::pse_control_v1::actor('cancel'); IF $actor=NONE { RETURN {status:403,body:{error:'control_actor_refused'}}; }; LET $body=fn::pse_control_v1::json($request); LET $expiry=fn::pse_control_v1::expiry($body); LET $reply=fn::pse_control_v1::cancel($expiry,$actor,$body.request); RETURN {status:IF $reply.status=NONE {409} ELSE {200},body:$reply}; };
DEFINE API '/pse/control/v1/reconcile' FOR post
    PERMISSIONS WHERE $auth!=NONE
    MIDDLEWARE api::req::max_body(65536),api::res::body('json')
    THEN { LET $actor=fn::pse_control_v1::actor('cancel'); IF $actor=NONE { RETURN {status:403,body:{error:'control_actor_refused'}}; }; LET $body=fn::pse_control_v1::json($request); LET $expiry=fn::pse_control_v1::expiry($body); RETURN {status:200,body:fn::pse_control_v1::reconcile($expiry,$actor,$body.request)}; };
"#;
