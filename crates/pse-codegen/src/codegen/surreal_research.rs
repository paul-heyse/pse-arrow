// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Research reference admission composes original context and source protections.
pub(super) fn append(ddl: &mut String) { ddl.push_str(FUNCTIONS); }
const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_research_v1::context($expiry:int,$row:object) -> bool {
    LET $context=SELECT * FROM ONLY type::record('canonical_project_contexts',$row.context);
    IF $context=NONE OR $context.deleted OR $context.project!=$row.project OR $context.revision!=$row.revision OR $context.physical_context!=$row.physical_context { THROW 'PSE_REFUSAL:research_context_differs'; };
    fn::pse_project_v1::root($expiry,$row.revision,'research:'+$row.key);
    RETURN true;
};
DEFINE FUNCTION fn::pse_research_v1::dataset($expiry:int,$row:object,$generation:decimal) -> object {
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
    LET $old=SELECT * FROM ONLY type::record('canonical_research_selections',$row.key);
    IF $old!=NONE {IF $old.request!=$row.request {THROW 'PSE_REFUSAL:project_operation_identity_changed';}; RETURN $old;};
    fn::pse_project_v1::project($expiry,$row.project,$generation);
    fn::pse_research_v1::context($expiry,$row);
    LET $dataset=SELECT * FROM ONLY type::record('canonical_research_datasets',$row.dataset);
    IF $dataset=NONE OR $dataset.project!=$row.project OR $dataset.revision!=$row.revision OR $dataset.context!=$row.context OR $dataset.physical_context!=$row.physical_context {THROW 'PSE_REFUSAL:research_context_differs';};
    IF bytes::len($row.selection)>131072 OR bytes::len($row.request)>262144 {THROW 'PSE_REFUSAL:research_bound';};
    RETURN CREATE ONLY type::record('canonical_research_selections',$row.key) CONTENT $row;
};
"#;
