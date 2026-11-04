---
id: ADR-0244
title: Share lossless Proto contract interpretation across tooling and host
status: proposed
date: 2026-11-04
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-05, DP-01, DP-02, DP-24]
blueprint: [§4.7, §21.7]
review: docs/design_review/reviews/design_review_plan65_proto_adapter_2026-11-04.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A selected independent reader needs product dependencies or duplicates option interpretation to consume an owner-derived contract.
verification: Bounded dependency assessment and shared adapter tests for original-pool interpretation, stripped options, unknown schemas, semantic changes and observation-only construction; host and generator consumer compilation.
---

# ADR-0244: Share lossless Proto contract interpretation across tooling and host

## Context

ADR-0241 places structural declarations and semantic comparison in pure `pse-contract`.
Plan65 A02 implements lossless Buffa descriptor acquisition, typed option writing and
normalization in `pse-codegen`. The next independent host evidence consumer cannot import
that crate's schema/Arrow/product dependencies merely to interpret the same options.

## Scope

Add lightweight `pse-contract-proto` for the existing Buffa adapter. Keep `pse-contract`
wire-independent and keep scientific facet meaning at its domain owner. This amends
blueprint §4.7/§21.7 through the proposed decision route before adding the crate; it does
not supersede the pure foundation or change native scientific or workflow authority.

## Drivers

Generation and independent retained-report inspection must enforce the same option schema,
presence, source coverage and interpretation rules. Changing a known option field or losing
an annotation must not become compatible through a different reader. Each acquired old/new
descriptor world needs its own available interpretation. These consumers require one shared
adapter and local tests without preparing scientific execution or starting a database.

## Options

Importing `pse-codegen` into the host introduces unrelated product dependencies. Repeating
the adapter in the host duplicates consequential schema correspondence and unknown-field
handling. Adding optional Buffa dependencies to the pure crate weakens its declared boundary.
Moving the adapter into diagnostics or build information assigns it an unrelated owner.
Select a small library with an actual generator and host consumer, rather than a service,
global pool or second structural vocabulary.

## Outcome

`pse-contract-proto` owns the transport adapter, typed core-option correspondence checks and
native adapter errors. Its dependencies are `pse-contract`, the selected Buffa runtime and
descriptor family, and direct error/diagnostic support. It imports no product, schema registry,
Arrow, database, runtime or codegen crate. Codegen owns graph lowering and compiler/plugin
invocation; domain owners supply explicit typed facet callbacks. The host consumes the same
adapter with only the policies available to that reader.

The same transport owner also absorbs the existing generic scalar/container crossing helpers
needed by generated host and scientific adapters. Selected maintained-validator errors retain
their original violations and compilation/evaluation distinctions. Native diagnostic presentation
stays with the consumer. Pure byte access at the actual scalar owner permits checked identity
and buffer conversion without adding Buffa to scientific identity crates or duplicating nominal
type implementations in each consumer. Generated native adapters and both consumers migrate
together; the runtime-only copy is removed.

### Consequences

Move the adapter and its focused tests, rather than copying them or preserving a forwarding
implementation. Translate errors at the actual consumer boundary. Buffa `DescriptorPool`
links each complete import closure, `DynamicMessage` and fallible `ReflectMessageMut::try_set`
check declared field shapes, and `ExtensionSet` preserves actual option payloads. The pure
checker remains the sole semantic comparison operation. Unavailable policy yields Unknown;
successful decoding never grants scientific use, test verdict, readiness or settlement.

### Compensating controls

Keep source-selected coverage independent of option presence. Test stripped nested metadata,
unknown/changed option schemas, new domain facets and separate original-world policies.
Retain actual bytes where original identity is promised. No lossy Prost descriptor round trip,
hand tag codec, global mutable descriptor pool or default-compatible unknown interpretation.

### Confirmation

The bounded assessment linked above evaluates this dependency decision. Plan65 A02/G02/J02
own implementation, shared negative controls and integrated consumers. Formal ADR acceptance
and assembled qualification remain separate; no new behavior is claimed by this proposal.

## Pros and cons

One shared adapter removes duplicated interpretation and keeps independent consumers local.
The additional crate requires a complete move, direct dependency wiring and consumer checks;
it provides no new scientific semantics or measured performance benefit by itself.

## More information

[Plan65a](../plans/65a-contract-foundations-and-generation.md) owns shared metadata delivery;
[Plan65j](../plans/65j-workflow-lifecycle-and-developer-evidence.md) owns original developer
evidence. [Buffa 0.9.2 reflection](https://github.com/anthropics/buffa/tree/v0.9.2/buffa-descriptor/src/reflect)
and its [guide](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md) characterize the
selected pool, dynamic message, extension and mutation functionality.

## Status history

- 2026-11-04: Proposed before adding the shared Proto adapter crate.
