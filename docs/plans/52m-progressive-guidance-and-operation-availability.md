---
title: "Plan 52m: Progressive guidance and operation availability"
status: draft
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, environment, documentation, libraries]
---

# Plan 52m: Progressive guidance and operation availability

## Target and foundation assessment

**Proposed:** reveal useful instructions and exact operation owners with compact responses, while
keeping an unrelated implementation edit from unnecessarily disabling a source-only capability.
This develops usability-review [O03](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O03),
[O05](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O05),
[O09](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O09) and
[O13](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O13).
The [coordinator](52-agent-effectiveness-and-engineering-workflows.md#usability-opportunity-dispositions)
owns progress. CAP/MCP's completed discovery, lazy guidance and restart fences remain the foundation.

**Interface-checked:** selection still returns guide bodies and expands families beyond the
immediate workflow question; prompts already use compact resource references. Inventory's module
path inference does not correctly resolve the `scripts.capabilities` package entry. Coherence follows
module-wide imports including sibling lazy imports, and native worker inference uses a name prefix.
`Interface.common` checks exact common paths, not their transitive closure. Preserve that distinction
when repairing the actual operation closures. These observations justify operation-aligned changes,
not deletion of safe fences or a claim that earlier MCP03 qualification was invalid.

Apply Core3.4, Heuristics1.0 and Test Assessment1.2, particularly AP-01/AP-05/AP-06/AP-07 and
H5/H6/H13/H18/H26. A new source lead is not a proven owner, visibility is not admission, and
instruction retrieval does not activate a skill. No semantic selection or second developer endpoint.

## Combined design

Canonical registration owns exact domain/workflow membership, actual callable ownership, fixed
workers and relevant guide references. Use it and Just's structured declaration rather than a second
editable command catalogue. Current-source facts and the endpoint's loaded interpretation remain
separate. A loaded receipt can report the serving revision while current owner queries report their
actual source cut. Source/worker changes are checked before effects under the actual relevant closure.

Selection produces a compact receipt: effective exact IDs, loaded revision, instruction-resource
references/digests, completeness and how to read full content. Keep critical effect/interpretation
constraints in tool descriptions and enforcement. Complete guidance remains deliberately retrievable;
there is no acknowledgment ceremony. Workflow intent can select declared relevant steps without
activating every operation in every mentioned family. Explicit whole-family selection remains useful.

## Work packages

<a id="NAV01"></a>
### NAV01 — Compact instruction reveal and useful workflow steps

Change selection and its actual consumers to resource-index delivery, reusing the existing prompt
and resource/tool projections. Compare an explicit full-instruction bundle for consumers that need
one; do not resend all bodies by default on repeated identical selection. Maintain atomic replacement,
unknown-ID refusal without partial changes, independent attachments and reset semantics.

Refine broad workflow declarations into meaningful intent/steps where the current family expansion
adds irrelevant operations. Definitions remain exact and authored, with a full path to applicable
guidance. Do not use similarity, silent summaries, truncation or larger host token limits to choose
instructions. Migrate native/tool-projected content and generated reference together; remove replaced
embedded guide copies when callers use the resource index.

Targeted acceptance: inspect-plan and a generated-contract task obtain the declared useful selection
and compact references; deliberate reads recover complete matching guidance. Repetition, concurrent
select/reset, invalid selection, reset and planning-only retrieval retain their original distinctions.
G03 assesses actual agent use and excess instruction loading; byte-size observations alone do not
establish productivity improvement or permit dropping applicable instructions.

<a id="NAV02"></a>
### NAV02 — Accurate ownership and local capability extension

Use known operation/worker metadata and structured Just declarations for established owners. Resolve
source-only `python -m` module, package `__main__.py` and nested module forms without executing imports
or arbitrary shell. Dynamic structure that cannot be resolved stays an attributed lead with explicit
uncertainty. Recipe outer grammar, parser coverage, native-route classification and actual bindings
remain separate facts. Preserve current public recipes; this work does not replace their native purpose.

Assess whether an existing registration/provider boundary already supplies each needed fact. Use a
custom provider only for genuine sourcing/lifecycle responsibility, not to rename a static loop.
An ordinary new capability should add its owner/binding and derive reference/navigation coherently.
Delete replaced path guessing and duplicate catalogue facts once actual consumers migrate.

Targeted acceptance includes the actual capability package launch, ordinary modules, nested modules,
unresolved dynamic commands and one new capability fixture with independent expected source owners.
Descriptions and generated links must lead to real declarations without discovery imports or service
startup. No all-repository execution or exhaustive new handwritten catalogue is required.

<a id="NAV03"></a>
### NAV03 — Dependency-aligned loaded operations and restart diagnosis

Separate lightweight shared contract/result definitions from binding composition and effectful
adapters where their reasons for change differ. Consume NAV02's real registered worker facts rather
than a `native_` naming heuristic; the pure `native_contract_schema` route must not acquire an unrelated
execution worker dependency. Retain conservative treatment of unknown dependencies and capture before
import/composition with verification afterward. A manually incomplete whitelist is not a sound repair.

Make loaded revision, affected operation and restart remedy inspectable without raw arguments or a
second status store. Reconnecting obtains a fresh coherent endpoint; it never replays the interrupted
action. Do not hot-reload handlers or automatically restart during effects. Shared interpretation
changes legitimately invalidate their consumers even when operation closures have become smaller.

Targeted acceptance distinguishes an unrelated coverage implementation edit, an actual handler/schema
edit, actual fixed-worker change, shared interpretation change and fresh-endpoint recovery. Verify
refusal before effects and unchanged source-only reads with science services unavailable. Remove
obsolete broad imports and prefix-based worker associations when consumers use actual declarations.
I14's endpoint-shutdown inquiry retains its existing separate owner; dependency refactoring cannot
claim that inquiry or native sibling settlement complete.

<a id="NAV04"></a>
### NAV04 — Supporting instruction resources and exact completion

Expose bounded supporting-file manifests and deliberate reads for owned process skills, with source
digests and the same loaded/captured interpretation as the guide that references them. No automatic
download, trust, activation or execution follows from serving a skill. Shared library skill resources
retain their repository-agnostic ownership and existing access boundaries.

Compare the existing lazy guidance owner with FastMCP4.1.3
[`SkillProvider`](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/providers/skills/skill_provider.py)
and [`SkillsDirectoryProvider`](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/providers/skills/directory_provider.py).
Construction scans/reads and later reads can observe live bytes; reload rescans. Adopt only if those
semantics are reconciled without defeating lazy startup and matching guide/resource revisions.
Experimental Skills negotiation remains unselected absent a concrete consumer.

Evaluate [`@mcp.completion`](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/docs/servers/completions.mdx)
for real prompt/resource-template parameters, such as exact capability IDs. It does not complete
ordinary tool arguments or improve argument-free prompts. Explicitly apply the intended exact
visibility policy; completion does not automatically inherit the referenced component's visibility.
If the host has no useful completion consumer, retain CON01's schema examples and record this narrower
outcome/revisit trigger instead of adding an unused completion surface.

Targeted acceptance: supporting files match the captured guide revision; changed/missing/oversized
files refuse truthfully; hidden/unknown completion references do not leak components under the chosen
policy. A positive host-dependent completion adoption needs actual consumer use. Delete any replaced
skill-resource implementation only after migration; do not keep both framework and bespoke providers
for the same meaning.

## Dependencies, verification and publication

**Proposed:** NAV01/NAV02 can start from delivered registrations. NAV03 consumes actual ownership/
worker facts and CON02's lightweight contract split as relevant; it does not wait for all result-family
migration. NAV04 consumes NAV01's reference/coherence contract. Core reveal improvements do not wait
for optional completion or framework-provider adoption. Future task/composition infrastructure must
not become a bootstrap dependency through registration imports.

Use targeted selection, inventory, guidance and coherence tests during implementation. Regenerate
changed canonical reference declarations as part of the change. G02/G03 and H02 own fresh actual
Codex checks, including fixed-inventory invokers and ordinary revealed tools where supported. H03
owns scope-end static/manual checks, not a campaign per package. Publish actual supported reveal,
restart and extension procedures in the capability/environment guides. **Interface-checked** static
foundations and **Proposed** packages are distinct; no new host test or measured benefit is recorded.
