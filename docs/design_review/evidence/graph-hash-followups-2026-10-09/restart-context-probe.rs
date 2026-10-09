// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Temporary benchmark diagnostic, removed from production after observation.
fn observed_admission() -> (ReplayAdmission, Value) {
    let admission = local_admission();
    let observation = pse_buildinfo::with_local_runtime_scope(|| {
        pse_buildinfo::LocalRuntimeObservation::capture(
            pse_buildinfo::LocalRuntimeRole::Worker,
            local_anchor as *const () as usize,
        )
    })
    .unwrap();
    let mut reconstructed = FramedHasher::new(Frame::BuildInputsV1);
    reconstructed
        .str("pse.local-runtime-replay.v2")
        .hash(&observation.identity().unwrap())
        .part(&[]);
    assert_eq!(
        reconstructed.finish_hash(),
        admission.identity(),
        "diagnostic capture differs from actual admission; no retry or forced hit"
    );
    (admission, serde_json::to_value(observation).unwrap())
}

