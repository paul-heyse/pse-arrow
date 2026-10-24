// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One pure finite-input target; its wrapper owns campaign lifetime and corpus.
#![no_main]
use pse_runtime::tooling::frozen_inspection::{FrozenEvidenceSet, decode_document};
libfuzzer_sys::fuzz_target!(|bytes: &[u8]| {
    if let Ok(set) = decode_document::<FrozenEvidenceSet, 3>(bytes) {
        // Invalid identities/member bounds remain refusals, never repaired by the harness.
        let _ = set.validate();
    }
});
