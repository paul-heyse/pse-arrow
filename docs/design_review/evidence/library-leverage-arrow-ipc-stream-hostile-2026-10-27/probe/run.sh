#!/usr/bin/env bash
# Usage (from the repository root):
#   scripts/pse-env -- bash <probe>/run.sh <probe-binary> <cases-dir> > <probe>/outputs/runs.txt
# Each (case, decoder, cap) runs in its own child shell so an abort/panic is observed, not fatal.
set -u
bin=$1; cases=$2
"$bin" gen "$cases" 2>&1 | sed 's/^/gen: /'
for f in "$cases"/*.arrows; do
  name=$(basename "$f" .arrows)
  for mode in reader schema-take decoder decoder-chunk64; do
    for cap in none 4GiB; do
      case $mode in
        reader) cmd=("$bin" reader "$f");;
        schema-take) cmd=("$bin" schema-take "$f");;
        decoder) cmd=("$bin" decoder "$f");;
        decoder-chunk64) cmd=("$bin" decoder "$f" 64);;
      esac
      if [ "$cap" = none ]; then
        out=$(bash -c '"$@"' _ "${cmd[@]}" 2>&1); rc=$?
      else
        out=$(bash -c 'ulimit -v 4194304; "$@"' _ "${cmd[@]}" 2>&1); rc=$?
      fi
      if [ $rc -gt 128 ]; then status="signal $((rc-128)) ($(kill -l $((rc-128))))"; else status="exit $rc"; fi
      echo "=== case=$name mode=$mode cap=$cap -> $status"
      printf '%s\n' "$out" | grep -v '^note: run with' | sed 's/^/    /'
    done
  done
done
