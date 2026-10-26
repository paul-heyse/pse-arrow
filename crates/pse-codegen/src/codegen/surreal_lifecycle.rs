// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Lifecycle composition delegates every destructive effect to its original owner.
pub(super) fn functions() -> &'static str {
    r#"
DEFINE FUNCTION fn::pse_retention_v1::drop_source_root($expiry:int,$problem:string,$revision:string,$root:string,$owner_kind:string,$owner:string) -> bool {
IF $owner_kind IN ['run','analysis','active_attempt'] { THROW 'PSE_REFUSAL:lifecycle_roots_are_released_only_by_their_owning_retirement'; };
LET $guard = type::record('canonical_guards', 'retention:' + $problem);
SELECT * FROM $guard FOR UPDATE; fn::pse_execution_v1::deadline($expiry);
LET $old = SELECT * FROM ONLY type::record('canonical_roots', $root) FOR UPDATE;
IF $old != NONE {
    IF $old.problem != $problem OR $old.revision != $revision OR $old.owner_kind != $owner_kind OR $old.owner != $owner { THROW 'PSE_REFUSAL:retained_owner_selection_changed'; };
    DELETE ONLY type::record('canonical_roots', $root);
    IF $owner_kind = 'product' {
        LET $association = SELECT * FROM ONLY type::record('canonical_product_associations',$root);
        IF $association != NONE {
            LET $descriptor = SELECT * FROM ONLY type::record('canonical_product_descriptors',$association.descriptor) FOR UPDATE;
            IF $descriptor != NONE {
                IF $descriptor.issuing_store != $association.issuing_store OR $descriptor.problem != $association.problem OR $descriptor.request != $association.request OR $descriptor.payload != $association.payload OR $descriptor.dependencies != $association.dependencies OR $descriptor.content != $association.content OR $descriptor.receipt != $association.receipt OR $descriptor.producer != $association.producer OR $descriptor.interpretation != $association.interpretation { THROW 'PSE_REFUSAL:released_association_descriptor_equality_differs'; };
                LET $remaining = SELECT key FROM canonical_product_associations WHERE descriptor = $association.descriptor AND issuing_store = $association.issuing_store AND problem = $association.problem AND request = $association.request AND payload = $association.payload AND dependencies = $association.dependencies AND content = $association.content AND receipt = $association.receipt AND producer = $association.producer AND interpretation = $association.interpretation AND type::record('canonical_roots',key).problem = problem AND type::record('canonical_roots',key).revision = revision AND type::record('canonical_roots',key).owner_kind = 'product' AND type::record('canonical_roots',key).owner = key ORDER BY key LIMIT 1;
                UPDATE type::record('canonical_product_descriptors',$association.descriptor) SET association = IF array::len($remaining) = 0 { NONE } ELSE { $remaining[0].key };
            };
        };
    };
};
fn::pse_execution_v1::touch($expiry,'source-creators:'+$problem);
fn::pse_execution_v1::deadline($expiry);
RETURN true;
};
DEFINE FUNCTION fn::pse_lifecycle_v1::selection($expiry:int,$issuer:string,$kind:string,$key:string) -> object {
    fn::pse_project_v1::authority($expiry,$issuer);
    IF $key='' OR string::len($key)>4096 { THROW 'PSE_REFUSAL:project_metadata_bound'; };
    LET $row=IF $kind='source_history' { SELECT * FROM ONLY type::record('canonical_revisions',$key) }
        ELSE IF $kind IN ['run_results','run_history'] { SELECT * FROM ONLY type::record('canonical_runs',$key) }
        ELSE IF $kind IN ['study_results','study_history'] { SELECT * FROM ONLY type::record('canonical_studies',$key) }
        ELSE IF $kind='evidence_artifact' { SELECT * FROM ONLY type::record('canonical_artifacts',$key) }
        ELSE IF $kind='draft_version' { SELECT * FROM ONLY type::record('canonical_project_draft_versions',$key) }
        ELSE { THROW 'PSE_REFUSAL:project_metadata_bound'; };
    LET $retirement=IF $kind IN ['run_history','study_history'] {SELECT * FROM ONLY type::record('canonical_history_retirements',IF $kind='run_history' {'run:'+$key} ELSE {'study:'+$key})} ELSE {NONE};
    IF $row=NONE AND $retirement=NONE { THROW 'PSE_REFUSAL:project_source_unavailable'; };
    IF $retirement!=NONE AND $retirement.issuing_store!=$issuer {THROW 'PSE_REFUSAL:project_issuing_authority_changed';};
    LET $project=IF $kind='draft_version' {SELECT * FROM ONLY type::record('canonical_projects',$row.project)} ELSE {NONE};
    LET $draft=IF $kind='draft_version' {SELECT * FROM ONLY type::record('canonical_project_drafts',$row.draft)} ELSE {NONE};
    LET $revision=IF $kind='source_history' {$key} ELSE IF $kind='draft_version' {$row.base_revision} ELSE {$row.revision ?? $retirement.revision};
    LET $generation=IF $kind='source_history' {$row.sequence} ELSE IF $kind IN ['run_results','run_history'] {$row.current_generation ?? $retirement.generation} ELSE IF $kind='evidence_artifact' {$row.creation_expires_at} ELSE {$row.generation ?? $retirement.generation};
    RETURN {issuing_store:$issuer,kind:$kind,key:$key,revision:$revision,generation:$generation,project_generation:$project.generation ?? NONE,draft_generation:$draft.generation ?? NONE};
};
DEFINE FUNCTION fn::pse_lifecycle_v1::check($expiry:int,$expected:object) -> object {
    LET $current=fn::pse_lifecycle_v1::selection($expiry,$expected.issuing_store,$expected.kind,$expected.key);
    IF $current!=$expected { THROW 'PSE_REFUSAL:project_generation_changed'; };
    RETURN $current;
};
DEFINE FUNCTION fn::pse_lifecycle_v1::explain($expiry:int,$selection:object) -> object {
    LET $s=fn::pse_lifecycle_v1::check($expiry,$selection);
    LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$s.revision);
    LET $roots=SELECT key,owner_kind FROM canonical_roots WHERE revision=$s.revision AND owner_kind!='history' ORDER BY key LIMIT 65;
    LET $pins=SELECT key FROM canonical_protections WHERE revision=$s.revision AND released=false AND (expires_at>time::micros() OR fn::pse_files_v1::ticket_open($expiry,key)) ORDER BY key LIMIT 65;
    LET $stages=SELECT key FROM canonical_stages WHERE problem=$revision.problem AND activated=false AND abandoned=false ORDER BY key LIMIT 65;
    LET $project_refs=SELECT key FROM canonical_research_references WHERE revision=$s.revision ORDER BY key LIMIT 65;
    LET $imports=SELECT key FROM canonical_project_imports WHERE complete=false AND (revision=$s.revision OR ($s.kind='draft_version' AND project=type::record('canonical_project_draft_versions',$s.key).project)) ORDER BY key LIMIT 65;
    LET $borrowers=IF $s.kind IN ['run_results','run_history'] {SELECT key FROM canonical_result_protections WHERE run=$s.key AND type::record('canonical_protections',key).released=false AND (type::record('canonical_protections',key).expires_at>time::micros() OR fn::pse_files_v1::ticket_open($expiry,key)) ORDER BY key LIMIT 65} ELSE {[]};
    LET $attempts=IF $s.kind IN ['run_results','run_history'] {SELECT key FROM canonical_attempts WHERE run=$s.key AND phase NOT IN ['terminal','terminal_compacted'] ORDER BY key LIMIT 65} ELSE {[]};
    LET $studies=IF $s.kind IN ['run_results','run_history'] {SELECT key FROM canonical_studies WHERE run=$s.key AND (SELECT * FROM ONLY type::record('canonical_study_retirements',key))=NONE ORDER BY key LIMIT 65} ELSE {[]};
    LET $members=IF $s.kind IN ['run_results','run_history'] {SELECT key FROM canonical_study_points WHERE run=$s.key AND (SELECT * FROM ONLY type::record('canonical_study_retirements',study))=NONE ORDER BY key LIMIT 65} ELSE {[]};
    LET $transfers=IF $s.kind IN ['run_results','run_history'] {SELECT key FROM canonical_artifact_staging_inputs WHERE run=$s.key ORDER BY key LIMIT 65} ELSE {[]};
    LET $obligations=IF $s.kind IN ['run_history','study_history'] {SELECT key FROM canonical_history_obligations WHERE owner=$s.key AND owner_kind=IF $s.kind='run_history' {'run'} ELSE {'study'} ORDER BY key LIMIT 65} ELSE {[]};
    LET $evidence=IF $s.kind='evidence_artifact' {SELECT * FROM ONLY type::record('canonical_artifacts',$s.key)} ELSE {NONE};
    LET $evidence_owners=IF $evidence!=NONE {SELECT key FROM canonical_artifact_obligations WHERE artifact=$s.key ORDER BY key LIMIT 65} ELSE {[]};
    LET $evidence_reads=IF $evidence!=NONE {SELECT key FROM canonical_artifact_reads WHERE artifact=$s.key AND type::record('canonical_protections',key).released=false AND (type::record('canonical_protections',key).expires_at>time::micros() OR fn::pse_files_v1::ticket_open($expiry,key)) ORDER BY key LIMIT 65} ELSE {[]};
    LET $draft=IF $s.kind='draft_version' {SELECT * FROM ONLY type::record('canonical_project_draft_versions',$s.key)} ELSE {NONE};
    LET $head=IF $draft!=NONE {SELECT * FROM ONLY type::record('canonical_project_drafts',$draft.draft)} ELSE {NONE};
    LET $savepoints=IF $draft!=NONE {SELECT key FROM canonical_project_savepoints WHERE version=$s.key ORDER BY key LIMIT 65} ELSE {[]};
    LET $other_versions=IF $draft.artifact!=NONE {SELECT key FROM canonical_project_draft_versions WHERE artifact=$draft.artifact AND key!=$s.key AND retired=false ORDER BY key LIMIT 65} ELSE {[]};
    LET $artifact_reads=IF $draft.artifact!=NONE {SELECT key FROM canonical_artifact_reads WHERE artifact=$draft.artifact AND type::record('canonical_protections',key).released=false AND (type::record('canonical_protections',key).expires_at>time::micros() OR fn::pse_files_v1::ticket_open($expiry,key)) ORDER BY key LIMIT 65} ELSE {[]};
    LET $captures=SELECT key FROM canonical_study_capture_protections
        WHERE (revision=$s.revision AND $s.kind='source_history')
            OR (run=$s.key AND $s.kind IN ['run_results','run_history'])
            OR ($s.kind IN ['study_results','study_history'] AND study=$s.key)
        ORDER BY key LIMIT 65;
    LET $blockers=[
        IF $s.kind='source_history' {$roots.map(|$r| {kind:'retained_source_owner',identity:$r.key})} ELSE {[]},
        $pins.map(|$r| {kind:'active_source_protection',identity:$r.key}),
        $stages.map(|$r| {kind:'unsettled_source_stage',identity:$r.key}),
        $imports.map(|$r| {kind:'incomplete_project_import',identity:$r.key}),
        IF $s.kind IN ['source_history','run_results','run_history'] {$project_refs.map(|$r| {kind:'project_research_reference',identity:$r.key})} ELSE {[]},
        $borrowers.map(|$r| {kind:'active_result_borrower',identity:$r.key}),
        $captures.map(|$r| {kind:'active_result_borrower',identity:$r.key}),
        $attempts.map(|$r| {kind:'unsettled_attempt',identity:$r.key}),
        $studies.map(|$r| {kind:'retained_study',identity:$r.key}),
        $members.map(|$r| {kind:'retained_study_point',identity:$r.key}),
        $transfers.map(|$r| {kind:'unsettled_artifact_transfer',identity:$r.key}),
        $obligations.map(|$r| {kind:'history_obligation',identity:$r.key}),
        $savepoints.map(|$r| {kind:'retained_savepoint',identity:$r.key}),
        $other_versions.map(|$r| {kind:'shared_draft_artifact',identity:$r.key}),
        $artifact_reads.map(|$r| {kind:'active_artifact_borrower',identity:$r.key}),
        $evidence_reads.map(|$r| {kind:'active_artifact_borrower',identity:$r.key}),
        $evidence_owners.map(|$r| {kind:'retained_artifact_obligation',identity:$r.key}),
        IF $evidence!=NONE AND $evidence.creation_expires_at>time::micros() {[{kind:'original_creation_window_unsettled',identity:$s.key}]} ELSE {[]},
        IF $head!=NONE AND $head.version=$s.key {[{kind:'current_draft_head',identity:$head.key}]} ELSE {[]}].fold([],|$all,$rows|array::concat($all,$rows));
    LET $batches=IF $s.kind='run_results' {SELECT payload_bytes FROM canonical_result_batches WHERE type::record('canonical_attempts',attempt).run=$s.key LIMIT 65} ELSE {[]};
    LET $logical=IF $draft!=NONE {IF $draft.retired {0dec} ELSE IF $draft.artifact=NONE {<decimal>bytes::len($draft.content)} ELSE {NONE}}
        ELSE IF $evidence!=NONE {$evidence.payload_bytes}
        ELSE IF $s.kind='run_results' AND array::len($batches)<65 {<decimal>math::sum($batches.payload_bytes)} ELSE {NONE};
    LET $retired=IF $s.kind IN ['run_history','study_history'] {SELECT * FROM ONLY type::record('canonical_history_retirements',IF $s.kind='run_history' {'run:'+$s.key} ELSE {'study:'+$s.key})} ELSE IF $s.kind='run_results' {SELECT * FROM ONLY type::record('canonical_result_retirements',$s.key)} ELSE IF $s.kind='study_results' {SELECT * FROM ONLY type::record('canonical_study_retirements',$s.key)} ELSE {NONE};
    RETURN {selection:$s,blockers:$blockers,blockers_complete:false,logical_bytes:$logical,detail_released:$draft.retired ?? $retired.complete ?? false};
};
DEFINE FUNCTION fn::pse_project_v1::retire_draft_version($expiry:int,$s:object) -> object {
    fn::pse_workbench_v1::publication($expiry);
    fn::pse_lifecycle_v1::check($expiry,$s);
    LET $v=SELECT * FROM ONLY type::record('canonical_project_draft_versions',$s.key) FOR UPDATE;
    LET $p=SELECT * FROM ONLY type::record('canonical_projects',$v.project) FOR UPDATE;
    LET $head=SELECT * FROM ONLY type::record('canonical_project_drafts',$v.draft) FOR UPDATE;
    IF $p.generation!=$s.project_generation OR $head.generation!=$s.draft_generation OR $v.generation!=$s.generation {THROW 'PSE_REFUSAL:project_generation_changed';};
    IF $v.retired {RETURN {complete:true};};
    LET $explanation=fn::pse_lifecycle_v1::explain($expiry,$s);
    IF array::len($explanation.blockers)!=0 {THROW 'PSE_REFUSAL:project_draft_changed';};
    LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$v.base_revision);
    fn::pse_execution_v1::source_creator($expiry,$revision.problem,'project-draft-version:'+$v.key);
    LET $root=SELECT * FROM ONLY type::record('canonical_roots','project-draft-version:'+$v.key) FOR UPDATE;
    IF $root=NONE OR $root.problem!=$revision.problem OR $root.revision!=$v.base_revision OR $root.owner_kind!='project' OR $root.owner!='project-draft-version:'+$v.key {THROW 'PSE_REFUSAL:retained_owner_selection_changed';};
    IF $v.artifact!=NONE {
        LET $artifact=SELECT * FROM ONLY type::record('canonical_artifacts',$v.artifact) FOR UPDATE;
        IF $artifact=NONE OR $artifact.kind!='project-draft' OR $artifact.revision!=$v.base_revision OR !$artifact.active OR $artifact.retiring {THROW 'PSE_REFUSAL:project_source_unavailable';};
        LET $owners=SELECT * FROM canonical_artifact_obligations WHERE artifact=$v.artifact AND owner_kind='project' AND owner=$v.draft LIMIT 65;
        IF array::len($owners)!=1 {THROW 'PSE_REFUSAL:project_draft_changed';};
        FOR $owner IN $owners {fn::pse_artifact_v1::release($expiry,object::remove($owner,['id']));};
    };
    DELETE ONLY type::record('canonical_roots','project-draft-version:'+$v.key);
    UPDATE ONLY type::record('canonical_project_draft_versions',$v.key) SET retired=true,content=encoding::base64::decode('');
    fn::pse_execution_v1::deadline($expiry); RETURN {complete:true};
};
DEFINE FUNCTION fn::pse_lifecycle_v1::release($expiry:int,$s:object) -> object {
    fn::pse_workbench_v1::publication($expiry);
    fn::pse_lifecycle_v1::check($expiry,$s);
    // Explanation is advisory. The original owner checks live guards in this
    // same effect transaction; no prior explanation is a deletion capability.
    LET $revision=SELECT * FROM ONLY type::record('canonical_revisions',$s.revision);
    LET $pending_import=SELECT key FROM canonical_project_imports WHERE complete=false AND (revision=$s.revision OR ($s.kind='draft_version' AND project=type::record('canonical_project_draft_versions',$s.key).project)) LIMIT 1;
    IF array::len($pending_import)!=0 {THROW 'PSE_REFUSAL:project_source_unavailable';};
    IF $s.kind='run_results' {
        LET $retirement=fn::pse_retention_v1::forget_run($expiry,$s.key);
        IF $retirement.pending ?? false {RETURN {complete:false};};
        RETURN {complete:(fn::pse_retention_v1::collect_run($expiry,$s.key)).complete};
    } ELSE IF $s.kind='study_results' {RETURN {complete:fn::pse_retention_v1::forget_study($expiry,$s.key)};}
    ELSE IF $s.kind IN ['run_history','study_history'] {
        LET $kind=IF $s.kind='run_history' {'run'} ELSE {'study'};
        fn::pse_history_v1::release($expiry,$kind,$s.key);
        RETURN {complete:(fn::pse_history_v1::collect($expiry,$kind,$s.key)).complete};
    } ELSE IF $s.kind='draft_version' {RETURN fn::pse_project_v1::retire_draft_version($expiry,$s);}
    ELSE IF $s.kind='source_history' {
        SELECT * FROM ONLY type::record('canonical_guards','retention:'+$revision.problem) FOR UPDATE;
        LET $explanation=fn::pse_lifecycle_v1::explain($expiry,$s);
        IF array::len($explanation.blockers)!=0 {THROW 'PSE_REFUSAL:retained_owner_selection_changed';};
        fn::pse_retention_v1::drop_source_root($expiry,$revision.problem,$s.key,'history:'+$s.key,'history',$s.key);
        RETURN {complete:true};
    };
    THROW 'PSE_REFUSAL:project_metadata_bound';
};
"#
}
