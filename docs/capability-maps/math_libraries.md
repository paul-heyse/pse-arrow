Yes. With that clarification, I would change the framing materially. You are not trying to minimize dependencies; you are trying to **minimize bespoke compiler machinery while building a robust mathematical compilation stack**.

For the current scope, where models are relatively simple algebraic systems rather than sophisticated thermodynamics, I think there are **four strong foundations around Symbolica and faer**:

1. **Oximo**, primarily for the optimization-model/container and solver-lowering machinery.
2. **Your existing Petgraph/rustworkx graph layer**, explicitly extended to equation-variable structural analysis.
3. **A units/dimensional-analysis library such as `uom`**, if physical dimensions are already part of model correctness.
4. **Actual solver interfaces/backends**, preferably accessed through a standardized lowering layer rather than writing solver-format machinery yourself.

I would *not* currently add another general AD system, expression system, sparse-matrix library, or symbolic rewriting engine. Symbolica + faer already occupy those roles surprisingly well.

## The compilation problem I think you actually need to cover

For a basic process model, the compiler needs to handle something like:

```text
Model facts
    ↓
instantiate indexed variables / parameters / equations
    ↓
resolve model topology and bindings
    ↓
construct mathematical expressions
    ↓
validate dimensions / domains
    ↓
simplify / canonicalize
    ↓
determine variable-equation incidence
    ↓
structural analysis
    ↓
classify LP / QP / NLP / etc.
    ↓
generate residual/objective evaluator
    ↓
generate derivative structures + values
    ↓
construct solver model / callbacks
    ↓
execute
    ↓
map solution back to domain IDs
```

The question should be: **how many of those boxes can mature libraries own?**

With that criterion, I would assign them as follows.

| Compiler responsibility | Library I would want owning it |
|---|---|
| Relational instantiation/index expansion | **DataFusion / Arrow** |
| Incremental semantic compilation | **Salsa** |
| Structural/topological compilation | **Petgraph + rustworkx-core** |
| Mathematical expression IR | **Symbolica** |
| Algebraic simplification/rewrite/substitution | **Symbolica** |
| Exact algebra | **Symbolica** |
| Derivative expressions/evaluators | **Symbolica** |
| Optimized numerical kernels / CSE / JIT | **Symbolica** |
| Sparse Jacobian/Hessian representation | **faer** |
| Matrix ordering / sparse numerical LA | **faer** |
| Optimization variables/constraints/objective/bounds | **Oximo is the strongest candidate** |
| Model-class classification / solver-facing model | **Oximo** |
| LP/MPS/NL interchange | **Oximo I/O** |
| Solver routing/results/options | **Oximo backends or thin direct adapters** |
| Physical dimensional correctness | **`uom`, conditionally** |

That leaves your own code focused primarily on **process semantics and glue between genuinely different representations**, rather than implementing generic mathematics.

---

# 1. Oximo becomes much more attractive under this framing

Previously I was concerned that Oximo duplicates Symbolica's expression representation. That remains true, but I think the right architecture makes that duplication much less problematic.

I would not use:

```text
Symbolica IR
     ↕
Oximo IR
     ↕
Symbolica IR
```

Instead, use Oximo as a **terminal lowering representation**:

```text
                  Symbolica
          canonical mathematical IR
                    │
                    │ one-way lowering
                    ▼
                  Oximo
        solver-model representation
                    │
         ┌──────────┼───────────┐
         ▼          ▼           ▼
       HiGHS      Gurobi      Clarabel ...
```

Oximo currently gives you variables, indexed domains, parameters, constraints, objectives, bounds, nonlinear expressions and multiple solver backends. It also recognizes a range of model classes including LP, QP/QCP, NLP, MILP, MIQP/MIQCP, MINLP and SOCP/MISOCP. :chatgpt-content-reference{index="1"}

That means Oximo could potentially eliminate bespoke implementations of:

- variable registries
- constraint registries
- objective handling
- bounds
- variable domains
- indexed families
- feasibility-only models
- model-class representation
- solver capability checks
- solver result representation
- dual/reduced-cost result handling
- many solver-specific option wrappers
- LP/MPS/NL serialization

Its solver abstraction already standardizes solver capability and result/status handling. :chatgpt-content-reference{index="2"}

### That is a substantial amount of boring compiler code

And it is exactly the kind of code I would rather you **not** own.

The bespoke integration then becomes approximately:

```text
fn lower_symbolica_expression(
    expr: SymbolicaExpression,
    variables: VariableMapping,
) -> OximoExpr
```

plus:

```text
CompiledVariable → Oximo Variable
CompiledConstraint → Oximo Constraint
CompiledObjective → Oximo Objective
```

That's a bounded compiler pass.

---

# 2. I would keep Symbolica authoritative and Oximo disposable

This distinction is important.

Oximo should not become where your mathematical meaning lives.

Suppose you have:

\[
F_1x_1+F_2x_2-F_3x_3=0.
\]

The authoritative object is:

```text
Symbolica expression
+
domain-variable mapping
+
equation metadata
```

You can regenerate an Oximo model at any time.

That means:

```text
                    authoritative
                        ↓
                   Symbolica
                        ↓
              Oximo lowering pass
                        ↓
              ephemeral solver model
```

This is analogous to a compiler having an optimized IR and then emitting LLVM IR or machine code.

The lower representation is important, but it isn't the source of truth.

### One caveat

Oximo is moving rapidly. The current `oximo-solver` 0.7.0 release was published on September 23, 2026, and its crates remain pre-1.0. :chatgpt-content-reference{index="3"}

So I would absolutely test and pin it.

But that argues for keeping it **behind a lowering boundary**, not against using it.

---

# 3. Use Oximo's model I/O rather than implementing solver file formats

This is another piece I would exploit aggressively.

Even if some solvers eventually get direct in-process adapters, supporting standard formats is useful:

```text
LP
MPS
NL
```

Oximo already has corresponding model I/O infrastructure. :chatgpt-content-reference{index="4"}

This gives you:

```text
Symbolica
    ↓
Oximo
    ├── direct solver backend
    ├── NL
    ├── MPS
    └── LP
```

That is useful both operationally and for validation.

For example, you could compile a model through your new Rust stack, export it, and compare it against another solver/toolchain without writing a serializer.

That is precisely the kind of generic compiler responsibility worth outsourcing.

---

# 4. Use your graph stack much more deeply in the equation compiler

This is probably the biggest already-adopted capability that I would now make explicit.

Your graph system shouldn't only understand:

```text
flowsheet:
pump → heater → reactor → separator
```

It should also understand the **compiled mathematical incidence graph**:

```text
variables ↔ equations
```

For example:

```text
e1 ── x1
 │
 ├── x2
 │
 └── x5

e2 ── x2
 │
 └── x3
```

This bipartite graph is hugely useful during compilation.

You can get:

- variable-equation incidence
- disconnected equation systems
- structurally underdetermined systems
- structurally overdetermined systems
- maximum matching
- block decomposition
- dependency ordering
- cyclic equation blocks
- initialization ordering
- Jacobian sparsity

Petgraph already provides maximum matching and its broader algorithm suite; you therefore should not write matching logic yourself. :chatgpt-content-reference{index="5"}

This yields an elegant progression:

```text
Symbolica expression
       │
       │ extract symbols
       ▼
Equation-variable incidence graph
       │
       ├── matching
       ├── connected components
       ├── SCC/block analysis
       └── ordering
       │
       ▼
Sparse derivative structure
```

Your existing `rustworkx-core` dependency gives you additional generic graph algorithms over Petgraph-compatible graph representations. :chatgpt-content-reference{index="6"}

So I would explicitly put:

> **structural equation analysis**

under the graph subsystem rather than inventing a separate process-simulator algorithm layer.

---

# 5. faer can own more of the compiler than merely linear solves

This is another adjustment I would make.

You already identified faer as numerical linear algebra, but its **symbolic sparse matrix representation** is very useful for compilation itself.

Faer separates sparse structure from numerical values:

```text
SymbolicSparseColMat
        +
numeric values
```

and exposes the symbolic structure independently. :chatgpt-content-reference{index="7"}

That maps perfectly onto:

```text
compile time:
Jacobian sparsity structure

runtime:
Jacobian numerical values
```

For example:

```text
compile once

J structure:
[ x x 0 x ]
[ 0 x x 0 ]
[ x 0 x x ]

        ↓

faer::SymbolicSparseColMat
```

Then solver iterations simply populate:

```text
values[nnz]
```

without rediscovering sparsity.

Faer also already provides AMD and COLAMD ordering, so you don't need a separate sparse-ordering package for basic work. :chatgpt-content-reference{index="8"}

I would therefore treat faer as:

> **the sparse structural/numerical matrix substrate**

rather than merely the linear algebra library.

---

# 6. Symbolica should own derivative generation initially

For your current simple-model target, I would **not introduce Echidna, `num-dual`, Enzyme, or another AD framework yet**.

Symbolica evaluators already:

- turn expressions into instruction programs,
- perform Horner optimization,
- perform CSE,
- recycle temporaries,
- support JIT compilation,
- and can generate multiple requested derivative components together. :chatgpt-content-reference{index="9"}

That is enough for your current scope.

Conceptually:

```text
residual expressions
      ↓
Symbolica

requested:
[r1, r2, r3, ...]
[∂r1/∂x1, ∂r1/∂x2, ...]
...

      ↓

optimized combined evaluator
```

Pair this with the incidence graph so you only request **structurally possible nonzero derivatives**.

That could be very powerful:

```text
Graph layer
    ↓
Jacobian sparsity coordinates
    ↓
Symbolica
    ↓
generate only needed derivatives
    ↓
faer sparse values
```

You may later find that a dedicated sparse AD engine such as Echidna is faster. But I would make that a measured runtime optimization, not foundational architecture.

---

# 7. `uom` is worth serious consideration even before advanced physics

I would not dismiss units just because thermodynamics isn't yet in scope.

Simple models already involve:

```text
flow
temperature
pressure
power
energy
cost
time
capacity
```

and dimensional mistakes are compiler errors conceptually, not solver errors.

`uom` provides a mature zero-cost compile-time dimensional system and a large prebuilt SI quantity/unit system. :chatgpt-content-reference{index="10"}

So for handwritten reusable component functions, you could make:

```rust
fn pressure_drop(flow: MassRate, resistance: ...)
```

instead of:

```rust
fn pressure_drop(flow: f64, resistance: f64)
```

That prevents a large class of mistakes.

### But I would distinguish two unit systems

Your runtime model is data-driven, so you still need something like:

```text
VariableMetadata {
    dimension: ...
    canonical_unit: ...
}
```

because Symbolica expressions aren't Rust generic types.

So:

```text
uom
    → handwritten Rust API correctness

your semantic dimension metadata
    → dynamically compiled Symbolica equation correctness
```

Eventually you can build a fairly small bridge between the two.

I would adopt `uom` rather than invent the Rust quantity type system. Its model works at the quantity level and normalizes units to base representations. :chatgpt-content-reference{index="11"}

I would not use `diman` as the default because its documentation still describes reliance on unstable const-generic features and recommends more mature alternatives such as `uom` when that matters. :chatgpt-content-reference{index="12"}

---

# 8. Solver libraries are dependencies, but they should not become compiler abstractions

For basic models I'd probably support at least:

```text
HiGHS
IPOPT
```

eventually.

HiGHS has a safe Rust binding and accepts sparse row/column problem construction. :chatgpt-content-reference{index="13"}

IPOPT's Rust interface explicitly supports sparse Jacobian/Hessian callback structures. :chatgpt-content-reference{index="14"}

But if Oximo covers the model class and solver backend adequately, I would prefer:

```text
your compiler
      ↓
Oximo
      ↓
HiGHS
```

rather than:

```text
your compiler
      ↓
bespoke HiGHS compiler
```

For a solver Oximo cannot represent optimally, add a direct backend:

```text
CompiledProblem
      ↓
IpoptAdapter
```

The adapter should consume exactly the same compiled structure.

---

# 9. The resulting foundation would be broader than the "minimal dependency" version

I would now propose the compilation architecture as:

```text
┌─────────────────────────────────────────────────────────────┐
│                    MODEL DATA FABRIC                        │
│                                                             │
│             Arrow + DataFusion + Delta                      │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                SEMANTIC COMPILATION                         │
│                                                             │
│                       Salsa                                 │
│                                                             │
│  definitions / bindings / types / dimensions / instances   │
└──────────────┬───────────────────────────────┬──────────────┘
               │                               │
               ▼                               ▼
     ┌─────────────────┐             ┌──────────────────────┐
     │ graph subsystem │             │      Symbolica       │
     │                 │             │                      │
     │ Petgraph        │             │ mathematical IR      │
     │ rustworkx-core  │             │ simplify / rewrite   │
     │                 │             │ differentiate        │
     │ topology        │             │ optimize evaluator   │
     │ incidence       │             │ JIT / codegen        │
     │ matching        │             │                      │
     │ blocks/SCC      │             └──────────┬───────────┘
     └────────┬────────┘                        │
              │                                 │
              └──────────────┬──────────────────┘
                             ▼
                 ┌───────────────────────┐
                 │ compiled math problem │
                 │                       │
                 │ variables             │
                 │ bounds                │
                 │ equations             │
                 │ objective             │
                 │ incidence             │
                 │ scaling               │
                 │ derivative structure  │
                 └───────────┬───────────┘
                             │
                    ┌────────┴─────────┐
                    ▼                  ▼
             ┌────────────┐      ┌────────────┐
             │    faer    │      │   Oximo    │
             │            │      │            │
             │ sparse IR  │      │ solver IR  │
             │ ordering   │      │ model class│
             │ matrices   │      │ model I/O  │
             │ solves     │      │ backends   │
             └────────────┘      └─────┬──────┘
                                       │
                           ┌───────────┼───────────┐
                           ▼           ▼           ▼
                        HiGHS       IPOPT*      Clarabel
```

`*` potentially direct if the native Symbolica derivative path is superior to the available Oximo NLP route.

---

## What this leaves bespoke

And this is the important measure.

With those libraries, I think the bespoke compiler shrinks toward:

```text
Domain model → mathematical primitives
Domain ID ↔ mathematical variable mappings
Dimension propagation for dynamic expressions
Symbolica → Oximo lowering
Symbolica symbol incidence extraction
Graph → faer sparsity conversion
Solver result → domain result mapping
Scaling / initialization policies
```

Those are **legitimate domain compiler responsibilities**.

You would *not* be writing:

```text
expression tree implementation
symbolic differentiator
CAS
polynomial engine
rewrite engine
CSE optimizer
numerical expression VM
JIT
sparse matrix library
AMD/COLAMD
graph algorithms
maximum matching
generic optimization model container
LP/MPS/NL writer
solver status abstraction
solver result container
units-of-measurement system
```

That is the architecture I think best matches your clarified goal.

### The three additional dependencies I would investigate first

If we take **Symbolica + faer + your already-established graph/DataFusion/Salsa stack as given**, my immediate priority would now be:

**1. Oximo 0.7.x** — determine exactly how much of model assembly, problem classification, solver lowering, solver I/O, and results it can own without undermining Symbolica as the canonical mathematical IR.

**2. `uom`** — determine whether its quantities can cleanly form the Rust-side dimensional semantics while a lightweight dynamic dimension representation is carried in Arrow/Symbolica metadata.

**3. Solver backends**, starting with HiGHS and IPOPT — not to create another abstraction, but to ensure the `CompiledProblem` produced by the pipeline contains exactly the information mature solver interfaces need.

Of those, **Oximo is the one I would investigate most deeply now**. If its current 0.7 architecture works cleanly as a *lowering target*, it could eliminate a surprisingly large portion of the remaining generic compiler/modeling code while allowing Symbolica to remain the substantially more powerful mathematical representation.

Here is the clean library list I would give the agent. I would distinguish **libraries to review as part of the target architecture** from libraries I mentioned specifically as **not currently necessary**, so there is no ambiguity.

### Core architecture libraries to review

| Library / crate | Role in the process simulator | What the agent should assess |
|---|---|---|
| **Apache Arrow Rust (`arrow`)** | Canonical typed columnar data representation and interchange | Custom/extension types for Symbolica expressions, schema metadata, zero/low-copy integration |
| **Apache DataFusion (`datafusion`)** | Relational compilation and execution | UDF/UDTF support around Symbolica, extension-type handling, relational model preparation |
| **Delta Lake / `delta-rs`** | Durable/versioned model and artifact persistence | Persisting model releases, compiled artifacts, Arrow schemas and Symbolica payloads |
| **Salsa (`salsa`)** | Incremental semantic compilation | Query boundaries for resolved models, instantiated components, equations and compiled artifacts |
| **Petgraph (`petgraph`)** | Primary graph representation and structural algorithms | Equation-variable incidence graphs, SCCs, connectivity, dependency ordering, matching capabilities |
| **`rustworkx-core`** | Extended graph-algorithm suite over Petgraph-compatible structures | Matching, connectivity, DAG algorithms and structural analyses missing from Petgraph |
| **Symbolica (`symbolica`)** | Canonical mathematical expression IR and mathematical compiler | Expressions, exact algebra, rewriting, simplification, differentiation, CSE, evaluator generation, JIT/codegen, programmable symbols, serialization |
| **faer (`faer`)** | Sparse/dense numerical linear algebra and sparse structural representation | `SymbolicSparseColMat`, Jacobian/Hessian storage, AMD/COLAMD ordering, sparse factorizations and solves |
| **Oximo (`oximo`, especially 0.7.x)** | Solver-facing optimization model / lowering target | Variables, bounds, constraints, objectives, indexed structures, model classification, solver abstraction, solver I/O |
| **Oximo expression/core crates (`oximo-expr`, `oximo-core`)** | Parts of Oximo relevant to lowering | Determine how much can be used without allowing Oximo's expression IR to replace Symbolica as the canonical IR |
| **Oximo solver layer (`oximo-solver`)** | Common solver interface and result abstraction | Solver capability classification, options, results, statuses, primal/dual handling |
| **Oximo I/O (`oximo-io`)** | Standard mathematical-model interchange | LP, MPS and NL export/import; eliminate bespoke solver-format writers |
| **`uom`** | Compile-time physical quantities / dimensional correctness in Rust code | Whether it can form the Rust-side dimensional type system and map cleanly to runtime model dimension metadata |
| **HiGHS / `highs`** | LP, convex QP and MILP solver backend | Oximo integration vs direct Rust adapter, sparse model construction, warm starts/results |
| **IPOPT / `ipopt`** | General continuous nonlinear-programming backend | Direct callback integration with Symbolica evaluators, sparse Jacobian/Hessian structures, whether direct integration is preferable to Oximo |
| **Clarabel / `clarabel`** | Convex conic optimization | Keep available as a solver target for LP/QP/SOCP and other conic formulations; likely not immediate core scope |

### Libraries explicitly mentioned but **not recommended as foundational right now**

These should only be reviewed if the agent finds a concrete gap in the stack above.

| Library | Why it is currently secondary |
|---|---|
| **Echidna (`echidna`)** | Significant overlap with Symbolica's numerical evaluator and derivative machinery. Revisit only if Symbolica lacks required sparse-AD performance/features. |
| **`num-dual`** | Valuable later for opaque scientific/physics kernels, but unnecessary as a second global AD framework for simple Symbolica-defined equations. |
| **Enzyme** | Another derivative/code-generation route; currently unnecessary duplication if Symbolica owns expressions and derivatives. |
| **`nalgebra`** | Useful locally and through dependencies, but `faer` should be the primary general numerical linear-algebra substrate. |
| **`sprs`** | Another sparse-matrix ecosystem; avoid duplicating `faer` unless a dependency or algorithm specifically requires it. |
| **`ndarray`** | Fine inside dependencies, but not needed as another architectural array abstraction beside Arrow and faer. |
| **`diman`** | Alternative dimensional-analysis crate; I favored `uom` as the more established foundation. |

### Future-domain libraries that are intentionally **out of the current basic-model scope**

These remain strong candidates later, but the agent does not need to make them foundational for the current compiler.

| Library | Future role |
|---|---|
| **FeOs (`feos`, `feos-core`)** | Thermodynamics, equations of state, phase-equilibrium/property calculations |
| **`quantity`** | Physical quantity system used in the FeOs ecosystem |
| **Diffsol (`diffsol`)** | ODE/DAE integration, stiff dynamics, events and sensitivities |
| **SCIP / `russcip`** | More advanced mixed-integer/combinatorial optimization |
| **`argmin`** | Small local optimization/root-finding/subproblem toolbox |

For the agent, I would give the **primary review set** as:

```text
arrow
datafusion
delta-rs
salsa

petgraph
rustworkx-core

symbolica
faer

oximo
oximo-expr
oximo-core
oximo-solver
oximo-io

uom

highs
ipopt
clarabel
```

And I would give it this design constraint:

> **Treat Symbolica as the canonical mathematical IR. Treat Oximo as a potential one-way solver-model lowering target, not a competing expression system. Treat faer as the sparse structural/numerical matrix substrate. Reuse Petgraph/rustworkx-core for equation-system structural analysis. Investigate the remaining libraries specifically for the generic compiler/runtime responsibilities they can eliminate, rather than merely cataloguing their features.**

That should give the agent a very clear research boundary.