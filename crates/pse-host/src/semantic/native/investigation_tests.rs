// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Consequential native relationship, origin and comparison cases.
use super::tests::{fixture, query};
use super::*;

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
