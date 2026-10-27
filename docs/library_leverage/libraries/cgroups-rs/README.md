# cgroups-rs (Rust)

Realizations considered: `cgroups-rs` 0.5.1 (released 2026-07-14; not in the lockfile).
Research date 2026-10-27. Sources: crate source 0.5.1 downloaded into scratch (`Cargo.toml`
dependencies; `src/fs/mod.rs` `parse_max_value`; module layout `src/fs/*` per controller,
`src/manager/{fs.rs,systemd.rs}`, `src/systemd/dbus/{client.rs,systemd_manager_proxy.rs}`).

## Types

### cgroups-rs/cgroup-manager
- **Need it serves:** create, configure, populate and tear down cgroups (v1 and v2), directly or
  through systemd, as a container runtime does.
- **Recognizable code shapes:** code that creates cgroups, writes controller limits and moves
  PIDs, or asks systemd to create scopes for workloads.
- **Mechanisms and APIs:** `fs::Cgroup` with per-controller types (`MemController`, `CpuController`,
  `PidController`, …), `fs::hierarchies`, `CgroupBuilder`; `parse_max_value` → `MaxValue::{Max,
  Value}`; a `manager` abstraction with `fs` and `systemd` backends, the latter calling
  `StartTransientUnit` through a zbus proxy.
- **Obligations discharged:** controller-file formats; v1/v2 differences; systemd transient-unit
  creation for container-style workloads.
- **Residual caller obligations:** policy; ancestor walks for effective limits; distinguishing an
  absent interface file from an inherited limit.
- **Semantic conditions:** oriented to managing cgroups it owns; read-only verification of an
  externally created placement is incidental.
- **Version / build conditions:** 0.5.1 depends unconditionally on `zbus` 5.8, `nix` 0.25,
  `thiserror` 1 (plus `oci-spec` optional).
- **Limits and negative results:** no `cgroup.kill`/`populated` event wait helper was found in the
  module list (not searched exhaustively); dependency weight is high for read-only use.
- **Evidence:** Interface-checked: source 0.5.1 (Cargo.toml and module listing; one parser read).

## Current applicability

### [CBO-019](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-019) — Linux process, cgroup and pressure facts parsed by hand in Rust and Python
- **Types:** `cgroups-rs/cgroup-manager`
- **Realization:** upgrade route 0.5.1; would add `zbus` 5.x, `nix` 0.25 (a second `nix` beside
  0.31.3) and `thiserror` 1 to the graph.
- **Conclusion:** ruled out here — the need is a read-only ancestor walk of three interface files
  with exact, fail-closed semantics; the crate is a manager whose dependency footprint exceeds the
  parsing it would remove, and it does not express "inherited vs absent `cpu.max`".
- **What it would discharge / what remains local:** only `max`/integer parsing; the walk and its
  semantics remain.
- **Semantic gaps against the CBO's preservation constraints:** none tested; the gap is cost and
  model, not correctness.
- **Unsettled premises and what would settle them:** would change if the runtime started creating
  per-child cgroups (CBO-004 containment route), where a manager could own creation/teardown —
  even then the direct file API is a few writes.
- **Evidence:** Interface-checked; Proposed (negative).
