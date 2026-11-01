# Development tooling

**Interface-checked, 2026-11-01, source `19e7b49a67df24c1bf73005a6186e145e930d53a`.**
The [collection baseline and limits](README.md) apply throughout.

The native host now owns command admission, original-result projection, process supervision
and engineering investigation. Query contracts separate selectors, predicates, answer demand
and presentation. Three-valued truth preserves unknown under negation; incomplete file
examination cannot establish a negative file predicate. Mandatory provenance survives
presentation. Generator publication captures and rechecks its inputs
(`pse-host/src/investigation/` and `xtask/src/codegen/code_intelligence.rs`).
These inspected strengths materially improve the reliability of audit discovery.

## <a id="da-043"></a>DA-043 — Small commands reauthenticate complete immutable artifact populations

**AP-07, DP-10; H12, H14, H18. Revalidated and extended.**

Ordinary command entry in `pse-host/src/cli.rs` acquires `EntryGuard`.
`generation/entry.rs` calls `GenerationManifest::load`, whose loop in `generation.rs` hashes
every manifest binary, including siblings the selected command will not execute.
A small source inspection therefore depends on the total prepared generation's bytes and on
sibling integrity.

Native provider selection in `native/providers.rs` recursively verifies immutability and then
validates the provider through `native.rs::file_map`, which recursively hashes provider files.
External tools are separately hashed by `apply_inputs`. Boundary activation also precedes
discovering enclosing foreground ownership (`boundary.rs` and `environment.rs`).
Native project and worker roots hash their own executable before dispatch.

The structural concern is fixed auxiliary work that grows with the generation/provider
population and can repeat across nested calls, even for tiny read operations.
No latency, throughput or capacity penalty was measured.
Authentication has a concrete integrity benefit, and this audit does not establish a cheaper
replacement preserving it. In particular, external tools can mutate outside an immutable
generation and their revalidation guards a real uncertainty. That is distinct from rescanning
all prepared immutable siblings.

## <a id="da-057"></a>DA-057 — The former native/Python authority split has retired

**Historical diagnosis retired at this baseline.**

The Python host-policy implementations named by the preceding account are absent from
current `scripts/`; retained Python recipes serve independent references and reference-tool
tests. Host path resolution now converges on `HostPaths`, including empty-XDG handling.

The legacy admission reader authenticates original resident records read-only; this does not
establish a second live policy writer. Host and scientific runtime profile readers also consume
different allocation responsibilities. Their coexistence alone is not duplicate authority.
No remaining conflicting interpretation was established in this stream.

## <a id="da-058"></a>DA-058 — Generated protection improved; cold-runtime enforcement is unverified

**Earlier multi-list diagnosis retired; external enforcement question remains open.**

`pse-host/src/ownership.rs` consumes the aggregate-generated ownership map produced by
`xtask/src/codegen/ownership.rs`. `agent_hooks.rs` uses a conservative protection fallback.
Worktree creation explicitly prepares and authenticates its host generation
(`pse-host/src/worktree.rs`). Architecture and accepted-ADR routes now intentionally provide
advisory workflow context; they are not claimed to be unconditional edit blocks.

The unprepared `scripts/pse-host` launcher exits 125, while a deliberate guard refusal exits 2.
Whether each actual agent runtime treats the former as blocking was not exercised.
This remains an external enforcement uncertainty, not a proved bypass.
Static deny entries and governance exclusions serve different responsibilities; independent
lists alone do not establish competing protection authority.

## Earlier identities

<a id="da-036"></a>**DA-036** was resolved for execution-profile authority in the earlier account.
Its later coexistence concern was incorporated into DA-057, whose Python premise has now retired.

<a id="da-037"></a>**DA-037** was merged into DA-057. Its Python reciprocal tooling web is no
longer a current production explanation.

Historical source evidence for both remains recoverable at source `19e7b49a6` and its history.
No new profile-policy or host-runtime qualification is implied.
