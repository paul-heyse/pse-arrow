"""Bounded passive-reader control; does not alter the supplied checkpoint."""

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path


def counts(operation):
    return {
        "detail": len(operation.get("detail", [])),
        "recent": sum(
            len(producer.get("recent", []))
            for producer in operation.get("producers", [])
        ),
        "producers": len(operation.get("producers", [])),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checkpoint", type=Path)
    args = parser.parse_args()
    raw = args.checkpoint.read_bytes()
    if len(raw) > 1 << 20:
        raise ValueError("Probe only accepts checkpoints up to 1 MiB")
    image = json.loads(raw)
    operation = max(image["operations"], key=lambda item: counts(item)["recent"])
    identity = operation["id"]
    locator = (
        "".join(f"{word:016x}" for word in identity["process_instance"])
        + ":"
        + str(identity["sequence"])
    )
    report = {
        "source": str(args.checkpoint),
        "source_bytes": len(raw),
        "operation": locator,
        "raw_counts": counts(operation),
        "control_description": "Scratch-only clock=null exercises existing unavailable-clock fallback",
    }
    with tempfile.TemporaryDirectory(prefix="pse-frozen-reader-control-") as directory:
        control = Path(directory) / "clock-absent.json"
        image["clock"] = None
        control.write_text(json.dumps(image))
        for name, checkpoint in [("original", args.checkpoint), ("control", control)]:
            completed = subprocess.run(
                [
                    sys.executable,
                    "-c",
                    "from pse.diagnostics import main; raise SystemExit(main())",
                    "inspect",
                    str(checkpoint),
                    locator,
                    "--window",
                    "60",
                    "--no-related",
                    "--json",
                ],
                capture_output=True,
                text=True,
                timeout=20,
                check=False,
            )
            result = {"exit_code": completed.returncode, "stderr": completed.stderr}
            if completed.returncode == 0:
                document = json.loads(completed.stdout)
                selected = document["operation"]
                result.update(
                    counts=counts(selected),
                    read_at=selected["read_at"],
                    interval_start=selected["interval_start"],
                    interval_end=selected["interval_end"],
                    coverage=selected["coverage"],
                    transport=document["transport"],
                )
            report[name] = result
    report["source_unchanged"] = args.checkpoint.read_bytes() == raw
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
