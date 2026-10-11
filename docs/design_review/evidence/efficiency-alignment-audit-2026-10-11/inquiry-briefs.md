# Independent efficiency audit: inquiry briefs

The maintainer approved this audit on 2026-10-11. Its principal document is
`docs/design_review/reviews/design_review_efficiency-alignment-audit_2026-10-11.md`.
These briefs describe the questions assigned, not conclusions or implementation work.

## Common brief

Audit `docs/design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md`
against the standard and functional target. Begin with the coordinator's context at
`docs/design_review/reviews/efficiency-alignment-audit-handoff-2026-10-11.md`.
The original evidence, ten reports, operation map and original briefs are under
`docs/design_review/evidence/efficiency-alignment-codebase-2026-10-11/`.
Treat all earlier judgments, including the handoff's doubts and prior dispositions, as leads.

Load `.agents/roles/worker.md`, the assigned role, `.codex/skills/design-review/SKILL.md`,
`.codex/skills/design-review-process-simulator/SKILL.md`, and the sources selected by
`docs/design_review/design_principles/standard.toml`: Core 3.4, Efficient Architecture 1.0,
Process Simulator 1.5, their template/review additions and the pse-arrow binding.

The target is interactive edit and re-solve without rebuilding structure for value changes;
sweeps, sensitivities, optimization and fitting; recycles and tears; dynamics from the same
definitions; new units, properties and solvers; and compiling/testing one responsibility with
its actual dependencies. Consider cold/warm operation, growth/skew, concurrency, live state,
cancellation and recovery where they reveal a consequential distinction. No invented SLA,
cost model, mandatory probe or exhaustive heuristic checklist is requested.

Only the standard and functional target judge the design. ADRs, blueprint decisions, plans,
policies and pins describe current design and may be overturned by a recommendation. Record
rule impacts, including what retaining a rule would leave unresolved. A repository rule cannot
waive a standard MUST. Preserve scientific meaning, required independent checks and truthful
outcomes when assessing faster or simpler alternatives.

Scope is the product and build/test/development loop. Documentation tooling is excluded by
the maintainer. Baseline is `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f`.
Minor concurrent validation edits in `scripts/test_resources.py`, its test and Plan 28e need
only a baseline note, not their own inquiry. Preserve all existing work.

Read-only investigation: no repository edits, builds, services, store mutations, campaigns,
formatters, dependency changes or further delegation. You are not alone in the checkout.
Source inspection and safe static queries are sufficient. Propose a light probe only when
it could change a diagnosis, remedy or priority; specify the question, command, conditions
and discriminating outcomes. The coordinator controls probes and must respect host admission.
Never stop or bypass another owner's work.

During the audit the maintainer arranged cancellation of the validation run and authorized
whatever experiments would help the audit. The coordinator verified the observer no longer
reported active native/store/build work. Inquiry agents still propose workload probes for
coordinator execution; source-only investigation does not wait for a probe.

For libraries, load relevant library skills and use Context7, upstream GitHub documentation/source
and pinned local source. Check actual pins/features. Do not consult
`docs/library-utilization.jsonl`, the library-catalog MCP or `docs/capability-maps/`.
Use `path::symbol` source references, stable architecture section IDs and version-specific
library references. A search with no hits is not enough to establish absence: inspect relevant
entrypoints, consumers and search coverage. Distinguish source evidence from executed tests
and measurements. Do not transfer timings between implementations or builds without evidence.

## Required return

Return a complete bounded report, with a scratch copy outside the repository if useful:

1. Scope inspected, sources and coverage limits.
2. A position on every assigned original finding, separating diagnosis from remedy.
3. Corrections to facts, corrective contracts, preservation, priority and prerequisites.
4. Positions on associated RC rows, including exact rule text/meaning where consequential.
5. Additional candidates under local inquiry IDs, each with one falsifiable cause, workload,
   consequence, standard identifiers, source evidence, correction, preservation and verification.
6. Strengths, contrary evidence and uncertain opportunities that are not findings.
7. Effects on foundation/gate judgments; proposed probes and unresolved decisions.

A sound diagnosis need not have a fully designed remedy. Do not manufacture findings from
unexamined breadth. Assess target operations independently before reconciling the original IDs.
The principal review will own final IDs and judgments; future plan creation owns disposition.

## A — Authoring, preparation and identity

Own comparison of F01–F07 and RC01–RC05; coordinate cross-boundary consequences of F23 with D.
Inspect the value/structural edit routes, starts/bounds and overlay lineage; selected closure
and absent-name dependencies; engagement/protection/settlement lifetime; compiler/Salsa reuse;
runtime/executable/native-generation observation. Read the original A/H and relevant L1/L2 reports.
Extend into parser and quantity inference where they materially affect the target. Challenge
stat/loader identity proposals and changed admission meaning rather than accepting their premise.

## B — Studies, coordination and results

Own F08–F15 and RC06–RC12. Read original D/E plus relevant H/L2 reports. Follow normal and
interrupted studies, retention and result reads, cancellation, publication, selection and seed
reuse. Distinguish actual shared writes, read locks, local pacing and index coverage. Challenge
proposed protocols with root-creation/reclaim phantoms, uncertain commits and cancellation races.
Assess output demand, heartbeat/seed traffic, simultaneous live cases and horizon state.

## C — Mathematical and native execution

Own F16–F22 and RC13–RC14. Read original B/C/L1 reports. Inspect callback layouts, evaluator
ownership, cubic/implicit execution, conditional units, initialization, root response and tears.
Extend into neglected adapters, IBEX isolation, continuation, IDAS adjoints, dynamics/shooting,
derived/factorable/collocation/surrogate mathematics and proof transport. Prioritize material
operations rather than claiming exhaustive source coverage. Challenge program/scratch/factor
reuse with branch, derivative, scaling, numerical-state and attempt-ownership distinctions.

## D — Python, relational boundaries and assurance

Own F23, F24, F29 and RC15–RC16. Read original F/L2 and relevant A/H reports. Trace actual
engine consumers and strong/weak assembly lifetime; validation context, Arrow crossings and
float parsing feature ownership. Extend into Python beyond boundary modules, runtime/import
construction, engine internals, stream pulls/GIL, conversions and cross-language live state.
Differentiate an absent public capability from machinery with real internal consumers.

## E — Build, tests and development

Own F25–F28 and RC17–RC19. Read original G and relevant H/L1 reports. Inspect actual dependency
closures, entry routes, test fixtures, ledger/runner cleanup and host admission. Extend into
product code generation, supervisor paths, script/product tests and parity setup. Counts are
supporting facts, not architecture verdicts. Preserve actual dependency and scientific checks.
Documentation builds and documentation lifecycle tooling are excluded.

## Principal review

A fresh design reviewer, uninvolved in A–E, receives their evidence and the coordinator's
verification/reconciliation. Independently read decisive sources, resolve or expose conflicts,
and write the comparative principal document. Cover every original F01–F29 and RC01–RC19,
new findings A01 onward and new rule impacts RC20 onward, all qualified by their owning review.
Reassess AP-01–AP-07, G1–G9 and PS-G1–PS-G3 at the actually inspected scope; do not inherit Revise
or any positive verdict. Separate scientific adequacy, architectural fitness, priorities and
prerequisites. Return the complete review to the coordinator for publication. No dispositions,
implementation changes or rule changes are authorized by this audit.
