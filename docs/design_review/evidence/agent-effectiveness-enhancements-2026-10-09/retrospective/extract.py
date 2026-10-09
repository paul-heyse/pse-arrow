#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only streaming extraction. Outputs contain metadata, hashes and classifications only."""

import argparse, ast, collections, datetime, gzip, hashlib, json, os, pathlib, re, subprocess
from zoneinfo import ZoneInfo


def utc_bound(value):
    dt = datetime.datetime.fromisoformat(value.replace("Z", "+00:00"))
    if dt.tzinfo is None:
        raise argparse.ArgumentTypeError("timestamps must carry a timezone")
    return (
        dt.astimezone(datetime.timezone.utc)
        .isoformat(timespec="milliseconds")
        .replace("+00:00", "Z")
    )


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument(
    "--repo-root",
    type=pathlib.Path,
    help="Repository to attribute; defaults to Git root from cwd or this script location",
)
parser.add_argument(
    "--sessions-dir", type=pathlib.Path, default=pathlib.Path.home() / ".codex/sessions"
)
parser.add_argument(
    "--claude-projects-dir",
    type=pathlib.Path,
    default=pathlib.Path.home() / ".claude/projects",
)
parser.add_argument(
    "--output-dir", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parent
)
parser.add_argument("--start-utc", type=utc_bound, default="2026-10-06T04:00:00.000Z")
parser.add_argument("--cutoff-utc", type=utc_bound, default="2026-10-09T10:52:57.000Z")
parser.add_argument("--timezone", default="America/New_York")
parser.add_argument(
    "--exclude-root",
    action="append",
    help="Repeatable excluded root IDs; replaces the historical default exclusion",
)
args = parser.parse_args()
if args.start_utc > args.cutoff_utc:
    parser.error("--start-utc must not follow --cutoff-utc")
LOCAL_ZONE = ZoneInfo(args.timezone)
if args.repo_root is None:
    for location in (pathlib.Path.cwd(), pathlib.Path(__file__).resolve().parent):
        result = subprocess.run(
            ["git", "-C", str(location), "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            args.repo_root = pathlib.Path(result.stdout.strip())
            break
    if args.repo_root is None:
        parser.error("cannot discover a Git root; supply --repo-root")
START = args.start_utc
END = args.cutoff_utc
EXCLUDED_ROOTS = args.exclude_root or ["01a1203a-a1fc-7b23-a3f0-d7861d61e900"]
ROOT = EXCLUDED_ROOTS[0]
REPO = str(args.repo_root.resolve())
OUT = args.output_dir.resolve()
OUT.mkdir(parents=True, exist_ok=True)
D = collections.Counter()
records = []
manifest = []
sessions = {}
examples = collections.defaultdict(list)


def string(s):
    try:
        if s[0] == '"':
            return json.loads(s)
        if s[0] == "'":
            return ast.literal_eval(s)
        if s[0] == "`" and "${" not in s:
            return s[1:-1]
    except Exception:
        pass
    return None


def objects(code):
    """Find literal tools.exec_command objects; never evaluate JavaScript."""
    for m in re.finditer(r"(?:tools\.)?(exec_command|write_stdin)\s*\(\s*\{", code):
        i = m.end() - 1
        start = i
        depth = 0
        quote = None
        escape = False
        while i < len(code):
            c = code[i]
            if quote:
                if escape:
                    escape = False
                elif c == "\\":
                    escape = True
                elif c == quote:
                    quote = None
            elif c in "\"'`":
                quote = c
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    yield m.group(1), code[start : i + 1]
                    break
            i += 1


LIT = r"""("(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`)"""


def field(obj, key):
    m = re.search(r'(?:["\']?' + key + r'["\']?)\s*:\s*' + LIT, obj)
    return string(m.group(1)) if m else None


def textout(o):
    if isinstance(o, str):
        return o
    if isinstance(o, list):
        return "\n".join(x.get("text", "") for x in o if isinstance(x, dict))
    return json.dumps(o)


def structs(o):
    found = []

    def walk(v):
        if isinstance(v, dict):
            if "exit_code" in v or ("session_id" in v and "output" in v):
                found.append(v)
            else:
                for x in v.values():
                    walk(x)
        elif isinstance(v, list):
            for x in v:
                walk(x)
        elif isinstance(v, str):
            dec = json.JSONDecoder()
            i = 0
            while i < len(v):
                m = re.search(r"[\[{]", v[i:])
                if not m:
                    break
                i += m.start()
                try:
                    x, j = dec.raw_decode(v[i:])
                    walk(x)
                    i += j
                except ValueError:
                    i += 1

    walk(o)
    return found


def category(cmd):
    if re.search(r"\b(write_stdin|sleep)\b", cmd):
        return "poll-or-wait"
    if re.search(
        r"(?:^|[;&\n]|--\s)\s*(?:cargo|nextest|pytest|maturin|rustc)\b|\bjust\s+(?:check|unit|test|hygiene|ready|turn-end|parity|native)",
        cmd,
    ):
        return "build-test-check"
    if re.search(
        r"\b(tail|head|cat|sed|rg|find|ls|git\s+(show|diff|status|log))\b", cmd
    ):
        return "search-read"
    if re.search(r"\b(python|python3)\b.*(?:<<| -c )", cmd):
        return "inline-script"
    if re.search(r"\bjust\b", cmd):
        return "recipe-other"
    if re.search(r"\bgit\b", cmd):
        return "git-other"
    return "other"


def attribution(cmd, workdir, cwd):
    cd = re.search(r'(?:^|[;&\n])\s*cd\s+(?:--\s+)?["\']?(/[^\s;\n&"\']+)', cmd)
    dest = cd.group(1) if cd else (workdir or cwd)
    if dest and not dest.startswith("/"):
        dest = os.path.normpath(os.path.join(cwd or "", dest))
    if dest == REPO or (dest and dest.startswith(REPO + "/")):
        return "in", (
            "explicit-cd" if cd else "workdir" if workdir else "session-or-ancestor-cwd"
        )
    if re.search(re.escape(REPO) + r"(?:/|\b)", cmd):
        return "ambiguous", "cross-repo-reference"
    return "out", "different-cwd"


def flags(txt, cat):
    f = []
    if re.search(r"Warning: truncated output|tokens truncated|Output truncated", txt):
        f.append("truncation")
    if "Blocking waiting for file lock" in txt:
        f.append("cargo-lock-wait")
    if cat not in ["search-read", "inline-script"]:
        for tag, pat in [
            (
                "usage-error",
                r"(?im)^(?:error: unexpected argument|error: Found argument|error: no test target named|error: package ID specification|error: Recipe .+ not found|error: Justfile does not contain|error: unknown recipe|error: Test run failed: no tests to run|error: no tests to run|error: failed to parse filterset)",
            ),
            (
                "environment-error",
                r"(?im)(?:^pse-env:|command not found|No space left on device|Permission denied|failed to connect|Connection refused|out of memory|Cannot allocate memory|license.*(?:expired|invalid)|failed to acquire.*(?:scope|allocation))",
            ),
            (
                "test-failure",
                r"(?im)(?:test result: FAILED|\bFAIL \[|^FAILED |[1-9][0-9]* failed(?:,| in)|tests failed:)",
            ),
            (
                "test-pass",
                r"(?im)(?:test result: ok|Summary.*\b0 failed|[1-9][0-9]* passed(?:,| in))",
            ),
        ]:
            if re.search(pat, txt):
                f.append(tag)
    return f


# Inventory all sessions cheaply; last complete line used only to choose candidate files.
for p in sorted(args.sessions_dir.expanduser().rglob("*.jsonl")):
    try:
        st = p.stat()
        with p.open("rb") as f:
            meta = json.loads(f.readline())
            f.seek(max(0, st.st_size - 262144))
            tail = f.read().splitlines()
        last = None
        for l in reversed(tail):
            try:
                last = json.loads(l).get("timestamp")
                break
            except Exception:
                continue
        m = meta.get("payload", {})
        src = m.get("source", {})
        sub = (
            src.get("subagent", {}).get("thread_spawn", {})
            if isinstance(src, dict)
            else {}
        )
        s = dict(
            path=str(p),
            bytes=st.st_size,
            mtime_ns=st.st_mtime_ns,
            id=m.get("id"),
            parent=m.get("parent_thread_id") or sub.get("parent_thread_id"),
            cwd=m.get("cwd"),
            version=m.get("cli_version"),
            role=m.get("agent_role"),
            agent_path=m.get("agent_path") or sub.get("agent_path"),
            first=meta.get("timestamp"),
            last=last,
        )
        sessions[s["id"]] = s
    except Exception:
        D["inventory_errors"] += 1
excluded = set(EXCLUDED_ROOTS)
while True:
    nxt = excluded | {i for i, s in sessions.items() if s["parent"] in excluded}
    if nxt == excluded:
        break
    excluded = nxt
for n, s in enumerate(sessions.values()):
    if not s["last"] or s["last"] < START or s["first"] > END:
        continue
    if s["id"] in excluded:
        D["excluded_tree_files"] += 1
        continue
    root = s
    seen = set()
    while root.get("parent") in sessions and root["id"] not in seen:
        seen.add(root["id"])
        root = sessions[root["parent"]]
    cwd = s["cwd"] or root["cwd"]
    calls = {}
    outputs = collections.Counter()
    seenitems = set()
    task = "unknown"
    win_events = 0
    inscope = 0
    digest = hashlib.sha256()
    with open(s["path"], encoding="utf8") as f:
        for line_no, line in enumerate(f, 1):
            try:
                x = json.loads(line)
            except Exception:
                D["malformed_lines"] += 1
                continue
            ts = x.get("timestamp", "")
            p = x.get("payload", {})
            typ = p.get("type")
            inwin = START <= ts <= END
            if ts > END:
                continue
            if inwin:
                digest.update(line.encode())
            if x.get("type") == "turn_context":
                cwd = p.get("cwd") or cwd
            if x.get("type") != "response_item":
                if inwin:
                    win_events += 1
                continue
            if inwin:
                win_events += 1
            if typ == "message" and p.get("role") == "user" and task == "unknown":
                msg = textout(p.get("content", []))[-2000:].lower()
                if re.search(
                    r"\b(design review|review architecture|review.*design)\b", msg
                ):
                    task = "review-heuristic"
                elif re.search(
                    r"\b(create.*plan|plan creation|implementation plan)\b", msg
                ):
                    task = "planning-heuristic"
                elif re.search(r"\b(implement|fix|execute|continue)\b", msg):
                    task = "implementation-heuristic"
            if typ in ["function_call", "custom_tool_call"]:
                cid = p.get("call_id")
                if cid in calls:
                    D["duplicate_call_ids"] += int(inwin)
                    continue
                name = p.get("name", "")
                code = p.get("input", p.get("arguments", ""))
                code = code if isinstance(code, str) else json.dumps(code)
                args = {}
                if typ == "function_call":
                    try:
                        args = json.loads(code)
                    except Exception:
                        D["malformed_call_arguments"] += int(inwin)
                shell = []
                if name.split(".")[-1] == "exec_command":
                    shell = [(args.get("cmd"), args.get("workdir"), "exec_command")]
                elif name.split(".")[-1] == "exec":
                    shell = [
                        (
                            field(o, "cmd") if nm == "exec_command" else "",
                            field(o, "workdir"),
                            nm,
                        )
                        for nm, o in objects(code)
                    ]
                    if inwin:
                        for nm in re.findall(r"tools\.([a-zA-Z0-9_]+)\s*\(", code):
                            D["nested_tool:" + nm] += 1
                            if (
                                nm.startswith("mcp__")
                                and attribution("", None, cwd)[0] == "in"
                            ):
                                D["in_scope_nested_mcp:" + nm] += 1
                    if inwin:
                        D["nested_exec_command_mentions"] += len(
                            re.findall(r"\bexec_command\s*\(", code)
                        )
                        D["nested_literal_commands"] += sum(
                            c is not None for c, w, nm in shell if nm == "exec_command"
                        )
                if inwin:
                    D["tool_calls_all_workspaces"] += 1
                    for cmd, wd, nm in shell:
                        if cmd is None:
                            D["dynamic_or_unparsed_shell_calls"] += 1
                valid = []
                for cmd, wd, nm in shell:
                    if cmd is None:
                        continue
                    scope, method = attribution(cmd, wd, cwd)
                    if inwin:
                        D[
                            (
                                "shell_attribution_"
                                if nm == "exec_command"
                                else "poll_attribution_"
                            )
                            + scope
                        ] += 1
                    if scope != "in":
                        continue
                    cat = "shell-poll" if nm == "write_stdin" else category(cmd)
                    h = (
                        hashlib.sha256(cmd.encode()).hexdigest()
                        if nm == "exec_command"
                        else None
                    )
                    r = dict(
                        session=s["id"],
                        root=root["id"],
                        role="subagent" if s["parent"] else "root",
                        agent_role=s["role"],
                        version=s["version"],
                        task_type=task,
                        timestamp=ts,
                        date=datetime.datetime.fromisoformat(ts.replace("Z", "+00:00"))
                        .astimezone(LOCAL_ZONE)
                        .date()
                        .isoformat(),
                        line=line_no,
                        call_id=cid,
                        tool=nm if name.split(".")[-1] == "exec" else name,
                        wrapper_tool=name,
                        category=cat,
                        command_sha256=h,
                        attribution=method,
                        native_exit=None,
                        status="unmatched-call",
                        flags=[],
                        window_call=inwin,
                    )
                    r["features"] = [
                        tag
                        for tag, pat in [
                            ("pse-env", r"\bscripts/pse-env\b"),
                            ("native-capability", r"--native(?:[=\s]|$)"),
                            ("recipe", r"\bjust\b"),
                            ("cargo", r"\bcargo\b"),
                            (
                                "log-read",
                                r"\b(tail|cat|rg|sed)\b[^\n]*(?:\.log|/tmp/|build/|sessions|jsonl)",
                            ),
                            (
                                "selector",
                                r"\b(unit-package|nextest|pytest|cargo\s+t)\b",
                            ),
                        ]
                        if re.search(pat, cmd)
                    ]
                    valid.append(r)
                    if inwin:
                        records.append(r)
                        inscope += 1
                toolscope = attribution("", args.get("workdir"), cwd)[0]
                if not shell and inwin and toolscope == "in":
                    r = dict(
                        session=s["id"],
                        root=root["id"],
                        role="subagent" if s["parent"] else "root",
                        agent_role=s["role"],
                        version=s["version"],
                        task_type=task,
                        timestamp=ts,
                        date=datetime.datetime.fromisoformat(ts.replace("Z", "+00:00"))
                        .astimezone(LOCAL_ZONE)
                        .date()
                        .isoformat(),
                        line=line_no,
                        call_id=cid,
                        tool=name,
                        category="non-shell-tool",
                        status="unmatched-call",
                        flags=[],
                        window_call=True,
                    )
                    valid.append(r)
                    records.append(r)
                    inscope += 1
                calls[cid] = dict(
                    records=valid,
                    name=name,
                    ts=ts,
                    shellcount=len(shell),
                    call_inwin=inwin,
                )
            elif typ in ["function_call_output", "custom_tool_call_output"]:
                cid = p.get("call_id")
                outputs[cid] += 1
                if outputs[cid] > 1:
                    D["repeated_output_call_ids_first_retained"] += int(inwin)
                    continue
                c = calls.get(cid)
                if c is None:
                    if inwin:
                        D["unmatched_outputs_all_workspaces"] += 1
                    continue
                if not c["records"]:
                    continue
                txt = textout(p.get("output"))
                ss = structs(p.get("output"))
                statuses = []
                toolstat = (
                    "yielded"
                    if re.search(
                        r"Script running with cell ID|Process running with session ID",
                        txt,
                    )
                    else "completed"
                )
                if re.search(r"(?m)^Script failed|^Error executing", txt):
                    toolstat = "tool-error"
                if not c["call_inwin"] and inwin:
                    D["inwindow_outputs_for_prewindow_calls"] += len(c["records"])
                    for r in c["records"]:
                        r["window_call"] = False
                        records.append(r)
                for i, r in enumerate(c["records"]):
                    r["status"] = toolstat
                    r["output_line"] = line_no
                    r["output_timestamp"] = ts
                    native = None
                    own = txt
                    if len(ss) == len(c["records"]) and c["shellcount"] == len(
                        c["records"]
                    ):
                        native = ss[i].get("exit_code")
                        own = ss[i].get("output", "")
                        r["native_session_id"] = ss[i].get("session_id")
                        r["native_running"] = (
                            ss[i].get("session_id") is not None and native is None
                        )
                    elif len(c["records"]) == 1:
                        m = re.search(
                            r'(?:Process exited with code|exit_code["\s:]+)\s*(-?\d+)',
                            txt,
                        )
                        if m:
                            native = int(m.group(1))
                    elif len(c["records"]) > 1:
                        D["ambiguous_nested_result_records"] += int(inwin)
                        own = ""
                        r["result_mapping"] = "ambiguous"
                        r["wrapper_observation_flags"] = flags(txt, "search-read")
                    r["native_exit"] = native
                    r["flags"] = flags(own, r["category"])
                    r["output_chars"] = len(txt)
                    if r["category"] == "search-read":
                        r["flags"] = [z for z in r["flags"] if z != "cargo-lock-wait"]
                    if r.get("native_running"):
                        r["status"] = "native-running"
                    if r["category"] == "non-shell-tool":
                        r["flags"] = flags(txt, "search-read")
                    if (
                        "truncation" in flags(txt, "search-read")
                        and "truncation" not in r["flags"]
                    ):
                        r["flags"].append("wrapper-truncation")
    manifest.append(
        {
            **s,
            "root": root["id"],
            "events_in_window": win_events,
            "attributed_records": inscope,
            "task_type": task,
            "window_event_sha256": digest.hexdigest(),
        }
    )
    if n % 100 == 0:
        print("processed", n, "records", len(records), flush=True)
# Immutable aggregate counts, and source pointers without command/output content.
stats = collections.Counter()
strata = collections.defaultdict(collections.Counter)
rh = collections.Counter()
for r in records:
    stats["records"] += 1
    stats["category:" + r["category"]] += 1
    stats["status:" + r["status"]] += 1
    stats["tool:" + r["tool"]] += 1
    if r.get("native_exit") is not None:
        stats["native_exit:" + str(r["native_exit"])] += 1
    for t in r["flags"]:
        stats["flag:" + t] += 1
    for t in r.get("features", []):
        stats["feature:" + t] += 1
    for dim in ["date", "version", "role", "task_type", "root"]:
        k = dim + ":" + str(r.get(dim))
        strata[k]["records"] += 1
        strata[k]["category:" + r["category"]] += 1
        strata[k]["status:" + r["status"]] += 1
        for t in r["flags"]:
            strata[k]["flag:" + t] += 1
    if r.get("command_sha256"):
        rh[(r["session"], r["command_sha256"])] += 1
stats["repeated_exact_command_occurrences_after_first"] = sum(
    v - 1 for v in rh.values() if v > 1
)
stats["distinct_repeated_exact_commands"] = sum(v > 1 for v in rh.values())
claude = []
for p in args.claude_projects_dir.expanduser().glob("*" + REPO.replace("/", "-") + "*"):
    fs = list(p.rglob("*.jsonl"))
    claude.append(
        dict(
            path=str(p),
            files=len(fs),
            bytes=sum(f.stat().st_size for f in fs),
            modified_since_start=sum(
                f.stat().st_mtime
                >= datetime.datetime.fromisoformat(
                    START.replace("Z", "+00:00")
                ).timestamp()
                for f in fs
            ),
        )
    )
(OUT / "source-manifest.json").write_text(
    json.dumps(
        dict(
            start_utc=START,
            cutoff_utc=END,
            timezone=args.timezone,
            excluded_root=ROOT,
            excluded_roots=EXCLUDED_ROOTS,
            excluded_tree_ids=sorted(excluded),
            source_files=manifest,
            claude_inventory_only=claude,
        ),
        indent=2,
    )
)
(OUT / "metrics.json").write_text(
    json.dumps(
        dict(
            method="literal-JavaScript nested extraction; event-time filtering; metadata plus per-command attribution",
            data_quality=dict(D),
            totals=dict(stats),
            strata={k: dict(v) for k, v in strata.items()},
            distinct_sessions=len({r["session"] for r in records}),
            distinct_roots=len({r["root"] for r in records}),
        ),
        indent=2,
    )
)
with gzip.open(OUT / "event-pointers.jsonl.gz", "wt", encoding="utf8") as f:
    for r in records:
        f.write(json.dumps(r) + "\n")
print(
    "DONE",
    len(records),
    "sessions",
    len({r["session"] for r in records}),
    "roots",
    len({r["root"] for r in records}),
    flush=True,
)
