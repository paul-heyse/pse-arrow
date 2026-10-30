// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use clap::{Arg, ArgAction, ArgGroup, builder::PossibleValue};
use description::{CommandMetadata, Information, OsValue, SecretDefaults};
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture() -> Command {
    Command::new("fixture").subcommand(
        Command::new("prepare")
            .arg(
                Arg::new("mode")
                    .long("mode")
                    .env("CATALOG_MODE")
                    .default_value("fast")
                    .value_parser([
                        PossibleValue::new("fast").alias("quick").help("fast mode"),
                        PossibleValue::new("private").hide(true),
                    ]),
            )
            .arg(
                Arg::new("read")
                    .long("read")
                    .action(ArgAction::SetTrue)
                    .conflicts_with("write"),
            )
            .arg(
                Arg::new("write")
                    .long("write")
                    .action(ArgAction::SetTrue)
                    .requires("mode"),
            )
            .group(
                ArgGroup::new("operation")
                    .args(["read", "write"])
                    .required(true),
            )
            .arg(
                Arg::new("secret")
                    .long("secret")
                    .default_value("private-not-for-export")
                    .add(SecretDefaults),
            )
            .add(CommandMetadata {
                examples: vec![vec!["prepare".into(), "--read".into()]],
                output_schemas: vec!["urn:fixture:response:v1".into()],
                invocation_route: vec!["fixture".into(), "prepare".into()],
                features: Vec::new(),
            }),
    )
}
#[test]
fn actual_parser_metadata_preserves_values_groups_conflicts_and_private_rule_limits() {
    let page = description::commands(fixture(), &["prepare".into()], false, &Window::default());
    assert_eq!(page.availability, Availability::Available);
    let record = &page.records[0];
    let mode = record
        .arguments
        .iter()
        .find(|arg| arg.id == "mode")
        .unwrap();
    assert_eq!(mode.environment, Some(OsValue::Utf8("CATALOG_MODE".into())));
    let Information::Available(values) = &mode.possible_values else {
        panic!("finite parser domain absent")
    };
    assert_eq!(values[0].value, "fast");
    assert_eq!(values[0].aliases, vec!["quick"]);
    assert!(values[1].hidden);
    let read = record
        .arguments
        .iter()
        .find(|arg| arg.id == "read")
        .unwrap();
    assert!(read.conflicts.contains(&"write".to_owned()));
    let group = record
        .groups
        .iter()
        .find(|group| group.id == "operation")
        .unwrap();
    assert!(group.required);
    assert!(!group.multiple);
    let write = record
        .arguments
        .iter()
        .find(|arg| arg.id == "write")
        .unwrap();
    assert!(matches!(write.requires, Information::Unavailable { .. }));
    assert!(
        !serde_json::to_string(&page)
            .unwrap()
            .contains("private-not-for-export")
    );
    assert!(record.short_help.contains("--mode"));
    assert!(record.long_help.contains("fast mode"));
    assert!(
        matches!(&record.output_schemas,Information::Available(values) if values==&vec!["urn:fixture:response:v1".to_owned()])
    );
    assert_eq!(record.examples, vec![vec!["prepare", "--read"]]);
}
static SELECTED: AtomicUsize = AtomicUsize::new(0);
static UNRELATED: AtomicUsize = AtomicUsize::new(0);
#[test]
fn scoped_queries_build_selected_deferred_nodes_without_hydrating_siblings() {
    fn deferred() -> Command {
        Command::new("fixture")
            .subcommand(Command::new("selected").defer(|command| {
                SELECTED.fetch_add(1, Ordering::SeqCst);
                command.arg(Arg::new("value").long("value"))
            }))
            .subcommand(Command::new("unrelated").defer(|command| {
                UNRELATED.fetch_add(1, Ordering::SeqCst);
                command.arg(Arg::new("other").long("other"))
            }))
    }
    SELECTED.store(0, Ordering::SeqCst);
    UNRELATED.store(0, Ordering::SeqCst);
    let selected =
        description::commands(deferred(), &["selected".into()], false, &Window::default());
    assert_eq!(selected.records[0].arguments[0].id, "value");
    assert_eq!(SELECTED.load(Ordering::SeqCst), 1);
    assert_eq!(UNRELATED.load(Ordering::SeqCst), 0);
    let all = description::commands(deferred(), &[], true, &Window::default());
    assert_eq!(all.records.len(), 3);
    assert_eq!(SELECTED.load(Ordering::SeqCst), 2);
    assert_eq!(UNRELATED.load(Ordering::SeqCst), 1);
}
#[test]
fn full_export_lists_every_visible_host_command_and_does_not_publish_private_verbs() {
    let page = commands(&[], true, &Window::default());
    assert!(
        page.records
            .iter()
            .any(|record| record.path == ["native", "prepare"])
    );
    assert!(
        page.records
            .iter()
            .any(|record| record.path == ["telemetry-service", "inspect"])
    );
    assert!(!page.records.iter().any(|record| {
        record
            .path
            .iter()
            .any(|part| part == "boundary" || part == "agent-hook" || part.starts_with("__"))
    }));
    assert!(
        page.records
            .iter()
            .all(|record| !record.short_help.is_empty() && !record.long_help.is_empty())
    );
    assert_eq!(page.exhaustiveness, Exhaustiveness::Exhaustive);
    assert!(page.records.len() > 20);
}
#[test]
fn unknown_custom_domain_is_not_an_available_finite_empty_domain() {
    let tree = Command::new("fixture")
        .arg(Arg::new("open").value_parser(|value: &str| Ok::<_, String>(value.to_owned())));
    let page = description::candidates(
        tree,
        &[],
        "open",
        &Window::default(),
        |_| -> Result<CandidateSet, ()> { panic!("unassociated provider called") },
    )
    .unwrap();
    assert_eq!(page.availability, Availability::Unknown);
    assert_eq!(page.exhaustiveness, Exhaustiveness::Unknown);
    assert_eq!(page.known_matches, None);
    assert!(!page.limitations.is_empty());
    let empty = Command::new("fixture").arg(Arg::new("empty").value_parser(
        clap::builder::PossibleValuesParser::new(Vec::<PossibleValue>::new()),
    ));
    let page = description::candidates(empty, &[], "empty", &Window::default(), |_| {
        Ok::<_, ()>(CandidateSet::unavailable("unused", "unused"))
    })
    .unwrap();
    assert_eq!(page.availability, Availability::Available);
    assert_eq!(page.exhaustiveness, Exhaustiveness::Exhaustive);
    assert_eq!(page.known_matches, Some(0));
}
#[test]
fn providers_are_lazy_and_prefix_pages_retain_source_coverage_and_continuation() {
    let calls = AtomicUsize::new(0);
    let tree = Command::new("fixture")
        .arg(
            Arg::new("selected")
                .long("selected")
                .add(CandidateProvider {
                    name: "selected-domain".into(),
                }),
        )
        .arg(
            Arg::new("unrelated")
                .long("unrelated")
                .add(CandidateProvider {
                    name: "unrelated-domain".into(),
                }),
        );
    let page = description::candidates(
        tree,
        &[],
        "selected",
        &Window {
            prefix: "p".into(),
            offset: 0,
            limit: Some(1),
        },
        |name| {
            calls.fetch_add(1, Ordering::SeqCst);
            assert_eq!(name, "selected-domain");
            Ok::<_, ()>(suggestions(
                Source {
                    owner: name.into(),
                    generation: Some("snapshot-1".into()),
                    freshness: Freshness::RetainedSnapshot,
                },
                Exhaustiveness::Open,
                ["prefix-b", "other", "prefix-a"]
                    .into_iter()
                    .map(|value| Candidate {
                        value: value.into(),
                        aliases: Vec::new(),
                        help: None,
                        hidden: false,
                    })
                    .collect(),
                vec!["open domain".into()],
            ))
        },
    )
    .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(page.records[0].value, "prefix-a");
    assert_eq!(page.continuation, Some(1));
    assert_eq!(page.known_matches, Some(2));
    assert_eq!(page.exhaustiveness, Exhaustiveness::Open);
    assert_eq!(page.source.generation.as_deref(), Some("snapshot-1"));
}
fn metadata() -> cargo_metadata::Metadata {
    let target = |name: &str, kind: &str, required: Vec<&str>| serde_json::json!({"name":name,"kind":[kind],"crate_types":["bin"],"src_path":format!("/fixture/src/{name}.rs"),"edition":"2024","required-features":required});
    serde_json::from_value(serde_json::json!({"packages":[{"name":"alpha","version":"0.1.0","id":"path+file:///fixture#alpha@0.1.0","manifest_path":"/fixture/Cargo.toml","dependencies":[],"targets":[target("alpha","lib",vec![]),target("journey","test",vec!["native"])],"features":{"default":[],"native":[]}}],"workspace_members":["path+file:///fixture#alpha@0.1.0"],"workspace_default_members":["path+file:///fixture#alpha@0.1.0"],"workspace_root":"/fixture","target_directory":"/fixture/target","version":1})).unwrap()
}
#[test]
fn retained_metadata_yields_real_target_intentions_without_invoking_cargo() {
    let metadata = metadata();
    let context = Context {
        metadata: Some(&metadata),
        metadata_generation: Some("retained-7"),
        metadata_freshness: Some(Freshness::RetainedSnapshot),
        ..Context::default()
    };
    let targets = context.provide(Domain::Targets.name()).unwrap();
    assert_eq!(
        targets
            .values
            .iter()
            .map(|value| value.value.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha:alpha", "alpha:journey"]
    );
    assert_eq!(targets.source.generation.as_deref(), Some("retained-7"));
    assert_eq!(targets.source.freshness, Freshness::RetainedSnapshot);
    assert!(
        targets.values[1]
            .help
            .as_ref()
            .unwrap()
            .contains("required_features=native")
    );
    let absent = Context::default().provide(Domain::Targets.name()).unwrap();
    assert_eq!(absent.availability, Availability::Unavailable);
    assert_eq!(absent.exhaustiveness, Exhaustiveness::Unknown);
}
#[test]
fn native_atom_candidates_come_from_validation_owner_and_do_not_claim_all_combinations() {
    let page = candidates(
        &["native".into(), "prepare".into()],
        "capabilities",
        &Window::default(),
        &Context::default(),
    )
    .unwrap();
    assert_eq!(page.availability, Availability::Available);
    assert_eq!(page.exhaustiveness, Exhaustiveness::Open);
    assert_eq!(page.records.len(), crate::native::capability_names().len());
    for candidate in page.records {
        crate::native::validate_capabilities(&candidate.value).unwrap();
    }
}
#[test]
fn bound_absent_invocation_directory_is_known_empty_and_reads_create_nothing() {
    let root = tempfile::tempdir().unwrap();
    let env = [("XDG_STATE_HOME".into(), root.path().as_os_str().to_owned())].into();
    let state =
        crate::paths::CheckoutState::from_paths(root.path(), &root.path().join("common"), &env)
            .unwrap();
    let store = InvocationStore::new(state);
    let context = Context {
        invocations: Some(&store),
        ..Context::default()
    };
    let set = context.provide(Domain::Invocations.name()).unwrap();
    assert_eq!(set.availability, Availability::Available);
    assert_eq!(set.exhaustiveness, Exhaustiveness::Exhaustive);
    assert!(set.values.is_empty());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}
#[test]
fn description_pages_have_serialization_contract_schemas() {
    let page = commands(&["native".into()], true, &Window::default());
    let schema = schemars::generate::SchemaSettings::draft2020_12()
        .for_serialize()
        .into_generator()
        .into_root_schema_for::<Page<description::Description>>();
    let value = serde_json::to_value(page).unwrap();
    let validator = jsonschema::validator_for(schema.as_value()).unwrap();
    assert!(
        validator.is_valid(&value),
        "{:?}",
        validator
            .iter_errors(&value)
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
    );
}
#[test]
fn zero_page_limits_refuse_without_reading_a_provider() {
    let tree = Command::new("fixture").arg(Arg::new("value").add(Domain::Targets.association()));
    let page = description::candidates(
        tree,
        &[],
        "value",
        &Window {
            limit: Some(0),
            ..Window::default()
        },
        |_| -> Result<CandidateSet, ()> { panic!("invalid query evaluated provider") },
    )
    .unwrap();
    assert_eq!(page.availability, Availability::Unavailable);
    assert_eq!(page.known_matches, None);
    assert!(page.continuation.is_none());
}
#[test]
fn non_unicode_declared_defaults_keep_their_original_platform_bytes() {
    use std::os::unix::ffi::OsStringExt as _;
    let original = std::ffi::OsString::from_vec(vec![b'a', 0xff, b'b']);
    let tree =
        Command::new("fixture").arg(Arg::new("path").long("path").default_value_os(original));
    let page = description::commands(tree, &[], false, &Window::default());
    let arg = page.records[0]
        .arguments
        .iter()
        .find(|arg| arg.id == "path")
        .unwrap();
    assert!(
        matches!(&arg.defaults,Information::Available(values) if values==&vec![OsValue::PlatformBytes(vec![b'a',0xff,b'b'])])
    );
}
