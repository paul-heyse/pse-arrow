# Opt-in view decode backing allocation accounting

Profile revision: 2 (2026-11-04), capability-checked nested decoding.

Source: published `buffa` 0.9.2 archive, SHA-256
`a92f2f5df67a9d5ccfc65237bfc954c56328aee40a04a9b92381ecba370be246`.
Retained VCS metadata identifies `1f5d80e2c8d31c147742d8ec826a3219c4fcd4a7`.
The published manifest, original manifest, license and source are retained.
Compare against `https://static.crates.io/crates/buffa/buffa-0.9.2.crate`.

`DecodeContext::with_allocation_budget` attaches a caller-owned shared `Cell`.
The budget is phase-monotonic: every requested full new backing layout is debited
before allocation, never refunded, including failed allocation attempts. Copies,
descendants and merges draw from the same cell. Limit, layout-overflow and
fallible-reservation failures have distinct error variants. An absent budget
preserves stock behavior. Existing retained allocations require separate admission.

The opted-in generated eager/lazy view decoders account singular/group/oneof boxes,
repeated and map wire stores, packed scalar extenders, lazy fragments/raw elements,
unknown-field state boxes and spans, and synthetic closed-enum bytes. Vector
growth requests the larger of required length and checked double capacity using
`try_reserve_exact`. The exact pinned standard library's Global Vec implementation
stores that requested capacity; old and new layout charges remain spent even if
growth happened in place. A box uses a fallibly reserved exact single-element
vector, converted without a second allocation to a box of that element.

This is source-qualified logical requested backing storage, not allocator usable
size, metadata, RSS, stack or universal OOM containment. Owned decoding, ordinary
stock generated callers, cloning, conversion, default caches, encoding, validation,
native adoption and application work are outside this contract. Lazy getters create
fresh contexts and do not retain this allocation budget; a complete probe traversal
must explicitly decode/merge the exposed fragments under its original shared cell.
`MessageView::allocation_accounted` and `LazyMessageView::allocation_accounted`
default to false; only opted-in generated implementations declare the capability.
Every nested eager decoder checks the actual child capability before defaults or
decode, including ordinary imported messages, extern paths and WKTs. Unaccounted
children refuse; separately generated accounted external children are supported.
The provided eager entry/merge loops also refuse stock roots under a budget.
Explicit lazy traversal calls `DecodeContext::require_allocation_accounted` with
the deferred child's capability before decoding/merging its fragments; a parent's
accounted raw-fragment scan does not establish any child decode capability.

Changes: `src/message.rs`, `src/error.rs`, `src/lib.rs`, `src/view.rs`,
`src/types.rs`; new `src/decode_allocation.rs`. Source tests exercise pre-growth
refusal and state preservation, exact fit, spare capacity, overlap, shared siblings
and overflow. Companion generator compile controls execute real generated code.
Reproduce from the checkout root through the prepared native boundary:

```sh
scripts/pse-env -- cargo test -p buffa --lib allocation_ --locked
scripts/pse-env -- scripts/pse test pse-codegen:buffa_allocation
```

Run under the approved checkout activation and actual user-systemd scope. No
upstream contribution/acceptance or complete receiving-phase memory qualification
is claimed. The owning Plan65e retains production activation obligations.
