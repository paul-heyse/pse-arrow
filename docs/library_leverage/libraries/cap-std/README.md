# cap-std family: cap-std, cap-fs-ext, cap-tempfile (Rust)

Realizations considered: `cap-std`/`cap-fs-ext`/`cap-tempfile`/`cap-primitives` 4.0.3 (released
2026-08-20; none in the lockfile). Research date 2026-10-27. Sources: crate sources 4.0.3 downloaded
into scratch: `cap-tempfile/src/tempfile.rs`, `cap-fs-ext/src/*.rs`,
`cap-primitives/src/fs/` (module list: `hard_link.rs`, `rename.rs`, `open.rs`,
`follow_symlinks.rs`, `via_parent/`, …). `cap-std` itself was inspected only through
`cap-primitives`, which implements its operations.

## Types

### cap-std/directory-confined-operations
- **Need it serves:** perform all file operations relative to an opened directory handle so that
  no path (via `..`, absolute paths or symlinks) can escape it, and refuse symlinks where needed.
- **Recognizable code shapes:** hand-written `no_symlinks(path)` walks over every ancestor;
  `O_NOFOLLOW` on the final component only; `symlink_metadata` checks before open; checks that a
  directory is owned by the current uid.
- **Mechanisms and APIs:** `cap_std::fs::Dir::open_ambient_dir`, then `Dir::open_with`,
  `create_dir`, `hard_link`, `rename`, `remove_file`, `metadata`; `cap_fs_ext::OpenOptionsFollowExt::
  follow(FollowSymlinks::No)`; `OpenOptionsSyncExt::{sync, dsync}` (`O_SYNC`/`O_DSYNC`);
  `DirExt`, `MetadataExt`; on Linux the primitives use `openat2(RESOLVE_BENEATH)` where available
  and fall back to manual component resolution.
- **Obligations discharged:** sandboxed path resolution, including intermediate-symlink handling,
  which is the hard part of "never traverse a symlink below this root".
- **Residual caller obligations:** ownership/mode checks, fsync ordering, locks, link-count
  checks, digests.
- **Semantic conditions:** symlinks that stay inside the root are allowed unless the caller opts
  out per open; this is a sandbox ("stay beneath"), not a "no symlink anywhere" policy.
- **Limits and negative results:** no locking API in cap-fs-ext 4.0.3 (trait list: `DirEntryExt`,
  `DirExt`, `FileTypeExt`, `IsFileReadWrite`, `MetadataExt`, `OpenOptionsFollowExt`,
  `OpenOptionsMaybeDirExt`, `OpenOptionsSyncExt`, `Reopen`); no no-clobber rename.
- **Evidence:** Interface-checked: sources listed.

### cap-std/capability-tempfile-replace
- **Need it serves:** write a file inside a directory capability and atomically move it into place.
- **Mechanisms and APIs:** `cap_tempfile::TempFile::new(&dir)` (anonymous `O_TMPFILE` on Linux,
  named fallback), `TempFile::replace(name)` (link the anonymous inode to a random name through
  `/proc/self/fd`, then `Dir::rename` over the destination), `TempFile::new_anonymous`.
- **Semantic conditions:** `replace` overwrites the destination; there is no no-clobber variant.
- **Limits and negative results:** no fsync of data or directory performed by `replace`
  (search scope: `src/tempfile.rs` for `sync`).
- **Evidence:** Interface-checked: `cap-tempfile-4.0.3/src/tempfile.rs`.

## Current applicability

### [CBO-003](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-003) — Durable file publication and the sealed-file byte layer on `std::fs`
- **Types:** `cap-std/directory-confined-operations`, `cap-std/capability-tempfile-replace`
- **Realization:** upgrade route 4.0.3 (new dependency family; `cap-primitives` 4.0.3 pulls
  `ambient-authority`, `fs-set-times`, `io-extras`, `io-lifetimes` and `rustix`; whether its
  `rustix` requirement unifies with the 1.1.5 pin was not checked).
- **Conclusion:** unsettled for the sealed-file store's symlink/alias guards; ruled out for
  publication itself (no no-clobber, no durability).
- **What it would discharge / what remains local:** the hand-written "no symlinks on the path" and
  copy-boundary checks could become directory-handle-relative operations beneath the store root.
  Hard-link rejection (`nlink == 1`), private ownership and the fsync order stay local.
- **Prerequisites, configuration or extension:** holding an open `Dir` for the store root for the
  store's lifetime; using `FollowSymlinks::No` on final components.
- **Semantic gaps against the CBO's preservation constraints:** the store's rule appears to be
  "no symlink at all on the protected path"; cap-std's default is "nothing escapes the root",
  which tolerates internal symlinks unless every open opts out. Not a gap if every operation
  opts out, but the guarantee is then local again.
- **Unsettled premises and what would settle them:** whether the store's symlink rule is about
  escape (cap-std fits) or about any alias (local check remains) — design owner; a reading of the
  offline copy-boundary guard was not done in this pass.
- **Evidence:** Interface-checked; application Proposed.
