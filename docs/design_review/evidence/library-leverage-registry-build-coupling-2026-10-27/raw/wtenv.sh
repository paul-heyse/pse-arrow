#!/usr/bin/env bash
# Run a command from the scratch worktree root with the worktree's own pse-env selection.
# Drops the main checkout's session-specific wrapper/venv variables so pse-env recomposes them.
cd "$(dirname "$0")/wt" || exit 1
exec env -u RUSTC_WRAPPER -u SCCACHE_CONF -u VIRTUAL_ENV -u VIRTUAL_ENV_PROMPT -u SCCACHE_SERVER_UDS \
  scripts/pse-env -- "$@"
