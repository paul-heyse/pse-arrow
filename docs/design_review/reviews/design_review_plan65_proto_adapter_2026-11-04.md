---
title: "Plan65 shared Proto adapter boundary assessment"
date: 2026-11-04
status: complete
standard: core-3.4
heuristics: "1.0"
tier: design
purpose: target
scope: ADR-0244 shared descriptor adapter and generator/host dependency direction
verdict: accept
doc_role: review
doc_owner: docs/plans/65a-contract-foundations-and-generation.md
doc_topics: [design, proto, code-intelligence]
---

# Shared Proto contract adapter

**Accept the proposed boundary at document and inspected-interface evidence level.**
Move the existing Buffa descriptor adapter into `pse-contract-proto`, retain semantic
checking in wire-independent `pse-contract`, and leave domain facet interpretation with
its native owner. This gives generation and the independent host the same option
interpretation without giving the host schema/Arrow/product dependencies. It does not
establish a completed extraction, runtime qualification or performance benefit.

This is the bounded independent assessment requested for proposed ADR-0244, using core
3.4, efficient-architecture heuristics 1.0 and the repository binding. Its focus combines
the concrete library boundary and semantic authority. The reviewer is implementing the
downstream developer-evidence consumer and domain facet policy, but did not author the
core descriptor adapter or this ADR. No broad diagnostic audit or scientific review is
claimed: the question is a specific library/dependency boundary, with no change to
chemical meaning, numerical execution, native custody or workflow verdict construction.

The inspected baseline is `main` at `66ccea813`, the proposed ADR-0244 working draft,
and the foundation worktree's current
`pse-codegen::codegen::boundary_projection::metadata` and `metadata::check` sources.
These include `DescriptorPolicy`, separate old/new `DescriptorInput` policy instances,
`NativeSelection` and decoded-domain callbacks. They are moving source, not an immutable
completed implementation. Plan65a owns implementation disposition; Plan65g/65j own
the independent consumers.

## Responsibility and change scenarios

The relevant phenomena have adequate distinct representations: a complete acquired
descriptor world; original bytes and source association; native source-selected coverage;
known versus unavailable interpretation; native interpretation version; data versus
observation-only construction; operation effects; and domain facets with their own policy.
The adapter translates actual descriptor facts into the pure checker's model. It does
not determine scientific validity or create a native performing capability.

| Responsibility | Target owner | Reason for this dependency direction |
| --- | --- | --- |
| Native declarations and semantic/evolution judgment | `pse-contract` | Shared meaning remains usable without a wire runtime. |
| Buffa linking, typed core-option schema correspondence and lossless normalization | `pse-contract-proto` | Generator and independent reader consume this same transport interpretation. |
| Graph lowering, source compiler/plugin invocation and generated publication | `pse-codegen` / `xtask` | These need generation facilities and may depend on source registries. |
| Scientific/registry facet interpretation | Domain owner, supplied explicitly | A host lacking that policy reports Unknown rather than importing the product or inventing semantics. |
| Original private evidence, waited status and rerun eligibility | Native host workflow/invocation owners | Descriptor readability cannot replace custody or current contextual checks. |

Three representative changes support the seam. A core option gains a new known
interpretation: one transport adapter and pure checker change, with both consumers
exercising the same refusal behavior. A domain adds a physical facet: its explicit
interpreter changes while an unaware host remains Unknown. A consumer reads an old
world: that world's policy and own descriptor pool govern its decoding; the new world's
pool cannot reinterpret its bytes. No global mutable pool or service is required.

## Alternatives and library fit

Importing `pse-codegen` into the host imports a registry/generation responsibility and
unrelated Arrow/product dependencies. A host-local transport normalizer would duplicate
consequential option schema, presence and unknown-field rules. Adding optional Buffa to
the pure crate obscures its actual dependency contract. Diagnostics and build information
are not owners of schema interpretation. The additional small library is justified by
two concrete consumers and an existing cohesive implementation to move; a forwarding
facade that leaves either interpretation path behind would not satisfy this decision.

Buffa supplies the generic mechanics directly: `DescriptorPool` links the complete
import closure, `DynamicMessage` decodes against its own pool, checked `try_set` enforces
typed option writes, and extension payloads remain available through native option
storage. The adapter owns the project's known-option correspondence and approval policy,
not a new protobuf codec or generic schema language. Relevant APIs are characterized by
the [Buffa 0.9.2 reflection source](https://github.com/anthropics/buffa/tree/v0.9.2/buffa-descriptor/src/reflect)
and [guide](https://github.com/anthropics/buffa/blob/v0.9.2/docs/guide.md).

Pool preparation belongs to the selected immutable request/world lifetime. Moving this
code does not justify caching every schema globally, repeatedly compiling schemas per
field, or constructing full product contexts for one report read. Keeping source bytes
alongside a linked world serves distinct identity and interpretation purposes. Actual
allocation and latency benefits were not measured.

## Judgments and implementation constraints

| Foundation | Target judgment and basis |
| --- | --- |
| AP-01 | Satisfied: transport, semantic policy, generation and native custody have separate reasons to change. |
| AP-02 | Satisfied: a narrow descriptor input/policy/check interface hides Buffa normalization details without promising runtime independence it does not provide. |
| AP-03 | Satisfied: explicit domain callbacks compose interpretation; neither host nor pure model imports a product registry. |
| AP-04 | Satisfied: unknown interpretation, source coverage, construction role and effects remain behavioral distinctions enforced by the single checker. |
| AP-05 | Satisfied: typed options, explicit presence and known-schema correspondence replace comments/default guesses. |
| AP-06 | Satisfied: complete descriptor fixtures and explicit policies can test the library without solvers, a database or a host process manager. |
| AP-07 | Satisfied for this boundary: one selected world is linked locally, existing work is moved, and no extra service/store/coordination lifecycle is introduced. |

G1–G8 have no identified target-design gap within this boundary: authority is singular;
meaning and unsupported policy remain explicit; inspection has no performing effects;
original identity is retained independently of interpretation; and generic decoding uses
Buffa. G9 passes from the individual foundation judgments. Scientific profile gates are
outside the stated subject because this extraction neither interprets chemical contracts
nor changes their numerical realization.

The following are necessary preservation constraints, not implementation acceptance:

- Move the actual adapter, typed core schema guards, typed errors and focused tests.
  Remove the original implementation and migrate both callers; do not keep two copies.
- `pse-contract-proto` must import no codegen, registry, Arrow, database, runtime or product
  crate. Translate its errors at each consumer boundary rather than exposing `SchemaError`.
- Each old/new input supplies its own interpretation policy and pool. Domain callbacks
  examine the actual decoded value before normalization; unavailable policy stays Unknown.
- Native coverage comes from the approved source declaration/bundle policy independently
  of acquired option presence. Reading an acquired descriptor cannot approve its own
  stripped fields or missing native owners by deriving expected coverage from itself.
- Retain countercontrols for unknown schemas, stripped nested metadata, changed option
  types/tags/defaults, absent imports, distinct original-world policies and observation-only
  construction. Passing correspondence round trips alone cannot establish these properties.

No design correction to the proposed crate boundary is required. The explicit source-
coverage provenance and per-world policy constraints should remain visible in the ADR
and public interface documentation during extraction. ADR-0244's crate/dependency and
architecture revision is the rule impact; no change to pure model authority or native
custody is recommended. No product tests or qualification campaigns were run for this
assessment.

## Addendum: shared native crossing mechanics

**Accept the scope amendment as a proposed design, with the preservation constraints
below.** This addendum assesses ADR-0244 at `f158cb399`, after the host consumer exposed
its dependency on `pse-runtime::projection_support`. It does not revise the earlier
descriptor-adapter assessment or promote the amendment to runtime acceptance. The
reviewer has not authored the proposed helper move or pure scalar-access mechanism.

The inspected runtime module combines general scalar/container conversion, duplicate and
layout checks, generated-validator errors, native identity/buffer bindings, and runtime
diagnostic presentation. Copying it into the host would replicate consequential
crossing behavior. Importing it from the runtime would defeat the independent host's
dependency direction. Moving the generic mechanics to the existing transport owner
has the same concrete consumers and reason for change as the approved adapter seam.
The actual scalar owns its byte representation and checked construction; the transport
helper owns protobuf conversion; native presentation owns diagnostic categorization.

This split satisfies AP-01–AP-06 and preserves their earlier judgments. It removes
incidental runtime ownership without making identities depend on Buffa or moving a
scientific interpretation into transport code. AP-07 remains satisfied at proposal
level: source-generated typed crossings share the existing operations, with no new
erased value tree, service, pool or persistence lifecycle. Native borrowed observations
still require an encode-only route, and the approved pure checker still owns metadata
judgment. No new core gate gap is identified; G8 specifically favors the maintained
validator's actual result over a bespoke substitute. G1–G7/G9 acceptance remains about
this proposed boundary, not the unexecuted migration.

The following constraints distinguish the correction from relocation or weaker behavior:

- Move generic scalar/container mechanics and their tests once; delete the runtime-only
  copy after runtime and host call sites migrate. Transport code must import no native
  model, identity, runtime or registry crate merely to convert a nominal value.
- Put wire-independent byte access and checked reconstruction at the actual scalar
  owner, through `pse-contract` contracts if a shared trait is needed. Semantic IDs,
  content hashes and owned byte buffers retain their distinct native widths and
  constructors. No Buffa dependency belongs in the identity crate, and no byte-access
  contract belongs on a live verdict, readiness, custody or settlement capability.
- Resolve Rust coherence at the typed boundary. A shared conversion trait cannot simply
  receive the previous runtime-local implementations for two external types. Generated
  scalar arms can call the pure owner's access/constructor contract; they must not
  recreate nominal IDs with unchecked casts or add consumer-local identity mirrors.
- Preserve exact byte/float representations, optional absence, ordered collections,
  fixed extents and map/set duplicate refusal. A generic helper must not silently
  truncate a fixed identity, overwrite a repeated key or turn an unavailable alternative
  into a default. Keep strict adoption and original-wire-retaining interpretation
  separate where those policies already differ.
- Keep the maintained validator's typed error as the cause, including compilation,
  evaluation and individual violations. Incoming data rejection, outgoing native
  producer failure and outgoing wire validation remain distinguishable. Consumer
  wrappers may assign native diagnostic classes; transport must not flatten the cause
  into a string or import the model's presentation trait to reproduce runtime policy.
- Generated observation-only roots must remain borrowed encode-only through every
  helper entry point. A new shared scalar or container helper cannot introduce an
  inverse constructor for sealed owners or a reconstructible parent containing one.

The settling implementation checks are focused helper controls for exact extents,
absence, duplicate refusal and typed validator causes, plus compilation of the generated
runtime and host consumers. Existing one-way generation countercontrols must continue
to distinguish observation from native construction. The amendment's value is a single
shared implementation and correct dependency direction; no speed, allocation or full
workflow qualification claim is made. Formal ADR acceptance and the Plan65 implementation
disposition remain separate from this assessment.
