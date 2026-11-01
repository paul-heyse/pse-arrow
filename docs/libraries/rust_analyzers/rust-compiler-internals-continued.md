# Rust Compiler Internals: `rustc_mir_dataflow`, `rustc_borrowck`, and `rustc_mir_transform`

Technical capability reference | Rust 1.99.1 (stable) | Researched October 29, 2026

## 1. Architectural orientation

These three crates implement complementary parts of Rust's MIR-based analysis and transformation infrastructure. Together they provide an integrated system for control-flow-sensitive reasoning, ownership and lifetime verification, and compiler-level program transformation.

They are internal compiler crates, not independently stabilized libraries. Their APIs, phase assumptions, and intermediate representations can change between Rust releases.

Rust 1.99.1, released October 15, 2026, is the latest stable release as of this review. The reference targets that release, distinguishing unstable development functionality where relevant.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://blog.rust-lang.org\&sz=32)

Rust Blog+3



### 1.1 Responsibilities and boundaries

| Crate                 | Principal responsibility                                      | Distinctive capabilities                                                                                                                                  |
| --------------------- | ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `rustc_mir_dataflow`  | General-purpose MIR abstract interpretation                   | Forward/backward analyses, fixed-point solving, lattice domains, move paths, initialization and liveness tracking, per-location results                   |
| `rustc_borrowck`      | Semantic verification of MIR ownership and lifetimes          | NLL region inference, borrow conflict detection, move and initialization checking, place overlap, closure requirements, Polonius integration              |
| `rustc_mir_transform` | MIR phase transitions, lowering, validation, and optimization | Pass orchestration, drop elaboration, coroutine lowering, CFG simplification, constant and value propagation, inlining, SSA analysis, runtime preparation |

The critical distinction is between computing facts, enforcing language rules, and rewriting the program.

- `rustc_mir_dataflow` computes abstract state over a MIR control-flow graph. Its framework is not intrinsically tied to ownership checking.
- `rustc_borrowck` combines dataflow results with Rust-specific lifetime, borrowing, type, and place-access semantics.
- `rustc_mir_transform` changes MIR representations while respecting the validity requirements of each compiler phase. Not all transformations are optimizations; some implement required language semantics.

These crates operate on MIR definitions and infrastructure supplied by `rustc_middle`, with MIR construction primarily handled by `rustc_mir_build`. They are integrated into rustc's query-driven compilation architecture rather than being independently executable engines.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+3



### 1.2 The combined compiler model

HIR → MIR construction

`rustc_mir_build` / `rustc_middle::mir`

Analysis-phase MIR

Dataflow

State, moves, liveness

Borrow checker

Regions, loans, conflicts

Post-analysis cleanup and lowering

Drop elaboration, runtime semantics, coroutine transformation

Optimized MIR → code generation

Constant propagation, inlining, simplification, value optimization

Conceptual dependency flow, not a literal enumeration of rustc query or pass execution order. Individual analyses and transformations also run at intermediate stages.

This division creates three important architectural capabilities:

1. Reusable semantic analysis. A single dataflow framework supports multiple analyses with different domains, transfer functions, and interpretations of MIR.
2. Flow-sensitive language validation. Borrow checking uses MIR points, place projections, moves, and inferred region relationships rather than relying solely on lexical scope.
3. Phase-aware transformation. The compiler can retain analysis-specific MIR constructs until the relevant checks have finished, then remove or lower them before optimization and code generation.

The rest of this reference examines the advanced mechanisms behind these capabilities and the constraints on combining them.

## 2. `rustc_mir_dataflow`: MIR abstract interpretation

Primary role: Compute conservative program properties at MIR control-flow locations using reusable transfer functions, abstract domains, and fixed-point iteration.

The crate's most significant architectural contribution is that analyses express their semantics while the framework handles iterative propagation across the control-flow graph.

Primary references:

[https://rustc-dev-guide.rust-lang.org/mir/dataflow.html](https://rustc-dev-guide.rust-lang.org/mir/dataflow.html?utm_source=chatgpt.com)

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_dataflow/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_dataflow/index.html?utm_source=chatgpt.com)

### 2.1 The analysis engine

The central interface is `Analysis<'tcx>`.

| Component                                                        | Capability and significance                                                                |
| ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `Domain`                                                         | Abstract state implementing `Clone + JoinSemiLattice`; not limited to an individual bitset |
| `Direction`                                                      | `Forward` or `Backward` traversal, supporting reaching-state and future-use analyses       |
| `bottom_value`                                                   | Initial abstract value assigned to blocks before convergence                               |
| `initialize_start_block`                                         | Establishes analysis-specific entry conditions                                             |
| `apply_primary_statement_effect`                                 | Main statement transfer function                                                           |
| `apply_primary_terminator_effect`                                | Terminator transfer and successor-edge handling                                            |
| `apply_early_statement_effect` / `apply_early_terminator_effect` | Separate effects within a MIR location for finer-grained observations                      |
| `apply_call_return_effect`                                       | Applies state changes only on successful call-return paths                                 |
| `get_switch_int_data` / `apply_switch_int_edge_effect`           | Branch-sensitive refinement for individual `SwitchInt` successors                          |
| `iterate_to_fixpoint`                                            | Computes the converged analysis results                                                    |

The framework supports analyses more sophisticated than conventional gen/kill bit-vector problems.

Important version distinction: `GenKillAnalysis` is a former specialized analysis trait and should not be treated as a current interface. The unified `Analysis` trait handles both general and gen/kill analyses; the `GenKill` abstraction for manipulating sets remains available. Upstream documentation reports that precomputing block transfers through the former specialized trait did not deliver the intended performance advantage.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1



### 2.2 Lattice semantics and convergence

The framework uses a join-semilattice to merge abstract states at control-flow joins.

Its central requirements include:

- Conservative merging: `join` combines information from distinct execution paths according to the domain's ordering.
- Monotone propagation: A change to an incoming state must not make the resulting outgoing state decrease in the lattice ordering across solver iterations.
- Finite convergence: Suitable finite-height domains and monotone transfers ensure that repeated propagation reaches a fixed point.
- Domain-specific precision: The lattice determines which distinctions survive joins. Precision is not established merely by using the framework.

`JoinSemiLattice`, `HasTop`, `HasBottom`, `FlatSet<T>`, and `MaybeReachable<T>` provide reusable abstractions for these semantics.

`FlatSet<T>` is useful for constant-like analyses in which a known value is retained only while all relevant paths agree. `MaybeReachable<T>` distinguishes an unreachable execution state from a reachable state whose abstract information is otherwise empty.

That distinction matters: unreachable is not equivalent to reachable-but-unknown. Confusing them can make constant propagation or other abstract interpretations unsound.

The solver also does not imply arbitrary path sensitivity. Flow joins can lose correlations, and the resulting precision depends on the abstract domain and edge-transfer logic.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2



### 2.3 Statement, terminator, and edge-sensitive effects

The `Analysis` interface exposes several mechanisms that substantially improve semantic fidelity.

Early versus primary effects. An individual MIR statement can have logically distinct stages. The early effect supports observing partial statement effects before the primary transfer runs. This is useful where evaluating operands and updating a destination must be distinguished. The distinction is also relevant to two-phase borrow checking.

Successful-return effects. Function-call destinations are initialized only when execution follows the appropriate successful return path. `apply_call_return_effect` separates those changes from the terminator's general effects, avoiding incorrect propagation onto unwind paths.

Conditional-edge refinement. `SwitchInt` analysis can derive information from a tested discriminant and refine the state separately on each outgoing edge. `SwitchIntData` allows preparatory information to be computed once, rather than redundantly for every successor.

Direction-aware computation. Backward analyses can propagate future-use information toward earlier statements, while forward analyses propagate facts about preceding execution. Both share the same solver and representation interfaces.

These features make the framework suitable for flow-sensitive analyses that would otherwise require a bespoke CFG iterator, edge-state representation, and fixed-point worklist.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1



### 2.4 Built-in analysis families

The crate includes several reusable analyses under `impls`.

| Analysis                                | What it determines                                                            | Important distinction                                               |
| --------------------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| `MaybeInitializedPlaces`                | Places that could be initialized                                              | An existential, may-initialization property                         |
| `MaybeUninitializedPlaces`              | Places that could be uninitialized                                            | Supports detecting potentially invalid uses                         |
| `EverInitializedPlaces`                 | Whether initialization has occurred without an intervening storage-dead event | Important for reassignment restrictions on immutable bindings       |
| `MaybeLiveLocals`                       | Locals whose values may be used subsequently                                  | Backward value liveness, not borrow-region liveness                 |
| `MaybeTransitiveLiveLocals`             | Liveness accounting for dead-assignment chains                                | Avoids treating certain dead stores as meaningful downstream uses   |
| `MaybeStorageLive` / `MaybeStorageDead` | Possible storage lifetime states                                              | Distinct from value initialization and semantic use                 |
| `MaybeRequiresStorage`                  | Whether a local must retain storage at a location                             | Relevant to storage allocation and lowering                         |
| `MaybeBorrowedLocals`                   | Locals potentially referenced or pointed to                                   | Explicitly unsuitable as a replacement for borrowck's loan analysis |
| `borrowed_locals`                       | Locals whose addresses are borrowed somewhere in the body                     | A useful whole-body property rather than a complete loan model      |

The important separation is among initialized state, value liveness, storage liveness, and active loans.

These are not interchangeable:

- A value can be initialized but dead.
- Storage can exist for a local whose value is uninitialized.
- A value may remain live without being borrowed.
- A reference can exist without the corresponding local being live according to a simple value-use analysis.

Notably, the documented `MaybeBorrowedLocals` implementation ignores fake borrows and explicitly warns that it should not be used by borrowck. That makes it appropriate for narrower storage-oriented reasoning, not for reproducing Rust's complete ownership checks.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2



### 2.5 Move-path infrastructure

The `move_paths` module provides a higher-precision representation of places that can be independently initialized or moved.

Key types include `MoveData`, `MovePath`, `MovePathIndex`, `MovePathLookup`, `MoveOut`, `Init`, `InitIndex`, and `LookupResult`.

Mechanisms:

- Canonical indexed paths: A `MovePathIndex` identifies a relevant MIR place or movable subplace. This allows compact state sets and efficient references.
- Partial initialization: Individual fields can be tracked independently, preserving the distinction between a partially moved aggregate and its remaining initialized fields.
- Move/init events: `MoveOut` and `Init` capture the changes that analysis transfer functions must propagate.
- Hierarchical state: Ancestor and descendant paths support reasoning about whole aggregates and subfields.
- Reverse lookup: `MovePathLookup::find` distinguishes an exact tracked path from a lookup that can only resolve to a tracked ancestor.
- Selective path construction: `MoveData::gather_moves` uses MIR visitation and semantic restrictions rather than creating unrestricted paths for every possible projection.

For example, Rust permits moving an eligible field from a tuple while continuing to use another field. That requires greater precision than a single initialized/uninitialized bit for the entire tuple.

Conversely, the implementation does not construct ordinary independently movable paths for arbitrary array-index projections or moves through borrowed references. These restrictions limit the analysis space and reflect language semantics.

`MovePath` is therefore a compiler-selected semantic partition of places, not a universal representation of every addressable runtime memory location.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2



Primary references:

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_dataflow/move_paths/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_dataflow/move_paths/index.html?utm_source=chatgpt.com)

[https://rustc-dev-guide.rust-lang.org/borrow-check/moves-and-initialization/move-paths.html](https://rustc-dev-guide.rust-lang.org/borrow-check/moves-and-initialization/move-paths.html?utm_source=chatgpt.com)

### 2.6 Value analysis: abstract values associated with places

`value_analysis` is an important, more specialized layer for tracking abstract properties of values stored in MIR places.

Its machinery includes `Map`, `State`, `PlaceIndex`, `ValueIndex`, `TrackElem`, and `ValueOrPlace`.

The design provides:

- Place-to-value mapping: A hierarchy of tracked locals and projections associates selected places with abstract values.
- Recursive invalidation: When a place is overwritten or its value becomes unknown, information about its tracked subplaces can be invalidated.
- Aggregate-aware copying: Information for tracked descendants can propagate through assignments when their place relationship is known.
- Abstract value domains: An analysis chooses what the stored value represents; constant propagation is one concrete application.
- Reachability separation: The analysis state distinguishes unreachable execution from a reachable mapping of places to abstract values.
- Tracking controls: Place collection and value limits provide ways to constrain the amount of retained analysis state.

The current implementation also distinguishes full place collection from on-demand tracking through `PlaceCollectionMode`.

The key opportunity is to reuse an established representation for place-sensitive abstract interpretation, rather than independently implementing projection trees, invalidation, assignment propagation, and state joins.

This is not a complete model of arbitrary heap aliasing or pointer identity. A tracked MIR place and an underlying runtime allocation are different concepts; analyses must remain conservative where place relationships cannot be proved.

Upstream source:

[https://doc.rust-lang.org/stable/nightly-rustc/src/rustc_mir_dataflow/value_analysis.rs.html](https://doc.rust-lang.org/stable/nightly-rustc/src/rustc_mir_dataflow/value_analysis.rs.html?utm_source=chatgpt.com)

An important existing consumer is MIR `DataflowConstProp`, which combines this layer with constant-value lattices and the main dataflow engine.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

source+2



### 2.7 Results, inspection, and debugging

Dataflow computation produces `Results<A>` containing the analysis and its converged block-boundary states.

Two principal observation modes are available:

- `ResultsCursor`: Random-access queries at selected MIR locations. It can reconstruct local state by replaying effects from an appropriate boundary state. Methods distinguish block entry/end and states before or after a primary effect.
- `ResultsVisitor` / `visit_results`: Systematic traversal of many locations, avoiding unnecessary repeated reconstruction. `visit_reachable_results` is useful when only reachable blocks should participate.

This separation matters for performance. A checker interested in a small set of drops or calls has a different access pattern from a diagnostic or optimizer that consumes every statement.

`DenseLocationMap` and `PointIndex` provide compact indexing over MIR locations, while the Graphviz infrastructure exposes analysis states and state changes for inspection.

The compiler development guide documents `-Z dump-mir` combined with `-Z dump-mir-dataflow` for generating dataflow-oriented `.dot` output. These are internal compiler debugging facilities, not stable command-line interfaces.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+3



### 2.8 Practical limits of the abstraction

Four boundaries are especially consequential:

1. It is intraprocedural at the framework level. The analysis operates over a MIR body. Interprocedural reasoning requires summaries or additional machinery.
2. It is conservative rather than a concrete interpreter. Path joins, abstraction, and uncertain pointer relationships can discard information.
3. Its results depend on the MIR phase. The meaning and availability of constructs change as the transformation pipeline progresses.
4. It does not implement borrow checking by itself. Region constraints, place-conflict rules, loan activation, and Rust-specific ownership diagnostics belong to `rustc_borrowck`.

The strongest general use case is a MIR analysis whose state can be represented as a monotone abstract domain with explicitly defined statement, terminator, and edge effects.

## 3. `rustc_borrowck`: ownership, loans, and region inference

Primary role: Verify that operations represented in MIR obey Rust's ownership, initialization, borrowing, and lifetime requirements.

The crate combines several distinct forms of analysis. Its central query is `mir_borrowck`, while its internal implementation spans move analysis, borrow-set construction, MIR type checking, region solving, access validation, and diagnostics.

Primary references:

[https://rustc-dev-guide.rust-lang.org/borrow_check.html](https://rustc-dev-guide.rust-lang.org/borrow_check.html?utm_source=chatgpt.com)

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/index.html?utm_source=chatgpt.com)

### 3.1 Borrow-checking pipeline

The borrow checker performs approximately the following logical operations:

1. Prepare analysis MIR. Obtain a local MIR body and replace its regions with appropriate inference variables, while identifying universal regions from the surrounding type context.
2. Collect move and loan information. Construct move paths, initialization events, and the set of borrows represented in the body.
3. Type-check MIR. Generate region constraints arising from subtyping, liveness, reborrows, and other typing requirements.
4. Solve region constraints. Determine the inferred region values and check obligations against universally quantified regions and type bounds.
5. Compute location-specific loan state. Establish which borrows can affect operations at each MIR location.
6. Validate accesses. Check moves, reads, writes, borrows, initialization requirements, and place conflicts.
7. Produce diagnostics and propagated requirements. Report invalid operations and communicate certain closure requirements to their enclosing contexts.

This is a dependency outline rather than a strict single-pass implementation sequence; some information is collected together or revisited.

Crucially, NLL operates over the places where a reference is required to remain valid, rather than simply equating a loan with the entire lexical scope of its originating variable.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+2



### 3.2 Integrated borrow-checking dataflow

A significant implementation detail in Rust 1.99 is that the borrow checker exposes an integrated dataflow analysis through its internal `Borrowck` and `BorrowckDomain` types.

`Borrowck` composes three analyses:

- `Borrows`: Which loans may remain active or relevant.
- `MaybeUninitializedPlaces`: Which places may not contain initialized values.
- `EverInitializedPlaces`: Which initialization events may already have occurred.

`BorrowckDomain` combines the corresponding state into a single abstract domain containing `borrows`, `uninits`, and `ever_inits`.

This enables a unified traversal to provide the related state required for access checking. It is a concrete demonstration of the value of composing analyses through `rustc_mir_dataflow` rather than maintaining independent traversal infrastructure for every concern.

However, this dataflow state is only part of borrow checking. Loan liveness also depends on the results of region inference, and move legality depends on MIR place and type semantics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2



### 3.3 Loan representation and borrow-set construction

The `borrow_set` machinery records loans introduced by MIR borrow operations.

Important concepts include:

| Concept                  | Purpose                                                                                                         |
| ------------------------ | --------------------------------------------------------------------------------------------------------------- |
| `BorrowSet`              | Indexed collection of borrows associated with a MIR body                                                        |
| `BorrowData`             | Metadata describing an individual loan, including its borrowed place and relevant region/activation information |
| `BorrowIndex`            | Compact identity for a loan in dataflow sets                                                                    |
| `GatherBorrows`          | MIR traversal identifying borrow-producing operations                                                           |
| `Borrows`                | Dataflow transfer logic for loan state                                                                          |
| Out-of-scope calculation | Determines locations where loans cease to be relevant to access checking                                        |

Borrow identity should not be confused with reference-variable identity. Multiple references and reborrows can relate to the same underlying place, and their constraints must be evaluated in relation to the loan and its lifetime.

The separate `MoveData` and `BorrowSet` structures capture a fundamental semantic distinction:

- A move path represents a place whose initialization state matters.
- A loan represents a borrowing relationship that constrains what operations can occur.

Both refer to MIR places, but they index different kinds of facts.

Borrow-checker source:

[https://doc.rust-lang.org/stable/nightly-rustc/src/rustc_borrowck/borrow_set.rs.html](https://doc.rust-lang.org/stable/nightly-rustc/src/rustc_borrowck/borrow_set.rs.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

source+2

### 3.4 NLL region inference

Non-lexical lifetime inference is more sophisticated than assigning a single start and end line to each reference.

A region value can be understood as a set containing:

- MIR control-flow locations where the region must be valid.
- Abstract `end('a)` elements representing relationships to universal lifetimes.
- Placeholder elements used when reasoning about higher-ranked lifetime relationships.

Region inference uses `RegionVid`, `UniversalRegions`, `UniversalRegionRelations`, `OutlivesConstraintSet`, and `RegionInferenceContext`, among other types.

Three constraint families are particularly important:

| Constraint | Meaning                                                 | Origin                                             |
| ---------- | ------------------------------------------------------- | -------------------------------------------------- |
| Liveness   | Region R must contain point P                           | Value/type liveness in MIR                         |
| Outlives   | Region R1 must contain the value of R2                  | Subtyping and related type obligations             |
| Member     | An inferred region must equal a permitted choice region | Opaque `impl Trait` lifetime capture relationships |

The region solver works with these relationships rather than treating lifetimes as source-code intervals.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+2



#### Constraint graphs and SCC compression

`RegionInferenceContext` maintains an outlives-constraint graph and computes its strongly connected components.

If two regions mutually outlive each other through a cycle, they are constrained to have equivalent inferred values. Collapsing these regions into an SCC allows the solver to share state.

The resulting SCC graph is a DAG. Constraint propagation can therefore exploit graph ordering instead of naively repeating every constraint across every region.

Important structures include `ConstraintGraph`, `Sccs<RegionVid, ConstraintSccIndex>`, `RegionValues`, and `LivenessValues`.

This is a consequential performance mechanism: it reduces both redundant state storage and repeated propagation through cycles of equivalent constraints.

Distinction from `rustc_mir_dataflow`: Region inference is also a form of fixed-point computation, but it is a specialized graph-constraint solver, not simply an invocation of the ordinary MIR block dataflow engine.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1



#### Universal regions and higher-ranked lifetimes

Universal regions express obligations that must hold regardless of the lifetimes supplied by a caller.

`UniversalRegionRelations` records known outlives relationships from signatures, bounds, and implied constraints.

Higher-ranked types introduce another challenge: the compiler needs to reason about lifetimes whose concrete identity is deliberately unknown.

The placeholder/universe machinery handles this by distinguishing which region names an inference variable is permitted to reference. It prevents an existential variable created in an outer scope from improperly capturing a placeholder introduced under a higher-ranked binder.

Relevant implementation areas include:

- `universal_regions`
- `handle_placeholders`
- `renumber`
- `region_infer`
- `type_check::free_region_relations`

This is fundamental to correct checking of higher-ranked trait bounds and polymorphic function signatures, not merely a diagnostic refinement.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+2



#### Member constraints and opaque types

The `impl Trait` machinery introduces constraints that cannot always be reduced to ordinary pairwise outlives relations.

For example, an inferred region inside an opaque hidden type may be required to correspond to one of the lifetimes that the opaque type is permitted to capture.

A member constraint represents a finite set of possible region choices. The solver uses lower bounds, upper bounds, and a minimal-choice rule to find a valid candidate.

This provides a key interaction between MIR lifetime inference and Rust's opaque-type capture semantics. An independently written region solver that handles only liveness and outlives relationships would miss this functionality.

Primary reference:

[https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference/member-constraints.html](https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference/member-constraints.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide

### 3.5 Place conflicts and projection-sensitive borrowing

Borrow checking must determine whether an access potentially conflicts with an existing loan.

The `places_conflict` machinery compares MIR place projections progressively from their roots.

It can reason about:

- Distinct fields of an aggregate.
- Dereference projections and their relationship to the borrowed place.
- Whole-place accesses versus subplace borrows.
- Constant and runtime-indexed projections.
- Access depth and whether a read/write is shallow or deep.

An important implementation distinction is `PlaceConflictBias`, which distinguishes situations where uncertainty must be resolved conservatively toward overlap or toward non-overlap for the specific predicate being computed.

For example, two runtime indices `a[i]` and `a[j]` cannot generally be assumed distinct. Conversely, two provably different fields of a struct can often be treated as disjoint.

The conflict logic therefore provides structural place reasoning, but it is not a general-purpose symbolic alias solver. It does not prove arbitrary relationships among runtime index expressions.

The `prefixes` module adds rules for determining which ancestor places are relevant to a given access, including distinctions involving dereferences and shallow accesses.

Primary reference:

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/places_conflict/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/places_conflict/index.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1

### 3.6 Two-phase borrows

Two-phase borrowing is a specialized capability that allows certain implicit mutable borrows to be reserved before they become fully active.

A two-phase borrow has:

- Reservation: The initial borrow, which behaves in a shared-like manner for permitted intervening operations.
- Activation: The point at which the borrow becomes fully mutable.

This mechanism supports expressions such as `vec.push(vec.len())`, where the receiver's mutable borrow is prepared before evaluating the remaining argument.

Implementation details:

- Two-phase eligibility is established during type checking and MIR construction.
- Borrow data records reservation and activation information.
- `GatherBorrows` identifies activation locations.
- Dominator information contributes to deciding whether a given MIR point precedes activation.
- Access checking distinguishes conflicts during reservation, activation, and the fully active phase.

Two-phase borrowing is deliberately restricted. It does not mean that arbitrary explicit `&mut` expressions can be divided into reservation and activation phases.

This mechanism illustrates why borrow checking needs more than a single boolean recording whether a mutable borrow exists.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide



### 3.7 Closure requirements and cross-body reasoning

Borrow checking is primarily body-local, but certain lifetime obligations must cross body boundaries.

`ClosureRegionRequirements`, `ClosureOutlivesRequirement`, and related structures allow borrow checking a closure to produce obligations that cannot be proved solely from the closure body.

These obligations are then verified in the surrounding context that constructs the closure.

Two major cases are:

- Region-outlives requirements: Relationships involving captured or external lifetimes.
- Type-outlives requirements: Bounds involving generic types and regions that may need to be proved in the parent context.

This is an important form of compositional interprocedural reasoning. It does not require globally re-solving arbitrary bodies, but it does allow the checker to transfer specific lifetime obligations across the closure boundary.

Primary reference:

[https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference/closure-constraints.html](https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference/closure-constraints.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+1

### 3.8 Polonius integration: stable NLL versus experimental successor

Polonius is a particularly important version-sensitive area.

Status as of October 29, 2026: Rust 1.99.1 stable still uses the established NLL borrow checker by default. Polonius Alpha was enabled by default on the nightly channel in August 2026, with stabilization still in progress. It should not be described as a stabilized Rust 1.99 language feature.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://blog.rust-lang.org\&sz=32)

Rust Blog+2



There are also two meaningfully different Polonius implementations:

|                      | Legacy Polonius                                                         | Polonius Alpha / next                                 |
| -------------------- | ----------------------------------------------------------------------- | ----------------------------------------------------- |
| Implementation       | Original Datalog-style analysis using `polonius-engine`                 | Integrated graph-based implementation in rustc        |
| Main representation  | Relational input facts and derived results                              | Combined region/CFG constraints and loan reachability |
| Performance position | Original comprehensive formulation had substantial scaling difficulties | More pragmatic, performance-conscious subset          |
| Current status       | Experimental legacy implementation                                      | Nightly preview, not yet stable in Rust 1.99          |
| Principal role       | Research, diagnostics, compatibility, and experimentation               | Intended successor to established NLL checking        |

#### Polonius Alpha's capability

The in-tree analysis models loan propagation through a graph incorporating region relationships and control-flow relationships.

It supports a more location-sensitive account of when a loan remains relevant, which can accept certain conditional-borrow patterns that standard NLL rejects.

Noteworthy mechanisms include:

- Edges representing type-derived relationships between regions.
- Edges representing propagation through the control-flow graph.
- Loan reachability from an introduction point to a later program point.
- Constraint relationships that may propagate in either direction where needed.
- A lazy graph-based approach intended to avoid the cost of the original general Datalog formulation.

The 2026 project goal identifies lending-iterator patterns and NLL problem case #3 as central improvements.

However, Polonius Alpha is not the full original Polonius model. Full flow-sensitive behavior for some more exotic conditional reborrows remains future work.

The Rust team reported that Polonius Alpha's preview encountered some performance regressions, including a worst observed case of approximately 2–3× borrow-checking time. That is an upstream observation on particular workloads, not a general compilation-speed estimate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+3



Primary references:

[https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/](https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/?utm_source=chatgpt.com)

[https://goals.rust-lang.org/2026/polonius.html](https://goals.rust-lang.org/2026/polonius.html?utm_source=chatgpt.com)

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/polonius/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/polonius/index.html?utm_source=chatgpt.com)

### 3.9 Compiler-consumer API: extracting borrow-checking facts

One of this crate's most valuable and easily overlooked capabilities is its explicitly exposed `consumers` module.

Its central function is `get_bodies_with_borrowck_facts`.

Rather than returning only whether a body passes borrow checking, it can expose the analyzed MIR and its associated facts.

`BodyWithBorrowckFacts` contains:

| Field                      | Information                                                     |
| -------------------------- | --------------------------------------------------------------- |
| `body`                     | MIR body with the region identifiers used by the analysis       |
| `promoted`                 | Promoted MIR bodies                                             |
| `borrow_set`               | Identified loans                                                |
| `region_inference_context` | Inferred lifetime and region-constraint information             |
| `location_table`           | Mapping between MIR locations and finer-grained Polonius points |
| `input_facts`              | Optional relational facts usable by legacy Polonius             |
| `output_facts`             | Optional computed legacy Polonius results                       |

`ConsumerOptions` distinguishes three levels of extraction:

- `RegionInferenceContext`: MIR, borrows, and region inference without generating full Polonius facts.
- `PoloniusInputFacts`: Adds input facts and location mapping, without running the legacy Polonius engine.
- `PoloniusOutputFacts`: Runs that engine and makes its derived output available.

Important distinction: Requesting Polonius input facts does not mean running Polonius, and the legacy input/output facts are not interchangeable with the newer in-tree Polonius Alpha representation.

The API also exposes helpers such as `calculate_borrows_out_of_scope_at_location` and `places_conflict`, allowing consumers to reuse existing semantic logic rather than approximating it from MIR syntax alone.

This capability is relevant to compiler-integrated ownership analysis, semantic inspection, research tooling, and structured extraction of lifetime-related facts.

There are substantial integration restrictions:

- It copies bodies and regenerates region identifiers.
- It must operate on a suitable type-checking root and accounts for nested bodies.
- It can panic if required MIR has already been stolen by a later compilation query.
- Its own documentation warns against invoking this expensive extraction during ordinary compilation.
- The underlying API remains compiler-internal and unstable.

The API is consequently an integration point, not a cheap general query over already-optimized MIR.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+3



Primary reference:

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/consumers/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/consumers/index.html?utm_source=chatgpt.com)

### 3.10 Diagnostics and explanation of borrow-checking failures

The checker contains substantial infrastructure for explaining semantic failures rather than merely detecting them.

This includes:

- Tracking the origin and location of moves and invalid uses.
- Distinguishing read, write, move, mutation, and borrow-conflict categories.
- Identifying the relevant region relationships that force a loan to remain valid.
- Tracing constraint paths to explain why one lifetime must outlive another.
- Connecting errors to source spans, argument names, captured variables, and closure contexts.
- Detecting unnecessary mutable declarations and related lint opportunities.

`RegionInferenceContext` contains methods for finding constraint paths, assigning region names, identifying relevant source locations, and selecting constraints for diagnostics.

These facilities are important because the abstract state alone generally does not explain why a particular access was rejected.

The diagnostic layer converts analysis provenance into information suitable for compiler error reporting. It is not, however, a standalone proof-certificate API with a guaranteed persistent output schema.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2



## 4. `rustc_mir_transform`: phase-aware MIR lowering and optimization

Primary role: Transform MIR from a representation suitable for semantic analysis into forms suitable for compile-time evaluation, optimization, and code generation.

`rustc_mir_transform` is broader than an optimizer. It implements required semantic lowering, compiler checks, MIR cleanup, optimization passes, and pass orchestration.

Primary references:

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/index.html?utm_source=chatgpt.com)

[https://rustc-dev-guide.rust-lang.org/mir/passes.html](https://rustc-dev-guide.rust-lang.org/mir/passes.html?utm_source=chatgpt.com)

[https://rustc-dev-guide.rust-lang.org/mir/optimizations.html](https://rustc-dev-guide.rust-lang.org/mir/optimizations.html?utm_source=chatgpt.com)

### 4.1 MIR dialects and phase invariants

`rustc_middle::mir::MirPhase` identifies three major dialects:

| Dialect    | Primary purpose                                          | Key properties                                                                   |
| ---------- | -------------------------------------------------------- | -------------------------------------------------------------------------------- |
| `Built`    | Initial MIR construction and early checking              | Retains constructs and semantics needed by early MIR analyses                    |
| `Analysis` | Borrow checking, region inference, and semantic analysis | Constant promotion is made explicit; conditional-drop semantics remain available |
| `Runtime`  | CTFE, optimization, and code generation                  | Semantic lowering completed; permitted MIR constructs become more restricted     |

`Analysis` and `Runtime` have their own subphases, represented by `AnalysisPhase` and `RuntimePhase`.

These are different semantic dialects represented using the same `Body` data structure. A `Drop` terminator, for example, does not have identical semantics in analysis MIR and runtime MIR.

This has a major consequence for transformations: syntactically valid MIR is not necessarily valid MIR for a particular phase.

The compiler's MIR validation machinery checks these constraints, including restrictions on allowed statement forms, terminators, types, and control-flow relationships.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1



### 4.2 Query-driven transformation pipeline

The main pipeline is implemented through compiler queries rather than a single universally callable transformation function.

| Query or entry point                     | Role                                                                                                          |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `mir_built`                              | Initial MIR produced by MIR construction                                                                      |
| `mir_const_qualif`                       | Const qualification associated with early MIR                                                                 |
| `mir_promoted`                           | Prepares analysis MIR and separates promoted constants                                                        |
| `mir_drops_elaborated_and_const_checked` | Ensures borrow checking has occurred; performs analysis-to-runtime preparation                                |
| `optimized_mir`                          | Produces MIR prepared and optimized for code generation                                                       |
| `mir_for_ctfe`                           | Produces runtime MIR suitable for compile-time evaluation without the ordinary optional optimization sequence |
| `promoted_mir`                           | Supplies promoted bodies for subsequent constant evaluation                                                   |
| `build_drop_shim`                        | Creates drop glue for a type, exposed for compiler drivers                                                    |

The transformation crate further separates internal pass sequences through functions such as `run_analysis_cleanup_passes`, `run_analysis_to_runtime_passes`, `run_runtime_lowering_passes`, `run_runtime_cleanup_passes`, and `run_optimization_passes`.

Important distinction: The compiler development guide still uses some historical query names, such as `mir_const`. The current crate index exposes `mir_const_qualif`; the relevant compiler release's query definitions should take precedence over older narrative documentation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1



### 4.3 Drop elaboration: compiling initialization-dependent destruction

Drop elaboration is among the most significant mechanisms in the entire MIR stack because it translates a semantic obligation into explicit runtime control flow.

During analysis, a `Drop` terminator can mean drop this place if it is initialized. After lowering, runtime MIR must explicitly determine whether the destructor is actually called.

The transformation uses `MaybeInitializedPlaces`, `MaybeUninitializedPlaces`, move-path information, and drop-flag state from `rustc_mir_dataflow`.

The compiler distinguishes four classes of drop:

| Drop category | Meaning                                           | Transformation                                              |
| ------------- | ------------------------------------------------- | ----------------------------------------------------------- |
| `Static`      | Definitely initialized                            | Retain the drop without a runtime initialization check      |
| `Dead`        | Definitely uninitialized                          | Remove the unnecessary drop                                 |
| `Conditional` | Either wholly initialized or wholly uninitialized | Introduce a conditional drop, potentially guarded by a flag |
| `Open`        | Potentially partially initialized                 | Expand into the appropriate field-level drop obligations    |

For `Open` drops, the transformation accounts for partially moved aggregates and active enum variants. It must execute destructors for initialized fields without attempting to destroy fields that have been moved.

Drop flags are updated as initialization and move events occur. They can be omitted where static analysis has already proved the necessary state.

This is a particularly strong example of native capability composition:

`MoveData` and initialization analyses → classification of drop obligations → selective flag construction → explicit runtime MIR → drop glue.

Why it matters: The compiler does not need to retain a general-purpose runtime ownership tracker. It specializes the ownership-dependent destruction logic to each MIR body, inserting only the necessary dynamic decisions.

The transformation also participates in correct ordering around assignments, unwinding, and compile-time const checking.

Primary reference:

[https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html](https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide

### 4.4 Coroutine and async state-machine lowering

The `coroutine` module performs a much more structural transformation than ordinary local optimization.

It converts coroutine MIR containing suspension points into a state-machine representation.

Its responsibilities include:

- Identifying locals that must survive suspension.
- Computing the state-machine layout, including captured upvars, state discriminator, and preserved locals.
- Rewriting references to persistent locals so that they access state-machine fields.
- Converting `yield` and return operations into state transitions.
- Constructing resume/poll control flow that dispatches according to the current state.
- Constructing corresponding drop logic for suspended, completed, and never-started states.
- Handling poisoning and unwinding paths.
- Supporting by-move bodies for coroutine closures and `AsyncFnOnce`-related behavior.
- Integrating drop handling, including the async-drop machinery present in the current compiler implementation.

The implementation uses distinct internal modules such as `coroutine::layout`, `coroutine::drop`, and `coroutine::by_move_body`.

The compiler documentation identifies the unresumed, completed, and poisoned states as special cases; additional states correspond to suspension points.

Consequential distinction: A local that is ordinary stack-like MIR state before lowering may become a field of a movable or pinned state-machine object afterward. This affects aliasing, storage, drop behavior, and the validity of analyses performed against that local.

The same general mechanism underlies Rust async function lowering, but the state-machine transformer does not independently implement the language's entire async type system or executor.

Primary references:

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/coroutine/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/coroutine/index.html?utm_source=chatgpt.com)

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/coroutine/layout/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/coroutine/layout/index.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1

### 4.5 Constant promotion, const evaluation, and related lowering

Constant promotion extracts eligible constant-like expressions into separately represented promoted MIR bodies.

`promote_consts` identifies eligible candidates; `mir_promoted` and `promoted_mir` expose the resulting bodies through compiler queries.

This is distinct from both constant propagation and CTFE:

| Mechanism            | Function                                                                         |
| -------------------- | -------------------------------------------------------------------------------- |
| Constant promotion   | Changes where and how certain constant expressions are represented and evaluated |
| Const qualification  | Determines whether constructs satisfy compile-time const restrictions            |
| CTFE                 | Evaluates eligible MIR expressions/functions using the compiler's interpreter    |
| Constant propagation | Replaces or simplifies operations when analysis proves values are known          |

`mir_for_ctfe` provides the MIR representation consumed by compile-time evaluation. It still requires the appropriate semantic lowering, but does not run the ordinary optional optimization pipeline.

Other relevant transformation modules include `ctfe_limit`, which inserts evaluation counters for selected loops and calls, and `required_consts`, which gathers constants that evaluation must account for.

The crucial integration boundary is `rustc_const_eval`: `rustc_mir_transform` prepares and modifies MIR, whereas the interpreter and much of const checking belong to the separate compiler crate.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2



### 4.6 Optimization families

The optimization pipeline contains complementary passes operating at different levels of abstraction.

| Capability family            | Important implementations                                                                              | Core function                                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| CFG simplification           | `SimplifyCfg`, `simplify_branches`, `match_branches`, `unreachable_prop`, `unreachable_enum_branching` | Remove unnecessary branches, blocks, and proven-impossible paths                                 |
| Constant reasoning           | `dataflow_const_prop`, `single_use_consts`, `instsimplify`, `trivial_const`                            | Propagate known scalar values and simplify operations                                            |
| Value redundancy             | `gvn`, `copy_prop`, `ref_prop`, `dest_prop`                                                            | Reuse equivalent computations and eliminate unnecessary copies or intermediates                  |
| Data layout and aggregates   | `sroa`, `large_enums`, `remove_zsts`                                                                   | Expose scalar operations, reduce aggregate overhead, and remove irrelevant zero-sized operations |
| Lifetime/storage cleanup     | `dead_store_elimination`, `remove_storage_markers`, `simplify::SimplifyLocals`                         | Remove unnecessary writes, declarations, and storage annotations                                 |
| Range reasoning              | `ssa`, `ssa_range_prop`                                                                                | Identify suitable SSA locals and propagate proven value ranges                                   |
| Interprocedural optimization | `inline`, `cross_crate_inline`, `check_inline`                                                         | Integrate eligible callees into callers subject to constraints                                   |
| Edge rewriting               | `jump_threading`, `add_call_guards`                                                                    | Restructure CFG edges to expose optimization opportunities                                       |
| Runtime simplification       | `lower_intrinsics`, `lower_slice_len`, `elaborate_box_derefs`, `remove_unneeded_drops`                 | Replace higher-level constructs with simpler runtime operations                                  |

The presence of a pass does not mean it executes in every build. Some are mandatory, some are optimization-level gated, and some have additional applicability or cost restrictions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2



#### A. Dataflow constant propagation

`DataflowConstProp` reuses `rustc_mir_dataflow` directly.

Its architecture combines:

- `Analysis` for forward fixed-point propagation.
- `value_analysis::Map` for tracking eligible places.
- `State<FlatSet<Scalar>>` for representing known scalar values.
- Constant-evaluation interpreter machinery for evaluating eligible expressions.
- `ResultsVisitor` and a subsequent MIR mutation pass to replace proven values.

This is an abstract interpretation over selected MIR places, not full symbolic execution.

The upstream implementation limits the analysis according to optimization settings and excludes places whose addresses have been taken unless aliasing can be handled safely.

Why it matters: The combination avoids duplicating CFG propagation, place indexing, abstract-state joins, and rewrite collection. It also demonstrates how optimization can reuse the same fundamental analysis framework as safety checking while maintaining different semantic goals.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

source



#### B. Global value numbering

`gvn` identifies computations that are provably equivalent, reusing an earlier computed value when legal.

Its implementation assigns symbolic value numbers to eligible expressions, tracks available SSA-defined values, and requires the relevant definition to dominate its replacement use.

Two subtleties are important:

- Reference identity and provenance: Distinct borrows must not be merged merely because their syntactic construction is similar.
- Constant determinism: Constants whose evaluation may create distinct allocation identities cannot automatically be treated as equivalent.

These restrictions distinguish MIR value numbering from textual common-subexpression elimination. The optimization must preserve Rust's operational semantics, including pointer provenance.

Primary reference:

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/gvn/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/gvn/index.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust

#### C. SSA and value-range analysis

Rust MIR is not inherently a complete SSA representation.

The `ssa` module identifies locals that satisfy SSA-like conditions:

- They are assigned exactly once through the relevant supported assignment model.
- Their definition dominates all uses.
- Potential indirect writes disqualify the local unless Rust's type and immutability guarantees make them impossible.

The analysis can then support optimizations such as `ssa_range_prop`, which reasons about value ranges at specific locations.

This provides SSA-related optimization opportunities without requiring a wholesale conversion of MIR into a phi-node-based SSA IR.

It is particularly valuable where MIR local identity and Rust-specific place semantics should be retained while still obtaining some benefits of SSA reasoning.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1



#### D. Aggregate scalarization

`ScalarReplacementOfAggregates`, implemented in `sroa`, can replace eligible aggregate places with independent locals.

This enables later passes to reason about constituent values separately, exposing simplification and redundancy-elimination opportunities.

Its implementation includes escape analysis, computation of flattenable places, and rewriting of those places.

The applicability boundary is important: aggregates whose addresses escape or whose identity must remain observable cannot be naively split. Scalar replacement is therefore a selective transformation rather than a guarantee that all structs and tuples will disappear from generated MIR.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1



### 4.7 Inlining and optimization across boundaries

MIR inlining integrates callee bodies into a caller before backend code generation where the compiler can establish that the transformation is permitted and worthwhile.

The implementation includes `Inline`, `ForceInline`, `CallSite`, `Integrator`, and recursive-inlining or cycle-limiting machinery.

Consequential distinctions:

- MIR inlining versus LLVM inlining: MIR inlining happens before backend IR generation and can enable subsequent MIR optimizations.
- Cross-crate availability: Inlining depends on what MIR is available from referenced crates and the relevant compiler/codegen attributes.
- Semantic and cost constraints: Availability does not ensure that the compiler will inline. It must account for unsupported callee forms, recursive expansion, and code-size or compile-time tradeoffs.
- Generic MIR: Some transformations can happen before monomorphization, allowing the compiler to optimize a generic body without separately duplicating all analysis work for each instantiation.

This is one of the ways MIR optimization can complement rather than simply duplicate LLVM optimization.

Primary reference:

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/inline/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/inline/index.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+1

### 4.8 Linting, validation, and diagnostic preservation

The transformation crate also contains correctness and diagnostic passes that must not be confused with optimization.

Examples include:

- `known_panics_lint`: Detects statically known arithmetic failures and bounds errors.
- `check_packed_ref`, `check_alignment`, `check_null`, `check_pointers`: Detect or validate relevant MIR operations.
- `check_const_item_mutation` and `check_call_recursion`: Specialized MIR checks.
- `lint_and_remove_uninhabited`: Handles situations involving uninhabited values.
- `validate`: Checks MIR structural, type, and phase invariants.
- `cleanup_post_borrowck`: Removes false edges, user type annotations, and other analysis-only constructs after their purpose is complete.

An especially important constraint is that optimization can change diagnostics.

The compiler must avoid running certain transformations before error detection when they could erase code whose invalid behavior should be reported. The MIR phase documentation explicitly calls out the need to preserve known-panic diagnostics across the analysis-to-runtime transition.

Some MIR lints also conservatively assume that code is executable, which can produce false positives; such passes are not proofs of dynamic reachability.

This means pass ordering is partly a semantic and diagnostics contract, not merely a performance scheduling problem.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2



### 4.9 Pass manager and transformation extension points

`MirPass` and `MirLint` form the conceptual extension interfaces.

- `MirPass` transforms a mutable MIR `Body`.
- `MirLint` inspects MIR without modifying it.
- The pass-manager adapter `Lint` allows lint implementations to participate in the pass pipeline.
- Pass-policy and optimization-level controls determine whether particular passes can run.
- `run_passes` coordinates execution and optional phase changes.
- Validation and MIR dumping can be associated with pass boundaries.
- `MirPatch` and MIR visitors provide reusable ways to accumulate and apply targeted structural changes.

The crate's `declare_passes` macro also maintains an enumeration of pass names used for compiler debugging and pass selection.

Integration limitation: The pass traits and much of their implementation machinery are crate-private (`pub(crate)`) in current compiler sources. These are primarily in-tree rustc extension points, not a stable downstream plugin interface.

A compiler fork or an appropriately integrated compiler-driver arrangement may be required to introduce new passes at the desired stage. Merely adding `rustc_mir_transform` as a Cargo dependency does not provide a stable mechanism to register additional compiler passes.

The optimization controls include `-Z mir-opt-level`. These `-Z` debugging facilities are unstable and normally require a nightly compiler.

Primary reference:

[https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/pass_manager/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/pass_manager/index.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2

## 5. Cross-crate interactions and architectural capabilities

The value of these crates is particularly apparent in the mechanisms they share. They do not provide three independent services; they establish a common semantic substrate for multiple stages of compilation.

### 5.1 The most consequential combinations

| Combined capability               | How the crates cooperate                                                                            | Broader significance                                                                               |
| --------------------------------- | --------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Precise ownership validation      | Move paths and initialization dataflow feed borrow checking                                         | Avoids treating a partially moved aggregate as one indivisible variable                            |
| Efficient drop elaboration        | Initialization analyses classify drops; MIR transforms insert only necessary flags and branches     | Converts language-level destruction obligations into specialized runtime control flow              |
| Flow-sensitive loan checking      | Borrow-set dataflow combines with NLL region inference                                              | Distinguishes loans that may exist from loans whose lifetimes make them relevant at a location     |
| Async state-machine generation    | Liveness and storage analyses inform coroutine layout and transformation                            | Preserves only the state required across suspension while generating resume and destruction logic  |
| Constant propagation              | General dataflow, place-value maps, abstract lattices, and const evaluation support MIR rewriting   | Reuses an established abstract-interpretation engine                                               |
| Semantics-preserving optimization | Borrow checking finishes before transformations that remove analysis-only information               | Allows more aggressive runtime optimization without redefining source-language acceptance          |
| Compiler diagnostics              | Analysis provenance, MIR source information, and phase-aware lint passes explain invalid operations | Preserves useful diagnostics even where later optimizations could erase the offending instructions |

The drop-elaboration pipeline is the clearest end-to-end example, because the implementation explicitly uses `MaybeInitializedPlaces`, `MaybeUninitializedPlaces`, `MoveData`, and `ResultsCursor` to drive an actual MIR transformation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+2



### 5.2 Three different fixed-point and graph-reasoning mechanisms

Although the crates all perform graph-oriented reasoning, they should not be collapsed into one generic analysis concept.

| Mechanism            | Graph or structure                                    | State propagated                                      |
| -------------------- | ----------------------------------------------------- | ----------------------------------------------------- |
| General MIR dataflow | Control-flow graph of a body                          | Abstract program state between and within blocks      |
| NLL region inference | Region-outlives graph, SCC DAG, MIR location sets     | Inferred lifetime relationships and region membership |
| Polonius Alpha       | Combined loan, region, and control-flow relationships | Location-sensitive loan reachability                  |

A conventional dataflow analysis can prove that a place may be uninitialized. That does not necessarily establish whether a particular loan conflicts with an access: the checker also needs inferred region validity, borrow activation state, and place-conflict semantics.

Likewise, the NLL region solver can establish a lifetime relationship without directly determining the result of an arbitrary value-property analysis.

The systems share MIR and some abstract-interpretation principles, but solve different constraint problems with different precision and computational costs.

### 5.3 Information availability changes across phases

A particularly important design property is that MIR information becomes available and disappears at different compilation stages.

| Information or operation                                    | Most relevant stage              |
| ----------------------------------------------------------- | -------------------------------- |
| Source-oriented regions, user type annotations, false edges | Before post-borrow-check cleanup |
| Move paths, loans, original borrow-checking constraints     | Borrow-checking stage            |
| Promoted constant bodies                                    | After constant promotion         |
| Explicit runtime drop flags and destructor branches         | After drop elaboration           |
| Coroutine state fields and dispatch blocks                  | After coroutine lowering         |
| Rewritten, simplified, and inlined operations               | Optimization stage               |
| Final generic MIR for code generation                       | `optimized_mir`                  |

Consequently, later MIR is not always semantically richer MIR.

An optimized body may contain fewer operations and simpler CFG edges, but it will not necessarily retain the source-level borrowing structure, region constraints, or the origin of particular moves.

Similarly, pre-lowering MIR does not necessarily expose the exact control flow that will execute at runtime.

This creates a natural distinction between analysis-oriented extraction and runtime-oriented inspection, even though both operate on MIR.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust+2



## 6. Integration boundaries and operational considerations

### 6.1 Compiler-private status and toolchain requirements

All three crates are part of the rustc implementation. They are distributed and versioned with the compiler rather than offering independent stable library compatibility guarantees.

For external compiler drivers using `rustc_private`, the rustc development guide documents the `rustc-dev` and `llvm-tools` components, as well as the need for a compatible compiler/toolchain configuration.

The integration surface commonly includes:

- `rustc_driver` / `rustc_interface` for executing or controlling compiler work.
- `rustc_middle` for MIR definitions, type information, queries, and `TyCtxt`.
- `rustc_mir_dataflow` for implemented analyses and the abstract-interpretation framework.
- `rustc_borrowck::consumers` where borrow-checking facts are explicitly exposed.
- Compiler-internal `rustc_mir_transform` code for adding or modifying transformations.

A compiler-driver callback is not automatically equivalent to an arbitrary MIR pass hook. Accessing an existing compiler query, extracting borrow-checking facts, and registering a transformation within the compiler's pass sequence have materially different integration requirements.

Official reference:

[https://rustc-dev-guide.rust-lang.org/rustc-driver/external-rustc-drivers.html](https://rustc-dev-guide.rust-lang.org/rustc-driver/external-rustc-drivers.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide

### 6.2 Query execution and MIR stealing

Some intermediate queries return a `Steal<Body<'tcx>>` wrapper, allowing a later query to take ownership of MIR without repeatedly cloning it.

This improves compilation efficiency but imposes a strict lifecycle constraint:

- Earlier analyses must consume the required MIR before a subsequent phase steals it.
- Attempting to access a stolen MIR body causes a compiler panic.
- Query ordering and dependencies therefore matter independently of the ordinary Rust type-level borrowing rules.
- Producing an independent copy of MIR for analysis is possible in certain interfaces, but incurs work and may require regenerating contextual information.

The borrow-checker consumer interface is designed partly around these constraints. It obtains appropriate MIR and region facts at a point where they can still be reconstructed consistently.

This is a major reason the internal query interfaces should not be interpreted as a freely addressable, persistent database of every MIR version.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+1



### 6.3 Observability and validation

The compiler offers useful native inspection facilities without requiring custom instrumentation.

| Facility                            | Information available                                                       |
| ----------------------------------- | --------------------------------------------------------------------------- |
| `-Z dump-mir`                       | MIR before/after passes, with filtering by body and pass                    |
| `-Z dump-mir-graphviz`              | Control-flow graph representations                                          |
| `-Z dump-mir-dataflow`              | Dataflow state and effects in Graphviz form                                 |
| `-Z mir-opt-level`                  | Control over MIR optimization levels, including experimental configurations |
| `MirPass` validation infrastructure | Structural and semantic checks associated with phase transitions            |
| `tests/mir-opt`                     | Before/after MIR transformation regression testing                          |
| `tests/ui`                          | Source-facing compiler diagnostics and behavior regression testing          |

The MIR debugging guide also documents filtering expressions that select combinations of bodies and pass names. This makes it possible to inspect the exact point where a semantic property or transformation becomes visible.

Higher MIR optimization levels and experimental passes are not automatically improvements in compilation performance or generated code quality. Some analyses impose explicit complexity limits; for example, the documented dataflow constant-propagation implementation constrains its analysis size at ordinary optimization levels.

Primary references:

[https://rustc-dev-guide.rust-lang.org/mir/debugging.html](https://rustc-dev-guide.rust-lang.org/mir/debugging.html?utm_source=chatgpt.com)

[https://rustc-dev-guide.rust-lang.org/mir/optimizations.html](https://rustc-dev-guide.rust-lang.org/mir/optimizations.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide+2

### 6.4 Important technical limitations

| Constraint                                          | Consequence                                                                                     |
| --------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| Unstable compiler-internal APIs                     | Interfaces and internal invariants can change between toolchain versions                        |
| MIR is lower-level than HIR                         | Source syntax, lexical grouping, and some higher-level intent are not directly preserved        |
| General dataflow is intraprocedural                 | Whole-program analyses require additional summaries or propagation mechanisms                   |
| Move paths do not represent all memory locations    | Arbitrary alias and heap-identity reasoning remains a separate problem                          |
| NLL regions are not simply source intervals         | A region must be interpreted using MIR points, universal-region elements, and constraints       |
| Borrow-checking facts are phase-dependent           | Optimized MIR alone cannot reconstruct the full original borrow-checking context                |
| Transformation interfaces are largely crate-private | Adding arbitrary optimization passes typically requires compiler-internal integration           |
| Early transformations affect accepted programs      | Pre-runtime passes must preserve the semantics and diagnostic behavior expected by the language |
| Optional optimizations have cost thresholds         | Greater analysis precision or pass coverage can increase compile time substantially             |
| Nightly Polonius is not stable NLL                  | Experimental loan analysis must not be assumed to describe stable borrow-checking behavior      |

These constraints are consequences of how rustc separates semantic verification from runtime preparation, not incidental weaknesses of the libraries.

## 7. Primary-source reference map

The following references provide the most useful routes for further investigation. URLs are written directly for standalone reuse.

| Subject                        | Official source                                                                                                                                                                                                     |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| MIR architecture               | [https://rustc-dev-guide.rust-lang.org/mir/index.html](https://rustc-dev-guide.rust-lang.org/mir/index.html?utm_source=chatgpt.com)                                                                                 |
| Dataflow framework             | [https://rustc-dev-guide.rust-lang.org/mir/dataflow.html](https://rustc-dev-guide.rust-lang.org/mir/dataflow.html?utm_source=chatgpt.com)                                                                           |
| `rustc_mir_dataflow` API       | [https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_dataflow/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_dataflow/index.html?utm_source=chatgpt.com)                                 |
| Move paths                     | [https://rustc-dev-guide.rust-lang.org/borrow-check/moves-and-initialization/move-paths.html](https://rustc-dev-guide.rust-lang.org/borrow-check/moves-and-initialization/move-paths.html?utm_source=chatgpt.com)   |
| MIR borrow checking            | [https://rustc-dev-guide.rust-lang.org/borrow-check.html](https://rustc-dev-guide.rust-lang.org/borrow-check.html?utm_source=chatgpt.com)                                                                           |
| `rustc_borrowck` API           | [https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/index.html?utm_source=chatgpt.com)                                         |
| NLL region inference           | [https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference.html](https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference.html?utm_source=chatgpt.com)                                         |
| Opaque-type member constraints | [https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference/member-constraints.html](https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference/member-constraints.html?utm_source=chatgpt.com)   |
| Closure region requirements    | [https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference/closure-constraints.html](https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference/closure-constraints.html?utm_source=chatgpt.com) |
| Borrow-checker consumers       | [https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/consumers/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_borrowck/consumers/index.html?utm_source=chatgpt.com)                     |
| MIR queries and passes         | [https://rustc-dev-guide.rust-lang.org/mir/passes.html](https://rustc-dev-guide.rust-lang.org/mir/passes.html?utm_source=chatgpt.com)                                                                               |
| MIR transformations            | [https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/index.html?utm_source=chatgpt.com)                               |
| MIR optimization framework     | [https://rustc-dev-guide.rust-lang.org/mir/optimizations.html](https://rustc-dev-guide.rust-lang.org/mir/optimizations.html?utm_source=chatgpt.com)                                                                 |
| Drop elaboration               | [https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html](https://rustc-dev-guide.rust-lang.org/mir/drop-elaboration.html?utm_source=chatgpt.com)                                                           |
| Coroutine lowering             | [https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/coroutine/index.html](https://doc.rust-lang.org/stable/nightly-rustc/rustc_mir_transform/coroutine/index.html?utm_source=chatgpt.com)           |
| MIR debugging                  | [https://rustc-dev-guide.rust-lang.org/mir/debugging.html](https://rustc-dev-guide.rust-lang.org/mir/debugging.html?utm_source=chatgpt.com)                                                                         |
| External rustc drivers         | [https://rustc-dev-guide.rust-lang.org/rustc-driver/external-rustc-drivers.html](https://rustc-dev-guide.rust-lang.org/rustc-driver/external-rustc-drivers.html?utm_source=chatgpt.com)                             |
| Polonius Alpha announcement    | [https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/](https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/?utm_source=chatgpt.com)                                       |
| Polonius 2026 project goals    | [https://goals.rust-lang.org/2026/polonius.html](https://goals.rust-lang.org/2026/polonius.html?utm_source=chatgpt.com)                                                                                             |
| Rust 1.99.1 release            | [https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/?utm_source=chatgpt.com)                                                                                     |

For strict version reproducibility, inspect the Rust repository at the specific `1.99.1` tag instead of assuming a mutable documentation URL always shows the same release:

[https://github.com/rust-lang/rust/tree/1.99.1/compiler/rustc_mir_dataflow](https://github.com/rust-lang/rust/tree/1.99.1/compiler/rustc_mir_dataflow?utm_source=chatgpt.com)

[https://github.com/rust-lang/rust/tree/1.99.1/compiler/rustc_borrowck](https://github.com/rust-lang/rust/tree/1.99.1/compiler/rustc_borrowck?utm_source=chatgpt.com)

[https://github.com/rust-lang/rust/tree/1.99.1/compiler/rustc_mir_transform](https://github.com/rust-lang/rust/tree/1.99.1/compiler/rustc_mir_transform?utm_source=chatgpt.com)

## 8. Synthesis: the key architectural distinctions

The three crates collectively demonstrate an important separation of concerns in rustc:

- `rustc_mir_dataflow` supplies reusable abstract interpretation. Its strongest capabilities are lattice-based reasoning, move-path precision, phase-aware transfer effects, and efficient access to converged analysis state.
- `rustc_borrowck` adds Rust's ownership semantics. Its distinguishing capabilities are NLL region inference, loan and place-conflict analysis, higher-ranked lifetime handling, closure obligations, and access to structured borrow-checking facts.
- `rustc_mir_transform` converts verified semantics into executable structure. Its most consequential capabilities include drop elaboration, coroutine state-machine generation, staged MIR lowering, generic MIR optimization, and validation of transformation invariants.

The most significant architectural opportunity is not any individual pass or algorithm. It is the ability to compute conservative semantic facts once in a shared MIR model, consume them for multiple compiler purposes, and progressively transform the representation after each class of semantic obligation has been discharged.

That arrangement explains both the sophistication of the available analysis infrastructure and its strict compiler-phase boundaries.

# Addendum — Rust Compiler Internals: Additional MIR Capabilities and Semantic Details

Libraries: `rustc_mir_dataflow`, `rustc_borrowck`, `rustc_mir_transform`\
Target version: Rust 1.99.1 (stable, released October 15, 2026)\
Research date: November 1, 2026\
Companion reference: Rust Compiler Internals: `rustc_mir_dataflow`, `rustc_borrowck`, and `rustc_mir_transform` (October 29, 2026)

## Scope

This addendum extends the original reference with capabilities not previously developed in sufficient detail. It focuses on three areas:

1. Dataflow infrastructure: control-flow-edge selection, initialization-event semantics, solver boundaries, and compiler-native analysis assertions.
2. Borrow-checking coordination: multi-body region solving, opaque-type inference, implied lifetime bounds, and composition of analysis results.
3. MIR transformation beyond optimization: compiler-generated functions, coverage instrumentation, ABI enforcement, type normalization, code-generation metadata, supplementary diagnostics, and pass policies.

The discussion is based on tagged Rust 1.99.1 compiler sources. Features exposed only by newer nightly compiler sources are distinguished from the target release. All APIs discussed here remain compiler-internal unless otherwise stated.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://blog.rust-lang.org\&sz=32)

Rust Blog



## 1. Additional `rustc_mir_dataflow` capabilities

### 1.1 State-dependent control-flow edge selection

The `Analysis<'tcx>` trait exposes a control-flow mechanism that is distinct from its already-documented statement, terminator, and branch-edge transfer effects:

`get_terminator_edges(&self, state, terminator, location) -> TerminatorEdges`

By default, this delegates to `terminator.edges()`. A forward analysis can override it to derive the terminator's successor-edge description from its current abstract state.

The distinction is between selecting the edges on which propagation occurs and refining the abstract state propagated along an edge.

| Mechanism                                                | Responsibility                                          | Consequence                                                                    |
| -------------------------------------------------------- | ------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `get_terminator_edges`                                   | Determine successor-edge structure using abstract state | Can exclude or specialize infeasible control-flow alternatives where supported |
| `apply_primary_terminator_effect`                        | Update the abstract state for the terminator            | Models its general effect on the current state                                 |
| `apply_call_return_effect`                               | Apply effects specific to normal call return            | Keeps successful-return initialization separate from unwinding                 |
| `get_switch_int_data` and `apply_switch_int_edge_effect` | Compute and apply branch-specific facts                 | Refines propagated state for individual `SwitchInt` outcomes                   |

In Rust 1.99.1, `get_terminator_edges` is invoked before `apply_primary_terminator_effect`. The framework documentation notes that this ordering is necessary for `MaybeInitializedPlaces`, while other analyses can accommodate either order.

The method is documented as forward-analysis-only. Its availability does not mean that a consumer can arbitrarily rewrite the CFG or discard successors without preserving analysis soundness.

This provides an important additional degree of freedom for abstract interpretation: where the abstract state proves some control-flow alternatives infeasible, the analysis may reason over a more precise effective graph rather than propagating unchanged information through every syntactic successor.

Source: `Analysis` implementation, Rust 1.99.1:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_dataflow/src/framework/mod.rs`

### 1.2 Fixed-point scheduling and directional initialization limits

The `iterate_to_fixpoint` implementation uses a ranked, dirty-block propagation algorithm rather than a generic unordered worklist.

Forward analysis traverses blocks in reverse postorder (RPO). The solver assigns each reachable block a rank and selects the lowest-ranked dirty block, returning to earlier ranks when a back edge invalidates already-propagated information.

Consequently, loop-free portions of a reachable CFG can be processed in a single pass, while loops are revisited when their incoming abstract states change. Backward analysis reverses the rank ordering, obtaining postorder-like processing without separately maintaining a postorder cache.

The framework also reuses a single temporary domain-state allocation across block processing to avoid unnecessary repeated allocation.

These are concrete scheduling optimizations within the general lattice-based framework; they do not change the requirement that transfer functions be monotone or establish an upper bound on iteration count independently of the chosen domain.

An important boundary concerns `initialize_start_block`. For forward analyses, it permits a non-bottom initial state at `START_BLOCK`. For backward analyses, Rust 1.99.1 does not support changing that initial state away from bottom: the implementation explicitly triggers a compiler bug if the method does so.

An analysis needing nontrivial exit-boundary conditions therefore cannot assume the forward initialization mechanism transfers directly to backward traversal. Its boundary representation must conform to the implementation's actual supported semantics.

Source: `Analysis::iterate_to_fixpoint` and associated comments:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_dataflow/src/framework/mod.rs`

### 1.3 Reusable initialization-event and drop-flag semantics

The `drop_flag_effects` module is a reusable bridge between MIR move/init records and initialization-presence state.

Its central abstraction is:

`DropFlagState::{Present, Absent}`

This represents whether a tracked move path contains a value requiring destruction, rather than whether a runtime flag has necessarily been materialized.

The module exposes `drop_flag_effects_for_function_entry` and `drop_flag_effects_for_location`, along with traversal helpers including `on_all_children_bits`, `on_lookup_result_bits`, and `move_path_children_matching`.

Its event interpretation includes:

- Function entry: arguments and their tracked descendants begin in `Present` state.
- Moves: moving from a tracked path marks that path and its descendants `Absent`.
- Drops: dropping an exactly tracked place marks the relevant path and descendants `Absent`, although the operation is not itself represented as a move.
- Initialization: `InitKind::Deep` marks a path and all descendants `Present`; `InitKind::Shallow` marks only the selected path.
- Successful-return-only initialization: `InitKind::NonPanicPathOnly` is intentionally omitted from the generic location-level initialization callback. Path-specific effects must be handled by the appropriate analysis machinery.

The distinction between exact and parent-only move-path lookup is consequential. A place resolved only to a tracked parent cannot be treated as if its individual descendants were independently tracked.

The module also contains an internal mechanism for enumerating inactive enum-variant move paths along a conditional edge. This supports branch-sensitive initialization reasoning where the active discriminant determines which variant-specific paths are meaningful.

The resulting abstraction separates three concerns:

`MoveData` identifies events and tracked paths; `drop_flag_effects` interprets their presence-state transitions; consumers determine how those transitions affect analysis facts or runtime-generated drop flags.

This makes the event interpretation reusable without requiring every consumer to reproduce the details of moves, drops, deep initialization, and move-path descendants.

Source: `drop_flag_effects.rs`:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_dataflow/src/drop_flag_effects.rs`

### 1.4 Compiler-native assertions over dataflow results

The `rustc_peek` module provides infrastructure for testing the results of individual dataflow analyses against expectations embedded in compiler test programs.

It supports checking, among others:

- `MaybeInitializedPlaces`
- `MaybeUninitializedPlaces`
- `MaybeLiveLocals`

The `sanity_check` entry point reads compiler-internal `rustc_mir` test attributes, constructs the requested analysis, and executes it to a fixed point. It then examines recognized uses of the internal `rustc_peek` intrinsic.

For each applicable intrinsic call, it reconstructs the corresponding MIR place and determines the analysis state at the relevant location using a `ResultsCursor`. A requested bit that is absent produces a compiler diagnostic such as `rustc_peek: bit not set`.

This creates a different form of validation from printing a dataflow graph. Graphviz output makes analysis states inspectable; `rustc_peek` can turn a source-level expectation about an abstract-state property into a repeatable compiletest assertion.

The intrinsic, its attributes, and the expected MIR shapes are compiler-testing internals. They are not stable application-level instrumentation or a general-purpose source-language assertion interface.

Source: `rustc_peek.rs`:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_dataflow/src/rustc_peek.rs`

## 2. Additional `rustc_borrowck` capabilities

### 2.1 Composition of independently solved analyses

The existing guide describes `Borrowck` and `BorrowckDomain` as an integrated analysis. An important implementation distinction is that this integration does not create a single joint fixed-point solve.

Rust 1.99.1 explicitly computes the fixed-point results of three constituent analyses separately:

- `Borrows`
- `MaybeUninitializedPlaces`
- `EverInitializedPlaces`

The `Borrowck` type implements `Analysis`, and its `BorrowckDomain` combines the individual states, but this composite analysis is not itself executed with `iterate_to_fixpoint`.

Indeed, its `initialize_start_block`, `apply_call_return_effect`, and `JoinSemiLattice::join` implementations are marked unreachable for that purpose.

Instead, the previously computed results are composed for the subsequent checking traversal, with the composite transfer functions dispatching to the appropriate constituent transfer logic.

This distinction matters because two kinds of integration are possible:

| Integration model          | Behavior                                                                                                                         |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Joint fixed-point solution | Several properties converge together, potentially requiring a larger combined lattice and iteration process                      |
| Composed result traversal  | Individual properties converge separately; their states are subsequently consumed together at the locations relevant to checking |

Rust 1.99.1 uses the second model here.

A general architectural consequence is that analyses can share a common MIR framework and be consumed in a unified checking operation without needing to solve all their abstract states simultaneously. This can simplify the relationship between independently meaningful analyses while retaining access to their combined location-specific facts.

Source: `rustc_borrowck/src/dataflow.rs`:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_borrowck/src/dataflow.rs`

### 2.2 Type-checking-root coordination and opaque hidden types

Borrow checking is more than a collection of independent function-body operations. `BorrowCheckRootCtxt` coordinates constraints and results for an entire type-checking root, including its nested bodies.

The root context maintains:

- Collected region-constraint results for individual bodies.
- Propagated borrow-check results from nested bodies.
- Resolved hidden types for opaque definitions.
- Deferred errors involving unconstrained regions in opaque hidden types.
- Optional compiler-consumer extraction state.

A particularly subtle dependency arises when opaque types and closure lifetime requirements interact.

A parent body's opaque-type defining use may depend on region requirements inferred from a nested closure. Conversely, checking a nested body's use of that opaque type may require knowing the hidden type established by its parent.

This creates a dependency cycle between discovering the opaque type and validating the region relationships that reference it.

The implementation resolves that cycle through staged constraint processing:

1. Collect region constraints across the relevant nested bodies and root.
2. Compute and propagate closure requirements that can be established before opaque-type resolution.
3. Resolve opaque hidden types and process deferred opaque-type constraints and errors.
4. Apply closure requirements again where resolved opaque types change the relevant obligations.
5. Finalize bodies whose constraints can now be checked.

Bodies that do not depend on opaque-type resolution can be completed earlier, avoiding the cost of deferring every body's analysis.

This is a significant form of cross-body semantic coordination. The borrow checker does not require arbitrary whole-program analysis, but its correctness can depend on reasoning about a type-checking root as a group rather than treating each closure as wholly independent.

The ordinary `mir_borrowck` query also returns a map of definition-site hidden types on success. It should therefore be understood as contributing to opaque-type resolution as well as ownership validation.

Primary sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_borrowck/src/root_cx.rs`

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_borrowck/src/lib.rs`

### 2.3 Implied outlives bounds and typing-environment boundaries

The `mir_borrowck_implied_outlives_bounds` query computes lifetime bounds implied by function signatures and relevant surrounding type information.

The important capability is not merely deriving additional `T: 'a`-like relationships. It is deriving them under a typing environment that respects when opaque hidden types may legitimately be revealed.

For type-checking roots, the query uses `TypingEnv::non_body_analysis`, preventing inappropriate revelation of hidden types defined in that scope.

For nested bodies, it uses `TypingEnv::post_typeck_until_borrowck`, where opaque normalization can be appropriate because their well-formedness and defining context are established through the parent.

The implementation normalizes signature inputs and outputs, computes implied bounds, and retains normalized signature information in the query result. The latter is important for connecting region inference variables introduced during normalization to the actual parameter and return types.

For associated functions and constants, relevant well-formedness information from impl headers is also considered.

The query returns a canonicalized response rather than a raw list of unrestricted lifetime assumptions. This preserves its role within the compiler's inference and obligation-solving infrastructure.

The architectural distinction is that a mathematically plausible lifetime relationship is not automatically a valid compiler assumption unless it was derived under an appropriate typing environment and its associated obligations are satisfied.

This also explains why indiscriminately normalizing opaque types before borrow checking can change the meaning of an analysis.

Source: `implied_bounds.rs`:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_borrowck/src/implied_bounds.rs`

### 2.4 Borrow-checker consumer configuration and Polonius execution

The existing `get_bodies_with_borrowck_facts` interface has an additional operational distinction worth retaining.

`ConsumerOptions::PoloniusInputFacts` is documented as the recommended option for extracting extensive information without the potentially significant additional work of executing the legacy Polonius engine.

However, its documentation explicitly states that execution under `-Zpolonius` can override the selected extraction level: in that configuration the consumer receives the results as if `PoloniusOutputFacts` had been requested.

Consequently, the cost and contents of the returned facts are determined by both the `ConsumerOptions` value and the active compiler configuration.

The input and output fields also belong to the legacy Polonius representation. They must not be interpreted as a stable serialization of the newer integrated Polonius Alpha algorithm.

This distinction is especially important for compiler-integrated inspection: a consumer that requests input facts to avoid the legacy solver's cost cannot assume the requested extraction level independently of compiler flags. The borrowing and MIR-stealing constraints discussed in the original document remain applicable.

Source: `consumers.rs`:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_borrowck/src/consumers.rs`

## 3. Additional analysis-to-runtime semantic transformations

### 3.1 Post-analysis normalization and explicit subtype representation

Rust 1.99.1 includes two required transformations that establish type-representation invariants after borrow checking.

`PostAnalysisNormalize` runs under `TypingEnv::post_analysis` and normalizes types, generic arguments, and constant operands according to the post-analysis typing rules.

Its most important responsibilities include revealing opaque hidden types and resolving associated-type representations that could not safely be normalized under an earlier typing mode.

The pass also contains conditional handling for `ProjectionElem::OpaqueCast`, removing no-longer-necessary opaque casts under the applicable trait-solver configuration.

Normalization is fallible: the implementation uses `try_normalize_erasing_regions` and retains an existing representation if normalization fails. This accommodates cases such as impossible generic predicates without treating every failed normalization as a compiler crash.

`Subtyper`, implemented in `add_subtyping_projections`, makes previously implicit assignment subtyping explicit.

Where the rvalue and destination types differ after appropriate region erasure and anonymization, it inserts an intermediate temporary and rewrites the assignment into a form using `Rvalue::Cast(CastKind::Subtype, ...)`.

Conceptually, an assignment

`destination: DestinationTy = expression: SourceTy`

becomes a source-typed temporary followed by an explicit subtype conversion into the destination.

The pass deliberately excludes certain generated dereference temporaries and generic reborrows, whose treatment is governed by other MIR mechanisms.

Both passes have `PassPolicy::Required`. Their positioning is semantically significant:

`PostAnalysisNormalize` → `Subtyper` → `ElaborateDrops`

The first pass establishes appropriate post-analysis type representations. The second makes subtype relationships explicit. Drop elaboration can then operate on the resulting runtime-oriented types and places.

Why this matters: post-borrow-check MIR is not obtained simply by erasing lifetime information. The compiler may need to reveal type representations, normalize associated types, and introduce explicit operations that were unnecessary or inappropriate during semantic analysis.

These transformations are required for correctness even when conventional optimization is disabled.

Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/post_analysis_normalize.rs`

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/add_subtyping_projections.rs`

### 3.2 Detecting impossible generic predicates

`ImpossibleClauses` handles MIR bodies whose declared generic bounds cannot be satisfied.

A compiler may analyze or optimize a generic function without any concrete call to that function. It therefore cannot always assume that a body is instantiated under a satisfiable set of predicates.

For example, with the appropriate unstable feature enabled, an item could declare an inherently contradictory requirement such as `String: Copy`.

The pass uses `has_impossible_clauses` to identify provably contradictory conditions. It includes specialized reasoning about `Sized` requirements on structurally unsized types, even where some surrounding type structure is generic.

Its general predicate processing deliberately excludes cases containing local generic names or certain alias constants, avoiding unsupported normalization or inference assumptions.

If the body is already tainted by errors or the applicable predicates are proved impossible, the pass replaces its executable body with a single `Unreachable` terminator and removes now-irrelevant declarations and debug information.

That eliminates the need for subsequent optimizations to reason about a body whose preconditions cannot hold.

The pass is `optional_non_optimization(true)`, reflecting that it is not a normal cost-driven optimization. Its justification is that replacing provably unreachable execution does not remove defined behavior.

It should not be understood as a complete satisfiability solver for arbitrary generic predicates. Its proof capability is intentionally narrower than the full trait-solving problem.

Source:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/impossible_clauses.rs`

### 3.3 ABI-aware unwinding and termination semantics

Unwinding is a distinct semantic dimension of MIR transformation.

`AbortUnwindingCalls` is a required pass that enforces the interaction among function ABIs, target exception capabilities, panic strategy, and unwind edges.

It inspects operations including calls, drops, assertions, and inline assembly capable of unwinding, using ABI-aware `fn_can_unwind` information.

Its behavior distinguishes several cases:

| Condition                                              | MIR-level treatment                                                                   |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------- |
| Callee or operation cannot unwind                      | Unnecessary unwind successors can become unreachable                                  |
| Target cannot support the relevant unwinding mechanism | Unwind paths are eliminated or made unreachable as appropriate                        |
| Callee may unwind but the containing function cannot   | Escaping unwind paths are rewritten to terminate execution                            |
| Cleanup must execute before termination                | Cleanup edges remain, with subsequent unwind-resume behavior converted to termination |

For example, an `extern "C-unwind"` function may produce a foreign unwind even when the Rust code is compiled with `panic=abort`. The compiler must prevent that unwind from violating the guarantees assumed for Rust-defined functions that cannot unwind.

On a target configuration without supported exception handling, the representation can be simplified further because some unwind operations are structurally impossible.

The pass also handles `UnwindResume` versus `UnwindTerminate`, removing newly unreachable cleanup blocks after rewriting.

These are semantic transformations, not merely attempts to improve generated-code efficiency.

A related but separate facility is `ffi_unwind_calls`.

Its `has_ffi_unwind_calls` query inspects early, built MIR to identify potentially escaping foreign unwinds from eligible foreign-function calls or inline assembly with `MAY_UNWIND`.

The information supports the `ffi_unwind_calls` lint and another query, `required_panic_strategy`, which can impose a crate-level constraint on the final panic runtime.

In particular, a crate compiled with `panic=unwind` that permits foreign unwinds to escape may require a final `panic=unwind` runtime. This prevents an incompatible `panic=abort` compilation unit from making invalid assumptions about whether Rust code can unwind across such boundaries.

Conversely, a `panic=unwind` crate without such FFI unwind behavior need not impose that same final-runtime requirement solely because of its local panic strategy.

This demonstrates an important interaction across compiler stages:

Early MIR analysis identifies potential FFI unwinding → crate-level metadata constrains linking compatibility → later MIR transformation enforces call-site unwind semantics.

The capabilities are not supplied by a generic control-flow simplifier or by LLVM alone. They encode Rust's ABI and panic-model obligations.

Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/abort_unwinding_calls.rs`

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/ffi_unwind_calls.rs`

### 3.4 Why the lowering order is constrained

Several of these transformations must execute in a specific order, independently of optimization profitability.

The Rust 1.99.1 `run_runtime_lowering_passes` sequence establishes important dependencies:

| Stage                                    | Invariant or purpose                                                 |
| ---------------------------------------- | -------------------------------------------------------------------- |
| `CriticalCallEdges`                      | Prepare relevant call-edge control-flow structure                    |
| `PostAnalysisNormalize`                  | Normalize types under post-analysis rules                            |
| `Subtyper`                               | Make remaining subtype relationships explicit                        |
| `ElaborateDrops`                         | Convert initialization-dependent destruction into runtime operations |
| `CheckDropRecursion`                     | Diagnose relevant recursive destruction behavior                     |
| `AbortUnwindingCalls`                    | Enforce ABI-specific unwind and termination behavior                 |
| `AddMovesForPackedDrops`                 | Handle moves needed for destruction of packed fields                 |
| `EraseDerefTemps` / `ElaborateBoxDerefs` | Remove or lower remaining specialized place representations          |
| `StateTransform`                         | Lower coroutine control flow into state-machine form                 |

The actual sequence also contains other diagnostic passes.

This ordering is not a freely rearrangeable list of independently correct optimizations. Normalization must precede some destruction processing; packed-drop lowering depends on earlier elaboration; coroutine conversion assumes particular runtime-MIR invariants.

An internal MIR extension must respect both the phase invariant of its input and the invariants expected by subsequent passes. A transformation that is locally well-typed can still be invalid at its chosen position in the pipeline.

Source: `run_runtime_lowering_passes`:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/lib.rs`

## 4. Compiler-generated MIR shims

### 4.1 MIR synthesis as a compiler service

The `shim` module extends `rustc_mir_transform` beyond modifying existing user-defined bodies.

It constructs MIR implementations for operations whose executable behavior is determined by Rust language semantics, type information, or calling conventions but does not correspond to an ordinary function body written in source.

The principal query integration is `mir_shims`, with the `ShimKind` representation defined in `rustc_middle`.

Relevant shim families include:

| Shim family                           | Generated behavior                                                                      |
| ------------------------------------- | --------------------------------------------------------------------------------------- |
| `VTable`                              | Adapt a virtual-method call to an appropriate concrete call and receiver representation |
| `FnPtr`                               | Adapt callable-trait invocation to an underlying function-pointer call                  |
| `Reify`                               | Produce a callable representation of a function item                                    |
| `ClosureOnce`                         | Adapt closure calling semantics, including consumption through `FnOnce`                 |
| `ConstructCoroutineInClosure`         | Construct a coroutine from coroutine-closure captures                                   |
| `DropGlue`                            | Generate type-directed destruction                                                      |
| `ThreadLocal`                         | Generate an appropriate thread-local access shim                                        |
| `Clone`                               | Generate clone-related glue where required by compiler semantics                        |
| `FnPtrAddr`                           | Handle function-pointer address conversion machinery                                    |
| `FutureDropPoll`                      | Construct asynchronous destruction polling glue                                         |
| `AsyncDropGlue` / `AsyncDropGlueCtor` | Construct asynchronous destruction bodies and constructors                              |

These operations vary substantially in shape.

A calling-convention shim may primarily adapt argument layout, receiver form, or invocation method. A drop shim may generate recursive destruction control flow. An asynchronous drop shim may itself require coroutine state-machine transformation.

A synthesized MIR body therefore has meaningful semantics and may require optimization, unwinding preparation, call-edge rewriting, and validation just as an ordinary body does.

### 4.2 Shim-specific transformation pipelines

The compiler does not route every synthesized body through exactly the same pipeline as source-authored functions.

The general shim-construction path uses selected passes including dereference handling, mentioned-item collection, packed-drop preparation, CFG simplification, forced inlining, unwind enforcement, and call-edge normalization.

Coroutine drop shims and asynchronous destruction shims have specialized paths. In particular, asynchronous drop glue construction can generate a body that is subsequently passed through `StateTransform` to create coroutine-style execution.

There are also validation differences: some generated shims are valid only under a post-analysis typing environment. Applying ordinary built-MIR validation indiscriminately can reject an otherwise appropriate generated body involving opaque types.

The key architectural insight is that MIR generation and MIR transformation are composable compiler services, but not all generated bodies pass through identical semantic stages.

This also explains why compiler consumers must distinguish source-defined MIR, synthetic MIR, shim MIR, and post-lowering MIR rather than assuming every body has the same origin or analysis history.

### 4.3 Async destruction availability

The presence of `FutureDropPoll`, `AsyncDropGlue`, and `AsyncDropGlueCtor` in Rust 1.99.1 establishes that the compiler contains implementation infrastructure for asynchronous destruction.

It does not establish that the user-facing `AsyncDrop` language feature is stable. The feature remains documented in the Rust Unstable Book, under `async_drop`, and its availability is subject to experimental feature gating.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rust Unstable Book



Similarly, an external compiler consumer cannot assume every internal `ShimKind` variant is independently constructible through a public, stable API.

Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/shim.rs`

`https://doc.rust-lang.org/nightly/unstable-book/language-features/async-drop.html`

## 5. MIR-based source coverage instrumentation

### 5.1 Instrumentation architecture

`rustc_mir_transform::coverage` implements source-based coverage instrumentation through the MIR pipeline.

The central pass is `InstrumentCoverage`, enabled through the appropriate coverage compilation configuration, such as `-C instrument-coverage`.

Its output combines executable instrumentation with metadata sufficient to relate runtime counter values to source-code regions and branches.

The implementation divides this operation into several stages:

1. Build a coverage-oriented control-flow representation through `CoverageGraph`.
2. Extract source information and mappings relating the MIR graph to source spans.
3. Calculate node-flow information and an ordering used to assign counters.
4. Insert `StatementKind::Coverage` statements containing virtual counter identifiers.
5. Store `FunctionCoverageInfo` in the MIR body for later consumption.
6. Resolve physical counters and expressions after later MIR transformations have affected the body.
7. Emit coverage intrinsics and source-mapping metadata during code generation.

The separation between virtual and physical counters is particularly significant.

A virtual counter represents a logical point in the coverage graph. A physical counter is actual instrumentation that increments when executable code is reached. Some logical counts can instead be represented as expressions over other counters.

Consequently, the number of source-level coverage regions does not directly determine the number of runtime counters. The compiler can use graph relationships and counter expressions to reduce instrumentation overhead.

The related `coverage_ids_info` query examines the final instance MIR to establish which instrumented basic coverage blocks remain after optimization.

It then determines which surviving blocks need physical counters and which can use expressions. Coverage information for blocks removed by optimization can be handled without pretending those blocks remain executable.

This is an important interaction between early semantic instrumentation and later optimization.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide



### 5.2 Source mapping, eligibility, and boundaries

Coverage instrumentation is inserted after MIR constant promotion as part of preparation for analysis MIR. The pass excludes promoted MIR bodies and certain ineligible functions.

Eligibility includes constraints such as:

- The definition must be function-like; constants are not directly instrumented for runtime coverage.
- Naked functions are excluded.
- Explicit `#[coverage(off)]` attributes can disable instrumentation.
- Coverage attributes can be inherited from enclosing definitions.
- Automatically derived implementations are normally treated as coverage-disabled unless applicable explicit attributes override that behavior.

The source mapping is informed by HIR information as well as MIR control flow. That matters because MIR operations can be transformed, lowered, combined, or eliminated while source coverage still needs to refer to meaningful source-code locations.

The source-level coverage system also depends on components outside `rustc_mir_transform`:

`rustc_mir_transform` constructs virtual instrumentation and mapping metadata; the code-generation backend emits the corresponding instrumentation and coverage map; LLVM's profiling runtime records execution counts; LLVM coverage tools decode and report them.

Thus, `InstrumentCoverage` is not itself a test runner, coverage database, or report renderer.

The official compiler development guide identifies `tests/mir-opt/coverage`, `tests/coverage`, and `tests/coverage-run-rustdoc` as distinct layers of validation, covering MIR instrumentation, emitted coverage maps, and end-to-end runtime reporting.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://rustc-dev-guide.rust-lang.org\&sz=32)

Rust Compiler Development Guide



Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/coverage/mod.rs`

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/coverage/query.rs`

`https://rustc-dev-guide.rust-lang.org/llvm-coverage-instrumentation.html`

## 6. MIR-derived dependencies and code-generation metadata

### 6.1 Mentioned-item collection before optimization

`MentionedItems` is a required MIR analysis that records code-generation-relevant references while the body is still generic.

Its visitor identifies, among other things:

| MIR construct                         | Recorded requirement                   |
| ------------------------------------- | -------------------------------------- |
| Ordinary and tail calls               | Referenced function type               |
| Drop terminators                      | Type potentially requiring drop glue   |
| Inline-assembly function symbols      | Function reference                     |
| Unsizing coercions                    | Possible vtable-generation requirement |
| Closure-to-function-pointer coercions | Referenced closure                     |
| Function-item reification             | Function reference                     |

These references are collected into MIR's `mentioned_items` information.

The collector deliberately mirrors relevant traversal logic in `rustc_monomorphize::collector`, but avoids performing monomorphization at the moment the references are recorded.

A critical detail is its execution before optional MIR optimization.

In `inner_optimized_mir`, rustc runs `MentionedItems` before `run_optimization_passes`. The source explicitly states that this preserves an item-discovery set that does not depend on optimization level.

That is important because optimizations may remove operations that originally referred to functions, destructors, or vtables. If all reference discovery depended only on the final optimized body, different optimization settings could lead to different generic dependency-discovery behavior.

A mentioned item is not automatically equivalent to a final emitted code-generation unit. The later monomorphization collector must still interpret those references in the context of concrete instantiations and its own reachability and emission rules.

The distinction is therefore among:

Generic references recorded in MIR, concrete instances discovered during monomorphization, and final code-generation artifacts.

These are related but not interchangeable representations.

Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/mentioned_items.rs`

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/lib.rs`

### 6.2 Deduced parameter attributes after optimization

`deduce_param_attrs` performs a different kind of analysis at the opposite end of the MIR optimization pipeline.

It examines `optimized_mir` to derive supplementary information about how function arguments and the return location are used.

The analysis records a compact `UsageSummary` with four categories:

| Usage flag      | Semantic meaning                                                             |
| --------------- | ---------------------------------------------------------------------------- |
| `DROP`          | The parameter may be dropped where destruction is required                   |
| `SHARED_BORROW` | A shared borrow occurs and could permit mutation unless the type is `Freeze` |
| `MUTATE`        | The parameter is mutated in a relevant way                                   |
| `CAPTURE`       | The parameter's storage or address may escape through a relevant operation   |

From this information, the compiler can derive two additional backend attributes for eligible indirectly passed parameters:

- `ReadOnly`: the parameter's indirect storage is not written through.
- `CapturesNone`: the corresponding pointer is not captured for use beyond the call.

The deduction is conservative. A destructor can invalidate both conclusions when a parameter's type requires dropping, and a shared borrow does not automatically establish read-only behavior for a type with interior mutability.

Calls and tail calls also require careful treatment. Moving an indirectly passed argument into another call may permit downstream mutation of the same underlying storage, even though no explicit assignment to the original parameter appears in the caller.

The compiler can serialize these deductions into crate metadata, allowing downstream compilation to benefit from eligible body-derived facts.

The inference is conditional:

- The implementation skips it when optimization is disabled or incremental compilation is active.
- It excludes bodies without applicable MIR and several unsupported parameter representations.
- It requires the relevant `Freeze` language-item infrastructure.
- Its ABI application is limited to eligible Rust ABI instances and indirectly passed arguments or return locations.
- The implementation relies on the assumption that subsequent code generation does not perform additional MIR transformations invalidating the inspected usage facts.

The currently implemented derived attributes are `readonly` and `captures(none)`-related. The internal documentation mentions additional `noalias`-style inference as a possible future extension, not as a released capability.

The key distinction from `MentionedItems` is intentional:

| Analysis              | Observed MIR                 | Primary requirement                                                                        |
| --------------------- | ---------------------------- | ------------------------------------------------------------------------------------------ |
| `MentionedItems`      | Before optional optimization | Preserve relevant code-generation dependencies independently of optimization-level changes |
| `deduced_param_attrs` | After MIR optimization       | Derive parameter guarantees that remain true of the code-generation representation         |

Together, these illustrate why no single MIR stage can be considered universally optimal for extracting all semantic and code-generation facts.

Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/deduce_param_attrs.rs`

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_middle/src/middle/deduced_param_attrs.rs`

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_ty_utils/src/abi.rs`

## 7. Coroutine witness analysis and storage layout

### 7.1 Suspension-point liveness as a type-system input

The `coroutine::layout` module exposes an important distinction between generating a coroutine state machine and establishing the semantic properties of values that can remain stored across suspension.

The `mir_coroutine_witnesses` query computes a `CoroutineLayout` containing types and storage relationships for values preserved across suspension points. This information is relevant to trait and region obligations, not merely physical layout optimization.

The underlying `locals_live_across_suspend_points` machinery combines several dataflow analyses:

| Analysis                     | Contribution                                                  |
| ---------------------------- | ------------------------------------------------------------- |
| `MaybeLiveLocals`            | Determines which values may be used after suspension          |
| `MaybeBorrowedLocals`        | Tracks locals whose addresses may have been borrowed          |
| `MaybeRequiresStorage`       | Determines whether storage must persist at a suspension point |
| `MaybeStorageLive`           | Bounds persistence according to storage-lifetime information  |
| `always_storage_live_locals` | Accounts for locals without explicit storage lifetime markers |

The combined result includes `saved_locals`, per-suspension-point live-local sets, source information, storage-liveness information, and `storage_conflicts`.

A key distinction concerns movable and immovable coroutines.

For a movable coroutine, ordinary value-liveness information is sufficient for the relevant saved-local computation because borrows into the coroutine's local storage cannot persist across suspension in the same way.

For an immovable coroutine, the analysis conservatively includes potentially borrowed locals, while still restricting the result according to storage requirements. This preserves necessary state for self-referential relationships that may remain valid only while the coroutine is pinned or otherwise immovable.

Therefore, the set of saved locals cannot be determined by ordinary syntactic last-use analysis alone.

### 7.2 Storage-conflict matrices and field reuse

`LivenessInfo::storage_conflicts` is a `BitMatrix<CoroutineSavedLocal, CoroutineSavedLocal>` describing saved-local pairs whose storage requirements conflict.

A conflict means that the corresponding locals cannot safely occupy overlapping storage in the generated coroutine representation.

Conversely, saved locals whose storage lifetimes do not conflict may be candidates for overlapping storage, subject to the other constraints of coroutine layout.

The compiler combines this information with per-state live-local sets to construct the coroutine's layout.

This is particularly useful because many values are needed only during different portions of a coroutine's execution. Preserving all values as permanently distinct storage would forgo opportunities to reduce state-machine size.

However, the absence of a storage conflict does not alone guarantee that two fields will be assigned the same storage. Layout also depends on type layout, alignment, state representation, and other implementation constraints.

### 7.3 Suspension diagnostics derived from witness information

The same layout and saved-local information supports diagnostics involving values that should not remain live across suspension.

The implementation contains `must_not_suspend` checking that examines relevant saved types and their source locations, including certain composite and trait-object forms.

It uses the information calculated for each suspension point to relate a potentially problematic type to the location where the coroutine suspends.

This illustrates a broader capability combination:

Liveness analysis → saved-local types and storage relationships → trait/region reasoning, suspension diagnostics, and state-machine layout.

The witness calculation therefore serves multiple compiler consumers. It is not merely a preparatory optimization for `StateTransform`.

Availability of specific language attributes and diagnostics remains governed by their Rust feature and lint configuration; the presence of internal checking logic does not imply every associated feature is stable.

Source:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/coroutine/layout.rs`

## 8. Additional MIR diagnostics and runtime checking

### 8.1 Place-sensitive liveness diagnostics

The `rustc_mir_transform::liveness` module implements the `check_liveness` query.

Its primary purpose is to identify unused assignments, unused captures, and related source-facing issues using MIR-level dataflow.

It operates through a `MaybeLivePlaces` analysis, rather than relying exclusively on the general `MaybeLiveLocals` abstraction.

This difference matters because diagnostic reasoning often needs information about individual places, captured fields, assignments, and source-level declarations that a coarse local-liveness bitset would not preserve.

The implementation performs special handling for closure bodies. In particular, a `FnMut` closure can mutate captured state during one invocation and read that state during a later invocation.

To represent this possibility, the analysis constructs a modified MIR body in which returns and unwind-resume points conceptually lead back to the starting block.

This makes repeated invocation visible to the analysis without requiring an unrestricted interprocedural analysis of every caller of the closure.

The checker also uses MIR debug information to recover source-oriented variable information, allowing it to distinguish meaningful assignments and captures when producing diagnostics.

The key distinction is that diagnostic liveness is not necessarily the same analysis as storage liveness, borrow-region liveness, or conventional value liveness.

Different consumers require different abstractions and different assumptions about how execution may continue.

Source:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/liveness.rs`

### 8.2 Edition-sensitive tail-expression destruction order

`lint_tail_expr_drop_order` is a more specialized example of MIR analysis used for source-language compatibility diagnostics.

Rust 2024 changed the lifetime of certain temporaries in tail expressions, allowing them to be dropped before local variables that previously would have been destroyed first.

The `tail_expr_drop_order` lint detects cases where this difference may be observable, particularly when a temporary has a significant user-defined destructor. It is an allow-by-default migration diagnostic rather than a guarantee that a behavior change is harmful.

The implementation uses `MaybeInitializedPlaces`, `MoveData`, move-path lookup, and control-flow traversal to determine whether relevant destructor operations can occur.

This is important because a syntactic `Drop` terminator does not alone prove that a destructor will actually run. A value may have been moved, remain uninitialized, or follow only particular control-flow paths.

The lint therefore combines source-level intent with MIR initialization facts and reachability-sensitive destruction analysis.

It demonstrates a further use of the dataflow framework: identifying potential observable changes in language semantics across editions.

Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/lint_tail_expr_drop_order.rs`

`https://doc.rust-lang.org/edition-guide/rust-2024/temporary-tail-expr-scope.html`

The Rust Edition Guide also explains why this lint does not offer a universally semantics-preserving automatic rewrite.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

The Rust Edition Guide



### 8.3 Optional runtime checks for invalid MIR operations

`rustc_mir_transform` contains passes that insert runtime checks for selected conditions associated with undefined behavior.

These must be distinguished from static diagnostics and MIR structural validation.

Three particularly relevant passes are:

| Pass             | Runtime property checked                                                               | Qualification                                                                       |
| ---------------- | -------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `CheckAlignment` | Whether selected pointer dereferences satisfy the required alignment                   | Uses target-specific alignment information and excludes trivial cases               |
| `CheckNull`      | Whether selected dereferences or reference constructions involve invalid null pointers | Distinguishes reference construction from other uses, including zero-sized pointees |
| `CheckEnums`     | Whether selected enum values have valid discriminants or niche representations         | Inserts checks at targeted locations rather than attempting full dynamic validation |

These passes use `PassPolicy::optional_non_optimization(sess.ub_checks())`.

They are conditional on the compiler's UB-check configuration and are not necessarily present in every compilation.

Alignment checks. `CheckAlignment` can insert pointer-address calculations and assertions for misaligned dereferences. It accounts for the required alignment of a projected field rather than merely checking the alignment of an enclosing pointer type.

Null-pointer checks. `CheckNull` distinguishes situations where constructing a reference to null is invalid even for a zero-sized pointee from other pointer operations where the pointee's size affects whether a dereference must be checked.

Enum validity checks. `CheckEnums` inserts checks for selected enum constructions and discriminant manipulations, accounting for direct discriminant encodings, niche-based encodings, and uninhabited variants.

The enum checker intentionally does not instrument every operation that could conceivably involve an invalid value. Its source explains that fully general checking comparable to Miri would generate excessive MIR.

These facilities provide selected runtime defenses and diagnostics. They do not replace the language's static safety rules, a complete memory-safety interpreter, or the compiler's structural MIR validator.

Their placement before optimization is also significant: checks can be inserted before subsequent transformations remove evidence of problematic operations.

Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/check_alignment.rs`

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/check_null.rs`

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/check_enums.rs`

## 9. MIR pass policy, isolation, and independent validation

### 9.1 Required passes versus optional operations

The `pass_manager` module defines a policy mechanism that is more expressive than a simple minimum optimization level.

`PassPolicy` distinguishes:

| Policy                             | Execution semantics                                                                                                        |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `Required`                         | Mandatory for the applicable MIR semantics or representation invariants; cannot be disabled through ordinary pass controls |
| `Optional { optimization: true }`  | Optimization whose execution depends on configuration and applicable optimization controls                                 |
| `Optional { optimization: false }` | Non-optimization operation, such as instrumentation or a runtime check, that may still be independently controlled         |

This distinction is important because the presence of a pass in an optimization-oriented sequence does not establish that it is itself an optimization.

`CheckEnums`, for example, inserts checks; `InstrumentCoverage` inserts instrumentation; `PostAnalysisNormalize` establishes required semantic invariants.

The pass manager also provides `WithMinOptLevel`, which restricts the activation of an otherwise eligible pass according to the configured MIR optimization level.

The `#[optimize(none)]` policy affects optional optimization passes, not every optional non-optimization or required pass.

Internal controls such as `-Zmir-enable-passes` and `-Zmir-opt-level` operate within these policy boundaries. They are unstable compiler controls, not standardized interfaces for downstream pass registration.

### 9.2 MIR pass interfaces and observational separation

`MirPass` exposes a mutable-body transformation interface, while `MirLint` is an inspection interface that receives an immutable MIR body.

The `Lint<T>` adapter enables a `MirLint` to participate in the pass pipeline.

This distinction permits some diagnostic behavior to be integrated into the same orchestration and profiling machinery as transformations without granting those diagnostics authority to mutate the body.

The pass manager also coordinates naming, profiler labels, MIR dumping, validation, and phase transitions.

These mechanisms are compiler-internal. Their existence does not create an independently loadable MIR plugin architecture.

In particular, the traits and associated pass-policy machinery are largely crate-private, and arbitrary downstream registration requires substantially more integration than implementing a trait in an external Cargo crate.

Source:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_transform/src/pass_manager.rs`

### 9.3 Isolated MIR transformation testing

Rust's `tests/mir-opt` infrastructure supports targeted assertions over individual MIR transformations.

Its test directives include:

| Mechanism                     | Verification capability                                                          |
| ----------------------------- | -------------------------------------------------------------------------------- |
| `//@ test-mir-pass: PassName` | Restricts execution to the selected pass for a focused MIR test                  |
| `EMIT_MIR ...diff`            | Compares the before/after transformation                                         |
| `EMIT_MIR ...before.mir`      | Captures pre-pass MIR                                                            |
| `EMIT_MIR ...after.mir`       | Captures post-pass MIR                                                           |
| `EMIT_MIR_FOR_EACH_BIT_WIDTH` | Allows architecture-width-specific expected outputs                              |
| FileCheck directives          | Match properties of emitted runtime MIR, including named basic blocks and locals |

The FileCheck path is distinct from individual pass snapshots: the Rust 1.99.1 test documentation specifies that FileCheck operates on runtime MIR from `--emit=mir`, not on the output of a selected individual pass.

Pass isolation reduces interference from unrelated transformations, including transformations whose behavior may change across compiler revisions.

Other relevant validation mechanisms include `-Zvalidate-mir`, compiler debug assertions, and `tests/ui` coverage of diagnostics. The development guide recommends MIR validation when investigating transformation bugs and experimental MIR optimization configurations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



These facilities complement `rustc_peek` rather than duplicating it:

- `rustc_peek` checks selected abstract dataflow facts.
- MIR-opt snapshots inspect representation changes.
- MIR validation checks compiler representation invariants.
- UI tests verify source-facing diagnostics.
- Code-generation and coverage tests inspect downstream emitted behavior.

Together, these mechanisms provide several levels of native verification without requiring a separate framework for every analysis or transformation.

Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/tests/mir-opt/README.md`

`https://rustc-dev-guide.rust-lang.org/mir/debugging.html`

`https://rustc-dev-guide.rust-lang.org/tests/compiletest.html`

## 10. Version and API availability boundaries

### 10.1 Newer nightly dataflow facilities

Newer nightly compiler documentation exposes `rustc_mir_dataflow::impls::precise_liveness`.

That module computes fine-grained allocation-liveness information, including split early/late statement points and storage-sharing opportunities.

It is a meaningful advanced development capability but is not part of the inspected Rust 1.99.1 `impls` module. The relevant nightly documentation describes Rust 1.101.0-nightly rather than the target stable compiler.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://doc.rust-lang.org\&sz=32)

Rust



The Rust 1.99.1 reference should therefore not incorporate its API or semantics as if they were available in the tagged stable release.

Sources:

`https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_mir_dataflow/src/impls/mod.rs`

`https://doc.rust-lang.org/nightly/nightly-rustc/rustc_mir_dataflow/impls/precise_liveness/index.html`

### 10.2 Availability is not equivalent to extensibility

The additional mechanisms in this addendum fall into several distinct categories.

| Category                                              | Examples                                                      | Integration boundary                                                                 |
| ----------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| Compiler-internal reusable machinery                  | Dataflow edge selection, drop-flag effects                    | Requires compatible rustc internal types and lifecycle                               |
| Internal compiler queries                             | `mir_coroutine_witnesses`, `deduced_param_attrs`, `mir_shims` | Access depends on the compiler query context and supported consumers                 |
| Compiler-owned transformations                        | `Subtyper`, `PostAnalysisNormalize`, `AbortUnwindingCalls`    | Generally not independently replaceable through a stable plugin interface            |
| Public compiler behavior with internal implementation | Source coverage instrumentation, edition migration lints      | User-facing feature may be stable while internal implementation APIs remain unstable |
| Experimental language/compiler capabilities           | Async destruction, newer nightly dataflow modules             | Require feature-specific version and channel verification                            |

This is particularly important for compiler-integrated analysis tools. The existence of a powerful internal module does not necessarily mean its functionality can be reused through a public API without participating in rustc's query, phase, and type-context machinery.

## 11. Combined architectural implications

The additions reveal several useful patterns that are not obvious when examining individual passes in isolation.

| Combined capability                        | Native composition                                                                                      | Architectural significance                                                                                                  |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Opaque-type-aware MIR lowering             | Root-level borrow checking, hidden-type inference, post-analysis normalization, explicit subtype casts  | Retains appropriate abstractions through type checking, then progressively establishes runtime representation               |
| ABI-correct execution                      | Early FFI unwind discovery, required panic strategy, unwind-edge rewriting                              | Coordinates compile-time analysis, crate metadata, linking requirements, and runtime control flow                           |
| Source coverage that survives optimization | Early virtual counter insertion, source mapping, post-optimization counter resolution, backend emission | Separates semantic coverage locations from runtime counter placement                                                        |
| Cross-crate code-generation precision      | Pre-optimization mentioned-item collection and post-optimization parameter-attribute deduction          | Preserves dependency completeness while extracting stronger guarantees from optimized representations                       |
| Coroutine correctness and compact state    | Borrow/storage liveness, witness calculation, storage-conflict matrices, state transformation           | Reuses semantic analysis for both type-system obligations and executable state-machine layout                               |
| Compiler-native analysis validation        | Dataflow assertions, isolated MIR pass tests, MIR validation, UI diagnostics, code-generation checks    | Verifies abstract facts, transformation effects, representation invariants, and observable behavior at different boundaries |

The common pattern is purpose-specific extraction and transformation at different stages of a shared MIR lifecycle.

A compiler does not need to preserve every form of information indefinitely, nor should it perform every analysis on the most optimized representation available.

Instead, Rust 1.99.1 uses early MIR to capture properties that later transformations could erase; borrow-checking-time MIR to resolve ownership, regions, and opaque types; post-analysis normalization to establish runtime semantics; and optimized MIR to derive additional code-generation information.

That phase-dependent specialization, supported by reusable dataflow and MIR infrastructure, is the principal additional architectural capability established by this addendum.