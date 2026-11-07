#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Exercise the supplied managed executable on isolated cache and endpoint pairs.
set -euo pipefail
binary=$(realpath "${1:?pass the prepared managed sccache executable}")
checkout=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
scratch=$(mktemp -d /tmp/pse-cache-probe.XXXXXXXX)
export SCCACHE_CONF="$checkout/.config/sccache.toml"
export SCCACHE_CLIENT_SIDE=1 SCCACHE_DIRECT=false SCCACHE_MAX_FRAME_LENGTH=268435456
export SCCACHE_CACHE_SIZE=512M
unset SCCACHE_LOG SCCACHE_ERROR_LOG SCCACHE_NO_CACHE SCCACHE_RECACHE
export SCCACHE_DIR="$scratch/disk" SCCACHE_SERVER_UDS="$scratch/primary.sock"
abstract="\x00pse-plan67-cache-probe-$UID"
finish() {
  SCCACHE_DIR="$scratch/disk" SCCACHE_SERVER_UDS="$scratch/primary.sock" "$binary" --stop-server >/dev/null 2>&1 || true
  SCCACHE_DIR="$scratch/abstract" SCCACHE_SERVER_UDS="$abstract" "$binary" --stop-server >/dev/null 2>&1 || true
  rm -rf -- "$scratch"
}
trap finish EXIT
refuse() {
  local expected=$1
  shift
  if "$@" >"$scratch/refusal" 2>&1; then
    echo "unexpected success for $expected" >&2
    return 1
  fi
  if ! grep -Fq -- "$expected" "$scratch/refusal"; then
    cat "$scratch/refusal" >&2
    return 1
  fi
}
listeners() {
  awk -v endpoint="$1" '$4=="00010000" && $8==endpoint {count++} END {print count+0}' /proc/net/unix
}

"$binary" --show-stats --stats-format=json | jq -e '.effective_cache_mode == null' >/dev/null
test ! -e "$SCCACHE_DIR"
test ! -e "$scratch/.pse-sccache"

# All six local codegen processes must reach the barrier while the other five
# still hold their work. Preprocessing and compiler detection remain ordinary.
mkdir "$scratch/bin" "$scratch/ready"
export PSE_CACHE_PROBE_GCC
PSE_CACHE_PROBE_GCC=$(command -v gcc)
export PSE_CACHE_PROBE_READY="$scratch/ready"
cat >"$scratch/bin/gcc" <<'SH'
#!/bin/sh
case " $* " in
  *' -E '*) ;;
  *' -x c -c '*)
    touch "$PSE_CACHE_PROBE_READY/$$"
    elapsed=0
    while [ "$(find "$PSE_CACHE_PROBE_READY" -type f | wc -l)" -lt 6 ]; do
      [ "$elapsed" -lt 100 ] || exit 125
      sleep 0.1
      elapsed=$((elapsed + 1))
    done
    ;;
esac
exec "$PSE_CACHE_PROBE_GCC" "$@"
SH
chmod 700 "$scratch/bin/gcc"
cd "$scratch"
jobs=()
for i in {1..6}; do
  printf 'int probe%d(void) { return %d; }\n' "$i" "$i" >"test$i.c"
  "$binary" "$scratch/bin/gcc" -c "test$i.c" -o "test$i.o" &
  jobs+=("$!")
done
for job in "${jobs[@]}"; do
  wait "$job"
done
test "$(listeners "$SCCACHE_SERVER_UDS")" -eq 1
test "$(find "$scratch/ready" -type f | wc -l)" -eq 6

cp test1.o expected.o
rm test1.o
"$binary" "$scratch/bin/gcc" -c test1.c -o test1.o
cmp test1.o expected.o
test "$(find "$scratch/ready" -type f | wc -l)" -eq 6

rustc=$(rustup which rustc --toolchain "$(cd "$checkout" && rustup show active-toolchain | cut -d' ' -f1)")
printf 'include!("member.rs");\n' >lib.rs
printf 'pub const N: i32 = 1;\n' >member.rs
"$binary" "$rustc" --crate-name cache_probe --crate-type lib --emit dep-info,metadata --out-dir . lib.rs
cp libcache_probe.rmeta expected.rmeta
rm libcache_probe.rmeta cache_probe.d
"$binary" "$rustc" --crate-name cache_probe --crate-type lib --emit dep-info,metadata --out-dir . lib.rs
cmp libcache_probe.rmeta expected.rmeta
grep -Fq member.rs cache_probe.d
printf 'pub const N: i32 = 2;\n' >member.rs
rm libcache_probe.rmeta cache_probe.d
"$binary" "$rustc" --crate-name cache_probe --crate-type lib --emit dep-info,metadata --out-dir . lib.rs
if cmp -s libcache_probe.rmeta expected.rmeta; then
  echo "changed included source restored stale Rust metadata" >&2
  exit 1
fi
"$binary" --show-stats --stats-format=json | jq -e '
  .effective_cache_mode == "ReadWrite"
  and (.stats.cache_hits.counts | to_entries | map(.value) | add) >= 2
  and .stats.cache_misses.counts.Rust == 2
  and .stats.cache_write_errors == 0
' >/dev/null

ln -s "$scratch" "$scratch/endpoint-alias"
ln -s "$scratch/disk" "$scratch/disk-alias"
SCCACHE_DIR="$scratch/disk-alias" SCCACHE_SERVER_UDS="$scratch/endpoint-alias/primary.sock" \
  "$binary" --show-stats --stats-format=json | jq -e '.effective_cache_mode == "ReadWrite"' >/dev/null
mkdir -m 700 "$scratch/conflicting-disk"
SCCACHE_DIR="$scratch/conflicting-disk" refuse "different cache directory or configuration" "$binary" --zero-stats
SCCACHE_SERVER_UDS="$scratch/other.sock" refuse "Address in use" "$binary" --start-server
SCCACHE_MAX_FRAME_LENGTH=16777216 refuse "different cache directory or configuration" "$binary" --show-stats
test "$(listeners "$SCCACHE_SERVER_UDS")" -eq 1
test ! -e "$scratch/other.sock"

"$binary" --stop-server >/dev/null
for _ in {1..50}; do
  if test "$(listeners "$SCCACHE_SERVER_UDS")" -eq 0 && flock -n "$SCCACHE_DIR" true; then
    break
  fi
  sleep 0.1
done
test "$(listeners "$SCCACHE_SERVER_UDS")" -eq 0
flock -n "$SCCACHE_DIR" true
test -S "$SCCACHE_SERVER_UDS"
"$binary" --zero-stats
test "$(listeners "$SCCACHE_SERVER_UDS")" -eq 1

printf 'foreign file\n' >"$scratch/foreign"
SCCACHE_DIR="$scratch/foreign-disk" SCCACHE_SERVER_UDS="$scratch/foreign" \
  refuse "foreign file" "$binary" --start-server
grep -Fxq 'foreign file' "$scratch/foreign"
mkdir -m 0777 "$scratch/unprotected"
SCCACHE_DIR="$scratch/unprotected" SCCACHE_SERVER_UDS="$scratch/unprotected.sock" \
  refuse "not owned and protected" "$binary" --start-server

SCCACHE_DIR="$scratch/abstract" SCCACHE_SERVER_UDS="$abstract" "$binary" --zero-stats
SCCACHE_DIR="$scratch/conflicting-disk" SCCACHE_SERVER_UDS="$abstract" \
  refuse "different cache directory or configuration" "$binary" --zero-stats
SCCACHE_DIR="$scratch/abstract" SCCACHE_SERVER_UDS="$abstract" "$binary" --stop-server >/dev/null

printf 'Managed cache startup, conflict, drain, and output restoration controls passed.\n'
