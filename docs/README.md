# pse-arrow

A process systems engineering core in Rust. Typed relations remain model authority;
computation placement follows
[blueprint D10](authoritative_design/blueprint.md#d10-choose-computation-per-operation-and-preserve-relational-authority):
typed Rust and MathIR for semantics, Salsa for bounded incremental reuse, graph
libraries for admitted projections, Arrow for columnar boundaries, DataFusion for
relational phases, and Delta Lake for selected durable products.
It is a clean-room re-implementation
of core IDAES-PSE capabilities, parity-tested against `idaes-pse 2.12.0`, and it
is **not affiliated with IDAES** — see
[Relationship to IDAES](relationship-to-idaes.md).

[Plan 13](plans/13-rust-computation-architecture.md) owns current implementation.
The [W19 repair checkpoint](plans/13-w19-repair-checkpoint.md) separates implemented
replacement/deletion work and passing isolated controls from incomplete functional
qualification and unrun performance work.

The distribution is `pse-arrow` on PyPI (the name `pse` is taken); the import
name is `pse` and the crate prefix is `pse-`. Everything is licensed
`MIT OR Apache-2.0`.

## Where authority lives

| Question | Authority |
|---|---|
| What is the design? | [`authoritative_design/blueprint.md`](authoritative_design/blueprint.md) — one file, stable section numbers, revised in git |
| Why does it read that way? | [`adr/`](adr/) — one record per decision, with the design principles §H fields |
| What was deliberately deferred? | [`adr/register.md`](adr/register.md) — one row per deferral, with its trigger and next check |
| How is the work sequenced? | [`plans/`](plans/) — living until done, then an appended Outcome |
| Is the design any good? | [`design_review/reviews/`](design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md) — **evidence, not authority** |
| What can the libraries actually do? | [`capability-maps/`](capability-maps/arrow-rust.md) and their [evidence](capability-maps/evidence/) — probes, lockfiles and captured outputs |
| What runs in CI, and who answers when it goes red? | [`dev/ci.md`](dev/ci.md) |

When the code and a plan disagree, the code is what runs. When the code and the
blueprint disagree, one of them is a bug, and the ADR says which.

## Reading order

1. **Orientation** — this page, then [Relationship to IDAES](relationship-to-idaes.md).
2. **The design** — blueprint [§0](authoritative_design/blueprint.md) (purpose and
   reading guide), then **§2** (the fourteen binding decisions D1–D14), then the
   section §0.3 points you at for what you are about to do.
3. **The decisions** — the [ADR index](adr/). ADR-0004 … ADR-0017 are
   D1–D14 one record each; ADR-0018 … ADR-0038 are the repository, supply-chain
   and process decisions.
4. **The evidence** — the [capability maps](capability-maps/arrow-rust.md) before
   believing any claim about what Arrow, DataFusion or a supporting crate does at
   the pinned version. Every claim in this book carries a design principles §D evidence
   label; `Proposed` means exactly that.

## Evidence vocabulary

Every claim in these documents is labelled with the design principles §D vocabulary —
`Proposed`, `Interface-checked`, `Implemented`, `Tested`, `Measured`,
`Formally established`. The labels describe different claims, not one ladder: a
measured implementation can still be incorrect, and an interface-checked design
can still need substantial engineering. The definitions are in
[`design_review/design_principles/core/design-principles.md`](design_review/design_principles/core/design-principles.md)
§D.
