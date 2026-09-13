// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonicalization throughput (blueprint §24.3).
//!
//! Phase-0 placeholder: `pse-ids` does not exist yet, so this measures `std`'s hasher over
//! a small slice. It exists to keep the bench target compiling and running in CI from the
//! first commit; the real group hashes canonical IPC bytes under the `pse.canon.v1`
//! contract (blueprint §5.3) and replaces this function wholesale.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "benchmark harness binary, same carve-out as xtask (clippy.toml)"
)]

use std::hash::{DefaultHasher, Hash, Hasher};
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

/// A stand-in for a canonical-IPC payload: 32 bytes, the size of one content hash.
const PAYLOAD: &[u8; 32] = b"pse.canon.v1 placeholder payload";

fn hash_small_slice(c: &mut Criterion) {
    c.bench_function("canonicalization/placeholder_hash_32b", |b| {
        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            black_box(PAYLOAD).hash(&mut hasher);
            black_box(hasher.finish())
        });
    });
}

criterion_group!(canonicalization, hash_small_slice);
criterion_main!(canonicalization);
