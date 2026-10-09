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
# Canonical application tests use the configured supervised loopback WebSocket server.
# Preserve its private profile paths and host loopback namespace when requested.
store=()
network=none
if [[ -n "${PSE_SURREAL_STATE:-}" ]]; then
  pse_state="$(realpath "$PSE_SURREAL_STATE")"
  case "$pse_state/" in
    "$root/"*) ;;
    *) mounts+=(-v "$pse_state:$pse_state:ro") ;;
  esac
  store=(-e "PSE_SURREAL_STATE=$pse_state")
  network=host
fi
exec docker run --rm --network "$network" --user "$(id -u):$(id -g)" \
  -e PSE_SOLVER_MEASURE -e SYMBOLICA_LICENSE "${store[@]}" "${pipeline_env[@]}" \
  "${mounts[@]}" -w "$root" "$image" "$@"
