# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected test requirements, consumed by preparation, admission and scheduling.

Catalogues are tool-produced. Declarations describe coherent owners, with bounded
fixture/peer exceptions, rather than a second manifest of every test name.
"""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


@dataclass(frozen=True)
class Requirement:
    effects: tuple[str, ...] = ()
    fixtures: tuple[str, ...] = ()
    capacity: str = "local"
    roles: tuple[str, ...] = ()

    def document(self) -> dict[str, object]:
        return {key: list(value) if isinstance(value, tuple) else value for key, value in asdict(self).items()}


LOCAL = Requirement()
CANONICAL = Requirement(("canonical",), capacity="canonical", roles=("canonical",))
NATIVE = Requirement(("native",), capacity="scientific", roles=("native",))
CANONICAL_NATIVE = Requirement(("canonical", "native"), capacity="scientific", roles=("canonical", "native"))

# Conservative package defaults retain today's admission. Local module exceptions
# have explicit owners; F01 can refine declared demand without changing this seam.
RUST_DEFAULTS = {
    "pse-operations": CANONICAL,
    "pse-runtime": CANONICAL_NATIVE,
    "pse-tests-conformance": CANONICAL_NATIVE,
    "pse-tests-engine": CANONICAL,
    "pse-backend-native": NATIVE,
}
RUST_LOCAL_MODULES = {
    "pse-runtime": (
        "math::flows::", "math::initialization::", "math::modeling::", "math::prediction::",
        "math::retention::", "math::settings::", "math::staged::", "math::tests::",
        "workflow::controls::", "workflow::diagnostic_documents::", "workflow::diagnostic_rows::",
        "workflow::diagnostics::", "workflow::dynamics::", "workflow::numerics::",
        "workflow::progress_documents::", "workflow::result_blocks::", "workflow::result_projection::",
        "workflow::time::",
    ),
}
RUST_FIXTURES = {
    "PSE_NATIVE_LANE_EXECUTABLE": ("pse-runtime", "math::native_child::tests::actual_portable_native_root_child_reconstructs_and_parent_screens"),
    "PSE_PRODUCER_FIXTURE_RECEIPT": ("pse-runtime", "math::portable::canonical_portable_body_tests::canonical_portable_body_reconnect_reuses_ordinary_scalar_solve_with_actual_tool_fixture_receipt"),
}
RUST_FIXTURE_FEATURES = {
    "PSE_NATIVE_LANE_EXECUTABLE": "solver-kinsol",
    "PSE_PRODUCER_FIXTURE_RECEIPT": "canonical-tests",
}
RUST_PEERS = {
    "external-restart": ("pse-operations", "canonical_execution::canonical_execution_server_unit::retained_canonical_store_recovers_a_new_generation_after_external_server_restart"),
    "operator-graph": ("pse-operations", "canonical_explanations::server_tests::canonical_operator_graph_evidence"),
    "operator-query": ("pse-operations", "canonical_studies::canonical_studies_server_unit::canonical_operator_query_evidence"),
}


def rust(binary: str, name: str) -> Requirement:
    package = binary.split("::", 1)[0]
    base = RUST_DEFAULTS.get(package, LOCAL)
    if any(name.startswith(prefix) for prefix in RUST_LOCAL_MODULES.get(package, ())):
        base = LOCAL
    fixtures = tuple(key for key, (owner, test) in RUST_FIXTURES.items() if owner == package and test == name)
    peers = tuple(key for key, (owner, test) in RUST_PEERS.items() if owner == package and test == name)
    if any(role.startswith("operator-") for role in peers):
        fixtures += ("PSE_CANONICAL_TOOLS",)
    managed = ("managed-primary",) if package == "pse-runtime" and name.startswith("managed_primary_") else ()
    return Requirement(base.effects, fixtures, base.capacity, (*base.roles, *fixtures, *peers, *managed))


def python(fixtures: Sequence[str], *, no_allocation: bool = False, roles: Sequence[str] = ()) -> Requirement:
    canonical = "canonical_substrate" in fixtures
    if canonical and no_allocation:
        raise ValueError("Effect-free Python test cannot depend on canonical_substrate")
    # The current canonical fixture explicitly prepares a worker context before
    # yielding. F01 may narrow that owner; declaration follows its actual effects.
    return Requirement(CANONICAL_NATIVE.effects if canonical else (), (), "scientific" if canonical else "local", (*CANONICAL_NATIVE.roles, *roles) if canonical else tuple(roles))


def rust_catalog(inventory: Mapping[str, object], *, required_roles: Sequence[str] = (), validate_declarations: bool = True, active_features: Mapping[str, Sequence[str]] | None = None) -> dict[str, dict[str, object]]:
    suites = inventory.get("rust-suites")
    if not isinstance(suites, dict):
        raise ValueError("Unknown nextest catalogue shape")
    result: dict[str, dict[str, object]] = {}
    found_roles: set[str] = set()
    for suite in suites.values():
        binary = suite["binary-id"]
        tests = suite["testcases"]
        # Validate exceptions against the full discovered owner, not the filtered
        # matches. A partial integration binary does not make a lib exception stale.
        if validate_declarations and suite.get("kind") == "lib":
            package = binary.split("::", 1)[0]
            for role, (owner, name) in RUST_FIXTURES.items():
                active = active_features is None or RUST_FIXTURE_FEATURES[role] in active_features.get(owner, ())
                if package == owner and active and name not in tests:
                    raise ValueError(f"Unmatched fixture declaration {role}: {owner}::{name}")
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


def validate_roles(requirements: Mapping[str, Mapping[str, object]], roles: Sequence[str]) -> None:
    observed = {role for requirement in requirements.values() for role in requirement.get("roles", [])}
    missing = set(roles) - observed
    if missing:
        raise ValueError("Required selected role missing: " + ", ".join(sorted(missing)))


def guard(requirements: Mapping[str, object], identity: str, effect: str) -> None:
    requirement = requirements.get(identity)
    if not isinstance(requirement, dict) or effect not in requirement.get("effects", []):
        raise ValueError(f"Undeclared {effect} demand for selected test {identity}")


def fixture_guard(requirements: Mapping[str, Mapping[str, object]], role: str, effect: str) -> None:
    consumers = [identity for identity, demand in requirements.items() if role in demand.get("fixtures", [])]
    if not consumers:
        raise ValueError(f"Native fixture {role} has no actual selected consumer")
    for identity in consumers:
        guard(requirements, identity, effect)


def required_roles(environment: Mapping[str, str]) -> tuple[str, ...]:
    return tuple(value for value in environment.get("PSE_TEST_REQUIRED_ROLES", "").split(",") if value)
