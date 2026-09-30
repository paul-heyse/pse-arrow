# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A fake oracle harness for the qualification tests. It follows the harness protocol, holds its
own coefficients (as a library holds its bundled data) and imports nothing of the tree.

The call picks the behaviour: `ok` answers every point; `mixed` answers NaN at the first point of
each subject and an error at the second; `crash` exits non-zero; `wrong_unit` answers in other
units; `short` answers one point too few. Each invocation appends one line (the number of
subjects) to the file named by FAKE_HARNESS_LOG, when set.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import sys

COEFFICIENTS = {
    "sp-a": {"T_r": 400.0, "p_r": 5.0e6, "n": [-7.5, 1.6, -3.2], "t": [1.0, 1.5, 3.0]},
    "sp-b": {"T_r": 520.0, "p_r": 3.2e6, "n": [-8.1, 2.4, -2.7], "t": [1.0, 1.8, 3.5]},
    "sp-c": {"T_r": 310.0, "p_r": 4.1e6, "n": [-6.9, 1.2, -4.4], "t": [1.0, 1.4, 2.6]},
}


def pressure(key: str, temperature: float) -> float:
    a = COEFFICIENTS[key]
    theta = 1.0 - temperature / a["T_r"]
    if theta < 0:
        return math.nan
    total = sum(n * theta**t for n, t in zip(a["n"], a["t"]))
    return a["p_r"] * math.exp(a["T_r"] / temperature * total)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--request", required=True)
    parser.add_argument("--result", required=True)
    options = parser.parse_args()
    request = json.load(open(options.request, encoding="utf-8"))
    log = os.environ.get("FAKE_HARNESS_LOG")
    if log:
        with open(log, "a", encoding="utf-8") as handle:
            handle.write(f"{len(request['subjects'])}\n")
    call = request["call"]
    if call == "crash":
        print("fake: the library cannot start", file=sys.stderr)
        return 3
    unit = "bar" if call == "wrong_unit" else "Pa"
    subjects = []
    for subject in request["subjects"]:
        (key,) = subject["key"]
        points = []
        for position, temperature in enumerate(subject["arguments"]["T"]):
            if call == "mixed" and position == 0:
                points.append(
                    {"value": None, "status": "nan", "message": "above the reducing temperature"}
                )
            elif call == "mixed" and position == 1:
                points.append({"value": None, "status": "error", "message": "no such point"})
            else:
                value = pressure(key, temperature)
                if math.isnan(value):
                    points.append(
                        {
                            "value": None,
                            "status": "nan",
                            "message": "above the reducing temperature",
                        }
                    )
                else:
                    points.append({"value": value, "status": "ok", "message": None})
        if call == "short":
            points = points[:-1]
        subjects.append({"key": subject["key"], "points": points})
    result = {
        "protocol": 1,
        "library": "FakeLib",
        "version": "1.2.3",
        "output_unit": unit,
        "subjects": subjects,
    }
    json.dump(result, open(options.result, "w", encoding="utf-8"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
