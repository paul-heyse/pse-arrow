# cacache

Realizations considered: upgrade route only. `cacache` 13.1.0 (released 2024-11-26; newest
release on 2026-10-27) is not in the workspace lockfile. Research date 2026-10-27.
Sources: docs.rs crate root page for 13.1.0 (function list, module list, crate description),
crates.io metadata.

## Types

### cacache/content-addressed-disk-store
- **Need it serves:** a local on-disk cache whose entries are addressed by content hash with
  integrity verification on read, usable from several processes.
- **Recognizable code shapes:** a hand-written cache directory with its own magic headers,
  checksums, temp-file publish and directory scan.
- **Mechanisms and APIs:** `write`/`write_sync` (key → content, returns SRI `Integrity`),
  `write_hash*` (content only), `read`/`read_sync`/`read_hash*` (verified; `Reader::check()`),
  `metadata*`, `exists*`, `copy*`/`hard_link*`/`reflink*` (materialise content to a path),
  `remove*`, `remove_hash*`, `clear*`, `list_sync`, raw `index` module; async by default, `_sync`
  variants; `*_with_algo` selects the hash algorithm.
- **Obligations discharged:** content addressing, integrity verification, commit-based writes
  ("data is only committed to the cache after you do `fd.commit()`"), per-entry metadata,
  materialisation by hard link or reflink.
- **Residual caller obligations:** size and entry bounds, eviction policy and scan limits (no
  size-limit, LRU or garbage-collection API is documented on the 13.1.0 crate page), cache
  location and permissions, key derivation, reservation before writing.
- **Semantic conditions:** integrity is the content hash of stored bytes, independent of any
  application header; the index is append-oriented; the docs page does not use the word "atomic"
  for publication.
- **Version / build conditions:** 13.1.0; async API pulls an async runtime feature set (sync API
  available).
- **Limits and negative results:** no bounded-cache or eviction facility at 13.1.0 (crate-root
  page only; module-level docs not read).
- **Evidence:** Interface-checked (docs.rs crate root).

## Current applicability

### [CBO-022](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-022) — Disk layer for compiled native evaluators
- **Types:** cacache/content-addressed-disk-store.
- **Realization:** upgrade route (new dependency `=13.1.0`).
- **Conclusion:** fits with conditions — it could own content addressing, verified reads and
  commit/materialise; the bounding (entries, bytes, scan), eviction and fresh-process reuse
  policy that make up much of the local cache would remain.
- **What it would discharge / what remains local:** discharge: checksum framing, verified read,
  staged publish; remain: `MAX_ENTRIES`/`MAX_BYTES`/`MAX_SCAN` policy and eviction, key derivation
  from source + compiler + flags, execution-scope checks, private copy semantics (the local cache
  hands out private copies that survive eviction).
- **Prerequisites, configuration or extension:** a sync-only use; an eviction pass over
  `list_sync` + `metadata` + `remove`.
- **Semantic gaps against the CBO's preservation constraints:** "compiled bytes carry headers and
  checksums" is replaced by SRI integrity (equivalent role); bounded scans need local code.
- **Unsettled premises and what would settle them:** whether cacache's index growth and
  concurrent-writer behaviour fit a bounded temp-directory cache — reading its `index` module and
  tests would settle it.
- **Evidence:** Interface-checked; Proposed.
