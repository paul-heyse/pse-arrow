# pounce

Library family account from the first applicability assessment (2026-10-27). Each page keeps its reusable types separate from its current applicability; this index routes to them.

| Page | Types | Current applicability |
| --- | --- | --- |
| [POUNCE presolve (structural decomposition)](presolve-and-fbbt.md) | `pounce-presolve/structural-decomposition`<br>`pounce-nlp/fbbt-tape` | CBO-023, CBO-024 |
| [POUNCE (sens-core, qp, convex, nlp TNLP, cinterface)](solvers-and-sensitivity.md) | `pounce/engine-agnostic-parametric-sensitivity`<br>`pounce/convex-qp-and-cone-form`<br>`pounce/tnlp-and-c-abi` | CBO-029, CBO-031 |

The reusable `pounce/engine-agnostic-parametric-sensitivity` type is retained without a current
leverage match after CBO-032 retired on its corrected premise; the implementation already uses
that surface. The comparative review preserves the local rationale.
