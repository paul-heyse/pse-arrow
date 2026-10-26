// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Research reference admission composes original context and source protections.
pub(super) fn append(
    registry: &crate::Registry,
    ddl: &mut String,
) -> Result<(), crate::SchemaError> {
    let relation = registry
        .relation("runtime.fitted_parameter_cells")
        .ok_or_else(|| crate::SchemaError::Codegen {
            language: "surreal",
            reason: "qualified fitted cell relation absent".into(),
        })?;
    ddl.push_str(&FUNCTIONS.replace("/* FITTED_CELLS_ID */", &relation.id.to_string()));
    Ok(())
}
const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_research_v1::context($expiry:int,$row:object) -> bool {
    LET $context=SELECT * FROM ONLY type::record('canonical_project_contexts',$row.context);
    IF $context=NONE OR $context.deleted OR $context.project!=$row.project OR $context.revision!=$row.revision OR $context.physical_context!=$row.physical_context { THROW 'PSE_REFUSAL:research_context_differs'; };
    fn::pse_project_v1::root($expiry,$row.revision,'research:'+$row.key);
    RETURN true;
};
DEFINE FUNCTION fn::pse_research_v1::dataset($expiry:int,$row:object,$generation:decimal) -> object {
    fn::pse_workbench_v1::publication($expiry);
    LET $old=SELECT * FROM ONLY type::record('canonical_research_datasets',$row.key);
    IF $old!=NONE { IF $old.request!=$row.request {THROW 'PSE_REFUSAL:project_operation_identity_changed';}; RETURN $old; };
    fn::pse_project_v1::project($expiry,$row.project,$generation);
    fn::pse_research_v1::context($expiry,$row);
    IF bytes::len($row.members)>131072 OR bytes::len($row.request)>262144 {THROW 'PSE_REFUSAL:research_bound';};
    IF $row.role NOT IN ['raw','curated','derived'] {THROW 'PSE_REFUSAL:research_context_differs';};
    IF $row.parent!=NONE {
        LET $parent=SELECT * FROM ONLY type::record('canonical_research_datasets',$row.parent);
        IF $parent=NONE OR $parent.project!=$row.project OR $parent.physical_context!=$row.physical_context {THROW 'PSE_REFUSAL:research_context_differs';};
    } ELSE { IF $row.role!='raw' {THROW 'PSE_REFUSAL:research_context_differs';}; };
    RETURN CREATE ONLY type::record('canonical_research_datasets',$row.key) CONTENT $row;
};
DEFINE FUNCTION fn::pse_research_v1::selection($expiry:int,$row:object,$generation:decimal) -> object {
    fn::pse_workbench_v1::publication($expiry);
    LET $old=SELECT * FROM ONLY type::record('canonical_research_selections',$row.key);
    IF $old!=NONE {IF $old.request!=$row.request {THROW 'PSE_REFUSAL:project_operation_identity_changed';}; RETURN $old;};
    fn::pse_project_v1::project($expiry,$row.project,$generation);
    fn::pse_research_v1::context($expiry,$row);
    LET $dataset=SELECT * FROM ONLY type::record('canonical_research_datasets',$row.dataset);
    IF $dataset=NONE OR $dataset.project!=$row.project OR $dataset.revision!=$row.revision OR $dataset.context!=$row.context OR $dataset.physical_context!=$row.physical_context {THROW 'PSE_REFUSAL:research_context_differs';};
    IF bytes::len($row.selection)>131072 OR bytes::len($row.request)>262144 {THROW 'PSE_REFUSAL:research_bound';};
    RETURN CREATE ONLY type::record('canonical_research_selections',$row.key) CONTENT $row;
};

DEFINE FUNCTION fn::pse_research_v1::publication_guard($expiry:int,$operation:string,$intent:object) -> bool {
    fn::pse_workbench_v1::publication($expiry);
    IF $intent.research=NONE {RETURN true;};
    LET $e=$intent.research;
    LET $selection=SELECT * FROM ONLY type::record('canonical_research_selections',$e.selection);
    LET $run=SELECT * FROM ONLY type::record('canonical_runs',$e.run);
    LET $attempt=SELECT * FROM ONLY type::record('canonical_attempts',$e.attempt);
    LET $manifest=SELECT * FROM ONLY type::record('canonical_result_manifests',$attempt.closed_manifest);
    LET $set=SELECT * FROM ONLY type::record('canonical_result_sets',$e.result_set);
    IF $selection=NONE OR $selection.project!=$intent.project OR $run=NONE OR $run.revision!=$selection.revision OR $run.source_selection!=$e.source_selection OR $attempt=NONE OR $attempt.run!=$e.run OR $attempt.phase NOT IN ['terminal','terminal_compacted'] OR $manifest=NONE OR $manifest.attempt!=$e.attempt OR $manifest.generation!=$attempt.generation OR $manifest.digest!=$e.manifest_digest OR $set=NONE OR $set.attempt!=$e.attempt OR $set.name!='/* FITTED_CELLS_ID */' OR $set.row_count=0dec {THROW 'PSE_REFUSAL:research_context_differs';};
    IF bytes::len($e.mapping)>131072 OR bytes::len($e.source_selection)>131072 {THROW 'PSE_REFUSAL:research_bound';};
    fn::pse_project_v1::root($expiry,$selection.revision,'research-fit-source:'+$operation);
    fn::pse_history_v1::retain($expiry,'project-research:research-publication:'+$operation,'run',$e.run,'audit',$intent.project);
    RETURN true;
};
DEFINE FUNCTION fn::pse_research_v1::publication_ack($expiry:int,$operation:string,$intent:object) -> bool {
    fn::pse_workbench_v1::publication($expiry);
    IF $intent.research=NONE {RETURN true;};
    LET $e=$intent.research;
    LET $selection=SELECT * FROM ONLY type::record('canonical_research_selections',$e.selection);
    CREATE ONLY type::record('canonical_research_publications',$operation) CONTENT {key:$operation,project:$intent.project,selection:$e.selection,run:$e.run,attempt:$e.attempt,revision:$operation,parent:$intent.expected,mapping:$e.mapping,request:$intent.request};
    LET $reference='research-publication:'+$operation;
    CREATE ONLY type::record('canonical_research_references',$reference) CONTENT {key:$reference,project:$intent.project,revision:$selection.revision,run:$e.run,attempt:$e.attempt,request:$intent.request};
    RETURN true;
};

DEFINE FUNCTION fn::pse_research_v1::run_input($expiry:int,$row:object) -> bool {
    fn::pse_workbench_v1::publication($expiry);
    IF $row.research_selection=NONE {IF $row.research_dataset!=NONE {THROW 'PSE_REFUSAL:research_context_differs';}; RETURN true;};
    LET $selection=SELECT * FROM ONLY type::record('canonical_research_selections',$row.research_selection);
    IF $selection=NONE OR $selection.dataset!=$row.research_dataset {THROW 'PSE_REFUSAL:research_context_differs';};
    IF $selection.revision!=$row.revision {
        LET $publication=SELECT * FROM ONLY type::record('canonical_research_publications',$row.revision);
        IF $publication=NONE OR $publication.selection!=$row.research_selection OR $publication.revision!=$row.revision {THROW 'PSE_REFUSAL:research_context_differs';};
    };
    fn::pse_execution_v1::deadline($expiry);RETURN true;
};
"#;
