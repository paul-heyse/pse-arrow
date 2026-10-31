# Opt-in cancellation branches

Source: the published `salsa` 0.28.5 crate archive, SHA-256
`27d2e6e19e39591cd99c01ac4141544b4d012cfd0831348629b8187fa9fe6d34`.
Its retained VCS metadata names `d434f8805c60ac60dce5f367ca91c5a7cd3e4c86`.
The original MIT and Apache licenses and published manifest are retained.

The local change adds `CancellationGroup` and `Storage::fork_cancellation_group`.
Only a deliberately forked branch inherits the monotonic group request through
ordinary storage clones. Every handle retains a separate existing cancellation
token, suppression bit, query stack and page cache. Local reset cannot clear a
group request; normal clones outside a branch keep their published behavior.
The group does not write to shared inputs or advance the database revision.

Changed upstream files: `src/lib.rs`, `src/storage.rs`, `src/zalsa_local.rs`
and `tests/parallel/main.rs` (registration only).
Added: `src/cancellation_group.rs`, `src/storage/cancellation_group_tests.rs`,
`tests/parallel/cancellation_group.rs`
and this provenance note. The focused tests
registered by `storage.rs` cover ordinary clone isolation, nested descendants, independent
groups, handle-local suppression, reset/drop behavior and shared input storage.
The parallel integration case uses entered barriers inside real tracked queries
on nested Rayon database clones, then joins both canceled computations and
recomputes the same memo from an independent request.

Reproduce the focused library test from the checkout root with the pinned
toolchain:

```sh
cargo test --manifest-path vendor/salsa/Cargo.toml --locked --lib cancellation_group_tests
cargo test --manifest-path vendor/salsa/Cargo.toml --locked --test parallel cancellation_group
```

Compare to the archive at
`https://static.crates.io/crates/salsa/salsa-0.28.5.crate` when updating or
replacing the patch. No upstream contribution or acceptance is claimed.
