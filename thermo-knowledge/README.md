# thermo-knowledge

A standalone tree for the thermodynamic knowledge base of
[Plan 24](../docs/plans/24-thermodynamic-knowledge-base.md): source procurement, a unified
domain model and consolidation into one PostgreSQL database, `pse_thermo`.

The tree is standalone. It imports, depends on and modifies no production code (`crates/`,
`python/pse`, `packages/`, `xtask/`), is not a member of the root uv project, and has its own
Python 3.12 environment, lock, lint and test configuration. The root lint and type-check
configuration excludes it.

**No third-party data is committed.** The store (`.store/`: raw sources, staged and canonical
Parquet) is gitignored, and database dumps stay out of the tree. The tree holds only the
project's own declarations, code, manifests, lock and generated SQL.

## Commands

From the repository root, `just -f thermo-knowledge/justfile <recipe>`; inside this
directory, `just <recipe>`. Every recipe runs in `thermo-knowledge/.venv`.

| Recipe | Does |
|---|---|
| `tk-sync` | create or refresh the locked environment |
| `tk-lock` | resolve dependencies and update `uv.lock` |
| `tk-test [args]` | run pytest |
| `tk-db-bootstrap` | create `pse_thermo` if missing (peer authentication over the Unix socket, no sudo) |
| `tk-db-status` | server version, database, user and schemas |
| `tk <subcommand>` | run the stage CLI (`tk --help`) |
| `tk-acquire [<id>...]` | acquire the declared sources into the raw store and update `sources.lock` (the only networked stage; [contract](docs/acquisition.md)) |
| `tk-acquire-check [<id>...]` | offline: recompute each stored tree hash and compare it with `sources.lock` |
| `tk-read [<id>...] [--force] [--list]` | read acquired sources into source-faithful Parquet under `.store/staged/<id>/<pin>/` with a `manifest.json`; `--list` shows each source's reader and stage state (not staged, current, stale) |
| `tk-load-src [<id>...]` | load staged Parquet into schema `src_<id>` of the configured database (types mapped from Arrow, primary key on `_locator`, field metadata as column comments, `_manifest` table) |
| `tk-map [<id>...] [--phase identity] [--force]` | map staged tables to canonical Parquet under `.store/canonical/<id>/`: `--phase identity` first (source entities and identity assertions), then, after `tk-resolve`, the records and the coverage report ([contract](docs/pipeline.md)) |
| `tk-resolve [--force]` | resolve the identity claims of every source with phase-1 output to canonical entities, with a report, under `.store/canonical/_resolution/` (curated decisions in `identity/decisions.toml`) |
| `tk-build [--sources <id>] [--dry-run]` | build the canonical schemas of the configured database from the declaration and the canonical Parquet, in one transaction |
| `tk-verify [--report <path>]` | run every check of `sql/verify/` against the built database and write the JSON report |
| `tk-generate` | write `sql/generated/schema.sql` from the declaration in `model/` and `forms/` ([contract](docs/meta-model.md)) |
| `tk-generate-check` | compare the committed generated tree with a fresh generation; fails on any missing, extra or changed file |
| `tk-survey [--check] [--report [--strict]]` | validate the source survey records (`survey/*.toml`) and their dispositions; `--report` writes `survey/residue-report.md`, `--strict` fails while a construct with a stated loss has no disposition ([format](docs/survey.md)) |

`tk build` replaces the canonical schemas (`meta`, `prov`, `tk`, `param`, `ev`, `qual`) of the
configured database in one transaction: the generated DDL and `sql/physical.sql`, the reified `meta`
rows and declared entities, the union of the resolution result and every source's canonical Parquet
(identical rows are one row, rows with one key and different content refuse the build), and the
schema fingerprint, recorded as a comment on schema `tk`, with one row per source built in
`meta.build_source`. Any failure, including a constraint the commit validates, rolls everything back.
It never touches a `src_<id>` schema. With no canonical Parquet it builds the empty schema; there is
no other build path. `--sources <id>` (repeat it, or comma-separate) selects sources; `--dry-run`
takes the union, runs every consistency check and reports rows per table without touching any
database. `tk db status` reports the recorded fingerprint, whether it matches the declaration, and
the sources built.

`tk verify` runs every check of `sql/verify/` against the built database: one per invariant the
declaration marks `enforced = "verify"` and one per structural check, each a named query returning the
violating rows ([contract](docs/pipeline.md), section 4). It prints a table, writes
`.store/verify-report.json` and exits non-zero when a check has violations, a declared verify
invariant has no check file or a check file names an invariant that does not exist.

`tk qualify [<case>...]` runs the cases of `qualification/`: each evaluates a form from the database
through the reference evaluator, asks an oracle harness (`oracles/<library>.py`) for the library's own
answer at the same points, compares and records a `qualification_run`; the Parquet of a run is kept
under `.store/canonical/_qualification/` and `tk build` loads it ([contract](docs/pipeline.md),
section 5). The stage subcommand `project` is a stub that a later packet fills; it exits non-zero with
a not-implemented message until then.

`tk read` runs the reader a source's `[payload]` names (`thermo_knowledge.readers.<reader>`, or a
script in a side environment, `envs/<env>/readers/<reader>.py`, run through `envs/tk-env.sh`); the
reader contract, the writer's refusals and the side-reader protocol are documented in
`thermo_knowledge/staging/reader.py`, `writer.py` and `side.py`. A stage is skipped when its reuse
key (lock tree hash, reader name and version, hash of the reader's source files, staging format)
is unchanged; `--force` reads again.

`tk survey` is not a pipeline stage. `thermo_knowledge/survey_index/` loads the survey records, reports
every deviation from [docs/survey.md](docs/survey.md) (the files, tables, records and keys concerned)
and validates the dispositions in `survey/dispositions/<source id>.toml`: one per construct whose
`precision` is not `exact` or whose `loss` is stated, each naming a model construct, an
alignment-notes item or a design-review finding. `--report` writes the generated, byte-stable
`survey/residue-report.md`: per-source counts, the `model_change` dispositions by `ref` and every
construct that still has none.

## Configuration

`thermo_knowledge/config.py` declares everything:

- `PSE_THERMO_STORE`: store root, default `thermo-knowledge/.store` (a relative value resolves
  against the repository root).
- `PSE_THERMO_DATABASE_URL`: database URL; `tk db url` prints the effective one. It connects
  to the server the operational store uses, and the tree refuses to operate on the
  operational database `pse`.
- `PSE_THERMO_KEEP_FAILED_DATABASES=1`: keep the disposable test database of a failed test
  for inspection.

Tests create a database named `pse_thermo_test_<uuid>` on the configured server and drop it
afterwards; they never touch `pse_thermo`.
