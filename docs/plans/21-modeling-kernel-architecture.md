---
title: Modeling kernel — knowledge-agnostic architecture
status: draft
date: 2026-09-26
parent: docs/plans/21-modeling-kernel.md
review_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md]
---

# Modeling kernel — knowledge-agnostic architecture

**Evidence level: Proposed.** This is the architecture of the modeling kernel that
[Plan 21](21-modeling-kernel.md) implements before any scientific model is ported. It refines
[Plan 20](20-idaes-capability-target.md): Plan 20's decisions A1–A10 were agreed by the
maintainer on 2026-09-26, and this document states how A3–A6 and A8–A9 are realized. Where it
differs from the Plan 20 companion documents, this document governs (see
[Plan 21 *Decisions*](21-modeling-kernel.md#decisions)).

## 1. The principle: mechanisms in code, knowledge in data

A process simulator at idaes-pse scope holds two kinds of content.

1. **Mechanisms.** Typing, composition, instantiation, conservation accounting, differentiation,
   discretization, implicit realization, solving, initialization, scaling, diagnostics,
   identity, lineage, publication. They are few, general, hard to get right, and belong in Rust
   on top of the libraries (Symbolica/Numerica, faer, native solvers, Salsa, Arrow).
2. **Scientific knowledge.** Unit operations, control volumes, state definitions, equations
   of state, activity models, correlations, reaction forms, phase-equilibrium formulations,
   costing correlations, controllers, surrogates, start estimates, scaling heuristics,
   parameter data and reference values. There is a great deal of it. It is open-ended, and
   it belongs in **packages**, as data written in one modeling language.

The kernel defines **no scientific concept**. It has no "unit model", "property package",
"equation of state", "control volume", "reaction", "cost" or "phase" type. Those are names
that packages give to ordinary definitions, interfaces, sets and tables. The **knowledge
boundary test** is the architecture's acceptance criterion:

> Porting any idaes-pse unit model, property method or package, reaction form, costing method,
> controller or surrogate consists only of package files (definitions, interfaces, functions,
> sets, tables, datasets, annotations and conformance tests). It is done when its conformance
> tests pass. If a port needs a Rust change, that is a **kernel gap**. The gap is generalized
> and closed in the kernel, and never special-cased for the model.

This is the core foundations applied to the whole simulator:

- **AP-01.** The reason a mechanism changes differs from the reason knowledge changes.
- **AP-03.** New knowledge composes through existing mechanisms.
- **AP-04.** Each scientific fact has one declaration.
- **DP-06.** Structure is authored once.
- **DP-16.** An extension is one authoritative declaration.
- **§E extension locality.**

## 2. Kernel concepts

Twelve orthogonal concepts carry every construct the idaes-pse characterization contains. The
[knowledge placement guide](21-knowledge-placement.md) shows where each IDAES construct maps.

| # | Concept | What it is | Replaces (current system) | Replaces (IDAES) |
|---|---|---|---|---|
| K1 | **Quantity types and units** | Complete physical type: kind, dimension, basis, reference, scale, shape. Units, conversion rules, constants. Existing `pse-quantity`, extended with quantity **type variables** for polymorphic functions | — (kept) | Pyomo units, `units_of_measurement` |
| K2 | **Entity kinds, entities and attributes** | Package-declared kinds (a package may declare `species`, `phase`, `element`, `reaction`) with typed attributes. Entities are identified members | `species`, `phases`, `elements`, `material_systems` relations | `Component`, `Phase` classes, `component_list` |
| K3 | **Sets** | Finite sets of entities or labels; derived sets (filter, product, union, projection, relation image); ordered and ragged sets; **continuous sets** (intervals) that are discretized or integrated. Compile-time values | `domains`, `domain_members`, material-derived domain refusals | Pyomo `Set`, `ContinuousSet`, `phase_component_set` |
| K4 | **Tables and datasets** | Typed tables keyed by sets (for example `stoich[r, p, j]`, `rpp4[j, coeff]`, `material_factor[m]`). Datasets supply values with provenance, units and a missing-value policy. Sources are inline, CSV, or Arrow/Parquet admitted through the data layer | `parameter_values`, `stoichiometry`, `species_elements`, `method_parameters`, `default_scaling` | `parameter_data`, config dictionaries, JSON parameter files |
| K5 | **Functions** | Pure, typed, quantity-polymorphic expression functions with explicit arguments (`fn smooth_max<Q>(a: Q, b: Q, ε: Δ<Q>) -> Q`). Validity annotations. **Verified piecewise** functions whose continuity order the kernel proves at breakpoints. The body is inlined at specialization | The `Function` enum names beyond primitives (smoothing, `cbrt`); the valve-law provider | `smooth_max`, `safe_log`, `cbrt` external function, method `return_expression`s, callbacks |
| K6 | **Definitions** | The one composable modeling unit: parameters, symbols (`var`, `param`, `let`, `alias`), equations (including implicit blocks), children (including indexed children), ports, accumulators and contributions, annotations, requirements, `when` variants, interface implementations | Templates and their 25 relation families; computation models; recipes | `ProcessBlockData`, `build()`, every unit, CV, state block, method class and costing block |
| K7 | **Interfaces** | Named contracts over definitions: required parameters, members (typed, indexed) with optional **default definitions**, ports. Parameters and children can be typed by interface (slots). Definitions `impl` interfaces | Provider specs as the only contract; method families; template kinds | Python duck typing, config callbacks, `get_method`, subclassing |
| K8 | **Lazy instantiation and dispatch** | A definitional member is instantiated only when demanded, together with its defining equations. Member-dependent **dispatch tables** bind a different implementation per set member; specialization groups members by implementation | Explicit per-call provider demand (D8, §9.6) | `build_on_demand`, per-component `get_method`, per-phase EoS |
| K9 | **Accumulators and contributions** | Indexed accounting subjects (`conservation` or `accounting`) with role-tagged contributions from any descendant, internal transfers, closure checks and tolerances | Scalar law instances, `physical_balances`, reaction projection | Control-volume balance construction, costing aggregation |
| K10 | **Operators** | Expression operators: arithmetic; primitive math (code); reductions over sets with filters; compile-time `if`; runtime `if` (value-only, flagged); **∂f/∂arg** on functions (Symbolica); **d(x)/d(s)** over continuous sets; integrals; table lookup; attribute access; explicit typed `convert` | Parts of the current DSL | Pyomo expressions, `DerivativeVar` |
| K11 | **Transformations and realization policies** | Kernel passes with declared contracts: discretization (schemes are data); implicit-block realization (inline or nested); elastic relaxation; continuation over parameters. Analysis mode and initialization stage are **compile-time facts** that `when` variants read | Retired discretization; refused phase equilibrium; supplied continuation | Pyomo DAE transformations, `homotopy`, `ipopt_l1` |
| K12 | **Annotations, requirements and tests** | Annotations on symbols and equations: `start`, `nominal`, `bounds`, `scale`, `valid`, `report`, `check`. Requirements: compile-time admission predicates with messages. Tests: conformance cases with expected values, tolerances and oracle sources | Refused initializer and scaler templates; scattered fixtures | Initializers, scalers, `ConfigurationError`, stream tables, IDAES tests |

Cases, analyses and studies (existing workflow concepts, generalized in Plan 21 K6) bind a root
definition to values, modes and policies. Packages (existing) version and scope everything.

## 3. Semantics

### 3.1 Definitions and instantiation

A definition is a pure declaration. **Instantiation** binds its parameters (typed compile-time
values, including sets, tables, entity references, definition references and interface-typed
slots) and produces an instance with an identity path.

- **Eager content.** Structural equations, accumulators, contributions, ports, requirements and
  `child` declarations are instantiated with their instance.
- **Lazy content.** Definitional members are instantiated only when some instantiated content
  (transitively) references them: `let` expressions, `var`s declared with a defining equation
  (`var x defined by eq …`), and implicit blocks. This is the whole of "property on demand",
  generalized. Each member owns its variables and defining equations. Their joint inclusion is necessary,
  but does not prove well-posedness: structural matching still validates the resulting model.
- **Guards.** `when predicate { … }` blocks are resolved from compile-time values:
  parameters, set sizes, entity attributes, interface capability queries (`implements`,
  `provides`), analysis mode, initialization stage and scope values. Unused alternatives leave no
  structure (§10.5 kept).
- **Requirements.** `require predicate : "message"` statements are evaluated at
  specialization, including over table data. Examples: element closure of every reaction,
  electroneutrality, and "phase equilibrium needs two phases". A violation refuses admission
  with the instance path and source span (DP-03). IDAES `ConfigurationError`s become
  requirements.
- **Scope values.** A parameter may default to a value published by an ancestor scope
  (`= scope.default_property_package`), resolved statically and recorded in lineage. This
  covers IDAES `useDefault` and flowsheet inheritance without runtime traversal.

### 3.2 Interfaces, defaults and slots

An interface declares:

- parameters;
- members, each with a quantity type, index sets and role (`var`, `let`, `fn`);
- ports;
- optional **default member definitions** written in terms of other members.

A definition that `impl`s the interface supplies the non-default members. It may override a
default; the override is recorded in lineage.

This single mechanism covers several things that are hand-coded elsewhere:

- **Thermodynamic identities are data.** An interface `HelmholtzPhase` requires
  `fn alpha_r(T, rho, n[j])` and gives default members `pressure`, `enth_mol`, `entr_mol`,
  `ln_phi[j]` and so on, as expressions over ∂-derivatives of `alpha_r`. A new equation of
  state implements one function. The IDAES explicit-δ cubic is an implementation that
  overrides `ln_phi`.
- **Strategies are slots.** A heat-exchanger parameter typed `DeltaT` accepts any definition
  implementing `DeltaT`: `LMTD`, `SmoothLMTD`, `AMTD` or `Underwood`. The same holds for valve
  characteristics, pressure-changer assumptions, performance curves and custom terms.
- **Property packages are compositions.** A package is a definition implementing a
  `ThermoPackage` interface by binding its members (phase models, state definition, VLE
  formulation, datasets) to other definitions.

Interfaces may extend interfaces, because contracts compose. There is **no implementation
inheritance**. **Presets** are named partial bindings (`preset Pump = PressureChanger(compressor
= true, assumption = PumpAssumption)`); they add no content, and lineage records them.

### 3.3 Dispatch

A child or a function-typed parameter may be indexed by a set and bound per member through a
table of definition or function references:

```text
child cp_ig[j in components] : IdealGasCp = pkg.cp_method[j]      // RPP4 for benzene, NIST for CO2 …
```

Specialization groups members that bind the same implementation with the same compile-time
parameters. Each group becomes one body specialization, and instances bind per member. This is
the IDAES per-component `get_method` without code. It keeps the "one evaluator per
specialization" direction (U3) and shares bodies across identical members.

### 3.4 Accumulators

An accumulator is declared in the definition that owns the subject:

```text
accumulate material[t in time, p, j in phase_components] conservation (tolerance = 1e-8 mol/s)
```

Any instantiated descendant contributes, with a role that fixes the sign: `inflow`, `outflow`,
`generation`, `consumption`, `accumulation`, `transfer(id, side)`, `heat_in`, `work_in` and
their duals. The kernel enforces the following:

- every contribution targets exactly one accumulator member;
- every contribution's quantity type equals the accumulator's;
- a transfer appears exactly twice with opposite sides;
- a conservation accumulator becomes the equation `Σ signed contributions = 0`;
- an accounting accumulator defines its value as the signed sum;
- closure is recomputed independently from the same contributions (PS-03, D7 kept).

What was "material, energy, element, momentum, charge, cost" in code is just the accumulator
name, index and quantity type chosen by package data.

### 3.5 Expressions and operators

The current DSL grows into the modeling language's expression layer:

- **Functions** (K5) inline at specialization. Physical typing runs on the inlined body and at
  the signature, with type variables solved per call.
- **`∂f/∂a`** differentiates a function with respect to a named argument, or an argument member
  such as `n[j]`. It returns a function with the same signature, and Symbolica performs it after
  inlining. There is no "holding constant" ambiguity, because arguments are explicit.
- **`d(x)/d(s)`** declares a derivative of an indexed variable over a continuous set. It is
  realized by discretization, or for time by the integrated DAE route.
- **Runtime `if`** on free values stays value-only and marks the body nonsmooth
  ([§7.3](../authoritative_design/sections/mathematics-and-compilation.md#section-7-3)).
  Smooth behaviour is written with data-defined smoothing functions.
- **Verified piecewise functions.** `piecewise` declares breakpoints and a claimed continuity
  order. The kernel proves value and derivative agreement at each breakpoint symbolically
  before admitting the claim. The directional valve law's C² quintic becomes data, and its
  smoothness claim is established rather than asserted.
- **Primitive functions** in code are only those the pinned Symbolica evaluator treats as
  built-in symbols: `exp`, `log`, `sqrt`, `sin`, `cos`, `abs`, plus power (the skill's index
  lists `Symbol::EXP`, `LOG`, `SIN`, `COS`, `SQRT`, `ABS`). Each keeps its domain obligation
  (§7.3). Hyperbolic functions, `asinh`, `sigmoid`, `softplus`, `log10`, `tan`, `cbrt` and every
  smoothing function are package data composed from these. A function with no elementary form
  (for example `erf`) is an external function (§5) until a library evaluator supports it.
  Everything else is package data.

### 3.6 Implicit blocks and realization

`implicit { unknowns } such that { equations } realize <policy>` declares that the unknowns are
defined by the equations. Examples are a density root, association site fractions, bubble
temperature, a phase split, or an isentropic outlet temperature. The block also carries:

- start annotations;
- bounds that select a branch;
- optionally, a **regime selection**: alternative equation sets with a selection criterion,
  such as minimum Gibbs energy or a stability predicate.

Realization is a policy value, set in data by the package or the analysis:

| Policy | Meaning | Derivatives |
|---|---|---|
| `inline` | Unknowns and equations join the outer problem (equation-oriented; IDAES default for modular packages) | Exact, from the outer program |
| `nested` | A nested-solve **evaluation stage**: the compiled inner residual and Jacobian, a qualified inner root solver (KINSOL, supplied through an `InnerSolver` capability owned by `pse-backend-native`), residual-verified convergence, and regime selection where declared | Implicit-function theorem from the converged inner Jacobian (faer), to the order the outer body needs |
| `accelerated(id)` | A registered **accelerator** for a recognized implicit form, for example analytic cubic roots. It is validated against `nested` on the same block by a conformance test | As the accelerator declares |

Flash, density roots, saturation and pure-fluid (p, h) inversion therefore do not need
bespoke closure code. They are implicit blocks in packages, and algorithms are optional
accelerators. A nested stage's per-worker warm start is declared with a determinism class (DP-11):
results agree within the inner tolerance regardless of the warm start, and the tolerance is
recorded. It never crosses attempts (DP-19).

### 3.7 Analysis modes and continuous sets

Analysis mode is a compile-time fact (`analysis.dynamic`, `analysis.route`). Package data
writes the accumulation contribution inside `when analysis.dynamic and has_holdup`. The kernel
has no dynamic model generator: the same definitions serve every mode (PS-11).

Continuous sets are realized in one of three ways:

- by the **discretization transformation**, using a scheme chosen per set by policy. Schemes
  are package data: stencil definitions for finite differences, and a collocation family whose
  nodes come from the kernel primitive `jacobi_roots(n, α, β)`, certified by Symbolica
  `isolate_real_roots`;
- by the **integrated route**, for time only (Diffsol/IDAS, existing);
- by **simultaneous time discretization**, for dynamic optimization.

Every generated row and variable keeps a mapping to `(symbol, coordinate)` (PS-05).

### 3.8 Annotations consumed by generic engines

| Annotation | Consumer | Semantics |
|---|---|---|
| `@start(expr)` | Initialization engine | Evaluated in structural predecessor order to propose starts; a start with provenance, never a fix (PS-08) |
| `@nominal(expr)` / `@scale(scheme)` | Numerical policy resolution | Ranked source "model hint"; `scale` selects the IDAES constraint scaling scheme names for derived term-magnitude nominals |
| `@bounds(lo, hi)` | Case layout | Declared physical bounds; case overrides by precedence |
| `@valid(ranges, policy)` | Admission and evaluation | Validity envelope compiled as obligations; `reject` or a selected, recorded extrapolation (PS-02) |
| `@check(expr ≥ −tol)` | Post-solve qualification | Knowledge-specific verification, for example tangent-plane stability of a reported phase split, recorded with the result (PS-10) |
| `@report(label)` | Result projections | Stream tables and performance tables as Arrow relations |
| `stage "name" { … }` | Initialization engine | Named compile-time variant; the initialization policy sequences stages and transfers values by symbol identity |

## 4. The pipeline

```text
package sources ──► parse + name resolution ──► typed IR (generic registry families)
                    (pse-authoring)              (generated pse-model values; registry-declared shapes)
case / analysis ──► root + bindings + facts (mode, stage, policies)          (pse-runtime admission)
                         │
                         ▼  Salsa-tracked (pse-compiler queries over pse-modeling logic)
   specialize: guards → requirements → interface default/override selection → eager instantiation
             → lazy demand closure → dispatch grouping
             → function inlining, ∂ → accumulators and contributions → ports and connections
             → annotations → lineage
                         ▼
   transform: discretize continuous sets → realize implicit blocks → elastic / continuation
                         ▼
   lower: bodies per (definition specialization, dispatch group) → Symbolica atoms
          (existing typed_math path) → case plan, structure, facts, artifacts        (pse-math)
                         ▼
   engines: numerical policy (hints, derived nominals) → initialization (starts, stages,
            blocks, homotopy) → class-specific solve → post-solve checks (declared + closure)
            → diagnostics → studies → results and publication  (runtime, backend-native)
                         ▼
   conformance runner: tests → cases → the same pipeline → expectations with tolerances
```

Every stage is identity-bearing and incremental. An edit to one function re-specializes only
its dependants. A runtime-value change reuses structure only when it cannot affect guards, membership, dispatch
or other structural facts. Structural numeric arguments invalidate their dependants (DP-09, PS-11). The mapping from
every generated row, variable and body back to `(package, definition, instance path, member,
dispatch group, interface default or override, transformation)` is the lineage published with
results (DP-21).

## 5. Crates and dependencies

This refines Plan 20 decision A4.

| Crate | Role in the kernel | Change |
|---|---|---|
| `pse-quantity` | K1, plus quantity type variables and polymorphic signatures | Extended |
| `pse-authoring` | Modeling-language front-end: lexer and parser, spans, round trip, identities, edits; the existing DSL becomes its expression layer | Extended |
| `pse-modeling` (new) | K2–K12 semantics: typed IR operations, name resolution, interface conformance, specialization, lazy demand, dispatch, accumulators, transformations, requirements, lineage. Pure: no Arrow, Tokio or native solvers | New crate |
| `pse-compiler` | Salsa workspace; specialization and transformation become tracked queries that call `pse-modeling` | Extended |
| `pse-math` | Lowering; the ∂ operator via Symbolica; the nested-solve evaluation stage (consuming an `InnerSolver` capability); external-function stage; `jacobi_roots`; piecewise continuity proofs; term magnitudes for derived nominals | Extended |
| `pse-backend-native` | `InnerSolver` implementation (KINSOL); existing class adapters | Extended |
| `pse-kernels` | Becomes the **external-function** host: a generalized provider contract with typed index shapes, used only where a function cannot be data (ONNX inference, optional reference libraries). The FeOS provider and the valve-law provider are deleted once their knowledge is data | Reduced |
| `pse-runtime` | Admission boundary, engines (initialization, studies), jobs, results, publication. Composition semantics are removed | Reduced |
| `pse-schema` | Registry: generic IR families replace the science-specific families (Plan 21 packet K2) | Consolidated |

Plan 20 A4 named three new crates: `pse-properties`, `pse-modeling` and `pse-thermo`. Only
`pse-modeling` remains. Thermodynamics has no code concept left to own: identities, packages
and closures are data (§3.2, §3.6).

Dependency ownership: `pse-modeling` consumes plain `pse-model`, `pse-quantity` and source
syntax from `pse-authoring`; `pse-compiler` calls the pure kernel and owns the only Salsa
database. It invokes `pse-math` for symbolic operations. Neither the language front end nor
`pse-modeling` depends on native mathematics, solvers, Arrow or runtime effects. `pse-math` defines `InnerSolver`, and the
runtime composition root injects the backend implementation into workers (AP-02, DP-17).

## 6. Registry consolidation

The IR is declared once in the registry (D1 kept). Deletions below occur when the last
consumer moves: generic composition in K3, external contracts in K5, dynamic authoring in
K6 and scientific knowledge/providers in K8. They are not all K1 deletions.
It is small and generic:

| Kept or generalized (generic) | Subsumed (science-specific, deleted) |
|---|---|
| `packages`, `documents`, `scopes`; `entities` (+ entity kinds and attributes); `domains`, `domain_members` (sets); `datasets`, `parameter_values` (tables); `templates` → definitions and their parts (parameters, symbols, equations, guards and features → `when`, submodels → children, ports, contributions, law instances → accumulators); interfaces; bindings and dispatch tables; functions; annotations; requirements; tests; `instances`, `connections`; `cases`, `case_specs`, `case_sets` (studies), `fit_cases` (estimation analyses), `observations`; reference physical families (units, quantities, conversion rules, constants) | `method_specs`, `method_selections`, `method_dependencies`, `method_provisions`, `method_parameters`, `method_parameter_axes`, `method_state_parameters`, `method_precedence`, `method_kernel_inputs`, `property_kinds`, `property_packages`, `phase_equilibrium_pairs`, `henry_declarations`, `reaction_packages`, `reaction_methods`, `reaction_applications`, `state_bounds`, `default_scaling`, `provider_scaling_bindings`, `native_providers` (FeOS columns → generic external functions), `directional_valve_laws`, `template_material_constraints`, `template_property_requirements`, `template_symbol_properties`, `element_projection_contracts`, `law_bindings`, `physical_balances` (derived), `species`, `phases`, `elements`, `species_elements`, `phase_species`, `stoichiometry`, `material_systems` (→ package entity kinds and tables), `dynamic_cases` (→ analysis declarations) |

The IDAES enumerations preserved by name (§6.14) become package enums in the prelude and chemistry
packages. The registry keeps only enums that the kernel itself interprets: roles, realization
policies, analysis modes and outcome tags.

## 7. Alignment with the design standard

| Principle | How the kernel meets it |
|---|---|
| AP-01 Separation of concerns | Mechanisms (kernel crates) and knowledge (packages) change for different reasons and live apart. Within the kernel: language, semantics, preparation, evaluation, solving and workflow have separate owners (§5) |
| AP-02 Stable contracts | Interfaces are the only contract knowledge consumers see. Realization policies and accelerators replace implementations behind an unchanged declaration. `InnerSolver` and external functions are capability contracts |
| AP-03 Composition | Children, dispatch, slots, presets, accumulators, annotations and `when` variants compose orthogonally. No knowledge addition edits kernel internals (knowledge boundary test) |
| AP-04 One authority | A scientific fact is declared once: a function, a table row, an interface default, a definition. Derived forms (∂ expansions, discretized rows, bodies) are keyed derivations |
| AP-05 Explicit structure | Requirements, capability queries, typed slots, declared realization and validity policies, and lineage make every decision inspectable |
| AP-06 Local reasoning and testing | Any definition or function can be tested alone through a conformance case. Kernel mechanisms are tested with synthetic knowledge in `pse-modeling`, with no runtime |
| DP-02 / PS-01 | Every symbol, table column and function signature carries a complete quantity type; conversion only through declared rules |
| DP-06 | Structure authored once, instantiated through sets, children and dispatch |
| DP-08 | Discretization, realization, elastic relaxation and continuation are transformations with declared contracts and lineage |
| DP-09 / PS-11 | Salsa-tracked specialization; value-only edits reuse structure; modes are facts, not models |
| DP-13 | Symbolic algebra, derivatives, roots (Symbolica), linear algebra (faer), iteration (native solvers); the kernel writes no Newton step, AD or factorization |
| DP-15 | Realization accelerators and external functions declare capabilities and are selected by policy, never by name |
| DP-16 | An extension is one package declaration; generated code derives from the registry IR |
| DP-21 | Lineage from every result to package, definition, member, dispatch, default and transformation |
| PS-02 | `@valid` envelopes on every function and definition; recorded extrapolation policy |
| PS-03 | Accumulators with independent closure |
| PS-04 / PS-05 | Structural analysis over the specialized case; topology from ports, incidence from equations, solve order from structure, all distinct |
| PS-06 | Smoothing and complementarity as data functions with typed ε; realization policies declared and reported |
| PS-07 | Derivative source per link: Symbolica exact; implicit-function derivatives from converged nested blocks; external functions declared |
| PS-08 | Starts and stages as data; transactional engine |
| PS-10 | Declared `@check`s plus closure plus original-space qualification |
| PS-13 | Shared checks for every definition come from the kernel; reference tests are data |

## 8. Alternatives considered

| Alternative | Why not |
|---|---|
| Code-level scientific concepts (Plan 20 companion documents: `pse-properties` identity rules, `pse-thermo` closures, property-package and costing-binding types) | Each new family of knowledge would still need a Rust concept and contract, which is the amplification this kernel removes |
| Relational rows as the authoring surface (current YAML template families) | Correct as a durable IR, but 25+ row families per template make porting ~100 units and ~70 methods error-prone. The language is a surface over the same IR (D1 kept) |
| Implementation inheritance | Competing authorities between base and derived bodies; presets, slots and composition cover every IDAES subclass relation found |
| Adopt Modelica (or a Modelica compiler) as the language | Modelica lacks complete quantity types with basis and reference, demand-driven members, accumulators with independent closure, interface-typed dispatch per set member, and the library-owned math and solver contracts here. It would be a second model language (§3.3) |
| Generic "rule engine" or fixed point for property resolution | Resolution is acyclic demand closure over declared members; a fixed point would hide cycles (DP-12) |

## 9. Risks

| Risk | Control |
|---|---|
| Language design errors are expensive to change once many packages exist | Plan 21 fixes the language with a minimal seed before any bulk port. The grammar is versioned (DP-24) |
| Lazy instantiation hides structure from authors | The inspection surface lists instantiated members and why (the demand chain) |
| Expression size from ∂ over large potentials (Plan 20 R1) | Measured in Plan 21 with PC-SAFT in the seed; `nested` realization and `let`-sharing as fallbacks |
| Generic nested realization is slower than hand algorithms | Accelerators are allowed behind the same declaration and validated against `nested` |
| Kernel gaps discovered during bulk porting | The knowledge boundary test makes them explicit. Each gap is generalized in the kernel with a synthetic test |
