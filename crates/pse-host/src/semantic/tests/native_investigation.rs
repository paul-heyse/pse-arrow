// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual public CLI journeys through the prepared native session and typed wire.
use super::*;
fn journey() {
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("Cargo.toml"),"[package]\nname='native-investigation-public'\nversion='0.0.0'\nedition='2024'\n[workspace]\n").unwrap();
    std::fs::write(
        project.path().join("Cargo.lock"),
        "version=4\n[[package]]\nname='native-investigation-public'\nversion='0.0.0'\n",
    )
    .unwrap();
    std::fs::copy(
        checkout().join("rust-toolchain.toml"),
        project.path().join("rust-toolchain.toml"),
    )
    .unwrap();
    let source = "pub struct Policy;\npub trait Base {}\npub trait Child:Base{}\nfn compare(){let mut n=1u8;let mutable=&mut n;let shared=&1u8;mutable;shared;}\nfn origin(){let captured=7u8;let _=format!(\"{captured}\");}\n";
    let file = project.path().join("src/lib.rs");
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
    let inspect = |needle: &str, extra: &[&str]| {
        let start = source.find(needle).unwrap();
        let start = start.to_string();
        let end = (start.parse::<usize>().unwrap() + needle.len()).to_string();
        let mut argv = vec![
            "inspect",
            "--source",
            file.to_str().unwrap(),
            "--range-start",
            &start,
            "--range-end",
            &end,
        ];
        argv.extend_from_slice(extra);
        session.run(&argv, 0)
    };
    let rhs = inspect("shared;", &[]);
    let reference = rhs["payload"]["reference"].clone();
    let reference: crate::investigation::ResultReference =
        serde_json::from_value(reference).unwrap_or_else(|e| panic!("reference: {e}; {rhs}"));
    let mut plural = reference.clone();
    if let crate::investigation::Locator::Source {
        range: Some(range), ..
    } = &mut plural.locator
    {
        range.byte_start = source.find("mutable;shared;").unwrap();
        range.byte_end = range.byte_start + "mutable;shared;".len();
    } else {
        panic!("expected guarded source operand: {rhs}");
    }
    let plural = plural.encode().unwrap();
    let reference = reference.encode().unwrap();
    let compared = inspect(
        "mutable;",
        &[
            "--view",
            "compatibility",
            "--against-result",
            &reference,
            "--operation",
            "could-coerce",
        ],
    );
    assert_eq!(
        compared["payload"]["native"]["observed"]["investigation"]["compatibility"]["result"], true,
        "{compared}"
    );
    let equal = inspect(
        "mutable;",
        &[
            "--view",
            "compatibility",
            "--against-result",
            &reference,
            "--operation",
            "equal",
        ],
    );
    assert_eq!(
        equal["payload"]["native"]["observed"]["investigation"]["compatibility"]["result"], false,
        "{equal}"
    );
    let start = source.find("mutable;").unwrap();
    let refused = session.run(
        &[
            "inspect",
            "--source",
            file.to_str().unwrap(),
            "--range-start",
            &start.to_string(),
            "--range-end",
            &(start + "mutable;".len()).to_string(),
            "--view",
            "compatibility",
            "--against-result",
            &plural,
            "--operation",
            "equal",
        ],
        125,
    );
    assert!(
        refused["payload"].is_null() && !refused["failures"].as_array().unwrap().is_empty(),
        "multiple RHS expressions must not produce a selected comparison: {refused}"
    );
    let origins = inspect("\"{captured}", &["--view", "format-origin"]);
    let observations = origins["payload"]["native"]["observed"]["investigation"]["origins"]
        .as_array()
        .unwrap();
    assert_eq!(observations.len(), 1, "{origins}");
    assert_eq!(observations[0]["target_kind"], "local");
    assert!(observations[0]["target"].is_object());
    session.stop();
}
#[tokio::test]
async fn native_investigation_public_entry() {
    let Some(gate) = std::env::var_os("PSE_NATIVE_INVESTIGATION_GATE") else {
        return;
    };
    let gate = PathBuf::from(gate);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while !gate.exists() {
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    journey();
}
#[tokio::test]
async fn public_native_comparison_and_origin_views_preserve_their_original_context() {
    independent_cli(
        "semantic::tests::native_investigation::native_investigation_public_entry",
        "PSE_NATIVE_INVESTIGATION_GATE",
    )
    .await;
}
