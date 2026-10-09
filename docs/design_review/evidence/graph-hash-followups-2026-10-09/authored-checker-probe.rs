// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

// Disposable test module included in typed_math_portable::tests. Source parsing,
// workspace initialization and setup are outside the publication measurements.
use super::*;
use crate::workspace::{CompilerContext, CompilerWorkspace, WorkspaceLimits};
use std::time::Instant;

fn source() -> String {
    let mut text = String::from("package p {fn law(x:Length)->Length=x;");
    for i in 0..32 {
        text.push_str(&format!("fn unused_{i}(x:Length)->Length=x;"));
    }
    text.push_str("def Root {var x:Length;eq residual:law(x)==0{m};}}");
    text
}

fn rows(text: &str) -> Arc<Vec<pse_modeling::Declaration>> {
    Arc::new(pse_authoring::language::parse(
        text,
        SemanticId::from_bytes([79; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    ).unwrap())
}

#[test]
fn authored_checker_publication_cost_and_boundaries() {
    let registry = Arc::new(standard_registry().unwrap());
    let preconditions = Arc::new(pse_quantity::PhysicalPreconditions::new(
        pse_quantity::generated::standard_preconditions(),
    ).unwrap());
    let base_source = source();
    let base = rows(&base_source);
    let root = base.iter().find(|row| row.name == "Root").unwrap().declaration_id;
    let scope = pse_modeling::PhysicalScope::default();
    let denied_scope = pse_modeling::PhysicalScope {
        package: None,
        documents: Some(BTreeSet::new()),
    };
    let inserted_source = base_source.replace("def Root", "fn inserted(x:Length)->Length=x;def Root");
    let absent_source = base_source.replace("residual:law(x)", "residual:missing(x)");
    let present_source = absent_source.replace("def Root", "fn missing(x:Length)->Length=x;def Root");
    let variants = vec![
        ("unchanged", base.clone(), scope.clone(), true),
        ("unrelated", rows(&base_source.replace("fn unused_15(x:Length)->Length=x;", "fn unused_15(x:Length)->Length=2*x;")), scope.clone(), true),
        ("selected", rows(&base_source.replace("fn law(x:Length)->Length=x;", "fn law(x:Length)->Length=2*x;")), scope.clone(), true),
        ("span", rows(&format!("\n\n{base_source}")), scope.clone(), true),
        ("revert", base.clone(), scope.clone(), true),
        ("membership-insert", rows(&inserted_source), scope.clone(), true),
        ("membership-delete", base.clone(), scope.clone(), true),
        ("absent", rows(&absent_source), scope.clone(), false),
        ("absence-to-present", rows(&present_source), scope.clone(), true),
        ("context-denied", base.clone(), denied_scope, false),
    ];
    let mut output = Vec::new();
    for (label, variant, variant_scope, succeeds) in variants {
        let mut checker_ns = Vec::new();
        let mut publication_ns = Vec::new();
        let mut preparation_ns = Vec::new();
        for _ in 0..7 {
            let context = CompilerContext {
                quantities: registry.clone(),
                preconditions: preconditions.clone(),
                providers: BTreeMap::new(),
            };
            let mut workspace = CompilerWorkspace::new(context, WorkspaceLimits::default()).unwrap();
            workspace.publish_modeling(base.clone(), scope.clone()).unwrap();
            let prepare = |workspace: &mut CompilerWorkspace| workspace.prepare_modeling_cancellable(
                root,
                pse_modeling::InstanceId::from_id(SemanticId::NIL),
                pse_modeling::Bindings::default(),
                pse_modeling::Limits::default(),
                Arc::new(AtomicBool::new(false)),
            ).unwrap();
            let initial = prepare(&mut workspace);
            let initial_identities: Vec<_> = initial.portable_bodies()
                .map(|body| body.semantic_identity()).collect();
            std::hint::black_box(initial);
            // Revert/delete must actually replace a different admitted inventory.
            if label == "revert" {
                let selected = rows(&base_source.replace("fn law(x:Length)->Length=x;", "fn law(x:Length)->Length=2*x;"));
                workspace.publish_modeling(selected, scope.clone()).unwrap();
                std::hint::black_box(prepare(&mut workspace));
            } else if label == "membership-delete" {
                workspace.publish_modeling(rows(&inserted_source), scope.clone()).unwrap();
                std::hint::black_box(prepare(&mut workspace));
            } else if label == "absence-to-present" {
                assert!(workspace.publish_modeling(rows(&absent_source), scope.clone()).is_err());
            }

            let type_context = pse_modeling::TypeContext {
                admissions: None,
                formula_authority: None,
                quantities: registry.as_ref(),
                preconditions: preconditions.as_ref(),
                scope: &variant_scope,
            };
            let start = Instant::now();
            let checked = pse_modeling::check(variant.as_ref(), &type_context);
            checker_ns.push(start.elapsed().as_nanos());
            assert_eq!(checked.is_ok(), succeeds, "direct checker {label}");
            let _ = std::hint::black_box(checked);

            let start = Instant::now();
            let published = workspace.publish_modeling(variant.clone(), variant_scope.clone());
            publication_ns.push(start.elapsed().as_nanos());
            assert_eq!(published.is_ok(), succeeds, "publication {label}");
            let _ = std::hint::black_box(published);
            let start = Instant::now();
            let prepared = prepare(&mut workspace);
            preparation_ns.push(start.elapsed().as_nanos());
            if !succeeds {
                assert_eq!(prepared.portable_bodies()
                    .map(|body| body.semantic_identity()).collect::<Vec<_>>(), initial_identities);
            }
            std::hint::black_box(prepared);
        }
        output.push(serde_json::json!({
            "case": label,
            "declarations": variant.len(),
            "publication_succeeds": succeeds,
            "direct_no_document_checker_ns": checker_ns,
            "publication_ns": publication_ns,
            "downstream_preparation_ns": preparation_ns,
            "failed_publication_previous_inventory_still_prepares": !succeeds,
        }));
    }
    println!("AUTHORED_CHECKER_PROBE {}", serde_json::json!({
        "samples_per_case": 7,
        "fixture": "authored-scalar-32-unselected-functions",
        "direct_checker_is_separate_comparator_not_internal_publication_timer": true,
        "parse_and_initialization_excluded": true,
        "measurements": output,
    }));
}
