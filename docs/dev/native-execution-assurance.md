# Native execution assurance

Execution evidence is separate from functional qualification. The surviving generic
`pse-engine` owns native construction and execution; `pse-catalog` supplies Delta
planners, selected-source witnesses and lifetime guards. `pse-testkit` is development
only and uses the same `EngineResources` and `EngineFactory` constructors as runtime.
This guide covers relational/storage execution, not the mathematical compiler or
native solver lifecycle. The latter uses Salsa preparation and the runtime's joined
native supervisor. See [ADR-0073](../adr/0073-native-engine-resource-boundaries.md) for
the original engine boundary and the current [Plan 14 inventory](../plans/14-execution-inventory.md)
and [workflow guide](native-workflow.md) for status and selected consumers.

## Choose the evidence for the claim

| Question | Evidence | Limit |
|---|---|---|
| Did this invocation exhaust its stream successfully? | `pse.operation` identity, explicit terminal field and complete local capture | Span closure alone also occurs after failure or early drop |
| Did the intended native operator execute? | Correlated native operator spans and terminal outcome | Names describe observed nodes; they do not prove their domain meaning or input/output values |
| Did planning retain the intended implementation? | Actual planner/function/extension identity and native plans | A rule name or pretty-printed plan is not implementation identity |
| What properties, expressions and metrics did the native plan expose? | `pse_testkit::introspection::visit_physical`, native downcasts, `PlanProperties` and `MetricsSet` | Use native metric units and partition/execution scope; reused plans can retain cumulative metrics |
| Was the resulting process model correct? | Targeted value/contract units and independent functional oracles | Execution tracing alone does not establish scientific or numerical correctness |
| Was a Delta operation durably committed? | Native transaction/version evidence and application reconciliation | A successful span, IO request or absent error is not a durable receipt |

`Capture::assess(operation_id, expected_nodes)` returns `Established`, `Violated` or
`Inconclusive` for the narrow execution claim. Missing roots, fields or requested
operators, unknown task propagation, open spans and truncated capture are inconclusive.
Failed, cancelled and abandoned operations violate a successful-completion claim.
`Established` does not mean every relevant operator was requested or every functional
property was checked. Retain value, ownership, cancellation and durability tests.

## Observation modes

Call `EngineFactory::with_observation` before creating the session:

- `Off` is the default: no query diagnostic capture or plan text rendering.
- `Contract` adds native phase/operator metrics and explicit operation terminal states.
- `Diagnostic` also captures bounded compact plan text and rule names. It enables no
  row previews. Its reserved diagnostic budget can explicitly refuse oversized capture.

The actual caller planner, functions, extensions and semantic optimizer rules survive
observation. The execution instrumentation rule is appended after semantic rules.
Instrumentation is removed before rebinding a nested semantic assembly and applied
once to each execution copy. Observation does not change implementation generation,
semantic settings identity, value admission or the independent freshness flag.
Preparation phase spans describe preparation; the execution operation root starts at
invocation. Do not infer a common operation identity for earlier standalone preparation.

Native phase spans and metrics replace repeated string inspection where their contracts
answer the question. Diagnostic plan rendering remains useful for inspection behavior
itself. Native observations complement the physical, ownership and publication
witnesses in Plan 14's acceptance inventory. M22 qualifies those complete journeys.

## Isolated development fixture

`NativeFixture::new` constructs finite native memory, spill, caches and an actual engine
factory without a Delta root, solver or runtime dependency. `into_factory` retains the
spill directory's lifetime in the factory. Catalog fixtures explicitly compose
`pse_catalog::assembly::planners()` with the generic planner.

A local capture follows this pattern (the crate unit tests are the executable example):

```rust,ignore
use pse_engine::session::assurance::ObservationPolicy;
use pse_testkit::capture::Capture;
use tracing::instrument::WithSubscriber;

let capture = Capture::new(4096, 1 << 20);
async {
    let factory = fixture.factory.clone()
        .with_observation(ObservationPolicy::Contract);
    // Create a declared candidate, prepare it and fully consume its owned stream.
    // Drop operation owners before assessing span completion.
}.with_subscriber(capture.dispatch()).await;
// Select the actual operation_id from capture.snapshot(), then assess that invocation.
```

Capture has explicit span-entry and retained-text limits. Formatting is bounded before
retaining a field. Overflow cannot silently become a passing assertion. Use one local
subscriber per capture; span IDs are local to that subscriber. A local capture is not an
OTLP export or collector receipt. No exporter is required for development assurance.

The process-wide DataFusion task hook preserves both span and subscriber dispatch for
async and blocking `JoinSet` work. A foreign preinstalled hook is not assumed equivalent:
its evidence is inconclusive. The tracing library's attempted hook installation can emit
an expected `AlreadySet` warning after the engine installs its own hook.

## Storage observations

`EngineResources` registers instrumented native stores before cache identities are
assigned. Repeated lookup and re-registration of the same returned store preserve its
actual identity. New store registrations get their own wrapper. Externally prewrapped
stores are the caller's assembly responsibility.

Delta opening, write attempts, publication commits and maintenance carry existing
operation/attempt/version information in declared spans. The actual native result is
recorded after the operation. No new durable receipt or retry protocol is introduced.
Native object-store spans cover calls through the registered wrapper. `get` completion
and later payload consumption are different boundaries; list/multipart behavior follows
the pinned method contract. Cloud IO, retries and destructive maintenance remain separate
qualification obligations.

## Commands and boundaries

- `just dev-native-engine`: selected isolated engine/testkit units, Arrow force-validation.
- `just dev-native-boundaries`: selected static manifest and error-contract tests.
- `just engine-boundary-check`: resolved dependency DAG and legacy-path deletion checks.
- `just family-check`: one native type universe; independently released tracing is
  classified separately and its Apache DataFusion dependencies remain checked.

All failure baselines are zero. Plan 14's historical local Linux qualification is
recorded in [M22](../plans/14-m22-execution.md); current remediation and qualification
belong to the [Plan 16 execution packet](../plans/16-p14-p18-execution.md).
Targeted units and compilation support implementation. Full integration and quality
checks run at the end of a plan.

`just assessment <output>` collects the full local default/native/Python scope without
stopping at the first failure. `just native-test` and `just native-python <output>`
are ordinary linked execution commands; `just py-sync-native` refreshes the extension.
After functional qualification, `just case-measure <output> --functional-from <report>`
collects the declared case campaign using Criterion CSV samples. Compilation is untimed
setup. Reports distinguish fresh execution, unchanged-input reuse and explicitly
reviewed transfer; neither a source digest nor a review document establishes behavior.

## Pinned references

Use the [DataFusion/Arrow skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/SKILL.md),
[Delta skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/deltalake/SKILL.md) and expanded
[tracing skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion-tracing/SKILL.md). The latter's execution,
context, lifecycle, metrics, filtering and storage briefs distinguish source observations
from executed consumer probes. The project lockfile remains the consumer version authority.
