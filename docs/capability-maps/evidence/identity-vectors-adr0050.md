<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Independent confirmation of ADR-0050 identity vectors

**Tested**, 2026-09-14: 11 frozen hash/identity vectors passed, 0 failed, baseline 0.
The separately compiled official portable C BLAKE3 implementation produced every
hexadecimal value recorded in ADR-0050. No `pse-ids` helper, Rust BLAKE3 invocation,
Python BLAKE3 package, downloaded code or added dependency was used as the oracle.

[The receipt](identity-vectors-adr0050.json) contains every exact input preimage,
derive-key context, requested XOF length, expected/actual output, compiler version,
and SHA-256 of all six C implementation/header inputs. The source is the official C
implementation shipped in the already resolved `blake3-1.8.7` Cargo source archive;
it was compiled directly with GCC 13.3.0. `b3sum` was absent. The C test driver
exercises each CPU-dispatch selection; all SIMD paths were disabled, and every
selection returned the same portable-C result. This independently invokes a
different language implementation, while sharing the upstream BLAKE3 project.

The preimages were constructed independently from ADR-0050's literal inputs. A
framed part is `len(value).to_bytes(8, "little") + value`. `Ordinal(7)` is eight
little-endian bytes, schema versions are four, and `(C, D)` is one 32-byte framed
part. The second named-ID vector uses the frozen registry ID as literal input.
The case preimage uses the written role/member ordering and fixed raw IDs/hashes,
without invoking a repository framing routine. The receipt makes those exact
bytes inspectable and replayable.

Run from the repository root with the existing pinned Cargo source cache:

```bash
PSE_BLAKE3_C_FILE=$(rg --files "${CARGO_HOME:-$HOME/.cargo}/registry/src" -g blake3.c | rg '/blake3-1.8.7/c/blake3.c$')
PSE_BLAKE3_C=$(dirname "$PSE_BLAKE3_C_FILE")
export PSE_BLAKE3_C
mkdir -p /tmp/pse-identity-c-oracle
gcc -O2 -DBLAKE3_TESTING -DBLAKE3_NO_SSE2 -DBLAKE3_NO_SSE41 \
  -DBLAKE3_NO_AVX2 -DBLAKE3_NO_AVX512 -DBLAKE3_USE_NEON=0 \
  "$PSE_BLAKE3_C/main.c" "$PSE_BLAKE3_C/blake3.c" \
  "$PSE_BLAKE3_C/blake3_dispatch.c" "$PSE_BLAKE3_C/blake3_portable.c" \
  -o /tmp/pse-identity-c-oracle/blake3-c
gcc --version
```

Replay each exact preimage and verify source provenance using the pinned Python
environment (only standard-library modules are required):

```bash
.venv/bin/python - <<'PY'
import hashlib
import json
import os
from pathlib import Path
import subprocess

receipt = json.loads(Path("docs/capability-maps/evidence/identity-vectors-adr0050.json").read_text())
source = Path(os.environ["PSE_BLAKE3_C"])
for name, expected in receipt["source_files_sha256"].items():
    assert hashlib.sha256((source / name).read_bytes()).hexdigest() == expected, name
for vector in receipt["vectors"]:
    command = ["/tmp/pse-identity-c-oracle/blake3-c", "--length", str(vector["length"])]
    if vector["context"] is not None:
        command += ["--derive-key", vector["context"]]
    result = subprocess.run(command, input=bytes.fromhex(vector["preimage_hex"]), capture_output=True, check=True)
    outputs = result.stdout.decode().splitlines()
    assert outputs and set(outputs) == {vector["expected"]}, vector["label"]
    print(vector["label"], outputs[0])
print(f"{len(receipt['vectors'])} passed; 0 failed; baseline 0")
PY
```

The [official C documentation](https://github.com/BLAKE3-team/BLAKE3/blob/f3149ec5bb5449af877ba20377a11008ff499fa2/c/README.md)
documents portable compilation, derive-key initialization and arbitrary-length
finalization. The actual source/version checks above are the reproducibility
authority for this receipt.

This evidence confirms framing and frozen cryptographic output only. It does not
establish semantic validity of a relation, snapshot, source projection or reused
computation. Those are separately validated from actual declarations and values.
The float-bit rows in ADR-0050 are representation rules rather than BLAKE3 vectors;
their executable controls remain in `pse-ids/tests/golden_vectors.rs`.
