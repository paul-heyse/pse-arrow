# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual pinned cargo-hack capture, interrupt/resume and Rust-repair acceptance.

Run explicitly through pse-env's build class; this is a small mechanism matrix,
not the repository-wide feature campaign.
"""

from __future__ import annotations

import argparse
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

from scripts import validation_feature_campaign as campaign
from scripts import validation_receipts


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    root = args.directory.resolve()
    require(not root.exists(), "fixture directory must be new")
    (root / "crates/fixture/src").mkdir(parents=True)
    (root / "vendor/optional/src").mkdir(parents=True)
    (root / ".cargo").mkdir()
    (root / "Cargo.toml").write_text(
        '[workspace]\nmembers=["crates/fixture"]\nexclude=["vendor/optional"]\nresolver="3"\n'
    )
    (root / "crates/fixture/Cargo.toml").write_text(
        '[package]\nname="feature-fixture"\nversion="0.1.0"\nedition="2024"\n[dependencies]\noptional-helper={path="../../vendor/optional",optional=true}\n[features]\ndefault=["a"]\na=[]\nb=[]\nc=[]\n'
    )
    (root / "vendor/optional/Cargo.toml").write_text(
        '[package]\nname="optional-helper"\nversion="0.1.0"\nedition="2024"\n'
    )
    (root / "vendor/optional/src/lib.rs").write_text("pub fn helper()->u32{2}\n")
    source = root / "crates/fixture/src/lib.rs"
    source.write_text(
        'pub fn value()->u32{1}\n#[cfg(feature="optional-helper")] pub fn optional()->u32{optional_helper::helper()}\n'
    )
    (root / "rust-toolchain.toml").write_bytes(
        (campaign.ROOT / "rust-toolchain.toml").read_bytes()
    )
    (root / ".cargo/config.toml").write_text("[build]\njobs=16\n")
    subprocess.run(["git", "init", "-q", str(root)], check=True)
    subprocess.run(["cargo", "generate-lockfile"], cwd=root, check=True)
    env = {**os.environ, "PYTHONPATH": str(campaign.ROOT)}
    program = 'from pathlib import Path; import sys; from scripts.validation_feature_campaign import run; raise SystemExit(run(Path(sys.argv[1]),Path(sys.argv[2]),"combinations",resume_from=Path(sys.argv[3]) if len(sys.argv)>3 else None))'
    first = root / "first"
    second = root / "resumed"
    third = root / "changed"
    with (root / "first-driver.log").open("wb") as log:
        process = subprocess.Popen(
            [sys.executable, "-c", program, str(root), str(first)],
            cwd=campaign.ROOT,
            env=env,
            stdout=log,
            stderr=subprocess.STDOUT,
        )
        deadline = time.monotonic() + 180
        sent = False
        while process.poll() is None:
            path = first / "feature-campaign.json"
            if path.is_file():
                report = json.loads(path.read_text())
                if (
                    len(report["combinations"]) > 2
                    and report["combinations"][2]["status"] == "running"
                ):
                    process.send_signal(signal.SIGINT)
                    sent = True
                    break
            require(
                time.monotonic() < deadline, "fixture did not reach third combination"
            )
            time.sleep(0.02)
        code = process.wait(timeout=180)
    require(sent and code == 130, f"interrupted campaign returned {code}")
    prior = validation_receipts.feature_campaign(first)
    require(
        [row["status"] for row in prior["combinations"][:3]]
        == ["passed", "passed", "interrupted"],
        "interrupt did not preserve two completed passes",
    )
    # Compare the actual safe-fixture display as text, never decode/evaluate it.
    # This independently exercises cargo-hack's print path against the adapter.
    printed = subprocess.check_output(
        [
            "cargo",
            "hack",
            "--keep-going",
            "check",
            "--workspace",
            "--feature-powerset",
            "--depth",
            "2",
            "--locked",
            "--print-command-list",
        ],
        cwd=root,
        env=env,
        text=True,
    )
    captured = prior["frozen"]["ordered_literal_argv"]
    display = ["cargo " + " ".join(argv) for argv in captured]
    actual = [line.strip().strip("`") for line in printed.splitlines() if line.strip()]
    require(
        actual == display,
        f"print/capture fixture ordering differs: {actual!r} != {display!r}",
    )
    require(
        any("--all-features" in argv for argv in captured),
        "all-feature special selection missing",
    )
    require(
        any(
            "--features" not in argv and "--no-default-features" in argv
            for argv in captured
        ),
        "no-default special selection missing",
    )
    require(
        any(argv[-2:] == ["--features", "default"] for argv in captured),
        "default selection missing",
    )
    for output, parent in ((second, first), (third, second)):
        if output == third:
            source.write_text(
                source.read_text().replace("value()->u32{1}", "value()->u32{3}")
            )
        with (root / (output.name + "-driver.log")).open("wb") as log:
            result = subprocess.run(
                [sys.executable, "-c", program, str(root), str(output), str(parent)],
                cwd=campaign.ROOT,
                env=env,
                stdout=log,
                stderr=subprocess.STDOUT,
                timeout=240,
                check=False,
            )
        require(
            result.returncode == 0, f"{output.name} campaign failed; see its driver log"
        )
    resumed = validation_receipts.feature_campaign(second)
    changed = validation_receipts.feature_campaign(third)
    require(
        sum(
            row.get("evidence_kind") == "unchanged-input-reuse"
            for row in resumed["combinations"]
        )
        == 2,
        "resume reran completed combinations",
    )
    require(
        all(row.get("evidence_kind") == "executed" for row in changed["combinations"]),
        "source repair did not invalidate every combination",
    )
    # The current independent no-default recipe remains separately enumerated.
    output = root / "no-default"
    output.mkdir()
    nodefault, _ = campaign.enumerate_commands(
        root, output, campaign.SELECTIONS["no-default"], env
    )
    require(
        len(nodefault) == 1 and "--no-default-features" in nodefault[0],
        "separate no-default selection changed",
    )
    print(
        json.dumps(
            {
                "fixture": str(root),
                "combinations": len(captured),
                "interrupted_completed": 2,
                "resumed_reused": 2,
                "after_source_change_executed": len(changed["combinations"]),
                "required_checks_covered": changed["required_checks_covered"],
                "no_default_commands": len(nodefault),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
