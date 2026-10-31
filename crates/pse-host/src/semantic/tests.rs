// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[test]
fn compact_native_sections_and_workspace_answers_round_trip_without_invented_facts() {
    for handles in [
        Vec::new(),
        vec![CandidateHandle {
            index: 3,
            fingerprint: "original-candidate".into(),
        }],
    ] {
        let section = Section {
            category: "workspace_symbols".into(),
            evidence: json!([{ "name":"run", "kind":"function" }]),
            omitted: 2,
            handles,
        };
        let emitted = serde_json::to_value(&section).unwrap();
        assert_eq!(emitted.get("handles").is_none(), section.handles.is_empty());
        let received: Section = serde_json::from_value(emitted.clone()).unwrap();
        assert_eq!(serde_json::to_value(&received).unwrap(), emitted);
        let answer = WorkspaceAnswer {
            session: AnalysisContext {
                backend: "original-native".into(),
                configuration: Configuration {
                    backend: Backend::NativeSnapshot,
                    project: "/selected/project".into(),
                    features: vec!["actual".into()],
                    no_default_features: false,
                    target: None,
                    build_macros: false,
                },
                position_encoding: "utf-8".into(),
                backend_status: Some(json!({"actual_target":"selected-target"})),
                preparation: "explicit-original-context".into(),
            },
            generation: 7,
            symbols: section,
            limitations: vec!["provider-ranked; not an absence proof".into()],
        };
        let emitted = serde_json::to_value(&answer).unwrap();
        let received: WorkspaceAnswer = serde_json::from_value(emitted.clone()).unwrap();
        assert_eq!(serde_json::to_value(received).unwrap(), emitted);
    }
    let malformed = json!({"category":"workspace_symbols","evidence":[]});
    assert!(
        serde_json::from_value::<Section>(malformed).is_err(),
        "known omission counts remain required; compact empty handles alone have a defined default"
    );
}

#[cfg(feature = "native-hir")]
#[tokio::test]
async fn native_snapshot_authenticated_cancel_settles_one_request_and_preserves_the_session() {
    use tokio_util::sync::CancellationToken;
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("Cargo.toml"),"[package]\nname='native-cancellation-control'\nversion='0.0.0'\nedition='2024'\n[workspace]\n").unwrap();
    std::fs::write(
        project.path().join("Cargo.lock"),
        "version=4\n[[package]]\nname='native-cancellation-control'\nversion='0.0.0'\n",
    )
    .unwrap();
    std::fs::copy(
        checkout().join("rust-toolchain.toml"),
        project.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let text = format!(
        "pub fn target()->u32{{7}}\npub fn many(){{let mut n=0u32;{}std::hint::black_box(n);}}\n",
        "n+=1;".repeat(30_000)
    );
    let file = project.path().join("src/lib.rs");
    std::fs::write(&file, &text).unwrap();
    let session = Session::new();
    session.run(&["tools", "prepare", "native-analysis"], 0);
    session.run(
        &[
            "code",
            "session",
            "prepare",
            "--backend",
            "native-snapshot",
            "--project",
            project.path().to_str().unwrap(),
        ],
        0,
    );
    session.ready();
    let mut environment: crate::environment::Environment = std::env::vars_os().collect();
    // Cargo supplies its selected toolchain to test processes. Explicit nested
    // preparation selects the checked-in toolchain afresh, retaining every
    // workload/placement owner and the production override refusal.
    environment.remove(std::ffi::OsStr::new("RUSTUP_TOOLCHAIN"));
    environment.insert(
        "XDG_STATE_HOME".into(),
        session.state.path().as_os_str().to_owned(),
    );
    let record = load(&session.root, &environment).unwrap().unwrap();
    let offset = text.find("n=0u32").unwrap();
    let prefix = &text[..offset];
    let query = Query {
        kind: Kind::Refs,
        file: file.clone(),
        line: prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
        column: prefix.rsplit('\n').next().unwrap().chars().count() + 1,
        if_source: Some(crate::source_read::source_digest(&text)),
        if_generation: Some(1),
        if_candidate: None,
        limit: 100_000,
        end_line: None,
    };
    let cancellation = CancellationToken::new();
    let request = {
        let root = session.root.clone();
        let environment = environment.clone();
        let record = record.clone();
        let token = cancellation.clone();
        tokio::spawn(async move {
            client_cancellable(&root, &environment, &record, &Request::Query(query), &token).await
        })
    };
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let status = client(&session.root, &environment, &record, &Request::Status {})
            .await
            .unwrap();
        if status["active_requests"].as_u64().unwrap() > 0 {
            break;
        }
        assert!(
            !request.is_finished(),
            "long native query finished before actual cancellation could be exercised"
        );
        assert!(
            tokio::time::Instant::now() < deadline,
            "native request never became active"
        );
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    let sibling = {
        let root = session.root.clone();
        let environment = environment.clone();
        let record = record.clone();
        let file = file.clone();
        tokio::spawn(async move {
            client(
                &root,
                &environment,
                &record,
                &Request::Query(Query {
                    kind: Kind::Explain,
                    file,
                    line: 1,
                    column: 8,
                    if_source: None,
                    if_generation: Some(1),
                    if_candidate: None,
                    limit: 20,
                    end_line: None,
                }),
            )
            .await
        })
    };
    cancellation.cancel();
    let cancelled = tokio::time::timeout(Duration::from_secs(60), request)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert!(
        matches!(cancelled,Error::Refusal(ref failure) if matches!(failure.state,FailureState::Cancelled)&&failure.backend_data.as_ref().is_some_and(|data|data["cpu_work_completed"]==true&&data["session_stopped"]==false)),
        "{cancelled:?}"
    );
    assert!(
        sibling.await.unwrap().is_ok(),
        "another native consumer must remain usable"
    );
    let status = client(&session.root, &environment, &record, &Request::Status {})
        .await
        .unwrap();
    assert_eq!(status["context_current"], true);
    assert_eq!(status["active_requests"], 0);
    session.stop();
}

fn checkout() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}
fn fixture(root: &Path) {
    std::fs::create_dir_all(root.join("member/src")).unwrap();
    std::fs::create_dir_all(root.join("macros/src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers=['member','macros']\nresolver='3'\n",
    )
    .unwrap();
    std::fs::write(root.join("member/Cargo.toml"),"[package]\nname='semantic-cli-control'\nversion='0.0.0'\nedition='2024'\n[dependencies]\nsemantic-macro-control={path='../macros'}\n[features]\nextra=[]\n").unwrap();
    std::fs::write(root.join("macros/Cargo.toml"), "[package]\nname='semantic-macro-control'\nversion='0.0.0'\nedition='2024'\n[lib]\nproc-macro=true\n").unwrap();
    std::fs::write(root.join("macros/src/lib.rs"), "extern crate proc_macro;\n#[proc_macro] pub fn answer(_:proc_macro::TokenStream)->proc_macro::TokenStream { \"pub const GENERATED: u32 = 11;\".parse().unwrap() }\n").unwrap();
    std::fs::write(root.join("shared-mode.txt"), "shared-one").unwrap();
    std::fs::write(
        root.join("Cargo.lock"),
        "version=4\n[[package]]\nname='semantic-cli-control'\nversion='0.0.0'\ndependencies=['semantic-macro-control']\n[[package]]\nname='semantic-macro-control'\nversion='0.0.0'\n",
    )
    .unwrap();
    std::fs::write(
        root.join("member/src/lib.rs"),
        "pub fn value()->u32 { 41 }\n#[test] fn control(){assert_eq!(value(),41);}\nsemantic_macro_control::answer!();\n#[cfg(build_one)] macro_rules! selected_cfg { () => { 11 }; }\n#[cfg(build_two)] macro_rules! selected_cfg { () => { 22 }; }\npub const SELECTED: u32 = selected_cfg!();\npub const SHARED: &str = env!(\"SEMANTIC_SHARED\");\n",
    )
    .unwrap();
    std::fs::write(root.join("member/build.rs"), r#"use std::io::Write;
fn main() {
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace = std::path::Path::new(&root).parent().unwrap();
    let mut witness = std::fs::OpenOptions::new().create(true).append(true).open(workspace.join("executions.txt")).unwrap();
    writeln!(witness, "executed").unwrap();
    println!("cargo::rustc-cfg=build_one");
    println!("cargo::rustc-check-cfg=cfg(build_one)");
    println!("cargo::rustc-check-cfg=cfg(build_two)");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../shared-mode.txt");
    println!("cargo::rustc-env=SEMANTIC_SHARED={}", std::fs::read_to_string(workspace.join("shared-mode.txt")).unwrap());
}
"#).unwrap();
}
struct Session {
    root: PathBuf,
    executable: PathBuf,
    state: tempfile::TempDir,
}
impl Session {
    fn new() -> Self {
        let root = checkout();
        let control = crate::generation::prepared(&root)
            .unwrap()
            .expect("prepare the native host explicitly before native lifecycle tests");
        Self {
            executable: control
                .executable(&root, crate::generation::HostTool::Pse)
                .unwrap(),
            root,
            state: tempfile::tempdir().unwrap(),
        }
    }
    fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&self.root)
            .env_remove("RUSTUP_TOOLCHAIN")
            .arg("-C")
            .arg(&self.root)
            .env("XDG_STATE_HOME", self.state.path());
        command
    }
    fn run(&self, arguments: &[&str], exit: i32) -> Value {
        let output = self.command().args(arguments).output().unwrap();
        let original = if output.status.code() != Some(exit) {
            self.original_failure()
        } else {
            String::new()
        };
        assert_eq!(
            output.status.code(),
            Some(exit),
            "argv={arguments:?}; stdout={}; stderr={}; original={original}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    fn original_failure(&self) -> String {
        use std::io::Read as _;
        let mut environment: crate::environment::Environment = std::env::vars_os().collect();
        environment.remove(std::ffi::OsStr::new("RUSTUP_TOOLCHAIN"));
        environment.insert(
            "XDG_STATE_HOME".into(),
            self.state.path().as_os_str().to_owned(),
        );
        let record = match load(&self.root, &environment) {
            Ok(Some(record)) => record,
            other => return format!("session_record={other:?}"),
        };
        let state = match crate::paths::CheckoutState::resolve(&self.root, &environment) {
            Ok(state) => state,
            Err(error) => return format!("invocation={}; state={error}", record.invocation),
        };
        let store = crate::invocation::InvocationStore::new(state);
        let stderr = store
            .directory(&record.invocation)
            .and_then(|directory| directory.stream(crate::extraction::OutputStream::Stderr))
            .and_then(|file| {
                let mut bytes = Vec::new();
                file.take(64 * 1024).read_to_end(&mut bytes)?;
                Ok(String::from_utf8_lossy(&bytes).into_owned())
            });
        let terminal = store.result::<Configuration, Value>(&record.invocation);
        format!(
            "invocation={}; stderr_first_64k={stderr:?}; original_terminal={terminal:?}",
            record.invocation
        )
    }
    fn ready(&self) {
        let deadline = Instant::now() + Duration::from_secs(45);
        loop {
            let status = self.run(&["code", "session", "status"], 0);
            let analyzer = &status["payload"]["analyzer"];
            assert!(analyzer["preparation_issue"].is_null(), "{status}");
            if analyzer["backend_status"]["quiescent"] == true
                && analyzer["automatic_reload_disabled"] == true
            {
                assert_eq!(analyzer["context_current"], true, "{status}");
                return;
            }
            assert!(
                Instant::now() < deadline,
                "native analyzer never became ready: {status}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    fn prepare(&self, root: &str) {
        self.run(
            &[
                "code",
                "session",
                "prepare",
                "--project",
                root,
                "--build-macros",
            ],
            0,
        );
        self.ready();
    }
    fn stop(&self) {
        assert_eq!(
            self.run(&["code", "session", "stop"], 0)["settlement"],
            "complete"
        );
        assert_eq!(
            self.run(&["code", "session", "status"], 0)["payload"]["original_native"]["native_drained"],
            true
        );
    }
    fn stale(&self, root: &str, file: &str) {
        let status = self.run(&["code", "session", "status"], 0);
        assert_eq!(
            status["payload"]["analyzer"]["context_current"], false,
            "{status}"
        );
        let refused = self
            .command()
            .args([
                "code",
                "session",
                "prepare",
                "--project",
                root,
                "--build-macros",
            ])
            .output()
            .unwrap();
        assert!(
            !refused.status.success(),
            "stale preparation must not be reused"
        );
        let answer = self.run(
            &["inspect", "entity", file, "--line", "1", "--column", "8"],
            1,
        );
        assert_eq!(answer["payload"]["state"], "stale", "{answer}");
    }
    fn expansion(&self, file: &str, line: &str, column: &str) -> String {
        let answer = self.run(
            &[
                "inspect", "entity", "--view", "macro", file, "--line", line, "--column", column,
            ],
            0,
        );
        answer["payload"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .find(|section| section["category"] == "macro_expansion")
            .unwrap()["evidence"]["expansion"]
            .as_str()
            .unwrap_or_else(|| panic!("no expansion: {answer}"))
            .to_owned()
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.command().args(["code", "session", "stop"]).output();
    }
}

fn native_journey() {
    let project = tempfile::tempdir().unwrap();
    fixture(project.path());
    let session = Session::new();
    let member = project.path().join("member");
    let root = member.to_str().unwrap();
    let file = member.join("src/lib.rs");
    let file = file.to_str().unwrap();
    assert_eq!(
        session.run(&["code", "session", "status"], 0)["subject_state"],
        "unprepared"
    );
    session.run(
        &[
            "code",
            "session",
            "prepare",
            "--project",
            root,
            "--build-macros",
        ],
        0,
    );
    let first = session.run(&["code", "session", "status"], 0);
    let original = first["payload"]["record"]["invocation"].clone();
    session.run(
        &[
            "code",
            "session",
            "prepare",
            "--project",
            root,
            "--build-macros",
        ],
        0,
    );
    assert_eq!(
        session.run(&["code", "session", "status"], 0)["payload"]["record"]["invocation"],
        original
    );
    let refused = session
        .command()
        .args([
            "code",
            "session",
            "prepare",
            "--project",
            root,
            "--features",
            "extra",
            "--build-macros",
        ])
        .output()
        .unwrap();
    assert!(!refused.status.success());
    session.ready();
    let text = std::fs::read_to_string(file).unwrap();
    let line = text.lines().nth(1).unwrap();
    let column = (line[..line.find("value").unwrap()].chars().count() + 1).to_string();
    let answer = session.run(
        &[
            "inspect", "entity", file, "--line", "2", "--column", &column,
        ],
        0,
    );
    assert!(
        answer["payload"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|section| section["category"] == "definition"
                && !section["evidence"].as_array().unwrap().is_empty()),
        "{answer}"
    );
    let preview = session.run(
        &[
            "propose", "rename", file, "--line", "1", "--column", "8", "--name", "renamed",
        ],
        0,
    );
    assert!(!preview["payload"]["proposal"].is_null());
    assert!(
        std::fs::read_to_string(file)
            .unwrap()
            .starts_with("pub fn value")
    );
    let stale = session.run(
        &[
            "inspect",
            "entity",
            file,
            "--line",
            "1",
            "--column",
            "8",
            "--if-source",
            "old-digest",
        ],
        1,
    );
    assert_eq!(stale["subject_state"], "stale");
    assert_eq!(stale["payload"]["state"], "stale");
    let witness = project.path().join("executions.txt");
    let executions = std::fs::read_to_string(&witness).unwrap();
    assert!(!executions.is_empty());
    assert!(session.expansion(file, "3", "25").contains("11"));
    assert!(session.expansion(file, "6", "29").contains("11"));
    // Request buffers are closed by now. Compiled preparation must remain fenced on disk.
    let macro_source = project.path().join("macros/src/lib.rs");
    let outputs = compiler_outputs(&project.path().join("target"));
    let text = std::fs::read_to_string(&macro_source)
        .unwrap()
        .replace("= 11", "= 22");
    std::fs::write(macro_source, text).unwrap();
    session.stale(root, file);
    assert_eq!(
        compiler_outputs(&project.path().join("target")),
        outputs,
        "stale macro queries must not compile"
    );
    assert_eq!(std::fs::read_to_string(&witness).unwrap(), executions);
    session.stop();
    session.prepare(root);
    assert!(session.expansion(file, "3", "25").contains("22"));

    let build_script = member.join("build.rs");
    let outputs = compiler_outputs(&project.path().join("target"));
    let text = std::fs::read_to_string(&build_script)
        .unwrap()
        .replace("rustc-cfg=build_one", "rustc-cfg=build_two");
    let executions = std::fs::read_to_string(&witness).unwrap();
    std::fs::write(build_script, text).unwrap();
    session.stale(root, file);
    assert_eq!(
        compiler_outputs(&project.path().join("target")),
        outputs,
        "stale cfg queries must not compile"
    );
    assert_eq!(std::fs::read_to_string(&witness).unwrap(), executions);
    session.stop();
    session.prepare(root);
    assert!(session.expansion(file, "6", "29").contains("22"));
    assert!(std::fs::read_to_string(&witness).unwrap().len() > executions.len());

    assert!(session.expansion(file, "7", "26").contains("shared-one"));
    let executions = std::fs::read_to_string(&witness).unwrap();
    std::fs::write(project.path().join("shared-mode.txt"), "shared-two").unwrap();
    session.stale(root, file);
    assert_eq!(std::fs::read_to_string(&witness).unwrap(), executions);
    session.stop();
    session.prepare(root);
    assert!(session.expansion(file, "7", "26").contains("shared-two"));

    let lock = project.path().join("Cargo.lock");
    let mut text = std::fs::read_to_string(&lock).unwrap();
    let original_lock = text.clone();
    text.push_str("# changed virtual workspace lock\n");
    let executions = std::fs::read_to_string(&witness).unwrap();
    std::fs::write(&lock, text).unwrap();
    std::fs::write(lock, original_lock).unwrap();
    session.stale(root, file);
    assert_eq!(std::fs::read_to_string(&witness).unwrap(), executions);
    session.stop();
    session.prepare(root);
    let manifest = project.path().join("Cargo.toml");
    let mut text = std::fs::read_to_string(&manifest).unwrap();
    text.push_str("# changed virtual workspace context\n");
    std::fs::write(manifest, text).unwrap();
    session.stale(root, file);
    assert_eq!(std::fs::read_to_string(&witness).unwrap(), executions);
    session.stop();
    session.prepare(root);
    // Retain the original native manifest-effect control as well as the effective-root
    // controls above: the backend itself observes this member manifest's blocked reload.
    let executions = std::fs::read_to_string(&witness).unwrap();
    let manifest = member.join("Cargo.toml");
    let mut text = std::fs::read_to_string(&manifest).unwrap();
    text.push_str("default=['extra']\n");
    std::fs::write(manifest, text).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let state = session.run(&["code", "session", "status"], 0);
        let analyzer = &state["payload"]["analyzer"];
        if analyzer["backend_status"]["quiescent"] == true
            && analyzer["backend_status"]["health"] == "warning"
        {
            assert_eq!(analyzer["automatic_reload_disabled"], true);
            assert_eq!(analyzer["context_current"], false);
            break;
        }
        assert!(
            Instant::now() < deadline,
            "native manifest change was not observed as a blocked reload: {state}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    let stale = session.run(
        &["inspect", "entity", file, "--line", "1", "--column", "8"],
        1,
    );
    assert_eq!(stale["payload"]["state"], "stale");
    assert_eq!(
        std::fs::read_to_string(witness).unwrap(),
        executions,
        "manifest watching must not rerun a compiler/build script after explicit initial preparation"
    );
    session.stop();
}

fn compiler_outputs(
    root: &Path,
) -> std::collections::BTreeMap<PathBuf, (u64, std::time::SystemTime)> {
    fn collect(
        root: &Path,
        files: &mut std::collections::BTreeMap<PathBuf, (u64, std::time::SystemTime)>,
    ) {
        for entry in std::fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            let metadata = std::fs::metadata(&path).unwrap();
            if metadata.is_dir() {
                collect(&path, files);
            } else {
                files.insert(path, (metadata.len(), metadata.modified().unwrap()));
            }
        }
    }
    let mut files = std::collections::BTreeMap::new();
    collect(root, &mut files);
    files
}

#[test]
fn native_cli_entry() {
    let Some(gate) = std::env::var_os("PSE_SEMANTIC_CLI_GATE").map(PathBuf::from) else {
        return;
    };
    let deadline = Instant::now() + Duration::from_secs(15);
    while !gate.exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    native_journey();
}

#[tokio::test]
async fn native_cli_reuses_original_session_fences_sources_and_proves_stop() {
    // Like B03's detached-front-end fixture, first place this independent CLI caller
    // in its own ordinary application scope. Product nested-detachment refusal stays intact.
    let scratch = tempfile::tempdir().unwrap();
    let gate = scratch.path().join("go");
    let manager = crate::HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let (slice, _, _) = manager.foreground_placement(&current).await.unwrap();
    let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "semantic::tests::native_cli_entry",
            "--nocapture",
        ])
        .env("PSE_SEMANTIC_CLI_GATE", &gate)
        .kill_on_drop(true);
    for key in [
        "PSE_HOST_ALLOCATION",
        "PSE_RESOURCE_CLASS",
        "PSE_MEMORY_MAX",
        "PSE_SLICE",
    ] {
        command.env_remove(key);
    }
    let mut child = command.spawn().unwrap();
    let pid = rustix::process::Pid::from_raw(child.id().unwrap().try_into().unwrap()).unwrap();
    let pidfd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).unwrap();
    let intent = crate::LaunchIntent::fresh_scope().unwrap();
    let mut unit = manager
        .start_scope(
            &intent,
            pidfd,
            &slice,
            &slice,
            Instant::now() + Duration::from_secs(10),
            Duration::from_secs(5),
        )
        .await
        .unwrap();
    std::fs::write(gate, b"original application scope established").unwrap();
    let status = child.wait().await.unwrap();
    unit.stop_and_drain(Instant::now() + Duration::from_secs(15))
        .await
        .unwrap();
    assert!(
        status.success(),
        "independent original CLI journey failed: {status}"
    );
}

#[tokio::test]
async fn socket_peer_cannot_supply_its_own_pid_or_receive_input_without_original_custody() {
    use std::io::Read as _;
    assert!(
        serde_json::from_value::<Request>(
            json!({"operation":"status","peer_pid":std::process::id()})
        )
        .is_err()
    );
    let root = checkout();
    let scratch = tempfile::tempdir().unwrap();
    let mut environment: crate::environment::Environment = std::env::vars_os().collect();
    environment.insert(
        "XDG_STATE_HOME".into(),
        scratch.path().join("state").into_os_string(),
    );
    let invocation = crate::envelope::CliInvocationId::new();
    let socket = socket_path(&root, &environment, &invocation, true).unwrap();
    let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    let peer = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut byte = [0];
        stream.read(&mut byte).unwrap()
    });
    let record = SessionRecord {
        version: 1,
        invocation,
        configuration: Configuration {
            backend: Backend::Lsp,
            project: root.clone(),
            features: vec![],
            no_default_features: false,
            target: None,
            build_macros: false,
        },
    };
    let refused = client(&root, &environment, &record, &Request::Status {}).await;
    assert!(refused.is_err());
    assert_eq!(
        peer.join().unwrap(),
        0,
        "no request bytes before original-custody authentication"
    );
    std::fs::remove_file(socket).unwrap();
}
