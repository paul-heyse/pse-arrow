# Request-owned cancellation snapshots

Source: the published `ra_ap_ide_db` 0.0.358 crate archive, SHA-256
`f08b0f04a8fecbe4a1b04cd0b03d5b0c8495a2a5de50638c6d66b5c3aba133ac`.
Its retained VCS metadata names `2ad81e72b7e978cb800dffe1bcce5c9f98032acf`
and records that publication used a dirty checkout; the archive checksum is
the exact source identity. The original published manifest is retained. MIT
and Apache license texts come from that recorded upstream revision.

The sole upstream source change adds `RootDatabase::request_snapshot` in
`src/lib.rs`. It delegates to the opt-in Salsa cancellation branch, cloning
handles over the original immutable input/query storage. Existing IDE
`Analysis` snapshots and their scoped parallel clones inherit this branch's
monotonic cancellation request while keeping local suppression and query
state independent. Other requests and the reusable original database remain
outside the branch.

Native product controls exercise the original IDE navigation, reference,
implementation and symbol APIs against this snapshot; no local replacement
resolver is retained. Compare to the archive at
`https://static.crates.io/crates/ra_ap_ide_db/ra_ap_ide_db-0.0.358.crate` when
updating or replacing the patch. No upstream contribution or acceptance is
claimed.
