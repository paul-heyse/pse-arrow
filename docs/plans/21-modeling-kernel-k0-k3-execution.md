---
title: Modeling kernel K0–K3 execution
status: done
date: 2026-09-26
adrs: [ADR-0097, ADR-0098, ADR-0099, ADR-0100, ADR-0101]
parent: docs/plans/21-modeling-kernel.md
review_sources: [docs/design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md]
---

# Modeling kernel K0–K3 execution

Authorized 2026-09-26. This packet owns execution status; Plan 21 owns adopted finding
dispositions. The detailed execution agreed in the conversation is condensed here without
creating a second architecture authority.

## Sequence and progress

| Packet | Owner and work | Acceptance | State |
|---|---|---|---|
| K0 | ADRs, target review, parity 2.13, amended target boundaries | Document-level target acceptance; version preflight; maintainer ADR route | implementation and preflight complete; formal ADR acceptance pending |
| K1 | authoring grammar, explicit identities, generated registry IR, package graph decoupling | Round trip, spans, identities, bounded parsing, strict generated contracts | implemented; targeted controls below |
| K2 | quantity schemes, generic kinds, pure definition/interface checking | Physical positive/negative controls; defaults; guarded members; table typing | implemented; targeted controls below |
| K3 | pure specialization, single compiler workspace, library math, accumulators, ports, lineage | Demand/dispatch/closure/derivative controls; incremental equals clean; finite execution | implemented; targeted controls below |

## Binding implementation decisions

- Durable values are registry-generated in `pse-model`; syntax trees are transient source views.
- `pse-modeling` has pure operations, `pse-compiler` the sole Salsa database, `pse-math`
  Symbolica integration, and runtime admission/effects. No native dependency in the pure kernel.
- Explicit IDs are allocated only by authoring actions, survive rename, and are preserved by
  canonical printing. Instance identity uses parent, child declaration and member identities.
- Bind explicit arguments, then presets, then defaults; explicit scope references search the
  nearest publishing ancestor. Defaults and overrides resolve before lazy dependency expansion.
- Structure depends on static bindings, guards, set membership, dispatch, mode and stage.
  Runtime numeric values cannot silently influence those decisions.
- Recursive expansion is a typed error; simultaneous equation cycles are separate structure.
  Lazy variable/equation bundles do not by themselves prove well-posedness.
- Conservation/accounting are generic indexed contracts. Transfers pair by scoped identity;
  independent closure retains original signed contributions.
- Winnow, petgraph, Salsa and Symbolica own their respective parsing, graph, incremental and
  symbolic mechanisms. No custom numerical evaluator or differentiation engine.
- Future constructs are typed and retained; unsupported execution names its missing capability.
- K3 exposes finite algebraic preparation through the existing typed mathematics path. K4–K9
  retain transformations, complete lowering, engines, conformance, science and campaigns.

## Retirement

Remove authoring's unnecessary physical graph dependency in K1. Remove generic runtime
composition branches only as consumers move in K3. External-provider shape belongs to K5,
dynamic authoring to K6, and scientific registry families, providers and vessel construction
to K8. No compatibility shim or second authority is introduced.

## Verification

**Tested:** Linux, pinned Rust 1.98.1; all Rust commands below include
`pse-relations/force-validate` through the recipes. Each final selected test scope has
**zero failures against a zero baseline**. These are targeted functional controls, not
workspace or product qualification.

| Command | Selected functional result |
|---|---|
| `just unit-package pse-authoring 'test(language::kernel_language::)'` | 6 passed: grammar, round trip, explicit identity, spans, limits, function signatures and partial selectors |
| `just unit-package pse-quantity 'test(generic_kind_tests::) \| test(index::)'` | 10 passed: generic kind admission, axis/subject identity and index operations |
| `just unit-package pse-modeling 'test(kernel_types::) \| test(kernel_specialization::)'` | 29 passed: typing, immutable data, demand, scope, overrides, indexed dispatch, accounting, ports, presets, annotations, exact integers and bounded expansion |
| `just unit-package pse-compiler 'test(workspace::modeling::tests::)'` | 11 passed: actual Symbolica values/derivatives, domain guards, empty indexed functions, function-reference tables, imports, structure, closure and incremental/clean equality |
| `just unit-package pse-compiler 'test(kernel_queries_reuse)'` | Salsa `WillExecute` events additionally confirm repeat-call memo reuse and recomputation after a value edit, while admitted mathematics is shared |
| `just unit-package pse-runtime 'test(workflow::modeling::)'` | 2 passed: original `.pse` document admission, version checks, immutable revisions, cancellation recovery and retained allocation release |
| `just py-test -m unit python/pse/tests/test_generated_contracts.py -k modeling_declaration_tag` | 1 passed on Python 3.14.7: exactly one tagged generated payload, typed nested fields and unknown-field refusal |
| `PSE_SOLVER_IMAGE=pse-kernel-parity-local:2026-09-26 just parity-container -m unit python/pse/parity/tests/test_00_preflight.py` | 5 passed on Python 3.13: IDAES 2.13.0, Ipopt 3.14 availability and recorded host probe |
| `PSE_SOLVER_IMAGE=pse-kernel-parity-local:2026-09-26 just parity-container -m unit python/pse/parity/tests/test_enum_contracts.py` | 45 passed, 5 component tests deselected: retained enum and class names against IDAES 2.13.0 |

**Implemented:** `just codegen`, `just conformance-fixtures` and `just py-sync` regenerated
the contracts/fixtures and refreshed the editable extension. Generation is not a behavioral
test. No integration, full solver journey, numerical parity, performance, formatting, lint,
governance or documentation campaign was run.

The preflight's local image is the pinned dev image
`ghcr.io/paul-heyse/pse-solvers:dev-e84fdce84e2f@sha256:39f6928ddcd3855ee097ac79244f87e3ebd4c9a2af371c44952e035e1ea45eb3`
with the Ubuntu `m4` build prerequisite installed. Its image ID is
`sha256:2c2c04679aecc845aa8940421c8057de9b1a84c4f31ce1993cd6af5d34ee1297`.
The published pin was not moved. Cold builds in the unmodified image still lack `m4`;
container maintenance owns incorporation and publication of that prerequisite. The parity
recipe now uses the container's `cc` and clears the host-only mold linker flags. This
preflight proves the reference environment, not numerical IDAES equivalence.

## Current checkpoint

The authorized through-K8 work corrected K1–K3 admission, visibility, immutable checked
contracts and reuse under E1 of the [K8 execution sequence](21-modeling-kernel-k8-execution.md).
That packet owns the corrections and their evidence. The earlier evidence in this packet
retains its original finite-kernel scope.

K1–K3 functional implementation is complete within the amended finite-kernel boundary.
K0's implementation, author target review and local reference preflight are complete;
ADR-0097–0101 remain proposed for the maintainer's decision-PR route. The authoritative
architecture collection is amended through the authorized decision/design route, recorded
in blueprint revision 59. Plan 21 retains the separate decision-PR route; the completed
K8 execution packet owns corrective conformance and assessment evidence. K9 is excluded.

### Implemented interfaces and ownership

- `pse-authoring::language` parses and canonically prints explicit-identity declarations.
  `authored.modeling_declarations` supplies generated Rust, Arrow and Python values.
- `pse-quantity` admits authored `EntityKindId` values for axes and subjects and checks
  complete quantity schemes. The physical reference package owns the initial kind IDs.
- `pse-modeling::check` checks the whole inventory before selection; `specialize` owns
  finite structure, default resolution, table admission, demand and immutable inspection.
- `SpecializedModel` retains instances, local body bindings, source/default/preset lineage,
  demand chains, original closure terms, ports and typed annotations. `assess_closure`
  consumes original contribution magnitudes separately from equation residuals.
- The existing `CompilerWorkspace` owns the Salsa inputs and queries. Runtime numeric
  initial values are excluded from finite body identity. Static bindings, selected
  function references and membership remain structural inputs.
- Function specialization flattens finite indexed arguments into explicit scalar formals,
  specializes structural arguments and shares selected bodies. Symbolica owns scalar,
  indexed, mixed and repeated partial derivatives and retains original domain obligations.
- `Runtime::modeling_from_documents` / `modeling_package` create immutable revisions;
  `ModelingPackage::prepare` uses existing CPU admission, cancellation, allocation ownership
  and compiler serialization. `PreparedModeling` exposes typed mathematics and existing
  library structural matching without claiming numerical rank or solving the model.

### Subsequent packet boundaries

K4 owns continuous transformations and implicit realization; K5 owns complete lowering,
per-group native evaluator assembly, verified piecewise and shaped external functions; K6
owns initialization and solve orchestration; K7 owns authored test execution and shared
conformance; K8 owns scientific knowledge and remaining legacy consumer retirement. K3
collects typed annotations for those consumers; collection does not claim their execution.
The new generic preparation API is Rust-facing; existing generated declarations are also
available in Python. No new Python solve facade or scientific parity coverage is claimed.

## Outcome

### What was built

**Implemented:** the generated language/IR, generic physical kinds, pure checker and finite
specializer, and their single compiler/runtime preparation path described above.
**Tested:** the targeted commands and conditions in Verification, with zero final failures.
F01–F05 dispositions belong to the parent plan, linked to this evidence.

### A mistake made and corrected

A generated unitful zero was initially ambiguous among quantity types sharing a unit.
Compiler output contracts and typed constant specialization now retain complete physical
meaning after source spans are stripped, including an empty polymorphic indexed sum.
A second control exposed precision loss in structural integers above 2^53; source literals
now retain exact integer values and checked integer arithmetic rejects overflow. Neither
fix substitutes dimensional equality for a complete physical contract.

### Deviations from the plan, deliberate

The original deletion list is staged by actual consumer replacement, as accepted in the
K0 target review. Authoring's physical-graph dependency and native closed-kind bindings
were replaced; scientific composition, dynamics and providers remain until K5/K6/K8 supply
their replacements. No compatibility compiler was added. K3 supplies finite admission and
inspection through existing math/structural libraries; it does not pull later solve or
scientific qualification into this packet. Comprehensive qualification remains a separately
requested activity under the current repository workflow.
