// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual immutable native worlds refine original candidates and retained acquired APIs.
use pse_host::investigation::*;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::PathBuf,
    process::{Command, Output},
};

const SOURCE: &str = "pub trait Operation { fn run(&self) -> u32; }\npub struct One;\npub struct Two;\nimpl Operation for One { fn run(&self) -> u32 { 11 } }\nimpl Operation for Two { fn run(&self) -> u32 { 22 } }\npub fn increment(value: u32) -> u32 { value + 1 }\npub fn invoke() { let a = One; let b = Two; let _ = a.run(); let _ = b.run(); let _ = increment(5); }\n#[test] fn selected() { assert_eq!(increment(1), 2); }\n";
struct Fixture {
    project: tempfile::TempDir,
    state: tempfile::TempDir,
    root: PathBuf,
    prepared: PathBuf,
    client: PathBuf,
    stopped: std::cell::Cell<bool>,
}
impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_owned();
        let prepared=pse_host::generation::prepared(&root).unwrap().expect("explicitly prepare the native host and native-analysis tool before this native public journey").executable(&root,pse_host::generation::HostTool::Pse).unwrap();
        let project = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        std::fs::create_dir(project.path().join("src")).unwrap();
        std::fs::create_dir(project.path().join(".cargo")).unwrap();
        std::fs::copy(
            root.join("rust-toolchain.toml"),
            project.path().join("rust-toolchain.toml"),
        )
        .unwrap();
        std::fs::write(project.path().join(".cargo/config.toml"), "").unwrap();
        std::fs::write(
            project.path().join("Cargo.toml"),
            "[package]\nname=\"composed-native-control\"\nversion=\"0.0.0\"\nedition=\"2024\"\n",
        )
        .unwrap();
        std::fs::write(
            project.path().join("Cargo.lock"),
            "version=4\n[[package]]\nname=\"composed-native-control\"\nversion=\"0.0.0\"\n",
        )
        .unwrap();
        std::fs::write(project.path().join("src/lib.rs"), SOURCE).unwrap();
        let initialized = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(project.path())
            .output()
            .unwrap();
        assert!(initialized.status.success(), "{initialized:?}");
        Self {
            project,
            state,
            root,
            prepared,
            client: std::env::var_os("CARGO_BIN_EXE_pse")
                .expect("native runner supplies current public executable")
                .into(),
            stopped: std::cell::Cell::new(false),
        }
    }
    fn environment(&self) -> BTreeMap<OsString, OsString> {
        let mut environment = std::env::vars_os().collect::<BTreeMap<_, _>>();
        // Cargo injects its own selected toolchain into test children. An independent
        // native construction selects the fixture/checkout pin through ordinary rustup.
        environment.remove(std::ffi::OsStr::new("RUSTUP_TOOLCHAIN"));
        environment.insert(
            "XDG_STATE_HOME".into(),
            self.state.path().as_os_str().to_owned(),
        );
        environment.insert("PSE_FORMAT".into(), "json".into());
        environment
    }
    fn command(&self, prepared: bool) -> Command {
        let mut command = Command::new(if prepared {
            &self.prepared
        } else {
            &self.client
        });
        command
            .current_dir(&self.root)
            .arg("-C")
            .arg(&self.root)
            .env_clear()
            .envs(self.environment());
        command
    }
    fn prepare(&self, target: Option<&str>) {
        self.prepare_with(target, &[])
    }
    fn prepare_with(&self, target: Option<&str>, features: &[&str]) {
        self.stopped.set(false);
        // The selection lives in the actual isolated operational namespace. Preparation
        // reuses warm compiler outputs; no descriptor is copied from another namespace.
        let _ = value(
            self.command(true)
                .args(["tools", "prepare", "native-analysis"])
                .output()
                .unwrap(),
        );
        let mut command = self.command(true);
        command
            .args([
                "code",
                "session",
                "prepare",
                "--backend",
                "native-snapshot",
                "--project",
            ])
            .arg(self.project.path())
            .arg("--no-default-features");
        if let Some(target) = target {
            command.args(["--target", target]);
        }
        if !features.is_empty() {
            command.args(["--features", &features.join(",")]);
        }
        let output = command.output().unwrap();
        if !output.status.success() {
            let original = pse_host::semantic::load(&self.root, &self.environment());
            let invocation = original
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|record| record.invocation.as_str());
            let result = invocation.map(|id| {
                self.command(false)
                    .args(["inspect", "invocation", id])
                    .output()
            });
            let stderr = invocation.map(|id| {
                self.command(false)
                    .args(["inspect", "log", id, "--stderr", "--length", "32768"])
                    .output()
            });
            panic!(
                "explicit native preparation failed: {output:?}; original semantic record: {original:?}; original result: {result:?}; original stderr: {stderr:?}"
            );
        }
        let _ = value(output);
    }
    fn prepare_lsp(&self) {
        self.stopped.set(false);
        let _ = value(
            self.command(true)
                .args([
                    "code",
                    "session",
                    "prepare",
                    "--backend",
                    "lsp",
                    "--project",
                ])
                .arg(self.project.path())
                .arg("--no-default-features")
                .output()
                .unwrap(),
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
        loop {
            let status = value(
                self.command(false)
                    .args(["code", "session", "status"])
                    .output()
                    .unwrap(),
            );
            let analyzer = &status["payload"]["analyzer"];
            assert!(analyzer["preparation_issue"].is_null(), "{status}");
            if analyzer["backend_status"]["quiescent"] == true
                && analyzer["automatic_reload_disabled"] == true
            {
                assert_eq!(analyzer["context_current"], true, "{status}");
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "explicit LSP fixture did not become ready: {status}"
            );
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
    fn stop(&self) {
        let stopped = value(
            self.command(true)
                .args(["code", "session", "stop"])
                .output()
                .unwrap(),
        );
        assert_eq!(stopped["settlement"], "complete", "{stopped}");
        let status = value(
            self.command(true)
                .args(["code", "session", "status"])
                .output()
                .unwrap(),
        );
        assert_eq!(
            status["payload"]["original_native"]["native_drained"], true,
            "{status}"
        );
        self.stopped.set(true);
    }
    fn source(&self, needle: &str) -> ResultReference {
        let path = self.project.path().join("src/lib.rs");
        let source = std::fs::read_to_string(&path).unwrap();
        let input = providers::acquire(
            &self.root,
            &pse_host::source_read::SourceRequest::Working { path: path.clone() },
        )
        .unwrap();
        let start = source.find(needle).unwrap();
        let line = source[..start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        ResultReference::new(Locator::Source {
            path: path.into(),
            source: SourceKind::Working,
            range: Some(MatchRange {
                byte_start: start,
                byte_end: start + needle.len(),
                line_start: line,
                line_end: line + needle.bytes().filter(|byte| *byte == b'\n').count(),
            }),
            guard: input.guard,
        })
    }
    fn request(&self, predicate: Predicate) -> Request {
        let mut request = Request {
            predicate,
            ..Default::default()
        };
        request
            .sources
            .paths
            .push(self.project.path().join("src/lib.rs"));
        request.answer.limit = None;
        request
    }
    fn search(&self, request: &Request) -> Value {
        value(self.search_output(request))
    }
    fn search_output(&self, request: &Request) -> Output {
        let file = self.state.path().join("selected-query.json");
        std::fs::write(&file, serde_json::to_vec(request).unwrap()).unwrap();
        self.command(false)
            .args(["search", "--request"])
            .arg(file)
            .output()
            .unwrap()
    }
    fn acquire_api(&self) -> pse_host::api::Artifact {
        self.acquire_api_for("composed-native-control")
    }
    fn acquire_api_for(&self, package: &str) -> pse_host::api::Artifact {
        self.acquire_api_at(self.project.path(), package)
    }
    fn acquire_api_at(&self, project: &std::path::Path, package: &str) -> pse_host::api::Artifact {
        let environment = self.environment();
        let snapshot = pse_host::dependency::capture(project, &environment).unwrap();
        let acquisition = pse_host::api::prepare(
            project,
            self.state.path(),
            &snapshot,
            package,
            None,
            &environment,
        )
        .unwrap();
        let output = Command::new(&acquisition.command[0])
            .args(&acquisition.command[1..])
            .current_dir(&acquisition.cwd)
            .env_clear()
            .envs(&acquisition.environment)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "original native rustdoc acquisition failed: {output:?}"
        );
        pse_host::api::finish(&acquisition, &snapshot).unwrap()
    }
}

#[test]
fn acquired_api_requires_the_original_transitive_feature_world_not_only_its_own_package() {
    let fixture = Fixture::new();
    let acquired = tempfile::tempdir().unwrap();
    let dependency = acquired.path().join("vendor/dependency");
    for path in [
        acquired.path().join("src"),
        acquired.path().join(".cargo"),
        dependency.join("src"),
    ] {
        std::fs::create_dir_all(path).unwrap();
    }
    std::fs::copy(
        fixture.root.join("rust-toolchain.toml"),
        acquired.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    std::fs::write(acquired.path().join(".cargo/config.toml"), "").unwrap();
    std::fs::write(acquired.path().join("Cargo.toml"),"[package]\nname=\"transitive-api-owner\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\nexclude=[\"vendor/dependency\"]\n[dependencies]\ntransitive-api-dependency={path=\"vendor/dependency\"}\n").unwrap();
    std::fs::write(acquired.path().join("Cargo.lock"),"version=4\n[[package]]\nname=\"transitive-api-owner\"\nversion=\"0.0.0\"\ndependencies=[\"transitive-api-dependency\"]\n[[package]]\nname=\"transitive-api-dependency\"\nversion=\"0.0.0\"\n").unwrap();
    std::fs::write(acquired.path().join("src/lib.rs"),"pub fn selected() -> transitive_api_dependency::Value { transitive_api_dependency::produce() }\n").unwrap();
    std::fs::write(dependency.join("Cargo.toml"),"[package]\nname=\"transitive-api-dependency\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[features]\nwide=[]\n").unwrap();
    std::fs::write(dependency.join("src/lib.rs"),"#[cfg(not(feature=\"wide\"))] pub type Value = u32;\n#[cfg(feature=\"wide\")] pub type Value = u64;\npub fn produce() -> Value { 1 }\n").unwrap();
    let initialized = Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(acquired.path())
        .output()
        .unwrap();
    assert!(initialized.status.success());
    let artifact = fixture.acquire_api_at(acquired.path(), "transitive-api-owner");
    let original = std::fs::read(acquired.path().join("Cargo.lock")).unwrap();
    let manifest = format!(
        "[package]\nname=\"composed-native-control\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[dependencies]\ntransitive-api-owner={{path={:?}}}\ntransitive-api-dependency={{path={:?}}}\n[features]\nwide=[\"transitive-api-dependency/wide\"]\n",
        acquired.path(),
        dependency
    );
    std::fs::write(fixture.project.path().join("Cargo.toml"), manifest).unwrap();
    std::fs::write(fixture.project.path().join("Cargo.lock"),"version=4\n[[package]]\nname=\"composed-native-control\"\nversion=\"0.0.0\"\ndependencies=[\"transitive-api-owner\",\"transitive-api-dependency\"]\n[[package]]\nname=\"transitive-api-owner\"\nversion=\"0.0.0\"\ndependencies=[\"transitive-api-dependency\"]\n[[package]]\nname=\"transitive-api-dependency\"\nversion=\"0.0.0\"\n").unwrap();
    std::fs::write(
        fixture.project.path().join("src/lib.rs"),
        "pub fn invoke() { let _ = transitive_api_owner::selected(); }\n",
    )
    .unwrap();
    let destination = fixture.source("transitive_api_owner::selected()");
    let mut request = fixture.request(Predicate::Refine {
        candidates: Box::new(Predicate::Api(ApiQuery {
            query: "selected".into(),
            artifact: Some(artifact.path.clone()),
            ..Default::default()
        })),
        filter: Refinement::UsedAt { destination },
    });
    request.sources = SourceSelector::default();
    fixture.prepare(Some(&artifact.context.target));
    let matching = fixture.search(&request);
    assert_eq!(
        matching["payload"]["summary"]["matches"], 1,
        "matching acquired/native transitive closures must prove actual use: {matching}"
    );
    fixture.stop();
    fixture.prepare_with(Some(&artifact.context.target), &["wide"]);
    let output = fixture.search_output(&request);
    assert!(
        !output.status.success(),
        "changed transitive dependency features cannot retain the original API-use proof: {output:?}"
    );
    let changed: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(changed["payload"]["summary"]["matches"], 0);
    assert_eq!(
        changed["payload"]["results"][0]["refinement"]["truth"], "unknown",
        "original structural API candidate survives unavailable enrichment: {changed}"
    );
    assert_eq!(
        std::fs::read(acquired.path().join("Cargo.lock")).unwrap(),
        original,
        "independent consumer contexts must not rewrite the acquired package lockfile"
    );
    fixture.stop();
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.stopped.get() {
            match pse_host::semantic::load(&self.root, &self.environment()) {
                Ok(None) => {}
                Ok(Some(_)) => match self
                    .command(true)
                    .args(["code", "session", "stop"])
                    .output()
                {
                    Ok(output) if output.status.success() => {}
                    other => eprintln!(
                        "original native fixture cleanup did not prove settlement: {other:?}"
                    ),
                },
                Err(error) => eprintln!(
                    "original native fixture owner could not be inspected for cleanup: {error}"
                ),
            }
        }
    }
}
fn value(output: Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}
fn method_calls() -> Predicate {
    Predicate::Syntax(SyntaxQuery {
        pattern: Some("$RECEIVER.run()".into()),
        ..Default::default()
    })
}

#[test]
fn actual_native_namesakes_refine_by_target_then_inspect_relationship_preview_and_test_selection() {
    let fixture = Fixture::new();
    fixture.prepare(None);
    let target = fixture.source("fn run(&self) -> u32 { 11 }");
    let request = fixture.request(Predicate::Refine {
        candidates: Box::new(method_calls()),
        filter: Refinement::ResolvesTo {
            target: target.clone(),
        },
    });
    let response = fixture.search(&request);
    let rows = response["payload"]["results"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{response}");
    let row = &rows[0];
    assert_eq!(row["context"], "a.run()");
    assert!(row["entity"]["session_invocation"].is_string());
    assert!(row["candidate_reference"]["locator"]["guard"]["sha256"].is_string());
    let primary = serde_json::to_string(&row["reference"]).unwrap();
    let inspected = value(
        fixture
            .command(false)
            .args(["inspect", "--result", &primary, "--view", "declaration"])
            .output()
            .unwrap(),
    );
    assert!(
        inspected["payload"]["native"]["facts"]["target"]["location"].is_object(),
        "{inspected}"
    );
    let selection = value(
        fixture
            .command(false)
            .args([
                "inspect",
                "test-selection",
                "--from-result",
                &primary,
                "--lib",
                "--no-default-features",
                "-E",
                "test(selected)",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(
        selection["payload"]["selection"]["groups"][0]["roots"][0]["package_name"],
        "composed-native-control"
    );
    assert_eq!(
        selection["payload"]["selected_tests"]["availability"],
        "unavailable"
    );
    let candidate = serde_json::to_string(&row["candidate_reference"]).unwrap();
    let preview = value(
        fixture
            .command(false)
            .args([
                "propose",
                "--result",
                &candidate,
                "--replacement",
                "a.run() + 1",
            ])
            .output()
            .unwrap(),
    );
    assert!(preview["payload"]["candidates"].is_array(), "{preview}");
    assert_eq!(
        std::fs::read_to_string(fixture.project.path().join("src/lib.rs")).unwrap(),
        SOURCE
    );
    let conjunction = fixture.search(&fixture.request(Predicate::All {
        within: MatchScope::Entity,
        clauses: vec![
            method_calls(),
            Predicate::Syntax(SyntaxQuery {
                pattern: Some("a.run()".into()),
                ..Default::default()
            }),
        ],
    }));
    assert_eq!(
        conjunction["payload"]["results"].as_array().unwrap().len(),
        1,
        "{conjunction}"
    );
    let implementations = fixture.search(&fixture.request(Predicate::Related {
        seed: fixture.source("fn run(&self) -> u32;"),
        relation: Relationship::Implementations,
        direction: Direction::Outgoing,
        depth: 1,
        max_nodes: 16,
    }));
    assert_eq!(
        implementations["payload"]["results"]
            .as_array()
            .unwrap()
            .len(),
        2,
        "the actual native trait method must expose both implementation declarations: {implementations}"
    );
    let mut related = fixture.request(Predicate::Related {
        seed: target,
        relation: Relationship::References,
        direction: Direction::Incoming,
        depth: 2,
        max_nodes: 16,
    });
    related.answer.order = ResultOrder::Stable;
    let references = fixture.search(&related);
    let records = references["payload"]["results"].as_array().unwrap();
    assert!(!records.is_empty(), "{references}");
    let mut addresses = std::collections::BTreeSet::new();
    for record in records {
        assert!(
            addresses.insert(serde_json::to_string(&record["reference"]).unwrap()),
            "relationship traversal repeated a native context/location"
        );
    }
    let mut hidden = related.clone();
    hidden.sources.exclude.push("**/lib.rs".into());
    let refused = fixture.search_output(&hidden);
    assert!(
        !refused.status.success(),
        "a relationship seed excluded from its selected source universe must refuse: {refused:?}"
    );
    let mut historical = related.clone();
    historical.sources.kind = SourceKind::Index;
    let refused = fixture.search_output(&historical);
    assert!(
        !refused.status.success(),
        "a working native seed cannot silently ignore a historical source selector: {refused:?}"
    );
    let mut bounded = related.clone();
    if let Predicate::Related { max_nodes, .. } = &mut bounded.predicate {
        *max_nodes = 1;
    }
    let response = fixture.search_output(&bounded);
    let bounded: Value = serde_json::from_slice(&response.stdout).unwrap();
    assert_eq!(
        bounded["payload"]["summary"]["matches"], 0,
        "the finite node population includes the seed: {bounded}"
    );
    assert_eq!(bounded["payload"]["summary"]["complete"], false);
    std::fs::write(
        fixture.project.path().join("src/lib.rs"),
        SOURCE.replace("a.run()", "b.run()"),
    )
    .unwrap();
    let file = fixture.state.path().join("stale.json");
    std::fs::write(&file, serde_json::to_vec(&request).unwrap()).unwrap();
    let stale = fixture
        .command(false)
        .args(["search", "--request"])
        .arg(file)
        .output()
        .unwrap();
    assert!(
        !stale.status.success(),
        "changed native/source context must not turn into a successful no-match: {stale:?}"
    );
    fixture.stop();
    std::fs::write(fixture.project.path().join("src/lib.rs"), SOURCE).unwrap();
    fixture.prepare_lsp();
    let method = serde_json::to_string(&fixture.source("fn run(&self) -> u32 { 11 }")).unwrap();
    let calls = value(
        fixture
            .command(false)
            .args(["inspect", "--result", &method, "--view", "calls"])
            .output()
            .unwrap(),
    );
    assert!(
        calls["payload"]["native"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|section| section["category"] == "incoming_calls_one_hop"
                && section["evidence"].as_array().is_some_and(|edges| edges
                    .iter()
                    .any(|edge| edge["from"]["name"] == "invoke"))),
        "only actual LSP call hierarchy proves the invoke caller; reference hits alone do not: {calls}"
    );
    fixture.stop();
}

#[test]
fn actual_acquired_api_constraints_join_native_use_and_prove_existing_destination_importability() {
    let fixture = Fixture::new();
    let artifact = fixture.acquire_api();
    fixture.prepare(Some(&artifact.context.target));
    let candidates = Predicate::Api(ApiQuery {
        query: "increment".into(),
        artifact: Some(artifact.path.clone()),
        input: Some("u32".into()),
        output: Some("u32".into()),
        ..Default::default()
    });
    for unit in [CountUnit::Files, CountUnit::Lines, CountUnit::Entities] {
        let mut request = fixture.request(Predicate::Any {
            within: MatchScope::File,
            clauses: vec![candidates.clone()],
        });
        request.answer.kind = AnswerKind::Count;
        request.answer.count_unit = unit;
        request.sources = SourceSelector::default();
        let output = fixture.search_output(&request);
        assert!(
            !output.status.success(),
            "an API artifact does not prove its source-file/line/native-entity population: {output:?}"
        );
        let answer: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(answer["payload"]["summary"]["complete"], false);
        assert!(
            answer["payload"]["summary"]["unknown"].as_u64().unwrap() > 0,
            "{answer}"
        );
        assert_ne!(answer["payload"]["summary"]["count"]["precision"], "exact");
        assert_ne!(answer["subject_state"], "no_match");
    }
    let destination = fixture.source("increment(5)");
    for filter in [
        Refinement::UsedAt {
            destination: destination.clone(),
        },
        Refinement::ImportableAt {
            destination: destination.clone(),
        },
    ] {
        let response = fixture.search(&fixture.request(Predicate::Refine {
            candidates: Box::new(candidates.clone()),
            filter,
        }));
        let rows = response["payload"]["results"].as_array().unwrap();
        assert_eq!(rows.len(), 1, "{response}");
        assert!(rows[0]["name"].as_str().unwrap().ends_with("::increment"));
        assert_eq!(
            rows[0]["candidate_reference"]["locator"]["artifact_sha256"],
            artifact.sha256
        );
        assert_eq!(
            rows[0]["candidate_reference"]["locator"]["context_key"],
            artifact.context.key().unwrap()
        );
        assert!(rows[0]["entity"]["declaration"].is_string());
        assert!(
            response["payload"]["sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source["origin"].is_null()),
            "API acquisition is not a working-source snapshot"
        );
    }
    let stale = fixture.request(Predicate::Refine {
        candidates: Box::new(candidates),
        filter: Refinement::UsedAt { destination },
    });
    std::fs::write(
        fixture.project.path().join("src/lib.rs"),
        SOURCE.replace("value + 1", "value + 2"),
    )
    .unwrap();
    let file = fixture.state.path().join("stale-api.json");
    std::fs::write(&file, serde_json::to_vec(&stale).unwrap()).unwrap();
    let output = fixture
        .command(false)
        .args(["search", "--request"])
        .arg(file)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "changed acquisition/native sources cannot preserve a stale API-use proof: {output:?}"
    );
    fixture.stop();
}

#[test]
fn acquired_dependency_aliases_unused_imports_privacy_and_two_native_contexts_preserve_authority() {
    let fixture = Fixture::new();
    let dependency = fixture.project.path().join("vendor/dependency");
    std::fs::create_dir_all(dependency.join("src")).unwrap();
    std::fs::write(
        dependency.join("Cargo.toml"),
        "[package]\nname=\"composed-api-dependency\"\nversion=\"0.0.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        dependency.join("src/lib.rs"),
        "pub fn increment(value: u32) -> u32 { value + 10 }\n",
    )
    .unwrap();
    std::fs::write(fixture.project.path().join("Cargo.toml"),"[package]\nname=\"composed-native-control\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\nexclude=[\"vendor/dependency\"]\n[dependencies]\ncomposed-api-dependency={path=\"vendor/dependency\"}\n[features]\nchanged=[]\n").unwrap();
    std::fs::write(fixture.project.path().join("Cargo.lock"),"version=4\n[[package]]\nname=\"composed-native-control\"\nversion=\"0.0.0\"\ndependencies=[\"composed-api-dependency\"]\n[[package]]\nname=\"composed-api-dependency\"\nversion=\"0.0.0\"\n").unwrap();
    let source = "#[doc(inline)] pub use composed_api_dependency::increment as renamed;\n#[doc(inline)] pub use composed_api_dependency::increment as secondary;\npub fn use_alias() -> u32 { renamed(5) }\nmod inner { pub fn public_unused() -> u32 { 3 } fn private_unused() -> u32 { 4 } fn increment(value: u32) -> u32 { value + 99 } }\npub mod destination { pub fn anchor() -> u32 { 0 } }\n";
    std::fs::write(fixture.project.path().join("src/lib.rs"), source).unwrap();
    let facade = fixture.acquire_api();
    fixture.prepare(Some(&facade.context.target));
    let use_site = fixture.source("renamed(5)");
    let alias_candidate = Predicate::Api(ApiQuery {
        query: "renamed".into(),
        artifact: Some(facade.path.clone()),
        input: Some("u32".into()),
        output: Some("u32".into()),
        ..Default::default()
    });
    let unknown = fixture.search_output(&fixture.request(Predicate::Refine {
        candidates: Box::new(alias_candidate.clone()),
        filter: Refinement::UsedAt {
            destination: use_site.clone(),
        },
    }));
    assert!(
        !unknown.status.success(),
        "native identity does not prove an unacquired API signature: {unknown:?}"
    );
    let unknown: Value = serde_json::from_slice(&unknown.stdout).unwrap();
    assert_eq!(
        unknown["payload"]["results"][0]["refinement"]["truth"], "unknown",
        "{unknown}"
    );
    assert_ne!(unknown["subject_state"], "no_match", "{unknown}");
    assert_ne!(
        unknown["payload"]["summary"]["count"]["precision"], "exact",
        "{unknown}"
    );
    let dependency_artifact = fixture.acquire_api_for("composed-api-dependency");
    let mut structural_request = fixture.request(alias_candidate.clone());
    structural_request.sources = SourceSelector::default();
    let structural = fixture.search(&structural_request);
    assert_eq!(
        structural["payload"]["results"].as_array().map(Vec::len),
        Some(1),
        "actual acquired public alias constraints must independently discover the structural candidate: {structural}"
    );
    for (name, matching) in [("renamed", NameMatch::Exact), ("rena", NameMatch::Prefix)] {
        let mut request = fixture.request(Predicate::Symbol(SymbolQuery {
            name: name.into(),
            matching,
            provider: DiscoveryProvider::Api,
            artifact: Some(facade.path.clone()),
            kinds: vec!["function".into()],
            ..Default::default()
        }));
        request.sources = SourceSelector::default();
        let discovered = fixture.search(&request);
        assert_eq!(
            discovered["payload"]["results"].as_array().map(Vec::len),
            Some(1),
            "public alias spelling must survive exact/prefix discovery: {discovered}"
        );
        assert_eq!(
            discovered["payload"]["results"][0]["reference"]["locator"]["definition"]["artifact_sha256"],
            dependency_artifact.sha256,
            "{discovered}"
        );
    }
    let joined = fixture.search(&fixture.request(Predicate::Refine {
        candidates: Box::new(alias_candidate.clone()),
        filter: Refinement::UsedAt {
            destination: use_site.clone(),
        },
    }));
    let alias = &joined["payload"]["results"][0];
    assert!(
        alias["name"]
            .as_str()
            .is_some_and(|name| name.ends_with("::renamed")),
        "{joined}"
    );
    assert!(
        alias["candidate_reference"]["locator"]["public_path"]
            .as_str()
            .unwrap()
            .ends_with("::renamed")
    );
    assert!(
        alias["entity"]["declaration"]
            .as_str()
            .unwrap()
            .ends_with("vendor/dependency/src/lib.rs"),
        "alias must resolve through actual native identity to the dependency, not the private same-spelled shadow: {joined}"
    );
    let old_native = serde_json::to_string(&alias["reference"]).unwrap();
    let dependency_candidate = Predicate::Api(ApiQuery {
        query: "increment".into(),
        artifact: Some(dependency_artifact.path.clone()),
        input: Some("u32".into()),
        output: Some("u32".into()),
        ..Default::default()
    });
    let mut dependency_request = fixture.request(Predicate::Refine {
        candidates: Box::new(dependency_candidate),
        filter: Refinement::UsedAt {
            destination: use_site,
        },
    });
    dependency_request.sources = SourceSelector::default();
    let dependency_join = fixture.search(&dependency_request);
    assert_eq!(
        dependency_join["payload"]["results"]
            .as_array()
            .unwrap()
            .len(),
        1,
        "{dependency_join}"
    );
    dependency_request
        .sources
        .paths
        .push(fixture.project.path().join("src/lib.rs"));
    let excluded = fixture.search(&dependency_request);
    assert_eq!(
        excluded["payload"]["summary"]["matches"], 0,
        "scope applies to the original API source nomination, not its use site: {excluded}"
    );
    dependency_request.sources.paths = vec![dependency.join("src/lib.rs")];
    let included = fixture.search(&dependency_request);
    assert_eq!(included["payload"]["summary"]["matches"], 1, "{included}");
    let destination = fixture.source("fn anchor() -> u32 { 0 }");
    for (pattern, expected) in [
        ("pub fn public_unused() -> u32 { 3 }", 1),
        ("fn private_unused() -> u32 { 4 }", 0),
    ] {
        let request = fixture.request(Predicate::Refine {
            candidates: Box::new(Predicate::Syntax(SyntaxQuery {
                pattern: Some(pattern.into()),
                ..Default::default()
            })),
            filter: Refinement::ImportableAt {
                destination: destination.clone(),
            },
        });
        let result = fixture.search(&request);
        assert_eq!(
            result["payload"]["summary"]["matches"], expected,
            "{result}"
        );
        assert_eq!(result["payload"]["summary"]["complete"], true, "{result}");
        assert!(
            result["payload"]["sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |source| source["native"]["analysis"]["configuration"]["target"]
                        == facade.context.target
                        && source["native"]["session_invocation"].is_string()
                ),
            "fully filtered answers still retain the actual native world: {result}"
        );
        if expected == 1 {
            assert!(
                result["payload"]["results"][0]["refinement"]["evidence"]["importable"] == true,
                "unused public importability must come from actual HIR facts: {result}"
            );
        }
    }
    fixture.stop();
    fixture.prepare_with(Some(&facade.context.target), &["changed"]);
    let refused = fixture
        .command(false)
        .args(["inspect", "--result", &old_native, "--view", "declaration"])
        .output()
        .unwrap();
    assert!(
        !refused.status.success(),
        "a replaced native context cannot reinterpret an old resolved identity: {refused:?}"
    );
    let changed = fixture.request(Predicate::Refine {
        candidates: Box::new(alias_candidate),
        filter: Refinement::ImportableAt { destination },
    });
    let output = fixture.search_output(&changed);
    assert!(!output.status.success());
    let unknown: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = unknown["payload"]["results"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        1,
        "structural alias discovery must survive unavailable cross-context enrichment: {unknown}"
    );
    assert_eq!(rows[0]["refinement"]["truth"], "unknown");
    assert_eq!(unknown["payload"]["summary"]["matches"], 0);
    assert!(rows[0]["reference"]["locator"]["artifact_sha256"].is_string());
    fixture.stop();
}
