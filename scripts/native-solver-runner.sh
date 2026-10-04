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
# Scoped pipeline libraries live in the same content-addressed host cache as the
# extracted solver prefix. Preserve their actual runtime paths in the isolated runner.
pipeline_env=()
for pse_prefix_name in IPOPT_DIR UNO_DIR PETSC_DIR; do
  pse_prefix="${!pse_prefix_name:-}"
  if [[ -n "$pse_prefix" && -d "$pse_prefix" ]]; then
    case "$pse_prefix/" in
      "$root/"*) ;;
      *) mounts+=(-v "$pse_prefix:$pse_prefix:ro") ;;
    esac
    pipeline_env+=(-e "$pse_prefix_name=$pse_prefix")
  fi
done
pipeline_env+=(-e "LD_LIBRARY_PATH=${LD_LIBRARY_PATH:-}")
target="${CARGO_TARGET_DIR:-$root/target}"
target="$(realpath "$target")"
case "$target/" in
  "$root/"*) ;;
  *) mounts+=(-v "$target:$target:ro") ;;
esac
# Durable-run tests reach the local operational store (ADR-0114) over its Unix socket,
# which works without a network. PGUSER names the peer-authenticated role, because the
# container has no passwd entry for the host user; the store honours it as libpq does.
store=()
if [[ -d /var/run/postgresql ]]; then
  store=(-v /var/run/postgresql:/var/run/postgresql -e PSE_DATABASE_URL
    -e "PGUSER=${PGUSER:-$(id -un)}")
fi
exec docker run --rm --network none --user "$(id -u):$(id -g)" \
  -e PSE_SOLVER_MEASURE -e SYMBOLICA_LICENSE "${store[@]}" "${pipeline_env[@]}" \
  "${mounts[@]}" -w "$root" "$image" "$@"
