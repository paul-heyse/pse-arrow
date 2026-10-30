// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use crate::correctness::CompilerUnit;
use serde_json::{Value, json};

fn metadata() -> Metadata {
    let target = |name: &str, kind: &str, required: &[&str]| {
        json!({
            "name": name, "kind": [kind], "crate_types": ["bin"],
            "src_path": format!("/workspace/alpha/{kind}/{name}.rs"),
            "required-features": required, "edition": "2024"
        })
    };
    let package = |name: &str, targets: Vec<Value>| {
        json!({
            "name": name, "version":"0.1.0", "id": format!("path+file:///workspace/{name}#0.1.0"),
            "source":null, "manifest_path":format!("/workspace/{name}/Cargo.toml"),
            "dependencies":[], "targets":targets,
            "features":{"default":[], "native":[], "canonical":[], "negative":[]}
        })
    };
    serde_json::from_value(json!({
        "packages":[package("alpha",vec![target("alpha","lib",&[]), target("same","test",&["native"]),target("same","bin",&["canonical"])]),package("beta",vec![target("beta","lib",&[])]),package("gamma",vec![target("gamma","lib",&[])])],
        "workspace_members":["path+file:///workspace/alpha#0.1.0","path+file:///workspace/beta#0.1.0","path+file:///workspace/gamma#0.1.0"],
        "workspace_default_members":["path+file:///workspace/alpha#0.1.0"],
        "workspace_root":"/workspace", "target_directory":"/workspace/target", "version":1
    })).expect("independent metadata fixture")
}
fn intent(target: Option<(&str, Option<&str>)>) -> PackageIntent {
    PackageIntent {
        package: "alpha".into(),
        features: Features::default(),
        target: target.map(|(name, kind)| TargetIntent {
            name: name.into(),
            kind: kind.map(str::to_owned),
        }),
    }
}
fn request(intents: Vec<PackageIntent>) -> Request {
    Request {
        operation: Operation::Test,
        scope: Scope::Packages(intents),
        platform: None,
        filters: Filters::default(),
        ignored: IgnoredPolicy::Default,
    }
}

#[test]
fn workspace_exclusions_and_default_members_keep_exact_package_ids() {
    let data = metadata();
    let mut input = request(vec![]);
    input.scope = Scope::Workspace {
        exclude: vec!["alpha".into(), "gamma".into()],
        features: Features::default(),
    };
    let actual = plan(&data, input).unwrap();
    assert_eq!(actual.groups().len(), 1);
    assert_eq!(
        actual.groups()[0].roots()[0].package_id(),
        "path+file:///workspace/beta#0.1.0"
    );
    assert_eq!(
        actual.groups()[0].arguments(),
        &[
            "test",
            "--locked",
            "-p",
            "path+file:///workspace/beta#0.1.0"
        ]
    );
    let mut input = request(vec![]);
    input.scope = Scope::DefaultMembers {
        features: Features::default(),
    };
    assert_eq!(
        plan(&data, input).unwrap().groups()[0].roots()[0].package_name(),
        "alpha"
    );
}

#[test]
fn explicit_target_activates_only_its_requirements_and_ambiguous_names_refuse() {
    let data = metadata();
    assert!(plan(&data, request(vec![intent(Some(("same", None)))])).is_err());
    let actual = plan(&data, request(vec![intent(Some(("same", Some("test"))))])).unwrap();
    assert_eq!(
        actual.groups()[0].roots()[0].features().enabled,
        BTreeSet::from(["native".into()])
    );
    assert_eq!(
        actual.groups()[0].arguments(),
        &[
            "test",
            "--locked",
            "-p",
            "path+file:///workspace/alpha#0.1.0",
            "--test",
            "same",
            "--features",
            "alpha/native"
        ]
    );
    assert!(
        plan(&data, request(vec![intent(None)])).unwrap().groups()[0].roots()[0]
            .features()
            .enabled
            .is_empty()
    );
    let mut prohibited = intent(Some(("same", Some("test"))));
    prohibited.features.absent.insert("native".into());
    assert!(plan(&data, request(vec![prohibited])).is_err());
}

#[test]
fn independent_feature_absence_and_defaults_never_become_a_union() {
    let mut positive = intent(Some(("same", Some("test"))));
    positive.features.defaults = false;
    let mut negative = intent(None);
    negative.features.defaults = false;
    negative.features.absent.insert("native".into());
    let actual = plan(
        &metadata(),
        request(vec![positive.clone(), negative, positive]),
    )
    .unwrap();
    assert_eq!(actual.groups().len(), 2);
    assert_eq!(
        actual.groups()[0].roots()[0].features().enabled,
        BTreeSet::from(["native".into()])
    );
    assert!(actual.groups()[1].roots()[0].features().enabled.is_empty());
    assert_eq!(
        actual.groups()[1].roots()[0].features().absent,
        BTreeSet::from(["native".into()])
    );
    assert!(
        actual
            .groups()
            .iter()
            .all(|group| group.arguments().contains(&"--no-default-features".into()))
    );
}

fn admitted(arguments: &[OsString], features: &[&str]) -> correctness::Resolution {
    correctness::Resolution {
        arguments: arguments.to_vec(),
        packages: vec!["alpha".into()],
        native_capabilities: BTreeSet::new(),
        units: vec![CompilerUnit {
            package_id: "path+file:///workspace/alpha#0.1.0".into(),
            package_name: "alpha".into(),
            target_name: "alpha".into(),
            target_kinds: vec!["lib".into()],
            source_path: "/workspace/alpha/lib/alpha.rs".into(),
            features: features.iter().map(|name| String::from(*name)).collect(),
            platform: None,
            mode: "test".into(),
            root: true,
        }],
    }
}

#[test]
fn actual_feature_absence_is_checked_after_cargo_admission() {
    let mut negative = intent(None);
    negative.features.absent.insert("native".into());
    let planned = plan(&metadata(), request(vec![negative])).unwrap();
    assert!(
        planned
            .clone()
            .resolve_with(|args| Ok(admitted(args, &["native"])))
            .is_err()
    );
    assert!(planned.resolve_with(|args| Ok(admitted(args, &[]))).is_ok());
}

#[test]
fn explain_and_execution_retain_platform_filters_ignored_and_pending_listing() {
    let mut input = request(vec![intent(None)]);
    input.platform = Some("x86_64-unknown-linux-gnu".into());
    input.filters = Filters {
        words: vec!["duplicate".into()],
        expressions: vec!["package(alpha) & test(duplicate)".into()],
    };
    input.ignored = IgnoredPolicy::Only;
    let planned = plan(&metadata(), input).unwrap();
    let expected = planned.clone();
    let resolved = planned
        .resolve_with(|args| {
            let mut actual = admitted(args, &[]);
            actual.units[0].platform = Some("x86_64-unknown-linux-gnu".into());
            Ok(actual)
        })
        .unwrap();
    assert_eq!(resolved.plan(), &expected);
    assert_eq!(resolved.plan().ignored(), IgnoredPolicy::Only);
    assert_eq!(resolved.plan().filters().words, ["duplicate"]);
    assert_eq!(
        resolved.admitted()[0].arguments,
        expected.groups()[0]
            .arguments
            .iter()
            .map(OsString::from)
            .collect::<Vec<_>>()
    );
    assert!(resolved.needs_binary_listing());
    assert!(resolved.native_capabilities().is_empty());
}

#[test]
fn admission_cannot_substitute_a_same_named_target_in_another_package_or_kind() {
    let planned = plan(
        &metadata(),
        request(vec![intent(Some(("same", Some("test"))))]),
    )
    .unwrap();
    assert!(
        planned
            .resolve_with(|args| Ok(admitted(args, &["native"])))
            .is_err()
    );
    let planned = plan(&metadata(), request(vec![intent(None)])).unwrap();
    assert!(
        planned
            .resolve_with(|args| {
                let mut actual = admitted(args, &[]);
                actual.units[0].package_id = "another-source#alpha@0.1.0".into();
                Ok(actual)
            })
            .is_err()
    );
}

#[test]
fn empty_unknown_and_check_test_policy_conflicts_refuse() {
    assert!(plan(&metadata(), request(vec![])).is_err());
    let mut unknown = intent(None);
    unknown.package = "missing".into();
    assert!(plan(&metadata(), request(vec![unknown])).is_err());
    let mut check = request(vec![intent(None)]);
    check.operation = Operation::Check;
    check.ignored = IgnoredPolicy::All;
    assert!(plan(&metadata(), check).is_err());
}

fn named_intent(name: &str) -> PackageIntent {
    PackageIntent {
        package: name.into(),
        target: None,
        features: Features::default(),
    }
}
fn multi_admitted(arguments: &[OsString], roots: &[(&str, &[&str])]) -> correctness::Resolution {
    let mut resolution = admitted(arguments, &[]);
    resolution.packages = roots.iter().map(|(name, _)| String::from(*name)).collect();
    resolution.units = roots
        .iter()
        .map(|(name, features)| CompilerUnit {
            package_id: format!("path+file:///workspace/{name}#0.1.0"),
            package_name: String::from(*name),
            target_name: String::from(*name),
            target_kinds: vec!["lib".into()],
            source_path: format!("/workspace/alpha/lib/{name}.rs"),
            features: features.iter().map(|name| String::from(*name)).collect(),
            platform: None,
            mode: "test".into(),
            root: true,
        })
        .collect();
    resolution
}

#[test]
fn compatible_roots_share_one_real_admission_and_qualified_features() {
    let mut alpha = named_intent("alpha");
    alpha.features.enabled.insert("native".into());
    let mut beta = named_intent("beta");
    beta.features.enabled.insert("canonical".into());
    let planned = plan(&metadata(), request(vec![alpha, beta])).unwrap();
    assert_eq!(planned.groups().len(), 1);
    assert_eq!(
        planned.groups()[0]
            .roots()
            .iter()
            .map(Root::package_name)
            .collect::<Vec<_>>(),
        ["alpha", "beta"]
    );
    assert_eq!(
        planned.groups()[0].arguments(),
        [
            "test",
            "--locked",
            "-p",
            "path+file:///workspace/alpha#0.1.0",
            "-p",
            "path+file:///workspace/beta#0.1.0",
            "--features",
            "alpha/native,beta/canonical"
        ]
    );
    let mut calls = 0;
    let resolved = planned
        .resolve_with(|arguments| {
            calls += 1;
            Ok(multi_admitted(
                arguments,
                &[("alpha", &["native"]), ("beta", &["canonical"])],
            ))
        })
        .unwrap();
    assert_eq!(calls, 1, "one set admission, not two scalar calls");
    assert_eq!(resolved.admitted().len(), 1);
    assert_eq!(
        resolved.admitted()[0]
            .units
            .iter()
            .filter(|unit| unit.root)
            .count(),
        2
    );
}

#[test]
fn defaults_all_and_target_selectors_preserve_separate_contexts() {
    let alpha = named_intent("alpha");
    let mut beta = named_intent("beta");
    beta.features.defaults = false;
    let mut gamma = named_intent("gamma");
    gamma.features.all = true;
    let planned = plan(&metadata(), request(vec![alpha, beta, gamma])).unwrap();
    assert_eq!(planned.groups().len(), 3);
    let alpha = intent(Some(("same", Some("test"))));
    let beta = named_intent("beta");
    assert_eq!(
        plan(&metadata(), request(vec![alpha, beta]))
            .unwrap()
            .groups()
            .len(),
        2
    );
}

#[test]
fn cargo_feature_absence_evidence_splits_only_the_conflicting_root() {
    let mut alpha = named_intent("alpha");
    alpha.features.absent.insert("native".into());
    let planned = plan(
        &metadata(),
        request(vec![alpha, named_intent("beta"), named_intent("gamma")]),
    )
    .unwrap();
    assert_eq!(planned.groups().len(), 1);
    let mut calls = Vec::new();
    let resolved = planned
        .resolve_with(|arguments| {
            let names: Vec<_> = arguments
                .windows(2)
                .filter(|pair| pair[0] == "-p")
                .map(|pair| {
                    pair[1]
                        .to_str()
                        .unwrap()
                        .split('/')
                        .next_back()
                        .unwrap()
                        .split('#')
                        .next()
                        .unwrap()
                })
                .collect();
            calls.push(
                names
                    .iter()
                    .map(|name| String::from(*name))
                    .collect::<Vec<_>>(),
            );
            Ok(match names.as_slice() {
                ["alpha", "beta", "gamma"] => multi_admitted(
                    arguments,
                    &[("alpha", &["native"]), ("beta", &[]), ("gamma", &[])],
                ),
                ["alpha"] => multi_admitted(arguments, &[("alpha", &[])]),
                ["beta", "gamma"] => multi_admitted(arguments, &[("beta", &[]), ("gamma", &[])]),
                _ => panic!("unexpected grouped admission"),
            })
        })
        .unwrap();
    assert_eq!(
        calls,
        [
            vec!["alpha", "beta", "gamma"],
            vec!["alpha"],
            vec!["beta", "gamma"]
        ]
    );
    assert_eq!(
        resolved
            .plan()
            .groups()
            .iter()
            .map(|group| group.roots().len())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(resolved.admitted().len(), 2);
}

#[test]
fn missing_extra_or_substituted_roots_refuse_without_splitting() {
    let planned = plan(
        &metadata(),
        request(vec![named_intent("alpha"), named_intent("beta")]),
    )
    .unwrap();
    for roots in [
        vec![("alpha", &[][..])],
        vec![("alpha", &[][..]), ("beta", &[][..]), ("gamma", &[][..])],
    ] {
        let mut calls = 0;
        assert!(
            planned
                .clone()
                .resolve_with(|arguments| {
                    calls += 1;
                    Ok(multi_admitted(arguments, &roots))
                })
                .is_err()
        );
        assert_eq!(calls, 1);
    }
    let mut calls = 0;
    assert!(
        planned
            .resolve_with(|_| {
                calls += 1;
                Err(invalid("ordinary Cargo failure"))
            })
            .is_err()
    );
    assert_eq!(calls, 1, "arbitrary errors never trigger a fallback");
}

#[test]
fn root_absence_does_not_reject_a_distinct_host_dependency_context() {
    let mut alpha = named_intent("alpha");
    alpha.features.absent.insert("native".into());
    let mut input = request(vec![alpha, named_intent("beta")]);
    input.platform = Some("target-triple".into());
    let planned = plan(&metadata(), input).unwrap();
    let resolved = planned
        .resolve_with(|arguments| {
            let mut resolution = multi_admitted(arguments, &[("alpha", &[]), ("beta", &[])]);
            for unit in &mut resolution.units {
                unit.platform = Some("target-triple".into());
            }
            let mut host = resolution.units[0].clone();
            host.root = false;
            host.platform = None;
            host.mode = "build".into();
            host.features.insert("native".into());
            resolution.units.push(host);
            Ok(resolution)
        })
        .unwrap();
    let units = &resolved.admitted()[0].units;
    assert_eq!(units.len(), 3);
    assert_eq!(
        (units[0].platform.as_deref(), units[0].mode.as_str()),
        (Some("target-triple"), "test")
    );
    assert_eq!(
        (units[2].platform.as_deref(), units[2].mode.as_str()),
        (None, "build")
    );
    assert!(units[2].features.contains("native"));
}

#[test]
fn explicit_target_and_host_proc_macro_roots_map_without_erasing_context() {
    let mut data = serde_json::to_value(metadata()).unwrap();
    data["packages"][1]["targets"][0]["kind"] = json!(["proc-macro"]);
    let data = serde_json::from_value(data).unwrap();
    let alpha = intent(Some(("alpha", Some("lib"))));
    let mut beta = named_intent("beta");
    beta.target = Some(TargetIntent {
        name: "beta".into(),
        kind: Some("proc-macro".into()),
    });
    let mut input = request(vec![alpha, beta]);
    input.platform = Some("target-triple".into());
    let planned = plan(&data, input).unwrap();
    assert_eq!(planned.groups().len(), 1);
    let resolved = planned
        .resolve_with(|arguments| {
            let mut resolution = multi_admitted(arguments, &[("alpha", &[]), ("beta", &[])]);
            resolution.units[0].platform = Some("target-triple".into());
            resolution.units[1].target_kinds = vec!["proc-macro".into()];
            resolution.units[1].mode = "build".into();
            Ok(resolution)
        })
        .unwrap();
    assert_eq!(resolved.admitted()[0].units[1].platform, None);
    assert_eq!(resolved.admitted()[0].units[1].mode, "build");
    let planned = plan(&data, request(vec![intent(Some(("same", Some("test"))))])).unwrap();
    assert!(
        planned
            .resolve_with(|arguments| {
                let mut resolution = multi_admitted(arguments, &[("alpha", &["native"])]);
                resolution.units[0].target_name = "same".into();
                resolution.units[0].target_kinds = vec!["bin".into()];
                resolution.units[0].source_path = "/workspace/alpha/bin/same.rs".into();
                Ok(resolution)
            })
            .is_err()
    );
}

#[test]
fn real_host_selection_admits_no_arrow_or_provider_and_builds_no_binary() {
    let data = cargo_metadata::MetadataCommand::new()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .no_deps()
        .exec()
        .unwrap();
    let mut host = intent(None);
    host.package = "pse-host".into();
    host.target = Some(TargetIntent {
        name: "pse_host".into(),
        kind: Some("lib".into()),
    });
    let mut diagnostics = named_intent("pse-diagnostics");
    diagnostics.target = Some(TargetIntent {
        name: "pse_diagnostics".into(),
        kind: Some("lib".into()),
    });
    let resolved = plan(&data, request(vec![host, diagnostics]))
        .unwrap()
        .resolve()
        .unwrap();
    assert_eq!(resolved.plan().groups().len(), 1);
    assert_eq!(resolved.plan().groups()[0].roots().len(), 2);
    assert_eq!(
        resolved.admitted()[0]
            .packages
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["pse-host", "pse-diagnostics"])
    );
    assert!(resolved.native_capabilities().is_empty());
    assert!(
        resolved.admitted()[0]
            .units
            .iter()
            .all(|unit| !unit.package_name.starts_with("arrow"))
    );
    assert!(
        resolved.admitted()[0]
            .arguments
            .iter()
            .all(|argument| !argument.to_string_lossy().contains("force-validate"))
    );
    assert!(resolved.needs_binary_listing());
}

#[test]
fn real_arrow_multi_root_admission_validates_each_root_and_rejects_negative_policy() {
    let data = cargo_metadata::MetadataCommand::new()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .no_deps()
        .exec()
        .unwrap();
    let selected: Vec<_> = [
        ("pse-columnar", "pse_columnar"),
        ("pse-relations", "pse_relations"),
    ]
    .into_iter()
    .map(|(package, target)| {
        let mut intent = named_intent(package);
        intent.target = Some(TargetIntent {
            name: target.into(),
            kind: Some("lib".into()),
        });
        intent
    })
    .collect();
    let planned = plan(&data, request(selected.clone())).unwrap();
    assert_eq!(planned.groups().len(), 1);
    let resolved = planned.resolve().unwrap();
    let resolution = &resolved.admitted()[0];
    let roots: Vec<_> = resolution.units.iter().filter(|unit| unit.root).collect();
    assert_eq!(
        roots
            .iter()
            .map(|unit| unit.package_name.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["pse-columnar", "pse-relations"])
    );
    assert!(
        roots
            .iter()
            .all(|unit| unit.features.contains("force-validate"))
    );
    let leaves: Vec<_> = resolution
        .units
        .iter()
        .filter(|unit| unit.package_name == "arrow-array")
        .collect();
    assert!(!leaves.is_empty());
    assert!(
        leaves
            .iter()
            .all(|unit| unit.features.contains("force_validate"))
    );
    let mut negative = selected[0].clone();
    negative.features.absent.insert("force-validate".into());
    assert!(
        plan(&data, request(vec![negative]))
            .unwrap()
            .resolve()
            .is_err()
    );
}

#[test]
fn real_separate_target_groups_share_invocation_metadata_without_building() {
    let data = cargo_metadata::MetadataCommand::new()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .no_deps()
        .exec()
        .unwrap();
    let selected = [("pse_host", "lib"), ("pse", "bin")]
        .into_iter()
        .map(|(name, kind)| {
            let mut intent = named_intent("pse-host");
            intent.target = Some(TargetIntent {
                name: name.into(),
                kind: Some(kind.into()),
            });
            intent
        })
        .collect();
    let planned = plan(&data, request(selected)).unwrap();
    assert_eq!(planned.groups().len(), 2);
    let cargo = correctness::real_cargo().unwrap();
    let mut context = correctness::AdmissionContext::default();
    let mut metadata_calls = 0;
    let mut graph_calls = 0;
    let resolved = planned
        .resolve_with(|arguments| {
            context.resolve_with(arguments, |arguments| {
                if arguments[0] == "metadata" {
                    metadata_calls += 1;
                }
                if arguments.iter().any(|argument| argument == "--unit-graph") {
                    graph_calls += 1;
                }
                correctness::capture(&cargo, arguments)
            })
        })
        .unwrap();
    assert_eq!(resolved.admitted().len(), 2);
    assert_eq!((metadata_calls, graph_calls), (1, 2));
}

#[test]
fn real_native_study_explain_derives_its_target_features_without_preparation() {
    let data = cargo_metadata::MetadataCommand::new()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .no_deps()
        .exec()
        .unwrap();
    let mut selected = intent(Some(("native_study", Some("test"))));
    selected.package = "pse-runtime".into();
    selected.features.defaults = false;
    let planned = plan(&data, request(vec![selected])).unwrap();
    assert_eq!(
        planned.groups()[0].roots()[0]
            .target()
            .unwrap()
            .required_features,
        BTreeSet::from(["native-study-tests".into(), "solver-kinsol".into()])
    );
    let before = serde_json::to_value(&planned).unwrap();
    let resolved = planned.resolve().unwrap();
    assert_eq!(serde_json::to_value(resolved.plan()).unwrap(), before);
    assert_eq!(
        resolved.native_capabilities(),
        BTreeSet::from(["compiler".into(), "klu".into()])
    );
    assert!(resolved.admitted()[0].arguments.iter().any(|argument| {
        argument
            .to_string_lossy()
            .contains("pse-runtime/force-validate")
    }));
    assert!(resolved.needs_binary_listing());
}
