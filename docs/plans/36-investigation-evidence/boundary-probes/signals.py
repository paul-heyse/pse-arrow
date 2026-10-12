"""Isolated callback discriminator: no canonical store, solve or shared runtime."""
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import sysconfig
import time
import pyarrow as pa
import signal_proxy


def run(mode):
    er, ew = os.pipe()
    sr, sw = os.pipe()
    cr, cw = os.pipe()
    started = time.monotonic()
    batch = pa.record_batch([[7, 11]], names=['x'])
    reader = pa.RecordBatchReader.from_batches(batch.schema, [batch, batch])
    capsule = reader.__arrow_c_stream__()
    signal_proxy.install(capsule, ew, sw, 1.5)
    imported = pa.RecordBatchReader._import_from_c_capsule(capsule)
    retained = imported.read_next_batch()
    sender = subprocess.Popen([sys.executable, "-c", "import os,signal,sys,time; os.read(int(sys.argv[1]),1); time.sleep(.2); os.kill(int(sys.argv[2]),signal.SIGINT)", str(er), str(os.getpid())], pass_fds=(er,))
    result = None
    error = None
    try:
        if mode == 'entered_pull':
            result = imported.read_next_batch()
        elif mode == 'before_pull':
            os.write(ew, b'E')
            time.sleep(1.5)
            result = imported.read_next_batch()
        else:
            signal_proxy.bridge(ew, cw, sw, 1.5)
    except KeyboardInterrupt:
        error = 'KeyboardInterrupt'
    elapsed = time.monotonic() - started
    sender.wait(timeout=5)
    os.set_blocking(sr, False)
    os.set_blocking(cr, False)
    def read(fd):
        try:
            return os.read(fd, 10).decode()
        except BlockingIOError:
            return ''
    settled, cancelled = read(sr), read(cr)
    imported.close()
    preserved = retained.column(0).to_pylist() == [7, 11]
    assert error == 'KeyboardInterrupt' and preserved
    if mode == 'entered_pull':
        assert elapsed >= 1.45 and settled == 'S' and cancelled == ''
    elif mode == 'before_pull':
        assert elapsed < 1.0 and settled == '' and cancelled == ''
    else:
        assert elapsed < 1.25 and settled == 'S' and cancelled == 'C'
    print(json.dumps({'case': mode, 'exception': error, 'elapsed_seconds': round(elapsed, 3), 'native_settlement': settled, 'cancellation_callback': cancelled, 'returned_batch': result is not None, 'retained_yielded_arrays_after_close': preserved, 'canonical_protection_release': 'not_exercised'}), flush=True)


if __name__ == '__main__':
    if len(sys.argv) == 2:
        run(sys.argv[1])
    else:
        import pse._native as native
        print(json.dumps({'python': sys.version, 'pyarrow': pa.__version__, 'native_path': native.__file__, 'cfg_force_validation': 'unknown-no-exported-cfg-report', 'build_info': str(native.build_info())}), flush=True)
        for mode in ('entered_pull', 'before_pull', 'ordinary_bridge'):
            subprocess.run([sys.executable, __file__, mode], check=True)
