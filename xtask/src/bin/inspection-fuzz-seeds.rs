// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual process-local diagnostic seed, with independently selected meaning.
use pse_diagnostics::observation::{ObservationHub, Outcome, Policy};
use pse_runtime::tooling::frozen_inspection::{
    FrozenEvidenceSet, MAXIMUM_DOCUMENT_BYTES, decode_document,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory =
        std::path::PathBuf::from(std::env::args().nth(1).ok_or("seed directory required")?);
    std::fs::create_dir_all(&directory)?;
    let hub = ObservationHub::new(Policy::default());
    let operation = hub.start("fuzz.actual-operation");
    let reader = operation.reader();
    operation.context().count("calls", "count", 7);
    operation.finish(Outcome::Completed);
    let set = FrozenEvidenceSet {
        operations: vec![reader.snapshot(std::time::Duration::from_secs(1))],
        ..Default::default()
    };
    let bytes = serde_json::to_vec(&set)?;
    let decoded = decode_document::<FrozenEvidenceSet, 3>(&bytes)?;
    if decoded.operations.len() != 1
        || decoded.operations[0].kind != "fuzz.actual-operation"
        || decoded.operations[0].outcome != Some(Outcome::Completed)
    {
        return Err("authentic seed meaning was not retained".into());
    }
    decoded.validate()?;
    std::fs::write(directory.join("actual-operation.json"), &bytes)?;
    std::fs::write(
        directory.join("unsupported-version.json"),
        br#"{"version":999,"checkpoints":[]}"#,
    )?;
    std::fs::write(
        directory.join("malformed.json"),
        br#"{"version":3,"checkpoints":[{"process":null}]}"#,
    )?;
    let oversized = vec![b' '; MAXIMUM_DOCUMENT_BYTES + 1];
    let size_error = decode_document::<FrozenEvidenceSet, 3>(&oversized)
        .err()
        .ok_or("oversized input was admitted")?
        .to_string();
    if !size_error.contains("exceeds 64 MiB") {
        return Err("oversized input did not reach its size guard".into());
    }
    let version_error =
        decode_document::<FrozenEvidenceSet, 3>(br#"{"version":999,"checkpoints":[]}"#)
            .err()
            .ok_or("unsupported version was admitted")?
            .to_string();
    if !version_error.contains("unsupported") {
        return Err("version control did not reach version refusal".into());
    }
    if decode_document::<FrozenEvidenceSet, 3>(br#"{"version":3,"checkpoints":[{"process":null}]}"#)
        .is_ok()
    {
        return Err("malformed checkpoint was admitted".into());
    }
    std::fs::write(
        directory
            .parent()
            .ok_or("seed directory needs a parent")?
            .join("seed-controls.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema":"inspection-fuzz-seeds-v1",
            "authentic_operation":{"kind":decoded.operations[0].kind,"outcome":decoded.operations[0].outcome},
            "oversized":{"bytes":oversized.len(),"error":size_error},
            "unsupported_version":{"version":999,"error":version_error},
            "malformed_checkpoint":"refused"
        }))?,
    )?;
    Ok(())
}
