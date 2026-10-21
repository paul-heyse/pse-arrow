---
title: "Plan48: Recommendation coverage"
status: done
date: 2026-10-20
adrs: []
review_sources: [docs/design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md]
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [design, performance, preparation, numerical-execution]
---

# Plan48: Recommendation coverage

The [coordinator](48-execution-performance-and-numerical-efficiency.md) owns all live
finding, rule and inquiry dispositions. This appendix maps all 162 atomized rows of the
[source register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md)
exactly once, preserving the audit's historical positions. It creates no additional tasks
or status ledger. The audit's corrected action/preservation text remains the source for
each row; its target behavior is developed in the linked package.

**Adopt** can mean preserve an already implemented capability, rather than build it again.
**Revise** schedules only the corrected contract. **Investigate** schedules the finite
question and conditional qualified adoption, not an unknown winner. **Omit** schedules no
implementation of the rejected claim; its links identify preserved behavior, corrected
context or the acceptance rule which prevents accidental reintroduction. Numeric target
omissions do not impose hidden SLAs. Repeated source proposals share packages and inquiries.

RC rows at the end map external identifiers to the twelve audit decisions accepted by the
operator on 2026-10-20. Those decisions retain their stated limits; they do not authorize
product execution. AF01–AF18 and N01–N04 have their own complete disposition mapping in
the coordinator, including additional items not present as register rows.

| Source recommendation | Audit position (historical) | Package / inquiry / decision route |
| --- | --- | --- |
| R1-F01 sparse implicit matrix construction | Revise | [48c/C02](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| R1-F01 factor reuse before reconstruction | Revise | [48c/C03](48c-mathematical-products-and-compiled-execution.md#work-packages); [I07](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC04](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-F01 zero-direction validity traversal | Investigate | [48c/C03](48c-mathematical-products-and-compiled-execution.md#work-packages); [I07](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| R1-F02 direct sparse HVP candidate | Adopt | [48c/C04](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| R1-F03 parent indexes and request-consumer map | Adopt | [48c/C01](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| R1-F03 physical grouping | Revise | [48c/C01](48c-mathematical-products-and-compiled-execution.md#work-packages); [48d/D05](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I17](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| R1-F04 native step semantics | Adopt | [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages); [RC01](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-F04 native default/refinement policy | Investigate | [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I12](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| R1-F05 conic working tolerance | Revise | [48d/D02](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I13](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| R1-F06 IDAS allocation reuse | Revise | [48e/E01](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I16](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC10](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-F06 callback scratch/borrows | Adopt | [48e/E02](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| R1-F07 parallel anchored shooting windows | Adopt | [48e/E03](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| R1-F08 compatible batch partition | Adopt | [48f/F01](48f-scheduling-resources-and-result-completion.md#work-packages) |
| R1-F09 local event-ready queue | Adopt | [48f/F02](48f-scheduling-resources-and-result-completion.md#work-packages) |
| R1-F10 lane-local Compact capture | Adopt | [48f/F03](48f-scheduling-resources-and-result-completion.md#work-packages) |
| R1-F10 separate commit/capacity stage | Investigate | [48f/F06](48f-scheduling-resources-and-result-completion.md#work-packages); [I30](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC12](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-F11 indexed original-quality recovery | Adopt | [48f/F09](48f-scheduling-resources-and-result-completion.md#work-packages) |
| R1-F12 bounded multi-function modules | Adopt | [48c/C05](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| R1-F13 coalesced ordinary observation | Revise | [48g/G02](48g-diagnostics-and-review-publication.md#work-packages); [RC03](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-F14 truthful Uno receipt | Adopt | [48d/D03](48d-native-controls-starts-and-recycle-policies.md#work-packages) |
| R1-C01 coupled DAE preconditioner | Revise | [48e/E05](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I21](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC05](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-C02 vector/JIT/device expansion | Investigate | [48c/C08](48c-mathematical-products-and-compiled-execution.md#work-packages); [I11](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC06](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-C03 shared local/durable bulk dispatch | Revise | [48f/F05](48f-scheduling-resources-and-result-completion.md#work-packages) |
| R1-C03 bounded durable commit stage | Investigate | [48f/F06](48f-scheduling-resources-and-result-completion.md#work-packages); [I30](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC12](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-C03 presentation-independent preparation key | Revise | [48f/F04](48f-scheduling-resources-and-result-completion.md#work-packages); [48f/F05](48f-scheduling-resources-and-result-completion.md#work-packages); [I28](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P01 eager whole lexical environments | Revise | [48a/A01](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P01 candidate-local memo | Revise | [48a/A02](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P01 kind/extent indexes | Adopt | [48a/A01](48a-source-checking-and-incremental-preparation.md#work-packages); [48a/A03](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P01 existing exhaustive oracle | Revise | [48a/A01](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P01 universal exponent and 2,048-size sweep | Omit as prerequisite | No implementation of omitted claim; preserve/interpret through; [48h/H02](48h-qualification-and-design-publication.md#work-packages); [I36](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P02 “no incrementality/any edit invalidates everything” | Omit | No implementation of omitted claim; preserve/interpret through; [48a/A03](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P02 finer tracked inputs/early cutoff | Revise | [48a/A03](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P02 pointer-or-hash equality | Revise | [48a/A03](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P02 retention scaling with N | Revise | [48a/A03](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P02 library durability | Revise | [48a/A03](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P02 signature-only separate compilation | Investigate | [48a/A05](48a-source-checking-and-incremental-preparation.md#work-packages); [I01](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P02 immutable checked-artifact cache | Revise | [48a/A05](48a-source-checking-and-incremental-preparation.md#work-packages); [I01](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC02](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P02 whole Salsa database persistence | Investigate | [48a/A05](48a-source-checking-and-incremental-preparation.md#work-packages); [I02](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC02](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P02 read-only phase/import parallelism | Revise | [48a/A06](48a-source-checking-and-incremental-preparation.md#work-packages); [I03](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P02 Salsa `rayon` feature as prerequisite | Omit | No implementation of omitted claim; preserve/interpret through; [48a/A06](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P03 description, canonical-SQL and invariant memoization | Adopt | [48b/B01](48b-registry-and-receiving-validation.md#work-packages) |
| P03 member-hash closure digest plus unchanged fingerprints | Revise | [48b/B01](48b-registry-and-receiving-validation.md#work-packages) |
| P03 lazy BOOTSTRAP | Revise | [48b/B01](48b-registry-and-receiving-validation.md#work-packages) |
| P03 generated frozen production catalog | Revise | [48b/B04](48b-registry-and-receiving-validation.md#work-packages); [I04](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P03 rkyv/bincode/zero-copy image | Investigate | [48b/B04](48b-registry-and-receiving-validation.md#work-packages); [I04](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P04 shared DomainEnvironment | Adopt | [48b/B02](48b-registry-and-receiving-validation.md#work-packages) |
| P04 prepared templates | Revise | [48b/B03](48b-registry-and-receiving-validation.md#work-packages) |
| P04 new trusted pointer handle | Revise | [48b/B03](48b-registry-and-receiving-validation.md#work-packages) |
| P04 full validation only at external ingress | Revise | [48b/B03](48b-registry-and-receiving-validation.md#work-packages) |
| P04 one scope per RunResult | Investigate | [48b/B03](48b-registry-and-receiving-validation.md#work-packages); [48b/B05](48b-registry-and-receiving-validation.md#work-packages); [I05](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P04 typed authored-document loader | Investigate | [48b/B05](48b-registry-and-receiving-validation.md#work-packages); [I06](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC11](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P06 indexed blocks | Adopt | [48c/C01](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| P06 grouped tiny blocks/session per shape | Revise | [48d/D05](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I17](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P06 parallel preparation | Revise | [48c/C01](48c-mathematical-products-and-compiled-execution.md#work-packages) (independent immutable numerical requests; separate from source-phase A06) |
| P09 coloring/selected sparse entries | Revise | [48c/C02](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| P09 retained coefficients/factors | Revise | [48c/C03](48c-mathematical-products-and-compiled-execution.md#work-packages); [I07](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC04](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P09 exact JVP count = number of colors | Omit as universal requirement | No implementation of omitted claim; preserve/interpret through; [48c/C02](48c-mathematical-products-and-compiled-execution.md#work-packages); [48h/H02](48h-qualification-and-design-publication.md#work-packages) |
| P10 multi-output arithmetic/CSE | Revise | [48c/C06](48c-mathematical-products-and-compiled-execution.md#work-packages); [I08](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P10 guards afterwards / f64 does not trap | Omit | No implementation of omitted claim; preserve/interpret through; [48c/C06](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| P10 exponent normalization | Revise | [48c/C06](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| P10 value reuse across orders | Revise | [48c/C06](48c-mathematical-products-and-compiled-execution.md#work-packages); [I08](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P10 provider contractions/reverse actions | Investigate | [48c/C08](48c-mathematical-products-and-compiled-execution.md#work-packages); [I10](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P10 instance batching | Revise | [48c/C06](48c-mathematical-products-and-compiled-execution.md#work-packages); [I08](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P11 packed scalar compilation | Adopt bounded form | [48c/C05](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| P11 one model-sized unit/all orders | Omit as universal policy | No implementation of omitted claim; preserve/interpret through; [48c/C05](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| P11 parallel compiler children | Revise | [48c/C05](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| P11 persistent compiled artifacts | Revise | [48c/C07](48c-mathematical-products-and-compiled-execution.md#work-packages); [I09](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P11 compiled default / JIT | Investigate | [48c/C07](48c-mathematical-products-and-compiled-execution.md#work-packages); [48c/C08](48c-mathematical-products-and-compiled-execution.md#work-packages); [I09](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [I11](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC06](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P11 bit-identical outputs | Revise | [48c/C05](48c-mathematical-products-and-compiled-execution.md#work-packages); [48c/C06](48c-mathematical-products-and-compiled-execution.md#work-packages) |
| P18 typed declaration masking | Adopt | [48a/A04](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P18 interned IDs and streaming hashing | Revise | [48a/A01](48a-source-checking-and-incremental-preparation.md#work-packages); [48a/A04](48a-source-checking-and-incremental-preparation.md#work-packages); [48b/B01](48b-registry-and-receiving-validation.md#work-packages) |
| P18 JSON only at boundaries | Revise | [48a/A04](48a-source-checking-and-incremental-preparation.md#work-packages) |
| P18 faster global allocator | Investigate | [48g/G06](48g-diagnostics-and-review-publication.md#work-packages); [I35](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P05 single coordinator means serial native solves | Omit | No implementation of omitted claim; preserve/interpret through; [48f/F03](48f-scheduling-resources-and-result-completion.md#work-packages); [48f/F08](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P05 spawned lanes | Investigate beyond lane-local fix | [48f/F08](48f-scheduling-resources-and-result-completion.md#work-packages); [I32](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P05 one shared admission | Revise | [48f/F04](48f-scheduling-resources-and-result-completion.md#work-packages); [I28](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P05 event frontier and batch partition | Adopt | [48f/F01](48f-scheduling-resources-and-result-completion.md#work-packages); [48f/F02](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P05 persistent common work-stealing pool | Investigate | [48f/F08](48f-scheduling-resources-and-result-completion.md#work-packages); [I32](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P05 width = cores | Omit as default | No implementation of omitted claim; preserve/interpret through; [48f/F07](48f-scheduling-resources-and-result-completion.md#work-packages); [48f/F08](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P05 parallel member pre/post | Revise | [48f/F04](48f-scheduling-resources-and-result-completion.md#work-packages); [48f/F08](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P07 whole-study scans each action | Omit blanket diagnosis | No implementation of omitted claim; preserve/interpret through; [48f/F05](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P07 introduce durable dependency counters | Omit duplicate mechanism | No implementation of omitted claim; preserve/interpret through; [48f/F05](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P07 500 ms charged per productive point | Omit | No implementation of omitted claim; preserve/interpret through; [48f/F05](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P07 heartbeat starts a supervisor | Omit | No implementation of omitted claim; preserve/interpret through; [48f/F05](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P07 warm managed primary missing | Omit | No implementation of omitted claim; preserve/interpret through; [48f/F05](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P07 batch discovery/claims | Investigate | [48f/F05](48f-scheduling-resources-and-result-completion.md#work-packages); [I29](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P07 live/push wakeups | Revise | [48f/F05](48f-scheduling-resources-and-result-completion.md#work-packages); [I29](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P07 separate commit/publication | Investigate | [48f/F06](48f-scheduling-resources-and-result-completion.md#work-packages); [I30](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC12](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P08 accounting/RSS ratio proves bad bound | Omit | No implementation of omitted claim; preserve/interpret through; [48f/F07](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P08 use computed bounds | Revise | [48f/F07](48f-scheduling-resources-and-result-completion.md#work-packages); [I31](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P08 replace opaque prepayment with growth | Investigate per producer | [48f/F07](48f-scheduling-resources-and-result-completion.md#work-packages); [I31](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P08 pool-scoped stack charging | Revise | [48f/F07](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P17 events only when detail selected | Revise | [48g/G03](48g-diagnostics-and-review-publication.md#work-packages) |
| P17 prepared fact handles/lean counters | Revise | [48g/G02](48g-diagnostics-and-review-publication.md#work-packages); [48g/G03](48g-diagnostics-and-review-publication.md#work-packages) |
| P17 one deadline check | Omit | No implementation of omitted claim; preserve/interpret through; [48g/G02](48g-diagnostics-and-review-publication.md#work-packages); [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages) |
| P17 universal native handle retention | Omit | No implementation of omitted claim; preserve/interpret through; [48d/D04](48d-native-controls-starts-and-recycle-policies.md#work-packages); [48e/E01](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P17 POUNCE-convex pool per batch | Omit blanket diagnosis | No implementation of omitted claim; preserve/interpret through; [48f/F08](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P17 HiGHS reset on every release | Revise | [48d/D04](48d-native-controls-starts-and-recycle-policies.md#work-packages); [48f/F08](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P17 quality ID map | Adopt | [48f/F09](48f-scheduling-resources-and-result-completion.md#work-packages) |
| P19 aggregate ordinary Salsa events | Revise | [48g/G02](48g-diagnostics-and-review-publication.md#work-packages) |
| P19 reliable phase closure | Adopt revised | [48g/G01](48g-diagnostics-and-review-publication.md#work-packages) |
| P19 lossless SPSC rings | Investigate | [48g/G04](48g-diagnostics-and-review-publication.md#work-packages); [I34](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P19 missing stage spans | Revise | [48g/G04](48g-diagnostics-and-review-publication.md#work-packages) |
| P05/P07/P08/P17/P19 numeric gates | Omit as requirements | No implementation of omitted claim; preserve/interpret through; [48h/H02](48h-qualification-and-design-publication.md#work-packages); [48f/F07](48f-scheduling-resources-and-result-completion.md#work-packages); [48g/G02](48g-diagnostics-and-review-publication.md#work-packages) |
| P12 KINSOL scaled-step interpretation | Adopt | [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages); [RC01](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P12 native roundoff step floor | Investigate | [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I12](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P12 explicitly call max-step API | Revise | [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages) |
| P12 one custom polish step | Omit | No implementation of omitted claim; preserve/interpret through; [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages) |
| P12 physical/native cone tolerance + bounded retry | Revise | [48d/D02](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I13](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P12 POUNCE-convex same relative defect | Investigate | [48d/D02](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I13](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P13 primal-only start is nearly cold | Omit | No implementation of omitted claim; preserve/interpret through; [48d/D03](48d-native-controls-starts-and-recycle-policies.md#work-packages) |
| P13 truthful receipts and compatible seed transport | Adopt | [48d/D03](48d-native-controls-starts-and-recycle-policies.md#work-packages) |
| P13 small barrier/push for any primal seed | Investigate | [48d/D04](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I14](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P13 Ipopt same-structure reuse | Investigate | [48d/D04](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I15](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC07](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P13 numeric versus structural options | Revise | [48d/D04](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I14](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P13 retained Uno handle | Investigate | [48d/D04](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I16](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC10](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P13 seeded iteration ratio target | Omit as requirement | No implementation of omitted claim; preserve/interpret through; [48d/D04](48d-native-controls-starts-and-recycle-policies.md#work-packages); [48h/H02](48h-qualification-and-design-publication.md#work-packages) |
| P14 native Anderson availability | Adopt as existing capability | Preserve existing capability through; [48d/D06](48d-native-controls-starts-and-recycle-policies.md#work-packages) |
| P14 Anderson m=3–5/delay1 default | Investigate | [48d/D06](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I18](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC08](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P14 Wegstein/Broyden | Investigate | [48d/D06](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I19](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P14 Newton tears | Revise | [48d/D06](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I19](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P14 indexed port exchange | Adopt | [48d/D06](48d-native-controls-starts-and-recycle-policies.md#work-packages) |
| P15 compatible IDAS storage/scratch reuse | Revise | [48e/E01](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I16](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P15 ReInit retains KLU symbolic | Omit | No implementation of omitted claim; preserve/interpret through; [48e/E01](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P15 once per (t,y) Jacobian | Omit weak key | No implementation of omitted claim; preserve/interpret through; [48e/E02](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P15 staggered corrector / native h0 choice | Investigate | [48e/E06](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I22](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P15 statistics per segment | Revise | [48e/E06](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I22](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P15 anchored CSC pattern | Adopt | [48e/E02](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P15 accepted-step-only guards | Omit | No implementation of omitted claim; preserve/interpret through; [48e/E02](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P15 omit backward reinitialization | Omit universally | No implementation of omitted claim; preserve/interpret through; [48e/E06](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I23](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P15 block/ILU/Schur preconditioner | Revise | [48e/E05](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I21](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes); [RC05](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| P16 no GN structure | Omit | No implementation of omitted claim; preserve/interpret through; [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P16 dedicated LM/TR least-squares route | Investigate | [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I24](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P16 constrained Ipopt GN | Adopt existing route | Preserve existing capability through; [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P16 exact Hessian only covariance | Omit | No implementation of omitted claim; preserve/interpret through; [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P16 curvature portfolio | Adopt | [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P16 always integrate sensitivities on objective | Omit | No implementation of omitted claim; preserve/interpret through; [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P16 demanded trajectory/sensitivity reuse | Revise | [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P16 automatic forward/adjoint | Investigate policy | [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I25](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P16 parallel experiments/windows | Revise | [48e/E03](48e-dynamics-fitting-and-preconditioning.md#work-packages); [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| P16 inexact integration tolerances | Investigate | [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I26](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| P16 final/profile assessment reuse | Revise | [48e/E07](48e-dynamics-fitting-and-preconditioning.md#work-packages) (assessment allocation/preparation and fresh independent checks); [48e/E08](48e-dynamics-fitting-and-preconditioning.md#work-packages) (shooting optimizer maps) |
| P16 shooting warm starts / retained optimizer | Investigate | [48e/E08](48e-dynamics-fitting-and-preconditioning.md#work-packages); [I27](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| §4.1 “all native stopping is stagnation” | Omit | No implementation of omitted claim; preserve/interpret through; [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages); [48d/D02](48d-native-controls-starts-and-recycle-policies.md#work-packages); [48e/E06](48e-dynamics-fitting-and-preconditioning.md#work-packages) |
| §4.4 tiny block cutoff / setup interval | Investigate | [48d/D05](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I17](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| §4.4 SPRAL/MUMPS, sparse-direct/Krylov choices | Investigate | [48d/D07](48d-native-controls-starts-and-recycle-policies.md#work-packages); [I20](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| §4.4 tangent predictor and native warm state | Revise | [48d/D03](48d-native-controls-starts-and-recycle-policies.md#work-packages); [48d/D04](48d-native-controls-starts-and-recycle-policies.md#work-packages) |
| §4.5 2,000–5,000-equation north-star flowsheet | Investigate follow-up | [48h/H02](48h-qualification-and-design-publication.md#work-packages); [I36](48-execution-performance-and-numerical-efficiency.md#investigation-outcomes) |
| §§4/11 universal compile-once/persist/publish-off-path | Revise | [48a/A05](48a-source-checking-and-incremental-preparation.md#work-packages); [48c/C07](48c-mathematical-products-and-compiled-execution.md#work-packages); [48f/F06](48f-scheduling-resources-and-result-completion.md#work-packages) |
| R1-RC01 | Adopt revised | [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages); [RC01](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-RC02 | Investigate | [48c/C08](48c-mathematical-products-and-compiled-execution.md#work-packages); [RC06](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-RC03 | Adopt revised | [48g/G02](48g-diagnostics-and-review-publication.md#work-packages); [RC03](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-RC04 | Adopt revised | [48e/E05](48e-dynamics-fitting-and-preconditioning.md#work-packages); [RC05](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R1-RC05 | Investigate | [48f/F08](48f-scheduling-resources-and-result-completion.md#work-packages); [RC09](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R2-RC01 | Adopt revised | [48a/A05](48a-source-checking-and-incremental-preparation.md#work-packages); [RC02](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R2-RC02 | Investigate | [48c/C08](48c-mathematical-products-and-compiled-execution.md#work-packages); [RC06](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R2-RC03 | Adopt | [48d/D01](48d-native-controls-starts-and-recycle-policies.md#work-packages); [RC01](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R2-RC04 | Adopt revised | [48c/C03](48c-mathematical-products-and-compiled-execution.md#work-packages); [RC04](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R2-RC05 | Investigate | [48d/D04](48d-native-controls-starts-and-recycle-policies.md#work-packages); [RC07](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R2-RC06 | Investigate | [48d/D06](48d-native-controls-starts-and-recycle-policies.md#work-packages); [RC08](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R2-RC07 | Adopt revised | [48e/E05](48e-dynamics-fitting-and-preconditioning.md#work-packages); [RC05](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
| R2-RC08 | Adopt revised | [48g/G01](48g-diagnostics-and-review-publication.md#work-packages); [48g/G02](48g-diagnostics-and-review-publication.md#work-packages); [RC03](48-execution-performance-and-numerical-efficiency.md#rule-decisions) |
