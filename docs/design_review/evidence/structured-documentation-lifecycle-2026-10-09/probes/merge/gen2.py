"""Discriminating scenarios: adjacent-element edits (S3) and same-element different-field edits (S4)."""
import json, subprocess, shutil
from pathlib import Path
EDITS = {
    "S3-adjacent-elements": (("EFF10 emitted-query and implementation acceptance remain.", "EFF10 emitted-query acceptance remains."),
                             ("affected integration remains.", "affected integration passes.")),
    "S4-same-element-two-fields": (("affected integration remains.", "affected integration passes."),
                                   ("Remove obsolete scope/reuse assumptions.", "Remove obsolete scope/reuse assumptions and their tests.")),
}
res = {}
for scen, ((a0, a1), (b0, b1)) in EDITS.items():
    for base in sorted(Path("S1-status-vs-new-row").glob("*/base.*")):
        cand = base.parent.name
        d = Path(scen) / cand; d.mkdir(parents=True, exist_ok=True)
        text = base.read_text(); ext = base.suffix
        assert text.count(a0) == 1 and text.count(b0) == 1, (cand, scen)
        (d / f"base{ext}").write_text(text)
        (d / f"ours{ext}").write_text(text.replace(a0, a1))
        (d / f"theirs{ext}").write_text(text.replace(b0, b1))
        shutil.copy(d / f"ours{ext}", d / f"merged{ext}")
        r = subprocess.run(["git", "merge-file", "-L", "ours", "-L", "base", "-L", "theirs", str(d / f"merged{ext}"), str(d / f"base{ext}"), str(d / f"theirs{ext}")], capture_output=True, text=True)
        merged = (d / f"merged{ext}").read_text()
        res.setdefault(cand, {})[scen] = {"conflicts": r.returncode, "both_edits_present": (a1 in merged and b1 in merged) if r.returncode == 0 else None}
print(json.dumps(res, indent=1))
