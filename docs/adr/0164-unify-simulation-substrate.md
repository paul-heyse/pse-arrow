---
id: ADR-0164
title: Unify authored models, compilation and durable results in SurrealDB
status: proposed
date: 2026-10-05
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-09, PS-10]
blueprint: [§D10, §D14, §4.1, §4.2, §5.3, §14.3, §14.4, §20, §20.4, §20.6, §21.1, §22.2]
review: docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: The selected SDK/server cannot preserve exact cells, guarded decisions or acknowledged commits under the supported restart scenario.
verification: Plan 28 A/B targeted codec, conflict, protection, restart, selected compilation, reconstruction and coordinate controls; E owns assembled qualification and performance measurements.
scenarios: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#representative-journeys]
---

# ADR-0164: Unify authored models, compilation and durable results in SurrealDB

## Context

PostgreSQL operational state, Delta publication, full source-bundle conversion and process-local compilation ownership make one scientific operation span several authorities and recovery protocols. The unified-substrate review supports a canonical graph and relational substrate at Proposed design strength. The maintainer authorized RC01–RC10, a clean rebuild and native queries plus Arrow, initially Plan 28a/28b and on 2026-10-06 the full 28a–28e scope with functional implementation before assembled qualification.

## Scope

This amends the governed architecture contracts. It replaces the canonical-store/publication portions of ADR-0114 and the global-build product-key interpretation; it preserves typed semantic declarations, physical admission, native solver ownership and complete outer executable attestation. ADR-0114 remains the historical implemented basis until its mechanism is removed; its formal supersession follows the decision PR route. Feature stabilization changes only when an actual surviving consumer warrants them.

## Drivers

Selected edits must preserve unrelated immutable objects. Scientific products and outcomes must survive restart with their actual interpretation and be queryable by problem/revision/run/output. Structural database selection must reduce repeated source hydration while shared Rust kernels retain scientific meaning. Protected preparations and long reads must remain valid through retention, and uncertain acknowledgments must settle by immutable operation identity.

## Options

Keeping PG/Delta while repairing local caches can improve individual hot paths but preserves cross-store visibility and source/result composition. An embedded database introduces the engine into application builds and each process's resource envelope. A supervised remote SurrealDB boundary supplies graph selection, relational result blocks and one transaction authority while library mathematics stays outside short transactions. Its costs are explicit supervision, exact codecs, guarded conflict decisions and bounded streamed visibility; database choice alone proves no speed improvement.

## Outcome

Repurpose `pse-operations` as the concrete thin remote SDK and typed-operation owner. The supported initial profile is authenticated loopback gRPC with RocksDB `sync=every&versioned=false`, explicit 4 MiB message limits and finite server/worker budgets. Schema and codecs derive from existing declarations. Finite scientific f64 cells preserve authoritative bits; unsigned integers use checked native decimal values. Interpretation versions are explicit.

Immutable object versions and membership intervals supply linear per-problem head-CAS revisions. Named head/name/scope/retention guards protect all consumed premises; conflict retries repeat the whole decision. Selected scientific admission remains one Rust kernel. Portable admitted descriptions retain complete positive, absent-name, membership and interpretation dependencies; native handles remain process-local. Producer-specific relevant identities govern reuse, while complete dirty-build attestation belongs to the outer run.

The maintainer confirmed the production-efficiency review's RC01 on 2026-10-07.
Three versioned products have separate lifetimes: relevant role-specific producer/build
inputs before linking; independently observed deployed artifact bytes associated with the
reviewed capture after build/install; and complete dirty outer source/build observation at
deployment/run admission. The latter includes executable composition roots, handwritten,
generated and vendored sources, but is not compiled into every associated consumer.
An unrelated outer edit updates provenance without rebuilding an unaffected role or
invalidating its scientific products. Consumed root/native/configuration changes invalidate
the affected role. Unknown consumed inputs and caller-manufactured matching receipts confer
no deployment eligibility. Actual worker startup and Python import check their own artifact
associations and required scientific/ABI compatibility, rather than equal all-tree hashes.
An installation without its owning authored checkout records absent source evidence and an
actual artifact observation. Ordinary admission and persistence remain available; missing
outer evidence never mints a scientific producer qualification. Supplied producer evidence
still requires independent current-artifact and consumed-input verification.
Historical identity and receipt meanings are never silently reinterpreted.

Normal runs retain scientific outcomes; ephemeral execution is explicit. Closed bounded staging admits an exact result descriptor atomically under the attempt fence. Protected selection becomes durable retained roots before preparation releases protection. Native selectors and bounded Arrow streams replace Runtime, modeling-knowledge and TableReader SQL APIs. Rebuild controlled artifacts, with no preservation importer, compatibility reader, dual writes or second production backend. Retire `pse-operations-queries` and `pse-catalog` when their last callers move.

The feature-unification crate remains with actual Arrow/DataFusion or remote-store consumers.
Pure semantic, numerical and native-ABI roots do not import that complete boundary closure
solely to stabilize features. The supported nightly workspace feature unification still
selects shared dependency features, and actual boundary consumers retain the generated
workspace hack. This narrows ADR-0122's all-member attachment without weakening the single
resolved type universe or enabling opt-in force-validation/native features globally.

Versioned generated SurrealQL functions execute bounded structural transitions and native bulk ingestion. Shared scientific kernels remain the admission owner. Dense results use self-contained, uncompressed Arrow IPC blocks in the same database with operation-shaped range metadata and preflighted decoded extents; ordinary scalar selections preserve exact cells and finite derived projections. Read consumers use separate restricted sessions and require successful statement completion before declaring a stream or export complete. ISO GQL and additional front ends require a concrete qualified consumer.

### Consequences

The maintainer separately accepted the remaining-enhancement review's RC01 on
2026-10-07. Ordinary preparation may reuse a versioned deployment-local product
without a source/compiler qualification receipt. Its opaque admission independently
observes the receiving worker or imported Python artifact, actual interpreter where
applicable, complete supported executable-backed ELF mappings and effective loader/runtime
configuration. The local guarantee requires exact deployed compatibility; it does not
claim relevant-source compatibility across rebuilds. Explicit `QualifiedProducer`
admission retains its stronger, separate reviewed-capture guarantee. Historical
unqualified products are never promoted by interpretation changes.

For the controlled Linux/glibc 2.39 profile, synchronous immutable reconstruction runs
inside `dl_iterate_phdr`'s loader write-lock scope. Mapping publication/removal is
excluded; loader-generation counters and independently observed bytes/configuration are
checked around construction. The callback cannot import code, wait for a loader thread,
perform asynchronous work or invoke caller/provider callbacks. Panic containment keeps
unwinding out of the C boundary. The owning executable/imported module TLS is
materialized before taking the loader write lock; construction touches no newly loaded
module TLS. The maintainer accepted preparation RC01–RC04 on 2026-10-08.
Receiving qualification belongs to an actual persisted acquisition or new eligible
publication, followed by independent immutable mathematical products in the initialized
process symbol universe. Pure memory retrieval consumes complete scientific identity,
not a repeated observation of the originating whole executable closure. Compatible
symbol registration is append-only; reserved PSE symbols reject incompatible prior
attributes and callbacks, and resetting the universe with live products is prohibited.
Stable active mathematical implementation remains a controlled-root premise; unmanaged
interposition, executable-map mutation and callback-bearing reconstruction confer no
local eligibility. The revised local namespace and runtime envelope are versioned;
historical descriptions never acquire the new guarantee by reinterpretation.
Reconstruction produces immutable mathematics and provider descriptions, with no loaded
provider handles; later evaluator/native use retains its separate scientific admission
and managed-generation lifetime. No long-lived cache object holds the loader lock.
Anonymous executable code, JIT, deleted/replaced or unidentified mappings, unsupported
loaders and unknown consumed configuration refuse reuse. Direct executable-map mutation,
concurrent configuration mutation and arbitrary privileged file writes are outside this
controlled profile and require readmission. Ordinary scientific admission and history
remain available after any refusal.

Complete exact preparation products retain specialization, projections, admitted bodies
and original scientific obligations under selected source/dependency versions, actual
instance, structural bindings, limits and consumed physical/provider interpretations.
Numerical case values bind later; each use constructs fresh current lineage and source
metadata. A pure owned frontier exposes known compiler-sealed requests before explicit
persisted acquisition; completion consumes the same frontier without another specialization.
Tracked mathematical queries perform no host observation, store request or publication.
Immutable basis and encoded-description retention use existing bounded owners and flights.

Every consumer rechecks its own selection and any acquisition dependency delta, then
explicitly settles each exact original description under the canonical retention guard.
An existing rooted acknowledgment preserves its original provenance and can settle after
protection expiry without new admission. Only absent acknowledgment requiring new eligible
publication obtains current producer admission; original provenance is not permission.
New admission requires live protection. Cancellation and retries consume the original
operation clock, and shared work retains charges through actual completion/drain.

The scoped loader premise is supported by the glibc 2.39
[iteration](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-iteratephdr.c),
[namespace publication](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-object.c)
[removal](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-close.c) and
[TLS allocation](https://github.com/bminor/glibc/blob/glibc-2.39/elf/dl-tls.c) owners.
This is a bounded supported profile, not a general plugin or deployment attestation API.

Storage, authoring, compilation and execution/result consumers migrate together. Immutable portable products require exact interpretation and reconstruction compatibility; a validated flag or current default is insufficient. Selected coordinate layouts include guards, branches, nested scopes and providers as well as arithmetic outputs. Offline backup quiesces and drains managed work before copying a stopped coherent database; restore checks interpretation before writes.

### Compensating controls

Recipe-owned targeted server tests cover acknowledgment recovery, conflicts, exact cells and protection. New numerical states retain admission checks and correctness tests force validation. Unknown source/native identity prevents persistent reuse. Rows streamed before statement completion remain provisional. Process-kill evidence does not establish physical power-loss survival; network deployment is separately qualified.

Strict reconstruction has an explicit trust boundary. Decoded receipt DTOs are not admitted
scientific witnesses. A non-serializable permit binds the complete compiler-issued payload
and its qualified request before the scientific owners can enter replay. Its only arbitrary
mint is a narrowly documented unsafe constructor used by the controlled storage decoder;
ordinary safe callers cannot create replay authority from chosen hashes or decoded receipt
fields. This is an inter-crate trust assertion, with no pointer, memory or native-ABI operation.
The runtime checks producer/deployment compatibility, stored payload identity, exact request
and dependency eligibility before minting it. Privileged storage modification is outside
the controlled-write contract and requires readmission. Generic safe publication cannot write the reserved scientific producer role; its
separate unsafe publication entry requires an actual compiler-issued admitted description.
The storage and runtime writer/reader trust boundaries are explicit, and governance records
the affected owners, with compiler fixture minting restricted to tests; the pure permit controls are the bounded non-FFI Miri scope.

Deployment producer qualification is also an explicit unsafe trust assertion: the operator supplies the reviewed actual tool capture, bound to the independently observed deployed artifact and its relevant role-specific inputs; complete dirty outer observation remains at deployment/run admission. Public embedding callers cannot safely mint an eligible deployment merely by manufacturing matching JSON hashes. Publication identities include the store-issued protection identity; repeated publication within one protected admission is idempotent, while independently admitted recompilation can retain a fresh product after explicit release of an older one. Request and dependency eligibility remain separate from publication identity. Generic root retention cannot create or move product roots.

Already rooted exact descriptions are reused atomically, so repeated preparation does not retain duplicate publications merely because it acquired a fresh pin.

Terminal admission has the same explicit scientific trust boundary. The lifecycle writer
can close and reconcile exact observations, but cannot infer scientific success from
chosen completion bytes. Its unsafe sealing entry requires the owning kernel's actual
completion classification and the observations admitted for that attempt. The runtime
asserts this only after scientific completion and closed-manifest reconciliation; tests
may mint controlled fixtures. This is a semantic authority assertion, not a memory operation.
The occurrence observation writer similarly requires actual admitted kernel facts or an
effect-free shared-policy refusal; stored DTOs cannot mint usable predecessor permission.

### Confirmation

The cited review establishes architectural reasoning for the unified target. Plan 28 owns findings and implementation evidence; its A/B companions record their local outcomes and E records assembled correctness and measurements. This proposed record neither certifies implementation nor changes status outside the decision PR route.

## Pros and cons

One substrate makes revision selection, product discovery, result queries and retention composable. It also makes the database's transaction, codec and supervised restart behavior essential application contracts rather than incidental infrastructure.

## More information

[Plan 28](../plans/28-surrealdb-unified-substrate.md) owns implementation and finding dispositions. [28a](../plans/28a-canonical-substrate-and-revisions.md) owns store/revision/protection contracts; [28b](../plans/28b-selected-compilation-and-reuse.md) owns selected admission, products and numerical reuse. Their shared declarations and adjacent lifecycle consumers remain coordinated there.

## Status history

- 2026-10-05 — proposed before dependent production changes, under maintainer implementation authorization. Formal acceptance and ADR-0114 supersession remain in the decision PR route.
- 2026-10-06 — maintainer authorized all companion functional scope; capability-informed function, result-block and read contracts are included before dependent changes. Assembled verification follows full functional implementation.

- 2026-10-07 — maintainer-authorized Plan 28f–28h execution records confirmed production RC01 before dependent identity changes; artifact association and complete outer observation have separate lifetimes. Proposed status and qualification remain unchanged.
