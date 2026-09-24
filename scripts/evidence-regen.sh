#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Regenerate the capability-map evidence from the committed, pinned manifests:
#   1. rustdoc-JSON extraction for the arrow/datafusion and supporting-crate scratch
#      projects (the committed lockfiles make it resolve to the same versions);
#   2. the probe programs, whose captured output the maps quote;
#   3. the Python API dumps and probes in the platform and parity environments;
#   4. the exported requirement files (derived from uv.lock, never hand-maintained).
# Runs the documented manual steps from docs/capability-maps/evidence/*/README.md.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
EV="docs/capability-maps/evidence"
NIGHTLY="$(grep -oE 'nightly-[0-9-]+' tooling/dfarrow-apiex/rust-toolchain.toml | head -n1)"
WORK="${PSE_EVIDENCE_WORK:-$ROOT/build/evidence}"
mkdir -p "$WORK"

extract() { # profile manifest lock genscript
  local dir="$WORK/$1"
  mkdir -p "$dir/src/bin"
  cp "$EV/rust/$2" "$dir/Cargo.toml"
  cp "$EV/rust/$3" "$dir/Cargo.lock"
  cp "$EV/rust/$4" "$dir/gen.sh"; chmod +x "$dir/gen.sh"
  echo '// extraction root' > "$dir/src/lib.rs"
  (cd "$dir" && cargo "+$NIGHTLY" fetch --locked && ./gen.sh)
}

echo "==> rustdoc extraction (toolchain $NIGHTLY)"
extract apisurface apisurface-Cargo.toml apisurface-Cargo.lock gen_rustdoc.sh
extract support    support-Cargo.toml    support-Cargo.lock    gen_rustdoc_support.sh

echo "==> probes"
cp "$EV"/rust/df_probe*.rs "$EV"/rust/arrow_probe2.rs "$EV"/arrow_probe.rs "$WORK/apisurface/src/bin/"
cp "$EV"/rust/probe_*.rs "$WORK/support/src/bin/"
(cd "$WORK/apisurface" && for b in df_probe df_probe_b df_probe_c3 df_probe_x arrow_probe2 arrow_probe; do
   cargo "+$NIGHTLY" run --quiet --offline --locked --bin "$b"; done) > "$EV/rust/probe_output.txt"
(cd "$WORK/support" && for b in probe_identity probe_graph probe_numerics probe_compile probe_parse; do
   cargo "+$NIGHTLY" run --quiet --offline --locked --bin "$b"; done) > "$EV/rust/probe_output_support.txt"

echo "==> python api dumps and probes"
uv sync --locked --group evidence
uv run --no-sync python "$EV/python/apidump.py" "$WORK/api-platform" pyarrow attrs cattrs msgspec numpy scipy
for p in probe_arrow probe_contracts; do uv run --no-sync python "$EV/python/$p.py"; done
UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --group parity --python 3.13
UV_PROJECT_ENVIRONMENT=.venv-parity uv run --no-sync python "$EV/python/apidump.py" "$WORK/api-parity" idaes

echo "==> exported requirement files (derived from uv.lock)"
uv export --frozen --no-hashes --no-dev --group evidence -o "$EV/python/requirements-platform.txt"
uv export --frozen --no-hashes --no-dev --group parity --python-version 3.13 -o "$EV/python/requirements-parity.txt"
echo "done; review the diff under $EV and update the maps' [probe] quotes if measurements changed"
