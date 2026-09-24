// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical Arrow transformation preflight refuses insufficient working memory.
mod support;
use pse_columnar::CanonicalizeOptions;
#[test]
fn canonicalization_fails_without_retaining_partial_allocations() {
    let registry = support::registry();
    let batch = support::batch(&registry, 2000, 0);
    let contract = pse_relations::canonical::contract(
        &registry,
        registry.relation("authored.samples").expect("relation"),
    )
    .expect("contract");
    let budget: std::sync::Arc<dyn pse_columnar::MemoryPool> =
        std::sync::Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 10));
    assert!(
        pse_columnar::canonicalize(&contract, &[batch], &budget, CanonicalizeOptions::default())
            .is_err()
    );
    assert_eq!(budget.reserved(), 0);
}
