# Code intelligence by question

Use the section that answers the current question. Exact grammar and result schemas come
from `pse help`, `pse commands` and `pse schema`; detailed preparation examples live in the
[environment guide](../../../../docs/dev/agent-environment.md#focused-command-and-source-context).
Examples below use operands such as `FILE`, `PACKAGE` and `INVOCATION` to replace deliberately.

- [Written source and structural changes](#written-source-and-structural-changes)
- [Resolved Rust context and edit proposals](#resolved-rust-context-and-edit-proposals)
- [Locked APIs and dependency context](#locked-apis-and-dependency-context)
- [Original failures and new compiler investigations](#original-failures-and-new-compiler-investigations)
- [Generated-source ownership](#generated-source-ownership)

## Written source and structural changes

Use source-only reads before preparing a semantic service when declaration structure or exact
written bytes will settle the question. These reads also work outside a pse-arrow checkout.

| Question | Capability and useful limits |
|---|---|
| Where is a declaration or document section? | `pse outline FILE --budget 20` reads nested Rust declarations or Markdown headings without Cargo, cfg evaluation or macro expansion. Multiple files are supported; use one file for a continuation. |
| What exactly surrounds this location? | `pse show FILE --at-line N --at-column N` returns the enclosing declaration/section. `--range-start` / `--range-end` select a half-open UTF-8 byte range. Line/column coordinates are one-based Unicode scalars. |
| What was staged or committed? | Add `--index` for the raw staged blob or `--revision REF` for a resolved committed blob. This avoids checkout changes and distinguishes Git bytes from current working bytes. |
| Which written shapes match a refactor? | `pse code query FILE --mode syntax --pattern 'call($ARG)'` returns structural candidates with captures and exact source identity. Rust is the current structural language. |
| Which matches occur in the intended context? | `--inside`, `--has` and `--exclude` constrain ancestors, proper descendants and excluded candidate shapes. `--strictness` selects matching policy; `--capture NAME` selects named or sequence captures without sigils. |
| What would a replacement change? | `code preview` uses the same pattern plus `--replacement` and optional `--select` candidate indices. Inspect candidates before selecting edits; selected overlaps and undefined template captures refuse. Neither source nor the Git index changes. |
| What changed, and which tests might matter? | `pse changes` separates Git layers and declared source/generated owners. `pse affected` supplies conservative test reach. Use these to narrow investigation, then select real tests; reach is not test success. |

For an outline continuation, pass `payload.page.next_cursor` unchanged as `--cursor`; changed
working bytes refuse. Source parsing has an explicit edition/language policy rather than an
inferred build context. A syntax match is a written candidate even when names are spelled alike;
it does not establish resolved identity or active cfg membership. `--mode semantic` structural
queries are currently unavailable and never fall back to syntax.

Allowances count nested declarations or matches. Review total/omitted and continuation facts
before concluding a search is complete. `--fields` can hide payload detail while mandatory
`providers` still records exact source generation and parser context. Read the selected
declaration or capture rather than dumping a large file solely to locate it.

## Resolved Rust context and edit proposals

Prepare a reusable analyzer only when resolving Rust meaning can change the next decision:

```bash
pse code session prepare --project PATH
pse code explain FILE --line N --column N
```

Select features, default-feature policy and platform during preparation. Restricted preparation
disables automatic compiler checking, build scripts and proc macros. `--build-macros` explicitly
runs correctness-admitted `check --workspace --all-targets` preparation for those outputs; choose
it when the needed semantics depend on them. Queries do not enable checking or rebuild inputs.

| Question | Selected capability |
|---|---|
| What entity/type does this expression name? | `code explain` requests hover, definitions and type resolution. Rendered hover remains rendered evidence when a structured field is unsupported. |
| What symbols does the semantic provider see in this file? | `code symbols`; prefer source-only `outline` when written declarations suffice. |
| Where is this actual entity referenced? | `code refs`, including declarations; use it when text matches would conflate same-spelled names. |
| What calls this function, and what does it call? | `code calls` supplies one-hop incoming/outgoing calls. Expand only the relevant neighbors; this is not a whole-program runtime trace. |
| Where are native implementation targets? | `code implementations` locates returned targets. It does not establish that every generic trait obligation holds. |
| What did this macro expand to? | `code macro` returns selected expansion text with invocation/provenance limits. Prepared macro inputs may be required. |
| Which tests are related to this entity? | `code tests` returns candidates with next actions through `pse tests` / `pse test`. It never executes analyzer-supplied argv. |
| Which APIs fit this editing position? | `code candidates`, then its guarded `code candidate` action for one useful choice. Lazy resolution retains imports, snippets and unresolved editing intent. |
| What inferred annotations help explain these lines? | `code hints` with a selected line/range, then guarded `code hint` resolution when needed. Avoid fetching the whole file's annotations. |
| Can the native provider establish a specific trait predicate here? | `code predicate FILE --line N --column N --text TEXT`. Preserve proven, contradictory and not-proven/unavailable outcomes; lack of proof is not a negative fact. |

Commands operate on working files, without assuming an editor's unsaved buffers. CLI positions
are one-based Unicode scalar coordinates, converted to the provider's negotiated encoding.
Use `--end-line` for selected hint/SSR ranges. Follow returned `--if-source`, generation,
configuration and candidate guards intact. Candidate indices are positions, not identities;
lazy actions also bind the original generation and native-record fingerprint.

Use edit previews when resolved identity or native assists can reduce a refactor's uncertainty:

| Intent | Proposal capability |
|---|---|
| Rename one resolved entity | `code edit rename FILE ... --name NAME` |
| Rename a file and inspect required semantic follow-up | `code edit file-rename FILE --to PATH` |
| Discover useful transformations at this position | `code edit assists`, then one guarded `code edit assist --select INDEX` |
| Rewrite with the provider's semantic structural search | `code edit ssr FILE ... --rule 'PATTERN ==>> REPLACEMENT'`, scoped to selected lines |

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

## Locked APIs and dependency context

Use `pse api` for the exact locked dependency context before designing against remembered APIs.
Context7 remains the first external library-documentation lookup under repository policy;
the acquired local API answers a distinct question about the pinned source and extraction context.

```bash
pse api PACKAGE --prepare
pse api PACKAGE --search TERM --budget 3
pse api PACKAGE ITEM_PATH --docs --examples --declaration
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

To inspect an upgrade's structural changes, select the original acquired input explicitly:

```bash
pse api PACKAGE --compare-with /original/acquisition/document.json --budget 10
```

Follow the returned cursor unchanged. Comparison reads both complete artifacts and requires
the backend's lossless typed decoder round trip; unsupported newer declarations remain
unavailable. Changed original bytes, extraction context or backend policy refuse continuation.
Structural changes alone are not a SemVer or behavioral compatibility verdict.

`--compatibility` explicitly executes separately installed native compatibility rules over
the two original captures. Prepare that optional tool with
`pse diagnostic-tools install cargo-semver-checks`. It neither generates a baseline nor updates
manifests/locks; inspect original findings, exit status, raw streams and differing premises.
`pse api PACKAGE --check-docs` separately runs workspace rustdoc lints with original rule/source
addresses; an ordinary lookup requires neither this check nor an analyzer session.

For dependency questions, distinguish declarations, resolved metadata and compiler instances:

| Question | Capability and boundary |
|---|---|
| Why is this package included? | `pse dep PACKAGE --why` returns shortest metadata inclusion paths. |
| Which authored feature links request this member? | `dep --feature-member VALUE` follows exact authored expansion links. Optionality, default requests and target expressions remain distinct from Cargo's resolved feature union. |
| Which actual target/host/build units does this selection admit? | `pse dep units DEPENDENCY ROOT --lib ...` uses the shared check grammar, including multiple roots, features and negative features. This is compiler-unit context, not compilation, provider readiness, a test population or impact proof. |

For a concrete unresolved API question, `pse probe new NAME --deps PACKAGE` creates an authored
isolated workspace seeded with locked dependencies. Edit its source to state the experiment;
`pse probe run NAME` explicitly resolves/runs it through correctness admission. Printed API
examples are not experiments. Preserve the probe's actual source and output conditions when
using its result.

## Original failures and new compiler investigations

Read original evidence first when a check already failed:

```bash
pse compiler diagnostics INVOCATION
```

Follow the addressed `compiler diagnostic` action for selected detail. `--source` compares
retained native snippets with current source; `--raw` retains exact Cargo JSON. Related-span
pages preserve secondary locations, child notes, suggested replacements/applicability and
macro context. These reads never compile. Matching visible lines does not establish original
whole-source identity. A returned `code explain` action is guarded against the newly observed
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
artifact kinds can also select that probe explicitly. Inspect `compiler artifacts INVOCATION`
for original context, native outcome and file index, then `compiler file INVOCATION INDEX
--length 4096` for a bounded range. The reader validates the historical whole-file SHA first,
scanning the complete file and stating that cost. Changed files refuse. Reads neither rerun
compilation nor prepare a structured compiler driver.

Use `probe rm` only when finished with the authored workspace. It takes the same lease,
authenticates recorded runs and requires original native drain; a terminal row or missing PID
alone does not establish safe disposal. Missing custody and unknown legacy captures remain
protected, including a still-running enclosing borrowed owner.

## Generated-source ownership

When source inspection hits a declared generated output, follow its returned `code origin`
action before planning an edit. The action binds the hit's original byte range/digest, reads
the existing contributor sidecar, exposes declared regeneration argv and offers an outline of
the known emitter. This helps find the authored declaration or generator that actually owns
the change. An unknown declaration location stays unknown.

Changed working bytes refuse the original-hit action. Origin drill-through neither edits
files nor regenerates output or starts an analyzer. Fix the authored owner and run the declared
generation route; repository generated-output protections still apply.
