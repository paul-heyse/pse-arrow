---
id: ADR-0189
title: Enforce pure invocation channels for selected macro producers
status: proposed
date: 2026-10-14
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-04, AP-07, DP-09, DP-19]
blueprint: [§3.1, §5.3, §14.3, §24.2]
review: git:6af92d1dee1ffa0b80dca71e03a63b489f2d940c:docs/design_review/reviews/design_review_plan39-pure-macro-channels_2026-10-14.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: None; additional environment/provenance proof targets were withdrawn by the maintainer and carry no revisit obligation.
verification: Actual compiler controls reject generated Paste environment selectors and Pest file attributes, preserve inline grammar parsing, invalidate changed owner/helper/features, and distinguish caller code identity from macro external-input assurance; no further strict producer/environment proof campaign is selected.
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

Review binds exact source and executable helper closure. Real compiler fixtures exercise generated selectors, unsupported forms and unchanged normal parsing. Scientific input hashing independently retains relevant caller code. The implemented tokens-only restriction remains; no additional environment/provenance proof investigation is scheduled.

### Confirmation

The bounded design assessment is linked above. Current implementation and test outcomes belong to Plan39b B1/B5. Targeted compiler behavior retains its stated scope. Plan39d owns ordinary functional acceptance; the additional strict deployment proof campaign is withdrawn.

### One finite preparation operation — authorized remaining Plan39 scope

The maintainer authorized the remaining-design execution on 2026-10-14, excluding the final campaign. The existing role-by-role package-family refresh can delete earlier roles' current inputs. Amend physical sequencing, not current-input verification: one private preparation operation performs dependency setup and the selected executor-family union refresh before any role capture. No subsequent role cleans beneath completed evidence. This is not a persistent compiler cache grant or permission for a Fresh artifact from a previous operation.

Actual Maturin child argv/SDK/environment remains authoritative and is enrolled late. Unknown executor families refuse before build. A first exact executor compilation context requires successful non-Fresh compiler evidence; later Fresh use requires that same operation's preceding exact compilation with unchanged dependency-complete source/configuration/reviewed environment/native/absence premises and original outputs. Unit-graph keys alone are insufficient. Build-script output associations remain distinct from executable compilation evidence; an emitted build-script-executed message alone does not prove execution in that child. Unknown, changed or incomplete premises refuse.

Protected historical evidence remains independently reconstructible; original-path current artifact/input/namespace/environment verification is unchanged. Existing coordinated ownership and before/after checks still apply: private preparation exclusion does not make arbitrary concurrent tools harmless. The bounded independent assessment is [the preparation-operation review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_plan39-preparation-operation_2026-10-14.md). That assessment preceded the implemented bounded correction; no additional review or assurance-code prerequisite is scheduled. Tiny compiler fixtures establish only this mechanism. Actual PSE all-role proof capture/install/import is withdrawn from Plan39d.

### Withdrawn environment/provenance proof targets

The maintainer withdrew the environment/provenance proof targets on 2026-10-14. Fresh all-role strict capture, source-completeness investigations, executor-input projections, compiler/launcher/installer phase associations and exact installer/environment reproduction are excluded from Plan39 and its inherited Plan38 acceptance. These targets are eliminated, not deferred; no revisit trigger or future campaign is retained. Existing producer code/tests and runtime safeguards remain. Scientific/model/result provenance, explicit Arrow force-validation, physical correctness and truthful cancellation/drain remain required. The earlier source-qualified executor projection and launcher/installation phase-chain target is withdrawn from this proposed record. Its bounded review retains historical scope only and authorizes no future proof work. Formal ADR status remains proposed.

## Pros and cons

The target removes unrelated caller review invalidation at the channel owner and keeps standard library semantics for supported pure inputs. It requires small maintained forks and explicit refusal of ambient forms; it does not eliminate source-qualified owner/helper review.

## More information

[Plan39b](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/plans/39b-deployment-assurance-and-preparation.md), [friction F02](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f02), [producer identity](../authoritative_design/sections/identity-and-publication.md).

## Status history

- 2026-10-14 — proposed after explicit operator selection; implementation and assembled qualification remain distinct.

- 2026-10-14 — maintainer withdrew the additional strict campaign, environment/source-completeness and compiler/launcher/installer phase-proof targets; no deferral or revisit requirement remains. Existing implementation and scoped evidence are preserved.
