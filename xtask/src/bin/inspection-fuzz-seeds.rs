// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual process-local diagnostic seed, with independently selected meaning.
use pse_runtime::tooling::frozen_inspection::{FrozenEvidenceSet,decode_document,MAXIMUM_DOCUMENT_BYTES};
use pse_diagnostics::observation::{ObservationHub,Policy,Outcome};
fn main()->Result<(),Box<dyn std::error::Error>> {
    let directory=std::path::PathBuf::from(std::env::args().nth(1).ok_or("seed directory required")?);
    std::fs::create_dir_all(&directory)?;
    let hub=ObservationHub::new(Policy::default());
    let operation=hub.start("fuzz.actual-operation");
    let reader=operation.reader();
    operation.context().count("calls","count",7);
    operation.finish(Outcome::Completed);
    let set=FrozenEvidenceSet {operations:vec![reader.snapshot(std::time::Duration::from_secs(1))],..Default::default()};
    let bytes=serde_json::to_vec(&set)?;
    let decoded=decode_document::<FrozenEvidenceSet,3>(&bytes)?;
    if decoded.operations.len()!=1 || decoded.operations[0].kind!="fuzz.actual-operation"
        || decoded.operations[0].outcome!=Some(Outcome::Completed) {return Err("authentic seed meaning was not retained".into());}
    decoded.validate()?;
    std::fs::write(directory.join("actual-operation.json"),&bytes)?;
    std::fs::write(directory.join("unsupported-version.json"),br#"{"version":999,"checkpoints":[]}"#)?;
    std::fs::write(directory.join("malformed.json"),br#"{"version":3,"checkpoints":[{"process":null}]}"#)?;
    let oversized=vec![b' ';MAXIMUM_DOCUMENT_BYTES+1];
    if decode_document::<FrozenEvidenceSet,3>(&oversized).is_ok() {return Err("oversized input was admitted".into());}
    if decode_document::<FrozenEvidenceSet,3>(br#"{"version":999,"checkpoints":[]}"#).is_ok() {return Err("unsupported version was admitted".into());}
    Ok(())
}
