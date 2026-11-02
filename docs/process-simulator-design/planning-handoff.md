---
title: "Planning handoff for GPT-6.1 Sol"
---

# Planning handoff for GPT-6.1 Sol

Scope a repository implementation plan that turns the existing native modeling system into a coherent process-authoring experience, then grows scientific coverage through complete engineering cases. Use the imported design to understand the target and current source to establish the implementation baseline. Choose the smallest coherent changes that preserve the engineering meaning.

## Essential reading

Read [AGENTS.md](../../AGENTS.md), [current work](../plans/README.md), the [architecture reading guide](../authoritative_design/sections/reading-guide.md), the local [plan-creation](../../.codex/skills/plan-creation/SKILL.md) and [create-plan](../../.codex/skills/create-plan/SKILL.md) skills, and the [selected standard](../design_review/design_principles/standard.toml). Apply Core 3.4, its efficiency guidance, ProcessSimulator 1.5, the repository binding and the test-assessment companion where evidence design matters. This planning inquiry does not initiate a broad diagnostic audit.

Then read:

- [Simulation overview](simulation/overview.md), [hierarchy](simulation/hierarchy-and-references.md) and [answer contracts](simulation/answer-kind-contracts.md).
- [Process structure](process/structure-and-connectivity.md), [materials/states](process/materials-states-and-physical-models.md), [cases](process/cases-specifications-and-studies.md) and [cross-industry contracts](process/cross-industry-contracts.md).
- [Architecture and authoring](delivery/architecture-and-process-authoring.md), [coding-agent brief](delivery/coding-agent-brief.md), [scientific sourcing](delivery/scientific-sourcing-and-catalog.md), [implementation sequence](delivery/implementation-sequence.md) and [qualification](delivery/equivalence-and-qualification.md).

Use the catalogs and portfolio references selectively as questions arise; the [index](index.md) routes them.

## Fixed direction

- Keep one authored scientific authority, immutable source revisions and existing admission/publication. A process-facing concept is a role, contract or projection over those identities; a concept row does not require its own Rust class or database table.
- Use a targeted Rust scientific pipeline within pse-arrow and its existing SurrealDB instance. Select content from actual case demand. The earlier thermo-knowledge dependency has been superseded.
- Canonical scientific definitions and data may live as admitted native objects in SurrealDB. Discovery indexes those exact revisions. Prepare data for repeated numerical evaluation through existing mathematical/native owners.
- Build meaningful equipment templates from reusable physical structure, balances, states, closures and subordinate models. Zero or more balance regions are allowed. Rating and design reuse a physical definition through different specifications.
- Keep Model, Specification, Query and MethodRequest meanings separate. Physical procedures, inventory carryover and campaigns differ from computational Study orchestration.
- Preserve exact material/phase/basis/datum, provenance, model capability and requested-output meaning. Solver convergence, original physical acceptance, reference correspondence and engineering applicability have separate evidence.
- Retain P0–P7 and the initial A–D delivery. Use broader industry contracts to avoid restrictive assumptions; select additional industry implementations as finite increments.

## Establish the live baseline

The import review verified GitHub main at `19e7b49a67df24c1bf73005a6186e145e930d53a`. The local checkout was already ahead at `f8b7d74d8232da58bf485e6e16f5bfde969c2dd7`, including Plan63. Refresh both before planning. Do not allocate Plan63 again or treat the imported source citations as a claim about a later tree.

Inspect the current [model/composition owner](../authoritative_design/sections/models-and-composition.md), [physical semantics](../authoritative_design/sections/physical-semantics.md), [publication owner](../authoritative_design/sections/identity-and-publication.md), [workflows/results](../authoritative_design/sections/workflows-and-results.md) and [scope limits](../authoritative_design/sections/scope-and-open-design.md). The [organization strategy](import-strategy.md) maps relevant stable sections.

Decisive source seams for the first delivery are:

| Responsibility | Existing source to inspect |
| --- | --- |
| Authored revision and selected process inspection | [modeling.rs](../../crates/pse-runtime/src/workflow/modeling.rs), [documents.rs](../../crates/pse-runtime/src/workflow/modeling/documents.rs) and [process specialization](../../crates/pse-modeling/src/specialize/process.rs) |
| Declared demands, Solve and initialization reports | [declared.rs](../../crates/pse-runtime/src/workflow/modeling/declared.rs) and [case resolution](../../crates/pse-runtime/src/workflow/modeling/cases.rs) |
| Local Study and explicit recording | [study.rs](../../crates/pse-runtime/src/workflow/study.rs) |
| Source admission and canonical publication | [canonical.rs](../../crates/pse-runtime/src/workflow/modeling/canonical.rs), [delta admission](../../crates/pse-runtime/src/workflow/modeling/canonical/delta.rs) and [canonical staging](../../crates/pse-operations/src/canonical_staging.rs) |
| Discovery and generated storage | [modeling knowledge registry](../../crates/pse-schema/src/catalog/modeling_knowledge.rs) and its current consumers |
| Measurements, fitting and scientific publication | [research.rs](../../crates/pse-runtime/src/workflow/research.rs) and [research_publication.rs](../../crates/pse-runtime/src/workflow/research_publication.rs) |
| Conformance and independent references | [conformance.rs](../../crates/pse-runtime/src/workflow/modeling/conformance.rs), [reference drivers](../../tests/reference/oracles) and current parity recipes |

Preserve concrete API distinctions: `with_declarations` does not make old source text authoritative; `prepare_flow` selects a semantic topology view; `prepare_declared` is Solve-only; `initialize_declared` retains the full initialization report; `LocalStudy` has no serialization contract and recording is explicit. Verify these against the selected revision before designing consumers.

## First delivery to make concrete

Use the existing [recycle-flash definitions](../../packages/reference/campaign/models/recycle-flash.pse) and [exchanger-costing definitions](../../packages/reference/campaign/models/exchanger-costing.pse). Keep the ideal benzene/toluene recycle and the exchanger's BT_PR/relational-density configuration distinct.

| Packet | Engineering outcome to scope | Decisive acceptance |
| --- | --- | --- |
| A: ordinary inputs and scientific sourcing | Admit ordinary scientific inputs for those definitions, decide text/declaration editing, and close one selected source-to-Rust-to-SurrealDB-to-case path | Exact records, conventions, dependencies and eligibility survive publication/reopen; unchanged import preserves scientific identity; oracle-only inputs remain in their permitted route |
| B: inspection and edits | Inspect selected units, states, ports, connection occurrences, scientific configuration and case roles; support a display rename, numeric edit and topology edit | Stable identity; component-order independence; parallel edges and isolates; invalid connection attribution; incremental meaning agrees with clean reconstruction |
| C: complete native journeys | Recycle default success, intended initialization failure/restoration, and solved-area exchanger design; rating has its own explicit specification | Original residual/bounds/domain and independent balance checks; complete initialization report; restored specification; semantic output readback; explicit local/recorded effects |
| D: matched reference and P1 closeout | Isolated matched IDAES counterparts with complete mapped outputs and relevant failure assertions | Frozen physics/data/environment, dimensional comparison allowances, missing/wrong-output controls and independent analytic checks; required unavailable prerequisites fail and unrun scope remains unattempted |

Retain the exchanger's unknown-area and pressure-extrapolation allowances and the limits of its costing claim. Do not turn test-only data into ordinary inputs, hide retries, or use a converged result as evidence of broader physics. D's independent reference construction can proceed once scientific inputs and correspondence settle; integrated closeout depends on the native journeys.

For subsequent work, preserve the original sequence: P2 scientific foundations; P3 generic equipment and steady flowsheets; P4 inventories/distributed temporal physics; P5 specialized families; P6 analysis/control/multiperiod/surrogates/inspection; P7 population closure. Refine dependencies from actual consumers instead of implementing every catalog item in advance.

## Questions the plan must settle

1. **Scope and ownership:** reconcile the expanded target with `docs/relationship-to-idaes.md`, blueprint scope and the principles binding. Identify stale descriptions separately from actual rule changes. Preserve the clean-room boundary and route consequential changes through existing decisions.
2. **Editing and inspection:** choose the authoritative editing representation, selected instantiated scope, semantic identifiers and value-only versus structural/scientific edit behavior. Keep physical topology, equation incidence and execution decomposition separate.
3. **Scientific intake:** identify the first cases' exact missing ordinary records, original source roles/variants/fits and smallest source-specific adapter. Reuse typed deltas, admission, staging, activation and research owners. Decide only the derived discovery queries those consumers need.
4. **Composition:** show how the same exchanger supports rating/design and one compatible closure substitution. Identify any genuinely missing generic contract before changing language/schema/compiler/runtime owners.
5. **Broader semantic fitness:** challenge molecular/molar seed assumptions with a selected contrasting case, such as the custom liquid-liquid extractor. Preserve characterization, justified conservation, population representations and external effects without requiring all those implementations in P1.
6. **Acceptance population:** keep the 12 domain witnesses, 55 notebook families, release/configuration coverage and scientific/performance campaigns distinct. Freeze the actual required configuration/case population in P0, including non-notebook sources and explicit held/archive dispositions.
7. **Existing work:** identify dependencies on Plans28/32/40/53/54/58 and 58h without copying their status or absorbing their independent qualification. Current 200/1,000-case campaigns retain their existing owners.

## Expected planning output

Create one coordinator under `docs/plans/` using the next unused plan identity, with companions only where coherent responsibilities need their own explanation. Give it the target behavior, current source-based gaps, settled/open decisions, responsibility owners, dependency-ordered packets, replacements/deletions and appropriate verification. The suggested filenames and architecture destinations are in the [strategy](import-strategy.md).

Detail the first delivery enough to execute; retain later P2–P7 scope and dependencies without inventing every implementation detail. Fix public engineering behavior, scientific assumptions, outputs, expected refusals and evidence. Delegate routine module/type/helper organization and local sequencing. Use targeted compile/unit checks during implementation and integrated/static/reference qualification at the selected scope end under AGENTS.md. Quantitative performance benefits need representative measurements.

Return the proposed plan, decisions that constrain execution and the next ready packet. Use the repository's normal decision and authorization routes for subsequent implementation.
