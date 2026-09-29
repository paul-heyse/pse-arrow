---
title: Models, composition and extension
status: current
---

# Models, composition and extension

Reusable models are authored definitions, interfaces, functions, sets, tables and
connections in the generic modeling IR. `pse-modeling` owns checking and specialization;
`pse-compiler` owns incremental preparation and lowering; `pse-math` owns library-derived
arithmetic and derivatives. Runtime admits immutable package closures, coordinates analyses
and retains results. The old template and scientific workflow constructors are removed.

## 10. Conservation laws and control volumes

> Decision: [ADR-0100](../../adr/0100-modeling-functions-and-accounting.md)
> (proposed; implementation authorized).

Conservation and accounting use the same generic accumulator contract. Scientific subjects,
units and tolerances belong to packages. Contributions carry roles and source identities;
the assembler and independent checks consume those original contributions.

### 10.1 Accumulators over contributions

An accumulator declares a physical type, conservation or accounting mode, index domains
and an explicit physical tolerance. Contributions target actual accumulator occurrences.
Specialization checks ownership, complete physical compatibility and selected membership.
Conservation produces a zero residual; accounting produces a reported sum without claiming
a physical balance. Signed terms remain separate sparse-assembly contributions, preserving
local derivative coordinates and independent original-term closure observations.
There is no law-binding registry or hard-coded material/energy/charge subject vocabulary.

### 10.2 Contribution roles and signs

Conservation roles distinguish inflow, outflow, generation, consumption, accumulation and
transfer. Accounting can use explicit positive or negative contributions. Transfer identity
and side describe paired internal movement; they are not inferred from labels. Physical
quantity checking precedes summation. Reaction and heat/work terms are ordinary authored
expressions, so their basis and energy conventions must be stated by their package.
Independent checks remain distinct from a solver's scaled residual or convergence status.

### 10.3 Lumped control volumes, steady and dynamic

The process bundle's `ControlVolume0D` composes declared inlet/outlet state contracts with
component-phase, component-total, element-total or total material balances, enthalpy and
pressure equations. Optional holdup adds material and energy inventories and accumulation
under analysis-mode facts. Unit definitions add their actual heat, work and reaction terms.

The vessel is authored knowledge over amount/internal-energy inventories, PC-SAFT/DIPPR
properties and a directional-valve function. Integrated and simultaneous analyses consume
those same definitions. No Rust vessel constructor or legacy dynamic-case lowering remains.
The supported native dynamic profile and numerical limitations are owned by §13.

### 10.5 Formulation selection without code

Definition parameters, scoped values, presets, interface slots and `when` predicates select
formulations at specialization. Requirements reject unsupported combinations before native
execution. Analysis mode and initialization stage are explicit compile-time facts, not
ambient switches. Defaults and overrides are checked before demand expansion. Inactive
branches do not contribute equations; demanding an inactive member refuses.

## 11. Unit models

### 11.1 What ships and what executes

The authored process bundle contains Feed, Product, Heater, Mixer, Flash, four Separator
formulations, HeatExchanger, PressureChanger presets, CSTR, PFR, PID and SSLW costing.
These compose generic definitions with the selected state/property contracts. The seed
covers steady, initialized, integrated and simultaneous routes through authored fixtures.
The reference-package source notes and execution packet delimit actual oracle evidence.

Variables and equations specialize over declared finite membership, with source/instance
identities and bounded expansion. Continuous axes are discretized or lowered to native
integration; they are not silently treated as finite sets. Interfaces, inherited defaults,
function slots, indexed children, explicit partials, guards and contribution roles are
checked contracts. Limits are explicit policy and do not alter the mathematics.

### 11.2 Worked variant: adding heat input to a unit

A heater binds a control volume's inlet and outlet packages and constrains shaft work to
zero. Its heat term is already part of the energy accumulator. An adiabatic configuration
fixes that term to zero; a heated configuration specifies duty or a resulting state through
case bindings. The equations and physical type of heat remain shared.

This change belongs to authored definitions and cases in `packages/reference/process`.
It introduces no scientific registry family or Rust constructor. Independent checks inspect
the original signed energy contributions after solving.

### 11.4 Variants and narrowing

Interfaces declare required members and defaults; definitions implement or extend them.
An explicit override resolves competing inherited meanings. A checked descendant refinement
survives a diamond independently of base order, while conflicting siblings need a declared
override. Input parameters and physical/function contracts remain invariant; a child may
refine a declared interface through checked extension or implementation.

A science variant changes package data. A generic semantic gap is corrected once in the
kernel with synthetic controls before its scientific caller is accepted.

## 12. Connectivity

A modeling port exposes an existing typed coordinate with its own declaration identity and
owning instance. Directed connection occurrences produce mathematical equality constraints.
An explicitly selected topology can also feed structural and recycle analyses.

### 12.1 Typed ports

Ports bind scalar or indexed members already present in the specialized model. They retain
source and instance lineage independently of the underlying symbol identity. Their declared
physical contract must equal the target coordinate's complete type. Package interfaces
compose sets of these ports; the kernel has no closed material/heat/work/signal vocabulary.
An alias does not duplicate numerical storage.

### 12.2 Connection admission

Every connection endpoint must resolve to a declared active port. Both coordinates must
have the same complete physical type, including kind, basis, datum, scale and subject.
Source names resolve before equation generation. Missing or incompatible endpoints are
attributed to the connection declaration.

Every connected port also has an authored `annotation connectivity port(incoming, outgoing)`.
Each maximum is a nonnegative integer or `many`: `(1,0)` admits an input, `(0,1)` an
exclusive output and `(0,many)` a signal output permitting fanout. These are generic
incidence limits; science chooses them in package definitions. Specialization checks
original occurrences, including indexed coordinates, and refuses absent, conflicting or
exceeded policies before equation execution. An alias cannot erase a port's incidence.

For an explicitly prepared flow graph, each endpoint must belong to its selected node;
multiple assignments to one destination are refused by graph admission. The selected graph
and the model's full mathematical connection set are distinct projections: graph selection
never deletes the model's equations. Package conservation requirements and the native
structural/original-equation checks remain independently applicable.

### 12.3 Connection expansion during selected lowering

Specialization creates an equality from each admitted connection using the existing typed
mathematical path. Its identity is the connection occurrence's identity, with source lineage
retained. Native layouts are derived coordinates only. Removing a coupling for initialization
uses an explicit prepared strategy or immutable case overlay; it does not mutate a package.

### 12.4 Values across connections

Equality constraints convey values during simultaneous solution. Block initialization uses
library structural analysis and predecessor-ordered conditional solves. Causal recycle uses
explicit selected inputs, outputs and tear groups; hidden inputs refuse preparation.
Resolved numerical magnitudes, bounds and integrality project into these coordinates through
the shared policy owner. Failed stages preserve the original specification and their
structured failure evidence.

### 12.5 Flow projection and recycle structure

`ModelingFlowSelection` names actual instantiated owners, including isolates, and a tear
decision for every selected connection. A selection that cuts an incident connection,
names an unknown owner or supplies an unrelated decision is refused. Source port ownership
supplies endpoints; residual incidence and variable names do not invent topology.

`FlowGraph::admit` checks physical bindings and explicit limits, retaining parallel connection
occurrences and consistent group policies. Native HiGHS provides weighted exact tear
selection with bound/gap observations; the explicit petgraph heuristic is a distinct choice.
KINSOL owns fixed-point iteration. Tear analysis and native seed reuse are prepared execution
choices, not authored scientific facts.

### 12.6 Translation between conventions

Connections do not reconcile different physical conventions. A translator is an ordinary
definition with explicit inlet/output types and equations for the required basis, reference
or coordinate change. Mixing and splitting need authored conservation equations, and signal
or state aliases acquire no implicit conservation claim. The seed provides explicit Mixer
and Separator definitions; it does not promise arbitrary convention conversion.

## 22. Authoring and the extension model

> Decision: [ADR-0099](../../adr/0099-modeling-language-and-identities.md)
> (proposed; implementation authorized).

Modeling source and generated declaration values share one admission boundary. A checked
package revision retains its actual dependency closure and physical context. Runtime and
Python wrappers cannot mutate checked products or substitute a different physical owner.

### 22.1 Package layout

The registry owns document globs and section mappings. A bundle has a `package.toml`
manifest with its identity policy and its dependencies, each a package identity with a
typed version requirement; a manifest declares no physical names, which the physical
document owns ([§8.1](physical-semantics.md#section-8-1)).
`models/*.pse` contains the generic modeling language, using the shared expression grammar.
`materials/*.yaml` supplies physical relation rows; `cases/*.yaml` supplies measurement and
fit declarations; `assertions/*.yaml` supplies expected evidence.

Loading retains exact source bytes and spans, rejects unknown shapes/fields and resolves IDs
before typed interpretation. A caller supplies the manifest closure explicitly; no ambient
package discovery or separately maintained alias merge changes its meaning. Imports govern
which modeling names a package may reference, and resolve by package identity
([§6.1](schema-and-relations.md#section-6-1)); the physical names are visible exactly when
the manifest depends on the declaring package.

### 22.2 Revisions and edits

Document ID assignment uses parser ranges and emits reviewable replacements with exact
before-images. Explicit declaration IDs survive renaming; source byte ranges and content
hashes are separate facts. Unchanged documents can reuse immutable parser owners.

A modeling package exposes immutable generated declarations. `with_declarations` admits a
new revision against the same retained physical context; it cannot mutate the old checked
product. The compiler's single Salsa workspace owns dependency tracking and reuse. A value
edit and a clean reconstruction must agree. Preparation keys include their declared semantic,
physical, profile and resource-policy dependencies; a digest alone never admits a model.

### 22.3 What is data and what is code

Scientific equations, coefficients, entity kinds, memberships, method dispatch, unit models,
controllers, costing, diagnostics and fixture expectations belong in authored packages.
Code owns generic grammar/checking/specialization, physical type algebra, compiler/library
integration, bounded native algorithms, lifecycle, diagnostics and publication.

Symbolica built-ins are code primitives. Other mathematical forms compose authored functions,
including polymorphic smoothing and verified piecewise functions. An external-function
capability is an explicit contract for an actual registered implementation (§9.4), not a
back door for scientific-name dispatch. A future scientific port needing Rust records a
kernel gap and a synthetic test before the port is accepted.

### 22.4 Agent changes

Agents follow the same authoring and admission contracts as human callers. Edits identify
the intended source declarations, preserve explicit identities, and retain source provenance.
Requirements, conformance checks and original-model validation apply independently of who
wrote a package. Runtime results and initialization overlays never write back into source.
Repository decision and generation rules govern contract changes; execution plans own
current work rather than creating another model authority.

## Additional model specializations

#### 10.4 Distributed control volume

The authored spatial control volume and PFR consume continuous axes with backward-difference
or Jacobi/Radau collocation declarations. Their equations and bounds remain the same generic
model; §13 owns the native analysis profiles and the execution packet scopes the seed evidence.

#### 11.3 Pressure-change assumptions

PressureChanger binds an authored assumption interface. Pump, Compressor and Turbine presets
supply their parameter choices; isentropic and isothermal equations compose the required
state contracts. There is no Rust pressure-changer factory. See the process bundle's source
notes and fixtures for the actual selected comparisons.

