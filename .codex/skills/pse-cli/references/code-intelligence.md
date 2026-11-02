# Code intelligence by question

Use the section that answers the current question. `pse COMMAND --help` supplies concise
syntax; `pse help COMMAND` supplies structured parser metadata. Result schemas come from
`pse schema`; detailed preparation examples live in the
[environment guide](../../../../docs/dev/agent-environment.md#focused-command-and-source-context).
Examples below use operands such as `FILE`, `PACKAGE` and `INVOCATION` to replace deliberately.

- [Written source and structural changes](#written-source-and-structural-changes)
- [Resolved Rust context and edit proposals](#resolved-rust-context-and-edit-proposals)
- [Locked APIs and dependency context](#locked-apis-and-dependency-context)
- [Original failures and new compiler investigations](#original-failures-and-new-compiler-investigations)
- [Generated-source ownership](#generated-source-ownership)

## Whole-checkout discovery and composed questions

Use `search` to find useful candidates, `inspect` to expand known source or a returned
reference, and `propose` to construct a checked preview. Preparation and execution remain
explicit. Ordinary source/text discovery needs no analyzer, database, API acquisition or Cargo.

```bash
pse search files --path crates/pse-host/src --glob '*.rs' --limit 10
pse search text 'CancellationToken' --literal --path crates/pse-host/src --context 1
pse search symbol EnvironmentSettings --path crates/pse-host/src
pse search syntax 'Command::new($BIN)' --path crates/pse-host/src --limit 8
pse inspect --source crates/pse-host/src/cli.rs --lines 10:40
pse search --request query.json --explain
```

The default is twenty useful arrival-order results. `--exists`, `--count` and `--all`
request different native examination/completion. `--order deterministic` enables guarded
continuations; pass the returned cursor unchanged. A small page establishes neither exact
totals nor exhaustive absence. No-match, partial I/O, stale context, unsupported predicates
and absent preparation remain different outcomes.

Use a typed JSON request for grouped predicates and batches; `pse schema search request`
describes its actual input. Same-file conjunction can have different witness nodes; same-node
conjunction holds on one native node; semantic conjunction binds one entity/context. Unavailable
information remains unknown, including under negation. Do not narrow global references to
only files containing a convenient spelling.

`search structured POINTER` selects JSON/TOML/YAML fields, with explicit typed equality;
it does not implement every jq/SQL operation. `search history` selects native log/blame/pickaxe
questions. Historical source does not inherit current semantic context. Prefer domain readers
for dependency features, tests and source ownership over scraping their serialized files.

`--result-fields name,context` chooses optional detail while source/reference/range stay mandatory.
`--group-by source`, `kind` or `name` groups returned witnesses, not universe-wide counts.
`--budget` bounds published whole rows independently of requested examination. NDJSON publishes
live complete records and final coverage; absent finality is incomplete delivery. Actual progress
is stderr presentation, not a fabricated percentage or success proof.

Saved requests, named queries and source scopes use optional `.config/pse-search.toml` or explicit
`--config`. Select saved queries with `--query NAME`. Explicit flags override selected request/config
values; arrays replace, and file-relative paths resolve at their source. `search --explain` exposes
effective values and provenance without starting providers or another host environment evaluator.

References bind original source/context, without a universal result database. A lexical observation
need not snapshot a full file. Before previews, exact hydration verifies the observed witness and
refuses stale premises. A reference must not silently reinterpret an identical path in another checkout.

## Written source and structural changes

For protobuf questions, use `search schema NAME --descriptor FILE`, then expand its guarded
reference with `inspect --result` or navigate directly with `inspect schema --descriptor FILE
FULL.NAME`. `inspect payload --descriptor FILE --message FULL.NAME --input PAYLOAD --encoding
binary` passively explains selected bytes; `json` and `textproto` are explicit alternatives.
Use schema inspection's `--compare-with BASE` for contract differences. These offline routes
do not prepare an analyzer or invoke an RPC. Validation coverage, original-byte preservation
and any generated-Rust association remain separate evidence claims.

Use source-only reads before preparing a semantic service when declaration structure or exact
written bytes will settle the question. These reads also work outside a pse-arrow checkout.

| Question | Capability and useful limits |
|---|---|
| Where is a declaration or document section? | `pse inspect outline FILE --budget 20` reads nested Rust declarations or Markdown headings without Cargo, cfg evaluation or macro expansion. Multiple files are supported; use one file for a continuation. |
| What exactly surrounds this location? | `pse inspect declaration FILE --at-line N --at-column N` returns the enclosing declaration/section. `--range-start` / `--range-end` select a half-open UTF-8 byte range. Line/column coordinates are one-based Unicode scalars. |
| What does this written literal, documentation or attribute mean? | Use `--detail facts` for selected interpreted values or `--detail mapped` for decoded-to-source mappings. Outlines default to navigation; declarations default to facts, raw ranges to navigation. Syntax matches carry their enclosing declaration, receiver, ordered arguments and owner-matched documentation/attributes; these remain source facts, without cfg or type inference. |
| What was staged or committed? | Add `--index` for the raw staged blob or `--revision REF` for a resolved committed blob. This avoids checkout changes and distinguishes Git bytes from current working bytes. |
| Which written shapes match a refactor? | `pse search syntax 'call($ARG)' --path FILE` returns structural candidates with captures and exact source identity. Rust is the current structural language. |
| Which matches occur in the intended context? | Native rule requests express ancestor/descendant/sibling/field constraints and repeated captures. Strictness selects matching policy. Syntax previews also retain native `--inside`, `--has`, `--exclude`, `--strictness` and `--capture` input controls. |
| What would a replacement change? | `propose syntax` uses the same pattern plus `--replacement` and optional `--select` candidate indices. Inspect candidates before selecting edits; selected overlaps and undefined template captures refuse. Neither source nor the Git index changes. |
| Can a grammar-aware edit handle separators and surrounding syntax? | `propose grammar FILE --byte-start N --byte-end N OPERATION` uses native Rust editing for where predicates, imports, bounds, associated items, match arms and expression precedence. Use its help for operation arguments; an exact source guard and preview remain required before applying edits. |
| What changed, and which tests might matter? | `pse inspect changes` separates Git layers and declared source/generated owners. `pse inspect affected` supplies conservative test reach. Use these to narrow investigation, then select real tests; reach is not test success. |

For an outline continuation, pass `payload.page.next_cursor` unchanged as `--cursor`; changed
working bytes refuse. Keep the same `--detail` on continuation. `detail_coverage` describes
selected fact ranges and file-level inclusion; omitted detail does not establish absent facts.
Unrequested fact mappings are `null`; a requested empty mapping is `[]`.
`--fields` projects output independently of detail demand. Source parsing has an explicit
edition/language policy rather than an
inferred build context. A syntax match is a written candidate even when names are spelled alike;
it does not establish resolved identity or active cfg membership. Semantic refinement explicitly
selects a prepared provider; unavailable information never silently becomes syntax evidence.

Allowances count nested declarations or matches. Review total/omitted and continuation facts
before concluding a search is complete. `--fields` can hide payload detail while mandatory
`providers` still records exact source generation and parser context. Read the selected
declaration or capture rather than dumping a large file solely to locate it.

For query correction, start with `search --request FILE --explain`: it shows resolved operands
and their contributing input origins without filesystem preflight or provider preparation.
Request-file paths are relative to that file. Missing explicit roots refuse before Begin and
may return a safe passive-explain recovery action. Execute returned argv unchanged when useful;
do not reconstruct hidden/redacted values. A later disappearance or stale prepared world has
different premises and may require a different action.

Consume both output streams. Command refusals appear on stderr; admitted-query failures appear
as stdout diagnostics. In JSON inspect `payload.summary` and `payload.diagnostics` even when
top-level `failures` is empty. In NDJSON consume diagnostic records and require terminal coverage.
Join each result's `source` with `payload.sources[].id`. Empty projected context text can coexist
with valid source/relationship evidence. Direct `inspect --source` reads current selected bytes;
`inspect --result` guards the original source/context and refuses stale evidence.

## Resolved Rust context and edit proposals

Prepare a reusable analyzer only when resolving Rust meaning can change the next decision:

```bash
pse code session prepare --project PATH
pse inspect entity --view explain FILE --line N --column N
```

Select features, default-feature policy and platform during preparation. Restricted preparation
disables automatic compiler checking, build scripts and proc macros. `--build-macros` explicitly
runs correctness-admitted `check --workspace --all-targets` preparation for those outputs; choose
it when the needed semantics depend on them. Queries do not enable checking or rebuild inputs.
Native snapshots use `--backend native-snapshot` and an explicitly prepared
`pse tools prepare native-analysis` image. Select `--test-context` for native cfg(test) semantics;
the default native world excludes test-only declarations. This selects a world, not test execution.

Choose one backend during explicit preparation. Native snapshot selection uses the explicitly
prepared native-analysis helper; consult actual readiness and supported capabilities rather
than assuming every LSP view is implemented there. HIR and compiler evidence retain separate
contexts. Portable semantic references bind the original session owner as well as source,
generation and lazy candidate fingerprint; a restarted generation counter is insufficient.

Semantic answers expose consumed facts under `observed`: symbols, source-linked relationships,
three-valued predicates, importability and guarded lazy candidates. `native_types` retains
original/adjusted type observations from the snapshot backend. Original provider documents in
`sections` remain available for inspection; use the typed fields for decisions and the returned
`followups` for expansion. The ordinary CLI can read these facts without loading the HIR backend.
Request-local grouped questions reuse their selected source/provider contexts and established
proofs while retaining the original use-site and source guards.

| Question | Selected capability |
|---|---|
| What entity/type does this expression name? | `inspect entity --view explain` requests hover, definitions and type resolution. Rendered hover remains rendered evidence when a structured field is unsupported. |
| Why does this operation capture, borrow or adjust a value? | Native explain/reference facts distinguish operation roles, implicit targets, original/adjusted types and capture/use causes. Keep separate use sites even when they resolve to one declaration. Unavailable native observations stay unavailable. |
| What layout does the selected target establish? | `inspect entity --view layout` returns fallible full extents only with native Sized evidence; unsized prefixes and unsupported generic/child layouts remain separate. `--compiler-layout INVOCATION` passively associates original type-size evidence through recorded guards; opaque compiler output supplies no numerical ABI equality. |
| What symbols does the semantic provider see in this file? | `inspect entity --view symbols`; prefer source-only `inspect outline` when written declarations suffice. |
| Where is this actual entity referenced? | `inspect entity --view refs`, including declarations; use it when text matches would conflate same-spelled names. |
| What calls this function, and what does it call? | `inspect entity --view calls` supplies one-hop incoming/outgoing calls. Expand only the relevant neighbors; this is not a whole-program runtime trace. |
| Where are native implementation targets? | `inspect entity --view implementations` locates returned targets. It does not establish that every generic trait obligation holds. |
| What did this macro expand to? | `inspect entity --view macro` returns selected expansion text with invocation/provenance limits. Prepared macro inputs may be required. |
| Which validation is relevant here? | `inspect entity --view tests` distinguishes file-native runnables from related tests, with native cfg, Cargo-target and snapshot-update intent where available. Follow guarded candidates through `pse tests` / `pse test`; discovery does not establish execution or coverage. |
| Which APIs fit this editing position? | `inspect entity --view candidates`, then its guarded `inspect entity --view candidate` action for one useful choice. Native declaration signatures remain separate from actual call-site inferred substitutions, receiver adjustments and supported obligations. Hypothetical calls retain unknown inference. Lazy resolution retains imports, snippets and unresolved editing intent. |
| What inferred annotations help explain these lines? | `inspect entity --view hints` with a selected line/range, then guarded `inspect entity --view hint` resolution when needed. Avoid fetching the whole file's annotations. |
| Can the native provider establish a specific trait predicate here? | `inspect entity --view predicate FILE --line N --column N --text TEXT`. Preserve holds, not-proven, invalid and unsupported outcomes. Not-proven remains unknown under conjunction or negation. |

Commands operate on working files, without assuming an editor's unsaved buffers. CLI positions
are one-based Unicode scalar coordinates, converted to the provider's negotiated encoding.
Use `--end-line` for selected hint/SSR ranges. Follow returned `--if-source`, generation,
configuration and candidate guards intact. Candidate indices are positions, not identities;
lazy actions also bind the original generation and native-record fingerprint.

Use edit previews when resolved identity or native assists can reduce a refactor's uncertainty:

| Intent | Proposal capability |
|---|---|
| Rename one resolved entity | `propose rename FILE ... --name NAME` |
| Rename a file and inspect required semantic follow-up | `propose file-rename FILE --to PATH` |
| Discover useful transformations at this position | `inspect entity --view assists`, then one guarded `propose assist --select INDEX` |
| Rewrite with the provider's semantic structural search | `propose ssr FILE ... --rule 'PATTERN ==>> REPLACEMENT'`, scoped to selected lines |

Inspect original/replacement regions, file operations, placeholders and provider context.
These are immutable checked proposals; no edit command applies changes or establishes that a
multi-file edit compiles. Keep unsupported native fields unavailable instead of inferring a
type, capture model or guarantee from display text.

`code session status` observes original readiness and ownership. `code session stop` settles
the original service and descendants. A canceled query leaves the session reusable. Changed
Cargo, toolchain or prepared macro inputs require explicit stop and preparation. Passive
status/stop remain available even when current host source cannot compile. An unavailable
semantic answer is a reason to inspect provider context or select another explicit method,
not to silently label a text match semantic evidence.

An explicit cancellation retains the receive path through the operation's terminal result.
Failures distinguish schema, constraint, context, validator, provider, transport and settlement
phases. `observed_cancellation` records actual CPU settlement or live-LSP return/detachment,
alongside the original work outcome. A missing terminal leaves settlement unobserved to the
caller. Live-LSP detachment does not prove completion inside rust-analyzer. A helper prepared
with the replaced protocol must be stopped and explicitly prepared again; original native
status and stop authority remain available.

For component fixtures that deliberately prepare independent persistent semantic sessions,
use the existing ordinary-application-scope fixture when selecting them through `pse test`.
Direct fixtures without that adaptation retain the qualification guide's top-level `cargo t`
route. Independent detachment directly from an enclosing workload continues to refuse.

## Locked APIs and dependency context

Use `pse inspect api` for the exact locked dependency context before designing against remembered APIs.
Context7 remains the first external library-documentation lookup under repository policy;
the acquired local API answers a distinct question about the pinned source and extraction context.

```bash
pse api PACKAGE --prepare
pse inspect api PACKAGE --search TERM --budget 3
pse inspect api PACKAGE ITEM_PATH --docs --examples --declaration
```

Acquisition explicitly compiles documentation in an isolated context; ordinary lookup never
compiles. Use an unambiguous locked name, `name@version` or source-qualified package ID.
`--target` selects the extraction target. Search or item paths narrow the answer; `--kind`,
`--input`, `--output-type`, `--trait-name`, `--unsafe-only`, `--async-only` and `--const-only`
help locate a capability by its signature or role. Confirm option spelling through help.

Expand `--docs` only for useful candidates. `--examples` includes authored examples, hidden
setup and native expectations; it executes none of them. `--declaration` exposes original
format-qualified item data when a compact signature omits complex bounds or associated data.
A facade can require explicit acquisition of its defining package. A missing artifact is
unprepared, not an empty API; unresolved references and automatic/blanket impl omissions
remain visible. Returned APIs do not alone establish ordinary-build applicability.
Rich contracts retain generic binders, associated constraints, attributes and stability facts.
`pse api PACKAGE --extract-examples` explicitly obtains rustdoc's transformed doctest inputs;
`inspect api PACKAGE --native-examples FILE` reads that original format-qualified output.
Wrappers, hidden setup and not-run modes remain visible; extraction executes no example.

To inspect an upgrade's structural changes, select the original acquired input explicitly:

```bash
pse inspect api PACKAGE --compare-with /original/acquisition/document.json --budget 10
```

Follow the returned cursor unchanged. Comparison reads both complete artifacts and requires
the backend's lossless typed decoder round trip; unsupported newer declarations remain
unavailable. Changed original bytes, extraction context or backend policy refuse continuation.
Structural changes alone are not a SemVer or behavioral compatibility verdict.

`pse api PACKAGE --compare-with ORIGINAL --compatibility` explicitly executes installed native rules over
the two original captures. Prepare that optional tool with
`pse diagnostic-tools install cargo-semver-checks`. It neither generates a baseline nor updates
manifests/locks; inspect original findings, exit status, raw streams and differing premises.
`pse api PACKAGE --check-docs` separately runs workspace rustdoc lints with original rule/source
addresses; an ordinary lookup requires neither this check nor an analyzer session.

For dependency questions, distinguish declarations, resolved metadata and compiler instances:

| Question | Capability and boundary |
|---|---|
| Why is this package included? | `pse inspect dependency PACKAGE --why` returns shortest metadata inclusion paths. |
| Which authored feature links request this member? | `inspect dependency --feature-member VALUE` follows exact authored expansion links. Optionality, default requests and target expressions remain distinct from Cargo's resolved feature union. |
| Which actual target/host/build units does this selection admit? | `pse inspect dependency-units DEPENDENCY ROOT --lib ...` uses the shared check grammar, including multiple roots, features and negative features. This is compiler-unit context, not compilation, provider readiness, a test population or impact proof. |

Add `--native-world` to dependency-unit inspection to compare actual prepared-world roots,
features, roles, cfg and edition. Ambiguous/missing correspondences remain explicit.
`--metadata-original` retains the separately captured metadata instead of inferring it from units.

For a concrete unresolved API question, `pse probe new NAME --deps PACKAGE` creates an authored
isolated workspace seeded with locked dependencies. Edit its source to state the experiment;
`pse probe run NAME` explicitly resolves/runs it through correctness admission. Printed API
examples are not experiments. Preserve the probe's actual source and output conditions when
using its result.

## Original failures and new compiler investigations

Read original evidence first when a check already failed:

```bash
pse inspect diagnostic INVOCATION
```

Follow the addressed `inspect diagnostic` action for selected detail. `--source` compares
retained native snippets with current source; `--raw` retains exact Cargo JSON. Related-span
pages preserve secondary locations, child notes, suggested replacements/applicability and
macro context. These reads never compile. Matching visible lines does not establish original
whole-source identity. A returned `inspect entity --view explain` action is guarded against the newly observed
whole-source digest and uses a separately prepared current session; its answer belongs to
current semantics, not retroactively to the original compilation.

Generate new evidence only for a concrete compiler question that source/API/semantic inspection
cannot settle. `--question` records that question without inferring an answer:

```bash
pse compiler generate PACKAGE --kind mir --body FUNCTION --pass ElaborateDrops --question 'remaining field drops'
```

| Kind | Useful question and evidence limits |
|---|---|
| `mir` | Inspect selected before/after MIR for a body/pass, such as remaining drops or lowered control flow. Body/pass selectors are native substrings; unsupported broad filters refuse. |
| `facts` | Explicit pinned compiler observations for an exact owning body and its nested bodies. `--fact-focus captures`, `suspension` or `cleanup` narrows native extraction before expensive queries. |
| `thir` | Inspect typed lowering of the selected unit. Native rustc offers no body filter for this view. |
| `type-sizes` | Inspect encountered concrete layouts under the selected compiler/target; no cross-target ABI guarantee. |
| `mono-items` | Inspect collected monomorphizations; collection is distinct from runtime reach. |
| `evaluate` / `compiler probe` | Evaluate a deliberately authored isolated probe for concrete constants/layout observations; stdout meaning belongs to that source. |

The library is the default target; use `--bin NAME` when needed. Cargo features and `--target`
select actual admitted units. Unsupported body/pass narrowing refuses. Dump generation disables
incremental reuse and requests sccache no-cache within that explicit stage so side-effect files
are produced again. Files are opaque, pinned-compiler evidence, not host-inferred borrow/liveness
facts, runtime behavior or speed measurements. No component is installed implicitly.

Create/edit an authored `pse probe` before `pse compiler probe NAME --question QUESTION`.
The original probe owner retains its lease, graph, toolchain and output meaning; compiler
artifact kinds can also select that probe explicitly. Inspect `inspect compiler-artifacts INVOCATION`
for original context, native outcome and file index, then `inspect compiler-artifact INVOCATION INDEX
--length 4096` for a bounded range. The reader validates the historical whole-file SHA first,
scanning the complete file and stating that cost. Changed files refuse. Reads neither rerun
compilation nor prepare a structured compiler driver.

For typed questions, explicitly prepare `pse tools prepare compiler-analysis`, then use
`compiler generate PACKAGE --kind facts --body FUNCTION --fact-focus cleanup --question QUESTION`
or the same selection through `compiler probe NAME`. This optional stage requires the matching
rustc-dev toolchain and never links compiler-private crates into ordinary host construction.
`inspect compiler-facts INVOCATION --fact-focus cleanup --max-bytes 16384` reads only successfully
published original facts. `--body` selects one acquired body; whole bodies stay indivisible.
Source/stored capture types, conservative saved fields, native may-initialization, phases and
normal/unwind edges have different meanings. ABI termination differs from a panic during cleanup.
These facts establish neither general loan/liveness analysis nor actual runtime resource release.
Missing families require explicit new acquisition; passive reads never regenerate them. Follow
original artifact actions for full context and byte evidence when the compact answer is insufficient.

Use `probe rm` only when finished with the authored workspace. It takes the same lease,
authenticates recorded runs and requires original native drain; a terminal row or missing PID
alone does not establish safe disposal. Missing custody and unknown legacy captures remain
protected, including a still-running enclosing borrowed owner.

## Generated-source ownership

When source inspection hits a declared generated output, follow its returned `inspect origin`
action before planning an edit. The action binds the hit's original byte range/digest, reads
the existing contributor sidecar, exposes declared regeneration argv and offers an outline of
the known emitter. This helps find the authored declaration or generator that actually owns
the change. An unknown declaration location stays unknown.

Changed working bytes refuse the original-hit action. Origin drill-through neither edits
files nor regenerates output or starts an analyzer. Fix the authored owner and run the declared
generation route; repository generated-output protections still apply.
