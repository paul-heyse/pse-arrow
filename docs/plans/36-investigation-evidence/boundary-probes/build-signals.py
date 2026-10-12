"""Build the isolated C-stream signal discriminator for this interpreter."""
from pathlib import Path
import subprocess
import sysconfig

root = Path(__file__).resolve().parent
subprocess.run(
    [
        "cc", "-shared", "-fPIC", "-O2",
        "-I" + sysconfig.get_path("include"),
        str(root / "signal-proxy.c"),
        "-o", str(root / ("signal_proxy" + sysconfig.get_config_var("EXT_SUFFIX"))),
    ],
    check=True,
)
