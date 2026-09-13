#!/usr/bin/env bash
# Generate rustdoc JSON for every crate in the pinned arrow + datafusion families.
# Fails soft per crate; the log is the receipt.
set -u
cd "$(dirname "$0")"
TC="${DFARROW_TOOLCHAIN:-nightly}"
export RUSTDOCFLAGS="-Z unstable-options --output-format json"
LOG=rustdoc_gen.log
: > "$LOG"
{ echo "toolchain: $(rustc +$TC --version)"; echo "cargo:     $(cargo +$TC --version)"; echo "started:   $(date -Is)"; } >> "$LOG"

TARGETS=$(awk '/^\[dependencies\]/{f=1;next} /^\[/{f=0} f && /^[a-z]/{print $1}' Cargo.toml)
TARGETS="$TARGETS datafusion-proto-models"
ok=0; fail=0
for t in $TARGETS; do
  if cargo +$TC rustdoc --offline -p "$t" >>"$LOG" 2>&1; then
    echo "OK   $t" >> "$LOG"; ok=$((ok+1))
  else
    echo "FAIL $t" >> "$LOG"; fail=$((fail+1))
  fi
done
{ echo "finished:  $(date -Is)"; echo "ok=$ok fail=$fail"; } >> "$LOG"
ls -la target/doc/*.json >> "$LOG" 2>&1
