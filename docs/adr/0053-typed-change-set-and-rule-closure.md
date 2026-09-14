---
id: ADR-0053
title: Complete typed change-set keys and bounded recursive rule contracts
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-07, DM-09, DM-14, DM-24, DM-31]
blueprint: [§6.3, §6.7, §6.10, §6.11, §6.13, §14.2, §20.1, §22.2]
review: docs/design_review/reviews/design_review_wave-1-contract-corrections_2026-09-14.md#6-acceptance-gates
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A required change key cannot be represented by its registered row, or a consumer requires unbounded/distinct recursion
verification: `just test-package pse-schema -p pse-relations`; change-set, rename, recursion and semantic fingerprint fixtures in plan 03

---

# ADR-0053: Complete typed change-set keys and bounded recursive rule contracts

## Context

The key type pse.index_tuple cannot represent integer/text composite primary keys. A mandatory replacement row cannot represent delete. Rule serialization omits semantic expressions/options and has no explicit recursive self-reference.

## Scope

Amends the cited blueprint sections within the approved Wave 1 boundary. It supplements existing accepted decisions; their arguments remain immutable. Implementation is authorized by the maintainer's approved execution plan; formal ADR acceptance remains the decision-PR lifecycle.

## Drivers

Explicit semantics, enforceable validity, complete dependencies and honest capability claims before reuse or performance work.

## Options

Retain the inconsistent scaffold: rejected because its consumers cannot preserve the declared meaning. Build a general replacement platform: rejected without a demonstrated need. Complete the existing typed contracts: selected.

## Outcome

Use StagedRowRef { staged_port: text, staged_ordinal: u64 } for row_key and nullable row. Stage full schema-valid one-row members under distinct ordered operation/role ports in a durable sidecar envelope whose complete expected ports derive from the operations. Insert may share key/payload; update preserves the registry PK; delete has no replacement; key changes require delete plus insert. Staging never claims canonical model membership. Rename uses identity-bound source references from the exact base, renders all retained target/expression text, and stages document bytes, hashes and spans atomically. Refuse an unresolvable complete rename inventory without moving the ref. Rules persist actual typed expression/head/join/union/recursion semantics and migration defaults; dependency rows use exact rule IDs. RecursiveRef binds the nearest lexical Recursive node, is illegal in its seed, and is excluded from external reads. Persist the resolved target and recursion options in typed rows. Phase 0 supports UNION ALL with seed-row or explicit finite depth bounds; exceeding a bound with pending results is failure, never truncated success. Unsupported recursion is refused. Invariant head evidence does not constitute a write to the constrained authored relation.

Indexed targets bind local template-domain names through `authored.instance_domain_bindings`, keyed by `(instance_id, domain_name)` with an explicit `domain_id` foreign key. P1 validates the declared template domain name and actual domain compatibility, resolves each concrete selector to exactly one member in that domain, and stores its member ID in declaration order. Missing bindings, unknown/ambiguous members, repeated dimensions, wrong arity and domains owned outside the explicit instance context are errors. Wildcards retain their declared shape; no coordinate or label is hashed into an identity.

A commit ref atomically names the admitted snapshot manifest and an optional paired exact revision sidecar receipt. The receipt carries the declared revision identity, relation and encoding checksum; reopening validates the actual revision row against the snapshot, rather than inferring a revision from a content hash. General snapshot aliases may omit the receipt. Commit aliases require it after their first publication. Immutable revision, change-set and pass-record relations are created before the conditional ref write; failure leaves the old complete ref in force. Prepublication pass records carry nullable input/output snapshot fields, rather than invented snapshot identities. A commit with model and case content publishes the case tip with its admitted model parent, making one ref update expose the complete pair.

A derivation's optional fingerprint records an actual execution's existing stage or plan-evidence fingerprint. Standalone prepublication invariant evaluation has no such execution identity and stores null, matching its absent snapshot reference. Supporting keys preserve actual typed subject rows and the complete declared dependency bindings, including the binding context for negative claims. No invented digest or registry fingerprint stands in for execution attribution or semantic validation.

A durable change-set receipt is an immutable control object at `changes/<encoding_checksum>.json`. It binds the exact typed header and operation sidecars, every named one-row staging artifact, original/replacement document references, the expected base revision receipt and output revision target. The revision ref may attach this exact receipt in the same atomic publication value. A staging artifact admits one authored/reference row without minting a snapshot or planner constraints. Receipt admission verifies actual schema, values, row references and the complete named inventory; operation application and rename binding remain the C2 authority and must be replayed by authoring when that semantic history is consumed.

### Continuous-domain validity

**Proposed before implementation.** A domain with `continuous = true` has exactly one
`authored.continuous_domains` row with its actual domain ID. Its parent domain row has
a non-null `unit_id`, equal to the detail row's unit. A domain with `continuous = false`
has no continuous detail. The nullable parent unit remains available for discrete domains;
missing continuous units are rejected rather than inferred from kind or a hash. Member
ordinals are unique within each actual domain, making their declared order unambiguous.

These obligations are executable relational invariants over unpublished candidate rows.
Primary-key checks establish the detail's at-most-one cardinality; an anti-join establishes
required presence, and actual Boolean/unit comparisons establish applicability and meaning.
Grouped member ordinals produce the exact violating member keys. Required conformance
fixtures exercise both states, absent and conflicting units/details, and repeated ordinals.

### Local conditional publication

**Proposed; interfaces checked against Rust 1.98.1 and object_store 0.13.2.**
`Catalog::open_local` provides the same explicit conditional publication predicate
on supported Unix local filesystems. Independently opened catalogs coordinate through
one stable advisory lock file; every PSE mutable writer must use that path, and external
writers that bypass it are outside the supported contract. Under the exclusive OS lock,
the writer compares the complete actual current control bytes with the retained observed
bytes, or checks actual absence for creation. Metadata tokens and hashes do not substitute
for this comparison. Stale state fails without an automatic retry. Generic remote stores
continue to use their native conditional operation.

Immutable files and their containing directories are synchronized before publishing a
reference. A mutable replacement is written to an exclusively created sibling temporary
file, synchronized, atomically renamed and followed by parent-directory synchronization.
Cancellation before rename leaves the old value; after rename the operation reports its
actual outcome. A failure synchronizing the directory after rename explicitly reports
that visibility changed and durability is unconfirmed, requiring a reread. Abandoned
temporary files are never references. Lock acquisition and exact rereads retain cancellation
and finite resource bounds. Unsupported platforms or failed locking/synchronization are
typed refusals, never an unlocked overwrite. This relies on the local filesystem honoring
locking, atomic rename and synchronization; it makes no guarantee about failing hardware,
network filesystems, or writers bypassing the protocol.

The pinned local ObjectStore implementation rejects `PutMode::Update`; ordinary overwrite
alone cannot implement this contract. Rust's `File::try_lock` and Unix rename route supply
the required mechanism without a new dependency. Directory synchronization is necessary
in addition to file synchronization ([Linux fsync contract](https://man7.org/linux/man-pages/man2/fsync.2.html)).
Acceptance requires independently opened catalogs racing from the same observed value,
stale exact-byte rejection, cancelled lock waits, interrupted temporary writes and successful
second local commits. Power-loss behavior remains interface-checked rather than a tested
hardware guarantee.

## Consequences

Generic rule keys are not restricted to semantic IDs. `inferred.undecided.key` therefore uses the lossless typed row-key text contract shared by diagnostic references: `["pse.rule-key.v1", [[column_name, tagged_cell], ...]]`. The registry's sole Cell literal codec preserves primitive kinds, exact float bits and nested values; decoding checks the exact declared head-key names, order, types and values. This field references actual candidate rows and is never evidence that a rule head holds. Dropping non-ID key components or replacing them with a hash is invalid.

Resolved case/activation/observation targets use the declared key of their member: `symbol_decl_id` for a symbol or indexed symbol group, `equation_decl_id` for an equation, and the paired `port_template_id`/`port_name` for a template port. No fabricated port identity replaces its composite key. The member kind selects exactly one shape; instance wildcards contain no member reference. P1 resolves these references from actual staged instance/template membership and P2 checks the selected shape and owning relationship. Explicit entity rename preserves these references and re-renders their text; changing a port key is a relationship delete/insert.

Consumers must use checked contracts and explicit unsupported outcomes. Fixtures and generated output change with their authoritative declarations; this record does not claim runtime acceptance.

Revision 22 clarifies commit revision allocation: callers may preassign distinct nonzero model/case IDs so authored case rows can bind them before P2. Otherwise fresh UUIDv7 IDs are selected. Actual candidate case rows must bind the selected model revision without driver rewriting. Each new parent names the exact observed predecessor; an explicit receipt-chain walk checks IDs, rows and links before CAS and rejects cycles, missing links and reused revision IDs. Hash identities do not prove lineage validity.

### Compensating controls

The scoped design review, negative fixtures and plan 03 terminal gates guard the correction. Existing canonical framing and dependency-family pins remain unchanged except the explicitly named direct promotion.

### Confirmation

`just test-package pse-schema -p pse-relations`; change-set, rename, recursion and semantic fingerprint fixtures in plan 03. Each result records mode and failure count against baseline zero. The execution ledger distinguishes source inspection, focused tests and complete-wave acceptance.

## Pros and cons

The correction removes an ambiguity or false guarantee with bounded implementation work. Conservative rejection/copying/recomputation may cost more until a separately measured refinement is justified.

## More information

[Wave 1 execution plan](../plans/03-wave-1-foundations.md); blueprint §6.11, §14.2, §22.2; charter DM-59 and G1-G7. The authorized blueprint amendment uses PSE_DESIGN_EDIT=1 and must carry a revision row in the decision PR.

## Status history

- 2026-09-14 — proposed before implementation; maintainer approved the correction plan and its validation-before-hashing clarification.

- 2026-09-14 — revision 19 proposed before implementation: nullable execution attribution on standalone prepublication derivations; no new hash context.
- 2026-09-14 — local conditional publication clarified before implementation: stable cooperative OS lock, complete observed-byte comparison and explicit synchronization outcomes within blueprint §20.1.

- 2026-09-14 — revision 27 proposed before implementation: exact continuous-domain detail/unit cardinality and per-domain member ordinal uniqueness.
