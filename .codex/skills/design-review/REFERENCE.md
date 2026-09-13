# Design Review — Reference

Companion to [SKILL.md](SKILL.md). §1 is a lookup table. Everything after it is a set of lenses and calibration examples — angles that have turned up real defects before, offered because they are useful, not because the review owes them. The normative standard is `docs/design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md`.

---

## §1 Principle index (DM-01–DM-60)

Titles and requirement levels, for accurate citation and for gate reasoning. A MUST-level gap on in-scope behavior cannot be waived by an exception record — it narrows the supported scope or the design is unresolved against that requirement. A SHOULD-level deviation can be accepted with a §10 record.

### Group 1 — Semantic authority and the modeling boundary

| ID | Level | Title |
|---|---|---|
| DM-01 | MUST | Make meaning—not storage shape—the primary model |
| DM-02 | MUST | Assign one authority to each semantic fact and revision |
| DM-03 | MUST | Unify logical contracts without mandating one physical structure |
| DM-04 | MUST | Declare the semantic boundary and expose opaque behavior |
| DM-05 | SHOULD | Separate intent from mechanisms and incidental technology |

### Group 2 — Semantic types, schemas, and invariants

| ID | Level | Title |
|---|---|---|
| DM-06 | MUST | Type semantic distinctions, not only machine representations |
| DM-07 | MUST | Make validity rules explicit and enforce them at identified boundaries |
| DM-08 | MUST | Represent absence, unknowns, uncertainty, invalidity, and failure distinctly |
| DM-09 | MUST | Model relationships and valid domains explicitly |
| DM-10 | SHOULD | Keep important structure typed and queryable |

### Group 3 — Identity, versions, and consistency

| ID | Level | Title |
|---|---|---|
| DM-11 | MUST | Use stable semantic identity independent of physical location |
| DM-12 | MUST | Distinguish entity, revision, artifact, and execution identity |
| DM-13 | MUST | Separate definitions, specifications, policies, observations, and results |
| DM-14 | MUST | Publish semantically consistent revisions through explicit commit boundaries |
| DM-15 | MUST | Define canonicalization and equivalence before using content identity |

### Group 4 — Declarative composition and reusable structure

| ID | Level | Title |
|---|---|---|
| DM-16 | SHOULD | Represent material structure and policy as declarations |
| DM-17 | SHOULD | Use templates and bindings instead of copied construction logic |
| DM-18 | SHOULD | Preserve high-level structure until expansion is required |
| DM-19 | MUST | Select behavior through declared capabilities and explicit bindings |
| DM-20 | MUST | Make inspection and validation semantically non-mutating |

### Group 5 — Compilation, derivation, and semantic preservation

| ID | Level | Title |
|---|---|---|
| DM-21 | SHOULD | Use explicit intermediate representations and progressive lowering |
| DM-22 | MUST | Give every meaningful transformation a contract |
| DM-23 | MUST | Make derived representations traceable and non-competing |
| DM-24 | MUST | Preserve semantics across rewrites and lowerings |
| DM-25 | SHOULD | Unify operation contracts while allowing specialized implementations |

### Group 6 — Planning, execution, mutable state, and effects

| ID | Level | Title |
|---|---|---|
| DM-26 | SHOULD | Separate preparation from repeated execution |
| DM-27 | SHOULD | Represent important workflows as inspectable plans or state machines |
| DM-28 | MUST | Declare effects, ambient inputs, and nondeterminism |
| DM-29 | MUST | Isolate mutable workspaces and commit their outcomes explicitly |
| DM-30 | MUST | Make partial failure and recovery explicit |

### Group 7 — Dependencies, incrementality, and concurrency

| ID | Level | Title |
|---|---|---|
| DM-31 | MUST | Expose every dependency that can affect meaning or output |
| DM-32 | MUST | Key reuse to semantic dependencies rather than convenience |
| DM-33 | SHOULD | Invalidate at the smallest trustworthy semantic granularity |
| DM-34 | MUST | Keep different relationship structures semantically distinct |
| DM-35 | MUST | Make concurrency respect dependencies, ownership, and declared ordering |

### Group 8 — Execution representations and performance

| ID | Level | Title |
|---|---|---|
| DM-36 | SHOULD | Choose physical layouts for demonstrated access patterns |
| DM-37 | SHOULD | Cross expensive boundaries in coarse, typed units |
| DM-38 | SHOULD | Match the execution mechanism to the operation’s semantics |
| DM-39 | MUST | Evaluate performance end to end and distinguish evidence from expectation |
| DM-40 | MUST | Declare precision, approximation, ordering, and determinism requirements |

### Group 9 — Boundaries, providers, and extensibility

| ID | Level | Title |
|---|---|---|
| DM-41 | MUST | Keep adapters mechanical and domain conversions explicit |
| DM-42 | MUST | Make interchange loss-aware and reject silent semantic degradation |
| DM-43 | MUST | Negotiate capabilities and expose unsupported behavior |
| DM-44 | MUST | Make extensions complete, versioned, and conformance-testable |
| DM-45 | MUST | Treat trust and authority as explicit execution constraints |

### Group 10 — Provenance, reproducibility, and explainability

| ID | Level | Title |
|---|---|---|
| DM-46 | MUST | Preserve source-to-result lineage through transformations |
| DM-47 | MUST | Represent diagnostics as structured evidence |
| DM-48 | MUST | Define and support the required reproducibility contract |
| DM-49 | SHOULD | Make changes understandable at the level of meaning |
| DM-50 | SHOULD | Observe the model lifecycle, not only low-level operations |

### Group 11 — Evolution, generation, and verification

| ID | Level | Title |
|---|---|---|
| DM-51 | MUST | Evolve schemas and semantics through explicit migrations |
| DM-52 | SHOULD | Generate repeated mechanical artifacts from shared contracts |
| DM-53 | MUST | Verify invariants and equivalence across representations |
| DM-54 | MUST | Test adversarial lifecycle and boundary conditions |
| DM-55 | SHOULD | Make contracts and extension paths discoverable to humans and agents |

### Group 12 — Architectural leverage and disciplined improvement

| ID | Level | Title |
|---|---|---|
| DM-56 | SHOULD | Optimize for fewer independent semantic decisions—not fewer lines |
| DM-57 | SHOULD | Prefer a small coherent core with explicit extension mechanisms |
| DM-58 | SHOULD | Scale architectural machinery to demonstrated needs |
| DM-59 | MUST | Make design claims falsifiable and label uncertainty |
| DM-60 | MUST | Turn the principles into change-level review and regression controls |

### Scope-to-group routing

A starting orientation, not an allocation. It exists to help justify the §7 applicability note; the scope decides which groups actually matter.

| If the scope is… | Groups that often carry the findings |
|---|---|
| A new canonical model, schema, or catalog | 1, 2, 3, 11 |
| A pipeline, compiler, or pass architecture | 5, 6, 7, 10 |
| A storage or physical-layout change | 3, 8, 9 |
| A provider, backend, adapter, or FFI boundary | 9, 8, 5 |
| An incremental, caching, or recompute system | 7, 3, 10 |
| A workflow, orchestration, or execution engine | 6, 7, 10 |
| An extension, plugin, or registry mechanism | 4, 9, 12 |
| A refactor claiming leverage or simplification | 12, 4, 11 |

---

## §2 Lenses that tend to pay off

Nothing here is a required step. These are the angles that have repeatedly turned up real defects, and the note on what it takes for each to hold up as evidence rather than as a suspicion.

### When the subject is a document

**Reconstructing beats reading.** Building the template's tables yourself from the document — rather than checking whether the document's own version looks complete — tends to surface the gaps quickly, because every cell you have to invent is a decision the design hasn't made:

- The **authority table** (§2): one row per semantic fact, with owner, revision boundary, and permitted update path. The list of cells you had to invent *is* the evidence for the finding.
- The **invariant table** (§3), with an enforcement boundary and a failure behavior per invariant. An invariant with neither is unresolved (DM-07), whatever the surrounding prose claims.
- The **stage graph** (§4), with inputs, output contract, effects, and invalidation per stage. Undeclared effects and ambient inputs land under DM-28.
- The **absence lattice** (DM-08): for each value that can be missing, which of unspecified / unknown / not-applicable / uncertain / invalid / partial / failed does the design distinguish, and which collapse into one representation? States that collapse when callers need to tell them apart are a G2 concern.

**Sorting the load-bearing sentences** is a cheap way to keep claim strength honest:

| Kind | How it reads | What it deserves |
|---|---|---|
| Specification | States what holds, where enforced, what is rejected | Assess it directly |
| Intention | A desirable property with no mechanism — "the model is the single source of truth" | Unresolved until a mechanism is named |
| Assumption | Rests on an external system, library, or later decision | If not stated as an assumption, that's DM-59 |
| Benefit assertion | Performance, simplicity, extensibility | Hypothesis unless evidence is cited (DM-39, DM-59) — label it |

**The divergence sentence.** For a core mechanism, writing the one sentence two implementers would read differently is often the whole finding. Familiar shapes: "identity is content-based" (over which canonical form, with which normalizations — DM-15); "invalid rows are rejected" (at ingest, at query, or at publication — DM-07); "the plan is cached" (keyed on what, invalidated by what — DM-32); "conversion is lossless" (byte, structural, semantic, or approximate — template §3).

At `deep`, the counter-design is worth the time: the smallest alternative delivering the same observable outcome. If the proposal's extra machinery can't pay for itself against it, that's a DM-58 finding that usually outranks the local ones.

### When the subject is code

Six recurring defect shapes. The second column is what it takes for one to be reportable rather than suspected — below that bar, it belongs in the Method note as something you looked at and couldn't settle.

| Shape | What makes it evidence | Gate · principles |
|---|---|---|
| **Second authority** — the same semantic fact independently editable in two places (constant and schema; schema and validator; validator and adapter; migration and model; production default and a fixture encoding the same rule) | Both sites cited, plus the absence of a derivation, generation step, or assertion linking them. Two representations derived from one source are not a second authority — worth checking for the derivation first | G1 · DM-02, DM-23 |
| **Unguarded boundary** — an external input, partial construction, or deserialization reaching an operation whose correctness assumes an invariant | The entry point, the operation, and either no check between them or one that is advisory (logs, warns, opt-in) rather than rejecting. Type-level enforcement counts; say so and close it | G3 · DM-07, DM-22 |
| **Hidden effect** — `validate`, `check`, `inspect`, `plan`, `explain`, `analyze` paths that mutate, populate caches, lazily initialize, register, or read ambient state (clock, env, filesystem, global config, thread-local, RNG) | The mutation or ambient read cited, plus a caller that reasonably assumes purity. Memoization that can't change observable semantics isn't a finding — but say why you concluded that | G4 · DM-20, DM-28 |
| **Silent degradation** — default-on-absence, catch-alls flattening distinct failure classes, unsupported-feature branches with different semantics, lossy conversions (narrowing, truncation, timezone or precision drop, NaN/null/sentinel conflation) | The branch, what the caller observes in the degraded versus supported case, and no declared loss or approximation policy. A declared, selected approximation is aligned (DM-40, DM-42) | G2/G7 · DM-08, DM-42, DM-43 |
| **Incomplete reuse key** — a cache, memo, fingerprint, or incremental key missing something that can change the result: policies, provider versions, configuration, environment, flags, schema revision, upstream artifact identity | The key construction, the omitted dependency, and a situation where changing it yields a stale hit. Volatile inputs in the key — timestamps, paths, addresses — are the mirror-image defect and also DM-32 | G6 · DM-31, DM-32 |
| **Unbacked capability** — the accepted-operation surface (enum, trait, op registry, supported-types list, config schema, public API) wider than the lowering that handles it | The set difference, with the accepted variant and the missing or fallback handling both cited. An explicit typed `Unsupported` is aligned; a silent fallback to different semantics is G7. Documentation of coverage is the claim, not the proof | G7 · DM-43, DM-44 |

Tracing the path that executes — the `impl` actually selected rather than the trait's doc comment, the branch taken under the real configuration — is what makes these hold. Where dispatch is dynamic or config-dependent, saying which path was traced and which wasn't keeps the coverage note honest.

### When the subject is both

Beyond what each half yields:

| Question | Finding shape |
|---|---|
| Do the document's semantics and the implementation's match? | Divergence against the pair; name which is authoritative and how they reconcile (DM-02) |
| Does the document describe a state the code has passed through? | Stale specification — say whether the document is normative-forward (a target) or descriptive (a record); one that is neither is a competing authority |
| Does the code implement semantics the document omits? | Undocumented surface; DM-04, DM-55 |
| Does the document claim capabilities the code lacks? | G7 — narrow the claim or relabel it Proposed |
| What evidence label does each design claim now deserve? | Re-label per charter §D. A Proposed claim becoming Implemented is often this mode's most useful output |

---

## §3 Finding calibration

Adequate and inadequate versions of the same observation. The difference is consistently the same three things: a concrete consequence, evidence at the right grain, and citations that do work.

### A — Authority

**Inadequate.** "Violates DM-02: the schema definition appears in multiple places, which creates a single-source-of-truth problem. Recommend consolidating."

No sites cited, no demonstration that the two can drift, no consequence, and "consolidating" isn't a direction with a surface area.

**Adequate.** "Column nullability for the state-variable table is independently editable in two places: the Arrow schema built at `catalog/schema.rs:88` and the validation predicate at `ingest/validate.rs:214`, which re-lists the required fields as string literals. Neither derives from the other and no test compares them. **Consequence:** adding an optional field to the schema without editing the validator makes ingest reject valid rows; making a field required in the validator without editing the schema admits nulls that downstream kernels assume absent, panicking at `solve/assemble.rs:131`. **Correction:** derive the validator's required-field set from the schema (~2 call sites), or generate both from one declaration. **Verification:** a test asserting set equality between schema-required and validator-required fields; it fails today. DM-02, DM-07."

### B — Absence semantics

**Inadequate.** "DM-08 is not fully satisfied; the design should distinguish absence states more clearly."

**Adequate.** "The design represents an unsolved variable, a variable outside its declared domain, and a variable whose upstream computation failed all as a null in the result column (§4.3, 'unavailable values are null'). **Consequence:** a diagnostics consumer counting nulls to report convergence can't separate 'not yet computed' from 'computation failed' from 'not applicable to this unit', so a failed solve and a partial solve produce the same report. **Correction:** a typed status beside the value, or distinct null metadata — a one-declaration change if made before the result contract has consumers. **Verification:** a negative test asserting the three cases are distinguishable at the result boundary. DM-08, DM-30, G2."

### C — Over-construction

**Inadequate.** Silence — or praise for extensibility. This is the finding that most often goes unwritten.

**Adequate.** "§6 introduces a provider registry with dynamic capability negotiation for solver backends, but the design names exactly one backend and no near-term second. The registry adds a capability-declaration surface, a negotiation protocol, and a conformance-test obligation (DM-44) the design doesn't fund. **Consequence:** the first genuine second backend will need capability distinctions the current negotiation vocabulary can't express, so the registry gets rewritten rather than extended — the cost paid twice. **Correction:** keep the trait boundary and the typed `Unsupported` result (the cheap, justified seam); defer the registry and protocol until a second backend exists. **Verification:** none needed — this removes machinery. DM-58, DM-57, charter §F, 'speculative flexibility with no demonstrated consumer'."

### D — Evidence labels

**Inadequate.** "The design is validated end to end — see the integration tests."

**Adequate.** "Round-trip guarantee: **Tested** for the structural case (`tests/roundtrip.rs::schema_roundtrip`, 14 fixture schemas, exact structural equality) and **Proposed** for the semantic case — nothing exercises metadata preservation or extension-type identity, which §3 claims is preserved. Either narrow the claim to structural round-trip or add the metadata case before making it. DM-53, DM-59."

---

## §4 How the sections tend to compress

Template sections are `DESIGN_REVIEW_TEMPLATE.md` §1–§11. A sketch of what usually survives, not a rule — the scope decides.

| Section | Document subject | Code subject | At `compact` |
|---|---|---|---|
| §1 Decision and scope | Plus Method and coverage | Plus Method and coverage | Compressed; Method note still earns its place |
| §2 Authority and lifecycle | Reconstructed; invented cells marked | Reconstructed; where each authority lives | Prose, with the authority gaps named |
| §3 Contracts and invariants | Usually the core of the review | Enforcement sites cited | Often merges into §7 |
| §4 Derivation and execution | Full | Tracing real paths | Prose |
| §5 Journeys | Extension and failure at minimum | Same, through real code | One journey, chosen for relevance |
| §6 Gates | Tabular | Tabular | Tabular |
| §7 Findings + applicability | Tabular | Tabular | Tabular |
| §8 Alternatives | Worth the work at standard/deep | Worth the work at standard/deep | Optional; say why omitted |
| §9 Verification plan | Proposed checks | Existing coverage and gaps, named | Top gaps only |
| §10 Exceptions | Only if deviations exist | Only if deviations exist | Only if deviations exist |
| §11 Decision and changes | Required | Required | Required |

---

## §5 Appendix — where these shapes tend to appear in Arrow / DataFusion / Rust

Illustrative for this repository. The charter is technology-neutral and so is the review; this is a list of places the shapes in §2 have a habit of landing, and their absence proves nothing.

| Pattern | Shape | Principles |
|---|---|---|
| `Schema` / `Field` constructed in more than one module for the same logical table | Second authority | DM-02, DM-23 |
| `DataType` matched exhaustively in several places, each with its own coercion rules | Second authority, silent degradation | DM-02, DM-25, DM-42 |
| Sentinel floats (`NaN`, `-1`, `f64::MAX`) where Arrow null or a typed status is the meaning | Silent degradation | DM-08, DM-06 |
| `unwrap` / `expect` / `unwrap_or_default` at a deserialization or FFI boundary | Unguarded boundary | DM-07, DM-42 |
| UDF `signature()` / `return_type()` wider than the kernel actually handles | Unbacked capability | DM-43, DM-44 |
| Plan-node or `ExecutionPlan` equality/hash as a cache key without config, session, or provider version | Incomplete reuse key | DM-31, DM-32 |
| Optimizer or analyzer rules rewriting without an equivalence statement | — | DM-24, G6 |
| `TableProvider::supports_filters_pushdown` returning `Exact` for a filter the scan applies approximately | Unbacked capability | DM-43, DM-24 |
| Row-at-a-time loops or per-row FFI crossings where a `RecordBatch` is the natural unit | — | DM-37, DM-38 |
| Schema metadata (units, domains, provenance) dropped by a conversion, `project`, or `cast` path | Silent degradation | DM-42, DM-46 |
| Floating-point tolerance, ordering, or reduction-order assumptions left undeclared in numeric kernels | Silent degradation | DM-40, DM-48 |
| `Arc<Mutex<…>>` or interior mutability inside a path documented as pure planning or inspection | Hidden effect | DM-20, DM-28, DM-29 |
| Benchmarks measuring a kernel while the design's performance claim is end-to-end | — | DM-39, DM-59 |
