"""Probe: one plan slice in four encodings; edit, merge, query and render behaviour.

Run: venv/bin/python -I formats/gen.py <out-dir>
"""

from __future__ import annotations

import io
import json
import subprocess
import sys
import tomllib
from pathlib import Path

import tomlkit
from ruamel.yaml import YAML
from ruamel.yaml.scalarstring import LiteralScalarString

OUT = Path(sys.argv[1])
OUT.mkdir(parents=True, exist_ok=True)

P01_BODY = (
    "Record required ADR/review/design amendments, derived-state cutover boundaries and\n"
    "analysis admission/fencing design. Reconcile fixture provenance and external versus\n"
    'invalid internal comparisons. Publish shared decisions before "dependent" code.\n'
    "\n"
    "- Acceptance: emitted-query controls pass.\n"
    "- Consumers: EFF10 fencing.\n"
)
P02_BODY = (
    "Expand scope/effective configuration and bump interpretation. Detect every reviewed\n"
    "omission with positive controls; exercise actual unchanged-input refusal.\n"
    "\n"
    "Remove obsolete scope/reuse assumptions.\n"
)
CHECKPOINT = (
    "Execution is in progress. EFF01 binds the scripts/configuration/benchmark closure and\n"
    "effective native configuration under input interpretation 3.\n"
    "\n"
    "ADR-0169–0172 have target-design Accept verdicts at Proposed strength; those verdicts\n"
    "do not qualify product mechanisms.\n"
)
PACKETS = [
    ("EFF00", "Adopt affected contracts", "in progress: EFF10 emitted-query acceptance remains", P01_BODY),
    ("EFF01", "Complete applicability (F02)", "implemented; affected integration remains", P02_BODY),
]

# --- (1) light Markdown: heading-per-element, invisible kind markers, state as a list field
md = ["---", "title: Fixture plan", "status: in-progress", "baseline: 4c24721e6", "---", "",
      "# Fixture plan", "", "## Packets <!-- kind: packets -->", ""]
for pid, title, state, body in PACKETS:
    md += [f"### {pid} — {title} {{#{pid.lower()}}}", "", f"- State: {state}", "", body.rstrip("\n"), ""]
md += ["## Current checkpoint <!-- kind: checkpoint -->", "", CHECKPOINT.rstrip("\n"), ""]
(OUT / "plan.md").write_text("\n".join(md), encoding="utf-8")

# --- (2) YAML with literal block scalars (ruamel round-trip, explicit indent)
yaml = YAML()
yaml.indent(mapping=2, sequence=4, offset=2)
ydoc = {
    "title": "Fixture plan",
    "status": "in-progress",
    "packets": [
        {"id": pid, "title": t, "state": st, "body": LiteralScalarString(b)} for pid, t, st, b in PACKETS
    ],
    "checkpoint": LiteralScalarString(CHECKPOINT),
}
buf = io.StringIO()
yaml.dump(ydoc, buf)
(OUT / "plan.yaml").write_text(buf.getvalue(), encoding="utf-8")

# --- (3) TOML with multi-line literal strings (no escapes, no indentation)
toml = ['title = "Fixture plan"', 'status = "in-progress"', ""]
for pid, t, st, b in PACKETS:
    toml += ["[[packets]]", f'id = "{pid}"', f'title = "{t}"', f'state = "{st}"', "body = '''", b + "'''", ""]
toml += ["checkpoint = '''", CHECKPOINT + "'''", ""]
# top-level key after arrays of tables would bind to the last table, so move checkpoint up
toml_text = "\n".join(toml[:3] + ["checkpoint = '''", CHECKPOINT + "'''", ""] + toml[3:-3])
(OUT / "plan.toml").write_text(toml_text, encoding="utf-8")
assert tomllib.loads(toml_text)["packets"][0]["body"] == P01_BODY

# --- (4) JSON (as an editor-schema'd file would be)
jdoc = {"$schema": "./plan.schema.json", "title": "Fixture plan", "status": "in-progress",
        "packets": [{"id": p, "title": t, "state": s, "body": b} for p, t, s, b in PACKETS],
        "checkpoint": CHECKPOINT}
(OUT / "plan.json").write_text(json.dumps(jdoc, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

# --- exact-string edit: can the agent's old_string (copied from the prose as rendered) be found?
SENT_A = "derived-state cutover boundaries and\nanalysis admission/fencing design"
SENT_Q = 'before "dependent" code'
print("== exact-string findability of a 2-line prose span / a quoted phrase")
for f in ("plan.md", "plan.yaml", "plan.toml", "plan.json"):
    text = (OUT / f).read_text(encoding="utf-8")
    print(f"{f:10} two-line span: {SENT_A in text!s:5}  quoted phrase: {SENT_Q in text!s:5}  "
          f"lines={text.count(chr(10))} bytes={len(text.encode())}")

# --- merge: two agents edit different paragraphs of the same packet body
def edit(text: str, which: int, f: str) -> str:
    a = ("Publish shared decisions", "Publish agreed shared decisions")
    b = ("- Consumers: EFF10 fencing.", "- Consumers: EFF10 fencing and EFF11 closure.")
    old, new = (a, b)[which]
    if f == "plan.json":
        old, new = json.dumps(old)[1:-1], json.dumps(new)[1:-1]
    assert old in text, (f, old)
    return text.replace(old, new, 1)

def state_edit(text: str, f: str, pid: str) -> str:
    old = {"EFF00": "in progress: EFF10 emitted-query acceptance remains",
           "EFF01": "implemented; affected integration remains"}[pid]
    return text.replace(old, "done", 1)

print("== git merge-file (exit = number of conflicts)")
for f in ("plan.md", "plan.yaml", "plan.toml", "plan.json"):
    base = (OUT / f).read_text(encoding="utf-8")
    for name, ours, theirs in (
        ("S-a body para 1 || body para 2 (same element)", edit(base, 0, f), edit(base, 1, f)),
        ("S-b state EFF00 || state EFF01 (adjacent elements)", state_edit(base, f, "EFF00"), state_edit(base, f, "EFF01")),
        ("S-c state EFF00 || body EFF00", state_edit(base, f, "EFF00"), edit(base, 0, f)),
    ):
        d = OUT / "merge" / f / name.split()[0]
        d.mkdir(parents=True, exist_ok=True)
        for n, t in (("base", base), ("ours", ours), ("theirs", theirs)):
            (d / n).write_text(t, encoding="utf-8")
        r = subprocess.run(["git", "merge-file", "-p", str(d / "ours"), str(d / "base"), str(d / "theirs")],
                           capture_output=True, text=True)
        print(f"{f:10} {name:52} conflicts={r.returncode}")

# --- programmatic state edit: byte stability of round-trip writers
print("== programmatic round-trip edit of one state field")
y = YAML(); y.indent(mapping=2, sequence=4, offset=2); y.preserve_quotes = True
src = (OUT / "plan.yaml").read_text(encoding="utf-8")
d = y.load(src); d["packets"][0]["state"] = "done"; b = io.StringIO(); y.dump(d, b)
changed = [l for l in zip(src.splitlines(), b.getvalue().splitlines()) if l[0] != l[1]]
print(f"ruamel 0.19.1: changed lines={len(changed)} total-line-delta={b.getvalue().count(chr(10)) - src.count(chr(10))}")
src = (OUT / "plan.toml").read_text(encoding="utf-8")
d = tomlkit.parse(src); d["packets"][0]["state"] = "done"; out = tomlkit.dumps(d)
changed = [l for l in zip(src.splitlines(), out.splitlines()) if l[0] != l[1]]
print(f"tomlkit 0.15.1: changed lines={len(changed)} total-line-delta={out.count(chr(10)) - src.count(chr(10))}")
