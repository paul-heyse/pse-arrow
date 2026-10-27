#!/usr/bin/env bash
# measure.sh <label> <cmd...>: run from the worktree root with the worktree's pse-env, compiler cache disabled,
# under GNU time -v; log to raw/<label>.log
P="$(cd "$(dirname "$0")" && pwd)"; label=$1; shift
cd "$P/wt" || exit 1
{ echo "# label=$label start=$(date -Is) cmd=$*"; \
  env -u SCCACHE_CONF -u VIRTUAL_ENV -u VIRTUAL_ENV_PROMPT -u SCCACHE_SERVER_UDS -u PSE_SCCACHE_BINARY RUSTC_WRAPPER= \
    scripts/pse-env -- /usr/bin/time -v "$@"; echo "exit=$?"; } > "$P/raw/$label.log" 2>&1
grep -E "^exit=|Elapsed|Maximum resident|Finished" "$P/raw/$label.log"; grep -c "Compiling" "$P/raw/$label.log"
