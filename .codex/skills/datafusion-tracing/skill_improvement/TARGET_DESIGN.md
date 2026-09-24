# Target design: a DataFusion tracing programming reference

Proposed 2026-09-18. The [assessment](README.md) records the starting point. Names and layouts
below describe the intended interfaces; they do not imply those interfaces are implemented.

## 1. Start from the information the agent has

| Entry mode | Useful first result |
|---|---|
| Instrumentation task | Relevant capabilities, observable effects, and conditions distinguishing alternatives |
| Known symbol or macro | Full contract, callable access path or invocation grammar, profile, and evidence |
| Missing or unexpected telemetry | Competing explanations and the observation that distinguishes them |
| Existing code | Candidate integration points and the semantic assumptions needed to interpret a match |
| Crate or dependency combination | Role, declared requirements, selected features, and compatibility evidence |

No entry mode is compulsory. Direct files remain usable alongside a small query helper. The
entry point introduces the reference, its release scope, and these routes; detailed cautions
belong with the affected capability.

### Task routes

Task vocabulary includes ordinary descriptions such as “explain planning time,” “trace physical
operators,” “attach a query identifier,” “see storage requests,” “preview a few rows,” “trace
ends before fields appear,” and “stdout works but the collector is empty.”

| Task family | Capabilities and alternatives | Distinctions to expose |
|---|---|---|
| Observe query planning | Phase instrumentation, per-rule instrumentation, selected phase builders | Session-state transformation, selected phase set, detail volume, plan rendering |
| Observe physical execution | Execution instrumentation macros and the resulting optimizer rule | Rule placement, plan rewrites, execution groups, partitions, repeated use |
| Inspect query measurements | Native plan metrics and their recording on spans | Availability, units, aggregation, timing, dynamic field names, retained plan state |
| Inspect representative rows | Preview disabled, default formatter, custom formatter | Output limit, partition collection, formatting errors, data exposure, buffer retention |
| Correlate work | Declared custom fields, parent spans, async instrumentation, trace context | Static field schema, value recording, parent versus link, polling and task boundaries |
| Observe storage | Wrapped store registered at the consumer's integration point | Request versus returned stream lifetime, method coverage, errors, multipart operations |
| Select telemetry | Macro target/level, global and per-layer filtering, SDK sampling | Where data is suppressed, which layer still receives it, cost before suppression |
| Export telemetry | Local formatting, bridge layer, SDK processor, exporter | Provider ownership, runtime/features, batching, flush/shutdown, receipt boundary |
| Diagnose an incomplete trace | Construction, execution, filtering, sampling, close, export evidence | An absent field or span has several possible causes; identify the next discriminating observation |
| Match a consumer environment | Manifest requirements, lockfile, enabled features, target | Declared versus resolved compatibility; duplicate type universes; tested combinations |

These are coverage families, not a requirement for one document per row. Broad inventory and
reviewed semantic depth remain separate. The number of indexed symbols is not a measure of
how well a capability has been characterized.

### Crate roles

Retain all eleven indexed crates and give each a compact role with relevant entry points:

| Crate | Role in this reference |
|---|---|
| `datafusion-tracing` | Planning and execution instrumentation, options, previews, metric recording |
| `instrumented-object-store` | Instrumented wrapper at the object-store API boundary |
| `tracing` | Span/event construction, fields, current context, and future instrumentation |
| `tracing-core` | Metadata, dispatch, subscriber interfaces, and field recording contracts |
| `tracing-attributes` | Attribute-based instrumentation and its argument/field behavior |
| `tracing-futures` | Async instrumentation and subscriber propagation adapters |
| `tracing-subscriber` | Registry, layers, filters, formatting, and subscriber installation |
| `tracing-opentelemetry` | Translation between tracing spans and OpenTelemetry context/data |
| `opentelemetry` | Telemetry API, trace context, and propagation interfaces |
| `opentelemetry_sdk` | Provider, processor, sampler, resource, and lifecycle ownership |
| `opentelemetry-otlp` | Exporter construction, transport features, and delivery configuration |

An indexed crate is available reference material, not an obligatory application dependency.
Retain canonical ownership and usable imports separately. DataFusion and `object_store` are
boundary references; the subject does not need a duplicate query-engine or storage API catalog.

### Lifecycle routes

Represent the relationships an agent needs to compose, including where observations can stop:

```text
options / macro invocation
  → session planning wrappers or physical instrumentation rule
  → query planning / physical execution / storage operations
  → span creation, field updates, stream completion or drop
  → subscriber and layer handling
  → OpenTelemetry context, sampling and processing
  → exporter attempt → receiver acknowledgement / retained collector data
```

This is an orientation map, not a strict execution timeline. Filters may suppress span creation;
context is established before child work; layers can record at different lifecycle callbacks.
Individual contracts establish those details. A local formatted close event and a collector
record are different observations.

## 2. Use capability briefs and exact operation contracts

A brief describes a coherent effect and its alternatives. An operation record preserves the
upstream API used to produce it. Both are useful without turning the skill into a tutorial.

| Brief element | Information to retain |
|---|---|
| Purpose and vocabulary | Desired observation and ordinary task aliases |
| Alternatives | Available mechanisms and the conditions under which their effects differ |
| Entry points | Functions, builders, macros, import routes, and visibility/reachability |
| Inputs | Exact types or macro token forms, state/plan/store ownership, options, field declarations |
| Outputs | Return value plus emitted spans/events/fields, naming, parents, and representation |
| Lifecycle and effects | Construction, planning, polling, completion/drop, formatting, export; sharing and retained state |
| Composition | Registration point, required subscriber/layer, parent context, features and dependency profile |
| Failures and missing output | Errors, panics, silent suppression, partial execution, formatting/export failure |
| Evidence and limits | Source/doc claims, inherited captures, local assertions, conflicts, and untested conditions |

Use only the dimensions that matter for the capability. Comparisons present facts and tradeoffs
without a mandatory `recommended` field. The agent can form a preference from the user's task.
Unknown behavior stays explicit rather than being filled with generic observability advice.

### Preserve the complete upstream contract

Extend the current model and rendering to retain full module/member/field/variant documentation,
raw rustdoc type trees, receiver and impl context, generics, bounds, source locations, attributes
when available, and documentation links. Keep the current compact API pages and TSV indexes as
discovery views. Unsupported rendering or link resolution produces a visible diagnostic.

Operation identity includes package version, capture/build profile, defining owner, member, and
impl context. Raw rustdoc IDs remain local to their artifact. Hosted and private captures have
different environments and IDs; they must not be merged as if they were one compilation.
Deduplication by method name alone loses meaningful impl distinctions.

For the subject crates, retain the source-based reachability classification. A canonical private
path is a locator, not an import instruction. A reachable builder contract includes the public
call that returns it and a consumer composition establishing callable methods. An internal
wrapper remains source evidence even when its rustdoc item says `pub`.

For macros, preserve each invocation arm, selected level, explicit/default target behavior,
accepted option/state/field syntax, expansion source, and callable result. Grammar extraction
does not establish expansion behavior; source inspection and compile/runtime examples answer
different parts of that contract.

## 3. Make evidence specific to the claim

The current probe and snapshot assets stay useful. Richer records make their limits visible:

| Record | Purpose |
|---|---|
| Capability | Task language, relevant operations, distinguishing conditions |
| Operation | Full API/macro contract and artifact/profile identity |
| Claim | One scoped statement, conditions, evidence kind, and supporting locators |
| Comparison | Alternatives, material differences, and unresolved assumptions |
| Composition | Connected operations, required context, state ownership, and resulting observation |
| Observation | Raw capture, normalization, scenario, subscriber/export setup, observed fields/lifecycle |
| Probe | Construction, assertion IDs, controls, environment, command, result, and known limits |

JSON records and generated TSV/Markdown views are sufficient. The purpose of these relationships
is accurate retrieval and maintenance, not an additional service or general knowledge platform.

Keep these distinctions explicit:

- Full docs, source inspection, upstream tests, upstream snapshots, compiled consumers, local
  runtime assertions, and authored interpretations have different authority.
- A probe confirming span existence does not confirm all columns of a snapshot-derived span row.
  Link the assertion to the claim; preserve the aggregate row's original provenance.
- An option correlated with a field in a snapshot is a lead. Causal attribution requires the
  relevant source path or a controlled construction. Dynamic metric fields remain open-ended.
- Execution status (`passed`, `failed`, `blocked`, `not_run`) is distinct from a claim verdict
  such as supported, contradicted, or recorded. Preserve legacy probe verdicts as legacy data.
- Existing captures are inherited evidence until rerun. A successful static check does not update
  their execution date or qualify a different feature/target profile.

Preserve raw snapshots alongside normalized projections. Record the exact malformed-snapshot
repair, input/output hashes, and dependent rows. Normalization may remove variable timing or IDs
for comparison while retaining raw values and the relationships needed to test parentage.

Compatibility has separate levels: declared requirement, selected dependency graph, successful
compilation, observed local export, and observed receipt. Version-number patterns may help
discovery but do not prove any of those levels. The bridge and SDK are optional for consumers
that only need local tracing output.

## 4. Characterize the seams that change implementations

The [worked examples](CAPABILITY_EXAMPLES.md) seed the first briefs. The deeper backlog includes:

| Seam | Questions needing precise evidence |
|---|---|
| Planning versus execution | Which phases are wrapped? What changes after later optimizer rewrites? Does repeated instrumentation duplicate wrappers? |
| Execution groups and ownership | When do metrics/previews finalize? What happens with partial partitions, retained plan handles, concurrent executions, errors, or early stream drop? |
| Fields and identity | Which keys must exist at creation? Which values are late? What is a tracing name/target versus `otel.name` or an exported attribute? |
| Async context | What context applies at construction, polling, spawn, and completion? When is explicit propagation needed? |
| Filtering and sampling | Which output branch suppresses data? What work has already occurred? Does a disabled parent still affect child context? |
| Preview | Default versus custom formatting, per-partition accumulation, final cap, failure paths, schema and retained buffers |
| Metrics | Native metric source, units, aggregation, recording time, repeated execution, fields absent from the current catalog |
| Object storage | Covered methods, returned streams, lazy reads/listing, multipart handles, errors and cancellation |
| Export | Layer/provider ownership, batch versus simple processor, transport features, timeouts, queue behavior, flush/shutdown and receipt |

No generic performance conclusion follows from a passing small query. Trace volume, formatting
cost, retained memory, and exporter loss each need an experiment appropriate to that claim.

## 5. Retrieval and authoring layout

Proposed layout follows the enhanced DataFusion skill while preserving existing artifacts:

```text
SKILL.md                         short router, release scope, evidence interpretation
reference.md                     schemas and precise lookup recipes
maintenance.md                   authoring, regeneration, probes, packaging
authoring/
  tasks.json                     task and symptom vocabulary
  crate-roles.json                package roles and boundaries
  capabilities/*.json            reviewed claims and comparisons
content/
  routes/                        task, crate, lifecycle and symptom views
  capabilities/                  briefs and explicit characterization coverage
  operations/                    complete member/field/macro contracts
  modules/                       preserved module documentation
  index/                         current TSVs plus operation and claim indexes
  api/ model/ catalogs/ seams/    retained compact reference views
  corpus/ probes/                 source, upstream captures, and probe projections
scripts/                         optional bounded find/show/compare reader
build/                           acquisition, extraction, generation and verification
```

Permanent evidence can stay in the current acquired/corpus/probe locations; stable locators and
manifests matter more than moving files. New authoring records have one source of truth and are
rendered into the task, comparison, and capability views.

Illustrative reader interface, not commands available today:

```text
find --task "planning is slow but execution is fast"
find --task "stdout has spans but the collector does not"
show tracing.preview --view contract
show datafusion_tracing::InstrumentationOptions --member preview_fn --view contract
compare tracing.rules.phase tracing.rules.full
show tracing.export --view evidence
```

Results contain matched task terms or facets, a bounded candidate set, decisive differences,
evidence pointers, and unknowns. Exact and alias lookup remain deterministic. Facets describe
reviewed properties; they do not infer type compatibility or diagnose a consumer automatically.
Long contracts expose truncation and a continuation or direct file, including the result identity
needed to resume without losing or duplicating content.

The reader uses local files and a small standard-library helper, with paths resolved from the
installed skill location. It does not need Cargo, a running query engine, source caches, a
collector, or network access. Additional search machinery is justified by retrieval failures,
not by the number of indexed crates.

## 6. Maintenance and transfer

Separate a reader bundle from the optional research bundle. The reader carries brief/contract
evidence needed for offline inspection and explicit locators for larger artifacts. The research
bundle adds acquisition inputs, source, probe fixtures/locks, and run receipts. Neither needs
compiler targets or paths into the original checkout. Preserve license and notice provenance.

Source/capture changes invalidate dependent claims, comparisons, and probes. Changes to the
candidate surface can invalidate a comparison even if its previously selected API is unchanged.
Repinning the subject also requires checking DataFusion integration, macro/source behavior,
capture formats, and wiring dependencies independently.

Copied-bundle checks exercise lookups from an unrelated directory with the original checkout
and hidden caches unavailable. Research reproduction records its own tools and prerequisites;
offline reading and offline compilation are separate promises. Context7 can contribute discovery
leads during maintenance, but neither it nor a sibling skill is a reader dependency.
