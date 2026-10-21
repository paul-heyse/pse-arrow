# Library utilization catalog

**Retired reference — 2026-10-20.** The catalog MCP server, refresh/query helpers, resolver and
dedicated tests have been removed. This page explains retained historical data for existing
review/plan consumers; it is not an operator guide or a current inventory.

For current research use [library overviews and their authoring guide](libraries/README.md),
then Context7 and primary sources. Only native-solver-libraries remains an active library skill.
The retained JSONL records keep their own dated verification stamps, pins and source identities;
those observations and old skill profiles do not establish current dependencies, API behavior or
present usage. No catalog refresh or replacement indexing service is part of the current workflow.

The former catalog recorded which third-party libraries the workspace used and, where curated,
which capabilities were used through which local abstractions and files. The following describes
that retired representation and producer, not available commands or services.

| File | What |
|---|---|
| `docs/library-utilization.jsonl` | The catalog: one JSON object per line, `kind` is `library` or `capability` |
| `build/library-usage.sqlite` | Gitignored usage index written by a full `--write` run: every library item the source references, and each file's mtime and size at indexing time |

The retired producer wrote the JSONL and a local usage index. Its regeneration recipe, dry-run
entrypoint and rust-analyzer resolver no longer exist. Historical producer details are recoverable
from Git alongside the original catalog; retaining this data does not imply that a usable SQLite
index remains on disk.

## Records

A `library` record holds the pin, declared and compiled features (`features`,
`resolved_features`), the workspace crates that link it (`linked_by`, `dependents`), the covering
library skill and its pin delta, `status`, and the files that reference it by role (`used_in`).
`status` is `used`, `test-only`, `dormant-only`, `declared-unused` or `not-used` (skill-covered, no
direct dependency).

A `capability` record was **hand-curated**: `id`, `lib`, `feature`, `use`, `items`, `as_written`,
`wrapper`, and the computed `files`, `n_files`, `status`, `verified`. The retired script preserved curated fields, rewrote item names to defining paths and recomputed
files and status; it did not invent capability descriptions. Whether a retained snapshot contains
curated capability records must be read from that snapshot, not inferred from this format note.

The old producer selected direct dependencies covered by then-enabled library skills or already
present in the catalog. Its gap report described that historical selection, not a complete view
of current dependencies.

## Retired pipeline

| Stage | Reads | Gives |
|---|---|---|
| S0 | `cargo metadata`, `Cargo.lock`, the enabled skills' manifests, nightly `cargo build --unit-graph` | libraries, pins, features, compiled features and which crates link them |
| S1 | every workspace `.rs` file | a lexical reference scan, roles (`src`, `test`, `dormant`) and `status` |
| S2 | `tools/lu-resolve` (rust-analyzer, `ra_ap_*` 0.0.352, no build, proc-macro server off) | every library item each file resolves, by defining path |
| S3 | the skills' `content/index` tables | canonical item names |

The removed resolver used its own workspace, nightly-2026-09-13 toolchain and local target
directory. Its old source/lock/toolchain are recoverable from Git; they are not active setup inputs.

## Retired catalog server

The removed server used local read-only FastMCP over stdio and a temporary fastmcp==4.0.5
environment. Its registrations are gone from both runtimes. The names below explain historical
tool results cited by existing consumers; none is an available local catalog tool.

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

The old server returned full selections. A miss (`no_reference_found`) came from the usage index with its
blind spots: references inside macro and derive output, methods a proc macro generates, files
outside the module tree, and types that are only inferred. It does not mean the library is unused.

## Retired tests and tooling

The eight dedicated catalog/scan/name/resolver test modules were removed with their implementation.
Their former isolated FastMCP/pytest invocation and catalog-only lint exclusions are retired.
Retained reports do not carry a current test pass or authorize restoring the mechanism.
