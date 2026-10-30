// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;

#[test]
fn format_preflight_refuses_new_version_before_version_specific_shape() {
    let error =
        decode(br#"{"format_version":999,"index":"deliberately incompatible"}"#).unwrap_err();
    assert!(matches!(
        error,
        ApiError::Format {
            actual: 999,
            expected: 61
        }
    ));
    assert!(decode(br#"{"index":{}}"#).is_err());
    assert!(matches!(
        decode(br#"{"format_version":61,"index":"bad"}"#),
        Err(ApiError::Json(_))
    ));
}

#[test]
fn source_snapshot_digest_observes_non_rust_authored_inputs_and_refuses_symlinks() {
    let temporary = tempfile::tempdir().unwrap();
    fs::write(temporary.path().join("README.md"), "authored docs").unwrap();
    fs::create_dir(temporary.path().join("target")).unwrap();
    fs::write(temporary.path().join("target/disposable"), "build output").unwrap();
    let mut files = Vec::new();
    source_files(temporary.path(), Path::new(""), &mut files).unwrap();
    assert_eq!(
        files,
        vec![(PathBuf::from("README.md"), b"authored docs".to_vec())]
    );
    std::os::unix::fs::symlink("README.md", temporary.path().join("linked")).unwrap();
    assert!(source_files(temporary.path(), Path::new(""), &mut Vec::new()).is_err());
}

#[test]
fn authored_examples_preserve_expectations_and_hidden_setup_without_execution() {
    let docs = "```rust\n# let setup = 1;\nassert_eq!(setup, 1);\n```\n\n```no_run\nloop {}\n```\n\n```compile_fail,E0308\nlet x: u8 = \"wrong\";\n```\n\n```ignore-linux\nmissing();\n```\n\n```should_panic\npanic!();\n```\n\n```python\nprint('excluded')\n```";
    let examples = query::examples(docs);
    assert_eq!(examples.len(), 5);
    assert!(matches!(
        examples[0].expectation,
        query::Expectation::CompileAndRun
    ));
    assert_eq!(examples[0].hidden_setup, vec!["let setup = 1;"]);
    assert!(!examples[0].displayed.contains("let setup"));
    assert!(examples[0].expanded.contains("let setup"));
    assert!(matches!(
        examples[1].expectation,
        query::Expectation::CompileOnly
    ));
    assert!(matches!(
        examples[2].expectation,
        query::Expectation::CompileFail
    ));
    assert!(matches!(
        examples[3].expectation,
        query::Expectation::Ignored
    ));
    assert!(matches!(
        examples[4].expectation,
        query::Expectation::ShouldPanic
    ));
    assert!(
        examples
            .iter()
            .all(|example| example.execution == "not_run")
    );
}

#[test]
fn public_aliases_and_trait_default_methods_retain_their_public_import() {
    use rustdoc_types::{Id, Item, ItemEnum, Visibility};
    let item = |id, name: &str, inner| Item {
        id: Id(id),
        crate_id: 0,
        name: Some(name.into()),
        span: None,
        visibility: Visibility::Public,
        docs: None,
        links: Default::default(),
        attrs: Vec::new(),
        deprecation: None,
        stability: None,
        const_stability: None,
        inner,
    };
    let function = rustdoc_types::Function {
        sig: rustdoc_types::FunctionSignature {
            inputs: Vec::new(),
            output: None,
            is_c_variadic: false,
        },
        generics: rustdoc_types::Generics {
            params: Vec::new(),
            where_predicates: Vec::new(),
        },
        header: rustdoc_types::FunctionHeader {
            is_const: false,
            is_unsafe: false,
            is_async: false,
            abi: rustdoc_types::Abi::Rust,
        },
        has_body: true,
        default_unstable: None,
    };
    let mut document = rustdoc_types::Crate {
        root: Id(0),
        crate_version: Some("1.0.0".into()),
        includes_private: false,
        index: Default::default(),
        paths: Default::default(),
        external_crates: Default::default(),
        target: rustdoc_types::Target {
            triple: "x86_64-unknown-linux-gnu".into(),
            target_features: Vec::new(),
        },
        format_version: 61,
    };
    document.index.insert(
        Id(0),
        item(
            0,
            "demo",
            ItemEnum::Module(rustdoc_types::Module {
                is_crate: true,
                items: vec![Id(1), Id(2)],
                is_stripped: false,
            }),
        ),
    );
    document.index.insert(
        Id(1),
        item(
            1,
            "Alias",
            ItemEnum::Use(rustdoc_types::Use {
                source: "hidden::Original".into(),
                name: "Alias".into(),
                id: Some(Id(3)),
                is_glob: false,
            }),
        ),
    );
    document.index.insert(
        Id(2),
        item(
            2,
            "Helpful",
            ItemEnum::Trait(rustdoc_types::Trait {
                is_auto: false,
                is_unsafe: false,
                is_dyn_compatible: true,
                items: vec![Id(5)],
                generics: rustdoc_types::Generics {
                    params: Vec::new(),
                    where_predicates: Vec::new(),
                },
                bounds: Vec::new(),
                implementations: vec![Id(4)],
            }),
        ),
    );
    document.index.insert(
        Id(3),
        item(
            3,
            "Original",
            ItemEnum::Struct(rustdoc_types::Struct {
                kind: rustdoc_types::StructKind::Unit,
                generics: rustdoc_types::Generics {
                    params: Vec::new(),
                    where_predicates: Vec::new(),
                },
                impls: vec![Id(4)],
            }),
        ),
    );
    document.index.insert(
        Id(4),
        item(
            4,
            "",
            ItemEnum::Impl(rustdoc_types::Impl {
                is_unsafe: false,
                generics: rustdoc_types::Generics {
                    params: Vec::new(),
                    where_predicates: Vec::new(),
                },
                provided_trait_methods: vec!["help".into()],
                trait_: Some(rustdoc_types::Path {
                    path: "Helpful".into(),
                    id: Id(2),
                    args: None,
                }),
                for_: rustdoc_types::Type::ResolvedPath(rustdoc_types::Path {
                    path: "Original".into(),
                    id: Id(3),
                    args: None,
                }),
                items: Vec::new(),
                is_negative: false,
                is_synthetic: false,
                blanket_impl: None,
            }),
        ),
    );
    document
        .index
        .insert(Id(5), item(5, "help", ItemEnum::Function(function)));
    let paths = query::test_public_paths(&document);
    assert!(
        paths
            .iter()
            .any(|(path, import)| path == "demo::Alias::help"
                && import.as_deref() == Some("demo::Helpful"))
    );
    assert!(!paths.iter().any(|(path, _)| path == "demo::Original"));
    document.index.insert(
        Id(0),
        item(
            0,
            "demo",
            ItemEnum::Module(rustdoc_types::Module {
                is_crate: true,
                items: vec![Id(6)],
                is_stripped: false,
            }),
        ),
    );
    document.index.insert(
        Id(6),
        item(
            6,
            "dependency",
            ItemEnum::Use(rustdoc_types::Use {
                source: "dependency".into(),
                name: "dependency".into(),
                id: Some(Id(7)),
                is_glob: true,
            }),
        ),
    );
    document.paths.insert(
        Id(7),
        rustdoc_types::ItemSummary {
            crate_id: 8,
            path: vec!["dependency".into()],
            kind: rustdoc_types::ItemKind::Module,
        },
    );
    document.external_crates.insert(
        8,
        rustdoc_types::ExternalCrate {
            name: "dependency".into(),
            html_root_url: None,
            path: PathBuf::from("/isolated/libdependency.rmeta"),
        },
    );
    let routes = query::external_uses(&document, Some("Command::arg"));
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].public_prefix, "demo");
    assert_eq!(routes[0].crate_name, "dependency");
    assert_eq!(routes[0].suffix, "Command::arg");
}

#[test]
fn doc_unit_features_require_exact_package_target_source_and_version() {
    let graph=br#"{"version":1,"roots":[1],"units":[{"pkg_id":"same-name-other-source","features":["wrong"],"mode":"doc","platform":"target-triple","target":{"name":"library","src_path":"/source/lib.rs"}},{"pkg_id":"exact-source","features":["actual-doc-feature"],"mode":"doc","platform":"target-triple","target":{"name":"library","src_path":"/source/lib.rs"}}]}"#;
    let features = documentation_features(
        graph,
        "exact-source",
        "library",
        "target-triple",
        Path::new("/source/lib.rs"),
    )
    .unwrap();
    assert_eq!(features, BTreeSet::from(["actual-doc-feature".into()]));
    assert!(
        documentation_features(
            graph,
            "same-name-other-source",
            "library",
            "target-triple",
            Path::new("/source/lib.rs")
        )
        .is_err()
    );
    assert!(
        documentation_features(
            graph,
            "exact-source",
            "library",
            "other-target",
            Path::new("/source/lib.rs")
        )
        .is_err()
    );
    assert!(
        documentation_features(
            br#"{"version":2,"roots":[],"units":[]}"#,
            "exact-source",
            "library",
            "target-triple",
            Path::new("/source/lib.rs")
        )
        .is_err()
    );
}

#[test]
fn extraction_removes_reexec_transport_from_both_context_and_actual_compiler_inputs() {
    let original = Environment::from([
        ("PSE_HOST_REEXEC".into(), "per-process-marker".into()),
        ("CODEX_TOOL_CALL_ID".into(), "per-client-request".into()),
        ("_".into(), "per-shell-last-command".into()),
        ("RUSTFLAGS".into(), "--cfg meaningful".into()),
        ("CUSTOM_BUILD_INPUT".into(), "authored".into()),
    ]);
    let actual = extraction_environment(&original).unwrap();
    assert!(!actual.contains_key(OsStr::new("PSE_HOST_REEXEC")));
    assert!(!actual.contains_key(OsStr::new("_")));
    assert!(!actual.contains_key(OsStr::new("CODEX_TOOL_CALL_ID")));
    assert_eq!(
        actual.get(OsStr::new("RUSTFLAGS")),
        original.get(OsStr::new("RUSTFLAGS"))
    );
    assert_eq!(
        actual.get(OsStr::new("CUSTOM_BUILD_INPUT")),
        original.get(OsStr::new("CUSTOM_BUILD_INPUT"))
    );
}
