# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cache service ownership and actual raced-child containment controls."""

from __future__ import annotations

import contextlib
import fcntl
import io
import os
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

import pytest

from scripts import compiler_cache as cache


def alive(pid: int) -> bool:
    try:
        return Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[0] != "Z"
    except FileNotFoundError:
        return False


class CompilerCacheTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.addCleanup(self.temporary.cleanup)
        self.configuration = self.root / "sccache.toml"
        self.configuration.write_text("client_side_mode = true\n")
        self.configuration.chmod(0o600)
        self.env = {
            "HOME": str(self.root),
            "PATH": os.defpath,
            "PSE_SCCACHE_BINARY": str(Path(sys.executable).resolve()),
            "SCCACHE_DIR": str(self.root / "cache"),
            "SCCACHE_CONF": str(self.configuration),
            "SCCACHE_CACHE_SIZE": "100G",
            "XDG_STATE_HOME": str(self.root / "state"),
            "XDG_RUNTIME_DIR": str(self.root / "runtime"),
            "XDG_CONFIG_HOME": str(self.root / "config"),
        }

    def service(self, owner: subprocess.Popen[bytes]) -> cache.Service:
        return cache.Service(
            {"socket": str(self.root / "cache.sock")},
            owner.pid,
            "start",
            "a" * 32,
            "/owned",
            (1, 2),
        )

    def owner(self) -> subprocess.Popen[bytes]:
        owner = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
        self.addCleanup(lambda: owner.wait(timeout=5))
        self.addCleanup(lambda: owner.terminate() if owner.poll() is None else None)
        return owner

    def client(self, *, exits: bool) -> Path:
        binary = self.root / "sccache"
        child_pid = self.root / "child.pid"
        binary.write_text(
            f"#!{sys.executable}\n"
            "import os, subprocess, sys, time\n"
            "assert os.environ['SCCACHE_NO_DAEMON'] == '1'\n"
            "assert os.environ['SCCACHE_CLIENT_SIDE'] == '1'\n"
            "child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)'])\n"
            f"open({str(child_pid)!r}, 'w').write(str(child.pid))\n"
            + ("os._exit(0)\n" if exits else "time.sleep(60)\n")
        )
        binary.chmod(0o700)
        return binary

    def assert_child_drained(self) -> None:
        pid = int((self.root / "child.pid").read_text())
        end = time.monotonic() + 2
        while alive(pid) and time.monotonic() < end:
            time.sleep(0.01)
        assert not alive(pid), (
            "raced foreground cache child escaped its caller lifetime"
        )

    def test_actual_service_death_refuses_and_drains_foreground_fallback_group(
        self,
    ) -> None:
        owner = self.owner()
        binary = self.client(exits=False)
        timer = threading.Timer(0.25, owner.terminate)
        timer.start()
        self.addCleanup(timer.join)
        with (
            patch.object(cache, "same", return_value=True),
            patch.object(cache, "real_binary", return_value=binary),
            pytest.raises(cache.CacheError, match="service died"),
        ):
            cache.compile_command(["compiler"], self.env, self.service(owner))
        self.assert_child_drained()

    def test_actual_client_completion_drains_unexpected_foreground_fallback(
        self,
    ) -> None:
        owner = self.owner()
        binary = self.client(exits=True)
        with (
            patch.object(cache, "same", return_value=True),
            patch.object(cache, "real_binary", return_value=binary),
        ):
            assert (
                cache.compile_command(["compiler"], self.env, self.service(owner)) == 0
            )
        self.assert_child_drained()

    def test_actual_caller_signal_drains_fallback_and_preserves_signal_status(
        self,
    ) -> None:
        owner = self.owner()
        binary = self.client(exits=False)
        timer = threading.Timer(0.25, lambda: os.kill(os.getpid(), signal.SIGTERM))
        timer.start()
        self.addCleanup(timer.join)
        previous = signal.getsignal(signal.SIGTERM)
        with (
            patch.dict(os.environ, self.env, clear=True),
            patch.object(cache, "ensure", return_value=self.service(owner)),
            patch.object(cache, "same", return_value=True),
            patch.object(cache, "real_binary", return_value=binary),
        ):
            assert cache.main(["compiler"]) == 128 + signal.SIGTERM
        assert signal.getsignal(signal.SIGTERM) == previous
        self.assert_child_drained()

    def test_identity_change_after_success_refuses_result(self) -> None:
        owner = self.owner()
        binary = self.client(exits=True)
        with (
            patch.object(cache, "same", side_effect=[True, True, False]),
            patch.object(cache, "real_binary", return_value=binary),
            pytest.raises(cache.CacheError, match="identity changed after"),
        ):
            cache.compile_command(["compiler"], self.env, self.service(owner))
        self.assert_child_drained()

    def test_daemon_environment_scrubs_all_job_and_secret_authorities(self) -> None:
        record = {
            "cache": "/cache",
            "cache_size": "100G",
            "configuration": "/config",
            "socket": "/socket",
        }
        with patch.dict(
            os.environ,
            {
                "PSE_HOST_ALLOCATION": "job",
                "PSE_NATIVE_OPERATION": "native",
                "PSE_NATIVE_OPERATION_HANDOFF": "handoff",
                "AWS_SECRET_ACCESS_KEY": "secret",
                "SCCACHE_ERROR_LOG": "/secret",
                "RUSTC_WRAPPER": "job-wrapper",
            },
        ):
            selected = cache.daemon_environment(record)
        assert selected["SCCACHE_IDLE_TIMEOUT"] == "0"
        assert selected["SCCACHE_NO_DAEMON"] == "1"
        assert selected["SCCACHE_CLIENT_SIDE"] == "0"
        for key in (
            "PSE_HOST_ALLOCATION",
            "PSE_NATIVE_OPERATION",
            "PSE_NATIVE_OPERATION_HANDOFF",
            "AWS_SECRET_ACCESS_KEY",
            "SCCACHE_ERROR_LOG",
            "RUSTC_WRAPPER",
        ):
            assert key not in selected

    def test_foreground_explicit_endpoint_and_administrative_calls_do_not_ensure_resident(
        self,
    ) -> None:
        for arguments, extra in (
            ([], {"SCCACHE_START_SERVER": "1", "SCCACHE_NO_DAEMON": "1"}),
            (["--show-stats"], {}),
            (["--stop-server"], {}),
        ):
            with (
                self.subTest(arguments=arguments, extra=extra),
                patch.dict(
                    os.environ,
                    {
                        **self.env,
                        **extra,
                        "SCCACHE_SERVER_UDS": "/explicit/measurement.sock",
                    },
                    clear=True,
                ),
                patch.object(cache, "ensure") as ensure,
                patch.object(os, "execve") as execute,
            ):
                assert cache.main(arguments) == 0
                ensure.assert_not_called()
                assert (
                    execute.call_args.args[2]["SCCACHE_SERVER_UDS"]
                    == "/explicit/measurement.sock"
                )

    def test_admin_missing_endpoint_refuses_without_guessing_global_server(
        self,
    ) -> None:
        with (
            patch.dict(os.environ, self.env, clear=True),
            patch.object(os, "execve") as execute,
            contextlib.redirect_stderr(io.StringIO()),
        ):
            assert cache.main(["--stop-server"]) == 125
        execute.assert_not_called()

    def test_real_binary_refuses_wrapper_script(self) -> None:
        self.env["PSE_SCCACHE_BINARY"] = str(self.client(exits=True))
        with pytest.raises(cache.CacheError, match="ELF"):
            cache.real_binary(self.env)

    def test_readiness_requires_actual_executable_group_registration_and_socket_peer(
        self,
    ) -> None:
        endpoint = self.root / "owner.sock"
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.bind(str(endpoint))
            endpoint.chmod(0o700)
            binary = Path(sys.executable).resolve()
            record = {
                "version": 1,
                "unit": "owned.service",
                "binary": str(binary),
                "binary_identity": list(cache.identity(binary)),
                "socket": str(endpoint),
            }
            observation = {
                "MainPID": str(os.getpid()),
                "ActiveState": "active",
                "InvocationID": "a" * 32,
                "ControlGroup": "/owned",
            }
            with (
                patch.object(cache, "observe", return_value=observation),
                patch.object(cache.operation, "process_group", return_value="/owned"),
                patch.object(cache, "peer", return_value=os.getpid()),
                patch.object(cache, "registered", return_value=True),
            ):
                assert cache.ready(record, self.env, time.monotonic() + 1) is not None
                with patch.object(cache, "peer", return_value=os.getpid() + 1):
                    assert cache.ready(record, self.env, time.monotonic() + 1) is None
                with patch.object(cache, "registered", return_value=False):
                    assert cache.ready(record, self.env, time.monotonic() + 1) is None
                with patch.object(
                    cache.operation, "process_group", return_value="/job"
                ):
                    assert cache.ready(record, self.env, time.monotonic() + 1) is None

    def test_live_foreign_endpoint_refuses_readmission_without_unlink_or_allocation(
        self,
    ) -> None:
        binary = cache.real_binary(self.env)
        signature = cache.hashlib.sha256(
            cache.json.dumps(
                [
                    str(Path(self.env["SCCACHE_DIR"]).resolve()),
                    cache.digest(binary),
                    cache.digest(self.configuration),
                    "100G",
                ]
            ).encode()
        ).hexdigest()[:16]
        state = (
            Path(self.env["XDG_STATE_HOME"]) / "pse-arrow/compiler-cache" / signature
        )
        cache.private(state)
        endpoint = self.root / "foreign.sock"
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as foreign:
            foreign.bind(str(endpoint))
            foreign.listen()
            endpoint.chmod(0o700)
            original = endpoint.stat().st_ino
            cache.operation.write_json(
                state / "current.json", {"unit": "old.service", "socket": str(endpoint)}
            )
            with (
                patch.object(cache, "ready", return_value=None),
                patch.object(cache, "observe", return_value={"MainPID": "0"}),
                patch.object(cache.host, "acquire") as acquire,
                pytest.raises(
                    cache.CacheError, match="original startup clock exhausted"
                ),
            ):
                cache.ensure(self.env, deadline=time.monotonic() + 0.05)
            assert endpoint.stat().st_ino == original
            acquire.assert_not_called()

    def test_expired_startup_clock_refuses_before_metadata_or_host_effects(
        self,
    ) -> None:
        with (
            patch.object(cache, "metadata") as metadata,
            patch.object(cache.host, "acquire") as acquire,
            pytest.raises(cache.CacheError, match="original startup clock exhausted"),
        ):
            cache.ensure(self.env, deadline=time.monotonic() - 1)
        metadata.assert_not_called()
        acquire.assert_not_called()

    def test_changed_startup_inputs_refuse_manual_restart_before_handoff(self) -> None:
        binary = Path(sys.executable).resolve()
        path = self.root / "service.json"
        record = {
            "record": str(path),
            "binary": str(binary),
            "binary_identity": list(cache.identity(binary)),
            "binary_sha256": cache.digest(binary),
            "configuration": str(self.configuration),
            "configuration_sha256": cache.digest(self.configuration),
            "startup_inputs": {"old-owner": "old"},
        }
        cache.operation.write_json(path, record)
        with (
            patch.object(cache.host, "inherit") as inherit,
            patch.object(os, "execve") as execute,
            pytest.raises(cache.CacheError, match="startup admission inputs changed"),
        ):
            cache.serve(path)
        inherit.assert_not_called()
        execute.assert_not_called()

    def test_service_publication_keeps_metadata_lock_outside_host_and_manager_ipc(
        self,
    ) -> None:
        profile = Mock(memory=2 * 1024**3, cores=(8,))
        allocation = Mock(profile=profile, nonce="a" * 32)
        allocation.environment.return_value = {
            cache.host.MARKER: "/allocations/" + "a" * 32
        }
        callbacks = 0

        def unlocked(*_arguments: object, **_keywords: object) -> None:
            nonlocal callbacks
            callbacks += 1
            locks = list((self.root / "state/pse-arrow/compiler-cache").glob("*/.lock"))
            assert len(locks) == 1
            with locks[0].open("r") as lock:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)

        def manager(*arguments: object) -> subprocess.CompletedProcess[str]:
            unlocked(*arguments)
            return subprocess.CompletedProcess([], 0, "", "")

        def ready(record: dict[str, object], *_arguments: object) -> cache.Service:
            unlocked()
            return cache.Service(
                record, os.getpid(), "start", "a" * 32, "/owned", (1, 2)
            )

        with (
            patch.object(cache.host, "select", return_value=profile),
            patch.object(cache.host, "acquire", return_value=allocation) as acquire,
            patch.object(cache.host, "enforce_parent", side_effect=unlocked),
            patch.object(cache.host, "enforce_allocation", side_effect=unlocked),
            patch.object(cache.host, "cpu_set", return_value=(8, 24)),
            patch.object(cache, "manager", side_effect=manager),
            patch.object(cache, "ready", side_effect=ready),
        ):
            original_deadline = time.monotonic() + 30
            service = cache.ensure(self.env, deadline=original_deadline)
        assert callbacks >= 5
        assert acquire.call_args.kwargs["deadline"] == original_deadline
        assert service.endpoint != self.env.get("SCCACHE_SERVER_UDS")
        assert service.endpoint.endswith(".sock")
        allocation.register.assert_called_once()
        units = list((self.root / "config/systemd/user").glob("*.service"))
        assert len(units) == 1
        unit = units[0].read_text()
        assert "Type=exec\nRestart=no" in unit
        assert "MemoryMax=2147483648" in unit
        assert "CPUAffinity=8 24" in unit
        assert "PIDFile" not in unit


if __name__ == "__main__":
    unittest.main()
