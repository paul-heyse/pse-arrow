# Parquet written by pyarrow, queried by DataFusion

Realizations considered:
- Installed `pyarrow` 25.0.1, a base Python dependency.
- Rust DataFusion is already used by the repository's development tooling for service-free
  evidence questions. Its pin is the workspace DataFusion family, which was not re-read
  here.
- Upgrade route on the Python side: `datafusion` (Python) 55.0.0 (PyPI, released
  2026-10-12). It bundles its own compiled engine, so it does not share the Rust
  workspace's type universe.

Research date 2026-10-27. Sources:
- PyPI JSON for `datafusion`.
- Repository inspection (static): the development evidence-query module registers Arrow
  batches built from parsed sources in an in-memory DataFusion catalog
  (`register_batch`, `MemoryCatalogProvider`). Its header says "No service ... Sources
  parse once and become the registry-owned Arrow relation".
- pyarrow Parquet APIs (`pyarrow.parquet.write_table`, `pyarrow.dataset`) are
  documentation knowledge.

## Types

### arrow-file-interchange/derived-index-as-files
- **Need it serves:**
  - A producer in one language writes typed columnar files.
  - A consumer in another language queries them through an embedded engine, with no
    service and no row-object translation.
  - The files are the rebuildable index.
- **Recognizable code shapes:**
  - Python adapters that emit JSON, which a Rust tool re-parses into structs and then into
    rows of a database or into Arrow batches.
  - TypedDicts, pydantic models, Rust structs and rows that all restate one record.
- **Mechanisms and APIs:**
  - Python side:
    - `pyarrow.Table.from_pylist(rows, schema=explicit_schema)`;
    - `pyarrow.parquet.write_table(table, tmp)` followed by `os.replace`.
    - A schema declared once can be generated from the existing registry or schema
      declaration into both languages.
  - Rust side: a `SessionContext` with `register_parquet` or a listing table over a
    directory. SQL or the DataFrame API.
  - Arrow IPC or Feather can replace Parquet when compression and statistics do not matter.
- **Obligations discharged:**
  - Typed interchange without a JSON hop.
  - Columnar query, projection and filter pushdown with Parquet statistics.
  - File-level atomicity through rename.
- **Residual caller obligations:**
  - Schema authority in one place.
  - Source-cut and identity metadata, either in Parquet key-value metadata or a sidecar.
  - Staleness refusal.
- **Semantic conditions:**
  - The Arrow type mapping must be explicit for nulls, timestamps, decimals and
    dictionaries. Inferring from Python objects drifts.
  - Parquet key-value metadata survives round trips and can carry the source digest.
- **Version / build conditions:** No new dependency if Python only writes and Rust reads.
  The Python `datafusion` package is needed only for Python-side querying.
- **Limits and negative results:**
  - No mutable index: rebuild only.
  - No server-side cursors or generations unless reimplemented.
- **Evidence:** Interface-checked: repository module header and API use (static); PyPI
  metadata. pyarrow and DataFusion file APIs are documentation knowledge.

## Current applicability

### [CBO-050](../../../library_leverage/codebase/developer-tooling.md#cbo-050) — Development evidence and knowledge pipeline split across two languages
- **Types:** `arrow-file-interchange/derived-index-as-files`.
- **Realization:** Installed `pyarrow` 25.0.1 plus the existing Rust DataFusion. No
  dependency change.
- **Conclusion:** Fits with conditions. Of the file-based routes it adds no engine and no
  dependency, because it extends the service-free DataFusion path that already exists for
  evidence questions. It does not address the knowledge store's graph and generation
  semantics, which today live in the service.
- **What it would discharge / what remains local:**
  - Discharged: the JSON hop between the Python adapters and the Rust tool, and, for
    read-only questions, the service round trip.
  - Remains local:
    - The adapters.
    - One schema authority, possibly the existing registry with generated Python and Rust
      forms.
    - Identity, staleness and generation checks.
    - MCP schemas.
- **Prerequisites, configuration or extension:** An explicit Arrow schema shared by both
  sides, and a rebuild command with atomic swap.
- **Semantic gaps against the CBO's preservation constraints:**
  - *Source cut and identity checks:* carried as file metadata.
  - *Disposable index:* yes.
  - *Explicit paths:* yes.
  - *Offline inspection:* yes, with DataFusion CLI, pyarrow or any Parquet reader.
- **Unsettled premises and what would settle them:**
  - Whether knowledge relations need a graph store.
  - The service premise.
  - Which consumers need the index while a scientific run holds the service.
- **Evidence:** Interface-checked (static repository inspection, PyPI); application
  Proposed.
