# Layout and evidence contracts

The reference covers eleven pinned crates. `content/PROVENANCE.json` identifies source captures,
versions, format versions and generated file hashes. Rustdoc build configuration and the actual
consumer dependency/features/target profile are separate. Hosted/private captures describe their
own environments; neither implies compatibility with an arbitrary consumer.

## Routes and records

| Location | Purpose |
|---|---|
| `content/routes/` | Task, crate, representation, lifecycle and symptom routes |
| `content/capabilities/` | Reviewed briefs, structured records, coverage and invalidation dependencies |
| `content/operations/`, `content/modules/` | Full upstream docs, raw type trees and artifact-scoped references |
| `content/api/`, `content/model/` | Original compact item/module views |
| `content/index/` | Searchable tab-separated projections |
| `content/catalogs/`, `content/seams/` | Grammar, options, observed telemetry and integration boundaries |
| `content/corpus/` | Retained upstream sources, examples, tests and trace snapshots |
| `content/probes/` | Legacy small-query observations and their controls |
| `skill_improvement/evidence/implementation/` | Current contract assertion receipts, captures and resolved profile |
| `build/acquired/` | Immutable acquisition inputs, including private rustdoc and license evidence |
| `authoring/` | Maintained capability records and corrections, separate from generated views |

## Operation identity and retrieval

`operations.tsv` columns are `path`, `id`, `kind`, `crate`, `record`, `page`, `signature`.
It has no header. Paths are defining paths; aliases resolve through `aliases.tsv`. Multiple rows
can have the same path because hosted/private captures and implementation contexts remain distinct.
The ID includes package/version, capture kind, path, kind and resolved implementation context.
Raw rustdoc IDs are artifact-local; the artifact digest and item ID locate exact original bytes.

Structured records retain full `docs`, `type_tree`, `type_references`, `impl_context`, `span`,
attributes, deprecation, raw links, resolved links, capture identity, reachability and public access
route. `contract-diagnostics.json` records unresolved links and display placeholders. Neither is
silently converted into an absent capability. `reader_notes` are authored annotations kept
separate from upstream prose.

The private captures contain implementation evidence as well as callable API. `visibility` is
raw rustdoc visibility. `reachability` is the source-based interpretation:

| Value | Interpretation |
|---|---|
| supported | Publicly reachable API under the recorded capture |
| doc-hidden | Public support surface marked hidden by upstream, such as a macro helper |
| reachable-undocumented | A public call returns the value; its type has no normal public import route |
| internal | Implementation evidence; no supported public construction route established |

The reader's `brief`, `contract` and `evidence` views reveal detail progressively. Bounded JSON
fragments carry byte offsets and a result SHA-256; subsequent fragments must have the same
fingerprint. A changed result calls for restarting retrieval. UTF-8 code points are not split.

## Existing compact indexes

These remain headerless TSV files:

| File | Columns |
|---|---|
| symbols | canonical_path, kind, crate, subject, visibility, reached_via, api_page, alias_count, method_count, summary |
| methods | owner_path, method, via_trait, visibility, signature, summary |
| aliases | access_path, canonical_path, defining_crate |
| macros | macro, arm_ordinal, arm, requires_options, accepts_target, accepts_fields, accepts_state, level, family |
| spans | span, target, level, field_count, fields, parents, occurrences, scenarios, conditional_on, verdict, repaired |
| span-fields | span, field, kind, occurrences, scenarios, basis, gate, example, verdict, repaired |
| scenarios | scenario, query, options, trace |
| previews | node, scenario, ordinal, lines, width, style, capture |
| compatibility | version, published, yanked, datafusion, msrv, edition, opentelemetry, opentelemetry_sdk, opentelemetry-otlp, tracing-opentelemetry, manifest |
| behaviors | probe, topic, verdict, question, construction, expect, control, control_expect, mode, evidence |
| questions | question, area, entry_point, rejected, why_rejected, recipe, probe |

Macro grammar is extracted from captures; expansion behavior comes from exact source or consumer
assertions. The INFO execution macro requires an options argument. Default target does not imply
a zero-argument macro invocation.

## Evidence boundaries

Legacy `confirmed`, `refuted`, `recorded`, `divergent` and `blocked` verdicts are preserved in the
legacy probe index. The new receipts use execution status `passed`, `failed`, `blocked`, `not_run`
and separately name assertions and their supported claims. An inherited capture keeps its date
and construction until rerun; a static integrity check does not refresh runtime evidence.

Snapshot span and field rows stay `recorded`. Field presence can depend on a node, option,
execution path and subscriber. The `datafusion.metrics.*` vocabulary is open. Field/option
correlation in upstream scenarios is not a universal causal guarantee.

One upstream snapshot needs an exact named repair before JSON parsing. `build/spans.py` records
the literal repair; raw corpus bytes remain unchanged and affected rows retain the repair marker.
Other parse failures are errors. The current structured consumer captures retain IDs, parents,
order and field values; their hashes identify actual observations, not normalized golden output.

The compatibility table contains manifest requirements and historical example combinations.
The new probe profile records actual resolved packages and features. Compilation, local SDK
export, OTLP transport acknowledgement and backend retention remain different claims.

## Structural queries

```bash
ast-grep scan -c queries/sgconfig.yml --filter '^project-' /path/to/consumer
```

Rules are syntax hints and never edit consumer code. Aliases, dynamic option values, receiver
identity and execution order may require semantic inspection. The preview hint now identifies
an inline builder without a limit setter; omitting a custom formatter alone is not a gap.
Read [queries/README.md](queries/README.md) before changing the corpus.

## Known limits

The focused consumer suite is not exhaustive across custom operators, arbitrary concurrency,
query shapes, data types, storage providers or platforms. Row limits and stream types do not
prove process memory bounds. Local SDK tests do not qualify OTLP delivery or a collector.
Unresolved links, excluded crates and unreviewed API remain visible. Missing evidence is not
negative capability evidence. Maintenance and qualification commands are in
[maintenance.md](maintenance.md).

## Retained inventory

The compact inventory has 502 canonical items, including 16 reachable-undocumented methods,
49 macro arms, 9 snapshot span rows, 69 observed fields and 35 preview captures. Its original
routes contain 18 questions and 8 seams; the compatibility history has 16 release rows.
There are 11 syntax rules. The 12 inherited probes contain 11 `confirmed` verdicts; those labels
are historical and separate from the new 12 consumer tests. The full reader exposes 3,504
operation contracts (3504 records) and 14 reviewed briefs. Counts describe coverage, not quality.
