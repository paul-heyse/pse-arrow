# Structured documentation and evidence lifecycle: supporting evidence

These documents support the principal
[structured documentation lifecycle review](../../reviews/design_review_structured-documentation-lifecycle_2026-10-09.md).
They were gathered in two rounds. The first round mapped the existing system and compared
authoring formats for a review whose direction the maintainer rejected: per-element markers,
closed vocabularies, a schema and a validating reader added agent effort. The review was then
rewritten against a corrected target, in which a requirement that adds agent effort without
clearly removing more fails. The second round researched lightweight approaches for that target:
plan structure and section queries, how agents reach a query, evidence collection and staleness,
and outward tooling.
That review owns the combined scope, architectural judgment, findings, rule impacts and
evidence limits. Nothing here is a competing verdict or a disposition ledger. Identifiers inside
these documents are local to the inquiries: journeys J1–J9, scope-run patterns P1–P8, probes P1
and P2, candidates (i)–(iv), exhibits X1–X5 and walk-throughs (a)–(h). Only the principal
review's finding and rule-impact identifiers are intended for later disposition.

## Baseline and scope

- **Checkout:** dirty `main` at `4c24721e691187e1a5b28398b29722fbde671da8`, observed
  2026-10-09/10. About 245–256 paths were changed or untracked, including concurrent Plan 28,
  32 and 33 work.
- **Untracked principal documents** among those inspected: Plans 32 and 33,
  ADR-0168 to ADR-0172, `docs/lifecycle.toml`, the lifecycle scripts, the efficiency review and
  its evidence, and both workspace-content-lifecycle reviews and the handoff.
- **Subject:** repository-specific reference content. That means `docs/`, the process skills,
  roles and instructions as structured content, and `build/` producer outputs.
- **Excluded by maintainer direction:** library skills, which the maintainer is handling
  separately.
- **Evidence focus:** evidence captured for design reviews and planning is the primary subject.
  Capability-map evidence and product qualification receipts are treated as distinct
  categories.
- **Framing:** evidence is assessed for reuse, validity and shared collection mechanisms. It is
  not assessed for claim-to-evidence traceability.

## Documents

| Document | Inquiry and method | Principal content |
|---|---|---|
| [content-model-and-journeys.md](content-model-and-journeys.md) | Code mapping (read-only) across guidance sources and sampled reviews, plans, ADRs, register and evidence | Element catalog with identity, states and which side writes each relation; nine consumer journeys; mechanical decisions agents make ad hoc; vocabulary owners; guidance as content; verification of planning-time corpus claims |
| [mechanisms-and-consumers.md](mechanisms-and-consumers.md) | Code mapping plus two read-only commands | Mechanism inventory classified as representation-independent, coupled or duplicating; where each duplicated fact is decided; discovery and scaffolding surfaces; edit-cost matrices; *Tested* classification quality of the scope projection |
| [evidence-reuse-and-collection.md](evidence-reuse-and-collection.md) | Code mapping (read-only) over review/plan evidence, capability-map evidence, `build/` producers and the resource ledger | Evidence catalog by question answered; repeated questions; stale and orphaned evidence with concrete condition changes; repeated collection machinery; existing reuse and validity concepts; document/producer boundary |
| [library-alternatives.md](library-alternatives.md) | Library research: package metadata, documentation, primary sources and *Measured* syntax-degradation runs | Typed-block Markdown, canonical data, traceability tools, schema layers, projection and query, rendering owner, evidence frameworks and Git-native state, with a comparison table |
| [representation-exhibits.md](representation-exhibits.md) | Verbatim excerpts, re-expressed under four candidates, with walk-throughs | The efficiency review and Plan 33 (primary), Plan 28 colliding IDs, the reopened WCL target, and the canonical-selection bundle; candidates (i)–(iv); walk-throughs (a)–(h); per-candidate summary |
| [plan-format-and-section-query.md](plan-format-and-section-query.md) | Second round: tool research with *Tested* probes on copies of real plans and skills | Light plan structure (heading-kind comments), a section/row query CLI probe, pairing section kinds with skill guidance without copying it, and going-forward migration; failure modes of YAML/TOML/JSON for prose editing |
| [agent-runtime-reach.md](agent-runtime-reach.md) | Second round: Claude Code and Codex documentation and CLI behaviour | Lowest-effort ways for both runtimes to run a query and receive its output (command, shared skill, SessionStart pointer, MCP, hooks), with caps and parity gaps |
| [evidence-collection-and-staleness.md](evidence-collection-and-staleness.md) | Second round: library research with *Tested* scratch probes on the pinned toolchain | PEP 723 and Rust frontmatter scripts, automatic capture of what a probe used, staleness from comparing that record with the tree, findability, and options for the smallest shared runner |
| [lifecycle-tooling-survey.md](lifecycle-tooling-survey.md) | Second round: outward survey with hands-on scratch trials | Agent-oriented planning, knowledge and freshness tools (Beads, Backlog.md, Spec Kit, OpenSpec, Basic Memory, IWE, mdq and others): what each automates and what compliance it asks; patterns worth adopting |
| [probes/README.md](probes/README.md) | Executed scratch probes | P1 (typed containers in markdown-it-py 4.2.0 / mdit-py-plugins 0.6.1), P2 (mdBook 0.5.4 over the staged live corpus; staging rewrite vs preprocessor; Pagefind anchors) and the `git merge-file` exhibit |

## Conditions and limits

- **Inquiries were read-only on the repository.** The commands executed:
  - `scripts/pse-env --docs --resource-class light -- python3 -B -m scripts.document_lifecycle scope`
  - `python3 -B -m scripts.adr index --check`, through the same wrapper
  - probes P1 and P2 and the merge exhibit, in a session scratch environment, with conditions in
    `probes/README.md`
- **Only small probe sources and results are retained here.** Scratch venvs, staged corpora,
  built sites, fetched upstream documentation and rendered fixtures were not retained. The
  commands to reproduce them are in `probes/README.md`.
- **Two deviations from the review plan.**
  - One first-round library-research run sent a synthetic 11-variant syntax fixture to GitHub's
    `POST /markdown` renderer. The fixture contained no repository content; it is shown in
    `library-alternatives.md` §0.
  - In the tooling survey, the beads CLI's default-on anonymous usage metrics were not disabled
    before its first commands. One detached upload attempt may have sent command names, the
    version, the OS and a hashed machine identifier, with no repository content. Metrics were
    then turned off and the queued events deleted. Details are in `lifecycle-tooling-survey.md`.
- **Retained probe material.** Second-round probe sources and small outputs are retained under
  `probes/plan-query/`, `probes/evidence-runner/` and `probes/tool-trials/`.
- **Search coverage.** Inbound-link and reference searches matched file names over `docs/`
  (excluding `docs/book`), `AGENTS.md`, `.codex/skills`, `.claude`, `scripts/` and the
  justfile. Paths constructed in other code were not traced exhaustively. Large JSON payloads
  were inspected by their top-level keys only.
- **Probe scripts locate the checkout** with `git rev-parse --show-toplevel`. Scripts under
  `probes/merge/` and `probes/p2/` read live repository files, so rerunning them against a later
  tree observes that tree.
