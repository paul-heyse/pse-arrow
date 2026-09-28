#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Run native test binaries in the solver image whose prefix native-solver-env.sh
# extracted: the pinned dev image, or the immutable PSE_SOLVER_IMAGE override.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
image="$(python3 scripts/solver-images.py runtime dev)"
mounts=(-v "$root:$root:ro")
target="${CARGO_TARGET_DIR:-$root/target}"
target="$(realpath "$target")"
case "$target/" in
  "$root/"*) ;;
  *) mounts+=(-v "$target:$target:ro") ;;
esac
# Durable-run tests reach the local operational store (ADR-0112) over its Unix socket,
# which works without a network. PGUSER names the peer-authenticated role, because the
# container has no passwd entry for the host user.
store=()
if [[ -d /var/run/postgresql ]]; then
  store=(-v /var/run/postgresql:/var/run/postgresql -e DATABASE_URL -e PSE_DATABASE_URL
    -e "PGUSER=${PGUSER:-$(id -un)}")
fi
exec docker run --rm --network none --user "$(id -u):$(id -g)" \
  -e PSE_SOLVER_MEASURE -e SYMBOLICA_LICENSE "${store[@]}" \
  "${mounts[@]}" -w "$root" "$image" "$@"
