# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-backed exposure families; inspection never invokes a suggested route.

These declarations own exposure metadata, not CLI grammar, implementation status or
scientific schemas. The composition root registers actual tools separately. Recipe
inspection is an explicit operation: importing this module and building ``inventory``
never starts Just, a compiler, a native extension or a service.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal

from scripts.capabilities.model import Capability, Domain, Guide, Workflow, WorkflowStep

if TYPE_CHECKING:
    from pathlib import Path

SOURCE_INVENTORY = (
    "docs/functional_reviews/agent-tool-discovery-and-access-2026-10-24.md"
)
SOURCE_REVIEW = (
    "docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md"
)
CAP05 = "docs/plans/52i-canonical-capabilities-and-progressive-reveal.md#CAP05"
ADP03 = "docs/plans/52j-native-contracts-and-tool-adaptation.md#ADP03"
ADP04 = "docs/plans/52j-native-contracts-and-tool-adaptation.md#ADP04"
ADP05 = "docs/plans/52j-native-contracts-and-tool-adaptation.md#ADP05"
MCP01 = "docs/plans/52k-unified-entry-point-and-derived-interfaces.md#MCP01"


@dataclass(frozen=True)
class _Family:
    capability: Capability
    recipes: tuple[str, ...]
    prefixes: tuple[str, ...]
    sources: tuple[str, ...]


# Native routes are independent operations; actual MCP members derive from each
# binding's family rather than a copied command grammar or migration ledger.
RETENTION = {
    "source-context": "Canonical task/concept/producer reads coexist with native affected previews and literal source inspection; unresolved dynamic reach stays explicit.",
    "language-navigation": "Canonical closed navigation uses the owned bridge; finite native request/serve commands remain useful for independent diagnosis.",
    "controlled-edits": "Canonical preview/apply/reconcile preserves source cuts and protection; native structural migrations and explicit edit probes remain independently useful.",
    "knowledge-recall": "Canonical exact source corpus preserves attribution; explicit session scans and derived-store import/rebuild retain bounded native owners.",
    "library-research": "Canonical library packets expose pins and supplied evidence; outline, source fetching and skill-link synchronization retain their explicit native effects.",
    "environment-preparation": "Canonical environment explanation is effect-free and selected assessments run readiness; installers, image builds and provider preparation retain native ownership.",
    "build-cache": "Canonical scoped compilation uses the check owner; cache experiments, uncached builds and compiler observers retain distinct populations and native artifacts.",
    "dependency-maintenance": "Canonical assessments consume advisory reports; named upgrades, lock resolution and family maintenance remain explicit native manifest/Git operations.",
    "python-extension": "Interactive extension preparation owns Maturin, installation and stubs; assessment profiles consume it without making endpoint startup an installer.",
    "validation-selection": "Canonical selected Rust invocations and assessments use actual normalized selection/assurance/result owners; Python/parity campaigns retain native scope.",
    "qualification-assessment": "Canonical selection/run and protected reads share Gate expansion/result composition; recipes remain independently useful native gate executors.",
    "verification-techniques": "Canonical coverage inspection verifies retained source/population digests; capture, mutation and fuzz retain actual populations and original native execution limits.",
    "workload-observation": "Canonical detached watch/results consume authentic owners; reference-run setup/drain and direct live observers remain distinct native operations.",
    "scientific-diagnostics": "Canonical checkpoint/frozen/invocation reads retain offline native codecs; richer scientific diagnostic/execution interfaces remain native.",
    "active-diagnostics": "Canonical typed actions/readers share capture kernels and helper drain; installation, taskdump builds and specialized trace commands retain native preparation.",
    "database-tooling": "Canonical toolkit adapters preserve VIEWER credentials/generation fences; lifecycle, schema/module publication and administrative maintenance retain native admission.",
    "build-storage": "Canonical metadata inspection grants no disposal; recursive measurement and authenticated reclamation retain distinct native lifetime/effect ownership.",
    "projects-revisions": "Canonical coarse project/revision/research/result bindings retain codecs and immutable identities; portable exchange, studies and full project APIs retain native resource ownership.",
    "scientific-inputs": "Scientific-input capture/admission and source-bound fixture preparation retain native producers; canonical source admission does not replace them.",
    "evidence-inspection": "Canonical protected/frozen reads preserve codecs; import/query/export and rebuildable knowledge projections retain native custody/effect admission.",
    "experiments-recovery": "Canonical archive/inspect/replay shares kernels/historical identity; bisect, comparison, database populations and recovery retain distinct native owners.",
    "scientific-references": "Named reference/measurement producers retain isolated environments and fresh-run meaning; historical replay is not current scientific qualification.",
    "generated-contracts": "Canonical contract generation/check shares recipes; full/physical/bindgen generation and compiled-stub publication remain native producer operations.",
    "documentation-lifecycle": "Canonical scope/build shares metadata/publication owners; retirement remains native preflight, and authoring remains protected repository editing.",
    "decisions-register": "Canonical concept/scope reads inspect decision status; allocation/supersession and due register-authored checks retain native authoring/execution meaning.",
    "session-specialists": "Selected sessions attach only the canonical endpoint; native explanation/configuration keeps prepared, attached and exercised distinct.",
    "producer-workers": "Assessments consume deployment qualification; producer capture and supervised workers remain native infrastructure with original leases/cancellation/admission.",
}


def _family(
    identity: str,
    title: str,
    domain: str,
    owner: str,
    description: str,
    effects: str,
    *,
    recipes: str = "",
    prefixes: str = "",
    sources: str = "",
    guides: tuple[str, ...] = (),
    route: tuple[str, ...] = (),
    prerequisites: tuple[str, ...] = (),
    exposure: Literal[
        "tool", "native", "deferred", "internal", "unsupported"
    ] = "native",
    work_owner: str = ADP05,
    reason: str | None = None,
) -> _Family:
    return _Family(
        Capability(
            id=identity,
            title=title,
            domain=domain,
            owner=owner,
            source=owner,
            description=description,
            effects=effects,
            guides=guides,
            route=route,
            prerequisites=prerequisites,
            exposure=exposure,
            reason=reason if reason is not None else RETENTION[identity],
            work_owner=work_owner if exposure == "deferred" else None,
        ),
        tuple(recipes.split()),
        tuple(prefixes.split()),
        tuple(dict.fromkeys((owner, *sources.split()))),
    )


def _families() -> tuple[_Family, ...]:
    # Purpose/effect associations only. Arguments/defaults come from each real owner.
    return (
        _family(
            "source-context",
            "Source context and declared ownership",
            "workspace",
            "scripts/workspace_tasks.py",
            "Task context, exact concepts, declared producers and change reach; unresolved dynamic reach remains explicit.",
            "Reads selected source, metadata and explicitly selected Git history.",
            recipes="context affected default",
            sources="scripts/workspace_context.py scripts/workspace_sources.py scripts/workspace_domains.py scripts/affected.py",
            guides=("root",),
            route=("just", "context"),
        ),
        _family(
            "language-navigation",
            "Language symbols and source navigation",
            "editing",
            "scripts/semantic_navigation.py",
            "Compiler/language-server symbol operations and exact source cuts; textual leads are not semantic edges.",
            "Language-server attachment and selected source reads are explicit.",
            guides=("rust", "python"),
            route=(".venv/bin/python", "-m", "scripts.semantic_navigation", "--help"),
            work_owner=ADP04,
        ),
        _family(
            "controlled-edits",
            "Protected edits and structural migrations",
            "editing",
            "scripts/semantic_edits.py",
            "Preview and apply owned language edits or codemods with generated/protected-path refusal and reconciliation.",
            "Preview reads sources; explicit apply mutates selected authored files.",
            sources="scripts/structural_migrations.py scripts/semantic_edit_probe.py",
            guides=("rust", "python", "generated"),
            route=(".venv/bin/python", "-m", "scripts.semantic_edits", "--help"),
            work_owner=ADP04,
        ),
        _family(
            "knowledge-recall",
            "Attributed knowledge and history",
            "knowledge",
            "scripts/knowledge_store.py",
            "Inspect attributed knowledge and explicitly bounded session history; historical statements are not current authority.",
            "Selected local history reads; knowledge mutations require their native command.",
            recipes="knowledge",
            sources="scripts/session_retrospective.py",
            guides=("root",),
            route=("just", "knowledge", "--help"),
            work_owner=ADP03,
        ),
        _family(
            "library-research",
            "Library sources and capability guidance",
            "knowledge",
            "scripts/library_skills.py",
            "Library overviews, selected portable skills and source/API outlines; missing coverage does not prove absence.",
            "Reads sources; skill sync explicitly changes selected skill links.",
            recipes="lib-outline skills-sync skills-check fetch-external",
            sources="scripts/fetch-external.sh .config/library-skills.toml docs/libraries/README.md",
            guides=("root",),
            route=("just", "context", "--library"),
        ),
        _family(
            "environment-preparation",
            "Environment explanation and preparation",
            "environment",
            "scripts/pse_env.py",
            "Resolve checkout, interpreter, native capabilities and workload placement through the existing environment boundary.",
            "Explanation reads configuration; preparation/install/build commands have explicit effects.",
            recipes="doctor doctor-json ready llvm-system slice-install solver-image solver-pin-update solver-pin-check solver-rebuild-check native-isolation-prepare native-pipeline-prepare resolve-math-profiles",
            prefixes="bootstrap",
            sources="scripts/pse-env scripts/setup-env scripts/doctor.py scripts/bootstrap.sh scripts/llvm_system.py scripts/solver-images.py scripts/solver-rebuild-check.sh scripts/installation_context.py scripts/cargo_tools.py scripts/preflight.py scripts/native_operation.py scripts/setup_report.py scripts/native-solver-env.sh scripts/native-math-env.sh scripts/native-pipeline-env.sh scripts/native-execution-env.sh scripts/native-solver-runner.sh",
            guides=("environment",),
            route=("scripts/pse-env", "--explain", "--json"),
            work_owner=ADP03,
        ),
        _family(
            "build-cache",
            "Compilation and compiler-cache diagnosis",
            "environment",
            "scripts/build_environment.py",
            "Compile the selected target and inspect compiler/native caches without substituting a workspace build for a scoped check.",
            "Compilation and cache experiments create artifacts; explicit cache actions can remove retained build data.",
            recipes="check check-library check-package check-test build-cache-probe build-frontend build-uncached bench-builds dev-native-engine dev-native-boundaries engine-boundary-check",
            prefixes="check-native- check-pipeline- check-solver-",
            sources="scripts/build_cache_probe.py scripts/build_measurements.py scripts/compiler_cache.py scripts/native_cache.py scripts/native_pipeline_cache.py scripts/sccache",
            guides=("environment", "rust"),
            route=("just", "check-package"),
        ),
        _family(
            "dependency-maintenance",
            "Pinned dependency maintenance",
            "environment",
            "Cargo.toml",
            "Inspect exact dependency families and deliberately update selected pins through the existing owners.",
            "Reports read declarations; update/lock commands deliberately mutate manifests and lockfiles.",
            recipes="upgrade lock-python lock-workspace versions metadata metadata-resolve deps-report family-check floors-latest audit-advisories audit-dependencies audit-shear",
            sources="pyproject.toml scripts/cargo_feature_check.py xtask/src/dependency_ceilings.rs",
            guides=("environment", "rust", "python"),
            route=("just", "deps-report"),
        ),
        _family(
            "python-extension",
            "Python extension preparation",
            "environment",
            "scripts/python_extension.py",
            "Select the actual development, correctness or linked native extension through the native build owner.",
            "Builds and installs an editable extension; correctness routes explicitly activate validation.",
            recipes="py-sync py-sync-native py-sync-correctness py-sync-native-correctness py-environment-ensure py-extension-prepare py-stubs-prepare py-extension-admit",
            guides=("python", "environment"),
            route=("just", "py-sync"),
        ),
        _family(
            "validation-selection",
            "Selected correctness and scientific validation",
            "validation",
            "scripts/select.py",
            "Scope-aware tests, conformance, parity and force-validation admission retain their actual scientific exclusions.",
            "Selected checks compile/run workloads; native/store fixtures may start only on explicit execution.",
            recipes="test test-package test-release doctest doctest-release parity modeling-conformance seed-conformance functional-test-target feature-absence native-test native-python py-test py-unit py-native-contracts list-native-contracts scientific-replay-miri native-setup-unit",
            prefixes="unit- feature-absence",
            sources="scripts/arrow_validation.py scripts/test_invocation.py scripts/test_requirements.py scripts/validation.py scripts/validation_scope.py scripts/case_audit.py scripts/test_resources.py scripts/validation_feature_campaign.py scripts/native_tests.py scripts/python_tests.py python/pse/conformance.py",
            guides=("qualification", "rust", "python"),
            route=("just", "assessment-list"),
        ),
        _family(
            "qualification-assessment",
            "Qualification groups and static checks",
            "validation",
            "scripts/validation_scope.py",
            "Explain and execute the chosen qualification group; selected passes do not qualify unexercised product scope.",
            "Listing reads declarations; checks run only when explicitly selected at the prescribed scope boundary.",
            recipes="assessment assessment-list hygiene quality governance governance-tests ci-fast ci-pr policy turn-end typecheck python-contracts-check setup-test setup-test-report producer-deployment-unit producer-identity-test producer-fixture worker-cli-unit-test worker-test python-runner-unit-test",
            prefixes="lint- clippy fmt features-",
            sources="scripts/check_agent_config.py scripts/check_python_contracts.py scripts/check_generated.py scripts/reuse_lint.py scripts/setup_report.py",
            guides=("qualification", "root"),
            route=("just", "assessment-list"),
        ),
        _family(
            "verification-techniques",
            "Coverage, mutation and boundary fuzzing",
            "validation",
            "scripts/assertion_sensitivity.py",
            "Explicit coverage/mutation and boundary-fuzz campaigns, preserving unexecuted and denied-campaign distinctions.",
            "Runs selected instrumented tests, mutants or fuzz fixtures and retains their evidence.",
            recipes="coverage mutants-file unsafe-surface",
            sources="scripts/coverage_correspondence.py scripts/boundary_fuzz.py scripts/audit_tools.py xtask/src/bin/frozen-inspection-fuzz.rs xtask/src/bin/inspection-fuzz-seeds.rs",
            guides=("qualification",),
            route=("just", "coverage"),
        ),
        _family(
            "workload-observation",
            "Workload observation and retained results",
            "execution",
            "scripts/activity.py",
            "Read authentic invocation handles, live/recorded observations, partial results and native continuations.",
            "Observation does not own, cancel or settle the subject workload; result detail may be released.",
            recipes="activity result reference-run",
            sources="scripts/activity_watch.py scripts/run_inspection.py scripts/validation_result.py scripts/validation_receipts.py scripts/reference_run.py scripts/test_run.py scripts/build_observation.py scripts/pressure_observation.py scripts/invocation_disposition.py",
            guides=("environment", "qualification"),
            route=("just", "activity"),
            work_owner=ADP03,
        ),
        _family(
            "scientific-diagnostics",
            "Scientific diagnostics and offline inspection",
            "diagnostics",
            "python/pse/diagnostics.py",
            "Inspect scientific results, frozen packs and source context through their existing protected/native owners.",
            "Supplied offline inspection stays offline; active queries and experiments require explicit selection.",
            recipes="diagnostics",
            sources="python/pse/_inspection.py python/pse/_diagnostic_query.py python/pse/_diagnostic_launch.py python/pse/_diagnostic_query_child.py scripts/numerical_investigation.py",
            guides=("diagnostics",),
            route=("just", "diagnostics", "--help"),
            work_owner=ADP04,
        ),
        _family(
            "active-diagnostics",
            "Active process diagnostics and optional profilers",
            "diagnostics",
            "scripts/active_diagnostics.py",
            "Explicit GDB/perf/taskdump/heap/memray/rr attachment and trace queries with exact subject identity.",
            "Attachment/sampling can affect a live process; installing tools and rebuilding diagnostic extensions are explicit.",
            recipes="diagnostic-tools diagnostic-python-tools trace-query py-sync-taskdump",
            sources="scripts/diagnostic_trace.py scripts/pipeline_review.py scripts/highs_lifecycle_diagnostic.py",
            prerequisites=(
                "The selected profiler/debugger and its native process permissions must be available.",
            ),
            guides=("diagnostics",),
            route=("just", "diagnostics", "active", "--help"),
            work_owner=ADP04,
        ),
        _family(
            "database-tooling",
            "Canonical database and specialist tooling",
            "storage",
            "scripts/surreal_server.py",
            "Select explicit service/namespace context and use native database tooling without bypassing project admission.",
            "Service lifecycle, queries, module builds and fixture mutation are explicit operations.",
            recipes="surreal surreal-module surrealkit surreal-test surreal-fixture-test canonical-init",
            sources="scripts/surreal_diagnostics.py scripts/surrealism_module.py scripts/surrealkit.py scripts/surrealkit_maintenance.py scripts/surrealkit_population.py scripts/surrealkit_source.py scripts/surreal_source.py scripts/surreal_telemetry.py scripts/surreal_rollout_producer.py scripts/grpc_wire.py xtask/src/surrealql_tools.rs",
            guides=("storage", "diagnostics"),
            route=("just", "surreal", "--help"),
            work_owner=ADP04,
        ),
        _family(
            "build-storage",
            "Build-storage and disk health",
            "storage",
            "scripts/build_storage.py",
            "Inventory build/cache placement and storage health before deliberately selected cleanup.",
            "Inspection reads storage facts; explicit cleanup changes only native-selected storage resources.",
            recipes="build-storage",
            sources="scripts/storage_health.py scripts/resource_store.py scripts/payload_files.py",
            guides=("environment", "storage"),
            route=("just", "build-storage", "--help"),
            work_owner=ADP03,
        ),
        _family(
            "projects-revisions",
            "Projects, revisions and scientific data",
            "projects",
            "xtask/src/project_tools.rs",
            "Inspect and mutate projects through guarded native operations with source revisions, explicit stores and protected lifetimes.",
            "Supplied-only inspection never builds implicitly; mutations and dedicated operator builds are explicit.",
            recipes="project project-build",
            sources="xtask/src/project_runtime.rs xtask/src/bin/pse-project.rs python/pse/_artifacts.py python/pse/_transfer.py python/pse/_runs.py python/pse/modeling.py python/pse/_modeling.py python/pse/_physical.py python/pse/_analyses.py python/pse/_research.py python/pse/_workflow.py python/pse/_strategies.py python/pse/_study_workbench.py",
            guides=("storage", "python"),
            route=("just", "project", "--help"),
            work_owner=ADP04,
        ),
        _family(
            "scientific-python-api",
            "Native scientific Python boundary",
            "projects",
            "python/pse/__init__.py",
            "Use the typed Python scientific surface and native codec/admission; exposure metadata does not reconstruct scientific fields or defaults.",
            "Imports the compiled boundary only when the user actually uses the native Python API.",
            sources="python/pse/_array.py python/pse/governance.py",
            guides=("python", "qualification"),
            route=("python/pse/__init__.py",),
            exposure="native",
            work_owner=ADP04,
            reason="Retain the useful native Python API; any MCP scientific binding must preserve generated schemas and actual native admission in ADP04.",
        ),
        _family(
            "scientific-inputs",
            "Captured scientific inputs and dependency reach",
            "projects",
            "scripts/scientific_inputs.py",
            "Plan53b captures exact authored scientific bytes and derived import/package closures through the Rust reader/parser/P0 resolver; conservative reach is not Python-reconstructed science.",
            "Explicit capture builds/runs the native reader and writes a capsule; retained capsule reads are separate.",
            sources="xtask/src/scientific_inputs.rs tests/fixtures/scientific-inputs.toml python/pse/_scientific_inputs.py",
            guides=("qualification", "python"),
            route=(".venv/bin/python", "-m", "scripts.scientific_inputs", "--help"),
            work_owner=ADP04,
        ),
        _family(
            "evidence-inspection",
            "Original evidence and derived tooling indexes",
            "evidence",
            "xtask/src/evidence_tools.rs",
            "Inspect supplied evidence offline or explicitly manage the independently disposable index, retaining native provenance and partiality.",
            "Offline reads do not build/start services; index/build/refresh mutations are explicit.",
            recipes="evidence evidence-build evidence-regen",
            sources="xtask/src/bin/pse-evidence.rs scripts/evidence_sources.py scripts/evidence_projection.py scripts/evidence_store.py scripts/evidence_detail.py scripts/evidence-regen.sh",
            guides=("qualification", "storage"),
            route=("just", "evidence", "--help"),
            work_owner=ADP04,
        ),
        _family(
            "experiments-recovery",
            "Experiments, isolation and recovery",
            "experiments",
            "scripts/experiments.py",
            "Explicit archive/replay, regression/comparison, fixture isolation and canonical portability/recovery controls.",
            "Experiments execute selected workloads and own distinct archives, worktrees and database resources; cleanup retains native lifetime rules.",
            recipes="experiment canonical-test canonical-portable-test canonical-recovery-test",
            sources="scripts/tooling_experiments.py scripts/regression_experiments.py scripts/test_archives.py xtask/src/bin/pse-canonical-recovery.rs xtask/src/canonical_recovery.rs",
            guides=("environment", "qualification"),
            route=("just", "experiment", "--help"),
            work_owner=ADP03,
        ),
        _family(
            "scientific-references",
            "Scientific references and performance campaigns",
            "experiments",
            "scripts/scientific_reference.py",
            "Selected independent scientific references, thermodynamic feeds and measured campaigns retain their exact conditions and qualification limits.",
            "Builds/runs explicitly selected reference or benchmark workloads and publishes their artifacts.",
            recipes="plan14-reference plan53a-reference feos-reference srk-reference nrtl-reference thermodynamic-feeds thermodynamic-campaign case-measure py-measure-force py-measure-production",
            prefixes="bench-",
            sources="scripts/case_measure.py scripts/thermodynamic_feeds.py scripts/thermodynamic_campaign.py scripts/plan14_reference.py scripts/pfr_reference.py scripts/cstr_controller_reference.py scripts/decay_fit_reference.py scripts/feos_entropy_reference.py scripts/idaes_srk_reference.py scripts/idaes_nrtl_reference.py scripts/pr_stability_reference.py scripts/gross2001_bank.py xtask/src/feos_reference.rs",
            guides=("qualification",),
            route=("just", "case-measure"),
        ),
        _family(
            "generated-contracts",
            "Schemas, registry generation and compiled stubs",
            "generation",
            "xtask/src/codegen.rs",
            "Find the authored declaration and generator before changing generated schemas, relations, documents or Python stubs.",
            "Generation publishes derived files; check routes compare without publication.",
            recipes="python-stubs python-stubs-check snapshots-accept",
            prefixes="codegen",
            sources="scripts/check_generated.py xtask/src/main.rs python/pse/_build.py",
            guides=("generated", "rust", "python"),
            route=("just", "codegen"),
        ),
        _family(
            "documentation-lifecycle",
            "Documentation and native lifecycle",
            "governance",
            "scripts/document_lifecycle.py",
            "Read native plan/package scope and retirement preflight; projections do not authorize retirement or close work.",
            "Inspection reads owned documents; explicit site/serve commands build or serve documentation.",
            recipes="docs docs-rust docs-serve docs-test doc-lint plan",
            sources="scripts/document_metadata.py scripts/docs.py",
            guides=("docs", "decisions"),
            route=("just", "docs"),
        ),
        _family(
            "decisions-register",
            "Decisions, ADRs and deferred triggers",
            "governance",
            "scripts/adr.py",
            "Use the accepted decision route, immutable accepted ADRs and the owning deferred-trigger register.",
            "Explicit creation/index/supersession updates authored or derived governance documents.",
            recipes="register-check register-lint",
            prefixes="adr-",
            sources="scripts/check_register.py",
            guides=("decisions",),
            route=("just", "adr-new"),
        ),
        _family(
            "session-specialists",
            "Explicit session and specialist attachment",
            "integration",
            "scripts/codex_session.py",
            "Launch selected Rust/Python/database specialists with checkout identity and native lazy lifetimes; configuration alone is not a verified attachment.",
            "Explicit launch starts selected specialist processes and a session; inventory never attaches or probes them.",
            recipes="capabilities codex-session agent-config-sync",
            sources="scripts/agent-config.py .codex/config.toml .claude/settings.json",
            guides=("environment", "root"),
            route=("just", "codex-session", "--help"),
            work_owner=MCP01,
        ),
        _family(
            "git-integration",
            "Git, isolation and repository integration",
            "integration",
            "scripts/worktree.py",
            "Retain native Git and owned worktree setup, dirty-hunk ownership and explicit integration/closeout decisions.",
            "Worktree creation and explicit Git/GitHub actions mutate checkouts or remote state; source inventory authorizes none.",
            recipes="worktree gh-setup gh-setup-check labels-sync wheels-check",
            sources="scripts/gh-setup.sh scripts/github-config.py scripts/labels-sync.sh .agents/roles/README.md",
            guides=("root", "environment"),
            route=("just", "worktree"),
            exposure="native",
            reason="Native Git and isolated worktrees remain useful access; generic closeout automation is separately owned by Plan52g G04.",
        ),
        _family(
            "producer-workers",
            "Producer deployment and worker integration",
            "execution",
            "scripts/producer_deployment.py",
            "Use explicit producer identity, deployment and worker execution owners rather than constructing another scheduler.",
            "Build/deploy/run effects require the selected native producer/worker context.",
            recipes="producer-deployment producer-identity pse-worker",
            sources="scripts/producer-cargo scripts/producer_cargo.py scripts/workspace.rs xtask/src/producer_identity.rs xtask/src/producer_preparation.rs xtask/src/producer_history.rs xtask/src/bin/pse-worker.rs xtask/src/bin/pse-native-lane.rs",
            guides=("environment",),
            route=("just", "producer-deployment", "--help"),
            work_owner=ADP04,
        ),
        _family(
            "host-implementation",
            "Internal host boundary implementations",
            "environment",
            "scripts/host_admission.py",
            "Admission, execution preparation and hook plumbing are implementation owners, not independent callable catalogue operations.",
            "Effects remain owned by their actual environment/session callers.",
            sources="scripts/agent-hooks.py scripts/cargo-bin/cargo-pse-correctness scripts/native-engine-boundaries.py scripts/__init__.py",
            guides=("environment", "tooling"),
            exposure="internal",
            work_owner=CAP05,
            reason="These helpers support public owners; exposing arbitrary internal dispatch would create a second command surface.",
        ),
    )


def _guides() -> tuple[Guide, ...]:
    return tuple(
        Guide(id=identity, title=title, path=path, kind=kind)
        for identity, title, path, kind in (
            (
                "root",
                "Repository instructions and task routes",
                "AGENTS.md",
                "markdown",
            ),
            (
                "environment",
                "Environment and workload ownership",
                "docs/dev/agent-environment.md",
                "markdown",
            ),
            ("rust", "Rust scope rules", ".claude/rules/rust.md", "markdown"),
            ("python", "Python scope rules", ".claude/rules/python.md", "markdown"),
            ("tooling", "Tooling scope rules", ".claude/rules/tooling.md", "markdown"),
            ("docs", "Documentation scope rules", ".claude/rules/docs.md", "markdown"),
            (
                "decisions",
                "Decision and plan rules",
                ".claude/rules/decisions.md",
                "markdown",
            ),
            (
                "generated",
                "Generated-file ownership",
                ".claude/rules/generated.md",
                "markdown",
            ),
            (
                "qualification",
                "Qualification scope and interpretation",
                "docs/dev/validation-assessment.md",
                "markdown",
            ),
            (
                "diagnostics",
                "Simulation diagnostics",
                "docs/dev/simulation-diagnostics.md",
                "markdown",
            ),
            (
                "storage",
                "Operational store ownership",
                "docs/dev/operational-store.md",
                "markdown",
            ),
        )
    )


def inventory(
    root: Path,
) -> tuple[
    tuple[Domain, ...], tuple[Workflow, ...], tuple[Guide, ...], tuple[Capability, ...]
]:
    """Prepare cheap exact metadata only; do not resolve installed availability."""
    domains = tuple(
        Domain(id=identity, title=title, description=description, guides=guides)
        for identity, title, description, guides in (
            (
                "workspace",
                "Workspace",
                "Source context, exact concepts and declared ownership.",
                ("root",),
            ),
            (
                "editing",
                "Editing",
                "Language navigation and protected source changes.",
                ("rust", "python"),
            ),
            (
                "knowledge",
                "Knowledge",
                "Attributed history and source-backed library research.",
                ("root",),
            ),
            (
                "environment",
                "Environment",
                "Environment, builds, caches and dependency preparation.",
                ("environment",),
            ),
            (
                "validation",
                "Validation",
                "Selected correctness, scientific checks and qualification.",
                ("qualification",),
            ),
            (
                "execution",
                "Execution",
                "Workload observation, native execution and result lifetimes.",
                ("environment",),
            ),
            (
                "diagnostics",
                "Diagnostics",
                "Offline scientific inspection and explicit active diagnostics.",
                ("diagnostics",),
            ),
            (
                "storage",
                "Storage",
                "Canonical services, namespaces and storage maintenance.",
                ("storage",),
            ),
            (
                "projects",
                "Projects",
                "Projects, source revisions and scientific inputs.",
                ("storage",),
            ),
            (
                "evidence",
                "Evidence",
                "Original evidence and independently derived indexes.",
                ("qualification",),
            ),
            (
                "experiments",
                "Experiments",
                "Selected references, isolation and recovery.",
                ("qualification",),
            ),
            (
                "generation",
                "Generation",
                "Authored declarations and derived contracts.",
                ("generated",),
            ),
            (
                "governance",
                "Governance",
                "Documentation, decisions and deliberate process skills.",
                ("docs", "decisions"),
            ),
            (
                "integration",
                "Integration",
                "Sessions, specialists, Git and checkout integration.",
                ("root", "environment"),
            ),
        )
    )

    # These intent groups are the only authored workflow membership. Optional
    # groups are revealed deliberately through capabilities_select.workflow_steps.
    def step(
        identity: str,
        purpose: str,
        *capabilities: str,
        default: bool = True,
        guides: tuple[str, ...] = (),
    ) -> WorkflowStep:
        return WorkflowStep(
            id=identity,
            purpose=purpose,
            capabilities=capabilities,
            default=default,
            guides=guides,
        )

    steps = {
        "inspect-plan": (
            step(
                "scope",
                "Read the exact task owner and page its current native scope without authorizing execution.",
                "workspace_task_context",
                "document_scope_query",
            ),
            step(
                "source-navigation",
                "Follow a selected source symbol or original source evidence when scope needs clarification.",
                "navigation_definition",
                "navigation_references",
                "knowledge_source_corpus",
                default=False,
            ),
            step(
                "native-route",
                "Inspect an exact native recipe when the plan names a command.",
                "workspace_recipe_describe",
                default=False,
            ),
        ),
        "change-generated-contract": (
            step(
                "declaration",
                "Read task context, locate authored producers and inspect the exact generated native schema before editing.",
                "workspace_task_context",
                "workspace_producer_relationships",
                "native_contract_schema",
            ),
            step(
                "edit",
                "Preview and deliberately apply an authored source change; reconcile any interrupted apply.",
                "navigation_rename_preview",
                "navigation_action_preview",
                "edit_preview_read",
                "edit_preview_apply",
                "edit_preview_reconcile",
                default=False,
            ),
            step(
                "regenerate",
                "Run the original generator after editing its declaration, then compare generated contracts.",
                "contracts_generate",
                "contracts_check",
                default=False,
            ),
        ),
        "plan-functional-review": (
            step(
                "prepare",
                "Retrieve the planning skill and original capability context; preserve the inquiry and planning phase boundary.",
                "skill-plan-functional-extension-review",
                "workspace_concepts_query",
                guides=("skill-plan-functional-extension-review",),
            ),
            step(
                "library-sources",
                "Inspect exact library/source evidence relevant to the selected functional inquiry.",
                "library_source_packet",
                "document_scope_query",
                default=False,
            ),
        ),
        "diagnose-run": (
            step(
                "original-result",
                "Read the authentic invocation and separate workload, assurance and drain outcomes before choosing diagnostics.",
                "product_run_read",
                "workload_watch",
            ),
            step(
                "scientific-detail",
                "Read selected native invocation/evidence and offline scientific diagnostics in an already prepared environment.",
                "native_invocation_read",
                "native_evidence_inspect",
                "native_diagnostic_inspect",
                default=False,
            ),
            step(
                "active-experiment",
                "Perform only a deliberately selected diagnostic experiment and read its retained original record.",
                "diagnostic_action",
                "diagnostic_action_read",
                default=False,
            ),
        ),
        "qualify-change": (
            step(
                "select",
                "Describe the relevant assessment and test closure before choosing correctness scope.",
                "assessment_select",
                "rust_test_describe",
            ),
            step(
                "execute",
                "Run the selected native checks at the authorized qualification boundary and retain authentic results.",
                "compilation_check",
                "rust_test_run",
                "assessment_run",
                default=False,
            ),
            step(
                "evidence",
                "Read original invocation and assessment evidence; a selected pass does not qualify unexercised scope.",
                "product_run_read",
                "rust_test_reference_read",
                "assessment_read",
                "coverage_correspondence_read",
                default=False,
            ),
            step(
                "prepare",
                "Inspect selected readiness and explicitly prepare only the required owner stage.",
                "capability_readiness",
                "capability_readiness_check",
                "python_extension_dev_stage",
                "python_extension_native_stage",
                default=False,
            ),
        ),
        "investigate-evidence": (
            step(
                "source",
                "Read attributed original source evidence with explicit completeness.",
                "knowledge_source_corpus",
            ),
            step(
                "retained-run",
                "Follow an authentic run or diagnostic action record without rerunning its subject.",
                "product_run_read",
                "diagnostic_action_read",
                default=False,
            ),
            step(
                "native-evidence",
                "Inspect exact native scientific evidence in the selected existing environment.",
                "native_evidence_inspect",
                "native_invocation_read",
                default=False,
            ),
            step(
                "archive",
                "Inspect original historical archive bytes without treating them as current-source qualification.",
                "experiment_inspect",
                default=False,
            ),
        ),
        "manage-project": (
            step(
                "context",
                "Inspect exact project metadata and revision source in an already prepared native environment.",
                "native_project_metadata",
                "native_revision_source",
            ),
            step(
                "select",
                "Deliberately select an existing project through its native owner.",
                "native_project_select",
                default=False,
            ),
            step(
                "source-admission",
                "Discover exact preparation requirements and explicitly admit authored source through its native validator.",
                "native_preparation_discover",
                "native_revision_preparation_discover",
                "native_source_admit",
                default=False,
            ),
            step(
                "results",
                "Inspect selected original project/result/research artifacts without asserting a solve outcome.",
                "native_project_bundle_inspect",
                "native_result_read",
                "native_result_inspect",
                "native_research_read",
                default=False,
            ),
        ),
        "run-experiment": (
            step(
                "archive",
                "Create an explicitly selected pure-test archive with original isolation and recovery ownership.",
                "experiment_archive",
            ),
            step(
                "inspect",
                "Verify exact retained archive bytes before interpreting or replaying historical results.",
                "experiment_inspect",
            ),
            step(
                "replay",
                "Deliberately replay historical executables into fresh output; replay does not qualify current source.",
                "experiment_replay",
                default=False,
            ),
        ),
    }
    workflows = tuple(
        Workflow(
            id=identity,
            title=title,
            description=description,
            guides=guides,
            steps=steps[identity],
        )
        for identity, title, description, guides in (
            (
                "inspect-plan",
                "Inspect a plan",
                "Find current native scope and relevant source owners without authorizing execution.",
                ("root", "decisions"),
            ),
            (
                "change-generated-contract",
                "Change a generated contract",
                "Find the declaration, edit its owner, regenerate and select affected checks.",
                ("generated", "rust", "python"),
            ),
            (
                "plan-functional-review",
                "Plan a functional review",
                "Retrieve preparation guidance; preserve the selected inquiry and phase boundary.",
                ("root",),
            ),
            (
                "diagnose-run",
                "Diagnose a run",
                "Follow authentic invocation, result, source and diagnostic owners.",
                ("diagnostics",),
            ),
            (
                "qualify-change",
                "Qualify a change",
                "Select explicit correctness scope and report the actual evidence boundary.",
                ("qualification",),
            ),
            (
                "investigate-evidence",
                "Investigate evidence",
                "Read original evidence with provenance and explicit completeness.",
                ("qualification",),
            ),
            (
                "manage-project",
                "Manage a project",
                "Use explicit source revision, store and protected resource context.",
                ("storage",),
            ),
            (
                "run-experiment",
                "Run an experiment",
                "Select the native experiment with its own isolation and recovery obligations.",
                ("environment", "qualification"),
            ),
        )
    )
    guides = list(_guides())
    capabilities = [family.capability for family in _families()]
    # Skills are instruction packages, not executable workflow handlers. Their own
    # files remain the single procedure owner; discovery does not execute a skill.
    for path in sorted((root / ".codex/skills").glob("*/SKILL.md")):
        name = path.parent.name
        guide_id = "skill-" + name
        relative = path.relative_to(root).as_posix()
        within_checkout = path.resolve().is_relative_to(root.resolve())
        if within_checkout:
            guides.append(Guide(id=guide_id, title=name, path=relative, kind="skill"))
        capabilities.append(
            Capability(
                id=guide_id,
                title=name,
                domain="knowledge"
                if name == "native-solver-libraries"
                else "governance",
                description="Read the owned skill instructions; preparation and action retain their separate phase and authorization boundaries.",
                owner=relative,
                source=relative,
                exposure="native",
                effects="Instruction retrieval only; no workflow execution.",
                guides=(guide_id,) if within_checkout else (),
                route=(relative,),
                reason="An owned instruction package remains native guidance, not a callable placeholder."
                if within_checkout
                else "Retained native guidance outside checkout; the native agent skill loader owns access. No checkout guidance resource is registered.",
            )
        )
    return domains, workflows, tuple(guides), tuple(capabilities)
