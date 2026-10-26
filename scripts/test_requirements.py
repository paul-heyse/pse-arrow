# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected test requirements, consumed by preparation, admission and scheduling.

Catalogues are tool-produced. Declarations describe coherent owners, with bounded
fixture/peer exceptions, rather than a second manifest of every test name.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Never, Protocol, TypedDict

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


class MarkerLike(Protocol):
    @property
    def kwargs(self) -> Mapping[str, object]: ...


class TestItemLike(Protocol):
    @property
    def nodeid(self) -> str: ...

    def get_closest_marker(self, name: str) -> MarkerLike | None: ...


class RequirementDocument(TypedDict):
    """Existing JSON projection of declared demand; no inferred runtime authority."""

    effects: list[str]
    fixtures: list[str]
    capacity: str
    roles: list[str]
    exclusion: str | None


@dataclass(frozen=True)
class Requirement:
    effects: tuple[str, ...] = ()
    fixtures: tuple[str, ...] = ()
    capacity: str = "local"
    roles: tuple[str, ...] = ()
    exclusion: str | None = None

    def __post_init__(self) -> None:
        if self.exclusion not in {None, "service-generation-maintenance"}:
            raise ValueError("Unknown test exclusion meaning")

    def document(self) -> RequirementDocument:
        return {
            "effects": list(self.effects),
            "fixtures": list(self.fixtures),
            "capacity": self.capacity,
            "roles": list(self.roles),
            "exclusion": self.exclusion,
        }


POLICY_VERSION = 2

LOCAL = Requirement()
CANONICAL = Requirement(("canonical",), capacity="canonical", roles=("canonical",))
NATIVE = Requirement(("native",), capacity="scientific", roles=("native",))
CANONICAL_NATIVE = Requirement(
    ("canonical", "native"), capacity="scientific", roles=("canonical", "native")
)

# Reviewed namespace owners declare actual fixture/native demand. Unlisted new
# namespaces are local; the before-effects guard refuses undeclared acquisition.
RUST_OWNERS = {
    "xtask": {
        "development::evidence::tests::evidence_native_scoped_roles_complete_generation_and_cursor_fences": CANONICAL,
    },
    "pse-operations": {
        **dict.fromkeys(
            (
                "canonical::canonical_server_unit::",
                "canonical_analyses::race_tests::",
                "canonical_analyses::canonical_analyses_server_unit::",
                "canonical_artifacts::canonical_artifact_tests::",
                "canonical_capability_tests::",
                "canonical_contexts::tests::",
                "canonical_execution::canonical_execution_server_unit::",
                "canonical_explanations::server_tests::",
                "canonical_files::catalog_controls::",
                "canonical_history::canonical_history_tests::",
                "canonical_lifecycle::canonical_lifecycle_tests::",
                "canonical_projects::tests::",
                "canonical_results::canonical_results_server_unit::",
                "canonical_result_retention::canonical_result_retention_server_unit::",
                "canonical_retention::canonical_server_unit::",
                "canonical_retention::canonical_server_unit_exact_conflicts::",
                "canonical_selection::canonical_server_unit::",
                "canonical_staging::canonical_server_unit::",
                "canonical_studies::canonical_studies_server_unit::",
                "canonical_study_module_tests::",
                "testing::canonical_server_unit::",
            ),
            CANONICAL,
        ),
    },
    "pse-runtime": {
        "workflow::result_inspection::canonical_result_inspection_server_unit::": CANONICAL,
        "workflow::projects::deadline_tests::": LOCAL,
        "workflow::fitting::profile::worker_tests::": LOCAL,
        **dict.fromkeys(
            (
                "workflow::modeling::service_tests::immutable_source_uses_explicit_receiver_after_original_caller_closes",
                "workflow::modeling::service_tests::unrelated_services_refuse_source_preparation_and_single_step_start_before_effects",
            ),
            LOCAL,
        ),
        "workflow::replay::tests::plan38b_": NATIVE,
        "workflow::shooting::tests::": NATIVE,
        "workflow::horizon::tests::": NATIVE,
        **dict.fromkeys(
            (
                "workflow::fitting::oracle::tests::persisted_original_fit_failure_withholds_exact_covariance_despite_retained_cells",
                "workflow::horizon::tests::horizon_records_one_durable_attempt",
                "workflow::horizon::tests::compact_horizon_post_science_pressure_moves_original_completion_on_refusal",
                "workflow::horizon::tests::compact_horizon_cancel_before_first_solve_closes_exact_empty_summary_membership",
                "workflow::horizon::tests::compact_multi_step_horizon_seals_one_original_completion_and_seed_inventory",
            ),
            CANONICAL_NATIVE,
        ),
        **{
            f"workflow::{owner}::": CANONICAL_NATIVE
            for owner in (
                "analyses",
                "artifacts",
                "caller_lifetime_tests",
                "connected_results",
                "data_documents_tests",
                "durable",
                "durable_tests",
                "physical_cache",
                "project_portability",
                "projects",
                "projects_consumer_tests",
                "result_snapshot::result_pruning_tests",
                "result_projection",
                "study_tests",
                "study_workbench::canonical_tests",
                "study_observation",
                "study_operation_admission_tests",
                "worker_tests",
            )
        },
        **dict.fromkeys(
            (
                "math::portable::canonical_deployment_tests::",
                "math::portable::canonical_portable_body_tests::",
                "workflow::modeling::canonical::",
                "workflow::modeling::project_comparison::",
                "workflow::modeling::sensitivity_tests::",
                "workflow::fitting::covariance_tests::",
                "workflow::live_simulation::tests::canonical_",
            ),
            CANONICAL_NATIVE,
        ),
        **{
            f"math::{owner}::": NATIVE
            for owner in (
                "initialization",
                "modeling",
                "native_child",
                "prediction",
                "solves",
                "staged",
                "surrogate",
                "tests",
            )
        },
        **{
            f"workflow::{owner}::": NATIVE
            for owner in (
                "fitting",
                "modeling",
                "simulation_results",
                "live_simulation",
                "run",
                "results",
            )
        },
        "worker::": CANONICAL_NATIVE,
        "managed_primary_": CANONICAL_NATIVE,
    },
    "pse-backend-native": {
        **{
            f"{owner}::": NATIVE
            for owner in (
                "kinsol::tests",
                "kinsol::native_build_tests",
                "ipopt::tests",
                "uno",
                "petsc",
                "root_isolation",
                "dynamics",
                "highs",
                "clarabel_tests",
                "restart_tests",
                "scip_tests",
                "l1_tests",
                "pounce_convex_tests",
                "measurement_tests",
                "solver_tests",
            )
        },
    },
    "pse-tests-conformance": {
        "acceptance::": CANONICAL_NATIVE,
        "native_profiles::": NATIVE,
    },
}

# These Python owners actually construct durable Runtime/worker contexts. Merely
# requesting an isolated database in a new module does not imply native demand.
PYTHON_DURABLE_OWNERS = frozenset(
    {
        "test_projects.py",
        "test_project_portability.py",
        "test_study_workbench.py",
        "test_project_contexts.py",
        "test_project_cli.py",
        "test_modeling_strategies.py",
        "test_preparation_discovery.py",
        "test_studies.py",
        "test_modeling_run.py",
        "test_solve_settings_projection.py",
        "test_modeling_kernel.py",
        "test_plan14_acceptance.py",
        "test_review_workflows.py",
        "test_scientific_boundary_adoption.py",
        "test_live_simulation.py",
        "test_canonical_results.py",
        "test_native_workflow.py",
        "test_lifecycle.py",
        "test_research.py",
        "test_capability_adapters.py",
    }
)

RUST_FIXTURES = {
    "PSE_NATIVE_LANE_EXECUTABLE": (
        "pse-runtime",
        "math::native_child::tests::actual_portable_native_root_child_reconstructs_and_parent_screens",
    ),
    "PSE_PRODUCER_FIXTURE_RECEIPT": (
        "pse-runtime",
        "math::portable::canonical_portable_body_tests::canonical_portable_body_reconnect_reuses_ordinary_scalar_solve_with_actual_tool_fixture_receipt",
    ),
}
RUST_FIXTURE_FEATURES = {
    "PSE_NATIVE_LANE_EXECUTABLE": "solver-kinsol",
    "PSE_PRODUCER_FIXTURE_RECEIPT": "canonical-tests",
}
RUST_PEERS = {
    "context-overlap": (
        "pse-operations",
        "testing::canonical_server_unit::plan30_rust_python_fixture_context_overlap",
    ),
    "external-restart": (
        "pse-operations",
        "canonical_execution::canonical_execution_server_unit::retained_canonical_store_recovers_a_new_generation_after_external_server_restart",
    ),
    "operator-graph": (
        "pse-operations",
        "canonical_explanations::server_tests::canonical_operator_graph_evidence",
    ),
    "operator-query": (
        "pse-operations",
        "canonical_studies::canonical_studies_server_unit::canonical_operator_query_evidence",
    ),
}


def _invalid_protocol(message: str) -> Never:
    """Keep this owner's document/observation refusals in its ValueError contract."""
    raise ValueError(message)


def rust(binary: str, name: str) -> Requirement:
    package = binary.split("::", 1)[0]
    owners = RUST_OWNERS.get(package, {})
    matches = [prefix for prefix in owners if name.startswith(prefix)]
    base = owners[max(matches, key=len)] if matches else LOCAL
    # Binary ownership is explicit; worker integration cases exercise durable
    # native process lifetime rather than a lib namespace.
    if package == "pse-runtime" and binary == "pse-runtime::worker":
        base = CANONICAL_NATIVE
    fixtures = tuple(
        key
        for key, (owner, test) in RUST_FIXTURES.items()
        if owner == package and test == name
    )
    peers = tuple(
        key
        for key, (owner, test) in RUST_PEERS.items()
        if owner == package and test == name
    )
    exclusion = (
        "service-generation-maintenance" if "external-restart" in peers else None
    )
    if "context-overlap" in peers:
        # This database-only peer still prepares an admitted native worker context.
        base = CANONICAL_NATIVE
    if any(role.startswith("operator-") for role in peers):
        fixtures += ("PSE_CANONICAL_TOOLS",)
    if (
        package == "pse-runtime"
        and name
        == "workflow::tests::workflow_durability_selection_uses_this_deployment"
    ):
        base = CANONICAL_NATIVE
    if package == "pse-operations" and name in {
        "canonical::maintenance::tests::generated_maintenance_pages_fence_generations_and_replay_exact_effects",
        "canonical::maintenance::tests::native_maintenance_event_catalog_roundtrips_and_refuses_changed_or_extra_events",
        "canonical_research::canonical_research_server_unit::research_lost_ack_settles_original_requests_and_reuse_is_explicit",
    }:
        base = CANONICAL
    managed = (
        ("managed-primary",)
        if package == "pse-runtime" and name.startswith("managed_primary_")
        else ()
    )
    return Requirement(
        base.effects,
        fixtures,
        base.capacity,
        (*base.roles, *fixtures, *peers, *managed),
        exclusion,
    )


def python(
    fixtures: Sequence[str],
    *,
    no_allocation: bool = False,
    roles: Sequence[str] = (),
    owner: str = "",
) -> Requirement:
    canonical = "canonical_substrate" in fixtures
    if canonical and no_allocation:
        raise ValueError("Effect-free Python test cannot depend on canonical_substrate")
    native = canonical and owner.rsplit("/", 1)[-1] in PYTHON_DURABLE_OWNERS
    base = CANONICAL_NATIVE if native else CANONICAL if canonical else LOCAL
    return Requirement(base.effects, (), base.capacity, (*base.roles, *roles))


def python_item(item: TestItemLike) -> Requirement:
    """Interpret the collected fixture closure and explicit markers once."""
    declared = item.get_closest_marker("test_requirements")
    role_values = declared.kwargs.get("roles", ()) if declared else ()
    if not isinstance(role_values, (list, tuple)) or any(
        not isinstance(role, str) for role in role_values
    ):
        raise TypeError("Declared test roles must be a sequence of strings")
    roles = tuple(role_values)
    if item.get_closest_marker("managed_primary") is not None:
        roles += ("managed-primary",)
    no_allocation = (
        "no_canonical_allocation" in getattr(item, "fixturenames", ())
        or item.get_closest_marker("no_canonical_allocation") is not None
    )
    base = python(
        getattr(item, "fixturenames", ()),
        no_allocation=no_allocation,
        roles=roles,
        owner=item.nodeid.split("::", 1)[0],
    )
    if declared and declared.kwargs.get("canonical"):
        if no_allocation:
            raise ValueError("Effect-free test declares canonical controls")
        base = Requirement(("canonical",), ("owned-controls",), "canonical", roles)
    if declared and declared.kwargs.get("native"):
        base = Requirement(
            tuple(sorted({*base.effects, "native"})),
            base.fixtures,
            "scientific",
            tuple(sorted({*base.roles, "native"})),
            base.exclusion,
        )
    exclusion = declared.kwargs.get("exclusion") if declared else None
    if exclusion:
        if not isinstance(exclusion, str):
            raise TypeError("Declared test exclusion must be a string")
        base = Requirement(
            base.effects, base.fixtures, base.capacity, base.roles, exclusion
        )
    return base


def scheduling(requirement: Mapping[str, object]) -> tuple[str | None, int | str]:
    """Project semantic demand onto existing runner groups, never host allocation."""
    if requirement.get("exclusion") is not None:
        return "store", "num-test-threads"
    if requirement.get("capacity") == "scientific":
        return "scientific", 2
    effects = requirement.get("effects", ())
    if not isinstance(effects, (list, tuple)):
        _invalid_protocol("Unknown requirement effects shape")
    if "canonical" in effects:
        return "store", 1
    return None, 1


def pytest_group(requirement: Mapping[str, object]) -> str | None:
    """Serialize actual scientific demand without grouping database-only cases."""
    exclusion = requirement.get("exclusion")
    if isinstance(exclusion, str):
        return exclusion
    return "scientific-owner" if requirement.get("capacity") == "scientific" else None


def nextest_configuration(
    configuration: dict,
    profile: str,
    inventory: Mapping[str, object],
    requirements: Mapping[str, Mapping[str, object]],
) -> None:
    """Apply catalogue-bound scheduling to a per-run copy of repository config."""
    import re  # noqa: PLC0415

    import tomlkit  # noqa: PLC0415

    overrides = tomlkit.aot()
    observed = set()
    suites = inventory.get("rust-suites")
    if not isinstance(suites, dict):
        _invalid_protocol("Unknown nextest scheduling catalogue shape")
    for suite in suites.values():
        if not isinstance(suite, dict) or not isinstance(suite.get("testcases"), dict):
            _invalid_protocol("Unknown nextest scheduling suite shape")
        binary = suite["binary-id"]
        for name, test in suite["testcases"].items():
            if test["filter-match"]["status"] != "matches":
                continue
            identity = binary + "::" + name
            observed.add(identity)
            if identity not in requirements:
                raise ValueError(
                    "Scheduling requirements differ from selected catalogue"
                )
            requirement = requirements[identity]
            group, threads = scheduling(requirement)
            if group is None:
                continue
            if "/" in name or "/" in binary:
                raise ValueError("Unsupported nextest scheduling test identity")
            override = tomlkit.table()
            override["filter"] = (
                f"binary_id(/^{re.escape(binary)}$/) & test(/^{re.escape(name)}$/)"
            )
            override["test-group"] = group
            override["threads-required"] = threads
            overrides.append(override)
    if observed != set(requirements):
        raise ValueError("Scheduling requirements differ from selected catalogue")
    profiles = configuration.setdefault("profile", tomlkit.table())
    selected = profiles.setdefault(profile, tomlkit.table())
    for existing in selected.get("overrides", []):
        overrides.append(existing)
    selected["overrides"] = overrides


def rust_catalog(
    inventory: Mapping[str, object],
    *,
    required_roles: Sequence[str] = (),
    validate_declarations: bool = True,
    active_features: Mapping[str, Sequence[str]] | None = None,
) -> dict[str, RequirementDocument]:
    suites = inventory.get("rust-suites")
    if not isinstance(suites, dict):
        _invalid_protocol("Unknown nextest catalogue shape")
    result: dict[str, RequirementDocument] = {}
    found_roles: set[str] = set()
    for suite in suites.values():
        binary = suite["binary-id"]
        tests = suite["testcases"]
        # Validate exceptions against the full discovered owner, not the filtered
        # matches. A partial integration binary does not make a lib exception stale.
        if validate_declarations and suite.get("kind") == "lib":
            package = binary.split("::", 1)[0]
            for role, (owner, name) in RUST_FIXTURES.items():
                active = active_features is None or RUST_FIXTURE_FEATURES[
                    role
                ] in active_features.get(owner, ())
                if package == owner and active and name not in tests:
                    raise ValueError(
                        f"Unmatched fixture declaration {role}: {owner}::{name}"
                    )
        for name, test in tests.items():
            status = test.get("filter-match", {}).get("status")
            if status not in {"matches", "mismatch"}:
                raise ValueError("Unsupported nextest filter-match shape")
            if status == "matches":
                requirement = rust(binary, name)
                identity = binary + "::" + name
                if identity in result:
                    raise ValueError("Duplicate selected catalogue identity")
                result[identity] = requirement.document()
                found_roles.update(requirement.roles)
    validate_roles(result, required_roles)
    return result


def validate_roles(
    requirements: Mapping[str, Mapping[str, object]], roles: Sequence[str]
) -> None:
    observed: set[str] = set()
    for requirement in requirements.values():
        declared = requirement.get("roles", [])
        if not isinstance(declared, (list, tuple)) or any(
            not isinstance(role, str) for role in declared
        ):
            raise TypeError("Unknown requirement roles shape")
        observed.update(declared)
    missing = set(roles) - observed
    if missing:
        raise ValueError(
            "Required selected role missing: " + ", ".join(sorted(missing))
        )


def guard(requirements: Mapping[str, object], identity: str, effect: str) -> None:
    requirement = requirements.get(identity)
    if not isinstance(requirement, dict) or effect not in requirement.get(
        "effects", []
    ):
        raise ValueError(f"Undeclared {effect} demand for selected test {identity}")


def fixture_guard(
    requirements: Mapping[str, Mapping[str, object]], role: str, effect: str
) -> None:
    consumers = []
    for identity, demand in requirements.items():
        fixtures = demand.get("fixtures", [])
        if not isinstance(fixtures, (list, tuple)):
            raise TypeError("Unknown requirement fixtures shape")
        if role in fixtures:
            consumers.append(identity)
    if not consumers:
        raise ValueError(f"Native fixture {role} has no actual selected consumer")
    for identity in consumers:
        guard(requirements, identity, effect)


def required_roles(environment: Mapping[str, str]) -> tuple[str, ...]:
    return tuple(
        value
        for value in environment.get("PSE_TEST_REQUIRED_ROLES", "").split(",")
        if value
    )
