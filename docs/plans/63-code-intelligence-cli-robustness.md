---
title: "Plan63: Code-intelligence and CLI robustness"
status: complete
doc_retention: while-dependent
doc_retention_reason: "Highest-numbered series; current qualification readers consume this Outcome and its companion evidence."
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

The maintainer authorized execution of the complete series on 2026-11-01, starting from
`61538bafbc806e9acbae6931acd86674fa0dffc4`. Publication alone did not authorize that execution.
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
| O01 | resolved | A01–A03 | Demand-driven construction, selected mappings and actual CLI size/schema controls; 63a Outcome |
| O02 | resolved | B01/B02 | Typed origins and root/race/permission/symlink controls; 63b Outcome |
| O03 | resolved | B01/B03 | Shared redacted recovery/native-failure projection and exact selector attribution; 63b Outcome |
| O04 | resolved | B04 | Schema-backed discovery, explanation, streams, joins and guarded expansions; Q01 evidence below |
| O05 | resolved | C01/B03 | Typed native context reasons and independent interpreter/source premises; 63c Outcome and Q01 |
| O06 | resolved | C02/C03 | Scoped references and honest graph domains; retain uncapped symbol population under the C03 decision |
| O07 | resolved | C04 | Actual usage-search cancellation, gRPC controls, managed stop and macro drain; Q01 and 63c evidence |
| O08 | resolved | A03/B03/C04/Q01 | Generated protocol8 validation, native fidelity and actual public schema consumption; Q01 evidence |

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
| Q01 | complete | Coordinator | Assembled behavior, selected qualification and enduring handoff; Outcome below |

## Verification and current checkpoint

Source-detail demand, typed resolution/recovery and native scope/lifetime changes are implemented.
The source and initial recovery slices are committed; shared/native work is undergoing targeted
verification. C03 retains the correct complete symbol population for the reasons in its companion.
Native contracts are generated from authored proto and fenced by protocol8, including optional
typed context reasons and paired/ordered occurrence bounds. No dependency family moved.

Targeted verification exposed and corrected shared test constructors, a native-path JSON fixture,
an alias-call expectation outside the pinned analyzer's support, manifest traversal admission and
a socket accidentally changing a prepared fixture's inputs. Independent implementation review
also identified public native-failure operand disclosure and runtime traversal-origin attribution;
repairs share the existing disclosure owner and carry typed selector context respectively.
The native lifetime assurance now includes actual usage-search entry in addition to admitted-worker
controls. Assembled public/native journeys and scope-end qualification subsequently completed
through Q01; the Outcome below supersedes this execution checkpoint.
Local package evidence belongs to its companion; this coordinator alone owns combined closure
and source-opportunity disposition. Earlier review measurements remain separate from new evidence.

Authoring validation: the existing `xtask lifecycle validate` returned no errors; its open-scope
projection recognized Plan63's opportunity/package owners. `just docs` and `just turn-end`
completed successfully. `git diff --check` initially found a trailing blank line in the new
lifecycle declarations; it was removed and the rerun passed. The final failure count is zero
against the zero target. These checks establish document wiring/publication only.
Independent read-only plan advice was incorporated for dependency slices, omitted-detail meaning,
test timing, association/graph consumption and interpreter freshness.

## Outcome

**Implemented (2026-11-01):** the authorized A/B/C scope and Q01 are complete on local Linux.
The source slice is `a1e10b6e6`, initial recovery slice `576ae8580`, and integrated native,
wire, provenance and final source-root corrections `c396cd211`. Enduring contracts are in
blueprint §21.7; §24.2 records qualification scope. Developer/pse-cli guidance and the generated
parser reference explain source detail, coverage, both diagnostic streams and guarded expansion.
Companions retain package evidence and the C03 investigation; they do not authorize new work.

Source detail now precedes construction. Navigation avoids interpreted facts and mappings;
facts/mapped modes construct only the selected ownership/range scope. Cursors bind demand and
original bytes. Typed input origins survive execution; missing effective roots refuse before
Begin, while later source disappearance and native context changes retain different premises.
Exact selector/role attribution replaces operand-prefix guessing. Native failure operands and
recursive cancellation failures pass through shared disclosure, and optional recovery is boxed
at its owner rather than enlarging every diagnostic consumer.

Native references use same-world occurrence scopes before result caps; graph traversal and
output remain separate. The existing request-local CPU/macro lifetime mechanism is retained
and more deeply qualified. Proto8 is authored/generated, with required context-reason presence,
paired/ordered occurrence bounds, unknown-enum refusal and checked outgoing requests/failures.
No analyzer family, workspace dependency or vendor patch changed.

### Functional verification

**Tested:** all counts below have a zero-failure target. Original retained workflow results
contain exact compiler policy, selected identities, captures and final settlement. Counts refer
to their separate populations and must not be added as unique coverage.

| Command / selection | Final evidence and conditions |
|---|---|
| Host-control command below | 151/151 passed, 692 outside the selection; invocation `01a19a8c-6f35-764c-8e8c-dc8cc3cae8af` |
| `pse test pse-host --lib --features native-hir` with the 63c Targeted verification filter | 49/49 passed, including 28 C controls; `01a19a6d-70c5-73e4-83da-f0d888fa955d` |
| `pse test pse-host:source_addressing pse-host:source_commands pse-host:investigation_evidence pse-host:agent_interaction pse-host:composed_investigations` | Initial 31/36 passed; five repaired identities passed using `pse test --rerun-failed 01a19a77-b01f-70ac-b0fa-037cc2c98281`, result `01a19a84-1d73-733b-ac43-0607a070e7b9` |
| `pse test pse-host:agent_interaction` after the final effective-target repair | 6/6 passed, including the additional symlink journey; `01a19a8c-289d-7558-aa2c-942aea534e84` |
| Managed native command below | Initial 9/10 passed, including whole-session macro stop and session reuse; the repaired authenticated-cancellation identity passed using `pse test --rerun-failed 01a19a77-dcf4-727e-a284-8a02e6051567`, result `01a19a84-7242-70bb-ae15-fd4c3b8dfdab` |

The host-control command was:

```bash
scripts/pse test pse-host --lib -E 'test(source_read::) | test(source_command::batch::) | test(source_query::) | test(associations::source_detail_tests::) | test(investigation::providers::) | test(investigation::config::) | test(investigation::recovery_tests::) | test(cli::investigation::recovery_tests::) | test(envelope::) | test(semantic::wire::) | test(semantic::failure_tests::) | test(semantic::rpc::)'
```

The managed native selection was:

```bash
scripts/pse test pse-host --lib --features native-hir -E 'test(semantic::tests::native_snapshot_authenticated_cancel_settles_one_request_and_preserves_the_session) | test(semantic::tests::extended_native_managed_stop_drains_slow_macro_without_stopping_another_session) | test(semantic::tests::native_cli_reuses_original_session_fences_sources_and_proves_stop) | test(prepared_xtask::tests::actual_prepared_xtask_exports_same_image_parser_schema_and_document_owner) | test(semantic::native::tests::bounded_graph) | test(semantic::native::investigation_tests::occurrence_scope)'
```

These controls used restricted native worlds plus explicitly extended proc-macro worlds as
named by each test. Actual `FindUsages::search` entry and original Salsa local cancellation
establish in-flight native evaluation separately from admitted-worker barriers. gRPC Cancel,
half-close, lost output, deadline, parent cancellation, nested snapshots, siblings and fresh
queries preserve their distinct evidence. Managed macro stop and authenticated reconnect
exercise original process custody and physical drain. One intentionally ignored failing
fixture, `original_relevance_failure_fixture`, was outside the ordinary integration population;
it is not a passing control or a skipped required journey.

Available native tools did not substitute for document interpretation: the retained-interpreter
staleness unit passed, and explicit `pse tools prepare xtask` followed by source associations
returned an available export with no declared document/plan binding. Missing interpreters,
inactive Rust files, unsupported mappings and restricted graph domains remain qualified unknowns.
Source-only cold reads and missing-root explanations prepare no native service.

**Measured:** `scripts/pse inspect outline crates/pse-host/src/source_read.rs --budget 2
--format json` emitted 3,481 stdout bytes with navigation coverage and no facts. The public
heavy-source fixture separately passed its below-8,000-byte bound with 100,000-character docs
and a 100,000-character literal. These are output-size observations and construction controls,
not latency, RSS, universal size or percentage-improvement claims.

### Static qualification and publication

**Tested:** `just codegen` published all ten schema targets; the final authored copyright-header
change used `xtask codegen --only code-intelligence`. Final `pse run-all clippy-default
clippy-no-default codegen-check` passed every child (`01a19a8c-3bb6-72a5-a17c-c01d5b17fe1c`).
Native/compiler lint passed with:

```bash
scripts/pse-env --native=compiler -- scripts/cargo-bin/cargo-pse-correctness clippy -p pse-host --all-targets --features pse-host/native-hir,pse-host/compiler-analysis --locked -- -D warnings
CARGO_HACK_CARGO_SRC="$PWD/scripts/cargo-bin/cargo-pse-correctness" scripts/pse-env --native=compiler -- cargo hack check -p pse-host --all-targets --feature-powerset --depth 2 --locked
```

The host powerset passed all four combinations without warnings; this is not the full-workspace
powerset campaign. `just governance` passed its governance, generation and family children.
`just hygiene` initially failed license headers and both Clippy recipes; the other sixteen
children passed. The three missing first-party copyright notices were repaired and
`lint-license` passed; both workspace Clippy failures then passed their final affected reruns.
Thus every required hygiene child has passing evidence, without relabeling the original
aggregate green. Existing Plan60/62 copyright exclusions retain their original historical scope.
`just turn-end` passed both children. `pse tools run --selection
5a6d9c8d-b310-415a-915f-bc59261378b2 -- lifecycle validate` returned no errors after every
package/disposition closed. `just docs` published 1,000 chapters with scoped search. These
checks qualify document wiring/publication, not additional product or scientific scope.

### Corrections, deliberate choices and limits

The initial host selection `01a19a5d-d0cf-7112-812b-617aa4b22fa8` passed 104/105 with one
wire-fixture failure; native selection `01a19a62-fee4-72d4-b9fe-9b4fdcd80d3b` passed 28/31
with three failures. Their original failures remain retained and are superseded only by the
named corrected selections, not reclassified as passes.

A mistake made and corrected was treating source edits and host preparation as independently
safe: overlapping edits caused construction refusals before tests. Source freezes made the
prepared generation reviewable. Test-constructor migrations, a JSON attempt on a non-UTF-8
native path, unsupported alias-call expectations, manifest traversal admission and a socket
inside a prepared fixture were corrected. Initial assembled tests also exposed legitimate
omission of empty context and outdated fixtures attempting nested detachment; their repaired
journeys use actual independent caller scopes, preserving production ownership guards.
Static qualification caught diagnostic stack growth and test-helper/lint issues; repairs
changed ownership/placement rather than suppressing production checks.

Independent implementation review found two substantive gaps: raw contextual failure operands
could bypass disclosure, and traversal races could lose or misattribute known origins. Both
were repaired and reassessed without further actionable findings. Final effective-target
inspection additionally found dangling explicit file links could become no-match. Preflight
and both explicit-root rechecks now follow effective targets, with passive/refusal/positive/race
controls; recursive link traversal retains its own policy.

Deliberate choices are the complete native symbol population (C03), explicit file-restriction
plus original-range filtering for plural ranges, and retention of the existing cancellation
owner instead of adding a global registry/watcher. Alias-call enumeration remains limited by
the pinned analyzer; prepared-product changes in the shared input watch may classify as
`SourceChanged`. No bounded symbol-work, universal graph completeness, new scientific/parity,
deployment, other-platform, measured speedup or formal ADR acceptance claim follows. Older
semantic selectors require explicit stop/repreparation for protocol8 queries.
