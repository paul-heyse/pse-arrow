import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from typed import rewrite
src, dst = Path(sys.argv[1]), Path(sys.argv[2])
dst.write_text(rewrite(src.read_text()))
