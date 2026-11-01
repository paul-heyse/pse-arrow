---
title: "Plan63: Code-intelligence and CLI robustness"
status: proposed
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/README.md
doc_topics: [current-work, design, libraries, diagnostics, validation]
review_sources: [docs/functional_reviews/code-intelligence-cli-robustness-2026-11-01.md]
adrs: [ADR-0233, ADR-0235]
---

# Code-intelligence and CLI robustness

## Purpose and baseline

**Proposed:** make existing agent investigations dependable by removing unnecessary source-detail
work, retaining useful failure causes, clarifying scope/readiness and qualifying native request
settlement. The source [robustness review](../functional_reviews/code-intelligence-cli-robustness-2026-11-01.md)
is consumed at revision `f2a6c1afe9f98aad204efcae33b027873acce43a`. That clean revision is also
the authoring source baseline. Its review examined production source `91f255e4d`; the difference
is review publication, not a changed product implementation.

Plan creation is authorized; production execution is not authorized by publishing these documents.
Plans59–62 remain completed foundations with their original qualification boundaries.
Blueprint §21.7 owns enduring contracts. Core3.4, efficiency heuristics1.0, test-assessment1.2,
process-simulator1.5 and the binding govern this proposal; no scientific acceptance follows.

The review measured a 4,280,601-byte two-entry outline whose full-file facts dominated output.
It also corrected the initial graph diagnosis: request-relative paths selected `/tmp/crates`.
Fixing those paths reached native evaluation but retained incomplete evidence. Plan63 must
correct unnecessary work and poor explanations without converting expected refusals into bugs.

## Combined target and foundation assessment

An agent discovers the command/schema, selects a scope and asks a bounded question. Execution
constructs only requested evidence, preserves source/session identity, and returns either useful
results or a classified refusal with safe recovery facts. Output finality, examination
completeness, scientific permission and physical settlement keep their different meanings.

The existing owners are suitable foundations:

- `source_read` already owns parsed source, facts and exact ranges, but constructs full-file
  facts after entry paging and before addressed-range filtering. Move demand before construction.
- `investigation::config` already owns precedence/path bases and exposes Figment origins.
  Preserve that interpretation through execution rather than create another path resolver.
- The investigation error/native Failure/public envelope owners already classify/protect evidence.
  Extend their structured facts; do not invent a parallel error hierarchy.
- Native requests use one prepared world and request-local cancellation groups. Preserve this
  isolation while qualifying scope lowering, stopping and cancellation propagation.
- Proto generation and direct generated-field projection already exist. O08 strengthens their
  actual consumers; it is not a separate protobuf rewrite.

The coordinator owns common semantic decisions and opportunity disposition. The companions own
local package state:

| Companion | Responsibility |
|---|---|
| [63a](63a-demand-driven-source-evidence.md) | Navigation, selected source facts and decoded mappings |
| [63b](63b-query-diagnostics-and-agent-recovery.md) | Resolution provenance, diagnostics, public consumption and common diagnostic declarations |
| [63c](63c-native-query-scope-and-settlement.md) | Readiness, reference/symbol/graph scope and actual native settlement |

## Shared decisions and rule impacts

1. Source detail is execution demand, distinct from payload field projection and record allowance.
   Ordinary outlines become navigation-only; explicit detail retrieves facts/mappings.
2. Request-file/saved-query path bases remain unchanged. Missing explicit roots refuse with their
   resolved operand and origin. Races, permissions and snapshot exclusions remain distinguishable.
3. Direct source and guarded-result inspection retain their different freshness premises.
   Search/source/association/graph responses keep their typed shapes and advertised schemas.
4. Reads never implicitly prepare tools, macros or worlds. Recovery suggests only an action
   capable of changing the actual limitation.
5. Native reference occurrence scope is restricted inside the selected world where representable.
   A result limit cannot become an unsound early native candidate cap.
6. The operator selected **investigate, then decide** for the native symbol-search extension on
   2026-11-01. C03 requires an implement-or-retain disposition, not unconditional extension delivery.
7. Cancellation request, observed result and actual settlement are separate. Siblings retain
   their independent cancellation authority.

No governance/rule exception is selected. These corrections fit the current ownership,
explicit-effect and proto/presentation separation. The source-detail default is an intentional
public behavior change: A01 migrates schemas, examples and consumers with it. No compatibility
shim retains eager extraction. Conforming wire changes use authored proto, generation and protocol
fencing; B01 names required routes before dependent code. ADR-0233/0235 remain proposed.
If implementation exposes an actual change to guard/context authority, request-relative semantics,
settlement guarantees or another recorded rule, present that specific impact for confirmation
before dependent work; routine refactors and tests do not require new approval gates.

## Opportunity dispositions

The source IDs refer to the review linked above. Scheduled is not implemented or resolved.

| Opportunity | Disposition | Work owner | Required evidence or trigger |
|---|---|---|---|
| O01 | scheduled | A01–A03 | Selective construction, mappings and ordinary-read size controls |
| O02 | scheduled | B01/B02 | Preserved path origins and classified root/race/context cases |
| O03 | scheduled | B01/B03 | Typed cause/correction through every selected projection |
| O04 | scheduled | B04 | Executed schema-backed agent journeys |
| O05 | scheduled | C01/B03 | Accurate readiness reasons and explicit useful recovery |
| O06 | scheduled | C02/C03 | Native scoped references, graph honesty and symbol disposition |
| O07 | scheduled | C04 | Deadline/disconnect/cancellation through actual settlement |
| O08 | scheduled | A03/B03/C04/Q01 | Producer-local and composed native/public contract controls |

## Dependencies and execution route

A01's agreed detail contract and B01's agreed failure/provenance contract can be designed
independently. Their implementations must migrate their first consumers with their producer.
A02/A03 do not wait for the entire B companion. B02 supplies resolved scope/origin to B03 and
C01; C01 supplies actual prepared membership to C02. C03's symbol investigation and C04's
settlement trace can begin from existing native owners without waiting for source-detail changes.
B04 examples develop against each working slice, and finish against the assembled system.

Q01 consumes implemented selected behavior, authored integration controls and documented
investigation outcomes, not merely agreed interfaces. B04's assembled execution evidence is
produced within this one scope-end campaign; B04 and Q01 close from the same original results
without requiring two runs or making B04's prior qualification a prerequisite.
Shared edit surfaces include CLI dispatch, investigation models, native proto,
generation and schema snapshots; coordinate their declaration owners when work is parallel.

### Q01 — Assembled acceptance and enduring handoff

Run a source-only cold journey, request-file error/correction journey, prepared native
reference/graph journey and cancellation/reuse journey through the actual CLI. Consume real
schemas and both output streams; preserve original failures and incomplete outcomes.
Include a doc/string-heavy source and enough excluded symbol candidates to reveal wrong stopping.
Do not require complete graph evidence from an intentionally restricted world.

After functional scope, run the relevant host/source/native feature and integration tests,
required static/manual checks and documentation publication under repository execution policy.
Use the supported correctness selector and actual force-validation closure; no unrelated
scientific campaign is added. Record commands, scope, zero baseline, actual failures and exclusions.
Separate source-output byte measurements from latency/memory claims. No performance percentage
or universal response-size guarantee is a completion requirement.

Move enduring contracts to blueprint §21.7 through its design route, operational guidance to its
existing developer/skill owners and executable examples to the CLI assurance owner. Update
ADR rationale only through its applicable route. Preserve the source review while this series
consumes it, then apply normal lifecycle rules without treating old records as a backlog.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| Q01 | open | Coordinator | Assembled behavior, selected qualification and enduring handoff |

## Verification and current checkpoint

**Proposed:** A/B/C tests and Q01 establish future acceptance. Review measurements and probes retain
their original conditions and provide no Plan63 implementation credit.
**Interface-checked:** authoring examined existing source/fact, resolution, native task and
generation seams; library mechanisms and limits are developed in the companions.
No product code, tests, dependencies or generated outputs changed during plan creation.

Next executable work, after execution authorization: A01 and B01 contract/consumer slices,
plus C03/C04's bounded native investigations. Local package evidence belongs to its companion;
this coordinator alone owns combined closure and source-opportunity disposition.

Authoring validation: the existing `xtask lifecycle validate` returned no errors; its open-scope
projection recognized Plan63's opportunity/package owners. `just docs` and `just turn-end`
completed successfully. `git diff --check` initially found a trailing blank line in the new
lifecycle declarations; it was removed and the rerun passed. The final failure count is zero
against the zero target. These checks establish document wiring/publication only.
Independent read-only plan advice was incorporated for dependency slices, omitted-detail meaning,
test timing, association/graph consumption and interpreter freshness.
