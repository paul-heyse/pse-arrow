# SQLite (Python `sqlite3`, optionally `sqlite-utils`)

Realizations considered:
- Installed: Python standard-library `sqlite3` with SQLite 3.53.1 (the `.venv` interpreter,
  CPython 3.14.8).
- Upgrade route: `sqlite-utils` 4.2.1 (released 2026-08-13), a convenience layer that is not
  needed for the capability.

Research date 2026-10-27. Sources:
- Local interpreter: `sqlite3.sqlite_version`.
- PyPI JSON for `sqlite-utils`.
- SQLite feature knowledge (FTS5, JSON functions, `STRICT` tables, `WITHOUT ROWID`, atomic
  file replacement through `VACUUM INTO` or the backup API). These were not re-read for
  3.53.1.

## Types

### sqlite/rebuildable-local-derived-index
- **Need it serves:** A disposable, file-based, service-free index that is rebuilt from
  authoritative source files and queried with SQL by tools and agents offline.
- **Recognizable code shapes:**
  - Development or knowledge indexes held in a server database that needs a running
    service, credentials and lifecycle supervision, for data that is entirely derived from
    files in the repository.
  - Hand-written in-memory query helpers over JSON projections.
- **Mechanisms and APIs:**
  - `sqlite3.connect(path)`, `executemany` and transactions.
  - `STRICT` tables for typed columns.
  - FTS5 virtual tables for text search.
  - The `json_each` and `json_extract` functions.
  - Build to a temporary file, then `os.replace` it into place for atomic swap.
  - The `PRAGMA user_version` or a metadata table records the source cut and digest.
  - `sqlite3` CLI or Python for offline inspection.
  - `sqlite-utils` adds table creation from dicts, upserts and FTS helpers.
- **Obligations discharged:**
  - Storage, indexing and ad hoc query.
  - Atomic file-level replacement.
  - Offline inspection with ubiquitous tooling.
  - No service process.
- **Residual caller obligations:**
  - Extraction from sources, which is the adapter layer.
  - Source-cut and identity checks, so the index refuses to serve when stale.
  - The schema.
  - Concurrency policy. Readers are fine, and a rebuild swaps atomically.
- **Semantic conditions:**
  - Single-writer.
  - Type affinity is lenient unless `STRICT` is used.
  - Floating-point and integer precision are 64-bit.
  - No Arrow-native interchange without an adapter. pyarrow can feed `executemany`, and
    reads return Python rows.
- **Version / build conditions:** It ships with CPython. FTS5 availability depends on the
  build, and is present in standard CPython builds (not re-verified here).
- **Limits and negative results:**
  - No columnar analytic engine.
  - Graph or relation traversal is through recursive common table expressions.
- **Evidence:** Interface-checked for the version only (local interpreter). Feature list is
  documentation knowledge.

## Current applicability

### [CBO-050](../../../library_leverage/codebase/developer-tooling.md#cbo-050) — Development evidence and knowledge pipeline split across two languages
- **Types:** `sqlite/rebuildable-local-derived-index`.
- **Realization:** Installed standard library. No dependency change.
- **Conclusion:** Unsettled. It technically fits the stated constraints (disposable and
  rebuildable, explicit paths, offline inspection, no daemon). The decisive premise is a
  repository decision: whether development evidence and knowledge must live in the
  canonical database service, with its identity, generation and cursor contracts already
  implemented there, or may use a tactical file store.
- **What it would discharge / what remains local:**
  - Discharged: service lifecycle for this index, transport, and part of the Rust
    service-client code for evidence and knowledge.
  - Remains local:
    - Source adapters, the bulk of the Python modules.
    - Identity and staleness checks.
    - Cursor and generation semantics, if retained.
    - MCP query schemas.
- **Prerequisites, configuration or extension:** One schema, a rebuild command with atomic
  swap, and source-cut metadata.
- **Semantic gaps against the CBO's preservation constraints:**
  - *Source cut and identity checks:* must be reimplemented as metadata checks.
  - *Disposable rebuildable index:* yes.
  - *Explicit paths only:* yes.
  - *Offline inspection:* yes.
  - The Rust side already answers finite evidence questions with service-free DataFusion
    over parsed sources. A third store would add a path unless one of the existing paths is
    removed (LOP-09 risk).
- **Unsettled premises and what would settle them:**
  - The repository premise "SurrealDB stays canonical; Parquet/SQLite tactical only" is
    recorded for runtime and persistence. Whether it extends to development evidence is a
    maintainer decision.
  - Which evidence and knowledge questions need graph or relation traversal across
    generations.
- **Evidence:** Interface-checked (version); application Proposed.
