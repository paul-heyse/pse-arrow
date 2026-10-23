// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Atomic project guards compose with the sole canonical source activation path.
pub(super) fn append(ddl: &mut String) {
    let maximum = pse_schema::catalog::substrate::SOURCE_PACKAGE_EDITS;
    ddl.push_str(
        &FUNCTIONS
            .replace("/* SOURCE_EDIT_SCAN */", &(maximum + 1).to_string())
            .replace("/* SOURCE_EDIT_LIMIT */", &maximum.to_string()),
    );
}
const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_project_v1::authority($expiry:int,$issuing:string) -> bool {
    fn::pse_execution_v1::deadline($expiry);
    IF (SELECT * FROM ONLY canonical_interpretations:current).issuing_store!=$issuing { THROW 'PSE_REFUSAL:project_issuing_authority_changed'; };
    RETURN true;
};
DEFINE FUNCTION fn::pse_project_v1::ack($expiry:int,$operation:string,$request:bytes) -> option<object> {
    fn::pse_execution_v1::deadline($expiry);
    LET $old=SELECT * FROM ONLY type::record('canonical_project_operations',$operation);
    IF $old!=NONE AND $old.request!=$request { THROW 'PSE_REFUSAL:project_operation_identity_changed'; };
    RETURN $old;
};
DEFINE FUNCTION fn::pse_project_v1::project($expiry:int,$key:string,$generation:decimal) -> object {
    fn::pse_execution_v1::deadline($expiry);
    LET $p=SELECT * FROM ONLY type::record('canonical_projects',$key);
    IF $p=NONE OR $p.deleted OR $p.generation!=$generation { THROW 'PSE_REFUSAL:project_generation_changed'; };
    RETURN $p;
};
DEFINE FUNCTION fn::pse_project_v1::root($expiry:int,$revision:string,$owner:string) -> bool {
    LET $r=SELECT * FROM ONLY type::record('canonical_revisions',$revision);
    IF $r=NONE { THROW 'PSE_REFUSAL:project_source_unavailable'; };
    fn::pse_execution_v1::source_creator($expiry,$r.problem,$owner);
    LET $lost=SELECT key FROM canonical_reclaimed_ranges WHERE problem=$r.problem AND from_sequence<=$r.sequence AND to_sequence>$r.sequence LIMIT 1;
    IF array::len($lost)!=0 { THROW 'PSE_REFUSAL:project_source_unavailable'; };
    UPSERT ONLY type::record('canonical_roots',$owner) CONTENT {key:$owner,problem:$r.problem,revision:$revision,sequence:$r.sequence,owner_kind:'project',owner:$owner};
    RETURN true;
};
DEFINE FUNCTION fn::pse_project_v1::create($expiry:int,$operation:string,$request:bytes,$project:string,$title:string) -> object {
    LET $old=fn::pse_project_v1::ack($expiry,$operation,$request); IF $old!=NONE { RETURN $old; };
    IF $project='' OR string::len($project)>4096 OR string::len($title)>4096 { THROW 'PSE_REFUSAL:project_metadata_bound'; };
    CREATE ONLY type::record('canonical_projects',$project) CONTENT {key:$project,title:$title,deleted:false,generation:0dec};
    LET $ack=CREATE ONLY type::record('canonical_project_operations',$operation) CONTENT {key:$operation,kind:'create',project:$project,request:$request,generation:0dec};
    RETURN $ack;
};
// Called inside EDIT before canonical_problems is changed. All branch writes,
// including ordinary source APIs, must present the matching project guard.
DEFINE FUNCTION fn::pse_project_v1::source_guard($expiry:int,$problem:string,$operation:string,$intent:option<object>) -> bool {
    IF $intent!=NONE {fn::pse_project_v1::authority($expiry,$intent.issuing_store);};
    LET $owners=SELECT * FROM canonical_project_branches WITH INDEX project_branch_problem WHERE problem=$problem LIMIT 2;
    IF $intent=NONE { IF array::len($owners)!=0 { THROW 'PSE_REFUSAL:project_publication_guard_required'; }; RETURN true; };
    IF $intent.kind NOT IN ['fork','publish'] { THROW 'PSE_REFUSAL:project_branch_changed'; };
    LET $stage=SELECT * FROM ONLY type::record('canonical_stages',$operation);
    IF $stage=NONE OR ($stage.expected_head ?? NONE)!=($intent.expected ?? NONE) { THROW 'PSE_REFUSAL:project_branch_changed'; };
    LET $ack=fn::pse_project_v1::ack($expiry,$operation,$intent.request);
    IF $ack!=NONE { THROW 'PSE_REFUSAL:project_operation_already_committed'; };
    fn::pse_project_v1::project($expiry,$intent.project,$intent.project_generation);
    IF $intent.kind='fork' {
        IF array::len($owners)!=0 OR (SELECT * FROM ONLY type::record('canonical_project_branches',$intent.branch))!=NONE { THROW 'PSE_REFUSAL:project_branch_changed'; };
        IF $intent.origin_revision=NONE { THROW 'PSE_REFUSAL:project_source_unavailable'; };
        fn::pse_project_v1::root($expiry,$intent.origin_revision,'project-origin:'+$intent.branch);
        LET $origin=SELECT * FROM ONLY type::record('canonical_revisions',$intent.origin_revision);
        LET $source=SELECT logical,scope,name,version FROM canonical_memberships WHERE problem=$origin.problem AND from_sequence<=$origin.sequence AND (to_sequence=NONE OR to_sequence>$origin.sequence) ORDER BY logical LIMIT /* SOURCE_EDIT_SCAN */;
        LET $changes=SELECT logical,scope,name,version FROM canonical_staged_edits WHERE stage=$operation ORDER BY logical LIMIT /* SOURCE_EDIT_SCAN */;
        IF array::len($source)!=array::len($changes) OR array::len($source)>/* SOURCE_EDIT_LIMIT */ { THROW 'PSE_REFUSAL:project_fork_closure_differs'; };
        IF $source!=$changes { THROW 'PSE_REFUSAL:project_fork_closure_differs'; };
        CREATE ONLY type::record('canonical_project_branches',$intent.branch) CONTENT {key:$intent.branch,project:$intent.project,problem:$problem,name:$intent.branch_name,origin_revision:$intent.origin_revision,generation:0dec};
    } ELSE {
        LET $branch=SELECT * FROM ONLY type::record('canonical_project_branches',$intent.branch) FOR UPDATE;
        IF array::len($owners)!=1 OR $branch=NONE OR $branch.problem!=$problem OR $branch.project!=$intent.project OR $branch.generation!=$intent.branch_generation { THROW 'PSE_REFUSAL:project_branch_changed'; };
        IF $intent.draft!=NONE {
            LET $draft=SELECT * FROM ONLY type::record('canonical_project_drafts',$intent.draft) FOR UPDATE;
            LET $version=SELECT * FROM ONLY type::record('canonical_project_draft_versions',$draft.version);
            IF $draft=NONE OR $draft.deleted OR $draft.branch!=$intent.branch OR $draft.generation!=$intent.draft_generation OR $version.base_revision!=$intent.expected { THROW 'PSE_REFUSAL:project_draft_changed'; };
        };
    };
    RETURN true;
};
DEFINE FUNCTION fn::pse_project_v1::source_ack($expiry:int,$problem:string,$operation:string,$intent:option<object>) -> bool {
    IF $intent=NONE { RETURN true; };
    fn::pse_project_v1::root($expiry,$operation,'project-head:'+$intent.branch);
    FOR $parent IN $intent.extra_parents {
        fn::pse_project_v1::root($expiry,$parent,'project-ancestry:'+$operation+':'+$parent);
        CREATE ONLY type::record('canonical_project_ancestry',[$operation,$parent]) CONTENT {key:$operation+':'+$parent,revision:$operation,parent:$parent};
    };
    UPDATE ONLY type::record('canonical_project_branches',$intent.branch) SET generation+=1dec;
    LET $project=IF $intent.kind='fork' { UPDATE ONLY type::record('canonical_projects',$intent.project) SET generation+=1dec } ELSE { SELECT * FROM ONLY type::record('canonical_projects',$intent.project) };
    CREATE ONLY type::record('canonical_project_operations',$operation) CONTENT {key:$operation,kind:$intent.kind,project:$intent.project,branch:$intent.branch,request:$intent.request,revision:$operation,generation:$project.generation};
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::pse_project_v1::draft($expiry:int,$operation:string,$request:bytes,$row:object,$expected:option<decimal>,$project_generation:decimal,$obligation:option<object>) -> object {
    LET $ack=fn::pse_project_v1::ack($expiry,$operation,$request); IF $ack!=NONE { RETURN $ack; };
    fn::pse_project_v1::project($expiry,$row.project,$project_generation);
    LET $branch=SELECT * FROM ONLY type::record('canonical_project_branches',$row.branch) FOR UPDATE;
    IF $branch=NONE OR $branch.project!=$row.project { THROW 'PSE_REFUSAL:project_branch_changed'; };
    LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$row.base_revision);
    IF $revision=NONE OR $revision.problem!=$branch.problem { THROW 'PSE_REFUSAL:project_source_unavailable'; };
    LET $old=SELECT * FROM ONLY type::record('canonical_project_drafts',$row.draft) FOR UPDATE;
    IF ($old.generation ?? NONE)!=$expected OR ($old!=NONE AND ($old.branch!=$row.branch OR $old.project!=$row.project)) { THROW 'PSE_REFUSAL:project_draft_changed'; };
    IF $row.generation!=($expected ?? 0dec)+1dec OR ($row.artifact!=NONE AND bytes::len($row.content)!=0) { THROW 'PSE_REFUSAL:project_draft_changed'; };
    IF $row.artifact!=NONE {
        LET $artifact=SELECT * FROM ONLY type::record('canonical_artifacts',$row.artifact);
        IF $artifact=NONE OR $artifact.revision!=$row.base_revision OR $artifact.kind!='project-draft' { THROW 'PSE_REFUSAL:project_source_unavailable'; };
        IF $obligation=NONE OR $obligation.artifact!=$row.artifact OR $obligation.owner_kind!='project' OR $obligation.owner!=$row.draft { THROW 'PSE_REFUSAL:project_draft_changed'; };
        fn::pse_artifact_v1::attach($expiry,$row.artifact,$obligation,true);
    };
    fn::pse_project_v1::root($expiry,$row.base_revision,'project-draft-version:'+$row.key);
    CREATE ONLY type::record('canonical_project_draft_versions',$row.key) CONTENT $row;
    UPSERT ONLY type::record('canonical_project_drafts',$row.draft) CONTENT {key:$row.draft,project:$row.project,branch:$row.branch,generation:$row.generation,version:$row.key,deleted:false};
    LET $project=SELECT * FROM ONLY type::record('canonical_projects',$row.project);
    LET $ack=CREATE ONLY type::record('canonical_project_operations',$operation) CONTENT {key:$operation,kind:'draft',project:$row.project,branch:$row.branch,request:$request,generation:$project.generation};
    fn::pse_execution_v1::deadline($expiry); RETURN $ack;
};
DEFINE FUNCTION fn::pse_project_v1::research($expiry:int,$operation:string,$request:bytes,$row:object,$project_generation:decimal,$obligation:option<object>) -> object {
    LET $ack=fn::pse_project_v1::ack($expiry,$operation,$request); IF $ack!=NONE { RETURN $ack; };
    fn::pse_project_v1::project($expiry,$row.project,$project_generation);
    fn::pse_project_v1::root($expiry,$row.revision,'project-research:'+$row.key);
    IF ($row.run=NONE)!=($row.attempt=NONE) { THROW 'PSE_REFUSAL:project_research_reference_differs'; };
    IF $row.run!=NONE {
        LET $attempt=fn::pse_history_v1::read_attempt($expiry,$row.attempt);
        LET $run=fn::pse_history_v1::read_run($expiry,$row.run);
        fn::pse_execution_v1::result_creator($expiry,$row.run);
        IF $attempt=NONE OR $run=NONE OR $attempt.run!=$row.run OR $run.revision!=$row.revision OR $attempt.phase NOT IN ['terminal','terminal_compacted'] OR $attempt.closed_manifest=NONE { THROW 'PSE_REFUSAL:project_research_reference_differs'; };
        LET $manifest=fn::pse_history_v1::read_manifest($expiry,$attempt.closed_manifest);
        IF $manifest=NONE OR $manifest.attempt!=$row.attempt OR $manifest.generation!=$attempt.generation { THROW 'PSE_REFUSAL:project_research_reference_differs'; };
        fn::pse_history_v1::retain($expiry,'project-research:'+$row.key,'run',$row.run,'audit',$row.project);
    };
    IF $row.artifact!=NONE {
        LET $artifact=SELECT * FROM ONLY type::record('canonical_artifacts',$row.artifact);
        IF $artifact=NONE OR $artifact.revision!=$row.revision { THROW 'PSE_REFUSAL:project_research_reference_differs'; };
        IF $obligation=NONE OR $obligation.artifact!=$row.artifact OR $obligation.owner_kind!='project' OR $obligation.owner!=$row.draft { THROW 'PSE_REFUSAL:project_research_reference_differs'; };
        fn::pse_artifact_v1::attach($expiry,$row.artifact,$obligation,true);
    };
    CREATE ONLY type::record('canonical_research_references',$row.key) CONTENT $row;
    LET $project=SELECT * FROM ONLY type::record('canonical_projects',$row.project);
    LET $ack=CREATE ONLY type::record('canonical_project_operations',$operation) CONTENT {key:$operation,kind:'research',project:$row.project,request:$request,generation:$project.generation};
    fn::pse_execution_v1::deadline($expiry); RETURN $ack;
};
DEFINE FUNCTION fn::pse_project_v1::navigate($expiry:int,$operation:string,$request:bytes,$project:string,$generation:decimal,$action:object) -> object {
    LET $old=fn::pse_project_v1::ack($expiry,$operation,$request); IF $old!=NONE {RETURN $old;};
    LET $p=SELECT * FROM ONLY type::record('canonical_projects',$project) FOR UPDATE;
    IF $p=NONE OR $p.generation!=$generation OR ($p.deleted AND $action.kind!='project_visibility') {THROW 'PSE_REFUSAL:project_generation_changed';};
    IF $action.kind='project_title' {UPDATE ONLY type::record('canonical_projects',$project) SET title=$action.name;}
    ELSE IF $action.kind='project_visibility' {UPDATE ONLY type::record('canonical_projects',$project) SET deleted=$action.deleted;}
    ELSE IF $action.kind='branch_name' {
        LET $branch=SELECT * FROM ONLY type::record('canonical_project_branches',$action.key) FOR UPDATE;
        IF $branch=NONE OR $branch.project!=$project {THROW 'PSE_REFUSAL:project_branch_changed';};
        UPDATE ONLY type::record('canonical_project_branches',$action.key) SET name=$action.name,generation+=1dec;
    } ELSE IF $action.kind='bookmark' {
        fn::pse_project_v1::root($expiry,$action.revision,'project-bookmark:'+$action.key);
        CREATE ONLY type::record('canonical_project_bookmarks',$action.key) CONTENT {key:$action.key,project:$project,name:$action.name,revision:$action.revision};
    } ELSE IF $action.kind='savepoint' {
        LET $head=SELECT * FROM ONLY type::record('canonical_project_drafts',$action.draft) FOR UPDATE;
        LET $version=SELECT * FROM ONLY type::record('canonical_project_draft_versions',$action.version);
        IF $head=NONE OR $version=NONE OR $head.project!=$project OR $version.draft!=$head.key OR $head.generation!=$action.generation OR $head.version!=$version.key {THROW 'PSE_REFUSAL:project_draft_changed';};
        CREATE ONLY type::record('canonical_project_savepoints',$action.key) CONTENT {key:$action.key,project:$project,draft:$head.key,version:$version.key,name:$action.name,deleted:false};
    } ELSE IF $action.kind='draft_visibility' {
        LET $head=SELECT * FROM ONLY type::record('canonical_project_drafts',$action.key) FOR UPDATE;
        IF $head=NONE OR $head.project!=$project OR $head.generation!=$action.generation {THROW 'PSE_REFUSAL:project_draft_changed';};
        UPDATE ONLY type::record('canonical_project_drafts',$action.key) SET deleted=$action.deleted,generation+=1dec;
    } ELSE IF $action.kind='savepoint_visibility' {
        LET $point=SELECT * FROM ONLY type::record('canonical_project_savepoints',$action.key);
        IF $point=NONE OR $point.project!=$project {THROW 'PSE_REFUSAL:project_draft_changed';};
        UPDATE ONLY type::record('canonical_project_savepoints',$action.key) SET deleted=$action.deleted;
    } ELSE IF $action.kind='restore' {
        LET $point=SELECT * FROM ONLY type::record('canonical_project_savepoints',$action.key);
        LET $head=SELECT * FROM ONLY type::record('canonical_project_drafts',$action.draft) FOR UPDATE;
        IF $point=NONE OR $head=NONE OR $point.deleted OR $point.project!=$project OR $point.draft!=$head.key OR $head.project!=$project OR $head.generation!=$action.generation {THROW 'PSE_REFUSAL:project_draft_changed';};
        UPDATE ONLY type::record('canonical_project_drafts',$head.key) SET version=$point.version,deleted=false,generation+=1dec;
    } ELSE {THROW 'PSE_REFUSAL:project_metadata_bound';};
    LET $new=UPDATE ONLY type::record('canonical_projects',$project) SET generation+=1dec;
    LET $ack=CREATE ONLY type::record('canonical_project_operations',$operation) CONTENT {key:$operation,kind:'navigate',project:$project,request:$request,generation:$new.generation};
    fn::pse_execution_v1::deadline($expiry);RETURN $ack;
};
DEFINE FUNCTION fn::pse_project_v1::diagnostic($expiry:int,$operation:string,$version:string,$message:string,$admitted:bool) -> object {
    IF string::len($message)>65536 {THROW 'PSE_REFUSAL:project_metadata_bound';};
    LET $v=SELECT * FROM ONLY type::record('canonical_project_draft_versions',$version);
    IF $v=NONE {THROW 'PSE_REFUSAL:project_draft_changed';};
    LET $old=SELECT * FROM ONLY type::record('canonical_draft_diagnostics',$operation);
    IF $old!=NONE {IF $old.version!=$version OR $old.message!=$message OR $old.admitted!=$admitted {THROW 'PSE_REFUSAL:project_operation_identity_changed';};RETURN $old;};
    fn::pse_execution_v1::deadline($expiry);
    RETURN CREATE ONLY type::record('canonical_draft_diagnostics',$operation) CONTENT {key:$operation,version:$version,message:$message,admitted:$admitted};
};
"#;
