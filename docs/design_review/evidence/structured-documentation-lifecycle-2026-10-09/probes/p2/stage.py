"""Stage the live docs corpus into a scratch directory using scripts/docs.py's own discover/stage."""
import subprocess
import sys
from pathlib import Path
ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip())
sys.path.insert(0, str(ROOT))
sys.dont_write_bytecode = True
from scripts import docs  # noqa: E402

dest = Path(sys.argv[1])
config = docs.configuration(ROOT)
pages = docs.discover(ROOT, config)
pages = docs.stage(ROOT, dest, pages, config)
print(f"staged {len(pages)} pages into {dest}")
