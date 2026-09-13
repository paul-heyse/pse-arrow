# Proposed architecture: an Arrow-native mathematical compiler

I would build this as a **Rust-native mathematical modeling platform whose authoritative representation is a versioned collection of typed Arrow relations, with DataFusion providing relational planning, model assembly, validation, and analytical execution.**

The crucial change would be this:

> **The model would no longer be a collection of objects that construct equations. The model would be a structured specification of domains, quantities, equations, dependencies, and numerical policies—from which object graphs and executable programs are generated.**

Pyomo would become one generated execution backend. It would no longer determine the platform’s data structures, extension mechanisms, or model-authoring conventions.

Your requested version pairing is coherent: DataFusion 55.0.0’s workspace uses Arrow **59.2.0**, and declares Rust **1.94.0** as its minimum supported version. I would pin the associated Arrow, Parquet, and DataFusion crates together rather than independently selecting their versions. 

There is one important boundary to preserve: **Arrow should unify the representation, but not dictate every numerical algorithm’s internal memory layout.** Sparse factorizations, expression evaluation, and graph algorithms should operate on efficient native structures derived from the canonical relations. Those structures must be reproducible execution artifacts—not competing sources of model truth.

The architecture below is a proposed platform design. Names such as `CanonicalMathGraph`, `KernelSpec`, and `CompilationPlan` describe abstractions to implement, not existing Arrow or DataFusion classes.

---

## 1. One authoritative substrate, three levels of mathematical representation

I would organize the platform around three related representations.

```text
Declarative model packages + engineering datasets + problem specification
                                  │
                                  ▼
                         CanonicalModel
              Physical entities, domains, model templates,
              property capabilities, connections, specifications
                                  │
                       Semantic compilation
                                  ▼
                      CanonicalMathGraph
              Typed, indexed symbolic equations and operators;
              explicit dependencies, units, assumptions, provenance
                                  │
                  Instantiation and numerical lowering
                                  ▼
                     CanonicalMathProblem
              Ordered unknowns, equations, objectives, derivative
              structure, initialization plans, scaling, backend bindings
                         │                   │
                         ▼                   ▼
                 Native Rust kernels    Generated Pyomo model
                         │                   │
                         └─────────┬─────────┘
                                   ▼
                    Results and diagnostic relations
```

**All three representations would use the same schema system, identity conventions, artifact model, and provenance mechanisms.** They are different compilation stages, not separate application databases.

### `CanonicalModel`: engineering intent

This contains statements such as:

- A reactor uses a specified material system and reaction model.
- A conservation law applies over a spatial domain.
- Two ports are connected through an explicitly selected connection rule.
- A quantity is specified, estimated, optimized, observed, or left to be solved.
- A numerical policy calls for a particular discretization or initialization strategy.

It should remain relatively compact. A distributed reactor should not require the author to enumerate every mesh-point variable.

### `CanonicalMathGraph`: mathematical meaning

This represents the selected physics as typed, indexed symbolic expressions.

For example:

\[
\frac{\partial n_j}{\partial t}
+
\frac{\partial F_j}{\partial x}
=
\sum_r \nu_{r,j} R_r.
\]

The graph records the meanings of \(n_j\), \(F_j\), the indexing domains, derivative operators, stoichiometry, and constitutive dependencies. It does not yet need to expand every expression into scalar solver objects.

### `CanonicalMathProblem`: an executable mathematical problem

This is the fully specified problem delivered to a backend: variable ordering, equation ordering, bounds, parameter bindings, sparse derivative structure, numerical transformations, and backend capabilities.

**The primary architectural advantage is late commitment.** Physical meaning survives independently of mesh resolution, solver choice, Pyomo class structure, or current numerical values.

---

## 2. Make the semantic schema—not merely the Arrow schema—the foundation

Arrow provides an excellent physical representation, but an Arrow schema alone does not express everything required for an engineering modeling system.

I would define a **semantic schema registry** that generates:

```text
Semantic relation definitions
        ├── Arrow schemas and field metadata
        ├── Typed Rust column views and builders
        ├── Serialization and import adapters
        ├── Referential and scientific invariant checks
        ├── Authoring-language schemas
        ├── Documentation and inspection interfaces
        └── Version migrations and compatibility tests
```

This is where much of the boilerplate disappears: a relation’s structure is declared once, rather than separately maintained in Rust structs, Python classes, Arrow schemas, validation code, and documentation.

`serde_arrow` is useful at this boundary and supports Arrow 59. I would use explicit schemas for production interchange, while reserving generated column builders for performance-sensitive paths. Schema inference from representative records would not be the production contract. 

### 2.1 The canonical relation families

I would start with the following model.

| Relation family | Representative relations | Meaning |
|---|---|---|
| **Identity and versions** | `model_versions`, `package_versions`, `entities`, `source_anchors` | Stable identities, ownership, names, package versions, source locations. |
| **Physical types** | `quantity_types`, `units`, `reference_states`, `conversion_rules` | Dimensions, quantity kinds, measurement conventions, material bases. |
| **Domains and coordinates** | `domains`, typed membership relations, `meshes`, `cells`, `faces`, `quadrature_rules` | Valid indexing combinations and spatial/temporal structure. |
| **Material systems** | `species`, `phases`, `phase_species`, `stoichiometry`, `material_systems` | Chemical and phase structure, admissible combinations, reactions. |
| **Model composition** | `templates`, `instances`, `feature_bindings`, `capability_requirements` | Declarative unit models, property formulations, composition, structural options. |
| **Symbols** | `symbols`, `symbol_instances`, `aliases` | Variables, parameters, derived quantities, references, indexing. |
| **Mathematics** | `expr_nodes`, `expr_args`, typed operator-payload relations | Symbolic operations and their dependencies. |
| **Problem statements** | `constraints`, `objectives`, `implicit_systems`, `events`, `complementarity_pairs` | Mathematical conditions to satisfy or optimize. |
| **Connectivity** | `ports`, `port_members`, `connections`, `connection_rules` | Physical interfaces and the rules that generate connection equations. |
| **Cases and observations** | `parameter_bindings`, `variable_specs`, `observations`, `measurement_models` | Operating conditions, initial guesses, estimation data, uncertainty. |
| **Numerical infrastructure** | `kernel_specs`, `pass_specs`, `initialization_plans`, `scaling_plans` | Registered computation and numerical strategies. |
| **Derived structure** | `incidence`, `partitions`, `sparse_patterns`, `backend_bindings` | Compilation products used by numerical backends. |
| **Execution and evidence** | `runs`, `solutions`, `residuals`, `diagnostics`, `derivations` | Results, statuses, explanations, and lineage. |

These should be **typed relations**, not a generic entity–attribute–JSON store.

For instance, stoichiometry deserves an explicit relation with reaction, phase, species, and coefficient columns. Hiding it inside a general-purpose metadata map would sacrifice exactly the structural advantages you are pursuing.

### 2.2 Separate identity from location and numerical state

I would use three distinct forms of identity:

**Stable semantic IDs**, such as 128-bit identifiers, identify authored entities and declarations across revisions.

**Artifact-local integer IDs** provide efficient indexing inside a compiled expression graph or solver vector.

**Content hashes** identify immutable versions of relations, packages, and compiled artifacts.

Names, Arrow row positions, dictionary codes, and memory addresses must never serve as semantic identity.

For generated quantities, identity should derive from the parent instance, template declaration, and domain-member identities—not from current numerical values.

### 2.3 Separate structure, specifications, and results

Three independently versioned concepts are necessary:

```text
Model revision:   What equations and physical entities exist?
Case revision:    What values, bounds, fixed statuses, and objectives apply?
Run:              What happened when a particular backend executed that case?
```

This separation permits a compiled model to support many operating cases without mixing results back into its definition.

It also avoids an important cache error: a change in numerical specifications might leave the equation topology unchanged while invalidating constant folding, initialization assumptions, or a solver-specific compiled representation.

---

## 3. Use Arrow’s richer structures deliberately

The substrate should exploit Arrow’s capabilities without mechanically using every available datatype.

| Arrow capability | Recommended architectural use |
|---|---|
| Primitive numeric arrays | Coefficients, coordinate values, operating data, residuals, solution vectors. |
| `Struct` | Small, well-defined composite records such as tagged bounds or measurement specifications. |
| `List` / `FixedSizeList` | Variable-length connectivity and fixed-size vector quantities where their semantics are uniform. |
| Dictionary encoding | Repeated categorical labels and enum presentation—not persistent identity. |
| Field and schema metadata | Schema versions, semantic type references, quantity annotations, interpretation contracts. |
| Extension types | Domain-specific logical types with explicit Arrow storage representations. |
| IPC and C Data/C Stream interfaces | Interchange between Rust components and the Python backend. |
| Parquet | Durable, partitioned relations and large result datasets. |

Arrow 59.2 includes Rust support for extension types, while the Arrow format specifies how extension names and metadata accompany their storage types. These mechanisms support the representation of domain-specific semantics; they do not, by themselves, implement those semantics in a query engine. 

### 3.1 Physical meaning requires more than a unit string

I would define a `QuantityType` containing:

```text
Numeric representation
Dimension exponents
Quantity kind
Material basis
Index/shape definition
Reference-state convention
Affine or coordinate semantics, where applicable
```

This allows the compiler to distinguish:

- Absolute temperature from temperature difference.
- Absolute pressure from gauge pressure.
- Molar enthalpy from mass-specific enthalpy.
- Enthalpies using incompatible reference states.
- Species composition from phase composition.
- A vector indexed by species from an equally sized vector indexed by spatial cells.

A mass-to-molar conversion is not a numeric cast. It requires a molecular-mass relationship. A gauge-pressure conversion requires a reference pressure.

Those dependencies should become explicit mathematical operations.

### 3.2 Avoid the heterogeneous-value-column trap

A relation like:

```text
variable_id | value
```

can legitimately contain temperatures, pressures, and flow rates. But its `value` field cannot truthfully carry one common unit annotation.

I would therefore require either a per-row `quantity_type_id` or type-specialized derived relations. Engineering-facing aggregate queries would use these typed views.

Otherwise, ordinary SQL could accidentally compute the sum of a pressure, a temperature, and a flow rate.

### 3.3 Null is not an optimization variable

An unknown in a mathematical problem must be represented by a **symbol**, not a null floating-point value.

Null can mean “no initial guess was supplied” or “this measurement is missing.” It must not mean “the nonlinear solver should determine this value.”

Similarly, an unbounded variable should use an explicit bound representation, not a NaN sentinel.

These distinctions make validation and cross-language behavior substantially more reliable.

---

## 4. Build a mathematical IR that is richer than DataFusion expressions

I would not make DataFusion’s `Expr` the canonical mathematical representation.

DataFusion provides logical and physical expression systems for query execution, and exposes extensibility through its planner and execution interfaces. Those are valuable implementation mechanisms, but a process-modeling IR also needs free unknowns, indexed quantification, differential operators, implicit systems, and backend-specific mathematical constraints. 

### 4.1 A normalized expression representation

A simplified portion of the proposed schema could look like this:

```text
expr_nodes
    node_id:          UInt64
    opcode:           UInt16
    quantity_type_id: UInt32
    scope_id:         UInt64

expr_args
    parent_node_id:   UInt64
    argument_ordinal: UInt32
    child_node_id:    UInt64

symbol_refs
    node_id:          UInt64
    symbol_id:        UInt64

float_constants
    node_id:          UInt64
    value:            Float64

kernel_calls
    node_id:          UInt64
    kernel_binding_id: UInt64

constraints
    equation_id:     UInt64
    owner_id:        UInt64
    body_node_id:    UInt64
    lower_bound_id:  UInt64
    upper_bound_id:  UInt64
```

This is illustrative schema notation. Integer IDs here are artifact-local; stable source identities are maintained separately.

Operator-specific payloads belong in typed relations. An expression node should not contain twenty nullable fields, most of which are irrelevant to its operator.

The authoritative argument representation would be `expr_args`. Packed adjacency arrays can be generated for execution, rather than independently maintained as a second graph definition.

### 4.2 Preserve indexed mathematics until expansion is necessary

The IR should directly represent operations such as:

```text
SumOver(domain, expression)
Gather(symbol, coordinate_mapping)
Broadcast(expression, domain)
Derivative(expression, continuous_domain)
Integral(expression, domain, quadrature_policy)
ImplicitSystem(equations, unknowns)
KernelCall(binding, arguments)
```

This matters enormously for distributed and multicomponent models.

A model containing a species balance over 1,000 cells should initially retain one indexed balance declaration—not immediately become thousands of manually assembled expression trees.

Scalarization becomes a compiler decision dictated by the selected backend.

### 4.3 Make every operator carry its mathematical contract

An operator definition should declare:

| Contract | Purpose |
|---|---|
| Input/output physical types | Reject dimensionally or semantically invalid expressions. |
| Index and shape rules | Validate reductions, broadcasting, and coordinate mappings. |
| Evaluation semantics | Define the numerical operation. |
| Derivative rules | Generate Jacobian, directional derivative, and adjoint computations. |
| Domain and smoothness information | Detect invalid inputs and select compatible numerical methods. |
| Sparsity/dependency rules | Generate structural analysis and efficient derivative layouts. |
| Rewrite conditions | Permit only justified simplifications. |
| Backend lowerings | Define how the operation becomes native code or Pyomo expressions. |

This creates a single extensibility point from which multiple capabilities follow.

For example, registering a new constitutive relation should not require unrelated handwritten implementations for evaluation, dependency reporting, unit inference, DataFusion wrapping, and Pyomo adaptation.

However, an opaque native function still needs a trustworthy derivative implementation or an explicitly supported differentiation route. A schema cannot invent the derivative of arbitrary foreign code.

### 4.4 Distinguish expression acyclicity from equation coupling

The expression DAG should be acyclic.

The mathematical problem can still contain tightly coupled equations, recycles, and implicit thermodynamics. Those cycles appear in the **variable–equation incidence structure**, not as cycles in expression ownership.

Keeping that distinction explicit simplifies serialization, evaluation ordering, and structural diagnostics.

---

## 5. Replace model-building methods with declarative templates

The central authoring abstraction should be a **model template**, not a class with a `build()` method.

A template would specify:

```text
Required parameters and capabilities
Declared quantities and indexing domains
Ports and interface contracts
Equation templates
Feature guards
Validity assumptions
Numerical hints
```

A heater, a reactor, a property model, and a costing correlation would all use this same mechanism.

### 5.1 Conservation laws become reusable operators

Rather than separate procedural builders for many balance variants, define a balance structurally:

\[
\frac{d}{dt}
\left(
\text{inventory of conserved quantity}
\right)
=
\text{net boundary flux}
+
\text{internal generation}.
\]

Its specification identifies the conserved quantity, domain, inventory expression, flux laws, source terms, and boundary conditions.

For a lumped unit, the domain has one control region.

For a distributed unit, the same abstract law is combined with cells, faces, geometry, and a discretization policy.

Material, energy, elemental, and charge balances become different bindings of the same underlying abstraction.

### 5.2 Property selection becomes explicit dependency resolution

A unit’s equations might require enthalpy flow, density, and viscosity.

The compiler would resolve those requirements against registered property capabilities, select the configured providers, and expand the required expression or kernel dependencies.

This replaces lazy attribute construction with an explicit sequence:

```text
Required engineering quantity
        ↓
Selected provider and version
        ↓
Required state formulation and parameters
        ↓
Dependency closure
        ↓
Generated equations and kernel bindings
```

Inspecting the model would never construct additional physics.

The selected provider must be persisted. A future package installation must not silently change which equation of state or correlation an existing model uses.

### 5.3 Connections must declare their physical rule

A connection is not merely a join between similarly named variables.

The platform should distinguish equality connections, conservation junctions, mixing relations, heat-transfer interfaces, and signal connections.

A mixer, for example, requires conservation and appropriate outlet-state relationships. It cannot be implemented by equating every inlet temperature.

Connection templates should therefore generate explicit equations with documented orientation and sign conventions.

### 5.4 Example: one heater template, different problem specifications

A simplified heater template could generate:

\[
F_{\mathrm{out},j}=F_{\mathrm{in},j},
\]

\[
\sum_p \dot H_{\mathrm{out},p}
-
\sum_p \dot H_{\mathrm{in},p}
-Q=0,
\]

\[
P_{\mathrm{out}}=P_{\mathrm{in}}.
\]

One case fixes \(Q\) and solves for outlet temperature. Another fixes outlet temperature and solves for \(Q\).

No new equipment script is needed.

A proposed authoring syntax might look like:

```yaml
model:
  material_system: benzene_toluene@1

  instances:
    heater:
      template: heater.lumped@1
      material_system: benzene_toluene@1
      features:
        dynamic: false
        pressure_change: false

case:
  specifications:
    heater.inlet.temperature: {fixed: "320 K"}
    heater.inlet.pressure: {fixed: "2 bar"}
    heater.outlet.temperature: {fixed: "370 K"}

  numerical_policy:
    initialization: structural_blocks_then_continuation@1
    scaling: physical_nominals@1
```

This is schematic, not a complete executable model. Flow and composition specifications would also be required.

The important point is that the authoring document compiles into canonical relations. It is not interpreted as a sequence of imperative operations against Pyomo objects.

---

## 6. Give DataFusion a substantial—but precise—role

DataFusion should be deeply integrated, not relegated to reporting after the model has been built elsewhere.

I would use it for four major responsibilities.

### 6.1 Relational model assembly

DataFusion joins and projections would assemble:

- Valid phase–species combinations.
- Template instances over domains.
- Property coefficient bindings.
- Reaction source terms.
- Connection mappings.
- Scenario parameter overlays.
- Measurement-to-symbol bindings.

For example, reaction generation can be assembled by joining stoichiometry with local reaction instances and then creating the corresponding indexed symbolic reduction.

That is different from numerically summing current reaction-rate values. The assembly stage creates mathematics; the evaluation stage computes values.

### 6.2 A snapshot-aware model catalog

Implement `CatalogProvider` and `TableProvider` interfaces over immutable model snapshots and derived artifacts.

A table scan would be pinned to a specific model or case revision. Projection and filter pushdown could reduce the data read for a requested compilation or analytical query.

DataFusion’s `TableProvider` exposes schema, scan planning, and pushdown contracts, including distinctions between exact and inexact filter handling. Those contracts must be implemented truthfully. 

A backend must not silently prune equations from a coupled mathematical problem merely because an analytical query requests a subset of outputs.

### 6.3 Domain-aware logical and physical operators

I would implement deterministic operators such as:

```text
ExpandIndexDomain
ResolvePropertyRequirements
GenerateStencil
AssembleIncidence
EvaluateConstitutiveKernel
ProjectSolution
```

DataFusion provides extension mechanisms for custom planning and execution operators. These make it possible to retain relational integration while invoking specialized Rust algorithms where ordinary SQL is unsuitable. 

An important implementation detail: **each DataFusion execution plan still produces a declared output schema.** A compiler pass that produces symbols, equations, and provenance would publish a typed artifact bundle containing several relations, rather than pretending that one output stream can contain arbitrary heterogeneous tables.

### 6.4 Semantically informed UDFs

DataFusion 55’s `ScalarUDFImpl` exposes useful hooks beyond evaluation, including `return_field_from_args`, simplification, interval evaluation, constraint propagation, and output-ordering information. I would generate appropriate wrappers from the kernel and quantity-type registries. 

For example, an enthalpy function could expose its output field semantics and supported validity information.

These hooks are useful integration points, **not an existing thermodynamic type checker, automatic differentiator, or certified nonlinear feasibility engine**. Those domain implementations remain part of the proposed platform.

### What I would explicitly keep out of ordinary UDFs

I would not expose an effectful nonlinear solver as an ordinary scalar function such as:

```sql
SELECT solve_process(model_id);
```

Instead, a typed execution plan would launch the solve, persist its run artifacts, and expose completed results as queryable relations.

This gives solver execution explicit lifecycle, cancellation, retry, provenance, and resource semantics rather than allowing those concerns to become hidden inside query evaluation.

---

## 7. Make compilation, initialization, scaling, and discretization first-class plans

The replacement for ad hoc workflow scripts should be a **versioned compiler and execution-plan system**.

### 7.1 Each compiler pass has a declared contract

A `PassSpec` would identify:

```text
Pass implementation and version
Required input relation schemas
Produced output relation schemas
Preconditions and postconditions
Dependency footprint
Determinism and numerical assumptions
Cache-key definition
Diagnostic and provenance outputs
```

The compiler derives a dependency DAG from these contracts.

A typical compilation sequence would be:

```text
Validate semantic schema
    → resolve packages and physical types
    → instantiate templates and valid domains
    → resolve property dependencies
    → generate connection equations
    → lower differential and spatial operators
    → perform domain-safe simplification
    → analyze incidence and equation structure
    → generate derivative and sparse-layout artifacts
    → generate scaling and initialization plans
    → lower to a selected backend
```

Individual stages can contain DataFusion plans, native graph algorithms, or symbolic transformations. Their inputs and outputs still belong to the same substrate.

### 7.2 Initialization becomes a generated plan over immutable overlays

An `InitializationPlan` would contain stages identifying equation subsets, temporary specifications, tear variables, continuation schedules, tolerances, and failure policies.

A stage could:

1. Solve a selected property subsystem.
2. Propagate its values into a downstream subsystem.
3. Solve strongly connected equation blocks.
4. Apply a continuation schedule to difficult terms.
5. Solve the complete problem.

Temporary fixings and deactivations would be **case overlays**, not mutations to the authoritative model.

Restoring the original problem means discarding the overlay—not remembering which Python objects were changed and reversing those mutations.

This makes initialization reproducible and inspectable. It does not guarantee that every nonlinear system admits an automatically successful initialization strategy.

### 7.3 Scaling becomes an explicit mathematical transformation

Store scaling as a transformation:

\[
x=x_{\mathrm{ref}}+D_x\hat{x},
\]

\[
\hat r(\hat{x})=D_r\,r(x_{\mathrm{ref}}+D_x\hat{x}),
\]

so that

\[
\hat J=D_rJD_x.
\]

The transformation records the variable and residual scales, offsets, assumptions, and mappings required to return results and sensitivities to physical coordinates.

Scaling is then no longer a collection of scattered annotations. It is a compiled artifact with a defined mathematical effect.

### 7.4 Discretization is another lowering pass

A continuous-domain model would retain derivative, flux, boundary, and geometry semantics until a policy selects finite-volume, finite-difference, or collocation rules.

The resulting mesh, stencil, interpolation, quadrature, and boundary relations become explicit artifacts.

Adaptive refinement produces a new discretization artifact and a recorded state-transfer mapping.

The core physical template remains unchanged.

### 7.5 Incrementality must follow scientific dependencies

I would cache separately:

| Artifact | Typical invalidation condition |
|---|---|
| Template expansion | Structural options, domain membership, package selection. |
| Property dependency closure | Required outputs, state formulation, provider capabilities. |
| Symbolic simplification | Expressions and any assumptions or parameter values used by the rewrite. |
| Sparse structure | Equation dependencies, variable treatment, discretization. |
| Numerical evaluation | Current values and parameter bindings. |
| Solver preparation | Backend settings, ordering, scaling, bounds, fixed-variable treatment. |

A value change should not automatically trigger a full rebuild. But it must invalidate any compiled result that depended on that value being constant or satisfying a particular condition.

---

## 8. Use a unified kernel contract, with native numerical layouts inside execution

The most important supporting abstraction is a **`KernelSpec`**.

It should govern a constitutive function’s participation in the entire system:

| Kernel contract | Contents |
|---|---|
| Identity | Provider, version, artifact digest. |
| Signature | Physical input/output types, shapes, indexing, parameter requirements. |
| Mathematical behavior | Explicit function or implicit system; smoothness and validity regions. |
| Derivatives | Available Jacobians, directional derivatives, adjoints, Hessian information. |
| Execution | Scalar and batch implementations, thread safety, resource requirements. |
| Failure behavior | Domain errors, nonconvergence, invalid states, recoverable conditions. |
| Backend bindings | Native execution, DataFusion wrapper, Pyomo-compatible lowering. |

### 8.1 One implementation contract, several generated adapters

The goal is not three independently maintained enthalpy implementations.

The preferred pattern is:

```text
Constitutive equation or registered Rust kernel
        ├── Scalar evaluation adapter
        ├── Batched Arrow adapter
        ├── Derivative adapter
        ├── DataFusion wrapper
        └── Pyomo/backend binding
```

For kernels written generically over supported numerical types, automatic differentiation can reduce derivative boilerplate. `num-dual` provides scalar and higher-order dual-number machinery, including facilities for implicit differentiation; it is well suited to relatively small constitutive calculations. It is not a reason to construct a huge dense dual vector for every unknown in an industrial flowsheet. 

### 8.2 Treat implicit thermodynamics explicitly

A flash calculation should not automatically be modeled as an ordinary pure scalar function.

It can instead remain an equation subsystem:

\[
G(z,u)=0,
\]

or be encapsulated as an implicit kernel with a declared branch-selection and convergence contract.

When \(G_z\) is nonsingular on the selected smooth branch,

\[
\frac{dz}{du}
=
-G_z^{-1}G_u.
\]

That derivative relationship can be part of the kernel’s implementation strategy. Phase transitions, root changes, and singular states need explicit handling rather than being hidden behind a nominally differentiable API.

### 8.3 Do not run relational queries inside every Newton iteration

The native numerical backend should compile the mathematical graph into an evaluation program operating on contiguous numerical arrays.

It would use:

```text
Ordered variable vector
Bound parameter arrays
Compiled evaluation instructions
Precomputed sparse derivative pattern
Reusable numerical workspaces
```

DataFusion is valuable for assembling these inputs, evaluating independent batches, and analyzing results.

It should not repeatedly rediscover the expression graph through joins during each residual evaluation.

For sparse linear algebra, `faer` provides native sparse structures, including separate symbolic and numerical compressed-column representations. Those are appropriate derived execution layouts for Jacobians and related calculations. 

Arrow buffers can be borrowed when layouts and ownership permit. Reordering, mutable solver workspaces, or incompatible layouts may require deliberate copies. “Arrow-native” should not become an obligation to preserve zero-copy at the expense of correctness or numerical performance.

### 8.4 Parallelize where the mathematics permits it

I would batch independent states, scenarios, coefficient evaluations, and compatible constitutive calls.

A tightly coupled nonlinear system does not become embarrassingly parallel because its inputs are columnar.

The runtime should also coordinate DataFusion, native worker pools, and solver-library threading to avoid oversubscription.

---

## 9. Reduce Python to one generic, coarse-grained Pyomo adapter

The Python boundary should be **model-independent**.

A single adapter would consume a `CanonicalMathProblem`, construct the necessary Pyomo objects, invoke the selected backend, and return Arrow result relations.

### 9.1 Transfer model bundles, not individual operations

Use PyO3 and the Arrow Python bridge to exchange batches or streams.

`arrow-pyarrow` 59.2 supports mappings for schemas, arrays, record batches, and readers through the Arrow interoperability interfaces. The PyCapsule interface standardizes the relevant C Data and C Stream exchange conventions. 

The bridge should transfer something like:

```text
Problem manifest
Symbol and parameter relations
Expression graph relations
Constraint and objective relations
Numerical specifications
Kernel bindings
Source maps
```

It should not call Python once per coefficient or Rust once per expression node.

For isolated worker processes, use an IPC-based transport. An in-process C interface does not make raw pointers valid across process boundaries.

### 9.2 Generate a solver model, not an IDAES object hierarchy

The adapter would:

**Construct symbols.** Map canonical IDs to Pyomo variables and parameters.

**Lower expressions.** Traverse the already ordered expression graph and construct symbolic expressions.

**Attach equations and objectives.** Use the canonical problem’s definitions, with efficient treatment of affine and other recognized structures.

**Apply the case and numerical plan.** Bind parameters, fix variables, set bounds and starting values, and configure the solve.

**Return results.** Map primal values, supported duals, residuals, termination conditions, and diagnostics back to canonical IDs.

There is no reason to recreate the full IDAES class hierarchy merely to produce an equivalent mathematical problem. Semantic ownership and engineering names remain in the substrate and source maps.

### 9.3 Preserve symbolic meaning across the boundary

The adapter must not evaluate an expression at its initial guess and substitute that number into the Pyomo model.

The symbolic dependence on decision variables must survive lowering.

Pyomo has its own expression representation, so transferring Arrow data efficiently does not eliminate the cost of constructing those symbolic objects. 

### 9.4 External Rust functions need actual solver integration

For expressions lowered entirely into Pyomo algebra, the solver interface can work with the generated expression structure.

For opaque Rust property kernels, the binding is more demanding. Pyomo’s external-function documentation explicitly distinguishes Python callbacks from compiled AMPL-compatible external libraries; expressing a function in Pyomo does not imply every solver interface can execute it. ASL-based nonlinear solvers require the corresponding compiled external-function route. 

I would therefore make external-function ABI compatibility, derivative availability, and backend support mandatory parts of `KernelSpec`, with end-to-end tests.

A generic Rust `extern "C"` function alone is not a complete solution.

---

## 10. Make persistence and reproducibility part of the same architecture

I would use immutable Arrow/Parquet artifacts plus a **versioned snapshot manifest**.

A manifest identifies the exact relation artifacts, schema versions, package versions, and compilation dependencies that define a model or problem.

### 10.1 Publish complete snapshots, not unrelated table updates

A write would proceed as follows:

```text
Construct proposed changes
    → validate all relation and scientific invariants
    → write immutable relation artifacts
    → write the snapshot manifest
    → conditionally update the branch/head reference
```

Readers remain pinned to their existing manifest.

`object_store` supplies useful primitives including atomic individual-object writes and conditional operations. The multi-relation snapshot protocol would be implemented by this platform; it is not automatically provided by Arrow, Parquet, or DataFusion. 

For an initial local implementation, a single writer with explicit snapshot publication is a reasonable simplification.

### 10.2 Record everything necessary to explain a run

A run should reference:

```text
Model and case revisions
Material/property package versions
Compiler and pass versions
Kernel artifact hashes
Discretization and scaling artifacts
Initialization plan
Backend and solver versions
Solver settings
Execution environment information
Result and diagnostic artifacts
```

This supports questions such as:

“Which property correlation produced the enthalpy term in this failed equation?”

“Which assumption allowed this variable to be eliminated?”

“What changed between these two cases?”

The answers become relational queries over provenance—not archaeology through execution logs.

### 10.3 Hash semantic content, not incidental batch layout

Content addressing requires canonicalization rules for row ordering, dictionary encodings, schema metadata, and floating-point representations.

Two logically identical relations split into different Arrow batches should not accidentally be treated as different scientific models.

Conversely, a change in reference-state convention must change the semantic identity of the relevant artifact, even when the numeric storage type remains `Float64`.

---

## 11. Supporting Rust libraries: a focused selection

I would keep the supporting stack deliberately focused. Each library should provide an algorithm or integration mechanism without becoming another authoritative representation.

| Library | Proposed role | Boundary to maintain |
|---|---|---|
| **`serde` + `serde_arrow`** | Schema-governed import/export and authoring adapters. Arrow 59 support is available. | Explicit schemas remain authoritative; no production inference from sample data.  |
| **`petgraph`** | Temporary graphs for topology, dependency ordering, and structural algorithms. | Persist graph facts in Arrow; graph objects are derived views. Additional matching/decomposition algorithms may require implementation.  |
| **`salsa`** | Incremental compilation and dependency-aware memoization. | Cache compiler computations, not the authoritative engineering model.  |
| **`egglog`** | Optional rule-based inference and equality-saturation optimization. | Only admit rewrites justified by physical types, domains, and numerical policy.  |
| **`num-dual`** | Derivatives for constitutive and implicit kernels. | Use appropriately sized derivative calculations; preserve sparse global structure.  |
| **`faer`** | Dense/sparse numerical linear algebra, diagnostics, and factorization workspaces. | Native matrices are compiled numerical artifacts, not another model definition.  |
| **`uom`** | Compile-time dimensional checking inside selected Rust kernels. | The dynamic quantity registry remains necessary for arbitrary user-defined models.  |
| **FeOs / `feos-core`** | Selected native thermodynamic model implementations. | Treat as capability providers; validate coverage and conventions rather than assuming complete IDAES replacement.  |
| **`diffsol` / DiffSL** | Optional native trajectory backend for supported differential-equation formulations. | Lower the canonical IR into it; do not introduce a second independently authored model language.  |
| **`blake3` + `object_store`** | Content addressing, artifact storage, snapshot publication. | Hashing and object storage do not themselves supply semantic equivalence or database transactions.  |

Two qualifications are especially important.

First, symbolic rewrites need conditions. For example, \(x/x\rightarrow1\) requires a nonzero-domain assumption. Floating-point reassociation also needs an explicit numerical policy; mathematical equivalence does not always mean identical floating-point behavior.

Second, the platform should not add a second dataframe engine, graph database, or vector database merely for completeness. Those additions would need a demonstrated capability gap. They are not prerequisites for the substrate described here.

---

## 12. What this replaces—and what would demonstrate that it works

The proposal is a wholesale alternative at the modeling layer:

| Existing architectural pattern | Proposed replacement |
|---|---|
| Classes constructing mutable model objects | Declarative templates compiled into immutable relations. |
| Construction-time Python configuration | Typed structural specifications with explicit dependency effects. |
| State-block attribute access triggering construction | Explicit property-demand resolution and dependency closure. |
| Separate mathematical and metadata representations | Typed expression IR linked directly to physical semantics and provenance. |
| Model-specific initialization scripts | Generated initialization plans with immutable specification overlays. |
| Scattered scaling annotations | Explicit, invertible scaling transformations. |
| Backend-specific model construction | Generic lowering from a backend-neutral mathematical problem. |
| Saved values requiring the original construction code | Complete model packages, specifications, and derived artifacts sufficient to rebuild the problem. |
| Agent-authored imperative edits | Validated structural change sets against the canonical schema. |

### A particularly valuable consequence for agent support

An agent could propose a structured change such as:

> Replace the selected heat-transfer correlation, verify its required properties, regenerate affected equations, compare sparsity and scaling, and report the resulting model differences.

That request could be resolved into a typed change set and compilation plan.

The agent would not need to locate and edit a collection of `build()`, initialization, scaling, and reporting methods. Nor would the platform need to trust arbitrary generated scripts as its normal extension mechanism.

### Build it through complete vertical slices

I would validate the architecture with three progressively demanding examples.

**A steady-state heater/mixer system** should demonstrate declarative templates, physical typing, connection generation, Arrow persistence, generic Pyomo lowering, and reproducible reload.

**A recycle system with an implicit flash calculation** should demonstrate dependency closure, structural analysis, initialization plans, derivative correctness, and explicit phase/validity behavior.

**A distributed dynamic reactor with costing or parameter estimation** should demonstrate continuous-domain lowering, reaction structure, sparse derivatives, case reuse, and the ability to combine physical and economic equations without another modeling framework.

The principal acceptance tests should include:

- Agreement between native and Pyomo residuals and derivatives at valid test points.
- Conservation, unit, basis, reference-state, and scaling consistency.
- Stable semantic identity despite changed batch boundaries or row ordering.
- Correct incremental invalidation when specifications or assumptions change.
- Complete provenance for generated equations and failed computations.
- Explicit failure when a selected backend cannot support an operator or derivative requirement.

I would benchmark construction, compilation, Python materialization, residual evaluation, derivative evaluation, and solver time separately. The strongest early gains are likely to be **less repeated construction work, less boilerplate, better inspection, and more reliable reuse**. Overall solve-time improvement must be measured rather than inferred from replacing Python data structures.

## Bottom line

**The advantageous pivot is not “represent IDAES objects in Arrow.” It is “make mathematical and physical semantics the data model, then compile every executable representation from it.”**

Arrow would provide the common typed representation and interchange. DataFusion would provide relational assembly, planning, validation, batch computation, and analysis. A Rust mathematical compiler would supply equation semantics, transformations, differentiation, and numerical plans. Pyomo would remain a replaceable generated backend.

That combination provides a credible route to the unified design you are seeking: **specialized physics still requires specialized mathematics, but the machinery for declaring, connecting, validating, initializing, scaling, executing, and explaining that mathematics can become largely schema-driven rather than model-specific scripting.**