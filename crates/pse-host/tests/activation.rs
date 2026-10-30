// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native activation crosses the shell boundary as one private data transfer.
#![cfg(test)]
use pse_host::environment::{CacheMode, activation_script};
use std::{
    ffi::OsString,
    fs,
    os::unix::{ffi::OsStringExt as _, fs::PermissionsExt as _},
    path::Path,
    process::{Command, Output},
};

fn checkout() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".config")).unwrap();
    fs::create_dir_all(root.path().join(".cargo")).unwrap();
    fs::write(root.path().join(".python-version"), "3.14.8\n").unwrap();
    fs::write(root.path().join(".cargo/config.toml"), "[build]\njobs=32\n").unwrap();
    fs::write(
        root.path().join(".config/build.toml"),
        "cache_size='100G'\ncache_max_frame_length=268435456\n",
    )
    .unwrap();
    fs::write(
        root.path().join(".config/diagnostic-tools.toml"),
        "[sccache]\nversion='0.19.0'\nmember='sccache'\n",
    )
    .unwrap();
    fs::write(root.path().join(".config/sccache.toml"), "").unwrap();
    fs::write(
        root.path().join(".config/agent-capacity.toml"),
        "[test-runner]\nrust_thread_stack_mib=16\n",
    )
    .unwrap();
    root
}
fn native(root: &Path) -> Command {
    let mut cmd = Command::new(
        std::env::var_os("CARGO_BIN_EXE_pse-host")
            .expect("Cargo/nextest supplies the selected runtime host executable"),
    );
    cmd.current_dir(root)
        .env_clear()
        .env("HOME", root)
        .env("PATH", "/usr/bin:/bin");
    cmd
}
fn shell(root: &Path, body: &str) -> Command {
    let mut cmd = Command::new("/bin/bash");
    cmd.args(["--noprofile", "--norc", "-c", body, "pse-activation-test"])
        .arg(
            std::env::var_os("CARGO_BIN_EXE_pse-host")
                .expect("Cargo/nextest supplies the selected runtime host executable"),
        )
        .current_dir(root)
        .env_clear()
        .env("HOME", root)
        .env("PATH", "/usr/bin:/bin");
    cmd
}
fn successful(output: &Output) {
    assert!(
        output.status.success(),
        "status {:?}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}
fn value<'a>(output: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    output.split(|byte| *byte == 0).find_map(|record| {
        record
            .strip_prefix(key)
            .and_then(|tail| tail.strip_prefix(b"="))
    })
}
#[test]
fn print_explain_and_help_do_not_evaluate_personal_code() {
    let root = checkout();
    fs::write(
        root.path().join(".envrc.local"),
        "printf hit >> \"$HOME/counter\"\nexport API_TOKEN='never-render-this-secret'\n",
    )
    .unwrap();
    for args in [
        &["env", "--print"][..],
        &["env", "--explain", "--json"],
        &["env", "--help"],
    ] {
        let output = native(root.path()).args(args).output().unwrap();
        successful(&output);
        assert!(!String::from_utf8_lossy(&output.stdout).contains("never-render-this-secret"));
        assert!(!root.path().join("counter").exists());
    }
}
#[test]
fn activation_sources_once_and_keeps_private_derived_paths_as_data() {
    let root = checkout();
    fs::write(root.path().join(".envrc.local"), "printf hit >> \"$HOME/counter\"\nexport API_TOKEN='private-token'\nexport OTEL_EXPORTER_OTLP_ENDPOINT='https://collector.test/private'\nexport OTEL_EXPORTER_OTLP_HEADERS='authorization=private-header'\nexport XDG_STATE_HOME=\"$HOME/private-state\"\n").unwrap();
    let output = shell(
        root.path(),
        "eval \"$(\"$1\" env --print)\" || exit; /usr/bin/env -0",
    )
    .output()
    .unwrap();
    successful(&output);
    assert_eq!(fs::read(root.path().join("counter")).unwrap(), b"hit");
    assert_eq!(
        value(&output.stdout, b"API_TOKEN"),
        Some(&b"private-token"[..])
    );
    assert_eq!(
        value(&output.stdout, b"PSE_NATIVE_TELEMETRY_SETTINGS"),
        None
    );
    assert_eq!(
        value(&output.stdout, b"PSE_TELEMETRY_CONFIGURATION_STATUS"),
        Some(&b"configured"[..])
    );
    assert_eq!(
        value(&output.stdout, b"PSE_SURREAL_STATE"),
        Some(
            root.path()
                .join("private-state/pse-arrow/surreal-functional-v2")
                .as_os_str()
                .as_encoded_bytes()
        )
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private-token"));
}
#[test]
fn failed_personal_file_changes_no_parent_values() {
    let root = checkout();
    fs::write(
        root.path().join(".envrc.local"),
        "export API_TOKEN=partial-private\nreturn 23\n",
    )
    .unwrap();
    let output = shell(
        root.path(),
        "if eval \"$(\"$1\" env --print)\"; then exit 99; fi; /usr/bin/env -0",
    )
    .output()
    .unwrap();
    successful(&output);
    assert_eq!(value(&output.stdout, b"API_TOKEN"), None);
    assert_eq!(value(&output.stdout, b"PSE_PYTHON"), None);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("partial-private"));
}
#[test]
fn documentation_selection_is_explicit_and_does_not_provision() {
    let root = checkout();
    let output = shell(
        root.path(),
        "eval \"$(\"$1\" env --print --docs --cache off)\" || exit; /usr/bin/env -0",
    )
    .output()
    .unwrap();
    successful(&output);
    assert_eq!(
        value(&output.stdout, b"UV_PROJECT_ENVIRONMENT"),
        Some(
            root.path()
                .join(".venv-docs")
                .as_os_str()
                .as_encoded_bytes()
        )
    );
    assert_eq!(value(&output.stdout, b"RUSTC_WRAPPER"), Some(&b""[..]));
    assert!(!root.path().join(".venv-docs").exists());
    assert!(!root.path().join(".local").exists());
}
#[test]
fn activation_preserves_non_utf8_newline_and_empty_values() {
    let root = checkout();
    let mut local = b"export LOCAL_RAW='local-".to_vec();
    local.extend_from_slice(&[0xff, b'\n']);
    local.extend_from_slice(b"tail'\nexport SHARED=personal\nexport EMPTY=personal\n");
    fs::write(root.path().join(".envrc.local"), local).unwrap();
    let caller = OsString::from_vec(b"caller-\xfe\n=tail".to_vec());
    let output = shell(
        root.path(),
        "eval \"$(\"$1\" env --print)\" || exit; /usr/bin/env -0",
    )
    .env("SHARED", &caller)
    .env("EMPTY", "")
    .output()
    .unwrap();
    successful(&output);
    assert_eq!(
        value(&output.stdout, b"LOCAL_RAW"),
        Some(&b"local-\xff\ntail"[..])
    );
    assert_eq!(
        value(&output.stdout, b"SHARED"),
        Some(caller.as_encoded_bytes())
    );
    assert_eq!(value(&output.stdout, b"EMPTY"), Some(&b""[..]));
}
fn fake_producer(root: &Path, body: &str) -> String {
    let executable = root.join("fake-host");
    fs::write(&executable, format!("#!/bin/bash\n{body}\n")).unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    activation_script(&executable, root, false, CacheMode::Auto).unwrap()
}
#[test]
fn partial_failed_duplicate_and_readonly_transfers_are_rejected_before_application() {
    let root = checkout();
    for body in [
        "printf 'pse-activation-1\\0+FIRST=partial\\0'; exit 7",
        "printf 'pse-activation-1\\0+FIRST=partial\\0pse-activation-end'",
        "printf 'pse-activation-1\\0+FIRST=partial\\0pse-activation-end\\0\\0'; exit 7",
        "printf 'pse-activation-1\\0+FIRST=partial\\0+FIRST=other\\0pse-activation-end\\0\\0'",
        "printf 'pse-activation-1\\0+FIRST=partial\\0+LOCKED=changed\\0pse-activation-end\\0\\0'",
        "printf 'pse-activation-1\\0+FIRST=partial\\0+BAD-KEY=value\\0pse-activation-end\\0\\0'",
    ] {
        let script = fake_producer(root.path(), body);
        fs::write(root.path().join("activate"), script).unwrap();
        let output = shell(root.path(), "readonly LOCKED=initial; if source ./activate; then exit 99; fi; test -z \"${FIRST+x}\" && test \"$LOCKED\" = initial").output().unwrap();
        successful(&output);
    }
}
#[test]
fn activation_disables_xtrace_while_values_are_in_scope() {
    let root = checkout();
    fs::write(
        root.path().join(".envrc.local"),
        "export API_TOKEN='never-trace-this-token'\n",
    )
    .unwrap();
    let output = shell(
        root.path(),
        "set -x; eval \"$(\"$1\" env --print)\"; test \"$-\" != \"${-//x/}\"",
    )
    .output()
    .unwrap();
    successful(&output);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("never-trace-this-token"));
}
