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

### Source-qualified compiler, launcher and installation contexts — deferred

The maintainer subsequently directed design-phase validation without proving minor environment changes or exact reproduction to this degree. This additional phase-context machinery is deferred; it is not a prerequisite for ordinary functional validation. Revisit only for a separately requested strict cross-build assurance requirement. The target review remains historical evidence, not implementation acceptance.

On 2026-10-14 the maintainer accepted the concrete D0 phase-context target in Plan39d. The existing whole-role context conflates executor compilation, Maturin's prebuild Python startup and uv's later replacement of installation metadata. Preserve these as separate owned operations and evidence, rather than claiming a mutated input namespace is unchanged. This amends the prior all-consumed-inputs-unchanged realization for the explicitly checked installation transition; it does not permit arbitrary changed inputs.

An executor projection requires an explicit current source-qualified completeness contract over exact owner/helper/source/features/platform, compiler/configuration/native inputs, executable I/O and actual associated compiler environment/generated suppliers. Existing runtime-I/O grants alone do not establish compiler relevance. One admitted executor view supplies its reuse basis and original-premise verification. Unknown/stale/incomplete projections keep conservative refusal. First compilation remains actual successful non-Fresh evidence in the same operation; later reuse keeps unchanged complete relevant premises and original outputs.

The build association records the actual Cargo child's inputs and remains immutable in protected history. Maturin's prebuild launcher separately records interpreter/startup and complete import-shadow/absence and `.pth` premises plus the actual delivered Cargo arguments/configuration. Installation separately binds selected wheel/distribution identity, site entry selection, target metadata/RECORD-directed removal, authenticated outputs and full before/after namespace reconciliation. Only source-qualified exact installation-output changes are permitted; unexplained membership, bytes, modes, links or aliases refuse.

Final verification keeps unchanged build/executor inputs current, verifies original launcher evidence historically, authenticates every allowed installation transition, and independently verifies current installed/imported context and full bytes with the existing exact RPATH-only replay. It neither applies the old whole-role unchanged-namespace verifier to a known changed launcher namespace nor silently rebinds the original build receipt. Metadata-only diagnostics do not qualify this contract. Plan39d owns revealing controls and actual all-role acceptance; the [independent phase-context assessment](../design_review/reviews/design_review_plan39-executor-and-installer-contexts_2026-10-14.md) accepts this target at Proposed strength before dependent implementation. Formal ADR status is unchanged.

## Pros and cons

The target removes unrelated caller review invalidation at the channel owner and keeps standard library semantics for supported pure inputs. It requires small maintained forks and explicit refusal of ambient forms; it does not eliminate source-qualified owner/helper review.

## More information

[Plan39b](../plans/39b-deployment-assurance-and-preparation.md), [friction F02](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f02), [producer identity](../authoritative_design/sections/identity-and-publication.md).

## Status history

- 2026-10-14 — proposed after explicit operator selection; implementation and assembled qualification remain distinct.
