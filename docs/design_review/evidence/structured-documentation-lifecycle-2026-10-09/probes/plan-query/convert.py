"""Probe: mechanical on-demand conversion. Appends `<!-- kind: X -->` to level-2 headings whose
legacy title alias matched; touches only those lines; writes to a new path."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import planq
src, dst = Path(sys.argv[1]), Path(sys.argv[2])
fm, lines, secs = planq.parse(src)
out = list(lines)
for s in secs:
    if s.kind_source == "legacy-title":
        line = out[s.start].rstrip("\n")
        out[s.start] = f"{line} <!-- kind: {s.kind} -->\n"
dst.write_text("".join(out), encoding="utf-8")
