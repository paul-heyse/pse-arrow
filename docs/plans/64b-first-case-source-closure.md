---
title: "Plan64b first-case scientific source closure"
status: in-progress
date: 2026-11-02
doc_role: plan
doc_owner: docs/plans/64b-native-scientific-intake.md
doc_topics: [current-work, design, validation]
---

# First-case scientific source closure

This is B01's bounded source-selection account for the two first cases. It does not
replace the package state in [64b](64b-native-scientific-intake.md) or H01's full finite
population. **Interface-checked:** the declarations and source notes below at
`7cce20ef4`; no ordinary scientific case, acquisition or publication is qualified by
this inventory. Source roles are transitive admission meaning, not editable badges.

## Selected cases and exact outstanding inputs

| Consumer | Actual selected input/owner | Existing source/eligibility | Ordinary-source decision and remaining work |
|---|---|---|---|
| Recycle and exchanger | Selected pressure rows | Original RPP4 pressure rows are oracle-only | Select acquired Poling Wagner2.5,5 rows for both CAS identities through the exact Chemicals artifact/revision recorded in `ordinary_bt.wagner_source`. A new authored2.5,5 evaluator preserves the actual coefficients' meaning; using them with the old3,6 evaluator is forbidden. This is explicit changed science, not fixture-default parity. |
| Recycle BTIdeal/BTLiquid | Liquid Cp | Original IDAES liquid rows are oracle-only | Select the existing independently published Perry seventh-edition DIPPR100 rows for both species. Preserve their original units and envelopes; consuming cases must explicitly disposition extrapolation rather than widening those bounds. |
| Recycle and exchanger | Vapor Cp | Original RPP4 gas rows are oracle-only | Select acquired FeOS Poling DIPPR100 rows for both species. The native adapter retains original kmol units and uses ordinary physical admission for mol normalization. The source artifact supplies no empirical envelope: `feos_unknown` preserves typed unknown applicability. Selected consuming bounds are not source validity. |
| Recycle | Energy datum/phase difference | Existing IDAES reference and translation are oracle-only | Use zero ideal-gas enthalpy at298.15K as an explicit ordinary convention. Liquid enthalpy offset uses acquired CRC298.15K vaporization values, not IDAES formation constants. No liquid entropy is invented. |
| Exchanger BT_PR | Benzene and toluene EOS critical values | Existing selected critical rows are oracle-only; existing published Poling bank covers benzene only | Select both acquired Yaws critical rows via `ordinary_bt.critical_source`. Preserve unknown empirical applicability and keep EOS critical data distinct from vapor-pressure fitted critical coordinates. |
| Exchanger BT_PR | Unlike interaction | Existing predictive-zero rule has oracle lineage | `ordinary_bt.zero_unlike` explicitly declares the ordinary prediction policy, retaining missing versus predicted-zero. It remains a prediction with unknown applicability, not a measured coefficient0. |
| Exchanger | Gas enthalpy/entropy datum | Existing formation and entropy translations are oracle-only | Use zero ideal-gas enthalpy/entropy at298.15K under the native stock convention, with the selected ideal-gas Cp integrals and PR residual contribution. Do not expose the removed oracle-coordinate observables as ordinary products. |
| Exchanger | SSLW area/material/pressure factors, cost index and scoped extrapolation | Existing authored costing definitions, with the campaign explicitly permitting unknown area and pressure extrapolation | Keep the exact existing sources, currency/base period, hot-side pressure and allowance selections during complete closure discovery. The present critical-data adapter establishes none of this cost population's ordinary use or a qualified equipment purchase price. |

The actual physical definitions remain the ideal recycle and co-current PR/relational-density
exchanger. Their test specifications, synthetic sampled feeds and expected values are
reference witnesses. Neither removing `fixture` nor changing the data role creates an
ordinary source path. A replacement source must pass contextual admission and preserve
the selected physical definition or explicitly identify changed science.

## Evidence and bounded source acquisition

The existing sources are declared in
[oracle parameter banks](../../packages/reference/data/oracles/idaes-2.13/models/parameters.pse),
[BTIdeal](../../packages/reference/seed-data/models/bt-ideal.pse),
[BT_PR](../../packages/reference/seed-data/models/bt-pr.pse),
[EOS selections](../../packages/reference/seed-data/models/eos-data.pse) and
[seed source notes](../../packages/reference/seed-data/sources.md).
The ordinary benzene candidate is separately declared in
[Poling parameter banks](../../packages/reference/data/poling2000/models/parameters.pse)
and its [source notes](../../packages/reference/data/poling2000/sources.md).
Its committed Parquet metadata names the source row and normalization. This is retained
source-attribution evidence, not a fresh verification of the original book.

The original book URL returned HTTP502 and NIST WebBook pages were inaccessible
(direct HTTP403). These failed routes did not establish scientific absence. Further
source work acquired the exact FeOS and Chemicals artifacts now retained in
[ordinary BT source artifacts](../../packages/reference/data/ordinary-bt/sources.md).
Their SHA256 values were checked on acquisition. The source entities in that package
own exact artifact revisions, row coordinates and hashes. This supplies the numerical
input population while preserving explicit changed-science and applicability limits.

## Implemented adapter boundary and handoff

`workflow::scientific_intake::feos_poling2000_bt_caloric_delta` reconciles the actually selected two-species gas Cp
artifact into the ordinary typed dataset against an exact parent. The package provides
ordinary `Ideal`, `Liquid` and `PR` bindings separately from unchanged oracle fixtures.
The replaced one-row Poling transcription prototype and its tests were removed after
the actual two-species FeOS adapter and ordinary source admission passed. The selected
adapter neither authenticates original publications nor changes provenance roles.
Guarded publication, fresh revision readback and complete consuming cases remain
obligations of B03/B04. Source population and definitions alone do not establish them;
B04 and the native delivery stay open until their actual acceptance runs succeed.
