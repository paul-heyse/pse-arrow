---
title: Post-Plan44 optimization inquiry and probe evidence
doc_role: evidence
doc_topics: [design, numerical-execution, diagnostics, performance]
doc_owner: docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md
---

# Post-Plan44 optimization evidence

Supporting material for [the principal review](../../reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md), which owns the integrated findings, target and judgment. Inspected production baseline: `91fd915134f99edeb8ef00368096f84761960d95`, 2026-10-18. Proposed work has no implementation disposition yet; a future authorized plan will own it. Old Plan44 status remains unchanged.

| Material | Role and evidence boundary |
| --- | --- |
| [Preparation inquiry](preparation-inquiry.md) | Source/library assessment of local admission, source maps, compiler retention, physical dependencies and mathematical products. No new runtime probe by its author. |
| [Native inquiry](native-inquiry.md) | Exact-release solver contracts, retained native observations, consumed adjoints, sparse preparation, quadrature and checkpoint investigations. No new native runtime probe by its author. |
| [Results inquiry](results-inquiry.md) | Current snapshot, promotion, evidence joins, graph publication, physical publication and relational/library alternatives. Retained timings remain once-only and inclusive. |
| [Independent assessment](independent-assessment.md) | Fresh reviewer independently read decisive production paths and challenged the combined target. It did not run tests or edit production. The principal records reconciliation. |
| [Public local probe](https://github.com/paul-heyse/pse-arrow/blob/f83bac0399d1d815b6ce91b692158b965fa054c7/docs/design_review/evidence/post-plan44-pipeline-optimization-2026-10-18/local-pipeline-probe.py) | Eight states, five sample times, Diffsol/Forward, ordinary .001 accuracy. Five independently checked joined simulations per invocation; one original and one corrected paged capture. No durable fixture or service. |
| [Local summary](local-pipeline-summary.json) | Both sets of once-only stage clocks, invocation outcome/coverage, selected per-operation counters and resource reports. Metrics retain their partiality; no summed full-run counts or speedup ratio. |
| [Paged raw observations](local-pipeline-observations.jsonl.gz) | Gzip JSONL of the second invocation, including catalogue pages, copied snapshots and actual returned invocation documents. Contains coverage/loss/generation facts. The initial invocation's clocks survive in the summary; it read only one catalogue page. |
| [Membership prototype](response-membership-probe.rs) / [counts](response-membership-results.jsonl) | Standalone std-only exact membership kernel, three shapes, positive cases and twelve equality/adverse checks each. Tests only membership; not full analysis admission, lifecycle or latency. |

## Executed checks

All commands ran from the checkout through its environment wrapper and pinned tools. The existing private test-resource registry was selected with `PSE_TEST_RESOURCE_STATE=/tmp/pse44-test-resources-2hulIR`; the shared registry and other owners were not altered. Native environment selection used the installed fast editable correctness extension, which reported `force-validate`, `native-solvers`, `solver-highs`. No full wheel/install, producer/environment proof or native rebuild was needed. The wrapper's small environment setup reported a successful existing dev build check; this is not a new product qualification claim.

```bash
PSE_TEST_RESOURCE_STATE=/tmp/pse44-test-resources-2hulIR \
  scripts/pse-env --native=solver,klu,isolation,uno,petsc --resource-class exclusive -- \
  .venv/bin/python \
  docs/design_review/evidence/post-plan44-pipeline-optimization-2026-10-18/local-pipeline-probe.py

PSE_TEST_RESOURCE_STATE=/tmp/pse44-test-resources-2hulIR \
  scripts/pse-env --resource-class compile -- \
  rustc --edition=2024 -C opt-level=2 \
  docs/design_review/evidence/post-plan44-pipeline-optimization-2026-10-18/response-membership-probe.rs \
  -o /tmp/post44-response-membership-probe

PSE_TEST_RESOURCE_STATE=/tmp/pse44-test-resources-2hulIR \
  scripts/pse-env --resource-class light -- /tmp/post44-response-membership-probe
```

**Tested:** two successful Python invocations, ten usable simulations total, zero assertion failures against a zero baseline. The initial driver was then changed only to paginate/observe the diagnostics correctly; it is not a second timing sample of an identical measurement arrangement. Authored checks and independent original-coordinate values passed. Alias release plus optional program-cache clear reported zero pool charge before normal runtime close in each invocation. The temporary spill directories were removed by their normal context owner.

**Tested / Measured counts:** Rust compilation and execution exited zero, no compile warnings, all three membership shapes passed and all twelve equality/adverse checks per shape passed. The final source/output uses `sample_times` for the time-grid count, correcting an initially ambiguous `outputs` field. The corrected source was recompiled and rerun. No production graph implementation or timing comparison was performed.

The coordinator inspected decisive source in addition to the inquiries. Inquiry source locators identify implementation evidence; architecture/decision references use stable section/ADR identifiers. Context7 and upstream sources are linked in each inquiry, with exact-pin mismatches identified. No evidence is drawn from the stale utilization catalog/capability maps.

## Retained Plan44 observations consumed

The review read `build/p44-resweep-t2zaldrm/cases`, `build/p44-density-22s_40tp/cases` and `build/p44-results-resweep-2vhngy5m/cases`, including case audits, observation/terminal-inspection records, memory reports and partial profile metadata. These are checkout-local ignored build artifacts, not promised publication assets. The [retained Plan44 case observations](../pipeline-performance-optimization-2026-10-17/case-observations.json) and [investigation outcome](../../../plans/44e-short-run-investigation-and-optimization.md) supply lasting context. Selected facts and their limits are restated in the principal review so its conclusions do not depend on readers possessing the build directory.

No new durable service run, graph/publication prototype, edit-pressure ladder, JIT, sparse factor owner, adjoint-axis implementation or long simulation was executed. Their proposed discriminators and required supporting implementation appear in the principal review's I01–I10. Read-only source research and simple probes do not establish their numerical or lifecycle qualification.

Only review/evidence and its publication wiring were added. Pre-existing environment/workflow edits were preserved. No experimental worktree was created or merged, no other owner's process was stopped, and no production source/plan/accepted ADR was changed. Documentation publication checks are recorded below when completed; they do not qualify production behavior.

## Publication checks

**Tested, documentation scope, zero baseline:** `scripts/pse-env --resource-class light -- just turn-end` exited zero: ADR index and formatting passed. Foreign-file digest comparison found no change to the six pre-existing dirty/untracked environment/workflow files. New review/support file links resolved locally.

The first `scripts/pse-env --docs --resource-class light -- just docs` failed because the new asset bundle was selected but its explicit file declaration was missing. Adding that declaration in `docs/site.toml` corrected the publication configuration; the rerun exited zero and published 481 chapters with scoped search. This is a repaired documentation result, not a claim that the initial build passed. Product-wide static/integration checks were not rerun for review-only changes.
