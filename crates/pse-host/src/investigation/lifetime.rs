// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Qualified lifetime evidence; source correspondence is checked rather than inferred from displays.
use super::{Locator, ResultReference};
use crate::{HostError, compiler_investigation::facts, semantic};
use schemars::JsonSchema;
use serde::Serialize;
use std::{collections::BTreeSet, num::NonZeroUsize, path::Path};
use tokio_util::sync::CancellationToken;

/// A body-level correspondence does not identify individual captures or executed effects.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub(crate) enum Correspondence {
    /// One body is bound to original bytes and the same selected compilation premises.
    Matched {
        compiler_body: usize,
        source_file: u32,
    },
    /// No compiler report was supplied.
    NotRequested,
    /// Historical, unavailable, incompatible or plural evidence cannot be joined.
    Unjoined { reasons: Vec<String> },
}
/// Supplied work evidence remains independently qualified; it is not a source-code execution link.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct OperationalEvidence {
    pub association: String,
    pub original: crate::invocation_view::ResultView,
}
/// One selected body's original native/compiler observations and optional original work outcome.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct Report {
    pub reference: ResultReference,
    pub native: semantic::Answer,
    pub compiler: Option<facts::Report>,
    pub compiler_reference: Option<ResultReference>,
    pub compiler_expansion: Vec<crate::envelope::NextAction>,
    pub correspondence: Correspondence,
    pub operational: Option<OperationalEvidence>,
    pub limitations: Vec<String>,
}

pub(crate) async fn inspect(
    root: &Path,
    locator: &Locator,
    compiler_evidence: Option<&str>,
    work_result: Option<&str>,
    token: &CancellationToken,
) -> Result<Report, HostError> {
    let mut selected = super::rust::source_inquiry(root, locator, token).await?;
    let native =
        super::rust::semantic_operation_cancellable(root, &selected, semantic::Kind::Body, token)
            .await?;
    if let Locator::Semantic {
        generation,
        source_sha256,
        ..
    } = &mut selected
    {
        *generation = Some(native.generation.to_string());
        *source_sha256 = Some(native.source.source_sha256().into());
    }
    let state = crate::paths::CheckoutState::resolve(root, &std::env::vars_os().collect())?;
    let body = native
        .observed
        .investigation
        .as_ref()
        .and_then(|facts| facts.body.as_ref());
    let compiler_reference = compiler_evidence.map(ResultReference::decode).transpose()?;
    let compiler_response = if let Some(reference) = &compiler_reference {
        let Locator::Evidence {
            invocation,
            kind: super::EvidenceKind::Artifact,
            selector,
        } = &reference.locator
        else {
            return Err(super::Error::Invalid(
                "lifetime compiler evidence requires an original artifact result reference".into(),
            )
            .into());
        };
        let _: crate::envelope::CliInvocationId = invocation.clone().try_into().map_err(|_| {
            super::Error::Invalid(
                "lifetime compiler reference requires an exact original invocation identity".into(),
            )
        })?;
        let ordinal = selector.as_deref().and_then(|value|value.strip_prefix("compiler:")).and_then(|value|value.parse().ok())
            .ok_or_else(||super::Error::Invalid("lifetime compiler evidence requires the original compiler:ORDINAL facts artifact selector".into()))?;
        let budget = NonZeroUsize::new(1_048_576)
            .ok_or_else(|| super::Error::Invalid("invalid lifetime byte budget".into()))?;
        Some(facts::observe_for_source(
            root,
            &state,
            invocation,
            ordinal,
            body.and_then(|body| body.location.as_ref()),
            budget,
        )?)
    } else {
        None
    };
    let (compiler, compiler_expansion) = compiler_response.map_or((None, Vec::new()), |response| {
        (Some(response.payload), response.next)
    });
    let correspondence = compiler
        .as_ref()
        .map_or(Correspondence::NotRequested, |compiler| {
            correspond(
                &native,
                body.and_then(|body| body.location.as_ref()),
                body.and_then(|body| body.package_id.as_deref()),
                body.and_then(|body| body.role.as_deref()),
                compiler,
            )
        });
    let operational = work_result.map(|selector| {
        let original = crate::invocation_view::result(&crate::invocation::InvocationStore::new(state.clone()), selector, None)?.payload;
        Ok::<_, HostError>(OperationalEvidence { association: "user_supplied_accompanying_evidence; no body-to-execution association is established".into(), original })
    }).transpose()?;
    Ok(Report { reference: ResultReference::new(selected), native, compiler, compiler_reference, compiler_expansion, correspondence, operational, limitations: vec![
        "Compiler presentation is limited to bodies nominated by the selected original source range and a one MiB whole-body budget; nomination alone is not correspondence. Use the retained compiler reference for the full acquisition.".into(),
        "A common body does not identify individual native/compiler captures; arrays are not zipped and place/type displays are not join keys.".into(),
        "Correspondence checks the selected body, crate-root bytes and recorded unit premises. The retained context identifies captured inputs; it does not assert every transitive compiler read was captured.".into(),
        "Compiler variant field indexes identify compiler saved storage. Storage, initialization, active loans, static cleanup and observed destruction are distinct facts; active loans are not extracted.".into(),
        "Cancellation or drop syntax and supplied operational evidence do not establish runtime destruction or CPU settlement of this body.".into(),
        "A shared-reference capture does not establish immutability or absence of interior mutation. Native adjustments and unresolved macro/call boundaries keep their original provider qualifications.".into(),
    ] })
}

fn correspond(
    native: &semantic::Answer,
    body: Option<&semantic::NativeLocation>,
    package: Option<&str>,
    role: Option<&str>,
    compiler: &facts::Report,
) -> Correspondence {
    let mut reasons = Vec::new();
    let Some(context) = native
        .native_types
        .as_ref()
        .and_then(|facts| facts.context.as_ref())
    else {
        return Correspondence::Unjoined {
            reasons: vec!["native type/unit context is unavailable".into()],
        };
    };
    if compiler.context.rustc_identity != context.compiler_identity {
        reasons.push("compiler identity differs".into());
    }
    if compiler.context.source.path() != context.crate_root
        || context.crate_source_sha256.as_deref() != Some(compiler.context.source.source_sha256())
    {
        reasons.push("original crate-root source bytes are missing or differ".into());
    }
    if compiler.edition.as_deref() != Some(context.edition.as_str()) {
        reasons.push("compiler edition is missing or differs".into());
    }
    if compiler.target.as_ref() != context.target.as_ref() || compiler.target.is_none() {
        reasons.push("compiler target is missing or differs".into());
    }
    let compiler_cfg = cfg_set(&compiler.cfg);
    let native_cfg = cfg_set(&context.cfg);
    if compiler_cfg.is_empty() || compiler_cfg != native_cfg {
        reasons.push("actual cfg/test context is missing or differs".into());
    }
    let units = compiler
        .context
        .roots
        .iter()
        .filter(|unit| {
            package == Some(unit.package_id.as_str())
                && unit.source_path == context.crate_root.to_string_lossy()
                && unit.features == context.features
                && role.is_some_and(|role| unit.target_kinds.iter().any(|kind| kind == role))
        })
        .count();
    if units != 1 {
        reasons.push("package/unit role or feature correspondence is absent or plural".into());
    }
    let Some(body) = body else {
        reasons.push("native body has no unique original source mapping".into());
        return Correspondence::Unjoined { reasons };
    };
    let mut matched = Vec::new();
    for (index, candidate) in compiler.bodies.iter().enumerate() {
        let Some(source) = candidate
            .source
            .as_ref()
            .filter(|source| !source.macro_expanded)
        else {
            continue;
        };
        let Some(file) = source
            .source_file
            .and_then(|id| compiler.source_files.iter().find(|file| file.id == id))
        else {
            continue;
        };
        if file.path.to_path_buf() != body.path
            || source.original_start.map(|v| v as usize) != Some(body.start)
            || source.original_end.map(|v| v as usize) != Some(body.end)
        {
            continue;
        }
        let native_digest = if native.source.path() == body.path {
            Some(native.source.source_sha256())
        } else {
            native
                .location_sources
                .iter()
                .find(|source| source.path == body.path)
                .map(|source| source.source_sha256.as_str())
        };
        if native_digest == Some(file.original_sha256.as_str()) {
            matched.push((index, file.id));
        }
    }
    if matched.len() != 1 {
        reasons.push(
            "original body/source correspondence is missing, changed, expanded or plural".into(),
        );
    }
    if reasons.is_empty() {
        let (compiler_body, source_file) = matched[0];
        Correspondence::Matched {
            compiler_body,
            source_file,
        }
    } else {
        Correspondence::Unjoined { reasons }
    }
}

fn cfg_set(values: &[String]) -> BTreeSet<String> {
    // ra_ap_cfg's intrinsic `true` is a tautology, not a selected compiler option.
    // Preserve every actual flag (including rust_analyzer/test) and the complete value.
    values
        .iter()
        .map(|value| {
            value.split_once('=').map_or_else(
                || value.trim().to_owned(),
                |(key, value)| format!("{}={}", key.trim(), value.trim()),
            )
        })
        .filter(|value| value != "true")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cfg_canonicalization_preserves_actual_flags_test_and_values() {
        assert_eq!(
            cfg_set(&["true".into(), "feature = \"a b\"".into()]),
            cfg_set(&["feature=\"a b\"".into()])
        );
        assert_ne!(cfg_set(&["rust_analyzer".into()]), cfg_set(&[]));
        assert_ne!(cfg_set(&["test".into()]), cfg_set(&[]));
    }
    #[cfg(feature = "native-hir")]
    #[test]
    fn lifetime_cli_entry() {
        let Some(gate) = std::env::var_os("PSE_LIFETIME_CLI_GATE") else {
            return;
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        while !Path::new(&gate).exists() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let session = semantic::tests::Session::new();
        session.run(&["tools", "prepare", "native-analysis"], 0);
        session.run(&["tools", "prepare", "compiler-analysis"], 0);
        let created = session.run(&["probe", "new", "lifetime-evidence"], 0);
        let project = std::path::PathBuf::from(created["payload"]["directory"].as_str().unwrap());
        std::fs::create_dir(project.join(".cargo")).unwrap();
        let manifest_path = project.join("Cargo.toml");
        let manifest = format!(
            "{}\n[features]\ndefault=[]\nalt=[]\n",
            std::fs::read_to_string(&manifest_path).unwrap()
        );
        std::fs::write(&manifest_path, &manifest).unwrap();
        // Pinned RA obtains cfg with -O and appends these unstable atomic predicates.
        // Select the same predicates/check settings explicitly for this positive fixture;
        // the production consumer must keep ordinary divergent cfg contexts unjoined.
        let mut flags = vec![
            "-Aexplicit_builtin_cfgs_in_flags".to_owned(),
            "-Cdebug-assertions=no".into(),
            "-Coverflow-checks=no".into(),
            "-Zub-checks=no".into(),
            "--cfg".into(),
            "rust_analyzer".into(),
        ];
        for width in ["8", "16", "32", "64", "128", "ptr"] {
            flags.extend([
                "--cfg".into(),
                format!("target_has_atomic_equal_alignment=\"{width}\""),
            ]);
        }
        flags.extend([
            "--cfg".into(),
            "target_has_atomic_load_store=\"128\"".into(),
        ]);
        std::fs::write(
            project.join(".cargo/config.toml"),
            format!("[build]\nrustflags={flags:?}\n"),
        )
        .unwrap();
        // Compiled and inspected, never run: each operation is source/static evidence only.
        let text = r#"use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
struct ResetGuard(Arc<AtomicBool>);
impl Drop for ResetGuard { fn drop(&mut self) { self.0.store(false, Ordering::Release); } }
pub fn target() {
    let cancellation = Arc::new(AtomicBool::new(false));
    let sibling_suppressed = Arc::new(AtomicBool::new(true));
    let cpu_cancel = Arc::clone(&cancellation);
    let cpu_owner = std::thread::spawn(move || {
        while !cpu_cancel.load(Ordering::Acquire) { std::thread::yield_now(); }
    });
    let retained_guard = ResetGuard(Arc::clone(&sibling_suppressed));
    let retained_future = async move {
        let _guard = retained_guard;
        std::future::pending::<()>().await;
    };
    let text = Box::new(String::from("é"));
    let cancel_and_settle = move || {
        cancellation.store(true, Ordering::Release);
        drop(retained_future);
        let _ = cpu_owner.join();
        let adjusted: &str = &text;
        std::hint::black_box(adjusted);
        sibling_suppressed.store(false, Ordering::Release);
    };
    std::hint::black_box(cancel_and_settle);
}
pub fn unrelated()->u32{3}
fn main() {}
"#;
        let file = project.join("src/main.rs");
        std::fs::write(&file, text).unwrap();
        let generated = session.run(
            &[
                "compiler",
                "probe",
                "lifetime-evidence",
                "--kind",
                "facts",
                "--body",
                "target",
                "--question",
                "capture and original source correspondence",
            ],
            0,
        );
        let invocation = generated["invocation_id"].as_str().unwrap();
        let inventory = session.run(&["inspect", "compiler-artifacts", invocation], 0);
        let acquired = session.run(
            &[
                "inspect",
                "compiler-facts",
                invocation,
                "--max-bytes",
                "1048576",
            ],
            0,
        );
        let ordinal = inventory["payload"]["index"]["files"]
            .as_array()
            .unwrap()
            .iter()
            .position(|file| {
                serde_json::from_value::<std::ffi::OsString>(file["path"].clone()).is_ok_and(
                    |path| {
                        Path::new(&path)
                            .file_name()
                            .is_some_and(|name| name == "facts.binpb")
                    },
                )
            })
            .unwrap();
        let reference = ResultReference::new(Locator::Evidence {
            invocation: invocation.into(),
            kind: super::super::EvidenceKind::Artifact,
            selector: Some(format!("compiler:{ordinal}")),
        })
        .encode()
        .unwrap();
        // Acquire the incompatible original before preparing a native world, then restore
        // its manifest. This exercises evidence mismatch without bypassing stale-world guards.
        std::fs::write(
            &manifest_path,
            manifest.replace("default=[]", "default=['alt']"),
        )
        .unwrap();
        let alternate = session.run(
            &[
                "compiler",
                "probe",
                "lifetime-evidence",
                "--kind",
                "facts",
                "--body",
                "target",
                "--question",
                "different actual feature context",
            ],
            0,
        );
        std::fs::write(&manifest_path, &manifest).unwrap();
        let alternate_invocation = alternate["invocation_id"].as_str().unwrap();
        let alternate_inventory =
            session.run(&["inspect", "compiler-artifacts", alternate_invocation], 0);
        let alternate_ordinal = alternate_inventory["payload"]["index"]["files"]
            .as_array()
            .unwrap()
            .iter()
            .position(|file| {
                serde_json::from_value::<std::ffi::OsString>(file["path"].clone()).is_ok_and(
                    |path| {
                        Path::new(&path)
                            .file_name()
                            .is_some_and(|name| name == "facts.binpb")
                    },
                )
            })
            .unwrap();
        let alternate_reference = ResultReference::new(Locator::Evidence {
            invocation: alternate_invocation.into(),
            kind: super::super::EvidenceKind::Artifact,
            selector: Some(format!("compiler:{alternate_ordinal}")),
        })
        .encode()
        .unwrap();
        session.run(
            &[
                "code",
                "session",
                "prepare",
                "--backend",
                "native-snapshot",
                "--project",
                project.to_str().unwrap(),
            ],
            0,
        );
        session.ready();
        let before = &text[..text.find("cancellation.store").unwrap()];
        let line = before.lines().count();
        let column = before.rsplit('\n').next().unwrap().chars().count() + 1;
        let joined = session.run(
            &[
                "inspect",
                "entity",
                file.to_str().unwrap(),
                "--line",
                &line.to_string(),
                "--column",
                &column.to_string(),
                "--view",
                "lifetime",
                "--compiler-evidence",
                &reference,
                "--work-result",
                invocation,
            ],
            0,
        );
        assert_eq!(
            joined["payload"]["correspondence"]["state"], "matched",
            "joined={joined}; original_acquired={acquired}"
        );
        assert_eq!(
            joined["payload"]["compiler"]["bodies"]
                .as_array()
                .unwrap()
                .len(),
            1,
            "one selected closure, not its sibling/owner population"
        );
        assert!(
            !joined["payload"]["native"]["native_types"]["captures"].is_null(),
            "{joined}"
        );
        let body = &joined["payload"]["compiler"]["bodies"][0];
        assert!(
            body["captures"].as_array().unwrap().len() >= 4,
            "cancel signal, retained future, CPU owner and sibling flag remain separate captures: {joined}"
        );
        assert!(
            body["cleanup"]
                .as_array()
                .unwrap()
                .iter()
                .any(|fact| fact["terminator"] == "drop"),
            "{joined}"
        );
        let phases = body["cleanup"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|fact| fact["phase"].as_str())
            .collect::<BTreeSet<_>>();
        assert!(
            phases.len() >= 2,
            "static phases remain qualified: {joined}"
        );
        assert!(
            joined["payload"]["operational"]["association"]
                .as_str()
                .unwrap()
                .contains("user_supplied")
        );
        assert_eq!(
            joined["payload"]["operational"]["original"]["original_invocation"],
            invocation
        );
        assert_eq!(
            joined["payload"]["operational"]["original"]["response"]["command"], "compiler probe",
            "original supplied work compiled the fixture; it did not execute target"
        );
        assert!(
            joined["payload"]["limitations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value
                    .as_str()
                    .is_some_and(|text| text.contains("CPU settlement")))
        );
        let source_id = joined["payload"]["correspondence"]["source_file"]
            .as_u64()
            .unwrap();
        let original = session.run(
            &[
                "inspect",
                "compiler-source",
                invocation,
                &source_id.to_string(),
            ],
            0,
        );
        assert_eq!(original["payload"]["content"]["data"], text);
        assert!(
            original["payload"]["content"]["data"]
                .as_str()
                .unwrap()
                .contains("drop(retained_future)")
        );
        assert!(
            original["payload"]["content"]["data"]
                .as_str()
                .unwrap()
                .contains("cpu_owner.join()")
        );
        let adjustment_start = text.find("&text;").unwrap();
        let adjustment_prefix = &text[..adjustment_start];
        let adjustment_line = adjustment_prefix.lines().count();
        let adjustment_column = adjustment_prefix
            .rsplit('\n')
            .next()
            .unwrap()
            .chars()
            .count()
            + 1;
        let adjusted = session.run(
            &[
                "inspect",
                "entity",
                file.to_str().unwrap(),
                "--line",
                &adjustment_line.to_string(),
                "--column",
                &adjustment_column.to_string(),
                "--view",
                "lifetime",
                "--compiler-evidence",
                &reference,
            ],
            0,
        );
        assert_eq!(
            adjusted["payload"]["correspondence"]["state"], "matched",
            "{adjusted}"
        );
        assert!(
            !adjusted["payload"]["native"]["native_types"]["adjustments"]
                .as_array()
                .unwrap()
                .is_empty(),
            "overloaded Box/String dereference remains an actual native observation: {adjusted}"
        );
        let offset = text.find('é').unwrap();
        let split = session.run(
            &[
                "inspect",
                "compiler-source",
                invocation,
                &source_id.to_string(),
                "--offset",
                &offset.to_string(),
                "--length",
                "1",
            ],
            0,
        );
        assert_eq!(split["payload"]["content"]["encoding"], "base64");
        assert_eq!(split["payload"]["content"]["data"], "ww==");
        assert_eq!(split["payload"]["next_offset"], offset + 1);
        let unrelated_line = text[..text.find("pub fn unrelated").unwrap()]
            .lines()
            .count()
            + 1;
        let unjoined = session.run(
            &[
                "inspect",
                "entity",
                file.to_str().unwrap(),
                "--line",
                &unrelated_line.to_string(),
                "--column",
                "8",
                "--view",
                "lifetime",
                "--compiler-evidence",
                &reference,
            ],
            1,
        );
        assert_eq!(
            unjoined["payload"]["correspondence"]["state"], "unjoined",
            "{unjoined}"
        );
        assert!(
            unjoined["payload"]["compiler"]["bodies"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let incompatible = session.run(
            &[
                "inspect",
                "entity",
                file.to_str().unwrap(),
                "--line",
                &line.to_string(),
                "--column",
                &column.to_string(),
                "--view",
                "lifetime",
                "--compiler-evidence",
                &alternate_reference,
            ],
            1,
        );
        assert_eq!(
            incompatible["payload"]["correspondence"]["state"], "unjoined",
            "{incompatible}"
        );
        assert!(
            incompatible["payload"]["correspondence"]["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value
                    .as_str()
                    .is_some_and(|text| text.contains("cfg/test") || text.contains("feature"))),
            "{incompatible}"
        );
        std::fs::write(&file, "changed after compilation").unwrap();
        let retained = session.run(
            &[
                "inspect",
                "compiler-source",
                invocation,
                &source_id.to_string(),
            ],
            0,
        );
        assert_eq!(
            retained["payload"]["content"]["data"], text,
            "expansion must read the retained bytes"
        );
        session.run(&["code", "session", "stop"], 0);
        session.run(&["probe", "rm", "lifetime-evidence"], 0);
    }
    #[cfg(feature = "native-hir")]
    #[tokio::test]
    async fn public_lifetime_joins_original_producer_and_native_body_and_keeps_other_bodies_unjoined()
     {
        semantic::tests::independent_cli(
            "investigation::lifetime::tests::lifetime_cli_entry",
            "PSE_LIFETIME_CLI_GATE",
        )
        .await;
    }
    #[cfg(feature = "native-hir")]
    #[test]
    fn lifetime_interrupt_cli_entry() {
        use std::process::Stdio;
        use std::time::{Duration, Instant};
        let Some(gate) = std::env::var_os("PSE_LIFETIME_INTERRUPT_GATE") else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(15);
        while !Path::new(&gate).exists() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let project = tempfile::tempdir().unwrap();
        std::fs::create_dir(project.path().join("src")).unwrap();
        std::fs::write(project.path().join("Cargo.toml"),"[package]\nname='lifetime-interrupt-fixture'\nversion='0.0.0'\nedition='2024'\n[workspace]\n").unwrap();
        std::fs::write(
            project.path().join("Cargo.lock"),
            "version=4\n[[package]]\nname='lifetime-interrupt-fixture'\nversion='0.0.0'\n",
        )
        .unwrap();
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        std::fs::copy(
            checkout.join("rust-toolchain.toml"),
            project.path().join("rust-toolchain.toml"),
        )
        .unwrap();
        // Actual cold HIR/body inference supplies work. There is no sleep, barrier or
        // test-only delay in production; the test requires observing the live request.
        let text = format!(
            "pub fn quick()->u32{{7}}\npub fn target(){{let mut n=0u32;{}std::hint::black_box(n);}}\n",
            "n+=1;".repeat(60_000)
        );
        let file = project.path().join("src/lib.rs");
        std::fs::write(&file, &text).unwrap();
        let session = semantic::tests::Session::new();
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
        let column = text.lines().nth(1).unwrap().find("n=0u32").unwrap() + 1;
        let mut interrupted = session
            .command()
            .args([
                "inspect",
                "entity",
                file.to_str().unwrap(),
                "--line",
                "2",
                "--column",
                &column.to_string(),
                "--view",
                "lifetime",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let status = session.run(&["code", "session", "status"], 0);
            if status["payload"]["analyzer"]["active_requests"]
                .as_u64()
                .unwrap_or(0)
                > 0
            {
                break;
            }
            assert!(
                interrupted.try_wait().unwrap().is_none(),
                "cold native body completed before actual SIGINT could be exercised; the fixture established no cancellation evidence"
            );
            assert!(
                Instant::now() < deadline,
                "original lifetime request never became active: {status}"
            );
            std::thread::yield_now();
        }
        let mut sibling = session
            .command()
            .args([
                "inspect",
                "entity",
                file.to_str().unwrap(),
                "--line",
                "1",
                "--column",
                "8",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let pid = rustix::process::Pid::from_raw(interrupted.id().try_into().unwrap()).unwrap();
        rustix::process::kill_process(pid, rustix::process::Signal::INT).unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        while interrupted.try_wait().unwrap().is_none() {
            assert!(
                Instant::now() < deadline,
                "SIGINT lifetime client did not await original request settlement"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        let cancelled = interrupted.wait_with_output().unwrap();
        assert_eq!(
            cancelled.status.code(),
            Some(125),
            "lifetime cancellation is a typed boundary response, not process termination: {cancelled:?}"
        );
        let response: serde_json::Value = serde_json::from_slice(&cancelled.stderr).unwrap();
        let diagnostic = response["failures"].to_string();
        assert!(
            diagnostic.contains("cancellation observed") && diagnostic.contains("CpuSettled"),
            "{response}"
        );
        while sibling.try_wait().unwrap().is_none() {
            assert!(
                Instant::now() < deadline,
                "sibling native inspection failed to settle"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        let sibling = sibling.wait_with_output().unwrap();
        assert!(sibling.status.success(), "{sibling:?}");
        let sibling: serde_json::Value = serde_json::from_slice(&sibling.stdout).unwrap();
        assert_eq!(sibling["payload"]["availability"], "available", "{sibling}");
        let status = session.run(&["code", "session", "status"], 0);
        assert_eq!(
            status["payload"]["analyzer"]["active_requests"], 0,
            "{status}"
        );
        assert_eq!(
            status["payload"]["analyzer"]["context_current"], true,
            "{status}"
        );
        session.run(&["code", "session", "stop"], 0);
    }
    #[cfg(feature = "native-hir")]
    #[tokio::test]
    async fn public_lifetime_sigint_observes_native_settlement_and_preserves_sibling() {
        semantic::tests::independent_cli(
            "investigation::lifetime::tests::lifetime_interrupt_cli_entry",
            "PSE_LIFETIME_INTERRUPT_GATE",
        )
        .await;
    }
}
