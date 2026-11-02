# Design principle audit

This living account diagnoses design-principle alignment in the implemented simulator and its
development tooling. Start with the [integrated system account](system-architecture.md), then
follow the primary finding homes below. Findings are diagnostic evidence; solution reviews and
owning plans retain their separate responsibilities.

## Whole-system refresh — 2026-11-01

**Subsequent scoped refresh — 2026-11-02:** the
[process-simulator target account](process-simulator-target.md) rechecks applicable scientific
and composition diagnoses at `03765c22e` for the new holistic target review. It adds one
initialization-report crossing under DA-039, preserves existing finding homes and distinguishes
new scientific target breadth from supported-behavior defects. The broader baseline below
and the separate test-account baseline are not silently advanced.

**Interface-checked.** Three new source diagnoses were found:

| Identity | Diagnosis | Primary home |
|---|---|---|
| [DA-061](structure-and-invariants.md#da-061) | Parser depth limits do not bound the constructed recursive tree | Structure |
| [DA-062](execution-fit.md#da-062) | Strategy acceptance and original quality apply different variable-bound tolerances | Execution fit |
| [DA-063](execution-fit.md#da-063) | Pending global study capture also gates unrelated renewal and cancellation | Execution fit |

Permission handoffs, repeated cross-owner composition and error projection remain meaningful
current concerns. The prior account also required substantial correction: the Python authority
split has retired; host diagnostics and generated-path ownership improved; retained-data services
can be constructed independently. Counts of mechanisms, files or consumers are insufficient to
retain several old diagnoses.

## Baseline and standard

- **Audited source:** `19e7b49a67df24c1bf73005a6186e145e930d53a`, clean working tree when
  investigation began. No product edits were included. Audit documentation changes are the
  publication, not an additional source baseline.
- **Preceding whole-system account:** 2026-10-29, source `f83bac039` plus its enumerated dirty
  work, reconciled there to `fbf08a43c`. The account immediately before this refresh is
  recoverable from `19e7b49a6` and Git history.
- **Scoped predecessors:** [Plan59 host contracts](plan59-host-contracts.md) and
  [Plan61 contract foundation](plan61-contract-foundation.md) retain their own dated source and
  coverage. Neither had refreshed the whole-system baseline. They remain scoped evidence.
- **Applied standard:** Core 3.4, efficiency heuristics 1.0, process-simulator profile 1.5,
  test-assessment 1.2 and the pse-arrow binding selected by
  [standard.toml](../design_principles/standard.toml).
- **Functional target:** the current process-systems-engineering core and its engineering
  workflows, interpreted through the architecture owners and applicable profile. Canonical
  SurrealDB, durable work surviving its caller, ordinary uncapped host execution and the
  existing native scientific boundaries remain premises, not exemptions from assessment.

No broad tests, scientific campaigns, fault injection or performance measurements were selected.
The operator chose static-first inspection with focused probes where useful.

## Method and code-intelligence use

The coordinator used the audit skill and native CLI skill, mapped current architecture and
source ownership, and compared the preceding account's premises with current source.
Six read-only reviewers first investigated independent questions without reading their assigned
old findings: scientific meaning, numerics, durable state, workflows, host engineering and
cross-owner contracts/assurance. They then revalidated relevant prior identities.
The coordinator read decisive sources, reconciled overlaps by structural cause and authored
this single collection. An additional independent challenge assesses the stable draft.

The newer code-intelligence capabilities supported distinct evidence questions:

- Parser-owned command discovery, request schemas and scoped text/syntax/symbol searches
  nominated declarations and consumers. Guarded source addresses and exact range expansion
  supplied the original bytes used for diagnosis.
- Source associations distinguished generated ownership, document connections and explicitly
  declared plan scope. After explicit preparation of the current document interpreter,
  `completion.rs` associations returned successfully. An undeclared owning-plan association
  was left undeclared; source membership was not treated as plan authority.
- An explicitly prepared immutable native snapshot supplied typed declaration/reference facts.
  For example, references to `workflow::contract` identified consumers to inspect for DA-039.
  The reference response retained truncation and omitted records; no call-site census or
  complete absence claim was inferred.
- Separate reviewers and exact source follow-up challenged semantic conclusions. A reference,
  similarly shaped function or observed type relationship did not itself establish a principle
  violation.

Native preparation was explicit:
`pse tools prepare native-analysis`, `pse tools prepare xtask` and
`pse code session prepare --backend native-snapshot --project .` completed successfully.
The prepared world used ra_ap_hir/ide 0.0.358, default features, target
`x86_64-unknown-linux-gnu`, no build/procedural macros and no test context.
Its original preparation identity was `01a1993e-3a51-75e4-961d-9c970103c260`.
This was analysis preparation, not a correctness build or compiler acceptance experiment.

Several limits affected actual use and are retained as evidence limits:

- An initial exact semantic expansion correctly refused before native preparation.
- An initially ambiguous multi-seed relationship request refused nonunique entity seeds.
  Narrower seeds subsequently reached a provider I/O failure. That graph query did not support
  any negative conclusion; the audit used scoped searches and original source instead.
- Semantic symbol search returned a similarly named type rather than the intended function;
  the coordinator corrected the source position before using references.
- An early association lookup lacked a current prepared document interpreter. Explicit
  preparation resolved that availability limit; it did not retroactively validate the earlier
  partial response.
- Outline record limits did not bound wide literal payloads in a large source file. Reviewers
  switched to source ranges rather than treating a small declaration budget as a small byte
  response.
- Incorrect range syntax and a mistaken source path produced explicit failures and were
  corrected before evidence was used.

These observations explain how the tools helped and where fallbacks were needed. They do not
qualify the complete code-intelligence product or establish a measured efficiency improvement.
No rustdoc acquisition, compiler-facts capture or affected-test execution was needed to settle
the adopted source diagnoses.

## Revalidation and lineage

Each identity has one primary home. Topic pages distinguish current source diagnoses, partially
refreshed observations, unresolved inquiries and inherited dispositions.

| Current disposition | Identities and primary evidence |
|---|---|
| Revalidated with narrower claims | [DA-003, DA-006, DA-048, DA-049](semantic-model-and-authority.md); [DA-007, DA-039, DA-050, DA-051](structure-and-invariants.md); [DA-020, DA-053, DA-054](boundaries-and-composition.md); [DA-021](local-reasoning-and-assurance.md); [DA-043](development-tooling.md); [DA-055](execution-fit.md) |
| Partially refreshed, with unexamined footprint excluded | [DA-059](boundaries-and-composition.md); [DA-047](study-execution-and-database-boundary.md) |
| Earlier broad diagnosis no longer established; bounded question retained | [DA-034, DA-035, DA-056](execution-fit.md); [DA-058](development-tooling.md); [DA-052, DA-060](boundaries-and-composition.md) |
| Named historical premise retired or corrected | [DA-015, DA-019](boundaries-and-composition.md); [DA-023, DA-024](local-reasoning-and-assurance.md); [DA-046](study-execution-and-database-boundary.md); [DA-057](development-tooling.md) |
| Inherited scope record | [DA-025](execution-fit.md#da-025) |
| Decision-closed; not reopened | [DA-014, DA-018](boundaries-and-composition.md); [DA-022, DA-030](execution-fit.md) |

Other earlier resolved and merged IDs remain in the topic pages as lineage pointers, with
their original evidence date or inherited status. They are not fresh qualification claims.
Retired IDs are never reused. Removal of obsolete detailed text is deliberate: Git preserves
the prior explanation rather than leaving it beside current claims as an apparent backlog.
This audit changes no implementation-plan completion state.

## Coverage and unresolved boundaries

The inquiry crossed model admission and compiler publication, original numerical quality,
initialization/continuation, solve routing, fitting and derived products, local and durable
studies, result inspection/export, canonical interpretation and retention, host process
ownership, diagnostics, generators and engineering investigation. It followed representative
journeys and changed premises beyond the diff; it was not a file census.

The full layered standard informed those questions. Meaning, identity and authority were
examined through model selection, derivations, completion and export. Composition and locality
were examined through routing, study execution and product/host dependencies. Execution,
resources and failure were examined through capture exclusion, leases, custody, parser bounds
and error settlement. Library boundaries were considered through class-native routing,
structural witnesses, typed provider selection, Arrow admission and canonical store ownership.
No alternative library selection or applicability research was required for these diagnoses.

For process-simulator principles, the streams inspected quantity admission and resolved syntax;
structural/well-posedness witnesses; domain guards and tolerance use; initialization,
continuation and class-based routing; original quality, derived permission and result scope.
This supplies qualitative architectural evidence across the profile. It does not establish
thermodynamic property validity, conservation or derivative accuracy, numerical convergence,
reference parity or any of PS-G1–PS-G3 as an executed acceptance gate.

Sampled assurance was read using the test-assessment companion. Test-only findings retain one
home in the [test account](../test-audit/README.md); this refresh does not update that account's
older baseline or duplicate its DTA findings.

Material limits include:

- no solver runs, numerical counterexample fixture, store fault injection or timing campaign;
- no complete macro-expanded call graph, all-feature/target analysis or reachability proof;
- no exhaustive foreign-key/invariant enforcement inventory or dormant-code deletion proof;
- no complete solver ABI/status/derivative qualification or every qualifier/export consumer;
- no cold-checkout guard execution in each actual agent runtime;
- no whole-architecture sentence audit, test-population audit or complete alignment certificate.

## Independent challenge and publication

An independent read-only reviewer challenged the stable draft against decisive source.
The adopted corrections were:

- DA-061 now names `Cursor::binary` and distinguishes parser recursion protection from an
  explicit AST-ancestry promise.
- DA-052 is downgraded: distinct nested formats with owner-local checks do not establish
  duplicated authority.
- DA-062's illustrative deviation must be finite and representable within the variable budget.

The challenger independently supported the three new mechanisms, DA-049's caller-supplied
usability gap and DA-039's diagnostic losses. DA-051's unsettled transactional premise and
DA-059's generation-policy inquiry remain explicitly bounded. This is independent source
challenge, not product qualification or selected remediation.

The audit-owned native session was stopped through `pse code session stop`.
The original preparation identity matched, the service was gone and settlement was complete.
No scientific service was started by this audit.

Documentation publication checks completed with zero failures against the zero target:
`just turn-end` (ADR index and formatting), `just docs` (HTML and scoped search), and
`git diff --check`. These checks validate publication, not runtime or scientific behavior.
Product tests and numerical/performance campaigns were not run.

## Reading and authority

The [integrated account](system-architecture.md) connects causes without duplicating finding
ownership. Primary topic pages are:

- [Scientific meaning and authority](semantic-model-and-authority.md)
- [Structure, invariants and diagnostics](structure-and-invariants.md)
- [Boundaries and composition](boundaries-and-composition.md)
- [Execution fit](execution-fit.md)
- [Local reasoning and assurance](local-reasoning-and-assurance.md)
- [Development tooling](development-tooling.md)
- [Study/database boundary](study-execution-and-database-boundary.md)

A subsequent review should cite this collection's exact publication revision and finding
anchors. Architecture owns contracts, ADRs own rationale, reviews own solution arguments and
plans own scheduled work. The present diagnosis selects no remedy and authorizes no production
change.
