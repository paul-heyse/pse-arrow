# sccache (C/C++ compile caching)

Realizations considered: the host's existing `sccache` installation, which is already the
workspace's Rust compiler cache. Its version was not recorded in this pass. Only documented
behaviour is considered. Research date 2026-10-27. Sources: Context7 `/mozilla/sccache` (pages
`docs/Configuration.md`, `docs/Local.md`, `docs/Rust.md`).

## Types

### sccache/compiler-output-cache
- **Need it serves:** reusing compiler outputs across processes for identical inputs and flags.
- **Recognizable code shapes:**
  - A local disk cache of objects or shared libraries keyed by a hash of the generated source,
    the compiler path and the flags.
- **Mechanisms and APIs:**
  - Run as a compiler wrapper (`sccache g++ …`).
  - Local disk backend: `SCCACHE_DIR`, `SCCACHE_CACHE_SIZE` (default 10G, with size-bounded
    eviction), `SCCACHE_DIRECT` (preprocessor-mode caching), `SCCACHE_LOCAL_RW_MODE`.
  - Optional multi-level backends.
- **Obligations discharged:**
  - hashing the inputs, including the preprocessed source and the arguments;
  - a bounded disk store with eviction;
  - sharing across processes through one server.
- **Residual caller obligations:**
  - Splitting compile from link. The documentation states that Rust crates which invoke the
    system linker cannot be cached; for C/C++ only compile steps are cacheable.
  - Loading and validating the produced library.
  - Resource admission for the wrapper process.
- **Semantic conditions:**
  - A background server process owns the cache.
  - "The local storage only supports a single sccache server at a time."
  - The cache lives outside the caller's memory and disk admission. Its eviction is asynchronous
    to the caller.
- **Version / build conditions:** depends on the installed sccache version.
- **Limits and negative results:**
  - No cached link step.
  - A daemon lifecycle.
  - No caller-visible typed refusal.
- **Evidence:** Interface-checked (documentation only).

## Current applicability

### [CBO-022](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-022) — Disk layer for compiled native evaluators
- **Types:** sccache/compiler-output-cache.
- **Realization:** the installed host tool, not a crate dependency.
- **Conclusion:** unsettled, leaning ruled out.
- **What it would discharge / what remains local:**
  - The local compiler invocation is two-phase (compile, then link) and runs under `prlimit`, so
    only the object phase could be cached.
  - Retaining the linked `.so` would still need the local cache.
  - Running the compiler through a daemon moves work outside the per-call resource scope.
- **Prerequisites, configuration or extension:** run the compiler wrapped by sccache inside the
  existing `prlimit` command.
- **Semantic gaps against the CBO's preservation constraints:**
  - "Compiled bytes carry headers and checksums" covers the final artifact, and sccache does not
    see that artifact.
  - The repository's persistence premise prefers no daemon (maintainer premise, 2026-10-21).
- **Unsettled premises and what would settle them:**
  - Whether the object phase dominates the cost. Timing it would settle this. Not authorized.
- **Evidence:** Interface-checked (documentation); Proposed.
