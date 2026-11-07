---
title: "Plan69f: Parser startup and argument forwarding"
status: planned
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69f-parser-startup-and-argument-forwarding.md
doc_topics: [current-work, design, libraries, validation]
review_sources: [docs/plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md]
---

# Parser startup and argument forwarding

This companion owns [I01](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#I01)
and supports O01 and O08. A01 owns descriptive catalog metadata, A03 owns the structured
usage error contract, and B02 owns the host-absent launcher. The current entry runs
`presentation_prefix` before strict parsing. That prefix builds an entire clap tree to
locate the forwarding boundary. Strict parsing then constructs its own command. The
review's isolated defer probe establishes library behavior and supplies no CLI latency result.
[Current entry and prefix](../../crates/pse-host/src/cli.rs) ·
[Review evidence](../plan_improvement_reviews/plan69-cli-library-leverage-2026-11-07.md#evidence-and-limits)

## F01 — Strict parse before usage-format recovery

Use clap 4.6.7 with the existing `derive`, `env` and parser features. Call
`Parser::try_parse_from` on the original `OsString` argv under the correct executable
identity. It returns the typed command or a clap error. On success, resolve the parsed
format with the current terminal policy and dispatch through the existing generation and
effect checks. Keep help and version on their successful display routes. Preserve the
boundary and hook refusal protocols. On the ordinary usage-error branch only, pass the
original OS-native outer prefix and the existing environment fallback to the format
selector, then project the original clap error through A03. A format hint from the
error path supplies presentation only. Remove the full-tree prefix build from successful
invocations. Keep the existing error-route scan unless a failing grammar control
requires a bounded correction. A separate lexical parser or clap `ignore_errors` is
outside the strict admission path.
[clap Parser](https://docs.rs/clap/4.6.7/clap/trait.Parser.html#method.try_parse_from) ·
[clap error context](https://docs.rs/clap/4.6.7/clap/error/struct.Error.html) ·
[Current early format policy](../../crates/pse-host/src/envelope/presentation.rs)

Preserve each accepting wrapper's existing trailing positional or `last` grammar.
`Arg::raw(true)` is a candidate for a route that already requires `--`: it captures
the trailing region and configures the positional to accept an open-ended vector of
hyphenated values. Use an OS-native value parser and test literal token preservation.
Where a route currently permits implicit forwarding, keep that contract until a deliberate
compatibility change is selected through the owner. Inspect `clap_lex` or a different
fallback only if the existing usage scan proves materially costly or cannot be verified.
[clap raw arguments](https://docs.rs/clap/4.6.7/clap/struct.Arg.html#method.raw) ·
[OS-native parser](https://docs.rs/clap/4.6.7/clap/builder/struct.OsStringValueParser.html)

Exercise an outer format before a forwarded executable, a child `--format`, non-UTF-8
and comma-bearing arguments, aliases, short bundles, and missing outer values. Check
structured usage when the outer grammar fails and that tokens beyond the forwarding
boundary are not interpreted as outer preferences. Check both executable identities
and the existing help, version and special refusals. Use a small parser fixture and
a prepared passive command to demonstrate the runtime path. Retain the host-absent
route in B02.

## F02 — Measure selective construction after F01

Start from eager construction. The pinned builder `Command::defer` accepts a
`fn(Command) -> Command` that completes the selected definition when built. Derived
deferral uses `#[command(defer = true)]` where the derive surface supports it.
`Command::build` recursively materializes the selected tree for complete consumers.
Keep A01's existing scoped traversal for scoped discovery, and full materialization
for full catalog, schema, or selected generators when required. Compare eager and
deferred construction on one representative prepared lightweight command under
matched conditions. Count avoided construction where practical and measure the
prepared invocation to separate that work from startup noise. Require parity of
scoped and full discovery, schema, help, and any selected completion. Retain eager
construction if the measured gain is immaterial or parity fails. Record the choice
and the actual route and conditions in the package outcome.
[clap builder deferral](https://docs.rs/clap/4.6.7/clap/struct.Command.html#method.defer) ·
[clap derive reference](https://docs.rs/clap/4.6.7/clap/_derive/index.html) ·
[Recursive build](https://docs.rs/clap/4.6.7/clap/struct.Command.html#method.build) ·
[Existing scoped catalog control](../../crates/pse-host/src/catalog/tests.rs)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| F01 | planned | Existing strict parser, usage envelope and special entry protocols | Successful paths skip full prefix reflection. Usage, raw token, format, help and version controls distinguish the supported cases. |
| F02 | planned | F01 and A01's scoped and full discovery contracts | One matched comparison with parity controls and a recorded eager or selective-deferred decision. |
