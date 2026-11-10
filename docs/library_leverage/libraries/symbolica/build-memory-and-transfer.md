# Build costs, memory and mathematical product transfer

Inspected 2026-11-09 at `d9e82627f` with unrelated concurrent Plan72 changes present.
Repository observations are **Interface-checked**; improvement directions are **Proposed**.
No timing or memory measurement is claimed. This is contextual discovery, not a target selection.

## Separate the costs before selecting a mechanism

| Cost | Current evidence | Useful research direction and countercase |
| --- | --- | --- |
| Clean Rust compilation | The workspace pins maintained Symbolica 3.0.1; `pse-math` imports it with GMP/MPFR. Numerica and Graphica are transitive locked packages. | A numerics-only consumer may use a companion directly and avoid the symbolic dependency closure. Changing import spelling inside a crate that still uses atoms/evaluators removes no closure. Trace actual target/host units before proposing a split or dependency change. |
| Incremental Rust compilation | Mathematical consumers reach the narrow `pse-math` integration. Generic domains and fixed dual macros can introduce specialization; dependency boundaries affect rebuild reach. | Compare stable generic numerical kernels with runtime-shaped programs. Less bespoke source is a maintainability gain, not evidence of less monomorphization or faster incremental builds. |
| Mathematical preparation | Shared definitions, symbolic partials, Horner/CPE, dual expansion and exact conversion are preparation distinct from Cargo. Existing code already shares joint evaluator outputs and exported facts. | Preserve compatible definitions/products across calls and give Symbolica a complete operation. A wider joint product may prepare unused derivatives or retain more backing; preparation reuse must include all consumed dependencies. |
| Runtime native compilation | `pse-math/compiled-evaluator` explicitly enables supplier code generation. `native_compiled.rs` owns compiler processes, output admission, module loading and scratch. | JIT/direct translation or shared compiled modules could avoid selected source/compiler work. Short-lived workloads can favor interpretation. Source generation, JIT startup, cache qualification and executable retention all count. |
| Repeated evaluation | Interpreted and admitted scalar compiled products already exist. Native C++ uses an explicit conservative floating profile, including no fast math or FP contraction. | Batch/SIMD kernels may amortize calls for many independent inputs. CUDA needs a suitable large workload and actual hardware; it does not accelerate symbolic preparation. Verify every function and numerical setting before extrapolating export support to production eligibility. |

Source owners: [manifest](../../../../crates/pse-math/Cargo.toml),
[library construction](../../../../crates/pse-math/src/library.rs),
[derivative construction](../../../../crates/pse-math/src/execution.rs),
[native compilation](../../../../crates/pse-math/src/native_compiled.rs).
The [numerical composition topics](../numerica/briefing.md) explain fixed/dynamic shape and backend conditions.

## Reduce representation work at its actual lifetime

Borrowed `AtomView` traversal, native collection/replacement and named aliases can avoid
materializing a second tree or repeatedly expanding shared expressions. The relevant question
is which repeated conversion remains after existing prepared-product reuse, not whether every
owned expression can become a borrow. A retained view that pins a large owner may cost more
than one compact copy.

Polynomial `zero_with_new_context` and `clone_with_context_of` can isolate reference-count
traffic between independent workers. They require compatible domains/variable maps; this is
not permission to silently change mathematical context. `compact` suits long-lived products,
not every temporary. Investigate only a workload actually sharing hot polynomial contexts.

`TermStreamer` suits many independent additive terms and explicit global normalization.
It adds disk serialization, sorting and merging. Shared Helmholtz definition graphs, global
factorization/GCD and coupled matching are revealing countercases: streaming their scalarized
expansion can discard the structure that made the operation manageable. Parallel transformation
must fit the existing operation owner and workstation policy; no smaller host cap is proposed.

## Transfer the right product

A formatted expression, packed atom, numerical instruction program, compiled module and guarded
admitted body have different meanings and receiving requirements. Text parsing can be simpler
for an infrequent human exchange; repeated trusted mathematical transfers may benefit from
native persistence. Neither a codec feature nor matching bytes establishes safe global symbol
registration, normalized destination ordering, bounded materialization or physical proof custody.

The current maintained supplier includes inspected packing and numerical-transfer facilities.
The newer instruction importer consumes already allocated structures and builds new validation
and evaluator backing. It is a candidate trusted program interface, not a passive-byte admission
mechanism. Preserve current symbol attributes/callback and namespace requirements when comparing
native state export/import. Numerical transfer cannot recreate a symbolic body for later class
evidence or differentiation.

The [Plan66 Symbolica dispositions](../../../plans/66-integrated-simulation-target.md#symbolica-improvement-dispositions)
own reconstruction, derivative sharing, precision and composition status. This broader inquiry
keeps alternative mechanisms eligible without reopening an implementation package. The supplier
provenance concern in the original November9 review has since been addressed by its owner;
it is not a new repair proposed here.

## Discriminating next operations

- Trace two real consumers of the same local derivative shape and qualified mathematical region; identify the exact construction removed and retained lifetime added.
- Trace a numerical-only consumer's compiler units with and without symbolic reach before treating direct Numerica/Graphica use as a build improvement.
- Compare one independently reusable provider-free module chain with one joint evaluator, including preparation, guard placement, input/output maps and current cancellation.
- For a transfer consumer, state whether it needs numerical execution only or subsequent symbolic work; compare the complete producer/receiver lifecycle on that requirement.

These questions can often be settled through source and interface inspection. Timing is warranted
only for consequential uncertainty; no benchmark campaign is required to justify a plausible
simplification. Existing libraries are the baseline option, and retained local policy must earn
its place through a concrete contract gap.

## Integrated numerical and structural preparation

The `f6e91eb37` extension identifies actual supplier-owned scalar-buffer packing adapters,
not only a hypothetical application loop. Their contracts differ; see the
[packed evaluator account](packed-evaluator-contract.md). Native JIT/C++ feature reach can
add compilation even when it removes host packing. The packed interpreter can avoid runtime
native compilation, but introduces numeric specialization and requires compatible lane/control
semantics. Callback fallback may still execute scalar functions independently per lane.

Structural template correspondence can also move the cost boundary: canonical graph conversion
and mathematical equality qualification earn value only if they remove preparation that survives
current definition-level reuse. Native indexed Symbolica canonicalization is another complete
route when the science is a genuine contraction. Preserve alias sharing through ordered formal
bindings rather than flattening repeated motifs merely to compare them. Exact matrix extraction
similarly needs comparison against sparse candidate-driven evidence, including conversion and
retained coefficient context. No measured clean-build, incremental-build or execution gain is
inferred from these source-established integration seams.
