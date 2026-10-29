# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Codex session-log miner for the accompanying agent-first Rust CLI review.

Read-only. Reproduces the review's measured tables from Codex rollout files:

    python3 -I rust-agent-cli-log-miner-2026-10-29.py mine ~/.codex/sessions calls.jsonl \
        2026-10-28T22:58 2026-10-29T22:58
    python3 -I rust-agent-cli-log-miner-2026-10-29.py report calls.jsonl 2026-10-29T06:32

`mine` keeps sessions whose cwd is /home/paul/pse-arrow and records whose timestamp falls in
the window. It writes one JSON line per tool call (exec_command, write_stdin, apply_patch)
found in `custom_tool_call` "exec" cells. Calls are deduplicated by `call_id`, because
forked subagents copy their parent's records. Results are paired in call order only when a
cell returned exactly one JSON envelope per exec/poll call; other calls stay unpaired.
Cells that printed only raw output carry no exit code.

`report` prints the category table, background-run lifecycle, re-read and orientation
counts, and Rust-interaction statistics for the post-retirement sub-window. Categories are
heuristic first-segment classifications of each command.
"""

from __future__ import annotations

import collections
import glob
import json
import os
import re
import sys

CWD = '"cwd":"/home/paul/pse-arrow'
TOOL = re.compile(r"tools\.(exec_command|write_stdin|apply_patch)\(")
CMD = re.compile(r'\bcmd\s*:\s*("(?:[^"\\]|\\.)*")')
SID = re.compile(r"\bsession_id\s*:\s*(\d+)")
TRUNC = re.compile(r"truncated output \(original token count: (\d+)\)")
PATCH_ERROR = re.compile(r"(?i)(failed to (find|apply)|invalid patch|patch rejected)")


def parse_cell(code: str) -> list[dict[str, object]]:
    calls: list[dict[str, object]] = []
    for match in TOOL.finditer(code):
        kind = match.group(1)
        rest = code[match.end() : match.end() + 20000]
        if kind == "exec_command":
            found = CMD.search(rest)
            try:
                command = json.loads(found.group(1)) if found else None
            except ValueError:
                command = None
            calls.append({"kind": "exec", "cmd": command})
        elif kind == "write_stdin":
            found = SID.search(rest[:300])
            calls.append({"kind": "poll", "sid": int(found.group(1)) if found else None})
        else:
            calls.append({"kind": "patch"})
    return calls


def envelopes(output: object) -> tuple[list[dict[str, object]], str]:
    found: list[dict[str, object]] = []
    raw = ""
    if not isinstance(output, list):
        return found, raw
    for item in output:
        text = item.get("text", "") if isinstance(item, dict) else ""
        raw += text
        if not text.startswith("{"):
            continue
        try:
            value = json.loads(text)
        except ValueError:
            continue
        if isinstance(value, dict) and ("exit_code" in value or "session_id" in value or "chunk_id" in value):
            found.append(value)
    return found, raw


def mine(root: str, out_path: str, start: str, end: str) -> None:
    seen: set[str] = set()
    pending: dict[str, tuple[list[dict[str, object]], str]] = {}
    duplicates = 0
    written = 0
    with open(out_path, "w") as out:
        for path in sorted(glob.glob(os.path.join(root, "2026/10/*/*.jsonl"))):
            with open(path) as handle:
                first = handle.readline()
            if CWD not in first:
                continue
            meta = json.loads(first)["payload"]
            session = meta.get("id")
            role = meta.get("agent_role") or "root"
            with open(path) as handle:
                for line in handle:
                    stamp = line[14:33]
                    if not (start <= stamp < end):
                        continue
                    if '"custom_tool_call"' in line and '"exec"' in line:
                        payload = json.loads(line)["payload"]
                        call = payload.get("call_id")
                        if call in seen:
                            duplicates += 1
                            continue
                        seen.add(call)
                        pending[call] = (parse_cell(payload.get("input", "")), stamp)
                    elif '"custom_tool_call_output"' in line:
                        payload = json.loads(line)["payload"]
                        call = payload.get("call_id")
                        if call not in pending:
                            continue
                        cell, started = pending.pop(call)
                        results, raw = envelopes(payload.get("output"))
                        expected = sum(1 for item in cell if item["kind"] in ("exec", "poll"))
                        paired = expected > 0 and expected == len(results)
                        index = 0
                        for position, item in enumerate(cell):
                            record: dict[str, object] = dict(item, session=session, role=role, ts=started, cell=call, pos=position, paired=False)
                            if item["kind"] in ("exec", "poll") and paired:
                                result = results[index]
                                index += 1
                                text = str(result.get("output") or "")
                                record.update(
                                    paired=True,
                                    exit=result.get("exit_code"),
                                    running_sid=result.get("session_id"),
                                    orig_tokens=result.get("original_token_count") or 0,
                                    visible_chars=len(text),
                                    tail=text[-900:],
                                )
                            if item["kind"] == "patch":
                                record["patch_error"] = bool(PATCH_ERROR.search(raw))
                            record["cell_trunc_orig"] = sum(int(x) for x in TRUNC.findall(raw))
                            out.write(json.dumps(record) + "\n")
                            written += 1
    print(f"window {start}..{end}: records {written}, cells {len(seen)}, fork-duplicate cells skipped {duplicates}")


def strip(command: str) -> str:
    command = command.strip()
    while True:
        new = re.sub(r"^(cd \S+ ?(&&|;) ?|env -u BASH_ENV |timeout \S+ |set -\S+( pipefail)?;? ?)", "", command)
        if new == command:
            return command
        command = new


def inner(command: str) -> tuple[bool, str]:
    body = strip(command)
    match = re.match(r"^(?:env (?:\S+=\S+ )+)?scripts/pse-env(?: --?\S+)* -- (.*)$", body, re.S)
    return (True, match.group(1)) if match else (False, body)


def category(command: str) -> str:
    _, body = inner(command)
    if re.match(r"(python3?|\.venv/bin/python)\b", body):
        return "python-edit" if re.search(r"write_text|open\([^)]*['\"]w", body) else "python-other"
    if re.match(r"(cat\s*>|cat <<|tee |printf .*>)", body):
        return "shell-write"
    if re.search(
        r"\b(just (check|unit|test|turn-end|ready|hygiene|codegen|clippy|lint|typecheck|governance|native|canonical|docs|run-all)"
        r"|cargo (t|c|test|check|clippy|build|nextest|run|fmt|metadata)\b|cargo-pse-correctness|bootstrap-host|scripts/pse-host|target/debug/)",
        body,
    ):
        return "build/test/run"
    if re.match(r"(tail|ps|systemctl|journalctl|df|du|pgrep|sleep)\b", body):
        return "observe-run"
    if re.match(r"git (add|commit|cherry-pick|merge|restore|rm|mv|stash|reset|worktree)", body):
        return "git-change"
    if re.match(r"git (diff|status)", body):
        return "git-inspect"
    if re.match(r"codex exec", body):
        return "delegate"
    return "read/search"


def truncated(record: dict[str, object]) -> bool:
    tokens = int(record.get("orig_tokens") or 0)
    return bool(record.get("paired")) and tokens > 400 and int(record.get("visible_chars") or 0) / 4 < 0.8 * tokens


def report(path: str, sub_start: str) -> None:
    records = [json.loads(line) for line in open(path)]
    execs = [r for r in records if r["kind"] == "exec" and r.get("cmd")]
    polls = [r for r in records if r["kind"] == "poll"]
    patches = [r for r in records if r["kind"] == "patch"]
    print(f"records {len(records)} exec {len(execs)} polls {len(polls)} patches {len(patches)} sessions {len({r['session'] for r in records})}")
    print("roles (exec):", collections.Counter(r["role"] for r in execs).most_common())
    print(f"paired exec {sum(1 for r in execs if r['paired'])}")
    print(f"truncated cells' original tokens {sum({r['cell']: r['cell_trunc_orig'] for r in records}.values())}")

    table: dict[str, collections.Counter[str]] = collections.defaultdict(collections.Counter)
    for r in execs:
        row = table[category(r["cmd"])]
        row["n"] += 1
        row["tok"] += r.get("orig_tokens", 0)
        row["trunc"] += truncated(r)
        if r.get("paired") and r.get("exit") not in (0, None):
            row["fail"] += 1
        if r.get("paired") and r.get("exit") is None and r.get("running_sid") is not None:
            row["bg"] += 1
    for r in polls:
        row = table["(poll)"]
        row["n"] += 1
        row["tok"] += r.get("orig_tokens", 0)
    total = sum(row["tok"] for row in table.values())
    print("\ncategory           n  fail   bg trunc  tokens(k) share")
    for name, row in sorted(table.items(), key=lambda kv: -kv[1]["n"]):
        print(f"{name:15s}{row['n']:5d} {row['fail']:5d} {row['bg']:4d} {row['trunc']:5d} {row['tok'] // 1000:9d}  {row['tok'] / max(total, 1):.0%}")

    final = {(r["session"], r["sid"]): r for r in polls if r.get("paired") and r.get("exit") is not None}
    background = [r for r in execs if r.get("paired") and r.get("exit") is None and r.get("running_sid") is not None]
    logged = [r for r in background if re.search(r">\s*\S+\.(log|json)|\btee\b", r["cmd"])]
    seen = [o for o in (final.get((r["session"], r["running_sid"])) for r in background) if o]
    failed = [o for o in seen if o["exit"] != 0]
    silent = [o for o in failed if not str(o.get("tail") or "").strip()]
    logs = {m for r in execs for m in re.findall(r"(/tmp/[\w.\-]+\.(?:log|json)|build/[\w.\-/]+\.(?:log|json))", r["cmd"])}
    tail_tokens = sum(r.get("orig_tokens", 0) for r in execs if re.match(r"(cd \S+ && )?tail\b", r["cmd"].strip()))
    print(f"\nbackgrounded {len(background)}; redirected {len(logged)}; final status seen {len(seen)}; non-zero {len(failed)}; non-zero with empty output {len(silent)}")
    print(f"distinct log/json paths {len(logs)}; tail-led output tokens {tail_tokens}")

    reads: collections.Counter[tuple[str, str]] = collections.Counter()
    for r in execs:
        for m in re.finditer(r"sed -n '(\d+),(\d+)p' (\S+)", r["cmd"]):
            reads[(r["session"], m.group(3).rstrip(";"))] += 1
        for m in re.finditer(r"\bcat (\S+\.(?:rs|md|toml|py|json))", r["cmd"]):
            reads[(r["session"], m.group(1).rstrip(";"))] += 1
    orientation = collections.Counter(
        m
        for r in execs
        for m in re.findall(r"(\.agents/roles/[\w-]+\.md|\.codex/skills/[\w-]+/SKILL\.md|\.claude/rules/[\w-]+\.md|AGENTS\.md|docs/plans/README\.md)", r["cmd"])
    )
    print(f"\nre-read pairs >=5: {sum(1 for v in reads.values() if v >= 5)}; max {max(reads.values(), default=0)}")
    print(f"justfile reads {sum(1 for r in execs if re.search(r'\bjustfile\b', r['cmd']))}")
    print(f"orientation reads {sum(orientation.values())}: {orientation.most_common(8)}")
    print(f"change-set lookups {sum(1 for r in execs if re.search(r'git (status --short|diff --stat|diff --numstat)', r['cmd']))}")
    print(f"registry source lookups {sum(1 for r in execs if '.cargo/registry' in r['cmd'])}")

    sub = [r for r in execs if r["ts"] >= sub_start and category(r["cmd"]) == "build/test/run"]
    env = collections.Counter(v for r in sub for v in re.findall(r"(?:^|[\s;'\"])((?:PSE|UNO|CARGO|RUST|NEXTEST)_[A-Z0-9_]+)=", r["cmd"]))
    native = collections.Counter(m for r in execs for m in re.findall(r"--native(?:=([a-z,]*))?\b", r["cmd"]))
    print(f"\nsub-window >= {sub_start}: build/test/run {len(sub)}; inline env assignments {sum(env.values())}")
    print("inline env:", env.most_common(12))
    print("native capability sets (full window):", native.most_common())
    print(f"bare cargo test (full window) {sum(1 for r in execs if re.search(r'(^|-- |&& )cargo test\b', r['cmd']))}")
    print(f"bootstrap-host {sum(1 for r in execs if 'scripts/bootstrap-host' in r['cmd'])}")
    print(f"ps probes for build processes {sum(1 for r in execs if re.search(r'\b(ps|pgrep)\b.*(rustc|cargo|nextest|clippy)', r['cmd']))}")
    print(f"nextest replay/store uses {sum(1 for r in execs if re.search(r'nextest (replay|store)', r['cmd']))}; affected uses {sum(1 for r in execs if re.search(r'xtask affected|just affected', r['cmd']))}")
    print(f"apply_patch {len(patches)} (cells reporting a patch error {sum(1 for r in patches if r['patch_error'])}); python edits {table['python-edit']['n']}; delegations {table['delegate']['n']}")


if __name__ == "__main__":
    if sys.argv[1:2] == ["mine"]:
        mine(sys.argv[2], sys.argv[3], *(sys.argv[4:6] or ["2026-10-28T22:58", "2026-10-29T22:58"]))
    elif sys.argv[1:2] == ["report"]:
        report(sys.argv[2], sys.argv[3] if len(sys.argv) > 3 else "2026-10-29T06:32")
    else:
        sys.exit(__doc__)
