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
                    test_context: false,
                },
                position_encoding: "utf-8".into(),
                backend_status: Some(json!({"actual_target":"selected-target"})),
                preparation: "explicit-original-context".into(),
            },
            generation: 7,
            symbols: section,
            observed: Vec::new(),
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
    let connected = Client::connect(&session.root, &environment, &record)
        .await
        .unwrap();
    assert!(
        connected
            .request(&Request::Status {}, None)
            .await
            .unwrap()
            .into_status()
            .unwrap()
            .context_current
    );
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
            .unwrap()
            .into_status()
            .unwrap();
        if status.active_requests > 0 {
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
        matches!(
            cancelled,
            Error::CancelledObserved {
                completion: Settlement::CpuSettled,
                ..
            }
        ),
        "{cancelled:?}"
    );
    assert!(
        sibling.await.unwrap().is_ok(),
        "another native consumer must remain usable"
    );
    let status = client(&session.root, &environment, &record, &Request::Status {})
        .await
        .unwrap()
        .into_status()
        .unwrap();
    assert!(status.context_current);
    assert_eq!(status.active_requests, 0);
    session.stop();
    // Reuse the connection that authenticated the real managed helper. Once that
    // original helper stops, a same-UID replacement socket cannot inherit custody.
    let socket = socket_path(&session.root, &environment, &record.invocation, false).unwrap();
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    let peer = tokio::spawn(async move {
        use tokio::io::AsyncReadExt as _;
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut byte = [0];
        tokio::time::timeout(Duration::from_secs(5), stream.read(&mut byte))
            .await
            .unwrap()
            .unwrap()
    });
    tokio::time::timeout(Duration::from_secs(10), async {
        while !peer.is_finished() {
            assert!(connected.request(&Request::Status {}, None).await.is_err());
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        peer.await.unwrap(),
        0,
        "a reconnect must authenticate before HTTP/2 bytes"
    );
    std::fs::remove_file(socket).unwrap();
}

fn checkout() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

#[cfg(feature = "native-hir")]
async fn extended_native_managed_macro_journey() {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    std::fs::create_dir_all(project.path().join("macros/src")).unwrap();
    std::fs::write(project.path().join("Cargo.toml"),"[package]\nname='managed-native-macro'\nversion='0.0.0'\nedition='2024'\n[workspace]\n[dependencies]\nmanaged-macro={path='macros'}\n").unwrap();
    std::fs::write(project.path().join("Cargo.lock"),"version=4\n[[package]]\nname='managed-native-macro'\nversion='0.0.0'\ndependencies=['managed-macro']\n[[package]]\nname='managed-macro'\nversion='0.0.0'\n").unwrap();
    std::fs::write(project.path().join("macros/Cargo.toml"),"[package]\nname='managed-macro'\nversion='0.0.0'\nedition='2024'\n[lib]\nproc-macro=true\n").unwrap();
    std::fs::copy(
        checkout().join("rust-toolchain.toml"),
        project.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let flag = project.path().join("target/slow-expansion");
    let entered = project.path().join("target/expansion-entered");
    let finished = project.path().join("target/expansion-finished");
    let macro_source = r#"extern crate proc_macro; #[proc_macro_derive(Slow)] pub fn slow(_:proc_macro::TokenStream)->proc_macro::TokenStream { if std::path::Path::new(__FLAG__).exists() { std::fs::write(__ENTERED__,"entered").unwrap(); std::thread::sleep(std::time::Duration::from_secs(60)); std::fs::write(__FINISHED__,"finished").unwrap(); } "impl Selected {pub fn expanded(){}}".parse().unwrap() }"#
        .replace("__FLAG__",&format!("{flag:?}"))
        .replace("__ENTERED__",&format!("{entered:?}"))
        .replace("__FINISHED__",&format!("{finished:?}"));
    std::fs::write(project.path().join("macros/src/lib.rs"), macro_source).unwrap();
    let file = project.path().join("src/lib.rs");
    let source = "#[derive(managed_macro::Slow)] pub struct Selected; pub fn selected(){Selected::expanded();}";
    std::fs::write(&file, source).unwrap();
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
            "--build-macros",
        ],
        0,
    );
    session.ready();
    let other_project = tempfile::tempdir().unwrap();
    std::fs::create_dir(other_project.path().join("src")).unwrap();
    std::fs::write(other_project.path().join("Cargo.toml"),"[package]\nname='independent-native-session'\nversion='0.0.0'\nedition='2024'\n[workspace]\n").unwrap();
    std::fs::write(
        other_project.path().join("Cargo.lock"),
        "version=4\n[[package]]\nname='independent-native-session'\nversion='0.0.0'\n",
    )
    .unwrap();
    std::fs::copy(
        checkout().join("rust-toolchain.toml"),
        other_project.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    std::fs::write(
        other_project.path().join("src/lib.rs"),
        "pub fn untouched()->u32 {7}",
    )
    .unwrap();
    let other = Session::new();
    other.run(&["tools", "prepare", "native-analysis"], 0);
    other.run(
        &[
            "code",
            "session",
            "prepare",
            "--backend",
            "native-snapshot",
            "--project",
            other_project.path().to_str().unwrap(),
        ],
        0,
    );
    other.ready();
    let mut environment: crate::environment::Environment = std::env::vars_os().collect();
    environment.remove(std::ffi::OsStr::new("RUSTUP_TOOLCHAIN"));
    environment.insert(
        "XDG_STATE_HOME".into(),
        session.state.path().as_os_str().to_owned(),
    );
    let record = load(&session.root, &environment).unwrap().unwrap();
    let connected = Client::connect(&session.root, &environment, &record)
        .await
        .unwrap();
    std::fs::write(flag, "expand slowly").unwrap();
    let offset = source.find("expanded();").unwrap();
    let query = Request::Query(Query {
        kind: Kind::Explain,
        file,
        line: 1,
        column: offset + 1,
        if_source: Some(crate::source_read::source_digest(source)),
        if_generation: Some(1),
        if_candidate: None,
        limit: 20,
        end_line: None,
    });
    let request = tokio::spawn(async move { connected.request(&query, None).await });
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    while !entered.exists() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "actual managed macro never entered; request_finished={}",
            request.is_finished()
        );
        assert!(
            !request.is_finished(),
            "macro request returned before actual expansion entry"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    // The original managed owner escalates and proves drain. Query transport
    // completion or a stop acknowledgement alone cannot establish this outcome.
    let session = tokio::task::spawn_blocking(move || {
        session.stop();
        session
    })
    .await
    .unwrap();
    assert!(
        !finished.exists(),
        "slow macro completed naturally; stop escalation was not exercised"
    );
    assert!(
        tokio::time::timeout(Duration::from_secs(30), request)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    assert_eq!(
        other.run(&["code", "session", "status"], 0)["payload"]["analyzer"]["context_current"],
        true
    );
    other.run(
        &[
            "inspect",
            "entity",
            other_project.path().join("src/lib.rs").to_str().unwrap(),
            "--line",
            "1",
            "--column",
            "8",
        ],
        0,
    );
    other.stop();
    drop(session);
}

#[cfg(feature = "native-hir")]
#[tokio::test]
async fn extended_native_managed_macro_entry() {
    let Some(gate) = std::env::var_os("PSE_NATIVE_MACRO_GATE") else {
        return;
    };
    let gate = PathBuf::from(gate);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while !gate.exists() {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    extended_native_managed_macro_journey().await;
}

#[cfg(feature = "native-hir")]
#[tokio::test]
async fn extended_native_managed_stop_drains_slow_macro_without_stopping_another_session() {
    independent_cli(
        "semantic::tests::extended_native_managed_macro_entry",
        "PSE_NATIVE_MACRO_GATE",
    )
    .await;
}

#[cfg(feature = "native-hir")]
fn contextual_candidates_public_journey() {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("Cargo.toml"),"[package]\nname='contextual-candidate-control'\nversion='0.0.0'\nedition='2024'\n[workspace]\n").unwrap();
    std::fs::write(
        project.path().join("Cargo.lock"),
        "version=4\n[[package]]\nname='contextual-candidate-control'\nversion='0.0.0'\n",
    )
    .unwrap();
    std::fs::copy(
        checkout().join("rust-toolchain.toml"),
        project.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let file = project.path().join("src/lib.rs");
    let source = r#"pub struct Pair;
pub struct Container<T>(T);
impl<T> Container<T> { pub fn choose<U:Clone>(&self,value:U)->U {value} pub fn make<U:Default>(&self)->U {U::default()} }
mod extension { pub trait Extra {fn extra(&self)->u32;} impl Extra for super::Pair {fn extra(&self)->u32{1}} pub trait Blanket {fn blanket(&self);} impl<T:Clone> Blanket for T {fn blanket(&self){}} }
mod first {pub trait Same {fn same(&self)->u32;} impl Same for super::Pair {fn same(&self)->u32{1}}}
mod second {pub trait Same {fn same(&self)->u32;} impl Same for super::Pair {fn same(&self)->u32{2}}}
fn selected(text:String,mut values:Vec<u32>,typed:Container<u32>,p:Pair) {
    let _: &str = text.trim();
    values.push(7u32);
    let _: Option<&u32> = values.first();
    let _:u64 = typed.choose(9u64);
    let _:&str = typed.choose("nine");
    let _:u64 = typed.make();
    let _ = &typed;
    p.extra(); p.blanket(); p.same();
}
"#;
    std::fs::write(&file, source).unwrap();
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
    let inspect = |needle: &str| {
        let offset = source.find(needle).unwrap();
        let line = (source[..offset].bytes().filter(|b| *b == b'\n').count() + 1).to_string();
        let column = (source[..offset]
            .rsplit('\n')
            .next()
            .unwrap()
            .chars()
            .count()
            + 1)
        .to_string();
        session.run(
            &[
                "inspect",
                "entity",
                file.to_str().unwrap(),
                "--line",
                &line,
                "--column",
                &column,
                "--view",
                "candidates",
            ],
            0,
        )["payload"]
            .clone()
    };
    let contextual = |answer: &Value, name: &str| {
        answer["native_types"]["callable_candidates"]
            .as_array()
            .unwrap()
            .iter()
            .find(|candidate| {
                candidate["target"]["name"] == name && candidate["context"].is_object()
            })
            .unwrap_or_else(|| panic!("missing native contextual {name}: {answer}"))
            .clone()
    };
    let trim = contextual(&inspect("trim();"), "trim");
    assert_eq!(trim["receiver"]["display"], "String", "{trim}");
    assert_eq!(
        trim["context"]["receiver_parameter"]["display"], "&str",
        "{trim}"
    );
    let steps = trim["context"]["receiver_adjustments"].as_array().unwrap();
    assert!(
        steps.iter().any(|step| step["source"]["display"]
            .as_str()
            .is_some_and(|ty| ty.contains("String"))
            && step["target"]["display"] == "str"),
        "{trim}"
    );
    assert_eq!(steps.last().unwrap()["target"]["display"], "&str", "{trim}");
    let push = contextual(&inspect("push(7u32)"), "push");
    assert_eq!(push["context"]["parameters"][0]["display"], "u32", "{push}");
    assert_eq!(push["context"]["result"]["display"], "()", "{push}");
    assert!(
        push["context"]["substitutions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["parameter"] == "T" && s["argument"]["display"] == "u32"),
        "{push}"
    );
    let first = contextual(&inspect("first();"), "first");
    assert!(
        first["context"]["result"]["display"]
            .as_str()
            .unwrap()
            .contains("u32"),
        "{first}"
    );
    let choose = contextual(&inspect("choose(9u64)"), "choose");
    for (parameter, argument) in [("T", "u32"), ("U", "u64")] {
        assert!(
            choose["context"]["substitutions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|s| s["parameter"] == parameter && s["argument"]["display"] == argument),
            "{choose}"
        );
    }
    assert_eq!(
        choose["context"]["parameters"][0]["display"], "u64",
        "{choose}"
    );
    assert_eq!(choose["context"]["result"]["display"], "u64", "{choose}");
    assert!(
        choose["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|obligation| obligation["parameter"] == "U"
                && obligation["declared_trait"]["name"] == "Clone"
                && obligation["subject"]["display"] == "u64"
                && obligation["outcome"] == "holds"),
        "{choose}"
    );
    let second_choose = contextual(&inspect("choose(\"nine\")"), "choose");
    assert_eq!(
        second_choose["context"]["result"]["display"], "&str",
        "{second_choose}"
    );
    assert_ne!(
        choose["context"]["callsite"],
        second_choose["context"]["callsite"]
    );
    assert_eq!(
        choose["context"]["substitution_owner"],
        second_choose["context"]["substitution_owner"]
    );
    let make = contextual(&inspect("make();"), "make");
    assert_eq!(make["context"]["result"]["display"], "u64", "{make}");
    assert!(
        make["context"]["parameters"].as_array().unwrap().is_empty(),
        "{make}"
    );
    assert!(
        make["context"]["substitutions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["parameter"] == "U" && s["argument"]["display"] == "u64"),
        "{make}"
    );
    let unresolved = inspect("typed;");
    assert!(
        unresolved["native_types"]["callable_candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|candidate| candidate["target"]["name"] == "choose"
                && candidate["context"].is_null()
                && candidate["obligations"].as_array().unwrap().iter().any(
                    |bound| bound["declared_trait"]["name"] == "Clone"
                        && bound["subject"].is_null()
                        && bound["outcome"] == "unsupported"
                )),
        "{unresolved}"
    );
    let extension = inspect("extra();");
    assert!(
        extension["native_types"]["callable_candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|candidate| candidate["required_import"].as_str().is_some()
                && candidate["applicability"] == "holds"
                && candidate["context"].is_null()),
        "{extension}"
    );
    let blanket = inspect("blanket();");
    assert!(
        blanket["native_types"]["callable_candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|candidate| candidate["applicability"] == "not_proven"
                && candidate["obligations"].as_array().unwrap().iter().any(
                    |obligation| obligation["declared_trait"]["name"] == "Clone"
                        && obligation["subject"]["display"] == "Pair"
                        && obligation["outcome"] == "not_proven"
                )),
        "{blanket}"
    );
    let same = inspect("same();");
    let alternatives = same["native_types"]["callable_candidates"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|candidate| candidate["target"]["name"] == "same")
        .collect::<Vec<_>>();
    assert_eq!(alternatives.len(), 2, "{same}");
    assert_ne!(
        alternatives[0]["target"]["location"],
        alternatives[1]["target"]["location"]
    );
    assert!(
        alternatives
            .iter()
            .all(|candidate| candidate["context"].is_null()
                && candidate["required_import"].as_str().is_some()
                && !candidate["limitations"].as_array().unwrap().is_empty()),
        "{same}"
    );
    let offset = source.find("choose(9u64)").unwrap();
    let addressed = session.run(
        &[
            "inspect",
            "--source",
            file.to_str().unwrap(),
            "--range-start",
            &offset.to_string(),
            "--range-end",
            &(offset + 6).to_string(),
            "--view",
            "candidates",
        ],
        0,
    );
    let addressed = contextual(&addressed["payload"]["native"], "choose");
    assert_eq!(
        choose["context"]["substitutions"],
        addressed["context"]["substitutions"]
    );
    session.stop();
}

#[cfg(feature = "native-hir")]
#[tokio::test]
async fn contextual_candidates_public_entry() {
    let Some(gate) = std::env::var_os("PSE_CONTEXTUAL_CANDIDATE_GATE") else {
        return;
    };
    let gate = PathBuf::from(gate);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while !gate.exists() {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    contextual_candidates_public_journey();
}

#[cfg(feature = "native-hir")]
#[tokio::test]
async fn native_contextual_candidates_public_preserve_adjustments_substitutions_and_obligations() {
    independent_cli(
        "semantic::tests::contextual_candidates_public_entry",
        "PSE_CONTEXTUAL_CANDIDATE_GATE",
    )
    .await;
}
#[cfg(feature = "native-hir")]
fn native_ssr_selected_lines_public_journey() {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("Cargo.toml"),"[package]\nname='native-ssr-selection-control'\nversion='0.0.0'\nedition='2024'\n[workspace]\n").unwrap();
    std::fs::write(
        project.path().join("Cargo.lock"),
        "version=4\n[[package]]\nname='native-ssr-selection-control'\nversion='0.0.0'\n",
    )
    .unwrap();
    std::fs::copy(
        checkout().join("rust-toolchain.toml"),
        project.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let file = project.path().join("src/lib.rs");
    let source = "pub fn selected()->u32 {\r\n    let α = 1 + 2;\r\n    let value = 1 + 2;\r\n    α + value\r\n}\r\npub fn outside()->u32 {1 + 2}\r\n";
    std::fs::write(&file, source).unwrap();
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
    let preview = session.run(
        &[
            "propose",
            "ssr",
            file.to_str().unwrap(),
            "--line",
            "2",
            "--column",
            "5",
            "--end-line",
            "3",
            "--rule",
            "1 + 2 ==>> 3u32",
        ],
        0,
    );
    let candidates = preview["payload"]["proposal"]["candidates"]
        .as_array()
        .unwrap();
    assert_eq!(candidates.len(), 2, "{preview}");
    let outside = source.find("pub fn outside").unwrap();
    for candidate in candidates {
        let start = candidate["range"]["start"].as_u64().unwrap() as usize;
        let end = candidate["range"]["end"].as_u64().unwrap() as usize;
        assert!(
            end < outside,
            "matching node outside selected original lines must be excluded: {preview}"
        );
        assert_eq!(&source[start..end], "1 + 2", "{preview}");
        assert_eq!(candidate["replacement"], "3u32", "{preview}");
    }
    assert_eq!(
        preview["payload"]["proposal"]["selected"]
            .as_array()
            .unwrap()
            .len(),
        2,
        "{preview}"
    );
    let one_line = session.run(
        &[
            "propose",
            "ssr",
            file.to_str().unwrap(),
            "--line",
            "3",
            "--column",
            "5",
            "--rule",
            "1 + 2 ==>> 4u32",
        ],
        0,
    );
    assert_eq!(
        one_line["payload"]["proposal"]["candidates"]
            .as_array()
            .unwrap()
            .len(),
        1,
        "omitted end-line selects only the requested original line: {one_line}"
    );
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        source,
        "native SSR remains preview-only"
    );
    session.stop();
}

#[cfg(feature = "native-hir")]
#[tokio::test]
async fn native_ssr_selected_lines_public_entry() {
    let Some(gate) = std::env::var_os("PSE_NATIVE_SSR_SELECTION_GATE") else {
        return;
    };
    let gate = PathBuf::from(gate);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while !gate.exists() {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    native_ssr_selected_lines_public_journey();
}

#[cfg(feature = "native-hir")]
#[tokio::test]
async fn native_ssr_public_excludes_same_matching_node_outside_selected_original_lines() {
    independent_cli(
        "semantic::tests::native_ssr_selected_lines_public_entry",
        "PSE_NATIVE_SSR_SELECTION_GATE",
    )
    .await;
}

#[cfg(feature = "native-hir")]
fn native_runnable_discovery_public_journey() {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("Cargo.toml"),"[package]\nname='native-runnable-control'\nversion='0.0.0'\nedition='2024'\n[workspace]\n[features]\ngated=[]\n[dependencies]\nexpect-test={path='expect-test'}\ninsta={path='insta'}\nsnapbox={path='snapbox'}\n[[bin]]\nname='gated-binary'\npath='src/gated.rs'\nrequired-features=['gated']\n").unwrap();
    std::fs::write(project.path().join("Cargo.lock"),"version=4\n[[package]]\nname='native-runnable-control'\nversion='0.0.0'\ndependencies=['expect-test','insta','snapbox']\n[[package]]\nname='expect-test'\nversion='0.0.0'\n[[package]]\nname='insta'\nversion='0.0.0'\n[[package]]\nname='snapbox'\nversion='0.0.0'\n").unwrap();
    for (package, macro_name) in [
        ("expect-test", "expect"),
        ("insta", "assert_snapshot"),
        ("snapbox", "str"),
    ] {
        let root = project.path().join(package);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            format!("[package]\nname='{package}'\nversion='0.0.0'\nedition='2024'\n"),
        )
        .unwrap();
        std::fs::write(
            root.join("src/lib.rs"),
            format!("#[macro_export] macro_rules! {macro_name} {{ ($($tokens:tt)*) => {{}}; }}\n"),
        )
        .unwrap();
    }
    std::fs::copy(
        checkout().join("rust-toolchain.toml"),
        project.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let file = project.path().join("src/lib.rs");
    let source = r#"#![feature(test)]
extern crate test;
pub fn target()->u32 {7}
/// ```
/// assert_eq!(2 + 2, 4);
/// ```
pub fn documented() {}
#[bench] fn benchmark(b:&mut test::Bencher) { b.iter(||target()); }
#[cfg(feature="gated")] #[test] fn inactive_test() {target();}
#[cfg(test)] mod tests {
    #[test] fn related() {super::target();}
    #[test] fn passive_snapshot() {expect_test::expect!["expected"]; insta::assert_snapshot!("value"); snapbox::str!["expected"];}
}
"#;
    std::fs::write(&file, source).unwrap();
    let main = project.path().join("src/main.rs");
    std::fs::write(&main, "fn main(){std::hint::black_box(7);}\n").unwrap();
    let gated = project.path().join("src/gated.rs");
    std::fs::write(&gated, "fn main(){}\n").unwrap();
    let session = Session::new();
    session.run(&["tools", "prepare", "native-analysis"], 0);
    let prepare = |test_context: bool| {
        let mut args = vec![
            "code",
            "session",
            "prepare",
            "--backend",
            "native-snapshot",
            "--project",
            project.path().to_str().unwrap(),
        ];
        if test_context {
            args.push("--test-context");
        }
        session.run(&args, 0);
        session.ready();
    };
    prepare(true);
    let answer = session.run(
        &[
            "inspect",
            "entity",
            file.to_str().unwrap(),
            "--line",
            "3",
            "--column",
            "8",
            "--view",
            "tests",
            "--budget",
            "100",
        ],
        0,
    );
    let tests = answer["payload"]["observed"]["tests"].as_array().unwrap();
    for kind in ["test", "test_module", "bench", "doctest"] {
        assert!(
            tests
                .iter()
                .any(|candidate| candidate["runnable_kind"] == kind),
            "missing actual native {kind}: {answer}"
        );
    }
    assert!(
        tests.iter().any(|candidate| candidate["test_id"]
            .as_str()
            .is_some_and(|id| id.ends_with("tests::related"))
            && candidate["relatedness"]
                .as_str()
                .unwrap()
                .contains("related_tests")),
        "{answer}"
    );
    assert!(
        tests
            .iter()
            .any(|candidate| candidate["runnable_kind"] == "doctest"
                && candidate["relatedness"]
                    .as_str()
                    .unwrap()
                    .contains("file runnable discovery")),
        "{answer}"
    );
    let snapshot = tests
        .iter()
        .find(|candidate| {
            candidate["test_id"]
                .as_str()
                .is_some_and(|id| id.ends_with("tests::passive_snapshot"))
        })
        .unwrap();
    for field in ["expect_test", "insta", "snapbox"] {
        assert_eq!(snapshot["snapshot_update"][field], true, "{snapshot}");
    }
    assert!(
        tests.iter().all(
            |candidate| candidate["cargo_args"].as_array().unwrap().is_empty()
                && candidate["executable_args"].as_array().unwrap().is_empty()
        ),
        "HIR has no exported argv; no reconstruction: {answer}"
    );
    assert!(
        !tests.iter().any(|candidate| candidate["test_id"]
            .as_str()
            .is_some_and(|id| id.ends_with("inactive_test"))),
        "inactive native HIR test must not be nominated: {answer}"
    );
    assert!(
        answer["payload"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|section| section["category"] == "native_runnable_discovery"
                && section["evidence"]["inactive_regions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|region| region["cfg_eligible"] == false)),
        "inactive native cfg observation is retained: {answer}"
    );
    let binary = session.run(
        &[
            "inspect",
            "entity",
            main.to_str().unwrap(),
            "--line",
            "1",
            "--column",
            "5",
            "--view",
            "tests",
        ],
        0,
    );
    assert!(
        binary["payload"]["observed"]["tests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|candidate| candidate["runnable_kind"] == "binary"
                && candidate["cargo_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|target| target["kind"] == "binary"
                        && target["required_features_satisfied"] == true)),
        "{binary}"
    );
    let gated_answer = session.run(
        &[
            "inspect",
            "entity",
            gated.to_str().unwrap(),
            "--line",
            "1",
            "--column",
            "5",
            "--view",
            "tests",
        ],
        0,
    );
    assert!(
        gated_answer["payload"]["observed"]["tests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|candidate| candidate["runnable_kind"] == "binary"
                && candidate["eligible"] == false
                && candidate["cargo_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|target| target["required_features"][0] == "gated"
                        && target["required_features_satisfied"] == false)),
        "{gated_answer}"
    );
    assert!(
        !gated_answer["next"]
            .as_array()
            .unwrap()
            .iter()
            .any(|next| next["argv"]
                .as_array()
                .is_some_and(|argv| argv.iter().any(|arg| arg == "test-selection"))),
        "ineligible native target must not route to current selection: {gated_answer}"
    );
    assert!(
        !answer["next"]
            .as_array()
            .unwrap()
            .iter()
            .any(|next| next["argv"]
                .as_array()
                .is_some_and(|argv| argv.iter().any(|arg| arg == "test-selection"))),
        "an external native project cannot nominate the current checkout's test selection: {answer}"
    );
    let offset = source.find("target()->").unwrap();
    let addressed = session.run(
        &[
            "inspect",
            "--source",
            file.to_str().unwrap(),
            "--range-start",
            &offset.to_string(),
            "--range-end",
            &(offset + 6).to_string(),
            "--view",
            "tests",
            "--budget",
            "100",
        ],
        0,
    );
    assert!(
        addressed["payload"]["native"]["observed"]["tests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|candidate| candidate["test_id"] == "tests::related"),
        "consolidated public inspection retains actual native relevance: {addressed}"
    );
    assert!(
        !addressed["next"]
            .as_array()
            .unwrap()
            .iter()
            .any(|next| next["argv"]
                .as_array()
                .is_some_and(|argv| argv.iter().any(|arg| arg == "test-selection"))),
        "consolidated external-project inspection shares the checkout selection boundary: {addressed}"
    );
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        source,
        "snapshot intent is passive"
    );
    session.stop();
    prepare(false);
    let ordinary = session.run(
        &[
            "inspect",
            "entity",
            file.to_str().unwrap(),
            "--line",
            "3",
            "--column",
            "8",
            "--view",
            "tests",
        ],
        0,
    );
    assert!(
        ordinary["payload"]["observed"]["tests"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|candidate| matches!(
                candidate["runnable_kind"].as_str(),
                Some("test" | "test_module" | "bench")
            ))
            .all(|candidate| candidate["eligible"] != true),
        "ordinary selected world does not admit test-profile candidates: {ordinary}"
    );
    session.stop();
}

#[cfg(feature = "native-hir")]
#[tokio::test]
async fn native_runnable_discovery_public_entry() {
    let Some(gate) = std::env::var_os("PSE_NATIVE_RUNNABLE_GATE") else {
        return;
    };
    let gate = PathBuf::from(gate);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while !gate.exists() {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    native_runnable_discovery_public_journey();
}

#[cfg(feature = "native-hir")]
#[tokio::test]
async fn native_runnables_public_classify_cfg_targets_and_passive_snapshot_intent() {
    independent_cli(
        "semantic::tests::native_runnable_discovery_public_entry",
        "PSE_NATIVE_RUNNABLE_GATE",
    )
    .await;
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
pub(crate) struct Session {
    root: PathBuf,
    executable: PathBuf,
    state: tempfile::TempDir,
}
impl Session {
    pub(crate) fn new() -> Self {
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
    pub(crate) fn run(&self, arguments: &[&str], exit: i32) -> Value {
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
        serde_json::from_slice(if output.stdout.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        })
        .unwrap()
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
    pub(crate) fn ready(&self) {
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
    pub(crate) fn stop(&self) {
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
        answer["payload"]["observed"]["relationships"]
            .as_array()
            .unwrap()
            .iter()
            .any(|fact| fact["relation"] == "definition" && fact["location"].is_object()),
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
    independent_cli("semantic::tests::native_cli_entry", "PSE_SEMANTIC_CLI_GATE").await;
}

pub(crate) async fn independent_cli(entry: &str, gate_name: &str) {
    // Like B03's detached-front-end fixture, first place this independent CLI caller
    // in its own ordinary application scope. Product nested-detachment refusal stays intact.
    let scratch = tempfile::tempdir().unwrap();
    let gate = scratch.path().join("go");
    let manager = crate::HostManager::connect_user(None).await.unwrap();
    let current = manager.current_unit().await.unwrap();
    let (slice, _, _) = manager.foreground_placement(&current).await.unwrap();
    let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", entry, "--nocapture"])
        .env(gate_name, &gate)
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
        version: SESSION_PROTOCOL_VERSION,
        invocation,
        configuration: Configuration {
            backend: Backend::Lsp,
            project: root.clone(),
            features: vec![],
            no_default_features: false,
            target: None,
            build_macros: false,
            test_context: false,
        },
    };
    for version in 1..SESSION_PROTOCOL_VERSION {
        let old = SessionRecord {
            version,
            ..record.clone()
        };
        let refused = Client::connect(&root, &environment, &old).await;
        assert!(
            matches!(refused,Err(Error::Unavailable(message)) if message.contains("incompatible protocol")),
            "old worlds retain administration only"
        );
    }
    let refused = client(&root, &environment, &record, &Request::Status {}).await;
    assert!(refused.is_err());
    assert_eq!(
        peer.join().unwrap(),
        0,
        "no request bytes before original-custody authentication"
    );
    std::fs::remove_file(socket).unwrap();
}
