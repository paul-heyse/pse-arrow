# Library utilization catalog

A flat, generated record of which third-party libraries the workspace uses, and (once curated)
which of their capabilities it uses, through which local abstraction, in which files. Agents read
it through the `library-catalog` MCP server; humans can read the JSONL directly.

| File | What |
|---|---|
| `docs/library-utilization.jsonl` | The catalog: one JSON object per line, `kind` is `library` or `capability` |
| `build/library-usage.sqlite` | Gitignored usage index written by a full `--write` run: every library item the source references, and each file's mtime and size at indexing time |

Regenerate both with `just library-catalog` (`scripts/library_utilization.py --write`). It edits
the tree and takes a few minutes the first time (it builds `tools/lu-resolve` offline), then one to
two. `uv run python scripts/library_utilization.py` alone is a
dry run: it prints the drift and exits 1 if there is any.

## Records

A `library` record holds the pin, declared and compiled features (`features`,
`resolved_features`), the workspace crates that link it (`linked_by`, `dependents`), the covering
library skill and its pin delta, `status`, and the files that reference it by role (`used_in`).
`status` is `used`, `test-only`, `dormant-only`, `declared-unused` or `not-used` (skill-covered, no
direct dependency).

A `capability` record is **hand-curated**: `id`, `lib`, `feature`, `use`, `items`, `as_written`,
`wrapper`, and the computed `files`, `n_files`, `status`, `verified`. The script never invents
capabilities. It keeps the curated fields, rewrites `items` to the defining paths, and recomputes
the files and status. **This repository starts with none**, so `capabilities_by_library` returns
`{"libraries": {}}` until they are written; the usage index and every other tool work without them.

Only direct workspace dependencies covered by an enabled library skill (or already in the catalog)
become `library` records. The rest are listed as `unlisted` in the dry-run report and by
`catalog_gaps("dependency_without_record")`.

## Pipeline

| Stage | Reads | Gives |
|---|---|---|
| S0 | `cargo metadata`, `Cargo.lock`, the enabled skills' manifests, nightly `cargo build --unit-graph` | libraries, pins, features, compiled features and which crates link them |
| S1 | every workspace `.rs` file | a lexical reference scan, roles (`src`, `test`, `dormant`) and `status` |
| S2 | `tools/lu-resolve` (rust-analyzer, `ra_ap_*` 0.0.352, no build, proc-macro server off) | every library item each file resolves, by defining path |
| S3 | the skills' `content/index` tables | canonical item names |

`tools/lu-resolve` has its own workspace and its own toolchain (`nightly-2026-09-13`, in its
`rust-toolchain.toml`) and builds under `tools/lu-resolve/target`, so it never touches the product
build directory.

Exit codes: 0 no drift (or written), 1 drift in a dry run, 2 setup failure, 3 the catalog changed
during the run. `--json` prints one JSON object; `--index PATH` overrides the index location.

## The catalog server

`scripts/library_catalog_mcp.py` is a local, read-only FastMCP server over stdio, registered in
`.mcp.json` as `library-catalog`. `.mcp.json` starts it with
`uv run --no-project --with fastmcp==4.0.5`, so it needs nothing from the project environment and
adds nothing to `pyproject.toml` or `uv.lock`.

| Tool | Answers |
|---|---|
| `capabilities_by_library` | Every capability the workspace uses, as JSON grouped by library; no parameters |
| `get_library` | A library's record with its capabilities, wrappers, features and skill |
| `library_usage` | Every resolved item of a library with all files and lines, by item or by file |
| `find_item` | An item under any spelling (facade path, macro `!`, `Type::method`, bare name): capabilities plus every resolved usage row |
| `find_by_file` | Capabilities for a file or directory, and every library item its files use |
| `search_capabilities` | Every capability matching the words, ranked, with optional filters |
| `catalog_gaps` | Not-used libraries, declared-unused, pin deltas, capabilities without a wrapper, dependencies without a record |
| `catalog_status` | Libraries with status and capability ids, skills, verified stamps, and what the usage index covers |

Nothing is truncated. A miss (`no_reference_found`) is answered from the usage index and states its
blind spots: references inside macro and derive output, methods a proc macro generates, files
outside the module tree, and types that are only inferred. It does not mean the library is unused.

## Tests

`scripts/tests/test_library_*.py` (pytest, `unit` tier; the stdio test is `component`). They need
fastmcp, which the project environment does not carry:

```bash
uv run --no-project --with fastmcp==4.0.5 --with pytest python -m pytest -c /dev/null \
  --rootdir . -p no:cacheprovider scripts/tests/test_library_*.py
```

The catalog scripts and their tests are excluded from ruff and pyrefly in `pyproject.toml`: they
use Python 3.14 syntax and are a maintainer tool set, not product source.
