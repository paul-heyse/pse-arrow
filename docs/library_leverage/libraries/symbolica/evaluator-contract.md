# Optimized evaluator boundaries

Reusable type: `symbolica/optimized-evaluator-boundaries`.
Research date 2026-11-02. **Interface-checked** against Symbolica 3.0.1 source and numerical
evaluation documentation, including the maintained source variant identified below.
No evaluator execution or timing probe was run.

`Atom::evaluator_multiple` builds one evaluator from several expressions. Joint optimization
can share common subexpressions across outputs. The evaluator is an execution representation:
Horner rewriting, common-subexpression elimination and intermediate-slot reuse can change
evaluation order and expression shape. The documentation explicitly describes fast-math
transformations. Mathematical equality alone does not prove identical floating-point results
or identical interval enclosure tightness.

`ExpressionEvaluator::export_instructions` exposes constants, temporary/output slots and a
structured instruction stream. It is useful for representation inspection or implementing a
selected downstream execution vocabulary. An importer must interpret arithmetic, powers,
functions, branches and sub-evaluators according to the exported domain. A function symbol
does not automatically convey an application's provider policy, auxiliary obligations or
fidelity requirements.

Export is an owned representation with allocation and lifetime costs; sharing a single export
for several inspections can avoid repeatedly reconstructing it. Export does not predict every
temporary allocation of evaluator construction, vectorization or code generation. Storage facts
from an already built representation are different from admission before construction.

Mapping coefficients changes the numeric domain. Preserve exact constants until the chosen
conversion boundary when exact reasoning is required, and qualify the specific conversion
routine rather than assuming that a name such as `to_f64` promises a desired rounding mode.
Likewise, a cooperative optimization abort is not automatically a typed cancellation failure
for every surrounding operation. Read the selected settings and failure path.

A useful comparison is two related outputs optimized jointly versus independently: assess
shared work, retained lifetime and numerical policy together. A revealing countercase is a
consumer that needs authored branch/obligation identity; an optimized program may omit that
identity even though it evaluates the same mathematical expression.

For the inspected maintained 3.0.1 supplier, the acquired surface adds two useful integration
details. `Atom::evaluator_multiple` returns an `EvaluatorBuilder`; the public-path record retains
its `AtomCore` import requirement. Preparation completes separately through fallible `build()`;
preserve its `EvaluationError`. Exported exact constants can contain placeholders identified
by `constant_functions`: resolve their symbolic definitions at the target precision before
using those slots. Match non-inlined function calls to `sub_evaluators` by symbol and tags,
recursively; unmatched calls remain external. These are source-variant conditions, not guarantees
inferred from the upstream version number. See the [maintained instruction source](../../../../vendor/symbolica/src/evaluate/instruction.rs)
and [pilot API evidence](../../pilot-evidence.md#api-facts). Numeric rustdoc item IDs are local
to that artifact and are not stable capability identities.

Sources: [numerical evaluation guide](https://symbolica.io/docs/numerical_evaluation),
[3.0.1 evaluation API](https://docs.rs/symbolica/3.0.1/symbolica/evaluate/index.html),
[3.0.1 instruction implementation](https://docs.rs/symbolica/3.0.1/src/symbolica/evaluate/instruction.rs.html).

## Selected composition contracts, refreshed November 9

**Interface-checked** against tagged 3.0.2 source; this adds a candidate-version account
without refreshing the entire November2 maintained-3.0.1 investigation.

`EvaluatorComposer` binds earlier outputs/constants to later inputs. `finish` requires
`Eq + Hash`, prunes dead work, interns constants and performs instruction CSE/CPE and stack
optimization without repeating symbolic Horner search. A single joint evaluator remains a
credible alternative when independent module reuse has no value. Parameter/coordinate meaning
is external to slot identity. [Tagged composer](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.2/src/evaluate/compose.rs).

`ExpressionEvaluator::from_instructions` validates structured control flow and definitions,
including nested bodies, then constructs evaluator backing. The instructions are already
allocated; this is neither a passive byte preflight nor an original-layout or symbolic-state
restore. Callback/function resolution and coefficient domain still matter.
[Tagged importer](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.2/src/evaluate/instruction/import.rs).

These contracts establish candidate mechanisms, not their suitability for a particular receiver
or a measured preparation benefit. No execution probe was run for this refresh.
