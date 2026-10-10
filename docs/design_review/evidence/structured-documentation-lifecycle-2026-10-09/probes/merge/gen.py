"""Generate candidate encodings of one Plan 33 slice (exact live prose) and run two concurrent-edit merges."""
import re, subprocess, json, shutil
from pathlib import Path
ROOT = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip())
PLAN = (ROOT / "docs/plans/33-efficiency-principles-remediation.md").read_text()
def row(anchor):
    return next(l for l in PLAN.splitlines() if l.startswith(f'| <a id="{anchor}"></a>'))
def cells(r):
    return [c.strip() for c in re.split(r"(?<!\\)\|", r.strip())[1:-1]]
F02 = next(l for l in PLAN.splitlines() if l.startswith("| [F02]"))
rows = {a: row(a) for a in ["eff00", "eff01", "ei01", "ei06"]}
C = {a: cells(r) for a, r in rows.items()}
F = cells(F02)
NEW_EFF01 = "Complete; affected integration passes under input interpretation 3."
EI07 = ("EI07 — Example added investigation", "Example question added concurrently by a second agent.", "Example integration and acceptance.")
EI08 = ("EI08 — Second example investigation", "Example question added concurrently by a third agent.", "Example integration and acceptance.")

def title_split(c0):  # '<a id="eff01"></a>EFF01 — Complete applicability (F02)'
    m = re.match(r'<a id="([^"]+)"></a>(\S+) — (.*)', c0)
    return m.group(1), m.group(2), m.group(3)

# ---------- (i) baseline: live tables verbatim ----------
def baseline(eff01_progress=None, extra=()):
    eff01 = rows["eff01"] if eff01_progress is None else "| " + " | ".join(C["eff01"][:3] + [eff01_progress]) + " |"
    ei_extra = "".join(f'| <a id="{t.split()[0].lower()}"></a>{t} | {q} | {i} |\n' for t, q, i in extra)
    return f"""## Implementation packets and dependencies

| Packet | Required input | Delivery and targeted acceptance | Progress |
|---|---|---|---|
{rows['eff00']}
{eff01}

## Bounded investigation packets

| Packet | Question, evidence and decision boundary | Integration and acceptance |
|---|---|---|
{rows['ei01']}
{rows['ei06']}
{ei_extra}
## Finding dispositions

| Finding | Scenario | Disposition | Decision/work owner | Evidence or completion condition |
|---|---|---|---|---|
{F02}
"""

# ---------- (ii) simplest viable: same tables, header 'Status', closed state token first ----------
def token_cell(cell, token):
    return f"`{token}` {cell}"
def simple(eff01_state=None, extra=()):
    e0 = C["eff00"][:3] + [token_cell(C["eff00"][3].removeprefix("In progress: "), "in-progress")]
    e1 = C["eff01"][:3] + ([token_cell(C["eff01"][3].removeprefix("Implemented; "), "implemented")] if eff01_state is None else [token_cell(eff01_state, "complete")])
    ei = lambda c: c + ["`planned`"]
    ei_extra = "".join(f'| <a id="{t.split()[0].lower()}"></a>{t} | {q} | {i} | `planned` |\n' for t, q, i in extra)
    f = F[:2] + [token_cell("", "scheduled").strip()] + [f"[EFF01](#eff01)"] + [F[4]]
    return f"""## Implementation packets and dependencies

| Packet | Required input | Delivery and targeted acceptance | Status |
|---|---|---|---|
| {' | '.join(e0)} |
| {' | '.join(e1)} |

## Bounded investigation packets

| Packet | Question, evidence and decision boundary | Integration and acceptance | Status |
|---|---|---|---|
| {' | '.join(ei(C['ei01']))} |
| {' | '.join(ei(C['ei06']))} |
{ei_extra}
## Finding dispositions

| Finding | Scenario | Status | Decision/work owner | Evidence or completion condition |
|---|---|---|---|---|
| {' | '.join(f)} |
"""

# ---------- (iii) typed blocks (P1 syntax: attributes in fence info), prose bodies ----------
def block(fam, ident, attrs, title, paras):
    a = " ".join([f"#{ident}"] + [f"{k}={v}" if re.fullmatch(r"[\w.:-]+", v) else f'{k}="{v}"' for k, v in attrs.items()])
    body = "\n\n".join(paras)
    return f"::: {fam} {title} {{{a}}}\n{body}\n:::\n"
def typed(eff01_state=None, extra=()):
    out = ["## Implementation packets and dependencies\n"]
    for key, state, note in [("eff00", "in-progress", C["eff00"][3].removeprefix("In progress: ")),
                             ("eff01", "implemented" if eff01_state is None else "complete",
                              C["eff01"][3].removeprefix("Implemented; ") if eff01_state is None else eff01_state)]:
        _, ident, title = title_split(C[key][0])
        out.append(block("packet", ident, {"state": state}, title,
                         [f"**Required input.** {C[key][1]}", f"**Delivery and targeted acceptance.** {C[key][2]}", f"**Progress.** {note}"]))
    out.append("## Bounded investigation packets\n")
    for key in ["ei01", "ei06"]:
        _, ident, title = title_split(C[key][0])
        out.append(block("investigation", ident, {"state": "planned"}, title,
                         [f"**Question, evidence and decision boundary.** {C[key][1]}", f"**Integration and acceptance.** {C[key][2]}"]))
    for t, q, i in extra:
        ident, title = t.split(" — ")
        out.append(block("investigation", ident, {"state": "planned"}, title,
                         [f"**Question, evidence and decision boundary.** {q}", f"**Integration and acceptance.** {i}"]))
    out.append("## Finding dispositions\n")
    out.append(block("disposition", "d-f02", {"finding": "review:efficiency-principles-codebase#f02", "scenario": "S01", "state": "scheduled", "owner": "EFF01"},
                     "F02", [F[4]]))
    return "\n".join(out)

# ---------- (iv) canonical YAML with Markdown block scalars ----------
def ylit(s, ind):
    return "|\n" + "\n".join(" " * ind + l for l in s.splitlines()) + "\n"
def yaml_(eff01_state=None, extra=()):
    o = ["packets:\n"]
    for key, state, note in [("eff00", "in-progress", C["eff00"][3].removeprefix("In progress: ")),
                             ("eff01", "implemented" if eff01_state is None else "complete",
                              C["eff01"][3].removeprefix("Implemented; ") if eff01_state is None else eff01_state)]:
        _, ident, title = title_split(C[key][0])
        o.append(f"  - id: {ident}\n    title: {title}\n    state: {state}\n    required_input: {ylit(C[key][1], 6)}    delivery: {ylit(C[key][2], 6)}    progress: {ylit(note, 6)}")
    o.append("investigations:\n")
    items = [(title_split(C[k][0])[1], title_split(C[k][0])[2], C[k][1], C[k][2]) for k in ["ei01", "ei06"]]
    items += [(t.split(" — ")[0], t.split(" — ")[1], q, i) for t, q, i in extra]
    for ident, title, q, i in items:
        o.append(f"  - id: {ident}\n    title: {title}\n    state: planned\n    question: {ylit(q, 6)}    integration: {ylit(i, 6)}")
    o.append("dispositions:\n")
    o.append(f"  - finding: review:efficiency-principles-codebase#f02\n    scenario: S01\n    state: scheduled\n    owner: EFF01\n    condition: {ylit(F[4], 6)}")
    return "".join(o)

CANDS = {"i-baseline.md": baseline, "ii-simple.md": simple, "iii-typed.md": typed, "iv-canonical.yaml": yaml_}
results = {}
for name, fn in CANDS.items():
    for scen, (ours, theirs) in {
        "S1-status-vs-new-row": ({"eff01_state" if fn is not baseline else "eff01_progress": NEW_EFF01}, {"extra": (EI07,)}),
        "S2-two-new-rows": ({"extra": (EI07,)}, {"extra": (EI08,)}),
    }.items():
        d = Path(scen) / name.split(".")[0]
        d.mkdir(parents=True, exist_ok=True)
        ext = "." + name.split(".")[1]
        (d / f"base{ext}").write_text(fn())
        (d / f"ours{ext}").write_text(fn(**ours))
        (d / f"theirs{ext}").write_text(fn(**theirs))
        shutil.copy(d / f"ours{ext}", d / f"merged{ext}")
        r = subprocess.run(["git", "merge-file", "-L", "ours", "-L", "base", "-L", "theirs",
                            str(d / f"merged{ext}"), str(d / f"base{ext}"), str(d / f"theirs{ext}")],
                           capture_output=True, text=True)
        results.setdefault(name, {})[scen] = {"exit_code_conflicts": r.returncode,
                                               "conflict_markers": (d / f"merged{ext}").read_text().count("<<<<<<<")}
print(json.dumps(results, indent=1))
