#!/usr/bin/env bash
# Generate rustdoc JSON for the supporting-library family. Fails soft per crate; the log is the receipt.
set -u
cd "$(dirname "$0")" || exit 1
TC="${DFARROW_TOOLCHAIN:-nightly}"
export RUSTDOCFLAGS="-Z unstable-options --output-format json"
LOG=rustdoc_gen.log
: > "$LOG"
{ echo "toolchain: $(rustc +"$TC" --version)"; echo "cargo:     $(cargo +"$TC" --version)"; echo "started:   $(date -Is)"; } >> "$LOG"
TARGETS="syn quote proc-macro2 prettyplease serde serde_arrow serde_yaml serde-saphyr toml winnow \
petgraph salsa egglog num-dual faer uom feos-core quantity diffsol blake3 rayon tokio tracing thiserror miette"
ok=0; fail=0
for t in $TARGETS; do
  if cargo +"$TC" rustdoc --offline -p "$t" >>"$LOG" 2>&1; then
    echo "OK   $t" >> "$LOG"; ok=$((ok+1))
  else
    echo "FAIL $t" >> "$LOG"; fail=$((fail+1))
  fi
done
{ echo "finished:  $(date -Is)"; echo "ok=$ok fail=$fail"; } >> "$LOG"
ls -la target/doc/*.json >> "$LOG" 2>&1
