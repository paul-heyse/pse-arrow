# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real interpreter signals preserve each synchronous owner's completion contract."""

import shlex
import subprocess
import sys
import sysconfig
from pathlib import Path

import pytest


@pytest.mark.unit
@pytest.mark.parametrize("mode", ["capture", "command"])
def test_signal_wait_keeps_original_exception_and_distinct_stop_policy(
    tmp_path: Path, mode: str
) -> None:
    # Isolate the real signal handler from pytest/xdist and retain actual native owners.
    source = r"""
import os
import signal
import sys
import threading
import time
from pathlib import Path
import pse
import msgspec
from pse import diagnostics
from pse.contracts.documents import (
    CaptureBudget, CaptureRequest, ClosedDuration, LaunchCaptureRequest,
)

mode, output = sys.argv[1:]
runtime = None
reader = None
command = None
if mode == "capture":
    from pse.tests.test_diagnostics import observed_runtime
    from pse.contracts import documents
    from pse.contracts.enums import CaptureKind
    settings = pse.EngineSettings(
        memory_limit_bytes=64 << 30, threads=2, target_partitions=1,
        spill_dir=str(Path(output) / "spill"), max_spill_bytes=1 << 30, batch_size=7,
        cache=pse.CacheSettings(working_bytes=16 << 30, concurrent_loads=2,
            inflight_bytes=64 << 20, inspection_bytes=4 << 20),
    )
    runtime, observations, reader = observed_runtime(settings)
    before = reader.snapshot()
    deadline = time.monotonic() + 5
    while True:
        try:
            capture = diagnostics.capture(CaptureRequest(
                version=1, checkpoint=str(observations.checkpoint),
                operation=msgspec.convert(
                    reader.id, type=documents.OperationIdBody, from_attributes=True,
                ),
                kind=CaptureKind.TIMELINE, output=str(Path(output) / mode),
                duration=ClosedDuration(secs=10, nanos=0),
                budget=CaptureBudget(memory_bytes=16 << 20, output_bytes=256 << 20),
            ))
            break
        except pse.InspectionError:
            assert time.monotonic() < deadline
            time.sleep(0.02)
else:
    command, capture = diagnostics.launch_capture(LaunchCaptureRequest(
        version=1, argv=("/bin/sh", "-c", "sleep 60 & wait"), cwd=None,
        output=str(Path(output) / mode), duration=ClosedDuration(secs=10, nanos=0),
        startup_timeout=ClosedDuration(secs=5, nanos=0),
        budget=CaptureBudget(memory_bytes=4 << 20, output_bytes=1 << 20),
    ))
cause = RuntimeError("original signal cause")
original = KeyboardInterrupt("original wait interruption")
original.__cause__ = cause

def interrupted(number, frame):
    raise original

previous = signal.signal(signal.SIGINT, interrupted)
# This Python thread can run only when the native waiter releases the interpreter.
timer = threading.Timer(0.05, lambda: os.kill(os.getpid(), signal.SIGINT))
timer.start()
try:
    try:
        if mode == "capture":
            capture.wait()
        else:
            command.wait()
    except KeyboardInterrupt as error:
        assert error is original
        assert error.__cause__ is cause
    else:
        raise AssertionError("pending native wait ignored the interpreter signal")
    if mode == "capture":
        # Timeline needs no perf privilege; the original operation remains unchanged.
        assert reader.snapshot().outcome == before.outcome
        assert capture.snapshot().operation is not None
    else:
        # Command interruption asks that owner to cancel and retains its actual status.
        assert command.try_wait() == 137
finally:
    timer.cancel()
    timer.join(timeout=5)
    assert not timer.is_alive()
    signal.signal(signal.SIGINT, previous)
    try:
        capture.stop()
    finally:
        if command is not None:
            command.close()
        if runtime is not None:
            runtime.close()
if command is not None:
    assert command.try_wait() == 137
"""
    result = subprocess.run(
        [sys.executable, "-c", source, mode, str(tmp_path)],
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr


@pytest.mark.unit
def test_pending_cpython_signal_precheck_keeps_native_stream_unconsumed(
    tmp_path: Path,
) -> None:
    # The tiny test extension sets the C pending flag and calls the actual native
    # bound method without a Python bytecode checkpoint between those actions.
    source = tmp_path / "pending_signal_control.c"
    source.write_text(r"""#include <Python.h>
static int last_call_raised = 0;
static int probes = 0;
static PyObject *probe(PyObject *self, PyObject *unused) {
    probes += 1;
    Py_RETURN_NONE;
}
static PyObject *armed_call(PyObject *self, PyObject *callable) {
    if (!PyCFunction_Check(callable)) {
        PyErr_SetString(PyExc_TypeError, "control requires a native bound callable");
        return NULL;
    }
    last_call_raised = 0;
    PyErr_SetInterrupt();
    PyObject *result = PyObject_CallNoArgs(callable);
    last_call_raised = result == NULL &&
        PyErr_ExceptionMatches(PyExc_KeyboardInterrupt);
    return result;
}
static PyObject *last_error(PyObject *self, PyObject *unused) {
    return PyBool_FromLong(last_call_raised);
}
static PyObject *probe_count(PyObject *self, PyObject *unused) {
    return PyLong_FromLong(probes);
}
static PyMethodDef methods[] = {
    {"armed_call", armed_call, METH_O, NULL},
    {"probe", probe, METH_NOARGS, NULL},
    {"last_error", last_error, METH_NOARGS, NULL},
    {"probe_count", probe_count, METH_NOARGS, NULL},
    {NULL, NULL, 0, NULL}
};
static struct PyModuleDef module = {
    PyModuleDef_HEAD_INIT, "_pending_signal_control", NULL, -1, methods
};
PyMODINIT_FUNC PyInit__pending_signal_control(void) { return PyModule_Create(&module); }
""")
    suffix = sysconfig.get_config_var("EXT_SUFFIX")
    linker = sysconfig.get_config_var("LDSHARED")
    assert isinstance(suffix, str)
    assert isinstance(linker, str)
    compiled = subprocess.run(
        [
            *shlex.split(linker),
            *shlex.split(str(sysconfig.get_config_var("CFLAGS") or "")),
            *shlex.split(str(sysconfig.get_config_var("CCSHARED") or "")),
            f"-I{sysconfig.get_path('include')}",
            str(source),
            "-o",
            str(tmp_path / f"_pending_signal_control{suffix}"),
        ],
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    assert compiled.returncode == 0, compiled.stdout + compiled.stderr
    control = r"""
import signal
import sys
import threading
from pathlib import Path
sys.path.insert(0, sys.argv[1])
import _pending_signal_control as control
import pyarrow as pa
import pse
from pse._build import compiled_features
assert "force-validate" in compiled_features()
assert threading.current_thread() is threading.main_thread()
cause = RuntimeError("original pending signal cause")
original = KeyboardInterrupt("pending before native stream entry")
original.__cause__ = cause

def interrupted(number, frame):
    raise original

previous = signal.signal(signal.SIGINT, interrupted)
stream = None
reader = None
try:
    # Negative control proves CPython's direct native dispatch does not consume
    # the flag. This no-check native probe returns before Python raises it.
    try:
        control.armed_call(control.probe)
    except KeyboardInterrupt as error:
        assert error is original
    else:
        raise AssertionError("pending C signal was not delivered")
    assert control.probe_count() == 1
    assert control.last_error() is False
    settings = pse.EngineSettings(
        memory_limit_bytes=64 << 30, threads=2, target_partitions=1,
        spill_dir=str(Path(sys.argv[1]) / "spill"),
        max_spill_bytes=1 << 30, batch_size=7,
        cache=pse.CacheSettings(working_bytes=128 << 20, concurrent_loads=2,
            inflight_bytes=32 << 20, inspection_bytes=4 << 20),
    )
    stream = pse.registry_table("reference.schema_relations", settings=settings)
    try:
        control.armed_call(stream._handle.next_batch)
    except KeyboardInterrupt as error:
        assert error is original
        assert error.__cause__ is cause
    else:
        raise AssertionError("native stream precheck ignored the pending flag")
    # The native method itself returned the error, unlike the no-check probe.
    assert control.last_error() is True
    assert stream.failure is None
    # Obtaining the controlled reader would consume the same one-shot stream.
    # Successful capsule export proves precheck issued no reader/pull.
    reader = pa.RecordBatchReader.from_stream(stream)
    assert reader.read_next_batch().num_rows > 0
finally:
    signal.signal(signal.SIGINT, previous)
    if reader is not None:
        reader.close()
    if stream is not None:
        stream.close()
"""
    result = subprocess.run(
        [sys.executable, "-c", control, str(tmp_path)],
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr
