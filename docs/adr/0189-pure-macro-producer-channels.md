---
id: ADR-0189
title: Enforce pure invocation channels for selected macro producers
status: proposed
date: 2026-10-14
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-04, AP-07, DP-09, DP-19]
blueprint: [§3.1, §5.3, §14.3, §24.2]
review: docs/design_review/reviews/design_review_plan39-pure-macro-channels_2026-10-14.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A consumed macro requires environment selectors or file grammar input, or a guarded owner/helper upgrade introduces another external input channel.
verification: Actual compiler controls reject generated Paste environment selectors and Pest file attributes, preserve inline grammar parsing, invalidate changed owner/helper/features, and distinguish caller code identity from macro external-input assurance; Plan39d retains assembled strict producer qualification.
---

# ADR-0189: Enforce pure invocation channels for selected macro producers

## Context

Plan39 B1 found that actual Paste arguments can select environment inputs absent from rustc dep-info. Source-qualified stock Paste and Pest therefore still require whole-caller external-I/O review. On 2026-10-14 the operator selected guarded pure owners after this bounded investigation, retaining ordinary Local and explicit selective qualification.

## Scope

Amend the macro external-input assurance realization of blueprint §5.3/§14.3. Preserve scientific code identity, actual unit/feature/helper closure, receipt interpretation and current deployment freshness. This does not change the Python public API or numerical grammar. ADR status remains proposed pending the decision-PR route.

For these transitive library owners, explicit root `patch.crates-io` path selection is necessary. Amend the old blanket patch prohibition in blueprint §3.1: permit exact-version repository-contained vendor sources only when the committed lock records one matching local package/version, without a registry/git source. Retain the ban on replacement tables, configuration-path overrides, aliases and unreviewed source substitution. A workspace dependency declaration alone cannot select a transitive crate's implementation; copying all intervening consumers would add unnecessary maintained forks.

## Drivers

A body edit must remain part of scientific identity without forcing an unrelated external-I/O grant to be reviewed again. The actual macro owner can enforce its input channels even when another macro generates the invocation. Testing that responsibility should require a small compiler fixture, rather than the full simulator or an invented Rust macro interpreter.

## Options

Stock owners with whole-caller review remain conservative but retain F02's manual amplification. Grep/token projection or a caller environment snapshot cannot account for generated selectors and actual compiler-process mutation. Instrumenting the whole compiler would add a larger unestablished ownership boundary. Select small source-qualified dependency patches: Paste rejects Env at its shared parser; Pest accepts literal inline grammars only and invokes the existing generator with file inclusion disabled; py_literal has one inline grammar authority. The library algorithms remain the parsing/generation owners. This adds maintained upstream patch obligations and deliberately refuses previously available compile-time forms.

## Outcome

A reviewed tokens-only invocation contract can remove caller external-I/O grant dependence only when the exact guarded executable source and complete selected helper/feature closure are current. Owner review is explicit evidence, never inferred from a package name, metadata flag or successful compilation. Unknown contracts or changed owners/helpers refuse selective eligibility. Stock effectful owners keep their whole-caller and actual grammar-consumption requirements.

### Consequences

Retain minimal patched dependency source with upstream version/provenance and licenses. Reject every Paste Env segment before environment access across paste/item/expr, including generated arguments. Reject Pest file attributes before generator calls; use `pest_generator::derive_parser(input, false)` for validated inline-only input. Replace py_literal's grammar file and include constant with one literal inline attribute preserving its exact grammar. No second authoritative grammar or compatibility macro path remains.

### Compensating controls

Review binds exact source and executable helper closure. Real compiler fixtures exercise generated selectors, unsupported forms and unchanged normal parsing. Scientific input hashing independently retains relevant caller code. A later effectful requirement reopens this decision rather than silently broadening tokens-only assurance.

### Confirmation

The bounded design assessment is linked above. Current implementation and test outcomes belong to Plan39b B1/B5. Targeted compiler behavior does not qualify the full deployment campaign; Plan39d owns that later scope.

### One finite preparation operation — authorized remaining Plan39 scope

The maintainer authorized the remaining-design execution on 2026-10-14, excluding the final campaign. The existing role-by-role package-family refresh can delete earlier roles' current inputs. Amend physical sequencing, not current-input verification: one private preparation operation performs dependency setup and the selected executor-family union refresh before any role capture. No subsequent role cleans beneath completed evidence. This is not a persistent compiler cache grant or permission for a Fresh artifact from a previous operation.

Actual Maturin child argv/SDK/environment remains authoritative and is enrolled late. Unknown executor families refuse before build. A first exact executor compilation context requires successful non-Fresh compiler evidence; later Fresh use requires that same operation's preceding exact compilation with unchanged dependency-complete source/configuration/reviewed environment/native/absence premises and original outputs. Unit-graph keys alone are insufficient. Build-script output associations remain distinct from executable compilation evidence; an emitted build-script-executed message alone does not prove execution in that child. Unknown, changed or incomplete premises refuse.

Protected historical evidence remains independently reconstructible; original-path current artifact/input/namespace/environment verification is unchanged. Existing coordinated ownership and before/after checks still apply: private preparation exclusion does not make arbitrary concurrent tools harmless. The bounded independent assessment is [the preparation-operation review](../design_review/reviews/design_review_plan39-preparation-operation_2026-10-14.md). Dependent assurance code waits for that assessment. Tiny compiler fixtures establish only this mechanism; actual PSE all-role capture/install/import remains Plan39d work.

## Pros and cons

The target removes unrelated caller review invalidation at the channel owner and keeps standard library semantics for supported pure inputs. It requires small maintained forks and explicit refusal of ambient forms; it does not eliminate source-qualified owner/helper review.

## More information

[Plan39b](../plans/39b-deployment-assurance-and-preparation.md), [friction F02](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f02), [producer identity](../authoritative_design/sections/identity-and-publication.md).

## Status history

- 2026-10-14 — proposed after explicit operator selection; implementation and assembled qualification remain distinct.
