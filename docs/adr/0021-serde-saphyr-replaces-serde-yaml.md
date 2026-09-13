---
id: ADR-0021
title: Load YAML with serde-saphyr; serde_yaml is banned
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-51, DM-30, DM-45]
blueprint: [§3.1, §3.3]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: `serde-saphyr` becomes unmaintained, or a document form the authoring language needs is not expressible through it
verification: `rust / deny` (`deny.toml` bans `serde_yaml`); `tests/governance/tests/banned_patterns.rs`; the hostile-document budget tests in `tests/conformance`

---

# ADR-0021: Load YAML with serde-saphyr; serde_yaml is banned

## Context

Blueprint §3.3 loads package and case documents from YAML and TOML with source spans. `serde_yaml` is deprecated (0.9.34+deprecated, last release 2024-03), and the deprecation is encoded only in semver build metadata, so `cargo audit` does not see it — the reason §3.1 requires `cargo deny` as well.

## Scope

Binds the YAML parser and the hostile-input posture of the document loader. TOML stays `toml` 1.1.6 with `Spanned<T>` supplying `pse.source_span`.

## Drivers

Authoring documents come from users and agents; a parser that panics on hostile input is a denial-of-service surface; typed errors with line and column are what diagnostics (§23.2) need.

## Options

Keep `serde_yaml` — rejected: unmaintained and invisible to `cargo audit`. `yaml-rust2` — rejected: no serde integration, so the typed-error property would be hand-built. JSON only — rejected: the authoring language is meant to be written by hand.

## Outcome

`serde-saphyr` 1.2.0 is the YAML loader: typed errors with line and column, hostile input refused without panics, and a `budget` bound on nesting, aliases and allocation.

### Consequences

The budget must be configured per document class and tested with adversarial inputs (charter DM-54); `serde_yaml` is banned in `deny.toml` so it cannot return through a transitive dependency.

### Compensating controls

`deny.toml` names `serde_yaml` with the reason and the review date; the conformance layer feeds the loader deeply nested and alias-bombed documents and asserts a typed refusal.

### Confirmation

`rust / deny` is a required check; the budget tests run in `rust / test`.

## Pros and cons

A less-used parser carries its own risk; the register carries the row that re-checks its maintenance status.

## More information

Blueprint §3.1 (supporting crates), §3.3 (document loading); review finding R2-5; supporting-libraries capability map.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
