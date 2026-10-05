---
title: "Plan 25b applicability implementation review"
date: 2026-10-01
standard: core-3.3/process-simulator-1.3
tier: change
purpose: conformance
evidence: Implemented
decision: Revise
---

# Plan 25b B4 independent implementation review

Baseline: dirty `main` at HEAD `8be7837f97ddaacf3b5c7fba44ac2faa90642c3d`, inspected 2026-10-01 during concurrent integration. This is a read-only, bounded implementation review against Plan 25b B4, proposed ADR-0141, ADR-0140 and the Sept 30 scoped target review. No builds, tests, formatters, repository edits or commits were performed. Evidence is **Implemented/static inspection**; none of the new controls below is **Tested** by this reviewer. Full product/scientific qualification remains Plan 25k.

Current disposition belongs in Plan 25b. Findings were sent to the coordinator as discovered; some repair work may already be in progress. Locations describe the inspected snapshot and may move during repair.

## Material findings

### IR25B4-01 — A union can waive a winning alternative's independent outside-region obligation

Locations: `crates/pse-model/src/applicability.rs:155-178`.

`effective_classification` gives UnknownEvidence precedence over OutsideRegion across dependencies. `observe` then records every union alternative with `required=false` and promotes only the winner's direct dependencies to required. Trigger a union with one alternative whose own predicate is false and whose required dependency is unknown. Give the union/dependency allow-unknown permission, but no extrapolation permission. The union is unknown, its chosen alternative's own outside observation is nonrequired, and the unknown dependency is permitted. The assessment admits although a required outside obligation has no permission. A nested union also fails to promote dependencies of its nested winning alternative because child observation receives `required=false`.

This conflicts with ADR-0141's independent unknown/outside obligations and retention of required dependencies. Preserve winning-path requirements recursively, including its own region obligation, while keeping losing alternatives informational. Close with separate flat and nested controls: allow-unknown alone refuses, adding extrapolation admits, and observations retain correct required flags. The coordinator has already assigned repair of this finding.

### IR25B4-02 — Implicit evidence capture hoists direct record reads out of runtime branches

Locations: `crates/pse-modeling/src/specialize/rewrite.rs:577-589`; `crates/pse-modeling/src/specialize/value.rs:544-550`; `crates/pse-modeling/src/scientific_selection.rs:150-165`; `crates/pse-modeling/src/specialize/applicability.rs:253-270`; `crates/pse-compiler/src/typed_math.rs:1047-1078`.

Runtime conditionals rewrite both branches. Direct numerical entity attribute reads from both branches are captured in one function-wide direct-record set. Missing/bindable claims for those records are appended to the function's applicability uses and evaluated before its numerical body, independently of branch demand.

Trigger `fn choose(x:Scalar,s:fit,t:fit)->Scalar = if x>0 then s.value else t.value`, where s is explicitly unrestricted and t has unknown evidence. At x>0, the inactive t claim still refuses without allow-unknown. Existing active-branch controls call separate `law` functions; they do not cover direct record reads in a shared function body. Preserve read-level active-path placement, including dependencies; a declared unconditional function claim remains unconditional. Close with both branch directions and permitted/refused unknown controls, asserting only active direct reads produce demanded observations.

### IR25B4-03 — Direct scientific record reads outside a function bypass the evidence gate

Locations: `crates/pse-modeling/src/scientific_selection.rs:150-165`; `crates/pse-modeling/src/specialize/value.rs:544-550`; `crates/pse-modeling/src/specialize/rewrite.rs:288-311`; `crates/pse-modeling/src/specialize.rs:1878-1885`; applicability invocation `crates/pse-modeling/src/specialize/functions.rs:735-736`.

`record_numeric` records only into Function scopes. Direct record attributes in a test expectation, let/member expression or equation have only the surrounding Instance scope, and applicability fallback runs only during finite function specialization. The static attribute read becomes a literal without an applicability obligation.

Trigger a declared scientific fit with an Applicability attribute bound to an unknown claim and evaluate `expect fit_record.value == 1 tolerance ...` directly. A function reading the same attribute would gain an implicit claim; the direct expression does not. This is an alternative production path around strict unknown-evidence admission, not just missing result qualification. Close with direct top-level/member/equation controls, preserving branch demand and refusing absent/bind-inapplicable claims unless a named permission is present. Treat ordinary structural records separately using admitted scientific meaning rather than scientific-name dispatch.

### IR25B4-04 — Successful generated observations drop instance and required/alternative attribution

Locations: `crates/pse-runtime/src/workflow/modeling/results.rs:1087-1100`; `crates/pse-schema/src/catalog/modeling.rs:1232-1285`; `crates/pse-modeling/src/specialize/applicability.rs:575-594`; `crates/pse-math/src/assembly.rs:792-804`.

Assembly observations correctly acquire actual instance identity, and `Observation.required` distinguishes union alternatives from required claims. `applicability_checks` drops both values. The generated relation has neither field; its key uses run/step/sample/claim.call/claim source/kind. A bound claim call hash includes source form/claim IDs, rewritten argument expressions and static records, but not actual model instance. Two instances using the same form/record can therefore emit equal generated keys at different physical inputs while their original observations are distinct. E3 also cannot distinguish an informational unknown union alternative from a required permitted unknown claim using the exported rows alone.

Close with lossless successful transport of instance and required status, a key/observation identity that distinguishes actual demanded occurrences, and roundtrip/multi-instance controls. Preserve original `call_id` meaning if deriving a separate observation key. Unknown reason and full permission target/scope lineage are likewise absent from success rows, although permission IDs may support declaration lookup; assess that against E3's consumed contract. The existing roundtrip control sets an instance on its input observation but never asserts instance survives.

## Positive source assessment and earlier repairs

- Unknown and explicit unrestricted remain separate representations. Permissions independently cover unknown and outside outcomes; exact record permissions require all claim records and family permissions match the declared owner.
- Required dependency observations outside a union are kept separate; mixed dependencies do not collapse to a single permitted result.
- Bindings retain canonical physical input types, source evidence, selected record IDs, form/call attribution and interval coverage identity. Arbitrary predicates cannot use the interval-endpoint shortcut; endpoint reduction requires an explicitly declared interval and equal nonaxis arguments.
- Mathematical/form domains remain separate stages and fail independently of data-use permission. Applicability stage tokens participate in dependency/demand retention; differentiation skips numeric differentiation of value-only evidence stages while retaining their demands. Cancellation and inactive nested function branches have relevant controls in source, not executed evidence here.
- Selection closure retention now follows consumed contexts and direct owned context attributes; instance captures avoid nested ownership leakage. Finite functions frame their scoped consumed selections after argument/body/guard/applicability work. These changes address the old IR25B-01/02 source mechanisms, but their focused executed closure/identity controls remain the coordinator's acceptance obligation.
- Refusal diagnostics retain full typed observations and attach instance attribution through MathError::Instance. Success transport has the separate IR25B4-04 gap.

## Coverage and limits

Inspected model assessment/truth tables, modeling claim admission and specialization, selection collectors/static reads, finite-function/instance identity, compiler evidence lowering and point checks, mathematical stage execution/demand/differentiation/branch handling, factorable evidence handling, runtime point/trajectory assessment, success/refusal projection, registry declarations, relevant migrated source claims, and the new compiler/runtime controls.

No end-to-end solver run, library documentation/API assessment, static/hygiene check, performance campaign or full qualification was attempted. The brief's earlier 8 model + 4 math passing controls preceded the latest refusal patch; 9 source-admission controls were failing during B3 integration. Those are supplied checkpoint facts, not this review's executed receipts. Current root-owned integration tests must establish repaired behavior.

Review result: **Revise for B4 implementation acceptance**, with concrete controls above. Architectural target acceptance is unchanged; static inspection does not establish Tested or integrated qualification.

Current dispositions and correction evidence belong to [Plan 25b](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/25b-scientific-knowledge-and-applicability.md#execution-checkpoint).
