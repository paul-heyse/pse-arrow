---
title: "Plan69: CLI library leverage"
status: proposed
date: 2026-11-07
doc_role: review
doc_owner: docs/plans/69-intent-first-cli-and-codex-workflows.md
doc_topics: [current-work, design, libraries, validation]
---

# Plan 69 CLI library leverage

## Judgment and baseline

**Proposed.** Amend Plan 69 around four existing boundaries: the clap command and catalog,
the resolved configuration, the owner diagnostic projection, and terminal presentation.
Two findings deserve early treatment. The CLI builds the entire command tree for early
error formatting, which can cancel the benefit of lazy parsing. Explicit JSON and NDJSON
currently draw progress on an attended stderr. Configuration acquisition and diagnostic
conversion also need sharper contracts before their use expands to product and schema actions.
The existing catalog extensions, Figment overlays, typed error envelope, and original result
owners give these changes a direct route into the plan.

Inspected main `ed8e3fce2`, the coordinator and 69a–69e, and the full working-copy library
guides, including all four uncommitted addenda. The coverage appendix records the snapshot
and dispositions. Applied core 3.4, its efficiency and test guidance, the pse-arrow binding
and process-simulator profile 1.5. This review covers CLI composition and materially affected
owners and consumers. Plan 66 owns scientific semantics and qualification, Plan 67 supplies
custody and fixtures, and Plan 68 owns version-specific database cutover. The guide examples
and Context7 results were used to locate questions. Exact released source controls version
and implementation claims, including where indexed development material disagrees.
The locked `pse-host` feature graph selects clap derive, env, string and unstable-ext,
Figment TOML and JSON, indicatif's default features, and miette derive and fancy-no-syscall.
Dynamic completion, Figment Env and Jail, and indicatif in-memory rendering are separate
feature or dependency choices for this host.
[Plan 69](../plans/69-intent-first-cli-and-codex-workflows.md) ·
[O01–O08 source review](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md) ·
[Selected standard](../design_review/design_principles/standard.toml)

The receiving plans cover [CLI contracts](../plans/69a-cli-contracts-configuration-and-presentation.md),
[workstation preparation](../plans/69b-codex-workstation-and-preparation.md),
[tests and results](../plans/69c-ordinary-testing-and-managed-results.md),
[product and data operations](../plans/69d-routine-product-and-data-operations.md), and
[source and live schema changes](../plans/69e-source-model-and-live-schema-operations.md).

<a id="I01"></a>
## I01 — Remove eager schema construction from the normal parse path

**Interface-checked.** Before `Cli::try_parse_from`, `presentation_prefix` constructs and
recursively builds a clap tree to distinguish outer `--format` from forwarded options.
The strict parser then constructs its own tree. The existing scoped catalog already has a
node-local traversal and a deliberate recursive route. Its test distinguishes selected
and unrelated deferred branches. Builder deferral and derive deferral can reduce the work
of parsing a selected command, while `build` intentionally materializes the complete tree.
Adding deferral at declarations alone leaves the earlier full construction in place.

**Proposed.** A01 and A03 should use strict clap parsing first. On success take the resolved
format from the typed command. On a usage refusal, invoke the presentation-prefix fallback
against the original OS-native argv, with explicit boundary controls. Preserve the special
boundary and hook refusal protocols, successful help/version exits, and envelope error codes.
Compare this with sharing an already-built parse tree. Error-only fallback avoids full
reflection on successful commands and keeps the strict parse authority. Do not use
`ignore_errors` as a route to effect admission. A bounded lexical fallback earns its place
only if the existing error-route scan is itself materially costly or too difficult to
verify. Prefer strict selected parsing over reimplementing the grammar with `clap_lex`.

For the commands that forward data, compare their existing trailing or `last` positional
grammar with `Arg::raw(true)` only where callers already supply or can deliberately migrate
to a required `--`. An explicit boundary can simplify recovery argv validation and avoid
delimiter or hyphen interpretation. Keep accepted invocation forms through the migration
unless a boundary change is expressly selected. Assert that child `--format`, a non-UTF-8
token and a comma-bearing token stay child data. Include aliases, short bundles, missing
values, and outer errors before and after the separator. After that change, compare eager
and deferred construction on one prepared lightweight command. Adopt selective deferral only
if the comparison demonstrates useful savings and equivalent scoped and full discovery,
schema, help and any selected completion. B02's host-absent bootstrap remains an independent
entry. AP-05 and AP-07 favor one strict parse and construction where a consumer needs it.
[Current CLI](../../crates/pse-host/src/cli.rs) · [Catalog tests](../../crates/pse-host/src/catalog/tests.rs) ·
[clap 4.6.7 Command](https://docs.rs/clap/4.6.7/clap/struct.Command.html) ·
[clap Arg](https://docs.rs/clap/4.6.7/clap/struct.Arg.html)

<a id="I02"></a>
## I02 — Extend the catalog already attached to the parser

**Interface-checked.** The catalog uses `CommandExt` for examples, schemas and invocation
routes, and `ArgExt` for candidate providers and hidden defaults. Its reflection reports
which conditions and typed domains it cannot see. Candidate providers can distinguish
unavailable inputs and incomplete knowledge. The performing command independently owns
`generation_intent`, including actions whose intent depends on parsed inputs.

**Proposed.** In A01, project effect and preparation classification from the same typed
declaration the performing command uses for static intent. Keep the existing catalog
metadata as the association point. For input-dependent effects, publish a conditional or
unknown classification and check representative parsed cases against the real intent.
Keep runtime admission with the performing command. A second editable command registry
would duplicate syntax and drift. Using descriptive metadata for admission would lose
input-dependent and runtime premises. Contain the existing experimental extension API
behind the catalog abstraction and preserve its external contract if the upstream
extension interface changes.

Use derived groups, local clap constraints, typed value parsers and occurrence-preserving
reads in C01, D01–D03 and E01–E03 where they replace a concrete duplicate syntax check.
Owner checks still decide target identity, compatibility and authorization. Attach
`PossibleValue` descriptions to finite enums and the existing lazy providers to selected
dynamic domains. For example, the passive package candidate query on `inspect test-selection`
currently reports an unknown domain. Check which package forms the receiver accepts before
attaching the workspace or dependency provider. A scoped list can be useful as a partial
domain when it says so. Test aliases, required and overlapping groups, defaults and explicit
values against actual parse results. Use `help_expected` with `debug_assert` as a targeted
development control if the selected subtree supports it, and keep example parsing as
the stronger check. Expose genuine reflection gaps instead of reverse-parsing usage strings.

Keep agent discovery on the current catalog. Static completion or man pages can reuse the
built tree when a shell or documentation consumer is identified. Prefer static completion
first. Consider dynamic candidates only if a real selection remains burdensome and the
existing passive provider can meet it without preparing work or disclosing private
identities. The experiment must preserve canonical command paths and bounded candidate
semantics. AP-01, AP-05 and DP-01 separate parse syntax, discoverable facts and authority.
[Catalog adapter](../../scripts/cli-catalog.rs) · [Candidate owners](../../crates/pse-host/src/catalog.rs) ·
[clap completion 4.6.11](https://docs.rs/clap_complete/4.6.11/clap_complete/)

<a id="I03"></a>
## I03 — Acquire settings once under the selected owner

**Interface-checked.** The investigation resolver already layers typed defaults, checkout
settings, named scope and request, then explicit overrides. It resets changed tagged
alternatives and resolves paths per source. Figment `Data::file` discovers a candidate
at construction and reads during composition. An absent discovery stays absent even if
the file later appears. `file_exact` holds the supplied path and attempts the read during
composition. It returns an error for an unavailable explicit file.

**Proposed.** In A02, define acquisition alongside merge policy. The selected checkout
owner locates its optional settings under the approved root. A caller-selected file
uses a checkout-anchored absolute path and exact read. Resolve defaults, selected
profile and explicit clap overrides through the same typed operation for explain and
execution. Preserve existing owner-specific path semantics and whole-value replacement
of alternatives. Reuse the current resolver pattern for B02, D01 and E03 only where
they genuinely share layered inputs. For an effect or preview crossing a process boundary,
pass the validated selection and its meaningful identity or revalidate the premise
under the owner. A passive explanation alone does not bind a later effect.

Compare the current tagged provider and Figment `Serialized` with direct typed config for
single-source settings. Keep native env and OS paths byte-exact. Admit Figment's structured
`Env` only for a named allowlisted preference with a reversible enough origin contract,
and reserve Global for a deliberately higher-priority setting. Check wrong cwd, missing
explicit file, malformed optional file, changed preview premise, mixed profiles and
explicit versus parser-default inputs. AP-04, DP-09 and H24 protect identity and remove
repeated reads without carrying stale authority.
[Current resolver](../../crates/pse-host/src/investigation/config.rs) ·
[Figment 0.10.19 file provider](https://docs.rs/figment/0.10.19/figment/providers/struct.Data.html)

<a id="I04"></a>
## I04 — Report effective provenance at the field where it is true

**Tested.** A recursively merged object can have a different tag from one of its leaves.
An appended array keeps its original outer tag while appended elements keep their
own. Numeric dotted indices do not traverse arrays. A custom profile name survives
in the selected profile, while the value's tag does not recover that name. Global
has an explicit tag category. These details matter if A02 explains values through
container metadata or relies on profile tags to name a source.

**Proposed.** Keep Plan 69's replacement policy for ordinary arrays. Derive a redacted
effective explanation from the same resolved values used for execution. Expose origin at
the leaf or whole replacement value. If a real consumer later requires appended lists,
walk typed elements or label the container mixed. Preserve selected profile separately
and capture an originating profile name at the provider when needed. Use `find_value`
before interpreting a missing `find_metadata` as an absent field. Use `Tagged<T>` and
`RelativePathBuf` only where the value needs that metadata downstream. Distinguish
source-native display from reconstructing input spellings. For open-key maps or arrays,
address by typed segments or traverse values when the consumer needs exact identity.

Compare this with provider inventory or a separately generated settings dump. Neither
establishes the winning value. Exercise mixed leaves, explicit override, dotted keys,
absent versus invalid sources, and a secret that cannot appear in explanation or recovery.
Where Figment errors lack reliable source spans, give provider and key information
instead of guessing coordinates. AP-04, DP-02 and H16 favor truthful field evidence
over storing full configuration copies.
[Figment value lookup](https://docs.rs/figment/0.10.19/figment/struct.Figment.html) ·
[Profile implementation](https://docs.rs/crate/figment/0.10.19/source/src/profile.rs)

<a id="I05"></a>
## I05 — Carry the owner diagnostic through the CLI boundary

**Interface-checked.** The current envelope already captures ordinary cause messages,
diagnostic-source delegation, related findings, source coordinates, clap context and
typed recovery. Its test fixtures cover inherited source and byte positions. The human
route renders miette separately. **Tested.** At the pinned release, `Report::new` kept
a diagnostic's code and downcast identity, while `IntoDiagnostic` erased the code
when given the same typed diagnostic through the generic error interface.

**Proposed.** In A03, define which typed fields must cross each B–E adapter before a
diagnostic is rendered. Pass the original owner code, structured facts, retry unit, and
redacted source reference or excerpt. Use contextual thiserror variants where effect
identity matters, and `#[from]` or transparent forwarding where the underlying meaning
already suffices. Prefer the existing typed projection over converting to strings or
recapturing remote failures as unrelated local reports. Use `Report::new` or `new_boxed`
for already diagnostic values at a report boundary. Use ordinary error adapters where
only the ordinary error interface exists. `WrapErr` can enrich narrative context but
cannot supply missing typed facts.

For source-change preview and schema refusal, use primary and secondary labels for
one source and related diagnostics with separately identified source for distinct
files. `NamedSource`, shared source or a bounded provider is useful only when the
owner can preserve exact byte coordinates and redact safely. Keep heterogeneous
`Vec<Report>` at composition boundaries where failures really differ. Keep concrete
findings for structured owner results with one schema. Use optional labels and help
instead of synthetic spans or unsafe argv. JSON rendering and `MietteDiagnostic`
serde can support local diagnostics and tests. The native envelope remains the machine
contract for failure identity, verdict, settlement, redaction and recovery.

Exercise usage, missing preparation, ambiguous production target, interrupted apply,
and multi-file validation through their actual receiving adapters. Assert exact original
identity, refusal category, bounded source lifetime, and independent disposition of warnings.
Include a Unicode span and a child without source. Keep related findings, ordinary
causes and delegated diagnostics distinguishable. AP-02, DP-02 and H24 protect
stable recovery across process boundaries.
[Current projection](../../crates/pse-host/src/envelope/diagnostic.rs) ·
[miette 7.6.0 Report](https://docs.rs/miette/7.6.0/miette/struct.Report.html) ·
[miette Diagnostic](https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html) ·
[thiserror 2.0.21](https://docs.rs/thiserror/2.0.21/thiserror/)

<a id="I06"></a>
## I06 — Make output format control the terminal surface

**Tested.** On the prepared CLI, an 80-match local text search emitted valid JSON and
NDJSON on stdout. With stderr connected to an xterm PTY, both emitted progress glyphs
and ANSI control bytes. With stderr piped, neither emitted progress. The investigation
renderer chooses its draw target solely from stderr terminal detection. It uses an
unknown length with the default bar style, which displayed `0/0` during the probe.

**Proposed.** Make the current search renderer the first A04 migration. Decide
presentation from the resolved format before constructing progress. JSON and NDJSON
get no interactive renderer regardless of stderr. Attended human mode may use stderr
with an explicit spinner for unknown totals. Unattended human mode can use occasional
owner phase lines where they help. Reuse that format decision for A03's graphical or
narratable diagnostics and explicit color policy. Keep structured logging independent.
Prefer local render handlers to global hooks so embedded and machine routes can select
their own output. The existing human handler separation is a good foundation.

Use a PTY control with stdout still piped and check stdout frames and stderr separately.
For human mode, check no-color, narrow terminals, Unicode and interleaved logs where
supported. For machine modes, assert no progress or terminal escapes on either stream
and preserve the structured error contract. For a broken terminal, preserve the
owner's outcome and apply existing delivery-failure semantics. AP-02 and AP-06
make presentation behavior independently testable.
[Search renderer](../../crates/pse-host/src/cli/investigation.rs) ·
[Output format](../../crates/pse-host/src/envelope/presentation.rs) ·
[Indicatif draw targets](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressDrawTarget.html)

<a id="I07"></a>
## I07 — Bind progress lifetime to owner events

**Tested.** Indicatif's buffered wrapper counted bytes when consumed, and its
write wrapper reached its numerical total without setting finished. With
`InMemoryTerm`, `MultiProgress::clear` erased the displayed line, a later draw
showed it again, and `remove` hid the detached bar. Source inspection also found
that removing a bar owned by another group asserts, and that last-handle drop
can trigger finish behavior. These are presentation lifecycle events.

**Proposed.** Give A04 receiving routes a small mapping from existing owner events
to phase, optional known total, and terminal presentation. Start with one bar or
phase lines. Register multiple bars only for a named concurrent workflow where
the operator needs to distinguish states, and give one renderer membership and
retirement responsibility. Choose finish, abandon or clear on the appropriate
owner transition. Avoid background steady ticks unless unattended work needs
animation and stop them when the display ends. Reuse the test harness for stream
purity. Consider `InMemoryTerm` for actual layout and removal assertions if multi-bar
behavior is selected.

Position updates and draw throttling are distinct in the pinned implementation.
Prefer existing coarsened owner events or count updates over reformatting a message
for every item. Adapter instrumentation is a candidate for a concrete I/O or
iteration boundary, such as D03 export bytes accepted, where the unit is truthful
and removing custom counters has value. Keep flush, durable publication and original
settlement with their owners. Suppress an ETA that represents no useful estimate.
Trace discovery, known total, failure after a full count, cancellation, and
settlement. Compare current and proposed display costs only if claiming a runtime
improvement. AP-07, H27, PS-10 and PS-12 preserve the difference between an
observation, presentation state and an accepted result.
[Indicatif state](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressState.html) ·
[MultiProgress](https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html) ·
[Adapters](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressBarIter.html)

<a id="I08"></a>
## I08 — Prove shared behavior through owner handoffs

**Proposed.** Qualify the shared changes through the six coordinator journeys.
Bring the inexpensive passive catalog, config and output controls forward into
A01–A04. Add one owning test per receiving boundary. Do not duplicate parser,
renderer and owner assertions in every integration case. The existing Plan 69
journeys carry the following additional observations.

| Journey | Added acceptance |
|---|---|
| C01–C02 ordinary tests and rerun | Catalog explains syntax and selection separately. Owner reports admitted population and original verdict. Failed rerun preserves the exact failure identity and forwarded filters. |
| C03–C04 result lookup and release | A bounded detail view retains diagnostic relationships and trustworthy provenance where it was captured. Held, retired and unknown remain distinct. Release stays with the owner. |
| D01–D03 product targeting and exchange | Ambiguous context refuses before a write. A resolved selection and original acknowledgement survive dispatch. Displayed export counts describe only the observed stage. |
| E01–E02 source change | The supported operation yields primary and related findings with correct file and byte coordinates. Stale source refuses before edits and partial publication retains the exact recovery unit. |
| E03 live schema | Stale or uncertain input stays distinguishable from completed activation in CLI and machine output. An exact retry uses the original operation identity and owner facts. |
| B01–B03 cold checkout | Help and catalog stay passive. Missing-host output points to an explicit prepared route. A child argument or config preference cannot silently change the original execution mode. |

Use isolated local fixtures for terminal, configuration and parser cases. Reuse existing
owner evidence for admission, custody and scientific checks where the premise matches.
The new guarantees require receiving controls only where the CLI crossing can lose
meaning. AP-06 and the test-assessment companion favor discriminating tests with
small setup. This extends acceptance without moving Plan 66 or 68 qualification into
the CLI review. [Plan 69 acceptance](../plans/69-intent-first-cli-and-codex-workflows.md#verification-and-completion) ·
[Test assessment](../design_review/design_principles/core/test-assessment.md)

## Integration order and decisions

Treat I06's reproduced format defect and I05's typed projection as early shared work.
Agree I01's error-path contract before attempting deferral. I02 can extend the catalog
without waiting for that experiment. Complete I03 before relying on I04's attribution.
Integrate I07 incrementally where a receiving owner already reports the required
events. Run I08's receiving controls as those packages land. Static completion, dynamic
completion, additional feature flags and richer terminal modes remain conditional on
their consumers. None of the recommended amendments requires a binding rule change.
If a proposal later changes a public argument boundary or a governed decision, record
its compatibility impact and route it through the existing owner before adoption.
These are proposed plan amendments. Plan 69 owns any later scheduled dispositions.

## Evidence and limits

| Evidence | Scope and result |
|---|---|
| **Interface-checked.** Prepared `scripts/pse commands --format json inspect invocation`, candidate inspection for `package` on `inspect test-selection`, and `scripts/pse --format json inspect invocation --definitely-unknown` | One command record, an explicit unknown candidate domain, and the expected structured usage refusal with `UnknownArgument`. Candidate availability is not equivalent to an empty accepted domain. |
| **Tested.** `cargo test --offline --test contracts --quiet`, isolated temporary Cargo package, pinned checkout toolchain, exact library versions in this review | Five controls passed against a zero-failure baseline. They exercised deferred build and raw argv, nested/array/profile tags, file acquisition, typed report conversion and raw source inference, and buffered read and write accounting. One terminal assertion failed because its default style did not print the message. The separate corrected `cargo test --offline --test clear --quiet` used an explicit `{msg}` template and passed with zero failures. The initial probe also corrected its assumption that `Data<Toml>` could be cloned by passing a borrowed provider. |
| **Tested.** Prepared `scripts/pse search text pse-cli-probe --literal --path <fixture> --all --format <mode>`, local 80-line fixture, stdout piped, stderr piped or xterm PTY | All four invocations exited zero. JSON produced one object and NDJSON produced 83 frames in both conditions. Both PTY runs emitted 711 bytes of progress on stderr against a zero-byte machine-progress expectation. Both pipe runs emitted zero. |

The probe used exact direct library releases and enabled indicatif's `in_memory` only
in the isolated terminal control. It did not qualify the proposed Plan 69 changes.
Source observations and targeted tests establish contracts in the stated conditions.
No comparative latency or throughput result is claimed. The existing Codex log review
supplies workflow context and no new log sample was needed.

## Appendix — Guide coverage and dispositions

The base and addendum section numbers are local to each guide. Cross-references group
capabilities that address one intervention. Conditional entries name the consumer or
evidence that would make the feature worthwhile. Companion crates were screened as
separate integrations, with no assumed compatibility or feature activation.

| Working-copy input | SHA-256 |
|---|---|
| [clap](../libraries/cli/clap.md) | `ad666dbca8d6fed0256596bd6389816637244f220c6515015d16331d2c29d1bc` |
| [Figment](../libraries/cli/figment.md) | `caf48c6e4356ceb304bdbfbb51446b84bacc553a439f7407dd8524eb6bb7b63a` |
| [indicatif](../libraries/cli/indicateif.md) | `410999fa916d5f93d48b0ec1aa6b7cf2513ed0d28000871e2d69092f3ba777bc` |
| [thiserror and miette](../libraries/cli/thiserror_and_miette.md) | `27a13a7438f581eb55452c13726a0aa7e577b192b1ca4a63fa729a42a9145cdd` |

### clap 4.6.7

| Guide coverage | Disposition |
|---|---|
| Base §§1–3 and addendum §§1, 4, 6, 9.3. Derive and builder, schema mutation, flattening, global fields, reusable and overlapping groups, macros, aliases, reflection and typed extensions. | Extend the existing parser-backed catalog, with verified local constraints and truthful reflection gaps. Use derive for fixed typed operations and builder augmentation where a real consumer shares it. `arg!` and `command!` earn use only if they simplify a builder declaration. I01–I02. |
| Base §§4–7 and addendum §§2, 5, 6. Arg actions, cardinality, typed functions, custom parser and factory, finite domains, case folding and conditional checks, OS-native values, occurrences, values and defaults, env and provenance. | Prefer typed local parsers, groups and finite metadata in test, product and model/schema commands. Preserve raw or grouped inputs where the receiving owner needs them. Explicitly test the effects of value names, aliases, conditional defaults and overrides on real grammar. Use `value_source` for explicit configuration overlays. I01–I04. |
| Base §§8, 11 and addendum §§2–3, 6, 9.1–9.2. Hierarchies, implicit abbreviations, globals and aliases, raw and trailing capture, external subcommands, multicall, fallible and consuming matches, parser reuse and update, `ignore_errors`, `no_binary_name`, `clap_lex`. | Preserve strict effect parsing and OS-native tail boundaries. Prefer unambiguous full names for agents. Consider owned matches and parser reuse where a receiving adapter consumes them. Multicall and external dispatch need a named deployment or plugin contract. Tolerant parsing and a lower-level lexer have a possible passive or error-only use if that specific boundary needs them. I01–I02. |
| Base §§9, 11 and addendum §7. Generated help/version, overrides and usage reflection, names, help styles, terminal controls, error context and validation. | Use existing catalog and structured clap projection. Consider subtree `help_expected` and `debug_assert`, with example parity and bounded scope. Preserve actual usage and distinct help/version behavior. Unify policy with human diagnostics and machine purity. I01–I02, I05–I06. |
| Base §10 and addendum §8. Static and dynamic completion, man pages, candidate builders, `CompleteEnv`, direct completion, value hints, path providers and shell-specific behavior. | Existing passive catalog meets agent discovery. Add generated shell or manual artifacts for a named human consumer. A dynamic candidate must improve on static and reuse passive, authorized candidate sources. Check command-wrapper hints on the actual shell. I02. |
| Base §§12–14 and addendum §§10–11. Dependency and feature options, companion crates, build identity, color, localization, portability and performance. | Preserve the current feature set initially. Consider unified color if I06 needs an adapter, long version from existing generation identity if B02 needs it, and a separate dependency only for a named unmet capability. No current consumer for alternate multicall packaging, localization, or additional platform claims. Treat `unstable-ext`, v5 behavior and dynamic completion as separate stability commitments. I01–I02, I06. |

### Figment 0.10.19

| Guide coverage | Disposition |
|---|---|
| Base §§1–4, 9 and addendum §§1, 7, 10. Provider graphs and formats, all merge modes, profiles, profile helpers, nested data, file and string providers, YAML merge keys, serialization and release differences. | Reuse typed defaults, current tagged provider, strict overlays and exact file acquisition where inputs are layered. Keep simple settings local. Preserve replacement arrays and reset tagged alternatives. Conditional on actual consumer needs for profile nesting, custom formats, YAML, and composed providers. `required()` and `search()` examples are outside the published pin. I03. |
| Base §§5, 10 and addendum §6. Env prefixes, filters and transformations, structured values, raw iteration, case behavior, interpolation, concurrency, secret files and framework patterns. | Keep byte-exact native env. Consider allowlisted structured preferences only with an explicit collision and origin policy. Preserve the checkout and effect owners. Separate secret-file adapters and framework-style provider abstractions need a named integration and redaction contract. I03–I04. |
| Base §§6–8, 9.4 and addendum §§2–5, 8. Typed extraction and focus, tags, metadata, error chains, interpolation, paths, tagged values, magic values, addressability, profiles and numeric fidelity. | Use strict extraction and one redacted explanation from the actual effective values. Use file-aware path values or tags only where consumers retain the distinction. `focus` and `extract_inner` can bound subconfig consumers. Avoid lossy coercion at identity or domain boundaries. Use typed lookup when dotted addressing cannot express a key. Utility helpers and `Magic`/`Either` need an existing provider or typed value use. I03–I05. |
| Base §§11–13 and addendum §§9–11. Jail, default features, runtime and thread behavior, architecture summary. | Use pure provider tests first, existing isolated subprocesses for cwd/env controls, and `Jail` only if its restoration and path helpers improve a real test without conflicting with global state. Keep optional features narrow. I03–I04, I08. |

### thiserror 2.0.21 and miette 7.6.0

| Guide coverage | Disposition |
|---|---|
| Base §§1–2 and addendum §§1, 7. Derives, sources, conversion, transparent delegation, field inference, formatting functions and defaults, manual display, backtrace and feature limits. | Keep typed causes, contextual variants, and explicit delegation at owner crossings. The isolated raw-source control passed. Prefer descriptive source-content field names for combined derives. Shared formatting is useful where it removes actual duplication. Require a debugging consumer before adding backtrace or compiler coupling. I05. |
| Base §§3.1–3.6 and addendum §§2–3. Protocol and derive, dynamic or optional help and labels, related findings, primary and collection labels, source providers and inheritance, multi-source identity and location. | Preserve source relationships and bounded content at the projection. Use heterogeneous or dynamic findings only at real aggregation points. Select one coordinate space per source. Caller-location introspection has value only for an explicit developer diagnostic. I05, I08. |
| Base §§3.7–3.8, 6 and addendum §4. Reports, constructors, macros, conversions, context, interop and typed information loss. | Preserve typed owners through `Report::new` or a typed projection. Use generic conversion only for ordinary errors and keep macros and dynamic reports at boundaries. Assess `Deref`, `AsRef`, boxing and downcast before relying on generic error bounds. The envelope owns control flow and compatibility. I05. |
| Base §§4–5, 7–8 and addendum §§5–6, 8. Graphical, narratable, debug and JSON handlers, highlighting, hooks, handler customization and tracking, serialization and platform features. | Keep local rendering and share format policy. Test accessibility and controlled source rendering first. Custom handlers, global or panic hooks, syntax highlighting, `serde` and extra feature surfaces require a distinct consumer. Miette JSON is useful for local assertions, with owner semantics carried by the native envelope. I05–I06. |

### indicatif 0.18.6

| Guide coverage | Disposition |
|---|---|
| Base §§1–4 and addendum §§1–3, 8. Progress handles and state, known/unknown totals, weak handles, independent throttles, layout, drop, finish and abandon, zombie lines, remove versus clear, ETA and duration. | Use explicit format gating and a simple owner-event mapping. A shared renderer can own multiple bars when a named workload needs them. A last-handle drop, a zero ETA, or a numerical total cannot establish an owner verdict. Test removal and cancellation if the selected renderer owns those transitions. I06–I07. |
| Base §§5, 7–8 and addendum §§5–7. Draw targets, terminal abstraction, in-memory terminal, rendering failures and geometry, templates and keys, spinner state, ANSI slices, tab expansion, width and formatting. | Select only the human presentation and test mechanisms the first receiving journey needs. Consider custom tracking, visible clipping, glyphs or unit formatting for a named display need. Check machine purity with a PTY, and use `InMemoryTerm` when layout assertions justify it. I06–I07. |
| Base §6 and addendum §§4, 7. Iteration, Rayon, sync and async I/O, streams, trait preservation, shared bar configuration and accounting. | Consider a wrapper for a concrete unit such as D03 accepted bytes. Verify buffered consumption, partial operations, readiness, seek and EOF at that boundary. Prefer current coarsened owner events where they already express the useful unit. Optional parallel and async features need that consumer. I07. |
| Base §§9–12 and addendum §§9 and synthesis. Features, platforms, logging and tracing bridges, ecosystem and role boundaries. | Start with existing `console`/indicatif integration, explicit formatting policy and logging coordination. Evaluate a bridge only for demonstrated interleaving on a real route. Prompting, extra platform-specific behavior and alternate runtimes lack a current Plan 69 receiver. I06–I07. |
