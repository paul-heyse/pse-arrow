# DuckDB (Python)

Realizations considered: not installed in `.venv`. Upgrade route `duckdb` 2.0.0 (PyPI,
released 2026-10-21, a new major line). A 1.x line would be the conservative pin;
which 1.x release is latest was not resolved.
Research date 2026-10-27. Sources: Context7 `/websites/duckdb_current`:
- securing extensions: `SET autoinstall_known_extensions = false`,
  `SET autoload_known_extensions = false`, `lock_configuration`;
- bundled extensions: ICU, JSON, Parquet and Autocomplete are statically linked in
  pre-built libraries (stated for the Go client page);
- the Parquet overview: "bundled with almost all clients".

Also PyPI JSON.

## Types

### duckdb/in-place-file-query-engine
- **Need it serves:** Answer SQL questions directly over JSONL, Parquet or CSV files, either
  with no import step or through a single-file database, for local analytic indexes that
  are rebuilt from files.
- **Recognizable code shapes:**
  - Pipelines that parse files into objects, convert them to rows, load them into a service
    database and then query.
  - Ad hoc Python aggregation over JSON projections.
- **Mechanisms and APIs:**
  - `duckdb.connect(path | ":memory:")`.
  - `read_json('glob', format='newline_delimited')` and `read_parquet('glob')`.
  - `CREATE TABLE AS SELECT` to persist.
  - Relational Python API.
  - Arrow interchange: `.arrow()` and `.fetch_arrow_table()`, and querying a pyarrow table
    by variable name.
  - Extension control settings for offline operation.
- **Obligations discharged:**
  - Parsing and type inference of JSON and Parquet.
  - Columnar SQL execution.
  - Arrow output.
  - Single-file persistence.
- **Residual caller obligations:**
  - Locking down extension autoload for offline determinism.
  - Explicit schemas instead of inference when identity matters.
  - Source-cut and staleness metadata.
- **Semantic conditions:**
  - Inferred JSON schemas can change with the data. Pin the schema with explicit
    `columns=`.
  - Autoinstall reaches the network for non-bundled extensions unless it is disabled.
  - Single-writer database files.
- **Version / build conditions:**
  - 2.0.0 is a new major. File-format and API compatibility relative to 1.x was not
    checked.
  - The Python wheels bundle JSON and Parquet. This is stated in general docs, not
    verified for the Python wheel specifically.
- **Limits and negative results:** No server-style durability or multi-process write
  coordination.
- **Evidence:** Interface-checked (Context7 docs pages named above; PyPI metadata). Python
  API details are documentation knowledge.

## Current applicability

### [CBO-050](../../../library_leverage/codebase/developer-tooling.md#cbo-050) — Development evidence and knowledge pipeline split across two languages
- **Types:** `duckdb/in-place-file-query-engine`.
- **Realization:** Upgrade route, a new Python dependency.
- **Conclusion:** Unsettled, and for the same deciding premise as the other file-store
  options (whether development evidence must sit in the canonical service). It adds a
  second embedded SQL engine beside the Rust DataFusion path that already answers finite
  evidence questions service-free.
- **What it would discharge / what remains local:**
  - Discharged: the Python-to-JSON-to-Rust-to-service hop for read-only questions, because
    Python adapters could write JSONL or Parquet and DuckDB could query it in place.
  - Remains local: adapters, identity and staleness checks, MCP schemas.
- **Prerequisites, configuration or extension:** Extension lockdown, explicit schemas, and
  a pin choice between the 1.x and 2.x lines.
- **Semantic gaps against the CBO's preservation constraints:** Offline inspection is yes,
  provided extension autoload is disabled.
- **Unsettled premises and what would settle them:**
  - The service premise, a maintainer decision.
  - Whether a Python-side SQL engine is wanted at all when DataFusion already serves the
    Rust side. The Parquet-plus-DataFusion route avoids a new engine.
- **Evidence:** Interface-checked; application Proposed.
