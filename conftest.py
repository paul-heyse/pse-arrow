# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Repository-wide pytest plugins (plan §5).

Three rules, each of which exists because its absence has produced a green run
that proved nothing:

1. **Marker discipline.** Every test carries exactly one of ``unit``,
   ``component``, ``integration`` or ``performance``. Violations are collected
   and reported as a single ``UsageError`` listing every offender, because
   fixing them one failure per run is how a suite stays unmarked for a year.
2. **``--performance``.** IDAES semantics: performance tests are never
   collected by accident, and ``--performance`` collects nothing else.
3. **``--parity``.** Without the flag, parity items are deselected with one
   line saying so. With it, the environment is verified once per session and
   the session *fails* if the environment is wrong. It never skips: a parity
   suite that skips is indistinguishable from one that passes.

Stdlib and pytest only: this file has to work before the project's own package
is importable.
"""

import os
import shutil
import sys
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

if TYPE_CHECKING:
    from collections.abc import Generator

#: Exactly one of these is required on every test item.
REQUIRED_MARKERS = frozenset({"unit", "component", "integration", "performance"})

#: The parity reference. Moving it is an ADR (plan §7).
PARITY_IDAES_VERSION = "2.13.0"

#: IDAES 2.13.0 classifies 3.10-3.14; the platform floor is 3.11.
PARITY_MAX_PYTHON = (3, 15)


def _require_native_libraries() -> None:
    """Stop with the fix when the installed extension cannot load its shared libraries.

    The linked native build needs the solver libraries that only the native environment
    puts on the library path. Importing ``pse`` happens before any test conftest can
    report it, so the dynamic loader is asked here, without running package code; only
    its missing-library failure is labelled. Exit 125 is the environment boundary's.
    Only this checkout's own package is probed: a run elsewhere never imports it.
    """
    import ctypes  # noqa: PLC0415 -- only this probe needs it
    import importlib.machinery  # noqa: PLC0415
    import importlib.util  # noqa: PLC0415

    try:
        spec = importlib.util.find_spec("pse")
    except (ImportError, ValueError):
        return
    found = spec.submodule_search_locations if spec is not None else None
    own = Path(__file__).resolve().parent / "python" / "pse"
    locations = [Path(p) for p in (found or []) if Path(p).resolve() == own]
    for location in locations:
        for suffix in importlib.machinery.EXTENSION_SUFFIXES:
            library = location / f"_native{suffix}"
            if not library.is_file():
                continue
            try:
                ctypes.CDLL(str(library))
            except OSError as error:
                if "cannot open shared object file" not in str(error):
                    return
                sys.stderr.write(
                    "pse-env: the installed pse extension needs the native "
                    f"environment ({error}); run just py-unit (it routes) or "
                    "scripts/pse-env --native -- pytest …, or rebuild with "
                    "just py-sync\n"
                )
                raise SystemExit(125) from error
            return


_require_native_libraries()


def pytest_addoption(parser: pytest.Parser) -> None:
    """Register the two opt-in suite flags.

    Args:
        parser: The pytest option parser.
    """
    parser.addoption(
        "--performance",
        action="store_true",
        dest="performance",
        default=False,
        help="collect only tests marked `performance` (IDAES semantics)",
    )
    parser.addoption(
        "--parity",
        action="store_true",
        dest="parity",
        default=False,
        help=(
            "run the IDAES parity suite; requires the parity environment and "
            "FAILS (never skips) without it"
        ),
    )


def pytest_configure(config: pytest.Config) -> None:
    """Fold `--performance` into the mark expression, IDAES-style.

    Args:
        config: The active configuration.
    """
    markexpr = str(config.option.markexpr)
    if config.option.performance:
        config.option.markexpr = "performance"
    elif markexpr:
        config.option.markexpr = f"{markexpr} and not performance"
    else:
        config.option.markexpr = "not performance"


def _marker_offences(items: list[pytest.Item]) -> list[str]:
    """Describe every item that does not carry exactly one required marker.

    Args:
        items: The collected items.

    Returns:
        One line per offending item, naming what was found.
    """
    offences: list[str] = []
    for item in items:
        found = {marker.name for marker in item.iter_markers()} & REQUIRED_MARKERS
        if len(found) != 1:
            got = ", ".join(sorted(found)) if found else "none"
            offences.append(f"  {item.nodeid}: {got}")
    return offences


def _check_markers(items: list[pytest.Item]) -> None:
    """Fail collection once, listing every item with wrong required markers.

    Args:
        items: The collected items, before mark-expression deselection.

    Raises:
        pytest.UsageError: If any item does not carry exactly one of the
            required markers.
    """
    offences = _marker_offences(items)
    if not offences:
        return
    required = ", ".join(sorted(REQUIRED_MARKERS))
    message = "\n".join(
        [
            f"{len(offences)} test(s) do not carry exactly one of: {required}",
            *offences,
            (
                "Every test declares its responsibility category; "
                "see [tool.pytest.ini_options].markers."
            ),
        ]
    )
    raise pytest.UsageError(message)


def _deselect_parity(config: pytest.Config, items: list[pytest.Item]) -> None:
    """Remove parity items from the run unless `--parity` was given.

    Args:
        config: The active configuration.
        items: The collected items; modified in place.
    """
    if config.option.parity:
        return
    kept: list[pytest.Item] = []
    deselected: list[pytest.Item] = []
    for item in items:
        (deselected if item.get_closest_marker("parity") else kept).append(item)
    if not deselected:
        return
    config.hook.pytest_deselected(items=deselected)
    items[:] = kept
    config.stash[_parity_deselected_key] = len(deselected)


_parity_deselected_key = pytest.StashKey[int]()
_parity_uncollected_key = pytest.StashKey[bool]()
#: The parity test tree. Its conftest imports IDAES, which only the parity environment
#: (`just parity`) installs, so it is collected only when parity is requested.
_PARITY_TESTS = Path(__file__).resolve().parent / "python" / "pse" / "parity" / "tests"


def pytest_ignore_collect(collection_path: Path, config: pytest.Config) -> bool | None:
    """Leave the parity tree uncollected unless `--parity` was given.

    Deselecting parity items happens after collection, too late for a tree whose import
    needs IDAES; without this a bare `pytest` or `just py-unit` fails in any environment
    but the parity one. With `--parity` the tree is collected and the session still
    fails, never skips, when the environment is wrong.

    Args:
        collection_path: The path pytest is about to collect.
        config: The active configuration.

    Returns:
        True to leave the path uncollected; None to let pytest decide.
    """
    if config.option.parity:
        return None
    if collection_path == _PARITY_TESTS or _PARITY_TESTS in collection_path.parents:
        config.stash[_parity_uncollected_key] = True
        return True
    return None


@pytest.hookimpl(wrapper=True)
def pytest_collection_modifyitems(
    config: pytest.Config,
    items: list[pytest.Item],
) -> "Generator[None, object, object]":
    """Enforce marker discipline, then deselect parity items when not opted in.

    The marker check runs *before* mark-expression deselection so that running a
    subset never hides an unmarked test.

    Args:
        config: The active configuration.
        items: The collected items.

    Returns:
        Whatever the wrapped hook implementations returned.
    """
    _check_markers(items)
    result = yield
    _deselect_parity(config, items)
    return result


def pytest_collection_finish(session: pytest.Session) -> None:
    """Persist selected identities before execution when an assessment requests it.

    Args:
        session: The collected test session.
    """
    path = os.environ.get("PSE_TEST_ENUMERATION")
    if path:
        for item in session.items:
            item.user_properties.append(("nodeid", item.nodeid))
        # Node IDs cannot contain newlines; collection is a line-oriented inventory.
        temporary = Path(path).with_suffix(f".{os.getpid()}.tmp")
        temporary.write_text("\n".join(item.nodeid for item in session.items) + "\n")
        temporary.replace(path)


def pytest_report_collectionfinish(config: pytest.Config) -> list[str]:
    """Say, in one line, that parity items were left out.

    Args:
        config: The active configuration.

    Returns:
        The lines to add to the collection summary.
    """
    count = config.stash.get(_parity_deselected_key, 0)
    if not count:
        if config.stash.get(_parity_uncollected_key, False):
            return [
                (
                    "parity suite not collected: pass --parity "
                    "(in the parity environment) to run it"
                )
            ]
        return []
    line = (
        f"deselected {count} parity test(s): pass --parity to run them "
        f"(needs idaes-pse=={PARITY_IDAES_VERSION} and ipopt on PATH)"
    )
    return [line]


def _parity_environment_problems() -> list[str]:
    """List every reason this environment cannot run the parity suite.

    Returns:
        One line per unmet precondition; empty when the environment is good.
    """
    problems: list[str] = []
    if sys.version_info >= PARITY_MAX_PYTHON:
        wanted = ".".join(str(part) for part in PARITY_MAX_PYTHON)
        problems.append(
            f"python {platform_version()} is >= {wanted}: idaes-pse "
            f"{PARITY_IDAES_VERSION} does not support it. Use "
            "`UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked "
            "--group parity --python 3.13`."
        )
    try:
        installed = version("idaes-pse")
    except PackageNotFoundError:
        installed = None
    if installed != PARITY_IDAES_VERSION:
        problems.append(
            f"idaes-pse is {installed or 'not installed'}, "
            f"expected {PARITY_IDAES_VERSION} (the parity pin; moving it is an ADR)."
        )
    if shutil.which("ipopt") is None:
        problems.append(
            "ipopt is not on PATH. Use the solver container "
            "(`just bootstrap-solvers`) or build docker/solvers/build.sh natively."
        )
    return problems


def platform_version() -> str:
    """Return the running interpreter's version as a dotted string.

    Returns:
        For example ``3.14.7``.
    """
    return ".".join(str(part) for part in sys.version_info[:3])


@pytest.fixture(scope="session", autouse=True)
def parity_env(request: pytest.FixtureRequest) -> None:
    """Verify the parity environment once per session when `--parity` is given.

    Fails the session rather than skipping: a parity suite that skips reports
    the same green as one that ran.

    Args:
        request: The fixture request, used to read the session's options.
    """
    if not request.config.option.parity:
        return
    problems = _parity_environment_problems()
    if problems:
        pytest.fail(
            "the parity environment is not usable; parity never skips:\n"
            + "\n".join(f"  - {problem}" for problem in problems),
            pytrace=False,
        )
