// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use std::num::NonZeroUsize;

#[test]
fn project_reader_refuses_invalid_target_before_preparation() {
    use super::super::{ProjectRead, ProjectReadSelection, ReaderError, Request, encode};
    use pse_host::investigation::NativePath;
    use std::{io, path::PathBuf};

    let parent = tempfile::tempdir().unwrap();
    let absent = parent.path().join("no-configured-service");
    let cancel = pse_columnar::CancellationToken::new();
    for (path, timeout, invalid_selection) in [
        (PathBuf::from("relative-state"), None, true),
        (absent.clone(), Some(25_001), true),
        (absent.clone(), None, false),
    ] {
        let bytes = super::request(
            "exact-project-read",
            Request::Project {
                canonical: ProjectReadSelection {
                    state: NativePath::from(path),
                    database: None,
                    timeout_ms: timeout,
                },
                query: ProjectRead::Show { project: "p".into() },
            },
        );
        let refusal = encode(&bytes, &cancel).unwrap();
        let value: serde_json::Value = serde_json::from_slice(refusal.bytes()).unwrap();
        assert_eq!(value["correlation"], "exact-project-read");
        assert_eq!(value["payload"]["kind"], "refusal");
        assert_eq!(refusal.exit_code(), 2);
        assert!(matches!(
            refusal.delivery_failed(ReaderError::Io(io::Error::other("fixture delivery failed"))),
            ReaderError::Delivery { response, .. }
                if matches!(*response, ReaderError::Invalid(_)) && invalid_selection
                    || matches!(*response, ReaderError::Canonical(_)) && !invalid_selection
        ));
    }
    assert!(!absent.exists());
}

#[test]
fn native_retained_read_model_uses_explicit_pool_and_disabled_spill() {
    let model = super::super::canonical::model(NonZeroUsize::new(256 << 20).unwrap()).unwrap();
    assert!(
        matches!(model.factory().pool().memory_limit(),datafusion::execution::memory_pool::MemoryLimit::Finite(limit) if limit==256<<20)
    );
    assert!(
        !model
            .native_state()
            .runtime_env()
            .disk_manager
            .tmp_files_enabled()
    );
}
