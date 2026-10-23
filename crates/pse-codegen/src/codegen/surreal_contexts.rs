// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Short native transactions for immutable engineering context history.
pub(super) fn append(ddl: &mut String) { ddl.push_str(FUNCTIONS); }
const FUNCTIONS: &str = r#"
DEFINE FUNCTION fn::pse_context_v1::available($expiry:int,$revision:string) -> bool {
    fn::pse_execution_v1::deadline($expiry);
    LET $r=SELECT * FROM ONLY type::record('canonical_revisions',$revision);
    IF $r=NONE {THROW 'PSE_REFUSAL:project_context_unavailable';};
    LET $lost=SELECT key FROM canonical_reclaimed_ranges WHERE problem=$r.problem AND from_sequence<=$r.sequence AND to_sequence>$r.sequence LIMIT 1;
    IF array::len($lost)!=0 {THROW 'PSE_REFUSAL:project_context_unavailable';}; RETURN true;
};
DEFINE FUNCTION fn::pse_context_v1::append($expiry:int,$row:object,$generation:decimal) -> object {
    fn::pse_execution_v1::deadline($expiry);
    LET $old=SELECT * FROM ONLY type::record('canonical_project_contexts',$row.key);
    IF $old!=NONE { IF $old.request!=$row.request {THROW 'PSE_REFUSAL:project_operation_identity_changed';}; RETURN $old; };
    fn::pse_project_v1::project($expiry,$row.project,$generation);
    LET $p=SELECT * FROM ONLY type::record('canonical_projects',$row.project);
    IF $p.deleted {THROW 'PSE_REFUSAL:project_context_unavailable';};
    IF $row.effective_to_us!=NONE AND $row.effective_to_us<=$row.effective_from_us {THROW 'PSE_REFUSAL:project_context_invalid';};
    IF $row.corrects!=NONE {
        LET $parent=SELECT * FROM ONLY type::record('canonical_project_contexts',$row.corrects);
        IF $parent=NONE OR $parent.project!=$row.project OR $parent.scope!=$row.scope {THROW 'PSE_REFUSAL:project_context_invalid';};
        LET $ancestors=SELECT * FROM canonical_context_ancestry WITH INDEX project_context_ancestor WHERE descendant=$row.corrects LIMIT 257;
        IF array::len($ancestors)>=256 {THROW 'PSE_REFUSAL:project_context_bound';};
        FOR $a IN $ancestors {CREATE ONLY type::record('canonical_context_ancestry',[$row.key,$a.ancestor]) CONTENT {key:$row.key+':'+$a.ancestor,descendant:$row.key,ancestor:$a.ancestor};};
        CREATE ONLY type::record('canonical_context_ancestry',[$row.key,$row.corrects]) CONTENT {key:$row.key+':'+$row.corrects,descendant:$row.key,ancestor:$row.corrects};
    } ELSE {IF $row.deleted {THROW 'PSE_REFUSAL:project_context_invalid';};};
    fn::pse_project_v1::root($expiry,$row.revision,'project-context:'+$row.key);
    LET $journal=SELECT * FROM ONLY type::record('canonical_context_journals',$row.project) FOR UPDATE;
    LET $sequence=($journal.sequence ?? 0dec)+1dec;
    UPSERT ONLY type::record('canonical_context_journals',$row.project) CONTENT {key:$row.project,sequence:$sequence};
    LET $new=CREATE ONLY type::record('canonical_project_contexts',$row.key) CONTENT object::extend($row,{recorded_sequence:$sequence});
    fn::pse_execution_v1::deadline($expiry); RETURN $new;
};
DEFINE FUNCTION fn::pse_context_v1::select($expiry:int,$project:string,$scope:string,$effective:int,$cut:decimal) -> option<object> {
    fn::pse_execution_v1::deadline($expiry);
    LET $journal=SELECT * FROM ONLY type::record('canonical_context_journals',$project);
    IF $cut>($journal.sequence ?? 0dec) {THROW 'PSE_REFUSAL:project_context_invalid';};
    LET $candidates=SELECT * FROM canonical_project_contexts WITH INDEX project_context_scope WHERE project=$project AND scope=$scope AND recorded_sequence<=$cut AND effective_from_us<=$effective AND (effective_to_us=NONE OR effective_to_us>$effective) LIMIT 257;
    IF array::len($candidates)>256 {THROW 'PSE_REFUSAL:project_context_bound';};
    LET $superseded=SELECT VALUE ancestor FROM canonical_context_ancestry WITH INDEX project_context_ancestor WHERE descendant IN $candidates.key LIMIT 65537;
    IF array::len($superseded)>65536 {THROW 'PSE_REFUSAL:project_context_bound';};
    LET $selected=$candidates.filter(|$c| $c.key NOT IN $superseded);
    IF array::len($selected)>1 {THROW 'PSE_REFUSAL:project_context_ambiguous';};
    IF array::len($selected)=0 OR $selected[0].deleted {RETURN NONE;};
    RETURN $selected[0];
};
"#;
