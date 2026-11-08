// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Compile and execute real opted-in generated view decoders. This fixture uses
//! only published dependencies and the sibling maintained runtime; no shared
//! target directory or source-tree-generated output is modified.
use buffa::Message;
use buffa_codegen::{CodeGenConfig, generated::descriptor::FileDescriptorSet};
use std::{fs, path::Path, process::Command};

#[test]
fn allocation_accounted_generated_decoders_compile_and_run() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("outside.proto"), r#"
        syntax = "proto2";
        package outside;
        message Unaccounted { repeated uint64 values = 1 [packed=true]; }
    "#).unwrap();
    fs::write(dir.path().join("ordinary.proto"), r#"
        syntax = "proto2";
        package ordinary;
        message Legacy { repeated uint64 values = 1 [packed=true]; }
    "#).unwrap();
    fs::write(dir.path().join("accounted.proto"), r#"
        syntax = "proto2";
        package accounted;
        message Prepared { repeated uint64 values = 1 [packed=true]; }
    "#).unwrap();
    fs::write(dir.path().join("allocation.proto"), r#"
        syntax = "proto2";
        package allocation;
        import "outside.proto";
        import "ordinary.proto";
        import "accounted.proto";
        message ImportedRoot {
            optional ordinary.Legacy singular = 1;
            repeated ordinary.Legacy repeated = 2;
            map<string, ordinary.Legacy> entries = 3;
            oneof pick { ordinary.Legacy message = 4; bool flag = 5; }
        }
        message AccountedRoot {
            optional accounted.Prepared singular = 1;
            repeated accounted.Prepared repeated = 2;
            map<string, accounted.Prepared> entries = 3;
            oneof pick { accounted.Prepared message = 4; bool flag = 5; }
        }

        message ExternalRoot {
            optional outside.Unaccounted singular = 1;
            repeated outside.Unaccounted repeated = 2;
            map<string, outside.Unaccounted> entries = 3;
            oneof pick { outside.Unaccounted message = 4; bool flag = 5; }
        }

        message Child { optional int32 value = 1; repeated uint64 values = 2 [packed=true]; }
        message Root {
            optional Child singular = 1;
            repeated Child children = 2;
            repeated uint64 numbers = 3 [packed=true];
            map<string, Child> entries = 4;
            oneof pick { Child left = 5; Child right = 6; }
            optional group Group = 7 { optional int32 value = 1; }
            repeated State states = 8 [packed=true];
            repeated double doubles = 9 [packed=true];
        }
        enum State { ZERO = 0; ONE = 1; }
    "#).unwrap();
    let descriptor = dir.path().join("descriptor.bin");
    let output = Command::new(std::env::var("PROTOC").unwrap_or_else(|_| "protoc".into()))
        .current_dir(dir.path()).args(["--include_imports", "--descriptor_set_out=descriptor.bin", "allocation.proto"])
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let fds = FileDescriptorSet::decode_from_slice(&fs::read(descriptor).unwrap()).unwrap();
    let mut config = CodeGenConfig::default();
    config.extern_paths.push((".outside.Unaccounted".into(), "crate::external::Unaccounted".into()));
    config.extern_paths.push((".accounted.Prepared".into(), "crate::accounted::Prepared".into()));
    config.lazy_views = true;
    config.emit_register_fn = false;
    let standard = buffa_codegen::generate(&fds.file, &["allocation.proto".into()], &config).unwrap();
    assert_published_default_output(dir.path(), &standard, &config);
    assert!(!config.allocation_accounted_views);
    config.allocation_accounted_views = true;
    let accounted = buffa_codegen::generate(&fds.file, &["allocation.proto".into()], &config).unwrap();
    config.allocation_accounted_views = false;
    let standard_again = buffa_codegen::generate(&fds.file, &["allocation.proto".into()], &config).unwrap();
    assert_eq!(standard.iter().map(|f| (&f.name, &f.content)).collect::<Vec<_>>(),
        standard_again.iter().map(|f| (&f.name, &f.content)).collect::<Vec<_>>());
    for file in accounted { fs::write(dir.path().join("src").join(file.name), file.content).unwrap(); }
    let mut external_config = CodeGenConfig::default();
    external_config.emit_register_fn = false;
    external_config.lazy_views = true;
    for file in buffa_codegen::generate(&fds.file, &["outside.proto".into()], &external_config).unwrap() {
        fs::write(dir.path().join("src").join(file.name), file.content).unwrap();
    }
    for file in buffa_codegen::generate(&fds.file, &["ordinary.proto".into()], &external_config).unwrap() {
        fs::write(dir.path().join("src").join(file.name), file.content).unwrap();
    }
    external_config.allocation_accounted_views = true;
    for file in buffa_codegen::generate(&fds.file, &["accounted.proto".into()], &external_config).unwrap() {
        fs::write(dir.path().join("src").join(file.name), file.content).unwrap();
    }
    fs::write(dir.path().join("src/lib.rs"), concat!(
        "pub mod allocation { include!(\"allocation.mod.rs\"); }\n",
        "pub mod ordinary { include!(\"ordinary.mod.rs\"); }\n",
        "pub mod accounted { include!(\"accounted.mod.rs\"); }\n",
        "pub mod external { include!(\"outside.mod.rs\"); }\n",
        include_str!("allocation_fixture.rs.inc")
    )).unwrap();
    let runtime = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().join("vendor/buffa").canonicalize().unwrap();
    fs::write(dir.path().join("Cargo.toml"), format!(r#"
        [package]
        name = "allocation-generated-controls"
        version = "0.0.0"
        edition = "2021"
        [dependencies]
        buffa = {{ version = "=0.9.2", path = {:?} }}
        [workspace]
    "#, runtime)).unwrap();
    // Reuse the exact pinned checkout graph as the resolver's starting point.
    let root_lock = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().join("Cargo.lock");
    fs::copy(root_lock, dir.path().join("Cargo.lock")).unwrap();
    fs::copy(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().join("rust-toolchain.toml"),
        dir.path().join("rust-toolchain.toml")).unwrap();
    let output = Command::new("cargo").current_dir(dir.path())
        .args(["test", "--offline", "--lib"]).output().unwrap();
    assert!(output.status.success(), "generated decoder controls:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("7 passed; 0 failed; 0 ignored"),
        "generated controls must all execute: {}", String::from_utf8_lossy(&output.stdout));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("warning:"),
        "generated fixture compilation must have zero warnings: {}", String::from_utf8_lossy(&output.stderr));
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
}

/// Compare default generation with the actual published 0.9.2 generator, rather
/// than asserting only that the new flag can be toggled back to its default.
fn assert_published_default_output(
    dir: &Path,
    standard: &[buffa_codegen::GeneratedFile],
    config: &CodeGenConfig,
) {
    use std::collections::BTreeSet;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let generator = root.join("tests/fixtures/buffa-stock");
    let output_dir = dir.join("published-output");
    fs::create_dir(&output_dir).unwrap();
    let lock = fs::read_to_string(generator.join("Cargo.lock")).unwrap();
    let entries: Vec<_> = lock
        .split("\n[[package]]\n")
        .filter(|entry| entry.lines().any(|line| line == r#"name = "buffa-codegen""#))
        .collect();
    assert_eq!(entries.len(), 1, "exactly one stock generator");
    for line in [
        r#"version = "0.9.2""#,
        r#"source = "registry+https://github.com/rust-lang/crates.io-index""#,
        r#"checksum = "bb4bea10ef85ea3d06a20412c67cb612527feb25f33367eab850139441fb5dd9""#,
    ] {
        assert!(entries[0].lines().any(|entry| entry == line), "stock generator lock lacks {line}");
    }
    let output = Command::new("cargo").current_dir(&generator)
        .args(["run", "--offline", "--locked", "--"])
        .arg(dir.join("descriptor.bin"))
        .arg(&output_dir)
        .arg("allocation.proto")
        .arg(config.lazy_views.to_string())
        .arg(config.emit_register_fn.to_string())
        .args(config.extern_paths.iter().flat_map(|(source, target)| [source, target]))
        .output().unwrap();
    assert!(output.status.success(), "published generator control:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    let actual: BTreeSet<_> = fs::read_dir(&output_dir).unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    let expected: BTreeSet<_> = standard.iter().map(|file| file.name.clone()).collect();
    assert_eq!(actual, expected, "default output population changed");
    for file in standard {
        assert_eq!(file.content, fs::read_to_string(output_dir.join(&file.name)).unwrap(),
            "default output changed: {}", file.name);
    }
}
