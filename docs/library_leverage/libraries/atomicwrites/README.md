# atomicwrites and stdlib atomic publication (Python)

Realizations considered: `atomicwrites` 1.4.1 (last release 2022-07-08; upstream marked
unmaintained); Python 3.14 stdlib (`os.open(O_EXCL|O_NOFOLLOW|O_CLOEXEC)`, `os.fsync`,
`os.replace`, `os.link`, `hashlib.file_digest`). Research date 2026-10-27. Sources: sdist
`atomicwrites-1.4.1/atomicwrites/__init__.py` in scratch; repository usages read in
`scripts/resource_store.py` (`publish`) and `scripts/telemetry_service.py` (`_write`) at baseline.

## Types

### atomicwrites/durable-replace-or-link
- **Need it serves:** write a temporary next to the target, fsync it, then either replace the
  target or publish only if absent, then fsync the directory.
- **Recognizable code shapes:** `temporary = path.with_name(f".{name}.{uuid}.tmp")`, `os.open(...
  O_EXCL ...)`, `os.fsync`, `temporary.replace(path)`, optional `sync_directory(path.parent)`,
  `finally: temporary.unlink(missing_ok=True)`.
- **Mechanisms and APIs:** `atomic_write(path, overwrite=False|True, mode=…)` context manager;
  `replace_atomic(src, dst)` (`os.rename` + directory fsync on POSIX), `move_atomic(src, dst)`
  (`os.link` + unlink, no-clobber) with directory fsync; `AtomicWriter` subclass hooks for
  custom temp creation.
- **Obligations discharged:** temp creation, data fsync, rename/link, directory fsync, cleanup.
- **Residual caller obligations:** `O_NOFOLLOW`, private mode and ownership checks, protected
  parent-directory chain, fsync of newly created ancestors.
- **Semantic conditions:** no-clobber via `link`+`unlink` (two names briefly); no
  `renameat2(NOREPLACE)` (the stdlib has no binding for it).
- **Version / build conditions:** unmaintained since 2022; pure Python.
- **Limits and negative results:** no `O_NOFOLLOW`, no permission enforcement, no locking.
- **Evidence:** Interface-checked: atomicwrites 1.4.1 source.

## Current applicability

### [CBO-047](../../../library_leverage/codebase/developer-tooling.md#cbo-047) — File-ledger primitives re-derived per script
- **Types:** `atomicwrites/durable-replace-or-link`
- **Realization:** upgrade route 1.4.1 (new, unmaintained dependency).
- **Conclusion:** ruled out here — the stdlib already composes the same steps in a few lines, the
  repository's stricter checks (`O_NOFOLLOW`, `0600`, protected ancestors, ancestor fsync) are
  outside the library, and an unmaintained dependency on stdlib-only paths is not acceptable.
  The real gap is uneven local discipline: observed at baseline, `resource_store.publish` fsyncs
  the directory (and ancestors) while `telemetry_service._write` does not.
- **What it would discharge / what remains local:** nothing beyond what a shared in-repo helper
  would; all policy remains.
- **Semantic gaps against the CBO's preservation constraints:** missing `O_NOFOLLOW`/`0600`.
- **Unsettled premises and what would settle them:** none.
- **Evidence:** Interface-checked; Proposed (negative). Digest helpers: `hashlib.file_digest`
  (3.11+) and `hashlib.sha256(json.dumps(..., sort_keys=True).encode())` are already stdlib
  one-liners; no library adds leverage for the 18–22 digest helper copies (count depends on
  the name pattern; 22 `def *digest*` in non-test scripts by rg at baseline).

**Reviewer correction (2026-10-27).** Confirmed ruled out. Python 3.14.8 exposes no `renameat2` or `RENAME_*` flags (it has `os.O_TMPFILE` and `os.pidfd_open`), so stdlib no-clobber publication is link plus unlink, or `ctypes`. The uneven discipline is wider than sampled: `host_admission.metadata` defaults to `durable=False` through `operation_lifetime.write_json`, which uses a PID-suffixed `O_TRUNC` temp without fsync or cleanup, and 22 zero-length `allocations.<pid>.tmp` files from 2026-10-10 remain. The comparative review recommends extending `scripts/host_metadata.py` into one stdlib module with private replace-publish at an explicit durability level, `O_EXCL` random temps, `finally` cleanup and stale-temp removal by the lock holder. See the [comparative review](../../reviews/host-processes-and-durable-files.md#cbo-047).
