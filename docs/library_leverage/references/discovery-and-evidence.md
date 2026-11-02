# Discovery and evidence

A library-led briefing answers three connected questions: what important capabilities exist,
what the repository currently does with them, and what deserves more investigation. It is an
optional entry to the existing leverage workflow, not an additional prerequisite.

## Combine sources according to what they establish

| Evidence | Useful meaning | Limit |
| --- | --- | --- |
| Comprehensive overview and primary documentation | Capability families, interactions and search handles | Does not describe local installation or fit |
| Cargo package/dependency facts | Exact package sources, declared features and dependency relationships | Metadata features are not every compiler-unit context |
| Acquired rustdoc export | Public paths, associated items, trait/impl relationships and declared contracts | Selected documentation context; IDs are artifact-local; `cfg(doc)` may expose additional APIs |
| Prepared semantic reference census | Resolved written references in the selected world | Imports and type mentions are not executed calls; inactive cfg and missing expansions limit coverage |
| Representative source reads | What an observed use accomplishes and who consumes it | Establishes only the inspected footprint |
| Focused upstream source/tests or a probe | A selected technical condition under stated premises | Does not establish general product qualification |

Retain defining package identity separately from public facade paths and dependency aliases.
Corroborate joins through original source/configuration evidence; names and numeric IDs from
different producers are not interchangeable. Ambiguous correspondence remains explicit.

The evidence workflow exports observations. Agents interpret those observations using the
repository's general objectives and configuration. A generated inventory is neither a ranked
recommendation nor a complete semantic model of the library.

## Shape the briefing

Place one `briefing.md` in the library-family account. State the inspection date/source and
whether the briefing consumes generated evidence or representative reads alone. Keep a compact
capability-family map, current usage explanation and a reasoned research shortlist. Link to
the broad overview, exact API/source evidence and reusable technical accounts.

Explain both promising and less relevant territory. This prevents discovery from becoming a
list of APIs the repository already happens to use. Qualify absent evidence as not observed
in its selected context. Preserve distinctions among available now, configuration-dependent,
upgrade-dependent and unsettled routes.

Use the toolkit's family-selected evidence bundle when prepared. Preparation and API acquisition
are explicit effects; ordinary source reading and inspection of retained artifacts need neither.
Keep large generated bundles in caller-selected output locations rather than copying them into
the briefing. Retain enough package, context and source identity to reproduce and challenge the
interpretation. A failed package operation remains a coverage gap even if others succeed.

## Run the local evidence workflow

`just library-evidence --output NEW_DIR --family tokio` reads already prepared inputs.
Select `datafusion-arrow` or `symbolica`, repeat `--family`, or omit it for all configured
families. The directory must be new. The family mapping in `.config/library-leverage.toml`
contains package selectors and overview routes; dependency owners supply versions and sources.

Add `--prepare` to explicitly prepare the selected native snapshot and `--acquire` to acquire
selected package APIs. These effects can compile providers and dependency documentation.
`--features`, `--target`, `--no-default-features`, `--test-context` and `--build-macros`
select the investigated world; they do not establish every other build configuration.
Use a separate output directory for another selection. Help and ordinary retained-input reads
do not prepare missing capabilities implicitly.

Read `manifest.json` and `README.md` first. The bundle retains selection and metadata,
reference observations and per-package API tables. Individual failed operations remain in
the manifest, other independent packages still run, and incomplete collection returns failure.
Do not describe that result as a complete census. To deepen a selected fact independently,
the host's `inspect dependency-usage` and `inspect api --export-jsonl` routes retain their own
prepared-world and acquired-artifact boundaries; consult current command help for selectors.

## Judge whether discovery helps

Use concrete questions rather than an inventory-size target. Can a reader find a useful feature
outside familiar entry points, recognize its mechanism, and explain a legitimate case where it
would fail to fit? Can another agent trace an apparent use through a facade to the defining crate?
Does the briefing prevent a notification API from being mistaken for physical task settlement,
or an optimized numerical program from being mistaken for an authored mathematical contract?

The pilot [DataFusion/Arrow](../libraries/datafusion/briefing.md),
[Tokio](../libraries/tokio/briefing.md) and [Symbolica](../libraries/symbolica/briefing.md)
briefings exercise those questions. They are research outputs with their stated evidence limits,
not proof of measured agent improvement. A fresh reader can answer the questions from each
briefing and then verify against its cited source; quantitative claims need a measured comparison.
