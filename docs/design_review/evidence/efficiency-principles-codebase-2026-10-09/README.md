# Codebase efficiency review: supporting evidence

The [principal review](../../reviews/design_review_efficiency-principles-codebase_2026-10-09.md)
owns the combined architectural judgment, finding identifiers and evidence limits. These
bounded inquiries support that judgment; they are neither implementation plans nor competing
finding-status ledgers. Recommendations remain Proposed until adopted through plan creation.

The inspected product baseline is `4c24721e691187e1a5b28398b29722fbde671da8` on `main`.
During inquiry startup, the maintainer committed the previously dirty graph/hash implementation
over `46545b2ad3e2999b4335692ac49af6091438c3fc`. Reviewers were informed and inspected the
resulting source. Documentation added for this review does not change the product baseline.

## Independent inquiries and reconciliation

| Supporting assessment | Scope and role |
| --- | --- |
| [Build and validation](build-and-validation.md) | Dependency/features/profiles, native installation and artifact admission, test selection, fixtures, qualification applicability, resource-history coordination and generation |
| [Preparation and mathematics](preparation-and-mathematics.md) | Authored occurrences, selected checking, physical context, Salsa, shared mathematical preparation, value rebinding and conditional block representations |
| [Execution and results](execution-and-results.md) | Native jobs and study lanes, fitting response assembly, result encoding/serving, analysis staging, retention and Python result boundaries |

Three independent reviewers supplied bounded source assessments. The coordinator inspected
decisive occurrence, selection, conditional-plan, result-encoding, fitting-loop, qualification
projection and analysis-retirement sources, reconciled their boundaries, and commissioned a
fresh principal reviewer. Supporting identifiers BF/PP/EX are local to these inquiries; only
the principal review's finding identifiers are intended for later disposition.

Current indexed description settlement, retained supplier topology, flow-policy lookup,
exact preparation equality, native admission/drain, concurrent study lanes, immutable native
generations and producer/deployment separation are preservation constraints. Older diagnoses
against their replaced implementations must not be transferred to this baseline.

## Columnar boundaries inspected by the coordinator

The registry in `crates/pse-schema/src/lib.rs` is retained through `OnceLock`; it is not
reassembled on each lookup. `ContractArena` in
`crates/pse-relations/src/resolved_contract.rs` shares resolved schemas and contract closure.
Prepared contexts and `ImplementationCache` reuse construction. These inspected owners
do not support a claim of per-call global registry/schema reconstruction.

`FieldCheckedBatch` in `crates/pse-relations/src/columnar.rs` retains local field admission
and actual allocation ownership. `readmit_context` reuses same-owner admission; a different
context evaluates its actual native obligations. Projection/gather/filter do not silently
promote local field validity into global uniqueness or completeness. Escaped arrays retain
their actual buffers and allocation owners. These distinctions must survive optimization.

An opportunity remains in selected gather: `take_source_copies` allocates a count vector over
the entire source population, and `take_reserved` forecasts whole-source decode extent times
copy multiplicity. `workflow/connected_results.rs` calls it for a nonempty selected subset;
`workflow/result_projection.rs` also uses it for full trajectory permutations, where whole-source
scope is appropriate. No ordinary exhaustion threshold or latency share was established.
Any narrower forecast must preserve duplicate multiplicity, variable-width/nested aliases,
checked bounds/nulls, zero-column batches, source ownership and admission before allocation.
The coordinator did not establish a production caller for `filter_reserved` in the inspected
runtime/engine/authoring source. Its forecast alone is not a principal finding.

The preliminary claim that ordinary scalar paging repeatedly decodes a block once per 64
matching cells was retracted. Ordinary exact entity/field/partition scalar keys are unique.
Complete scalar-cell index reconstruction after block admission remains a narrower opportunity,
not evidence of that paging amplification.

## Executed input-projection probe

**Tested:** the [probe](input-scope-probe.py) calls the actual
`scripts.validation_scope.input_identity` with synthetic before/after file digests and
environment values. It changes no repository files and starts no compiler, solver or service.

```bash
scripts/pse-env --resource-class light -- .venv/bin/python -B docs/design_review/evidence/efficiency-principles-codebase-2026-10-09/input-scope-probe.py
```

On the reviewed baseline, Python 3.14.7, input-scope version 2, the command exited 0.
The [retained output](input-scope-probe.json) uses `true` when the changed input changes
the projected identity. Five omitted source paths and six native/tool environment selectors
were undetected in both product scopes; crate source, `native_tests.py` and `IPOPT_DIR`
positive controls were detected in both scopes. Baseline: every consumed input should change
applicability when changed; the omissions violate that expectation. This is not a demonstration
of actual successful-receipt reuse or a scientific-result error.

The first invocation exited 125 before execution because host admission had no capacity.
`just ready` then passed its skills-sync and doctor steps against the zero-failure baseline;
the ordinary retry succeeded. No admission bypass was used. An earlier reviewer diagnostic
outside the pinned wrapper is superseded by this retained pinned execution.

## Qualification limits

The user-cancelled feature matrix remains cancelled. Its local-only receipt is
`build/assessment/20261009T222854.275137Z-features-powerset-2890517-689811/summary.md`:
the combination command was interrupted after 1,539.9 seconds, with the started `207/313`
entry in its log; the subsequent no-default command was interrupted after 7.5 seconds.
This was bounded feature compilation enumeration, not solve timing or full product testing.
The report has `input_coverage: false` and is correctly refused for unchanged-input reuse.

No full suite, native campaign, Python product journey, feature-matrix continuation or new
performance campaign was run for this review. Source establishes work amplification;
quantitative latency, RSS and throughput benefits remain unmeasured. Existing measurements
retain their original fixture, artifact, command and scope. In particular, preparation-key
hashing and complete warm preparation are different operations and their timings do not
establish a preparation speedup.
