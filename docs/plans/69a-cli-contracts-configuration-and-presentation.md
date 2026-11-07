---
title: "Plan69a: CLI contracts, configuration, and presentation"
status: planned
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69a-cli-contracts-configuration-and-presentation.md
doc_topics: [current-work, design, libraries]
---

# CLI contracts, configuration, and presentation

This companion owns [O01](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O01),
[O02](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O02), and
[O08](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O08). The
[coordinator](69-intent-first-cli-and-codex-workflows.md) owns shared decisions and assembled
verification. The current parser-derived catalog, investigation configuration, native envelope,
and investigation progress renderers are the foundations. Extend each where multiple receiving
commands have the same responsibility.

## A01 — One parser tree for intent and discovery

Expose name, argument requirements, effect category, owner, availability hints and usable examples
from the parser and its catalog metadata. Reuse clap 4.6.7 `CommandFactory` to obtain the command
tree, `Args` and `Subcommand` to compose it, `ArgGroup` for exclusive forms, and `TypedValueParser`
for identifiers, limits and OS-native paths. Treat parse success as syntax validity. A performing
owner verifies the selected identity, authorization and current state. Keep static help,
`pse commands`, machine schema and any generated completion derived from that same tree.

Start with the existing `pse inspect invocation` and `pse inspect log` entry points and correct
the environment guide's `pse result` spelling. Add focused detail through C03. Verify published
examples against parser acceptance and assert that discovery performs no preparation. Existing
Cargo, just and product helpers retain their owner roles. Remove obsolete advertised paths and
any duplicate first-party intent when its callers have moved. An arbitrary shell dispatcher
cannot supply the typed effect guarantees.

If a named human workflow benefits, generate static completion. Consider dynamic candidates
only when an actual discovery use case remains and the candidate path can stay passive.
Use `#[command(defer = true)]` only if a measured startup problem is attributable to tree
construction and eager and deferred forms yield equivalent help, schema, catalog and completion.
Keep eager construction if either condition fails. [Catalog](../../crates/pse-host/src/catalog.rs) ·
[clap guide](../libraries/cli/clap.md) · [clap 4.6.7 derive](https://docs.rs/clap/4.6.7/clap/_derive/index.html) ·
[Typed value parser](https://docs.rs/clap/4.6.7/clap/builder/trait.TypedValueParser.html)

## A02 — Resolved context with provenance

Resolve developer checkout and session identity from the existing owner. Product actions may
use a uniquely configured, authenticated target. Ambiguity returns valid authorized choices
before admission. Configuration can select a context or supply preferences. The typed action
and owner guards establish consent and permission for the effect.

For a shared, genuinely layered configuration, compose `Serialized::defaults`, approved
checkout settings, the selected profile, and an explicit CLI overlay. Determine explicitness
with clap `ArgMatches::value_source`. Keep highest precedence for explicit values. Use
`Toml::file_exact` for exact discovered settings or a source-tagged provider when the existing
owner needs its own discovery. Figment global values take precedence over selected profiles,
so avoid putting lower-priority settings there. Arrays replace wholesale and changed tagged
alternatives reset as a whole before merging. Extract strictly. Use `find_metadata` for origin,
and `Tagged<T>` or `RelativePathBuf` only where source context affects interpretation. Reuse the
investigation owner's reset logic where it fits. Single-source settings and byte-exact native
environment and tool paths stay with their current owners.

Provide a passive effective-settings view with value source, redaction and selector ambiguity.
Verify missing versus explicit input, whole-value changes, paths relative to their source,
non-UTF-8 inputs where supported, and ambiguity before effects. Migrate duplicated defaulting
and environment assembly in receiving commands to the shared context at their cutover.
[Current configuration](../../crates/pse-host/src/investigation/config.rs) ·
[Figment guide](../libraries/cli/figment.md) · [Figment 0.10.19](https://docs.rs/figment/0.10.19/figment/struct.Figment.html) ·
[clap value sources](https://docs.rs/clap/4.6.7/clap/parser/enum.ValueSource.html)

## A03 — Original outcomes and safe recovery

Extend the versioned `Envelope`, `ErrorRecord`, `RecoveryRecord` and `NextAction` projection
across receiving workflows. Use thiserror 2.0.21 typed variants and miette 7.6.0 source labels,
`related` and `diagnostic_source` where these relationships carry meaning. Retain structured clap
usage context, owner code, failed stage, verdict, settlement and original invocation or product
identity. Emit exact safe argv or a typed reference. Omit recovery operands when they cannot be
disclosed safely. Machine consumers use the native envelope. Use miette renderers for human
presentation and focused diagnostic checks. Migrate string-only wrappers with their callers.
[Native diagnostic projection](../../crates/pse-host/src/envelope/diagnostic.rs) ·
[Diagnostic guide](../libraries/cli/thiserror_and_miette.md) ·
[thiserror 2.0.21](https://docs.rs/thiserror/2.0.21/thiserror/) ·
[miette 7.6.0 Diagnostic](https://docs.rs/miette/7.6.0/miette/trait.Diagnostic.html)

## A04 — Quiet progress from owner events

Use indicatif 0.18.6 `ProgressDrawTarget::stderr` for attended human output and `hidden` for
JSON and NDJSON. Compare occasional phase lines for unattended human observers. Use a spinner
while the total is unknown. Use a determinate total only when the owner can account for it.
Add `MultiProgress` only for useful concurrent states, register bars before drawing and retire
them on settlement. Keep logs independent of hidden rendering. Use the current terminal
harness first and enable `InMemoryTerm` only if it adds a material assertion. Test machine
output purity, cancellation, unknown totals, and preservation of owner verdict and settlement.
[Existing renderer](../../crates/pse-host/src/cli/investigation.rs) · [Indicatif guide](../libraries/cli/indicateif.md) ·
[Draw target](https://docs.rs/indicatif/0.18.6/indicatif/struct.ProgressDrawTarget.html) ·
[MultiProgress](https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| A01 | planned | Existing parser and catalog | Complete discovery and accurate examples, with parser parity and passive behavior checked. |
| A02 | planned | Current identity and configuration owners | Shared redacted resolution used by receiving routes, with explicit precedence and ambiguity controls. |
| A03 | planned | Native envelope and original owner receipts | Structured recovery across receiving routes, including usage refusal and unresolved effects. |
| A04 | planned | A03 and actual owner events | Terminal and machine output controls, with no change to original verdict or settlement. |
