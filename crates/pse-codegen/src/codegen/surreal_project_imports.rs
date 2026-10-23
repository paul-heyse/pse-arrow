// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original import checkpoint and final artifact-owned project availability.
pub(super) fn append(ddl:&mut String) { ddl.push_str(FUNCTIONS); }
const FUNCTIONS:&str=r#"
DEFINE FUNCTION fn::pse_project_import_v1::available($project:string) -> bool {
    LET $pending=SELECT key FROM canonical_project_imports WITH INDEX project_import_project WHERE project=$project AND complete=false LIMIT 1;
    RETURN array::len($pending)=0;
};
DEFINE FUNCTION fn::pse_project_import_v1::bind($expiry:int,$key:string,$request:bytes,$artifact:object,$bytes:bytes,$provenance:bytes) -> object {
    fn::pse_execution_v1::deadline($expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_project_imports',$key) FOR UPDATE;
    IF $row=NONE OR $row.request!=$request {THROW 'PSE_REFUSAL:project_import_intent_differs';};
    IF $row.artifact!=NONE {IF $row.artifact!=$bytes {THROW 'PSE_REFUSAL:project_import_artifact_differs';}; RETURN $row;};
    IF $row.complete OR $artifact.kind!='project-import-evidence' OR $artifact.revision!=$row.revision OR $artifact.primary_problem!=$row.problem OR $artifact.active OR $artifact.retiring OR $artifact.provenance!=$provenance OR bytes::len($bytes)>131072 {THROW 'PSE_REFUSAL:project_import_artifact_differs';};
    RETURN UPDATE ONLY type::record('canonical_project_imports',$key) SET artifact=$bytes;
};
DEFINE FUNCTION fn::pse_project_import_v1::retire_artifact($expiry:int,$key:string,$request:bytes,$original:object,$bytes:bytes,$previous:option<bytes>,$next:bytes) -> object {
    fn::pse_execution_v1::deadline($expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_project_imports',$key) FOR UPDATE;
    IF $row=NONE OR $row.request!=$request {THROW 'PSE_REFUSAL:project_import_intent_differs';};
    IF $row.retired_artifacts=$next {RETURN $row;};
    IF $row.complete OR $row.artifact!=$bytes OR ($row.retired_artifacts ?? NONE)!=$previous OR bytes::len($next)>131072 {THROW 'PSE_REFUSAL:project_import_artifact_differs';};
    LET $actual=SELECT * FROM ONLY type::record('canonical_artifacts',$original.key) FOR UPDATE;
    IF $actual!=NONE AND $actual.active {THROW 'PSE_REFUSAL:project_import_artifact_differs';};
    LET $settled=fn::pse_artifact_v1::collect($expiry,$original);
    IF !$settled.complete {THROW 'PSE_REFUSAL:project_import_artifact_unavailable';};
    RETURN UPDATE ONLY type::record('canonical_project_imports',$key) SET artifact=NONE,retired_artifacts=$next;
};
DEFINE FUNCTION fn::pse_project_import_v1::finish($expiry:int,$key:string,$request:bytes,$artifact:object,$bytes:bytes,$obligation:object) -> object {
    fn::pse_execution_v1::deadline($expiry);
    LET $row=SELECT * FROM ONLY type::record('canonical_project_imports',$key) FOR UPDATE;
    IF $row=NONE OR $row.request!=$request OR $row.artifact=NONE OR $row.artifact!=$bytes {THROW 'PSE_REFUSAL:project_import_intent_differs';};
    IF $row.complete {RETURN $row;};
    LET $branch=SELECT * FROM ONLY type::record('canonical_project_branches',$row.branch) FOR UPDATE;
    LET $head=SELECT * FROM ONLY type::record('canonical_problems',$row.problem) FOR UPDATE;
    IF $branch=NONE OR $branch.project!=$row.project OR $branch.problem!=$row.problem OR $head.head!=$row.revision {THROW 'PSE_REFUSAL:project_import_source_changed';};
    LET $actual=SELECT * FROM ONLY type::record('canonical_artifacts',$artifact.key);
    IF $actual=NONE OR !$actual.active OR object::remove($actual,['id','active'])!=object::remove($artifact,['active']) OR $actual.retiring OR $actual.kind!='project-import-evidence' OR $actual.revision!=$row.revision {THROW 'PSE_REFUSAL:project_import_artifact_unavailable';};
    IF $obligation.artifact!=$actual.key OR $obligation.owner_kind!='project' OR $obligation.owner!=$key OR (SELECT * FROM ONLY type::record('canonical_artifact_obligations',$obligation.key))=NONE {THROW 'PSE_REFUSAL:project_import_artifact_unowned';};
    fn::pse_execution_v1::deadline($expiry);
    RETURN UPDATE ONLY type::record('canonical_project_imports',$key) SET complete=true;
};
"#;
