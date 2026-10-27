#!/usr/bin/env bash
# Disposable probe fixture: pinned source-built server, in-memory store, scratch port.
set -euo pipefail
D="$(cd "$(dirname "$0")" && pwd)"
B="${XDG_DATA_HOME:-$HOME/.local/share}/pse-arrow/tools/surreal/v3.3.4-mx1.0.0-effective-v1-3549c2a97bd565fd2497/source/target/release/surreal"
# Drop any inherited SURREAL_*/OTEL_* settings, mirroring the supervisor.
for v in $(env | grep -oE '^(SURREAL|OTEL)_[A-Z0-9_]+'); do unset "$v"; done
export SURREAL_USER=root SURREAL_PASS="$(cat "$D/pass")"
export SURREAL_GRPC_MAX_MESSAGE_SIZE=4194304
exec "$B" start --no-banner --log info --deny-guests --deny-net \
  --bind "127.0.0.1:$(cat "$D/port")" --query-timeout=90s --transaction-timeout=90s \
  --temporary-directory "$D/tmp" memory
