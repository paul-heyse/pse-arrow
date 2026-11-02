// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Consequential native relationship, origin and comparison cases.
use super::tests::{fixture, query};
use super::*;

#[tokio::test]
async fn occurrence_scope_preserves_plural_macro_definition_groups() {
    let (root, configuration, file) = fixture();
    let source = "macro_rules! both { ($name:ident) => { mod first { pub fn $name(){} fn caller(){$name();} } mod second { pub fn $name(){} fn caller(){$name();} } }; }\nboth!(shared);\n";
    std::fs::write(&file, source).unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let selected = query(&file, "shared", Kind::Refs);
    let original = engine
        .start_query(selected.clone(), CancellationToken::new())
        .settle()
        .await
        .unwrap()
        .unwrap();
    let original: Vec<ReferenceFact> = serde_json::from_value(
        original
            .sections
            .iter()
            .find(|section| section.category == "native_reference_roles")
            .unwrap()
            .evidence
            .clone(),
    )
    .unwrap();
    let original_groups = original
        .iter()
        .map(|reference| reference.entity)
        .collect::<BTreeSet<_>>();
    assert!(
        original_groups.len() >= 2,
        "one authored macro argument must nominate the two independent native module definitions: {original:?}"
    );
    let mut selected = selected;
    selected.occurrence_scope = Some(vec![super::super::OccurrenceFile {
        file: file.clone(),
        range: None,
    }]);
    let answer = engine
        .start_query(selected, CancellationToken::new())
        .settle()
        .await
        .unwrap()
        .unwrap();
    let references: Vec<ReferenceFact> = serde_json::from_value(
        answer
            .sections
            .iter()
            .find(|section| section.category == "native_reference_roles")
            .unwrap()
            .evidence
            .clone(),
    )
    .unwrap();
    let groups = references
        .iter()
        .map(|reference| reference.entity)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        groups, original_groups,
        "native definition groups must survive scoping: {references:?}"
    );
    assert_eq!(references.len(), original.len());
}

#[tokio::test]
async fn occurrence_scope_retains_every_edition_of_a_shared_module_file() {
    let (root, configuration, file) = fixture();
    std::fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nmembers=['old','new']\nresolver='3'\n",
    )
    .unwrap();
    for (name, edition) in [("old", "2021"), ("new", "2024")] {
        std::fs::create_dir(root.path().join(name)).unwrap();
        std::fs::write(root.path().join(name).join("Cargo.toml"), format!("[package]\nname='{name}'\nversion='0.0.0'\nedition='{edition}'\n[lib]\npath='../src/lib.rs'\n")).unwrap();
    }
    std::fs::write(root.path().join("Cargo.lock"), "version=4\n[[package]]\nname='new'\nversion='0.0.0'\n[[package]]\nname='old'\nversion='0.0.0'\n").unwrap();
    std::fs::write(&file, "pub fn target() {}\nfn caller(){target();}\n").unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    {
        let world = engine.world.lock().unwrap();
        ra_ap_hir::attach_db_allow_change(&world.db, || {
            let selected = scope::Occurrences::prepare(
                &world.db,
                &world.files,
                &world.digests,
                Some(&[super::super::OccurrenceFile {
                    file: file.clone(),
                    range: None,
                }]),
            )
            .unwrap();
            let lowered = selected
                .native
                .unwrap()
                .into_iter()
                .map(|(id, _)| id.edition(&world.db))
                .collect::<BTreeSet<_>>();
            assert_eq!(
                lowered,
                BTreeSet::from([
                    native_ra_syntax::Edition::Edition2021,
                    native_ra_syntax::Edition::Edition2024
                ])
            );
        });
    }
    let mut selected = query(&file, "target()", Kind::Refs);
    selected.occurrence_scope = Some(vec![super::super::OccurrenceFile {
        file: file.clone(),
        range: None,
    }]);
    let answer = engine
        .start_query(selected, CancellationToken::new())
        .settle()
        .await
        .unwrap()
        .unwrap();
    let call = std::fs::read_to_string(&file)
        .unwrap()
        .rfind("target()")
        .unwrap();
    assert!(answer.observed.relationships.iter().any(|fact| matches!(&fact.location, super::super::FactLocation::Bytes { path, start, .. } if path == &file && *start == call)), "{answer:?}");
}

#[tokio::test]
async fn selected_symbol_population_survives_many_excluded_candidates_before_eligible() {
    let (root, configuration, file) = fixture();
    std::fs::write(&file, "mod excluded; mod eligible;\n").unwrap();
    let excluded = root.path().join("src/excluded.rs");
    let eligible = root.path().join("src/eligible.rs");
    let population = (0..192)
        .map(|n| format!("pub fn candidate{n:03}() {{}}\n"))
        .collect::<String>();
    std::fs::write(&excluded, population).unwrap();
    std::fs::write(&eligible, "pub fn candidatezzz() {}\n").unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let request = |limit| WorkspaceQuery {
        query: "candidate".into(),
        matching: super::super::NativeNameMode::Prefix,
        limit,
        include_dependencies: false,
        if_generation: Some(1),
    };
    let early = engine
        .start_symbols(request(20), CancellationToken::new())
        .settle()
        .await
        .unwrap()
        .unwrap();
    assert!(early.observed.iter().all(|symbol| !matches!(&symbol.location, Some(super::super::FactLocation::Bytes { path, .. }) if path == &eligible)), "fixture must expose the false-negative early-cap hazard");
    let full = engine
        .start_symbols(
            request(crate::investigation::rust::symbol_candidate_limit()),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    let selected = full.observed.iter().filter(|symbol| matches!(&symbol.location, Some(super::super::FactLocation::Bytes { path, .. }) if path == &eligible)).collect::<Vec<_>>();
    assert_eq!(
        selected.len(),
        1,
        "consumer candidate policy must preserve the independently selected eligible file"
    );
    assert_eq!(selected[0].name, "candidatezzz");
    assert_eq!(full.observed.len(), 193);
}

#[tokio::test]
async fn occurrence_scope_preserves_alias_macro_unicode_roles_and_exact_ranges() {
    use crate::semantic::OccurrenceFile;
    let (root, configuration, file) = fixture();
    let main = "pub fn στόχος() {}\n#[macro_export] macro_rules! invoke { ($f:path) => { $f() }; }\nmod keep; mod excluded;\n";
    let keep = root.path().join("src/keep.rs");
    let excluded = root.path().join("src/excluded.rs");
    let included_source = "use crate::στόχος as alias;\nfn caller(){alias();crate::στόχος();crate::invoke!(crate::στόχος);}\n";
    std::fs::write(&file, main).unwrap();
    std::fs::write(&keep, included_source).unwrap();
    std::fs::write(&excluded, "fn caller(){crate::στόχος();}\n").unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let run = |scope| {
        let mut selected = query(&file, "στόχος", Kind::Refs);
        selected.occurrence_scope = scope;
        engine.start_query(selected, CancellationToken::new())
    };
    let all = run(None).settle().await.unwrap().unwrap();
    assert!(all.observed.relationships.iter().any(|fact| matches!(&fact.location, super::super::FactLocation::Bytes { path, .. } if path == &excluded)));
    let answer = run(Some(vec![OccurrenceFile {
        file: keep.clone(),
        range: None,
    }]))
    .settle()
    .await
    .unwrap()
    .unwrap();
    let references: Vec<ReferenceFact> = serde_json::from_value(
        answer
            .sections
            .iter()
            .find(|section| section.category == "native_reference_roles")
            .unwrap()
            .evidence
            .clone(),
    )
    .unwrap();
    assert!(!references.is_empty());
    assert!(
        references
            .iter()
            .all(|reference| reference.location.path == keep)
    );
    assert!(references.iter().any(|reference| reference.import));
    let expected = included_source.rfind("στόχος").unwrap();
    assert!(
        references
            .iter()
            .any(|reference| reference.location.start == expected),
        "macro argument reference must retain original Unicode token: {references:?}"
    );
    // The native owner records the renamed import; it does not promise to
    // follow every alias spelling from an original-definition query. Select a
    // known original-name occurrence independently for exact range acceptance.
    let direct = included_source.find("στόχος();").unwrap();
    let range = direct..direct + "στόχος".len();
    let ranged = run(Some(vec![OccurrenceFile {
        file: keep.clone(),
        range: Some(range.clone()),
    }]))
    .settle()
    .await
    .unwrap()
    .unwrap();
    assert_eq!(ranged.observed.relationships.len(), 1, "{ranged:?}");
    assert!(
        matches!(&ranged.observed.relationships[0].location, super::super::FactLocation::Bytes { path, start, end } if path == &keep && *start == range.start && *end == range.end)
    );
    let plural = run(Some(vec![
        OccurrenceFile {
            file: keep.clone(),
            range: Some(range),
        },
        OccurrenceFile {
            file: excluded.clone(),
            range: Some(0..std::fs::read_to_string(&excluded).unwrap().len()),
        },
    ]))
    .settle()
    .await
    .unwrap()
    .unwrap();
    assert_eq!(plural.observed.relationships.len(), 2, "{plural:?}");
    assert!(
        plural
            .limitations
            .iter()
            .any(|limitation| limitation.contains("range-union"))
    );
    let empty = run(Some(Vec::new())).settle().await.unwrap().unwrap();
    assert!(empty.observed.relationships.is_empty());
    let invalid = run(Some(vec![OccurrenceFile {
        file: keep,
        range: Some(expected + 1..expected + 2),
    }]))
    .settle()
    .await
    .unwrap();
    assert!(
        matches!(invalid, Err(Error::InvalidRequest(_))),
        "{invalid:?}"
    );
}

#[tokio::test]
async fn inactive_and_outside_world_sources_preserve_typed_context() {
    let (root, configuration, file) = fixture();
    std::fs::write(&file, "#[cfg(test)] mod inactive;\npub fn ready() {}\n").unwrap();
    let inactive = root.path().join("src/inactive.rs");
    let outside_directory = tempfile::tempdir().unwrap();
    let outside = outside_directory.path().join("loose.rs");
    for path in [&inactive, &outside] {
        std::fs::write(path, "pub fn target() {}\n").unwrap();
    }
    let engine = Engine::start(root.path(), configuration).unwrap();
    for (path, reason) in [
        (&inactive, super::super::FailureReason::InactiveSource),
        (&outside, super::super::FailureReason::OutsidePreparedWorld),
    ] {
        let error = engine
            .start_query(query(path, "target", Kind::Refs), CancellationToken::new())
            .settle()
            .await
            .unwrap()
            .unwrap_err();
        let failure = super::super::Failure::from_error(&error);
        assert_eq!(failure.context.unwrap().reason, reason, "{error:?}");
    }
    // A current active entity with an inactive requested occurrence file cannot
    // produce a complete-looking no-match. Classify the selector independently.
    let mut selected = query(&file, "ready", Kind::Refs);
    selected.occurrence_scope = Some(vec![super::super::OccurrenceFile {
        file: inactive.clone(),
        range: None,
    }]);
    let error = engine
        .start_query(selected, CancellationToken::new())
        .settle()
        .await
        .unwrap()
        .unwrap_err();
    let failure = super::super::Failure::from_error(&error);
    let context = failure.context.unwrap();
    assert_eq!(context.reason, super::super::FailureReason::InactiveSource);
    assert_eq!(context.operation, "reference_scope");
    assert_eq!(context.operand.unwrap().to_path_buf(), inactive);
}

#[tokio::test]
async fn parent_token_cancellation_reaches_actual_native_reference_owner_and_drains() {
    let (root, configuration, file) = fixture();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let parent = CancellationToken::new();
    let token = parent.child_token();
    let (entered_tx, entered_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let task = engine.start_query_observed(
        query(&file, "original()->", Kind::Refs),
        token.clone(),
        move |_| {
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            None
        },
    );
    entered_rx.await.unwrap();
    let mut sibling = engine.start_query(
        query(&file, "original()->", Kind::Refs),
        CancellationToken::new(),
    );
    parent.cancel();
    let settle = super::super::rpc::settle_cpu(task, &token);
    tokio::pin!(settle);
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(10), &mut settle)
            .await
            .is_err(),
        "cancellation cannot claim drain while actual native CPU owner is blocked"
    );
    release_tx.send(()).unwrap();
    assert!(matches!(settle.await, Err(Error::Cancelled)));
    assert!(sibling.settle().await.unwrap().is_ok());
    assert!(
        engine
            .start_query(
                query(&file, "original()->", Kind::Refs),
                CancellationToken::new()
            )
            .settle()
            .await
            .unwrap()
            .is_ok()
    );
    assert_eq!(engine.active.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn bounded_native_observation_preserves_relevance_incompleteness() {
    let (root, configuration, file) = fixture();
    std::fs::write(&file, "pub fn selected() {}\nfn first(){selected();}\nfn second(){selected();}\nfn third(){selected();}\n").unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let mut selected = query(&file, "selected()", Kind::Refs);
    selected.limit = 1;
    let answer = engine
        .start_query(selected, CancellationToken::new())
        .settle()
        .await
        .unwrap()
        .unwrap();
    assert!(
        answer
            .omitted_relationships
            .values()
            .any(|count| *count > 0),
        "{answer:?}"
    );
    assert!(!crate::relevance::native_incompleteness(&answer).is_empty());
}

#[tokio::test]
async fn documentation_links_resolve_in_native_owner_scope_and_preserve_unknown_targets() {
    let (root, configuration, file) = fixture();
    let source = "pub struct Same;\nmod nested {\n pub struct Same;\n /// Rejected alternative [Same](Same), unknown [Missing](Missing).\n pub fn selected() {}\n}\n";
    std::fs::write(&file, source).unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let answer = engine
        .start_query(
            query(&file, "selected()", Kind::DocumentationLinks),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    let links = answer.observed.investigation.unwrap().documentation_links;
    assert_eq!(links.len(), 2, "{links:?}");
    let same = links.iter().find(|link| link.written == "Same").unwrap();
    let target = same.target.as_ref().unwrap();
    assert!(
        target.start > source.find("mod nested").unwrap(),
        "namesakes must resolve in the doc owner's module"
    );
    assert!(source[target.start..target.end].contains("Same"));
    let unknown = links.iter().find(|link| link.written == "Missing").unwrap();
    assert!(unknown.target.is_none());
    assert!(!unknown.limitations.is_empty());
}

#[tokio::test]
async fn format_origins_skip_escaped_braces_and_refuse_custom_templates() {
    let (root, configuration, file) = fixture();
    let source = "macro_rules! custom { ($s:literal) => { $s }; }\n#[derive(Clone)]\n#[derive(Debug)]\nstruct Derived;\nfn target(){let named=7u8;let _=format!(\"{{literal}} {named}\");let _=custom!(\"{named}\");}\n";
    std::fs::write(&file, source).unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let answer = engine
        .start_query(
            query(
                &file,
                "\"{{literal}}",
                Kind::Origins {
                    kind: crate::semantic::OriginKind::Format,
                },
            ),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    let origins = answer.observed.investigation.unwrap().origins;
    assert_eq!(
        origins.len(),
        1,
        "escaped braces must not create references"
    );
    assert_eq!(origins[0].name.as_deref(), Some("named"));
    let occurrence = origins[0].occurrence.as_ref().unwrap();
    assert_eq!(&source[occurrence.start..occurrence.end], "named");
    let unknown = engine
        .start_query(
            query(
                &file,
                "\"{named}\"",
                Kind::Origins {
                    kind: crate::semantic::OriginKind::Format,
                },
            ),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap_err();
    assert!(matches!(unknown, Error::Unavailable(_)), "{unknown:?}");
    let derived = engine
        .start_query(
            query(
                &file,
                "Derived;",
                Kind::Origins {
                    kind: crate::semantic::OriginKind::Derive,
                },
            ),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap()
        .observed
        .investigation
        .unwrap();
    for name in ["Clone", "Debug"] {
        assert!(
            derived
                .origins
                .iter()
                .any(|origin| origin.name.as_deref() == Some(name)
                    && origin.target.is_none()
                    && origin.originating_adt.is_some()),
            "{derived:?}"
        );
    }
}
use crate::investigation::{Direction, Relationship, graph::SeedMatch};
use crate::semantic::{CompatibilityOperation as Op, GraphQuery, GraphRole, ImportTarget};

fn target(file: &Path, source: &str, needle: &str) -> ImportTarget {
    let start = source.find(needle).unwrap();
    let name = needle
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .map(char::len_utf8)
        .sum::<usize>();
    assert!(
        name > 0,
        "fixture operands select an exact identifier; the rest disambiguates its occurrence"
    );
    ImportTarget {
        file: file.into(),
        byte_start: start,
        byte_end: start + name,
        source_sha256: crate::source_read::source_digest(source),
    }
}
#[test]
fn binary_body_context_refuses_plural_roots_feature_drift_and_disabled_requirements() {
    let root = PathBuf::from("/selected/main.rs");
    let features = BTreeSet::from(["active".into()]);
    let target = crate::semantic::RunnableTargetFact {
        package_id: "actual-package".into(),
        manifest: "/selected/Cargo.toml".into(),
        root: root.clone(),
        name: "actual-bin".into(),
        kind: "binary".into(),
        required_features: vec!["active".into()],
        active_features: features.clone(),
        required_features_satisfied: true,
    };
    assert!(
        investigation::binary_context(std::slice::from_ref(&target), Some(&root), &features)
            .is_some()
    );
    assert!(
        investigation::binary_context(&[target.clone(), target.clone()], Some(&root), &features)
            .is_none()
    );
    assert!(
        investigation::binary_context(std::slice::from_ref(&target), Some(&root), &BTreeSet::new())
            .is_none()
    );
    let mut disabled = target.clone();
    disabled.required_features_satisfied = false;
    assert!(investigation::binary_context(&[disabled], Some(&root), &features).is_none());
    let mut missing = target.clone();
    missing.required_features.push("missing".into());
    assert!(investigation::binary_context(&[missing], Some(&root), &features).is_none());
    let mut library = target;
    library.kind = "library".into();
    assert!(investigation::binary_context(&[library], Some(&root), &features).is_none());
}
#[tokio::test]
async fn binary_body_keeps_actual_cargo_package_role_features_and_loader_target() {
    let (root, configuration, _library) = fixture();
    let file = root.path().join("src/main.rs");
    std::fs::write(
        &file,
        "fn selected()->u8{1}\nfn main(){let _=selected();}\n",
    )
    .unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let answer = engine
        .start_query(
            query(&file, "selected()->", Kind::Body),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    let body = answer
        .observed
        .investigation
        .as_ref()
        .unwrap()
        .body
        .as_ref()
        .unwrap();
    let context = answer
        .native_types
        .as_ref()
        .unwrap()
        .context
        .as_ref()
        .unwrap();
    let target = engine
        .runnable_targets
        .iter()
        .find(|target| target.kind == "binary" && target.root == file)
        .unwrap();
    assert_eq!(body.package_id.as_deref(), Some(target.package_id.as_str()));
    assert_eq!(body.role.as_deref(), Some("bin"));
    assert_eq!(context.crate_root, file);
    assert_eq!(context.features, target.active_features);
    assert_eq!(
        context.target.as_deref(),
        Some(engine.actual_target.as_str())
    );
    assert!(
        engine
            .packages
            .iter()
            .all(|package| package.library_root != file),
        "binary correspondence must not mutate library API qualification"
    );
}
#[tokio::test]
async fn compatibility_preserves_nominal_identity_direction_owners_unknowns_and_rhs_guards() {
    let (root, configuration, file) = fixture();
    let source = "mod a { pub struct Same; }\nmod b { pub struct Same; }\nfn first<T>(left:T){left;}\nfn second<T>(right:T){right;}\nfn compare(){let left=a::Same;let right=b::Same;left;right;let mut n=1u8;let m=&mut n;let r=&1u8;m;r;let unknown=missing;unknown;}\n";
    std::fs::write(&file, source).unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let compare = |left: &str, right: &str, operation| {
        query(
            &file,
            left,
            Kind::Compatibility {
                against: target(&file, source, right),
                against_generation: Some(1),
                operation,
            },
        )
    };
    let inspect = async |q| {
        engine
            .start_query(q, CancellationToken::new())
            .settle()
            .await
            .unwrap()
            .unwrap()
            .observed
            .investigation
            .unwrap()
            .compatibility
            .unwrap()
    };
    let distinct = inspect(compare("left;right;", "right;let", Op::Equal)).await;
    assert_eq!(distinct.result, Some(false));
    assert!(distinct.compatible_owner);
    assert!(distinct.left_type.contains("Same") && distinct.right_type.contains("Same"));
    let equality = inspect(compare("m;r;", "r;let", Op::Equal)).await;
    assert_eq!(equality.result, Some(false));
    let coercion = inspect(compare("m;r;", "r;let", Op::CouldCoerce)).await;
    assert_eq!(coercion.result, Some(true));
    let owners = inspect(compare(
        "left;}\nfn second",
        "right;}\nfn compare",
        Op::CouldUnify,
    ))
    .await;
    assert_eq!(owners.result, None);
    assert!(!owners.compatible_owner);
    let unknown = inspect(compare("unknown;}", "r;let", Op::CouldUnify)).await;
    assert_eq!(unknown.result, None);
    assert!(!unknown.limitations.is_empty());
    for generation in [Some(2), Some(0)] {
        let mut q = compare("m;r;", "r;let", Op::Equal);
        if let Kind::Compatibility {
            against_generation, ..
        } = &mut q.kind
        {
            *against_generation = generation;
        }
        assert!(matches!(
            engine
                .start_query(q, CancellationToken::new())
                .settle()
                .await
                .unwrap(),
            Err(Error::Stale)
        ));
    }
    let mut q = compare("m;r;", "r;let", Op::Equal);
    if let Kind::Compatibility { against, .. } = &mut q.kind {
        against.source_sha256 = "changed".into();
    }
    assert!(matches!(
        engine
            .start_query(q, CancellationToken::new())
            .settle()
            .await
            .unwrap(),
        Err(Error::Stale)
    ));
    let mut q = compare("m;r;", "r;let", Op::Equal);
    if let Kind::Compatibility { against, .. } = &mut q.kind {
        against.byte_start = source.find("left;right;").unwrap();
        against.byte_end = against.byte_start + "left;right;".len();
    }
    assert!(
        matches!(
            engine
                .start_query(q, CancellationToken::new())
                .settle()
                .await
                .unwrap(),
            Err(Error::InvalidRequest(_))
        ),
        "multiple expressions cannot silently select the first RHS"
    );
}
#[tokio::test]
async fn native_relationships_preserve_associated_local_field_context_and_actual_invocations() {
    let (root, configuration, file) = fixture();
    let source = "trait Base {}\ntrait Child: Base {fn associated(&self);}\nstruct Slot<T>{value:T}\nimpl Base for Slot<u8>{}\nimpl Child for Slot<u8>{fn associated(&self){}}\nfn callee(){}\nfn caller(){fn local(){} let callback=callee;callee();(callee)();local();let mut slot=Slot{value:1u8};let _=slot.value;slot.value=2;slot.value+=1;}\nfn outer(){mod inner_local{pub fn child(){}} struct Local;impl Local{fn method(){}}}\n";
    std::fs::write(&file, source).unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let intent = |seed: &str, relations| GraphQuery {
        seeds: vec![query(&file, seed, Kind::Explain)],
        relations,
        direction: Direction::Outgoing,
        traversal_paths: vec![file.clone()],
        output_paths: vec![file.clone()],
        depth: 3,
        max_nodes: 100,
        max_edges: 300,
        seed_match: SeedMatch::Any,
        exists: false,
    };
    let calls = engine
        .start_graph(
            intent("caller(){", vec![Relationship::Calls]),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    let callee_edges = calls
        .traversal
        .edges
        .iter()
        .filter(|e| calls.nodes[e.target].name.as_deref() == Some("callee"))
        .collect::<Vec<_>>();
    assert_eq!(
        callee_edges.len(),
        2,
        "callback value is not an invocation: {calls:?}"
    );
    assert!(
        calls
            .traversal
            .edges
            .iter()
            .any(|e| calls.nodes[e.target].name.as_deref() == Some("local")),
        "{calls:?}"
    );
    let uses = engine
        .start_graph(
            intent("caller(){", vec![Relationship::Uses]),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    let fields = uses
        .traversal
        .edges
        .iter()
        .filter(|e| uses.nodes[e.target].name.as_deref() == Some("value"))
        .map(|e| &uses.witnesses[e.witness])
        .collect::<Vec<_>>();
    assert!(fields.len() >= 4, "{uses:?}");
    assert!(
        fields
            .iter()
            .any(|w| w.declared_type.as_deref() == Some("T")
                && w.instantiated_type.as_deref() == Some("u8")),
        "{fields:?}"
    );
    assert!(fields.iter().any(|w| matches!(
        w.role,
        GraphRole::Reference {
            read: Some(false),
            write: Some(true),
            ..
        }
    )));
    assert!(fields.iter().any(|w| matches!(
        w.role,
        GraphRole::Reference {
            read: Some(true),
            write: Some(true),
            ..
        }
    )));
    let contained = engine
        .start_graph(
            intent("caller(){", vec![Relationship::Contains]),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    assert!(
        contained
            .traversal
            .edges
            .iter()
            .any(|e| contained.nodes[e.target].name.as_deref() == Some("local")),
        "{contained:?}"
    );
    let nested = engine
        .start_graph(
            intent("outer(){", vec![Relationship::Contains]),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    assert!(
        !nested.traversal.complete,
        "unsupported immediate owners retain unknown coverage"
    );
    assert!(
        nested
            .traversal
            .edges
            .iter()
            .all(
                |edge| nested.nodes[edge.target].name.as_deref() != Some("child")
                    && nested.nodes[edge.target].name.as_deref() != Some("method")
            ),
        "nested module and impl items must not be children of outer: {nested:?}"
    );
    let implementation = engine
        .start_graph(
            intent(
                "impl Child",
                vec![Relationship::ImplTrait, Relationship::ImplSelfType],
            ),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    assert!(
        implementation
            .traversal
            .edges
            .iter()
            .any(|e| e.relation == Relationship::ImplTrait
                && implementation.nodes[e.target].name.as_deref() == Some("Child")),
        "{implementation:?}"
    );
    assert!(
        implementation
            .traversal
            .edges
            .iter()
            .any(|e| e.relation == Relationship::ImplSelfType
                && implementation.nodes[e.target].name.as_deref() == Some("Slot")),
        "{implementation:?}"
    );
    let structural = engine
        .start_graph(
            intent(
                "Child: Base",
                vec![Relationship::Supertrait, Relationship::Contains],
            ),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    assert!(
        structural
            .traversal
            .edges
            .iter()
            .any(|e| e.relation == Relationship::Supertrait
                && structural.nodes[e.target].name.as_deref() == Some("Base")),
        "{structural:?}"
    );
    assert!(
        structural
            .traversal
            .edges
            .iter()
            .any(|e| e.relation == Relationship::Contains
                && structural.nodes[e.target].name.as_deref() == Some("associated")),
        "{structural:?}"
    );
}

#[tokio::test]
async fn visibility_and_origins_keep_local_targets_and_generated_impls_distinct() {
    let (root, configuration, file) = fixture();
    let source = "#[derive(Clone,Debug)]\nstruct Derived{n:u8}\nmod inner{fn secret(){} pub struct Fields{hidden:u8,pub shown:u8} pub fn destination(){secret();}}\nfn destination(){}\nfn format_it(){let captured=7u8;let _=format!(\"{captured} {captured}\");}\nfn other_format(){let captured=8u8;let _=format!(\"{captured}\");}\n";
    std::fs::write(&file, source).unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    for (destination, expected) in [("destination(){secret", true), ("destination(){}", false)] {
        let answer = engine
            .start_query(
                query(
                    &file,
                    "secret(){}",
                    Kind::Visibility {
                        destination: target(&file, source, destination),
                        destination_generation: Some(1),
                    },
                ),
                CancellationToken::new(),
            )
            .settle()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            answer.observed.investigation.unwrap().visible,
            Some(expected)
        );
    }
    for (field, expected) in [("hidden:u8", false), ("shown:u8", true)] {
        let answer = engine
            .start_query(
                query(
                    &file,
                    field,
                    Kind::Visibility {
                        destination: target(&file, source, "destination(){}"),
                        destination_generation: Some(1),
                    },
                ),
                CancellationToken::new(),
            )
            .settle()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            answer.observed.investigation.unwrap().visible,
            Some(expected),
            "field visibility must not inherit the public containing ADT"
        );
    }
    let mut format = query(
        &file,
        "\"{captured}",
        Kind::Origins {
            kind: crate::semantic::OriginKind::Format,
        },
    );
    let answer = engine
        .start_query(format.clone(), CancellationToken::new())
        .settle()
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        answer.availability,
        crate::semantic::Availability::Available
    ));
    let origins = answer.observed.investigation.unwrap().origins;
    assert_eq!(origins.len(), 2, "{origins:?}");
    assert!(origins.iter().all(|o| matches!(
        o.target_kind,
        crate::semantic::OriginTargetKind::Local
    ) && o.target.is_some()
        && o.occurrence.is_some()));
    assert_eq!(
        serde_json::to_value(&origins[0].target).unwrap(),
        serde_json::to_value(&origins[1].target).unwrap()
    );
    assert_ne!(
        serde_json::to_value(&origins[0].occurrence).unwrap(),
        serde_json::to_value(&origins[1].occurrence).unwrap()
    );
    for origin in &origins {
        let occurrence = origin.occurrence.as_ref().unwrap();
        assert_eq!(&source[occurrence.start..occurrence.end], "captured");
        let target = origin.target.as_ref().unwrap();
        assert_eq!(target.start, source.find("captured=7u8").unwrap());
        assert_eq!(&source[target.start..target.end], "captured");
    }
    let other = engine
        .start_query(
            query(
                &file,
                "\"{captured}\"",
                Kind::Origins {
                    kind: crate::semantic::OriginKind::Format,
                },
            ),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap()
        .observed
        .investigation
        .unwrap();
    assert_eq!(other.origins.len(), 1);
    let other_target = other.origins[0].target.as_ref().unwrap();
    assert_eq!(other_target.start, source.find("captured=8u8").unwrap());
    assert_ne!(
        other_target.start,
        origins[0].target.as_ref().unwrap().start,
        "same-spelled locals retain actual native binding identity"
    );
    format.limit = 1;
    let bounded = engine
        .start_query(format, CancellationToken::new())
        .settle()
        .await
        .unwrap()
        .unwrap()
        .observed
        .investigation
        .unwrap();
    assert_eq!(bounded.origins.len(), 1);
    assert!(bounded.limitations.iter().any(|l| l.contains("truncated")));
    let mut narrow_derive = query(
        &file,
        "Derived{",
        Kind::Origins {
            kind: crate::semantic::OriginKind::Derive,
        },
    );
    narrow_derive.limit = 1;
    let narrow_derive = engine
        .start_query(narrow_derive, CancellationToken::new())
        .settle()
        .await
        .unwrap()
        .unwrap()
        .observed
        .investigation
        .unwrap();
    assert_eq!(narrow_derive.origins.len(), 1);
    assert!(
        narrow_derive
            .limitations
            .iter()
            .any(|l| l.contains("limit reached"))
    );
    let derive = engine
        .start_query(
            query(
                &file,
                "Derived{",
                Kind::Origins {
                    kind: crate::semantic::OriginKind::Derive,
                },
            ),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap()
        .observed
        .investigation
        .unwrap();
    assert!(derive.origins.iter().any(|o| matches!(
        o.target_kind,
        crate::semantic::OriginTargetKind::OriginatingAdt
    ) && o.target.is_some()));
    let generated = derive
        .origins
        .iter()
        .filter(|o| {
            matches!(
                o.target_kind,
                crate::semantic::OriginTargetKind::GeneratedImpl
            )
        })
        .collect::<Vec<_>>();
    assert!(!generated.is_empty(), "{derive:?}");
    assert!(
        generated.iter().all(|o| o.target.is_none()
            && o.originating_adt.is_some()
            && !o.limitations.is_empty())
    );
}

#[tokio::test]
async fn assembly_origins_remain_invocation_operands_without_declaration_identity() {
    let (root, configuration, file) = fixture();
    let source = "pub fn assembly(value:usize){unsafe{core::arch::asm!(\"/* {operand} */\",operand=in(reg) value,options(nomem,nostack));}}\n";
    std::fs::write(&file, source).unwrap();
    let engine = Engine::start(root.path(), configuration).unwrap();
    let answer = engine
        .start_query(
            query(
                &file,
                "\"/* {operand}",
                Kind::Origins {
                    kind: crate::semantic::OriginKind::Format,
                },
            ),
            CancellationToken::new(),
        )
        .settle()
        .await
        .unwrap()
        .unwrap();
    let origins = answer.observed.investigation.unwrap().origins;
    assert_eq!(origins.len(), 1, "{origins:?}");
    assert!(matches!(
        origins[0].target_kind,
        crate::semantic::OriginTargetKind::AssemblyOperand
    ));
    assert_eq!(origins[0].name.as_deref(), Some("operand"));
    assert!(origins[0].target.is_none());
    assert!(origins[0].occurrence.is_some());
    assert!(!origins[0].limitations.is_empty());
}
