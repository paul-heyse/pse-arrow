---
title: "Plan 52i: Canonical capabilities and progressive reveal"
status: draft
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, environment, documentation, libraries]
---

# Plan 52i: Canonical capabilities and progressive reveal

## Purpose and foundation

**Proposed:** make every agent-facing capability discoverable through one structured FastMCP
entry point, with exact conceptual navigation, relevant instructions and deliberate reveal.
This companion develops the [unified-interface review](../functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md)
O01–O03 and the exposure portions of O09/Q01/Q06. The
[coordinator](52-agent-effectiveness-and-engineering-workflows.md#unified-interface-opportunity-dispositions)
owns all progress and source-qualified dispositions. Creating these documents does not implement
or attach the interface.

Existing `scripts/workspace_domains.py`, `workspace_context.py`, native lifecycle projections,
recipe introspection and semantic-navigation bridges supply useful facts. They currently expose
different discovery routes; `codex_session.py` attaches language/database specialists separately.
The [earlier inventory](../functional_reviews/agent-tool-discovery-and-access-2026-10-24.md#what-an-agent-can-already-use)
found useful tools beyond the initially named plans. Therefore the target models the repository's
whole agent capability surface, rather than wrapping a convenient subset of commands.

The focused foundation assessment selects those native operations and source owners for reuse.
It replaces duplicate exposure decisions and incomplete outer recipe grammar with canonical
registered components. It does not replace native ownership, literal/regex source search,
language symbol semantics or ordinary shell/Git access. Structured navigation has no semantic
search, embedding, ranked text discovery or fuzzy capability resolver.

## Target and authority boundaries

A capability has one exact identity and implementation binding, an owning domain, workflow
memberships, effect/prerequisite meaning, input/output contract references and guidance links.
Domain describes what it concerns; workflow describes why several capabilities are useful
together. Both are exact views over the same identities. Workflow membership never copies the
operation, authorizes its effects or invents another scheduler. Ordinary modules/providers
compose one public endpoint; internal providers are not separate required agent attachments.

Author semantic exposure metadata once at the registration owner, with typed constraints and
duplicate/dangling-identity checks. Derive FastMCP tags, catalogue views and generated reference
from that declaration. Infer signatures from the actual function/custom Tool contract, not
from a handwritten second CLI grammar table. Native schemas, source documents, result facts,
credentials and current plan status keep their present owners. Metadata such as effects is
descriptive; operation admission enforces the actual rule.

The bootstrap supplies concise orientation and exact domain/workflow listing, descriptions,
selection and reset. Describing a known hidden capability can return its contract/guidance
without calling it. Unknown, unavailable, hidden, unsupported and stale-loaded identities are
different states. A known operation can be unavailable because its selected native dependency
is absent without disappearing from the conceptual catalogue.

Selection changes advertised components only. Checkout, selected project/store, service identity
and process permissions remain explicit operation/attachment context. Reveal is not a security
boundary or permission grant. Bootstrap and deliberately offline reads remain reachable when a
database, language server, compiler or workload admission is unavailable.

### Library mechanisms and constraints

Use ordinary registration first and a custom provider only where exact lookup or deferred rich
schema preparation earns it. FastMCP4.1.3's
[custom providers](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/providers/custom.mdx)
and [visibility implementation](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/transforms/visibility.py)
provide composition and enable/disable/reset. Prepare a compact exact metadata index once per
loaded interface revision. For a small cheap catalogue, preparing schemas once may be simpler;
do not add a cache hierarchy without a reuse case. Listing fewer tools does not prove less
server work: a provider can enumerate/materialize all components before filtering, and default
lookup can scan its list. Avoid repeated full provider construction on every selection/read.

`ctx.enable_components`, disable and reset operate at transport-specific scope. Initial supported
use is one STDIO server process per attachment. The framework's STDIO/in-memory state is process
scoped, not separate client-conversation state inside one process. Authenticated HTTP state can
span a principal's connections; unauthenticated HTTP cannot supply the same reveal storage.
Do not advertise HTTP/session isolation without a selected requirement and its real tests.

Use [resources and ordinary skill providers](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/providers/skills.mdx)
for owned Markdown and scope guidance; prompts for genuinely parameterized task preparation.
`ResourcesAsTools`/`PromptsAsTools` may expose the same content if actual host presentation needs
them. The experimental Skills extension excludes relevant mounted/namespaced/proxy sources and
does not activate or execute skills; it is unselected. Returning a planning skill performs no
implementation and requires no acknowledgment before unrelated tools can run.

## Work packages

<a id="CAP01"></a>
### CAP01 — Canonical exposure model and first operation

Identify the minimum domain/workflow/component metadata needed by a real source-only operation,
a native-backed operation, a skill and an unavailable optional utility. Choose stable exact IDs
and one composition root. Keep unrelated native namespaces and tool transport names distinct;
an upstream proxy's name cannot silently collide with a local operation. Reject duplicate IDs,
unknown workflow references and contradictory schema ownership when assembling components.

Deliver the first source-only operation through this model alongside ADP02's small typed-host
slice and MCP01's bootstrap. A task-context or existing result read is suitable: explicit input
and checkout/context in, actual owner read, bounded outcome and reference out. Do not create a
placeholder operation whose eventual implementation will have different semantics.

Acceptance: one real registered capability supplies its signature, description, effects and
guidance to the bootstrap and callable dispatch from the same declaration. An independent fixture
with a duplicate identity fails composition; source-only use has no database/service startup.
Migrate its current discovery consumer if replaced. The model must admit remaining inventory
families without needing an operation-specific switch in a universal dispatcher.

<a id="CAP02"></a>
### CAP02 — Exact catalogue, descriptions and bounded preparation

Build domain/workflow browsing, exact capability description and referenced schema/guidance
lookup over canonical registrations. Provide deterministic order and explicit continuation for
large listings; retain selection/completeness and known-unavailable prerequisites. An empty
domain result differs from an unknown domain. Do not substitute keyword relevance for identity.

Extend existing context consumers to link canonical capability IDs where they describe access.
Their native source/status projections remain native reads, not copied catalogue facts. A
recipe may remain a useful low-level route, but an outer `*args` signature cannot masquerade as
a complete operation contract. Reference leaf ownership or complete the adapter in ADP03/ADP05.

Acceptance: find a useful unfamiliar tool outside Plans47/50/51/52 by declared domain/workflow,
describe its actual inputs and legitimate next step, and obtain the real implementation owner.
Check bounded page/lookup behavior with providers that expose accidental reconstruction counters;
this is a structural regression check, not a measured throughput claim. No scans of all raw
transcripts, service probes or scientific data happen merely to list capabilities.

<a id="CAP03"></a>
### CAP03 — Deterministic reveal, replacement and reset

Resolve an exact requested domain/workflow/capability selection to its effective component set.
Define replacement rather than accumulating hidden history; reset restores the documented
bootstrap. Serialize each process's selection changes or use an equivalently atomic mechanism.
The framework's read/append/put helpers alone do not prove concurrent replacement is safe.
Repeated selection is idempotent; unknown selections cause a structured refusal with no partial
state change. Return the selected IDs, loaded revision and applicable instructions.

Use native FastMCP visibility and list-change notifications where they provide this behavior.
Selection includes necessary instruction resources/prompts without changing project/service
context. Decide whether an unavailable capability is callable only to explain its prerequisite
or represented through description, and keep that choice consistent with ADP02's result contract.

Acceptance: two actual independent STDIO processes show independent selection, hidden-call
refusal, replacement, repeat selection and reset; concurrent select/reset cannot leave a union
that neither request chose. MCP02 owns actual Codex inventory refresh qualification. If host
refresh is unsupported, its exact same-endpoint adapter resolves the same canonical components;
no shell language or second developer endpoint substitutes for selection.

<a id="CAP04"></a>
### CAP04 — Progressive guidance from existing owners

Compose three levels: short root orientation, useful selected workflow/domain guidance and exact
operation constraints/examples. Link current AGENTS/rules/skills and native guides rather than
copying policies into a second editable manual. Generated reference derives signatures and
references; authored rationale remains authored. Keep critical effect/interpretation constraints
in callable descriptions and enforcement as well as deep links; retrieval is not an admission
gate. Relevant instructions may be substantial when the selected task needs them.

Serve resources/templates and selected skill directories without making discovery import native
extensions or start services. Readiness and loaded revision follow MCP03. Host compatibility
adapters must present the same content, not maintain another prompt/skill body. Preserve planning
versus execution skill pairs and explicit operator-selected review lenses.

Acceptance: a planning-only selection returns the proper preparation skill and current scope
instructions without effects; a generated-contract selection identifies its declaration owner
and generation route. Check authored-content links and contract/guidance revision alignment.
G03 owns actual agent task evaluation and instruction improvement, rather than CAP04 claiming
agent effectiveness from a resource listing.

<a id="CAP05"></a>
### CAP05 — Whole-repository classification and exposure completeness

Reconcile the source review's inventory against current recipes, scripts, Python/native entry
points, skills, optional utilities and session configuration. Include new relevant tooling added
since the review, without importing its separate plan's implementation obligations. For every
agent-useful family, select canonical operations, internal implementation, retained low-level
native access, or a justified unselected/unsupported capability with an owner and revisit cause.
Do not label an unadapted required operation complete merely because its family is listed.

Cover context/ownership, language edits/codemods, history/knowledge/library research, environment/
build/cache, verification/selection, workload observation/results, scientific/offline and active
diagnostics, database tooling, projects/revisions/data/evidence, experiments/isolation/recovery,
schema/codegen, docs/decisions/lifecycle and Git/integration. This is migration planning coverage,
not a new runtime inventory authority: the delivered view derives from component registration.

ADP05 owns actual operation/caller migration and deletion, MCP04 derived consumers, G02 integrated
workflows. Maintain their decisions at the numeric coordinator, with source links from relevant
package explanations, not a second mutable status table here. Removal of optional utilities
needs a technical reason; unknown or empty search evidence never proves capability absence.

Acceptance: inspect actual entry points and consumers, classify omissions explicitly, and show
that required families are both discoverable and usable through their real operations. A new
future scientific inspection operation should add its native binding plus local exposure metadata
and guide reference; it must not require a second endpoint or edits to unrelated workflow logic.

## Dependencies, acceptance and limits

CAP01/ADP02/MCP01 establish the first useful slice; CAP02–CAP04 and MCP02 extend it to actual host
discovery→instructions→operation→result. CAP05 begins classification early and reconciles final
coverage with ADP05; it is not an all-inventory barrier before useful operations can ship. Native
operations wait for ADP01/ADP04's actual decoder/lifetime integration, not merely agreed schemas.
MCP03 governs loaded metadata/handler/guidance coherence. G01 retains shared interaction meanings,
G03 actual agent evaluation, and H01–H04 assembled reconciliation/qualification/publication.

AP-01/AP-03 keep exposure separate from native effects while allowing coarse operations;
AP-02/AP-04/AP-05 require real dispatch/validation and meaningful state distinctions;
AP-06/AP-07 require local context and proportionate prepared work. No new scientific authority,
solver contract, distributed control plane or universal result ontology is selected. Missing
native assurance cannot be supplied by tool annotations. A clean catalogue is not full-plan
acceptance, and reveal is not proof of reduced runtime work or improved agent productivity.

**Proposed verification:** targeted metadata/reveal/guidance tests accompany functional changes;
actual attachment behavior belongs to MCP02, native contract fidelity to ADP01 and assembled
agent journeys to 52h. The review's bounded library-client probes are historical feasibility
evidence only. This authoring adds no production endpoint, host qualification or measured benefit.
