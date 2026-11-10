# Evaluator composition and execution discovery

Inspected 2026-11-09 at `d9e82627`, maintained Symbolica 3.0.1 and primary tagged 3.0.2 sources. **Interface-checked** source observations; **Proposed** research directions. No execution, compile probe or measurement. Earlier November 2 evaluator research retains its original evidence date.

## Already effective

`library::exact_evaluator` registers aliases, prepares multiple outputs together and selects direct translation to retain a shared instruction DAG. It configures optimizer limits and abort handling. A successful-but-truncated supplier result after abort is explicitly refused before export or retention. The adapter shares inspected representation facts. `native_compiled.rs` already provides an optional native compiler/module route; native arithmetic is not absent.

## Compose useful operations, not arbitrary semantic regions

Maintained `merge` concatenates outputs of evaluators with the same input convention. Its type bounds include `Eq + Hash`, suggesting exact-coefficient merging before conversion. It checks input count, not the semantic order of coordinates. Parameter correspondence remains caller meaning.

The 3.0.2 composer connects independently reusable programs without reconstructing symbolic expressions. Its potentially useful case is a real provider-free chain whose parts are needed independently. A single joint build is the countercase: it may already expose all useful sharing with less composition machinery. Reoptimization changes floating realization; eliminated arithmetic is not permission to skip a fallible provider or original guard. The [Plan66 disposition owner](../../../plans/66-integrated-simulation-target.md#symbolica-improvement-dispositions) retains composition scheduling; broad research does not activate it.

## Lifecycle restrictions change the useful design

The maintained `vectorize` rejects retained sub-evaluator bodies. Thus non-inlining to reduce program duplication can preclude later dualization; a strategy must consider the whole derivative pipeline. `set_real_params` must follow optimization and merge or its registrations can be lost. Neither real inputs nor a real-parameter declaration proves all results real. Stack reuse and CSE are separate improvements.

The tagged 3.0.2 importer checks structured instructions and nested programs; it is not a passive-byte pre-allocation gate, original-layout transfer or symbolic-body restore. Current maintained packing/f64 transfer and guarded receiving have distinct contracts. Replacing their machinery requires comparing the full receiving operation, not simply observing that import succeeds.

See the [evaluator contract](evaluator-contract.md) for reusable program boundaries and the newer composition/import interfaces.

## Backend investigation

Current native compiled scalar admission deliberately excludes unresolved callbacks, branching/control instructions, retained bodies and unsupported functions. Supplier C++/assembly/SIMD/CUDA/JIT capability is broader than this production profile, so expanding a backend is a candidate capability change, not merely a compiler flag.

A useful next case evaluates one provider-free function over many independent rows. Compare repeated interpretation, prepared native scalar execution and batch/SIMD or direct JIT translation under the same demanded output and numerical profile. Distinguish runtime compiler startup from mathematical optimization and Rust build time. Short-lived evaluation or operations outside the backend vocabulary can favor the interpreter. No universal speedup is claimed, and no benchmark is needed before a concrete consequential performance uncertainty emerges.

## Sources and next discriminator

Source: `vendor/symbolica/src/evaluate/{optimize.rs,dual.rs,instruction.rs,function_map.rs,export.rs}` and `crates/pse-math/src/{library.rs,native_compiled.rs,execution.rs}`. Upstream candidate sources: [composer](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.2/src/evaluate/compose.rs), [structured instruction importer](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.2/src/evaluate/instruction/import.rs).

Name the reusable pure chain and its actual consumer first. Determine whether its symbolic preparation is repeated, its inputs and scope are compatible, retained functions remain transformable, and backend lifetime makes native preparation worthwhile. Then applicability can compare current joint preparation, separate prepared modules and native composition without presupposing adoption.

## Where composition could earn its machinery

Consider an operation with a reusable constitutive module and several response products. One
consumer needs the module alone; another needs its output inside a residual/action program.
Rebuilding the second program from expanded expressions may repeat symbolic transformation and
Horner preparation. A composer can instead bind the already prepared module's outputs into the
second program, prune irrelevant outputs, and optimize the combined instructions. This is a
concrete capability beyond concatenating outputs with `merge`.

The complete comparison includes three credible routes: one joint expression build, separately
retained evaluators with host-level chaining, and library program composition. The first often
has the least machinery when independent reuse does not exist. The second can preserve cheap
module invalidation and avoid reoptimization. The third may share work across module boundaries
and remove repeated symbolic construction, but adds slot mapping and a composed product lifetime.
A symbolically equal module with different original guards, physical inputs or provider context
cannot be substituted merely because it returns the same number at one trial.

For derivatives, choose the requested mathematical product before choosing the physical program.
Keeping function bodies separate can reduce generated size, yet the inspected vectorization
restriction can require inlining before dual expansion. A large full derivative jet may be worse
than a selected action even when its arithmetic is well optimized. Real specialization must occur
after the transformations that rebuild its flags. These ordering constraints make an end-to-end
pipeline investigation more useful than independent endorsements of inlining, composition and AD.

The discriminating operation should therefore have independently consumed modules, known demand,
repeated preparation that survives current reuse, and a supported execution vocabulary. Inspect
its mathematical and operational dependencies first. Only a consequential unresolved tradeoff
needs a timing experiment; source evidence can establish deleted rebuilding without proving a
universal speedup.
