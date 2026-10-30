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