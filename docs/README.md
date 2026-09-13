# pse-arrow

An Arrow-native process systems engineering core in Rust: typed relations as the
only model authority, a relational math IR, DataFusion-based inference, native
NLP solving, and a generated Pyomo backend. It is a clean-room re-implementation
of core IDAES-PSE capabilities, parity-tested against `idaes-pse 2.12.0`, and it
is **not affiliated with IDAES** — see
[Relationship to IDAES](relationship-to-idaes.md).

The distribution is `pse-arrow` on PyPI (the name `pse` is taken); the import
name is `pse` and the crate prefix is `pse-`. Everything is licensed
`MIT OR Apache-2.0`.

## Where authority lives

| Question | Authority |
|---|---|
| What is the design? | [`authoritative_design/blueprint.md`](authoritative_design/blueprint.md) — one file, stable section numbers, revised in git |
| Why does it read that way? | [`adr/`](adr/README.md) — one record per decision, with the charter §H fields |
| What was deliberately deferred? | [`adr/register.md`](adr/register.md) — one row per deferral, with its trigger and next check |
| How is the work sequenced? | [`plans/`](plans/README.md) — living until done, then an appended Outcome |
| Is the design any good? | [`design_review/reviews/`](design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md) — **evidence, not authority** |
| What can the libraries actually do? | [`capability-maps/`](capability-maps/arrow-rust.md) and their [evidence](capability-maps/evidence/README.md) — probes, lockfiles and captured outputs |
| What runs in CI, and who answers when it goes red? | [`dev/ci.md`](dev/ci.md) |

When the code and a plan disagree, the code is what runs. When the code and the
blueprint disagree, one of them is a bug, and the ADR says which.

## Reading order

1. **Orientation** — this page, then [Relationship to IDAES](relationship-to-idaes.md).
2. **The design** — blueprint [§0](authoritative_design/blueprint.md) (purpose and
   reading guide), then **§2** (the fourteen binding decisions D1–D14), then the
   section §0.3 points you at for what you are about to do.
3. **The decisions** — the [ADR index](adr/README.md). ADR-0004 … ADR-0017 are
   D1–D14 one record each; ADR-0018 … ADR-0038 are the repository, supply-chain
   and process decisions.
4. **The evidence** — the [capability maps](capability-maps/arrow-rust.md) before
   believing any claim about what Arrow, DataFusion or a supporting crate does at
   the pinned version. Every claim in this book carries a charter §D evidence
   label; `Proposed` means exactly that.

## Evidence vocabulary

Every claim in these documents is labelled with the charter §D vocabulary —
`Proposed`, `Interface-checked`, `Implemented`, `Tested`, `Measured`,
`Formally established`. The labels describe different claims, not one ladder: a
measured implementation can still be incorrect, and an interface-checked design
can still need substantial engineering. The definitions are in
[`design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md`](design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md)
§D.
