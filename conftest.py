# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Repository-wide pytest plugins (plan §5).

Four rules, each of which exists because its absence has produced a green run
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
4. **``VerifyCleanup``.** A test that leaves an untracked file behind, or edits
   a tracked one, fails the session. Ported from ``idaes/conftest.py`` and
   extended to tracked-file modification.

Stdlib and pytest only: this file has to work before the project's own package
is importable.
"""

import os
import shutil
import subprocess
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
PARITY_IDAES_VERSION = "2.12.0"

#: IDAES 2.12.0 classifies 3.10-3.13; the platform floor is 3.11.
PARITY_MAX_PYTHON = (3, 14)


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
            "Every test declares its cost tier; see [tool.pytest.ini_options].markers.",
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


class VerifyCleanup:
    """Fail the session when a test leaves the working tree dirty.

    Tests write to ``tmp_path``. A test that writes into the repository instead
    makes the next run's results depend on the previous one's, and makes
    ``git status`` useless during a debugging session. Ported from
    ``idaes/conftest.py`` and extended: modifying a *tracked* file is caught too,
    which the original did not do.
    """

    def __init__(self, repo_root_dir: Path) -> None:
        """Bind the plugin to a repository checkout.

        Args:
            repo_root_dir: The root of the git working tree to watch.
        """
        self._repo_root_dir = Path(repo_root_dir).resolve()
        self._added_by_test: dict[str, list[str]] = {}
        self._modified_by_test: dict[str, list[str]] = {}

    def _git(self, *args: str) -> list[str]:
        """Run a read-only git command in the watched checkout.

        Args:
            *args: The git sub-command and its arguments.

        Returns:
            The command's output lines, or the error text when git failed, so a
            broken invocation shows up as a difference rather than as silence.
        """
        command = ["git", "-C", str(self._repo_root_dir), *args]
        try:
            text = subprocess.check_output(command, text=True).strip()
        except (subprocess.CalledProcessError, OSError) as exc:
            text = str(exc)
        return text.splitlines()

    def _untracked(self) -> list[str]:
        """Return the untracked, non-ignored files in the checkout.

        Returns:
            One path per untracked file, relative to the repository root.
        """
        return self._git("ls-files", "--others", "--exclude-standard")

    def _modified(self) -> list[str]:
        """Return porcelain status lines for tracked files only.

        Returns:
            One porcelain line per modified tracked file.
        """
        return self._git("status", "--porcelain", "--untracked-files=no")

    def pytest_report_collectionfinish(self) -> list[str]:
        """Announce what this plugin watches.

        Returns:
            The line to add to the collection summary.
        """
        return [
            f"watching for files created or modified by tests in {self._repo_root_dir}"
        ]

    @pytest.hookimpl(wrapper=True)
    def pytest_runtest_protocol(
        self,
        item: pytest.Item,
    ) -> "Generator[None, object, object]":
        """Diff the working tree around each test.

        Args:
            item: The test being run.

        Returns:
            Whatever the wrapped hook implementations returned.
        """
        untracked_before = set(self._untracked())
        modified_before = set(self._modified())
        result = yield
        added = set(self._untracked()) - untracked_before
        changed = set(self._modified()) - modified_before
        if added:
            self._added_by_test[item.nodeid] = sorted(added)
        if changed:
            self._modified_by_test[item.nodeid] = sorted(changed)
        return result

    @pytest.hookimpl(trylast=True)
    def pytest_terminal_summary(
        self,
        terminalreporter: pytest.TerminalReporter,
    ) -> None:
        """Report every test that dirtied the working tree.

        Args:
            terminalreporter: The reporter to write the section to.
        """
        sections = (
            ("Files added (and not cleaned up) by tests", self._added_by_test),
            ("Tracked files modified by tests", self._modified_by_test),
        )
        offenders: set[str] = set()
        for title, records in sections:
            if not records:
                continue
            offenders |= set(records)
            terminalreporter.section(title)
            for nodeid, paths in records.items():
                terminalreporter.write_line(nodeid)
                for path in paths:
                    terminalreporter.write_line(f"\t{path}")
        if offenders:
            terminalreporter.write_line(
                f"{len(offenders)} test(s) did not clean up after themselves; "
                "the exit status of the test session will be set to failed"
            )

    @pytest.hookimpl(trylast=True)
    def pytest_sessionfinish(self, session: pytest.Session) -> None:
        """Fail the session if any test dirtied the working tree.

        Args:
            session: The finishing session.
        """
        if self._added_by_test or self._modified_by_test:
            session.exitstatus = pytest.ExitCode.TESTS_FAILED


def _repo_root() -> Path | None:
    """Locate the git checkout this test run is inside.

    Returns:
        The absolute repository root, or None when pytest is running against an
        installed copy rather than a checkout (in which case there is nothing to
        keep clean).
    """
    try:
        text = subprocess.check_output(
            ["git", "-C", str(Path(__file__).parent), "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.DEVNULL,
        )
    except (subprocess.CalledProcessError, OSError):
        return None
    return Path(text.strip())


def pytest_addhooks(pluginmanager: pytest.PytestPluginManager) -> None:
    """Register the working-tree cleanliness plugin when inside a checkout.

    Args:
        pluginmanager: The session's plugin manager.
    """
    repo_root = _repo_root()
    if repo_root is not None:
        pluginmanager.register(VerifyCleanup(repo_root), name="pse-verify-cleanup")
