# Development tooling

The maintainer put development tooling in full audit scope for this refresh. It covers:
- the FastMCP capability server (`scripts/capabilities/`);
- recipes and `scripts/pse-env` / `scripts/pse_env.py` (host admission, capacity, supervised
  scopes);
- the substrate supervisor;
- `xtask`;
- `pse-runtime::tooling`;
- agent hooks;
- generated developer knowledge;
- evidence tooling outside the [test account](../test-audit/README.md).

This page holds causes specific to that layer. Causes that cross the product/tooling boundary
keep their homes:
- [DA-019](boundaries-and-composition.md#da-019): the deployment contract in tooling;
- [DA-020](boundaries-and-composition.md#da-020): tooling families in product crates;
- [DA-024](local-reasoning-and-assurance.md#da-024): the producer chain without a consumer.

The [integrated account](system-architecture.md#the-development-tooling-layer) discusses the layer
as a whole. Citations are `path::symbol` at `cbf45ccc9`.

## The layer in brief

The tooling is a second system of comparable weight:
- about 76k lines of `scripts/*.py`, plus about 86k of script tests;
- an 8.9k-line FastMCP endpoint;
- about 20k lines of `xtask`;
- 23k lines in `pse-runtime::tooling`;
- about 4k lines of operator guides.

Inside the layer, composition is often genuinely library-based:
- FastMCP visibility rules, transforms, middleware and `Client`;
- systemd scopes, slices and limits for enforcement;
- `pyo3-introspection` for stubs;
- `just --dump` for recipe truth.

The native schemas in the endpoint are consumed from the generated catalogue, not mirrored.

The defects concentrate in two places:
- the boundary with the product (DA-019 and DA-020);
- general tooling meanings that are restated per owner (below).

## <a id="da-036"></a>DA-036 — Execution-resource and workload meaning has no single authority; a test manifest and agent host policy decide production budgets

**Principles:** AP-04, AP-05, DP-01, DP-20; H21, H25; G1.

**Refresh status:** new. It overlaps [DTA-045](../test-audit/execution-efficiency.md#dta-045) and
[DTA-046](../test-audit/execution-efficiency.md#dta-046), which concern test scheduling and
simultaneous demand. This finding concerns authority.

**Diagnosis.** The meaning of "how much memory, how many lanes and which exclusivity a workload
gets" is decided in at least seven places, several of them deciding the same numbers:
1. `.config/agent-capacity.toml`, the agent host policy, defines classes, partitions and
   `[reference] store_gib`, `receiver_headroom_gib` and `observer_gib`.
2. `scripts/surreal_server.py::{reference_execution, reference_resources}` hard-codes the same
   values (16 GiB server, 12 GiB headroom, 4 GiB observer) and enforces `case_lanes == 16`.
   `scripts/host_admission.py` reads the toml for them, and its error text hard-codes
   "140GiB primary and 160GiB envelope".
3. The **production** 16-lane pool (memory, threads, math jobs) is derived from the reference
   conformance campaign's manifest, `packages/reference/conformance.toml [settings]`.
4. `pse-runtime/src/workflow/native_deployment.rs::NativeAllocation::validate` re-imposes the
   relational rules.
5. `xtask/src/bin/pse-worker.rs::{resource_budget, verify_placement}` re-checks cgroup quotas,
   with "sixteen case lanes" in its error text.
6. `scripts/capabilities/native.py::Resources` defaults to 16 GiB, 16 threads and 1 GiB of spill.
7. Recipe classes are chosen statically in three ways:
   - 64 `--recipe-default-class` sites;
   - name heuristics in `host_admission.classify`;
   - the maintainer's local `PSE_BUILD_MODE=unbounded` overlay, which remaps them
     (`pse_env.py`).

**Consequence.** For the scenario "move to a host with different RAM or cores, or resize the
16-lane pool":
- The edit spans the toml, the conformance manifest, supervisor constants, Rust validators and
  error text.
- A partial edit silently decouples admission accounting from the systemd `MemoryMax` that the
  supervisor applies.
- Changing a test campaign's threads or memory changes the production worker's pool.
- For callers the class is a remembered convention. AGENTS.md carries a gotcha about not passing
  `--resource-class compile`, and the effective behavior in the maintainer's mode differs from
  the declared default.

**Counterevidence.**
- Admission exists for real reasons: the recorded OOM incidents and concurrent agents.
- Deriving the reference profile from the conformance manifest avoids a second numeric
  declaration.
- Static per-recipe classes are simple, and the unbounded overlay is a deliberate maintainer
  selection.
- What these do not justify: the same numbers living in toml and literals, and test-campaign
  settings acting as the production profile's authority.

**Evidence:** *Implemented* for the duplicated literals and derivations. The `default` →
`functional` classification was read, not executed.

**Improvement direction.**
- One owned declaration of execution profiles and workload classes, from which admission, the
  supervisor, runtime validation, the worker and MCP defaults derive.
- The product's durable profile separated from test-campaign settings.

## <a id="da-037"></a>DA-037 — Process-ownership primitives are re-authored per tooling owner with divergent semantics, inside one cyclic import web that includes the production supervisor

**Principles:** AP-06, DP-01, DP-13, DP-16; H4, H25, H26, H27.

**Refresh status:** new. It is the production-side counterpart of
[DTA-026](../test-audit/test-execution-system.md#dta-026), whose test-side cycles are not
duplicated here.

**Diagnosis.**
- **"This recorded owner is still alive" has at least four realizations:**

  | Realization | Rule |
  |---|---|
  | `native_operation.start_identity`, `host_admission.caller_alive`, `test_resources.borrower_alive` | Boot id plus `/proc` start tick; zombies count as alive |
  | `surreal_server.reservation_live` | Adds zombie, dead and "unresolved" states |
  | `payload_files.owner_dead` | Adds the PID namespace; raises on uncertainty |
  | `pse-operations/src/canonical_files.rs` (Rust) | Adds invocation and cgroup |

- **Private JSON-plus-`flock` ledgers are re-authored per owner.** They cover host admission,
  `resource_store`, the supervisor's state lock and lifecycle owner, context admission, tooling
  activities, payload instances, native operations, the compiler cache and producer deployment.
- **Drain observation is implemented separately in four modules.**
- **These owners form one strongly connected import component:**
  `pse_env` ↔ `host_admission` ↔ `native_operation` ↔ `surreal_server` ↔ `test_resources` ↔
  `resource_store`.
  - The cycle is broken by in-function imports and a `sys.modules` alias
    (`surreal_server.py` ≈45–50).
  - The production supervisor consults the test-resource registry before stop and drain
    (`require_borrowers_drained`).
  - The MCP endpoint imports the supervisor (`capabilities/specialists.py`).
- **Every top-level recipe line pays the full lifecycle unless explicitly marked `light`**
  (23 of about 240 recipes are). That lifecycle is a ledger lock and admission, parent and
  allocation enforcement, `systemd-run --scope`, then a second interpreter
  (`pse_env.py::placement`, `main`).

**Consequence.**
- **No local reasoning (AP-06).** Changing one owner's lifetime rule means reading the whole
  cycle, which includes the production supervisor and the frozen generations that copy it
  ([DA-019](boundaries-and-composition.md#da-019)).
- **One owner, different answers.** A zombie launcher keeps host capacity in `caller_alive` but
  releases a lifecycle reservation in `reservation_live`. Only some owners authenticate the PID
  namespace. No failure has been demonstrated, but each new owner re-derives the rule.
- **Overhead on cheap commands.** Discovery and light recipes pay allocation, a scope and two
  interpreter starts (H26). This is unmeasured.

**Counterevidence.**
- Lifecycle safety needs coordination: crashes and PID reuse are real.
- Each extra condition protects something concrete.
- Extensive actual-process tests exist.
- `os.pidfd_open` is already used.
- No external library removes the domain conditions.
- The modules' own comments call the cycle "reciprocal ownership", so it is stated intent rather
  than an accident.
- What none of this justifies is re-authoring with drift, and a cycle that couples production
  supervision to test-resource internals.

**Evidence:** *Implemented* for the divergent implementations and the import web. The
per-command overhead and the practical incidence of divergence are hypotheses.

**Improvement direction.**
- One owner-identity and liveness primitive with one stated semantics (namespace, zombie,
  uncertainty, cgroup), shared by the Python owners and matching the Rust owner identity.
- Acyclic layering: environment → admission → service supervision → borrowers registering
  through a declared interface.
- A light path that creates no scope for work that needs none.
