# Independent efficiency audit evidence

This folder supports the [principal comparative audit](../../reviews/design_review_efficiency-alignment-audit_2026-10-11.md)
of the [original review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md).
The principal owns the combined judgment, stable finding and rule-impact identifiers, coverage
limits and priorities. These bounded inquiries neither disposition findings nor authorize
implementation. A future plan-creation step owns that decision.

## Baseline and method

The inspected checkout was `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` on
2026-10-11. Minor concurrent validation-script and Plan 28 status edits were preserved and,
as requested, were not made a separate audit subject. The original review, its ten inquiry
reports, operation map, briefs and coordinator handoff were leads to verify, not criteria.
The audit used Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5 selected by
`standard.toml`, with the pse-arrow binding and the functional target in profile Functional
target and blueprint §0/§0.2. Current decisions, policies and pins were treated as design
subject to rule impacts. Scope is product plus build, tests and development; documentation
tooling is excluded.

Five read-only design inquiries examined coherent responsibilities and adjacent consumers.
The coordinator read decisive source and reconciled shared contracts. A fresh principal
design reviewer, uninvolved in those inquiries, examined source independently and authored
the principal judgment. The coordinator publishes the returned reports with provenance
headers; their local candidate labels remain local. The assignment boundaries do not prescribe
the final argument or establish acceptance of their interactions.

| Supporting document | Bounded responsibility |
|---|---|
| [Inquiry briefs](inquiry-briefs.md) | Common conduct, target, questions and assigned boundaries |
| [Authoring, preparation and identity](authoring-preparation.md) | Original F01–F07, RC01–RC05; parser/quantity and observation extensions |
| [Studies, coordination and results](studies-coordination-results.md) | Original F08–F15, RC06–RC12; seed discovery, transport and live history |
| [Mathematical and native execution](mathematical-native-execution.md) | Original F16–F22, RC13–RC14; omitted numerical adapters, proof admission, surrogate and shooting |
| [Python, relational boundaries and assurance](python-relational-assurance.md) | Original F23/F24/F29, RC15–RC16; real engine consumers, IPC and Python boundaries |
| [Build, tests and development](build-test-development.md) | Original F25–F28, RC17–RC19; fixtures, ledger, admission, product generation and build dependencies |
| [Coordinator verification and reconciliation](coordinator-verification.md) | Bounded decisive checks, executed control, conduct and cross-inquiry reconciliation |

The principal's new finding identifiers reconcile local inquiry labels as follows. This is
navigation between evidence and judgment, not a finding-status ledger.

| Inquiry candidate | Principal treatment |
|---|---|
| B02 | [A01: incomplete latest-seed discovery](../../reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a01) |
| B01 | [A02: repeated admitted-seed transport](../../reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a02) |
| C-A01 | [A03: CPU allowance retained during IBEX exclusion waits](../../reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a03) |
| C-A02 | [A04: automatic full surrogate-state snapshots](../../reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a04) |
| C-A03 | [A05: shooting derivative demand](../../reviews/design_review_efficiency-alignment-audit_2026-10-11.md#a05) |
| D-N1 | Qualified C-stream interruption opportunity; no controlled signal/read observation |
| E-C01 | Narrow build-script invalidation opportunity; downstream rebuild and magnitude unmeasured |

## Evidence strength

The review relies mainly on static source inspection. Inquiries read applicable library
skills, used Context7/upstream documentation for discovery and checked pinned implementation
where release-specific semantics mattered. Current documentation is not silently treated as
proof of a different pinned version. Library use and ownership are assessed together, including
counterevidence to retiring wrappers with real consumed obligations. The stale utilization
JSONL, library-catalog MCP and capability maps were not used.

Source mechanisms are **Interface-checked**; corrective designs are **Proposed**. The single
new **Tested** control was:

```bash
scripts/pse-env -- just unit-package pse-buildinfo file_observation_cancellation_checks_each_read_chunk_and_never_seals --execution-effects native-local
```

Result against the zero-failure baseline: **1 passed, 0 failed, 17 unselected/skipped**, exit 0.
It exercised cancellation on a temporary 256 KiB file under the pinned checkout's isolated
Arrow-free library closure. The coordinator note records the conditions and exact scope.
The ordinary admission route was used after the maintainer arranged cancellation of the
other qualification run. No process belonging to another owner was stopped by this audit.

There are **no new performance measurements**. Earlier hash, ledger and import measurements
remain the original review's observations under its stated build/cache/host conditions. They
were not rerun or extrapolated into measured complete-operation latency. Proposed experiments
in the reports were not executed unless the coordinator note explicitly says otherwise. No
new numerical, canonical-store, solver, integration or whole-product qualification is claimed.

Coverage is substantial and selective. Each report names what it inspected and what it did
not. Unexamined breadth is not itself a finding. Decisive open premises are attached to the
remedy or judgment they affect; the principal review records the combined limits. The original
review remains a historical artifact with its original scope and findings, unchanged by this
audit.
