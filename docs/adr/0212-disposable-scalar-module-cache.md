---
id: ADR-0212
title: Reuse complete scalar compiler inputs through a bounded disposable cache
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-07, PS-06, PS-12]
blueprint: [§14.5]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A consumed compiler input is absent from the key, a cache hit changes original scalar meaning, or bounded lookup/publication introduces a material execution regression.
verification: Plan48c C07 checks fresh-process hit without compiler children, changed source/profile/export refusal, corruption/partial/bounds handling, private module/scratch lifetimes, cancellation and cold fallback with identical original scalar results.
---

# ADR-0212: Reuse complete scalar compiler inputs through a bounded disposable cache

## Context

Packed scalar compilation already has complete bounded generated C++ source, named exports,
input/output dimensions and an explicit conservative Linux x86-64 compiler/ABI profile.
Every fresh process currently starts compiler/linker children for the same inputs. Reusing
only those completed code bytes can remove that work without persisting scientific state.

## Scope

Plan48c C07 within blueprint §14.5. Retain explicit compiled selection and the interpreted
default. Cache only the existing pure scalar module representation; no checked model,
provider, receiving proof, symbolic library state, JIT ISA or completed evaluation is restored.

## Drivers

Remove repeated compiler/linker work on complete input matches. Follow the operator's
first-principles deployment preference, then diagnose actual composed performance; no
minimum workload size or marginal short-run timing win is an adoption prerequisite.

## Options

Keep every module ephemeral; persist incomplete function records; or cache code at the
existing complete compiler-input boundary. Select the last option. Its receiving owner has
actual generated source and ABI inputs rather than inferring compatibility from dimensions.

## Outcome

A private disposable cache keys the exact generated source bytes, conservative scalar
profile/compiler flags and path, named exports and dimensions. Physical encoding checksums
serve cache integrity, never semantic IDs or current scientific permission. Complete image
records include format, input key, byte extent and image checksum, published atomically.
A hit is bounded, copied into the current private temporary module owner and checked against
all demanded exports/dimensions before publication. The normal owner still supplies current
source/domain/provider/stop checks and independently loaded mutable scratch.

The local store has finite entry/byte bounds and bounded metadata scanning under a short
nonblocking publication lock. Busy, missing, corrupt, incompatible, oversized or unavailable
cache state chooses ordinary fresh compilation. Cache publication is optional after a
successful current module; a cache I/O failure does not change a valid numerical result.
Eviction cannot delete a live module's private copy or release its allocation owner.

### Consequences

Fresh-process compilation can reuse code without inheriting results, callbacks or execution.
The directory is disposable local temporary state. No exact environment match, installation
provenance, compiler-binary attestation or new proof manifest is introduced. The existing
supported scalar target/FP/ABI profile remains the compatibility contract.

### Compensating controls

Keep complete current compiler preflight and original module capacity on hits as well as
misses. Validate image extent/checksum before loading; validate every named export and shape.
Keep original cancellation checks before/after receiving and actual compiler drain on misses.
Atomic publication and private copies isolate concurrent eviction/replacement. Test actual
fresh processes and corrupt/partial images; a same-process reload alone is insufficient.

### Confirmation

**Implemented / Tested:** on2026-10-20, `just unit-package pse-math 'test(native_compiled::)'
--features compiled-evaluator --success-output immediate` passed12/12 selected controls with
zero compiler warnings/errors. Actual fresh processes verify cold compilation, a compatible
hit without compilation and invalid-module fallback with original scalar results; changed
source/compiler/dimensions, corruption/partial/bounds, current stop, private lifetime,
nonblocking lock and bounded eviction controls pass. Its later real concurrent-writer,
eviction and receiving extension passed1/1 through the same package/feature route, with zero
errors/warnings. These units establish neither a workload hit rate/speedup nor full Plan48 qualification.

## Pros and cons

Hits eliminate compiler children while preserving current scalar execution meaning. A small
bounded cache and receiving path add I/O and format handling; fresh compilation remains the
single numerical producer on a miss, rather than a second mathematical implementation.

## More information

- [Plan48c C07](../plans/48c-mathematical-products-and-compiled-execution.md#work-packages)
- Existing `pse-math` packed native compiler and scalar realization profile.

## Status history

- 2026-10-20 — Proposed before implementation under accepted RC06 and maintainer deployment-first guidance.
