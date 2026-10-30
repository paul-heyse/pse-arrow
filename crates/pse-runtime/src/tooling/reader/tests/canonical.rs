// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use std::num::NonZeroUsize;

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
