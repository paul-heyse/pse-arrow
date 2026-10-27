# atomic-write-file (Rust)

Realizations considered: `atomic-write-file` 0.3.1 (released 2026-08-11; not in the lockfile;
depends on `nix`). Research date 2026-10-27. Sources: crate source 0.3.1 downloaded from
static.crates.io into scratch (`src/lib.rs`, `src/imp/unix/mod.rs`, `src/imp/unix/linux.rs`,
`Cargo.toml` features).

## Types

### atomic-write-file/replace-with-durable-commit
- **Need it serves:** replace a file's whole content atomically and durably (readers see old or
  new content, never partial), keeping existing metadata.
- **Recognizable code shapes:** write to `.<name>.tmp`, `fsync`, `rename` over the target,
  `fsync` the directory.
- **Mechanisms and APIs:** `AtomicWriteFile::options().open(path)` → write → `commit()`; on Linux
  the temporary is created relative to an opened directory fd (`openat`), optionally unnamed with
  `O_TMPFILE` (feature `unnamed-tmpfile`) and linked in through `/proc/self/fd/<n>` with
  `AT_SYMLINK_FOLLOW`; commit does `fdatasync`/`sync_all`, `renameat` over the target and `fsync`
  of the directory; drop without commit unlinks the temporary and fsyncs the directory.
  `OpenOptionsExt` controls metadata preservation.
- **Obligations discharged:** temporary naming, data sync, atomic replacement, directory sync,
  directory-fd-relative operation (immune to path components changing mid-operation), cleanup.
- **Residual caller obligations:** content verification, locking, any no-clobber requirement.
- **Semantic conditions:** commit always replaces. The crate documents that "`create_new()` is
  also not provided because there is no way to ensure that a file never exists from the time an
  `AtomicWriteFile` is opened to the time it is committed."
- **Version / build conditions:** 0.3.1; Linux path depends on `nix`.
- **Limits and negative results:** no no-clobber publication, no locks, no resumable staging.
- **Evidence:** Interface-checked: source 0.3.1 as listed.

## Current applicability

### [CBO-003](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-003) — Durable file publication and the sealed-file byte layer on `std::fs`
- **Types:** `atomic-write-file/replace-with-durable-commit`
- **Realization:** upgrade route (new dependency) 0.3.1; would add a direct `nix` dependency
  edge (the lockfile currently has `nix` 0.31.3 only transitively; the crate's required `nix`
  range was not checked against it).
- **Conclusion:** ruled out here — every assigned Rust publication site requires no-clobber
  semantics, which the crate deliberately does not provide.
- **What it would discharge / what remains local:** n/a.
- **Semantic gaps against the CBO's preservation constraints:** "No-clobber publication" is
  violated by design.
- **Unsettled premises and what would settle them:** none for this CBO; it could fit a
  replace-semantics metadata writer (the Python ledger files in CBO-047 are replace-semantics,
  but they are Python).
- **Evidence:** Interface-checked.
