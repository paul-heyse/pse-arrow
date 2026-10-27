# tempfile (Rust)

Realizations considered: installed `tempfile` 3.27.0 (workspace pin `=3.27.0`, depends on `rustix`
^1.1.4 with `fs`); no newer release examined. Research date 2026-10-27. Sources: installed source
`tempfile-3.27.0/src/file/mod.rs` (`NamedTempFile::persist`, `persist_noclobber`, `TempPath`,
`disable_cleanup`), `src/file/imp/unix.rs` (`create` with `O_TMPFILE`, `persist`), `src/lib.rs`
(`Builder::permissions`, `Builder::disable_cleanup`). Context7 not consulted separately; the
installed source is the realization.

## Types

### tempfile/named-staging-noclobber-persist
- **Need it serves:** stage bytes under a temporary name in the destination directory and give
  them a final name only if that name does not exist, removing the staging name automatically on
  failure or drop.
- **Recognizable code shapes:** `create_new` of a `*.pending`/`*.tmp` sibling, write, then
  `hard_link(staging, final)` + `remove_file(staging)`; `Drop` impls that delete a pending path;
  comments saying "rename would replace an existing file".
- **Mechanisms and APIs:** `NamedTempFile::new_in(dir)` / `Builder::new().prefix(..).permissions(..)
  .tempfile_in(dir)` creates a randomly named file with `O_EXCL`; `NamedTempFile::persist_noclobber(path)`
  on Linux first tries `renameat2(.., RENAME_NOREPLACE)` (via `rustix::fs::renameat_with(..,
  RenameFlags::NOREPLACE)`), and falls back to `hard_link` + best-effort `unlink` only on
  `ENOSYS`/`EINVAL`; `TempPath` keeps the path-owning RAII guard when the file handle is closed
  early; `disable_cleanup(true)`/`keep()` turns RAII deletion off. `PersistError` returns the
  still-owned temporary on failure so the caller can retry or inspect.
- **Obligations discharged:** unique staging-name selection; no-clobber publication with a single
  atomic rename on filesystems supporting `RENAME_NOREPLACE` (ext4, xfs, btrfs, tmpfs); removal of
  the staging name on error/drop/unwind (not on SIGKILL or crash); mode at creation through
  `Builder::permissions`.
- **Residual caller obligations:** `file.sync_all()` before persist; fsync of the destination
  directory after persist (and of the staging directory when it differs); any content verification;
  cleanup of staging names left by crashes (random names make them harder to recognize than a
  deterministic pending name); single-link checks; symlink refusal on directory components;
  cancellation/byte-limit checkpoints during the write.
- **Semantic conditions:** staging and destination must be on the same filesystem; staging names
  are random, so a crashed writer cannot resume a partial staging file by name; on the fallback
  path the operation is not atomic and can leave the staging link (documented: "This method is not
  atomic. It can leave the original link to the temporary file behind"); `RENAME_NOREPLACE`
  fails with `EEXIST` when any entry, including a symlink, already holds the name, and does not
  follow a symlink at the destination; persist never fsyncs.
- **Version / build conditions:** `RENAME_NOREPLACE` route needs Linux ≥ 3.15 and filesystem
  support; behaviour above is from 3.27.0.
- **Limits and negative results:** no directory fsync, no data fsync, no anonymous (`O_TMPFILE`)
  staging for *named* persistence (`tempfile()`/`tempfile_in()` use `O_TMPFILE` but return an
  unnamed `File` with no link-into-place API), no locking, no digest verification. Search scope:
  whole `src/` of 3.27.0 for `sync`, `fsync`, `linkat`, `TMPFILE`.
- **Evidence:** Interface-checked: installed source 3.27.0 files listed above.

## Current applicability

### [CBO-003](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-003) — Durable file publication and the sealed-file byte layer on `std::fs`
- **Types:** `tempfile/named-staging-noclobber-persist`
- **Realization:** installed pin 3.27.0; no dependency move.
- **Conclusion:** fits with conditions for the simple export/capture/pack publishers; ruled out
  here as the whole canonical sealed-file write path.
- **What it would discharge / what remains local:** discharges staging naming, RAII removal of
  abandoned staging on error or cancellation-drop, and an atomic no-clobber rename. Data fsync,
  directory fsync ordering (data → name → directory → acknowledgement), single-link and alias
  checks, and the byte-limit/cancellation checkpoints remain local.
- **Prerequisites, configuration or extension:** `Builder::permissions(0o600)`; `new_in(dest_dir)`
  to keep the same filesystem; a small local "sync then persist then sync dir" helper around it.
- **Semantic gaps against the CBO's preservation constraints:** the canonical store's staging is
  deliberately deterministic (`<instance>.pending`) and resumable: a re-entry reopens an existing
  staging file, compares the already-written prefix and refuses `OperationReused` on mismatch.
  Random tempfile names erase that resume/idempotence check. Its publication also expects the
  staging inode to keep a second link until `remove_file` and then asserts one link; a rename
  changes that sequence (simpler, but the assertion and recovery reasoning must be re-derived).
  Observed at baseline: not every publisher fsyncs the same way (the export path fsyncs data but
  not the directory; the capture acknowledgement publisher writes, flushes and links without
  `sync_all`), so "the same sequence" in the CBO is not uniform.
- **Unsettled premises and what would settle them:** whether deterministic resumable staging is a
  required semantic of the sealed-file store or an incidental choice (design owner decision);
  whether the supported filesystems all honour `RENAME_NOREPLACE` (the current code comment
  restricts itself to "the supported local Linux filesystem").
- **Evidence:** Interface-checked (tempfile source); application Proposed.

**Reviewer correction (2026-10-27).** Narrow "fits with conditions for the simple export/capture/pack publishers": tempfile is ruled out for export, workbench and review pack. Their `.incomplete` staging is an architecture contract (interrupted staging remains `.incomplete`), and the review pack's `fail_at` publishes partial prefixes from it; RAII deletion and random names contradict both. tempfile fits only the diagnostic publishers, which would leave two publication disciplines. `xtask/src/producer_history.rs` already uses `persist_noclobber`. The comparative review recommends one `renameat_with(NOREPLACE)` helper instead (see `rustix`). See the [comparative review](../../reviews/host-processes-and-durable-files.md#cbo-003).
