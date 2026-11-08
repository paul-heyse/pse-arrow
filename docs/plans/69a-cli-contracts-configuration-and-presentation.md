---
title: "Plan69a: CLI catalog, configuration, and diagnostics"
status: complete
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69a-cli-contracts-configuration-and-presentation.md
doc_topics: [current-work, design, libraries]
---

# CLI catalog, configuration, and diagnostics

This companion owns [O01](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O01),
[O02](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O02), and
[O08](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O08). The
[coordinator](69-intent-first-cli-and-codex-workflows.md) owns shared decisions and assembled
verification. Extend the parser-derived catalog, investigation configuration and native
envelope where multiple receiving commands share the responsibility.
[69f](69f-parser-startup-and-argument-forwarding.md) owns parser startup and forwarding.
[69g](69g-output-policy-and-owner-driven-progress.md) owns A04, including local rendering.

## A01 — One parser tree for intent and discovery

Expose name, argument requirements, effect category, owner, availability hints and usable examples
from the parser and its catalog metadata. Reuse clap 4.6.7 `CommandFactory` to obtain the command
tree, `Args` and `Subcommand` to compose it, `ArgGroup` for exclusive forms, and `TypedValueParser`
for identifiers, limits and OS-native paths. Treat parse success as syntax validity. A performing
owner verifies the selected identity, authorization and current state. Keep static help,
`pse commands`, machine schema and any generated completion derived from that same tree.

[I02](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I02) extends
the current catalog. Clap's enabled `unstable-ext` feature lets `Command::add` and
`Arg::add` attach typed `CommandExt` and `ArgExt` metadata. The catalog retrieves it
with `get`. Reuse the existing adapters for examples, response schema, invocation
routes, candidate providers and secret defaults. Project static intent and preparation
from the performing command's typed declaration. Expose conditional or unknown intent
where parsed values matter, and verify representative cases against `generation_intent`.
The runtime keeps admission authority. Preserve the catalog's outward contract if the
experimental extension API changes.

For C01, D01–D03 and E01–E03, use derived groups and local clap requirements where they
replace existing syntax checks. Use `TypedValueParser` for local typed values, and
`ArgMatches::get_occurrences` or consuming equivalents where grouped values must
remain separate. Expose finite `ValueEnum` candidates with `PossibleValue` metadata.
For the passive package query on `inspect test-selection`, compare actual accepted
package forms with the existing provider's workspace or dependency coverage.
Attach that provider only for a useful subset with explicit scope and freshness.
Preserve unknown or unavailable coverage where the provider cannot answer.
[Catalog adapter](../../scripts/cli-catalog.rs) ·
[Command metadata](https://docs.rs/clap/4.6.7/clap/struct.Command.html#method.add) ·
[Argument metadata](https://docs.rs/clap/4.6.7/clap/struct.Arg.html#method.add) ·
[Occurrences](https://docs.rs/clap/4.6.7/clap/struct.ArgMatches.html#method.get_occurrences)

Start with the existing `pse inspect invocation` and `pse inspect log` entry points and correct
the environment guide's `pse result` spelling. Add focused detail through C03. Verify published
examples against parser acceptance and assert that discovery performs no preparation. Existing
Cargo, just and product helpers retain their owner roles. Remove obsolete advertised paths and
any duplicate first-party intent when its callers have moved. An arbitrary shell dispatcher
cannot supply the typed effect guarantees.

Keep example parsing and scoped versus full discovery controls. Use `help_expected`
with `debug_assert` as a targeted development check where a subtree can support it.
Keep the existing catalog for agent discovery. For a named shell or documentation
consumer, evaluate `clap_complete::generate_to` for static scripts or
`clap_mangen::Man::render` for man pages from a fully built tree. Consider
`clap_complete::engine::complete` or `CompleteEnv::try_complete` only if a real
selection remains cumbersome and can reuse authorized, passive, bounded candidates.
Those dynamic APIs require the companion crate's `unstable-dynamic` feature. Check
actual shell behavior and compatibility before proposing a companion pin. F02 owns
the deferred-construction experiment. [Catalog](../../crates/pse-host/src/catalog.rs) ·
[clap guide](../libraries/cli/clap.md) · [clap 4.6.7 derive](https://docs.rs/clap/4.6.7/clap/_derive/index.html) ·
[Typed value parser](https://docs.rs/clap/4.6.7/clap/builder/trait.TypedValueParser.html) ·
[Completion](https://docs.rs/clap_complete/4.6.11/clap_complete/) ·
[Manual generation](https://docs.rs/clap_mangen/0.3.3/clap_mangen/)

Receive [69c C05](69c-ordinary-testing-and-managed-results.md#c05--explain-test-selection-and-doctest-ownership)
through the existing parser catalog. A source-derived candidate, a Cargo-admitted target,
and a Nextest-enumerated test carry different availability. Doc-only source candidates
point to the existing rustdoc lane. Keep discovery passive and take candidate and admission
meaning from the selection owner.

## A02 — Resolved context with provenance

Resolve developer checkout and session identity from the existing owner. Product actions may
use a uniquely configured, authenticated target. Ambiguity returns valid authorized choices
before admission. Configuration can select a context or supply preferences. The typed action
and owner guards establish consent and permission for the effect.

[I03](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I03) adds
acquisition to the merge contract. The selected checkout owner discovers optional
settings under its approved root. Anchor a caller-selected path to that owner context
before the exact read. In Figment 0.10.19 with `toml` and `json`, `Format::file` locates
a file when constructing the provider and reads it during composition. Absent discovery
contributes no data. `Format::file_exact` retains the supplied path and reports a read
error during composition if it is unavailable. Use exact, absolute inputs for
caller-selected files, and use the existing tagged provider where the owner has already
acquired the source. Avoid doing a second read for explanation.

For a genuinely layered case compose `Serialized::defaults`, approved settings, the
selected profile and explicit CLI values. Use clap `ArgMatches::value_source` to
distinguish `CommandLine` from environment and parser defaults. Preserve the explicit
overlay as highest precedence. Reserve Figment Global for deliberately high-priority
values because it overrides the selected profile. `merge` recursively combines
dictionaries and replaces arrays. Reuse the investigation owner's pre-merge reset of
changed tagged alternatives, then use strict `extract` and owner validation. Keep
single-source settings and byte-exact environment and paths with their current owners.
Consider Figment `Env` only for a named allowlisted structured preference with defined
key collisions, provenance and redaction. A later process receives the validated
selection with its identity or revalidates the premise at the owner.

[I04](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I04) makes
the effective explanation a redacted projection of the same resolved values.
`find_value` returns a result, so use it to distinguish an absent key from invalid
composition before interpreting missing `find_metadata`. Report the tag of an effective
leaf or a whole replacement value. Container tags and provider inventory cannot
establish every child's origin. Preserve the selected profile separately. The pinned
`Tag::profile` distinguishes Default and Global but cannot name custom profiles.
Record originating profile at the provider if a receiver needs it. Use typed segments
or traverse actual values for literal dotted keys and arrays. Use `Tagged<T>` or
`RelativePathBuf` only when downstream meaning depends on origin or source-relative
paths. If a consumer justifies concatenated arrays, report element origins or a mixed
container. Report provider and key for errors that lack reliable source spans.

Provide a passive effective-settings view with value source, redaction and selector ambiguity.
Verify wrong cwd, missing explicit file, malformed optional file, mixed profiles,
explicit versus parser-default input, dotted keys, nested and array provenance,
changed preview premise, and secrets in explanation and recovery. Use pure provider
tests and existing isolated subprocesses for cwd and environment. Consider Figment
`Jail` only for a concrete gap that warrants process-global test state. Migrate
duplicated defaulting and environment assembly with their receiving commands.
[Current configuration](../../crates/pse-host/src/investigation/config.rs) ·
[Figment 0.10.19 file provider source](https://docs.rs/crate/figment/0.10.19/source/src/providers/data.rs) ·
[Composition and metadata](https://docs.rs/figment/0.10.19/figment/struct.Figment.html) ·
[Profile source](https://docs.rs/crate/figment/0.10.19/source/src/profile.rs) ·
[clap value sources](https://docs.rs/clap/4.6.7/clap/struct.ArgMatches.html#method.value_source)

## A03 — Original outcomes and safe recovery

[I05](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I05) extends
the versioned `Envelope`, `ErrorRecord`, `RecoveryRecord` and `NextAction` projection.
For each B–E receiver specify the original owner code, typed facts, effect identity,
retry unit, verdict, settlement and redacted source reference or bounded excerpt.
Keep clap usage context. Emit exact safe argv or a typed reference and omit operands
that cannot be disclosed. Migrate string-only wrappers with their callers.

Receive three additional owner projections from the
[SurrealDB and Rust tooling review](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md).
On adoption, E03 supplies the original nonce and separates pre-write refusal, uncertain phase
effect, settled phase, and committed activation. Recovery carries that same operation reference.
Configured apply and its receiving projection are deferred under the E03 adoption decision.
[E03 adoption](69e-source-model-and-live-schema-operations.md#configured-rollout-adoption)
[E02](69e-source-model-and-live-schema-operations.md#e02--preview-publish-and-recover-source-changes)
supplies a staged source identity and native primary and optional secondary byte labels,
with independent related findings for other files. Keep unreadable inputs distinct from
syntax failures and redact source excerpts at the receiving boundary.
[B02](69b-codex-workstation-and-preparation.md#exact-v4-prerequisites)
supplies the selected preparation identity and, for a selected standalone rollout tool,
the exact absent-release, rejected-override, or corrupt-generation disposition. Emit a
retry only when that route can restore the original prerequisite.

Use thiserror 2.0.21 contextual variants when a crossing adds typed facts. Use
`#[from]` and transparent forwarding where the source already supplies the
necessary meaning. Keep ordinary causes, miette `diagnostic_source`, and
independent `related` findings distinct. For miette 7.6.0, `Report::new` accepts
an owned `Diagnostic + Send + Sync + 'static` and `new_boxed` accepts its boxed
trait-object form. Prefer the
existing typed projection when no report boundary is needed. `IntoDiagnostic`
projects through the ordinary error interface and can discard an existing diagnostic
code. `WrapErr` adds narrative context without replacing structured facts.

Use optional help and primary or secondary labels for one source. Give related
findings for distinct files their own source identity and byte coordinates.
`NamedSource` can attach a name to known source content. Bound retention and omit
content where exact redacted rendering is unavailable. Reserve heterogeneous
`Vec<Report>` for a genuine aggregation. Machine consumers use the native envelope,
with miette JSON rendering or serde limited to focused diagnostic uses. A04 owns
local human rendering. Test the projection with a Unicode span, a child without
source, independent warnings, unsafe recovery operands, and a delegated cause.
[Native diagnostic projection](../../crates/pse-host/src/envelope/diagnostic.rs) ·
[thiserror 2.0.21 derive](https://docs.rs/thiserror/2.0.21/thiserror/#details) ·
[miette 7.6.0 Report](https://docs.rs/miette/7.6.0/miette/struct.Report.html) ·
[Diagnostic relationships](https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html) ·
[NamedSource](https://docs.rs/miette/7.6.0/miette/struct.NamedSource.html)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| A01 | complete | Existing parser and catalog | Passive package candidates, typed effect annotations and runnable examples are attached to the parser. The current help snapshot control passed and the parser-owned reference regenerated. Current passive discovery exposed the project publication status route. |
| A02 | complete | Current identity and configuration owners | Private project settings resolve defaults, a selected profile and CLI overrides in one read. The captured v4 journey verified unique and ambiguous configured targets, filtered unavailable aliases and refused a foreign issuer before any effect. |
| A03 | complete | Native envelope and original owner receipts | Preparation, project and source receivers carry owner state and exact operation identity. The live project control retrieved the original acknowledgement, and the source receiver preserved redacted staged labels. The configured schema nonce and activation receiver follow E03's explicit deferral. |
