// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual phase executions, complete fresh answers and failed-candidate isolation.
use super::test_observation::{Query, QueryObservations, metric_count, observed_workspace};
use super::*;
use pse_authoring::{
    ParseBudget,
    language::{IdentityPolicy, parse},
};

const TEXT: &str = "package p { def Root { var x: Scalar; eq balance: x == 1; } def Other { var y: Scalar; eq balance: y == 2; } }";

fn rows(text: &str) -> Vec<Declaration> {
    parse(
        text,
        SemanticId::from_bytes([93; 16]),
        IdentityPolicy::Named,
        ParseBudget::default(),
    )
    .unwrap()
}
fn observed() -> (CompilerWorkspace, QueryObservations) {
    observed_workspace(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
}
fn assert_fresh(
    workspace: &CompilerWorkspace,
    source: &[Declaration],
    revision: &ModelingRevision,
) {
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: &workspace.inputs.quantities,
        preconditions: &workspace.inputs.preconditions,
        scope: &revision.scope,
    };
    let fresh = pse_modeling::check::check(source, &context).unwrap();
    assert_eq!(&fresh, revision.checked.as_ref());
}

#[test]
fn consumed_body_edit_executes_one_source_and_cuts_off_equal_sibling_phases() {
    let (mut workspace, events) = observed();
    let mut source = rows(TEXT);
    let before = workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &source, &before);
    let root = before.checked.entry("p.Root").unwrap();
    let other = before.checked.entry("p.Other").unwrap();
    let instance = pse_modeling::specialize::root_instance(root);
    let other_instance = pse_modeling::specialize::root_instance(other);
    let root_before = workspace
        .specialize_modeling(root, instance, Bindings::default(), Limits::default())
        .unwrap();
    let other_before = workspace
        .specialize_modeling(
            other,
            other_instance,
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    let specializations = events.count(Query::Specialized);
    let parses = events.count(Query::Parsed);
    let types = events.count(Query::Typed);
    let resolutions = events.count(Query::Resolved);
    let members = events.count(Query::EffectiveMembers);
    let occurrences = events.count(Query::Occurrence);
    let expressions = events.count(Query::CheckedExpressions);
    let references = events.count(Query::ExpressionReferences);
    assert_eq!(parses, source.len());
    assert_eq!(types, source.len());
    let balance = source
        .iter_mut()
        .find(|row| row.value.equation.is_some())
        .unwrap();
    balance.value.equation.as_mut().unwrap().expression = "x == 3".into();
    let after = workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &source, &after);
    assert_ne!(before.checked, after.checked);
    assert_eq!(events.count(Query::Parsed) - parses, 1);
    assert_eq!(events.count(Query::Resolved) - resolutions, 0);
    assert_eq!(events.count(Query::Typed) - types, 1);
    assert_eq!(events.count(Query::EffectiveMembers), members);
    assert_eq!(events.count(Query::Occurrence) - occurrences, 1);
    assert_eq!(events.count(Query::CheckedExpressions) - expressions, 1);
    assert_eq!(events.count(Query::ExpressionReferences) - references, 1);
    let root_after = workspace
        .specialize_modeling(root, instance, Bindings::default(), Limits::default())
        .unwrap();
    let other_after = workspace
        .specialize_modeling(
            other,
            other_instance,
            Bindings::default(),
            Limits::default(),
        )
        .unwrap();
    assert!(!Arc::ptr_eq(&root_before, &root_after));
    assert!(Arc::ptr_eq(&other_before, &other_after));
    assert_eq!(
        events.count(Query::Specialized) - specializations,
        1,
        "equal selected sibling cuts off actual specialization"
    );
    let fresh = pse_modeling::specialize(
        after.checked.as_ref(),
        root,
        instance,
        &Bindings::default(),
        Limits::default(),
    )
    .unwrap();
    assert_eq!(root_after.as_ref(), &fresh);
    let executions = events.total();
    let repeated = workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap();
    assert!(Arc::ptr_eq(&after, &repeated));
    assert_eq!(events.total(), executions);
    // An ABA source transition must recover the original complete answer, not a stale B answer.
    let restored = workspace
        .publish_modeling(rows(TEXT), PhysicalScope::default())
        .unwrap();
    assert_eq!(before.checked, restored.checked);
    assert_fresh(&workspace, &rows(TEXT), &restored);
}

#[test]
fn namespace_add_delete_shadow_import_and_absence_match_fresh_complete_admission() {
    let (mut workspace, _) = observed();
    for text in [
        "package p {param outer:Scalar=1; def Root {var x:Scalar; eq e:x==outer;}}",
        "package p {param outer:Scalar=1; def Root {param outer:Scalar=2; var x:Scalar; eq e:x==outer;}}",
        "package p {param outer:Scalar=1; def Root {var x:Scalar; eq e:x==outer;}}",
        "package p {param outer:Scalar=1; def Root {var x:Scalar; var sibling:Scalar; eq e:x==outer;}}",
        "package p {param outer:Scalar=1; def Root {var x:Scalar; eq e:x==outer;}}",
        "package q {param value:Scalar=4;} package p {use q @ \"1.0.0\" as foreign; def Root {var x:Scalar; eq e:x==foreign.value;}}",
        "package q {param value:Scalar=4;} package p {use q @ \"1.0.0\" as renamed; def Root {var x:Scalar; eq e:x==renamed.value;}}",
    ] {
        let source = rows(text);
        let revision = workspace
            .publish_modeling(source.clone(), PhysicalScope::default())
            .unwrap();
        assert_fresh(&workspace, &source, &revision);
    }
    let before = workspace.modeling.as_ref().unwrap().revision.clone();
    let absent = rows("package p {def Root {var x:Scalar; eq e:x==missing_name;}}");
    assert!(
        workspace
            .publish_modeling(absent, PhysicalScope::default())
            .is_err()
    );
    assert!(Arc::ptr_eq(
        &before,
        &workspace.modeling.as_ref().unwrap().revision
    ));
    let supplied = rows(
        "package p {def Root {param missing_name:Scalar=5; var x:Scalar; eq e:x==missing_name;}}",
    );
    let revision = workspace
        .publish_modeling(supplied.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &supplied, &revision);
}

#[test]
fn failed_duplicate_parent_and_cycles_restore_exact_head_then_accept_repair() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let source = rows(TEXT);
    let head = workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap();
    let mut duplicate_identity = source.clone();
    duplicate_identity.push(source[1].clone());
    let mut duplicate_member = source.clone();
    let mut extra = source[2].clone();
    extra.declaration_id = DeclarationId::from_id(SemanticId::from_bytes([94; 16]));
    duplicate_member.push(extra);
    let mut absent_parent = source.clone();
    absent_parent[2].parent_id = Some(DeclarationId::from_id(SemanticId::from_bytes([95; 16])));
    let mut parent_cycle = source.clone();
    parent_cycle[0].parent_id = Some(parent_cycle[1].declaration_id);
    for invalid in [
        duplicate_identity,
        duplicate_member,
        absent_parent,
        parent_cycle,
        rows("package p {interface A extends B {} interface B extends A {} def Root {}}"),
        rows("package p {fn a(x:Scalar)->Scalar=b(x); fn b(x:Scalar)->Scalar=a(x); def Root {}}"),
    ] {
        let generation = workspace.generation();
        assert!(
            workspace
                .publish_modeling(invalid, PhysicalScope::default())
                .is_err()
        );
        assert_eq!(workspace.generation(), generation + 1);
        assert!(Arc::ptr_eq(
            &head,
            &workspace.modeling.as_ref().unwrap().revision
        ));
        assert!(workspace.selects_modeling_revision(&head));
    }
    let repaired = rows("package p {def Root {var x:Scalar; eq e:x==7;}}");
    let revision = workspace
        .publish_modeling(repaired.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &repaired, &revision);
}

#[test]
fn cancellation_after_source_staging_rotates_and_resets_for_next_candidate() {
    let cancel = Arc::new(AtomicBool::new(false));
    let arm = Arc::new(AtomicBool::new(false));
    let observed_cancel = cancel.clone();
    let observed_arm = arm.clone();
    let mut workspace = CompilerWorkspace::with_events(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
        Some(Box::new(move |event| {
            if observed_arm.load(Ordering::Acquire)
                && matches!(event.kind, salsa::EventKind::WillExecute { .. })
            {
                observed_cancel.store(true, Ordering::Release);
            }
        })),
    )
    .unwrap();
    let source = rows(TEXT);
    let head = workspace
        .publish_modeling(source, PhysicalScope::default())
        .unwrap();
    let edited = rows(&TEXT.replace("x == 1", "x == 9"));
    arm.store(true, Ordering::Release);
    let failure = workspace.publish_modeling_with_cancellable(
        edited.clone(),
        PhysicalScope::default(),
        Arc::default(),
        cancel,
    );
    assert!(matches!(
        failure,
        Err(CompileError::Cancelled)
            | Err(CompileError::Modeling(
                pse_modeling::ModelingError::Cancelled
            ))
    ));
    assert!(Arc::ptr_eq(
        &head,
        &workspace.modeling.as_ref().unwrap().revision
    ));
    assert_eq!(workspace.generation(), 1);
    let revision = workspace
        .publish_modeling(edited.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &edited, &revision);
}

#[test]
fn pressure_rotation_drops_generation_inputs_without_borrowing_historical_products() {
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits {
            retained_entries: 1,
            ..WorkspaceLimits::default()
        },
    )
    .unwrap();
    let first = workspace
        .publish_modeling(rows(TEXT), PhysicalScope::default())
        .unwrap();
    assert!(workspace.generation() > 0);
    assert_eq!(workspace.granular.entries(), 0);
    let generation = workspace.generation();
    let edited = rows(&TEXT.replace("x == 1", "x == 9"));
    let second = workspace
        .publish_modeling(edited.clone(), PhysicalScope::default())
        .unwrap();
    assert!(workspace.generation() > generation);
    assert_eq!(workspace.granular.entries(), 0);
    assert_ne!(first.checked, second.checked);
    assert_fresh(&workspace, &rows(TEXT), &first);
    assert_fresh(&workspace, &edited, &second);
}

#[test]
fn field_attribution_and_document_identity_are_exact_phase_inputs() {
    let (mut workspace, events) = observed();
    let source = rows(TEXT);
    let equation = source
        .iter()
        .find(|row| row.value.equation.is_some())
        .unwrap()
        .declaration_id;
    let mut docs = DocumentInventory::default();
    docs.field_spans.insert(
        equation,
        [(
            "equation.expression".into(),
            pse_authoring::SourceSpan::new(SemanticId::from_bytes([96; 16]), 10, 16),
        )]
        .into(),
    );
    let first = workspace
        .publish_modeling_with(
            source.clone(),
            PhysicalScope::default(),
            Arc::new(docs.clone()),
        )
        .unwrap();
    let parses = events.count(Query::Parsed);
    let types = events.count(Query::Typed);
    let resolutions = events.count(Query::Resolved);
    let expressions = events.count(Query::CheckedExpressions);
    docs.field_spans
        .get_mut(&equation)
        .unwrap()
        .get_mut("equation.expression")
        .unwrap()
        .document_id = SemanticId::from_bytes([97; 16]);
    let second = workspace
        .publish_modeling_with(source, PhysicalScope::default(), Arc::new(docs))
        .unwrap();
    assert_ne!(first.checked, second.checked);
    assert_eq!(events.count(Query::Parsed) - parses, 1);
    assert_eq!(events.count(Query::Typed), types);
    assert_eq!(events.count(Query::Resolved), resolutions);
    assert_eq!(events.count(Query::CheckedExpressions), expressions);
    assert_eq!(
        first
            .checked
            .expression_occurrence(equation, "equation.expression", 0)
            .unwrap()
            .source
            .document_id,
        SemanticId::from_bytes([96; 16])
    );
    assert_eq!(
        second
            .checked
            .expression_occurrence(equation, "equation.expression", 0)
            .unwrap()
            .source
            .document_id,
        SemanticId::from_bytes([97; 16])
    );
}

#[test]
fn relocating_unchanged_declarations_preserves_type_and_member_memos() {
    let (mut workspace, events) = observed();
    let mut source = rows(TEXT);
    workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap();
    let types = events.count(Query::Typed);
    let resolutions = events.count(Query::Resolved);
    let members = events.count(Query::EffectiveMembers);
    let expressions = events.count(Query::CheckedExpressions);
    for row in &mut source {
        row.source_start += 10;
        row.source_end += 10;
    }
    let revision = workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &source, &revision);
    assert_eq!(events.count(Query::Typed), types);
    assert_eq!(events.count(Query::Resolved), resolutions);
    assert_eq!(events.count(Query::EffectiveMembers), members);
    assert_eq!(events.count(Query::CheckedExpressions), expressions);
}

#[test]
fn expression_inputs_retain_signature_type_guard_and_missing_name_changes() {
    for (before, after) in [
        (
            "package p {fn law(x:Scalar)->Scalar=x;def Root {var x:Scalar;eq e:law(x)==x;}}",
            "package p {fn law(x:Temperature)->Scalar=x;def Root {var x:Scalar;eq e:law(x)==x;}}",
        ),
        (
            "package p {def Root {var x:Scalar;var y:Scalar;eq e:x==y;}}",
            "package p {def Root {var x:Scalar;var y:Temperature;eq e:x==y;}}",
        ),
        (
            "package p {def Root {param mode:Integer=0;when mode==0 {var x:Scalar;eq e:x==1;}}}",
            "package p {def Root {param mode:Integer=0;when missing_mode==0 {var x:Scalar;eq e:x==1;}}}",
        ),
        (
            "package p {def Root {param available:Scalar=1;var x:Scalar;eq e:x==available;}}",
            "package p {def Root {var x:Scalar;eq e:x==available;}}",
        ),
    ] {
        let (mut workspace, events) = observed();
        let source = rows(before);
        let head = workspace
            .publish_modeling(source.clone(), PhysicalScope::default())
            .unwrap();
        assert_fresh(&workspace, &source, &head);
        let executions = events.count(Query::CheckedExpressions);
        let changed = rows(after);
        let context = TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &workspace.inputs.quantities,
            preconditions: &workspace.inputs.preconditions,
            scope: &head.scope,
        };
        let expected = pse_modeling::check::check(&changed, &context)
            .unwrap_err()
            .to_string();
        let actual = workspace
            .publish_modeling(changed, PhysicalScope::default())
            .unwrap_err()
            .to_string();
        assert!(
            actual.contains(&expected),
            "fresh: {expected}; retained: {actual}"
        );
        assert!(events.count(Query::CheckedExpressions) > executions);
        assert!(Arc::ptr_eq(
            &head,
            &workspace.modeling.as_ref().unwrap().revision
        ));
    }
}

#[test]
fn retained_physical_admissions_match_fresh_for_calls_inheritance_guards_and_indices() {
    let (mut workspace, _) = observed();
    for text in [
        "package p {fn law(x:Power)->Power=x+x;interface Base {var x:Power;eq e:law(x)+x==x;}def Root:Base {}}",
        "package p {fn law(x:Power)->Power=x-x;interface Base {var x:Power;eq e:law(x)+x==x;}def Root:Base {}}",
        "package p {entity kind item {} entity item a {} entity item b {} set items:Set<item>={a,b};def Root {var x[j in items]:Scalar;eq e[j in items]:x[j]+1==2;}}",
        "package p {entity kind item {} entity item a {} entity item b {} set items:Set<item>={a,b};def Root {var x[j in items]:Scalar;eq e[j in items]:x[j]+2==3;}}",
        "package p {interface Base {param mode:Integer;when mode==0 {var x:Power;eq e:x+x==x;}}def Root:Base {param mode:Integer=0;}}",
        "package p {interface Base {param mode:Integer;when mode==1 {var x:Power;eq e:x+x==x;}}def Root:Base {param mode:Integer=0;}}",
    ] {
        let source = rows(text);
        let revision = workspace
            .publish_modeling(source.clone(), PhysicalScope::default())
            .unwrap();
        assert_fresh(&workspace, &source, &revision);
        assert!(
            revision
                .checked
                .declarations()
                .filter_map(|row| revision.checked.expression_occurrence(
                    row.declaration_id,
                    "equation.expression",
                    0
                ))
                .any(|field| !field.physical_admissions.is_empty())
        );
    }
}

#[test]
fn physical_visibility_change_rechecks_ambiguity_without_publishing_failed_scope() {
    let (mut workspace, _) = observed();
    let source = rows("package p {enum Temperature {a} def Root {}}");
    let hidden = PhysicalScope {
        documents: Some(Default::default()),
        ..PhysicalScope::default()
    };
    let head = workspace.publish_modeling(source.clone(), hidden).unwrap();
    assert_fresh(&workspace, &source, &head);
    let failure = workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap_err();
    let declaration = source
        .iter()
        .find(|row| row.name == "Temperature")
        .unwrap()
        .declaration_id
        .as_id();
    let diagnostic = pse_model::diagnostic::DiagnosticProjection::boundary_diagnostic(
        &failure,
        pse_diagnostics::DiagnosticStage::Modeling,
    );
    assert_eq!(diagnostic.sources, vec![declaration]);
    assert_eq!(
        diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingContract
    );
    assert!(matches!(diagnostic.observations.get("reason"),
        Some(pse_model::diagnostic::Observation::Text(reason))
        if reason == pse_modeling::ModelingContractReason::AmbiguousName.as_str()));
    assert!(Arc::ptr_eq(
        &head,
        &workspace.modeling.as_ref().unwrap().revision
    ));
    let repaired = workspace
        .publish_modeling(source, head.scope.clone())
        .unwrap();
    assert!(Arc::ptr_eq(&head, &repaired));
}

#[test]
fn consumer_validation_refusal_occurs_before_live_head_publication() {
    let (mut workspace, _) = observed();
    let head = workspace
        .publish_modeling(rows(TEXT), PhysicalScope::default())
        .unwrap();
    let result = workspace.publish_modeling_with_validation(
        rows(&TEXT.replace("x == 1", "x == 9")),
        PhysicalScope::default(),
        Arc::default(),
        Arc::new(AtomicBool::new(false)),
        |candidate| {
            assert_ne!(candidate, head.checked.as_ref());
            Err(CompileError::Missing("source supplier mismatch".into()))
        },
    );
    assert!(matches!(result, Err(CompileError::Missing(_))));
    assert!(Arc::ptr_eq(
        &head,
        &workspace.modeling.as_ref().unwrap().revision
    ));
    assert_eq!(workspace.generation(), 1);
}

#[test]
fn observation_attaches_to_actual_queries_and_survives_generation_and_operation_changes() {
    use pse_diagnostics::observation::{ObservationHub, Policy};
    // Construct before entering any operation: no caller is captured by the database.
    let (mut workspace, events) = observed();
    let hub = ObservationHub::new(Policy::default());
    let first = hub.start("cold preparation");
    let first_context = first.context();
    {
        let _entered = first_context.enter();
        let revision = workspace
            .publish_modeling(rows(TEXT), PhysicalScope::default())
            .unwrap();
        let executed = events.total() as u64;
        assert!(executed > 0);
        assert_eq!(
            metric_count(&first_context, "compiler.query_executions"),
            executed
        );
        let repeated = workspace
            .publish_modeling(rows(TEXT), PhysicalScope::default())
            .unwrap();
        assert!(Arc::ptr_eq(&revision, &repeated));
        assert_eq!(
            metric_count(&first_context, "compiler.query_executions"),
            executed
        );
        assert_eq!(
            metric_count(&first_context, "compiler.unchanged_revision_reuses"),
            1
        );
        workspace.rebuild(workspace.inputs.clone()).unwrap();
        workspace
            .publish_modeling(
                rows(&TEXT.replace("x == 1", "x == 3")),
                PhysicalScope::default(),
            )
            .unwrap();
        assert_eq!(
            metric_count(&first_context, "compiler.workspace_rebuilds"),
            1
        );
        assert_eq!(
            metric_count(&first_context, "compiler.query_executions"),
            events.total() as u64
        );
    }
    let frozen = metric_count(&first_context, "compiler.query_executions");
    let before = events.total() as u64;
    let second = hub.start("next preparation");
    let second_context = second.context();
    {
        let _entered = second_context.enter();
        workspace
            .publish_modeling(rows(TEXT), PhysicalScope::default())
            .unwrap();
    }
    assert_eq!(
        metric_count(&first_context, "compiler.query_executions"),
        frozen
    );
    assert_eq!(
        metric_count(&second_context, "compiler.query_executions"),
        events.total() as u64 - before
    );
    assert!(metric_count(&second_context, "compiler.query_executions") > 0);
}

#[test]
fn observation_and_test_callback_survive_failed_candidate_restore() {
    use pse_diagnostics::observation::{ObservationHub, Policy};
    let (mut workspace, events) = observed();
    workspace
        .publish_modeling(rows(TEXT), PhysicalScope::default())
        .unwrap();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("failed source and retry");
    let context = operation.context();
    let _entered = context.enter();
    let before = events.total() as u64;
    assert!(
        workspace
            .publish_modeling(
                rows("package p { def Root { var x: Scalar; eq e:x==missing; } }"),
                PhysicalScope::default()
            )
            .is_err()
    );
    assert_eq!(
        metric_count(&context, "compiler.failed_candidate_rotations"),
        1
    );
    let after_failure = events.total();
    workspace
        .publish_modeling(
            rows(&TEXT.replace("x == 1", "x == 4")),
            PhysicalScope::default(),
        )
        .unwrap();
    assert!(events.total() > after_failure);
    assert_eq!(
        metric_count(&context, "compiler.query_executions"),
        events.total() as u64 - before
    );
}

#[test]
fn observation_reports_memo_validation_without_inventing_query_execution() {
    use pse_diagnostics::observation::{ObservationHub, Policy};
    use salsa::Setter;
    let (mut workspace, events) = observed();
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("memo validation");
    let context = operation.context();
    let _entered = context.enter();
    assert!(provider(&workspace.db, workspace.inventory, "absent".into()).is_none());
    assert_eq!(metric_count(&context, "compiler.query_executions"), 1);
    // This query consumes providers, never environment: a new revision validates it.
    workspace
        .inventory
        .set_environment(&mut workspace.db)
        .to(ContentHash::from_bytes([77; 32]));
    assert!(provider(&workspace.db, workspace.inventory, "absent".into()).is_none());
    assert_eq!(metric_count(&context, "compiler.query_executions"), 1);
    assert_eq!(events.total(), 1);
    assert_eq!(metric_count(&context, "compiler.query_memo_validations"), 1);
}

#[test]
fn current_input_roots_do_not_accumulate_candidate_preflight_history() {
    let (mut workspace, events) = observed();
    let mut previous_by_shape = [None, None];
    let mut old_outputs = Vec::new();
    for edit in 0..16 {
        let shape = edit % 2;
        let text = format!(
            "package p {{ def Root {{ param k:Scalar={}; var x:Scalar; eq balance:x==k+{}; }} def Other {{ var y:Scalar; eq balance:y==2; }} }}",
            shape + 1,
            shape
        );
        let mut source = rows(&text);
        let equation = source
            .iter_mut()
            .find(|row| row.value.equation.is_some())
            .unwrap();
        let offset = i64::try_from(edit).unwrap();
        equation.source_start += offset;
        equation.source_end += offset;
        let revision = workspace
            .publish_modeling(source.clone(), PhysicalScope::default())
            .unwrap();
        assert_fresh(&workspace, &source, &revision);
        let known = workspace.granular.known_bytes(&workspace.db);
        assert!(known > 0);
        // Two warm candidates establish the actually retained old/current roots.
        // Subsequent candidates with the same allocation shape retain no history
        // allowance merely because the checker executed again.
        if edit >= 4 {
            assert_eq!(
                Some(known),
                previous_by_shape[shape],
                "edit {edit} added construction history to retained roots"
            );
        }
        previous_by_shape[shape] = Some(known);
        let before = {
            let s = workspace.observe_retention();
            (s.entries, s.known_bytes)
        };
        let executed = events.total();
        let repeated = workspace
            .publish_modeling(source, PhysicalScope::default())
            .unwrap();
        assert!(Arc::ptr_eq(&revision, &repeated));
        assert_eq!(
            {
                let s = workspace.observe_retention();
                (s.entries, s.known_bytes)
            },
            before
        );
        assert_eq!(events.total(), executed);
        assert_eq!(
            workspace.generation(),
            0,
            "ordinary small edits must not rotate from preflight history"
        );
        old_outputs.push(revision);
    }
    for revision in &old_outputs {
        assert_fresh(&workspace, &revision.rows, revision);
    }
    // The pinned Salsa input ingredient reports inline fields, but supplies no
    // automatic heap callback for these declared Source inputs.
    let report = <dyn Database>::memory_usage(&mut workspace.db);
    let source = report
        .structs
        .iter()
        .find(|info| info.debug_name().ends_with("Source"))
        .unwrap();
    assert!(source.size_of_fields() > 0);
    assert_eq!(source.heap_size_of_fields(), None);
    assert!(
        {
            let s = workspace.observe_retention();
            (s.entries, s.known_bytes)
        }
        .1 >= workspace.granular.known_bytes(&workspace.db)
    );
}

#[test]
fn unrelated_preceding_type_change_retains_exact_sibling_signature_products() {
    let (mut workspace, events) = observed();
    let text = "package p {fn law(x:Scalar)->Scalar=x; fn untouched(x:Integer)->Integer=x; def Root {var x:Scalar;eq e:x==1;}}";
    let source = rows(text);
    let before = workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &source, &before);
    let typed = events.count(Query::Typed);
    let resolved = events.count(Query::Resolved);
    let changed = rows(&text.replace("law(x:Scalar)->Scalar", "law(x:Temperature)->Temperature"));
    let after = workspace
        .publish_modeling(changed.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &changed, &after);
    assert_eq!(events.count(Query::Typed) - typed, 1);
    assert_eq!(events.count(Query::Resolved) - resolved, 1);
}

#[test]
fn consumed_preceding_named_type_change_refuses_then_repair_matches_fresh() {
    let (mut workspace, _) = observed();
    let text = "package p {fn law(x:Scalar)->Scalar=x; fn apply(f:law,x:Scalar)->Scalar=f(x); def Root {var x:Scalar;eq e:x==1;}}";
    let source = rows(text);
    let head = workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &source, &head);
    let changed = rows(&text.replace("law(x:Scalar)->Scalar", "law(x:Temperature)->Temperature"));
    let context = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: &workspace.inputs.quantities,
        preconditions: &workspace.inputs.preconditions,
        scope: &head.scope,
    };
    let expected = pse_modeling::check::check(&changed, &context)
        .unwrap_err()
        .to_string();
    let actual = workspace
        .publish_modeling(changed, PhysicalScope::default())
        .unwrap_err()
        .to_string();
    assert!(
        actual.contains(&expected),
        "fresh: {expected}; retained: {actual}"
    );
    assert!(Arc::ptr_eq(
        &head,
        &workspace.modeling.as_ref().unwrap().revision
    ));
    let repaired = workspace
        .publish_modeling(source.clone(), PhysicalScope::default())
        .unwrap();
    assert_fresh(&workspace, &source, &repaired);
}
