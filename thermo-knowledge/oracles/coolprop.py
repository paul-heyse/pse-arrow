# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Oracle harness for CoolProp (docs/pipeline.md section 5.2).

Reads a request file and writes a result file; it imports nothing of the tree and reads no
canonical record: CoolProp answers from the fluid data bundled in its own wheel.

Call `saturation_pressure_ancillary`: the saturation-pressure ancillary equation of a fluid
evaluated at temperatures in kelvin, answering in pascal. It is `AbstractState.saturation_ancillary(iP, 1, iT, T)`
of the Helmholtz backend: the direct evaluation of the fluid's `pS` ancillary curve
(`SaturationAncillaryFunction::evaluate`), not the equation of state's saturation solve and not
the superancillary. Above the ancillary's reducing temperature CoolProp returns NaN.

    python oracles/coolprop.py --request request.json --result result.json
"""

from __future__ import annotations

import argparse
import json
import math
import sys

PROTOCOL = 1
CALLS = {"saturation_pressure_ancillary": ({"T": "K"}, "Pa")}


def saturation_pressure_ancillary(
    fluids: dict[str, object], key: str, temperatures: list[float]
) -> list[dict[str, object]]:
    import CoolProp

    try:
        state = fluids.get(key)
        if state is None:
            state = fluids[key] = CoolProp.AbstractState("HEOS", key)
    except Exception as error:  # CoolProp raises ValueError for an unknown fluid
        return [
            {"value": None, "status": "error", "message": f"{key}: {error}"} for _ in temperatures
        ]
    points: list[dict[str, object]] = []
    for temperature in temperatures:
        try:
            value = state.saturation_ancillary(CoolProp.iP, 1, CoolProp.iT, temperature)
        except Exception as error:
            points.append({"value": None, "status": "error", "message": str(error)})
            continue
        if math.isnan(value):
            points.append(
                {
                    "value": None,
                    "status": "nan",
                    "message": f"T = {temperature!r} K is above the reducing temperature",
                }
            )
        elif math.isinf(value):
            points.append(
                {"value": None, "status": "error", "message": f"infinite at T = {temperature!r} K"}
            )
        else:
            points.append({"value": value, "status": "ok", "message": None})
    return points


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--request", required=True)
    parser.add_argument("--result", required=True)
    options = parser.parse_args()
    with open(options.request, encoding="utf-8") as handle:
        request = json.load(handle)
    if request.get("protocol") != PROTOCOL:
        print(f"coolprop: protocol {request.get('protocol')!r} is not {PROTOCOL}", file=sys.stderr)
        return 2
    call = request.get("call")
    if call not in CALLS:
        print(f"coolprop: unknown call {call!r} (one of {', '.join(CALLS)})", file=sys.stderr)
        return 2
    arguments, unit = CALLS[call]
    if request.get("arguments") != arguments or request.get("output", {}).get("unit") != unit:
        print(f"coolprop: call {call!r} takes {arguments} and answers in {unit}", file=sys.stderr)
        return 2
    import CoolProp

    states: dict[str, object] = {}
    subjects = []
    for subject in request["subjects"]:
        (key,) = subject["key"]
        points = saturation_pressure_ancillary(
            states, key, [float(t) for t in subject["arguments"]["T"]]
        )
        subjects.append({"key": subject["key"], "points": points})
    result = {
        "protocol": PROTOCOL,
        "library": "CoolProp",
        "version": CoolProp.__version__,
        "output_unit": unit,
        "subjects": subjects,
    }
    with open(options.result, "w", encoding="utf-8") as handle:
        json.dump(result, handle, allow_nan=False)
    return 0


if __name__ == "__main__":
    sys.exit(main())
