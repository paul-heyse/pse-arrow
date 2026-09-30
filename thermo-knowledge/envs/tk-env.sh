#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Side environments of the thermodynamic knowledge base (Plan 24): create from the lock,
# re-solve the lock, or run a command inside one. Called by the tk-env-* recipes of the
# tree's justfile; see envs/README.md.
#
#   tk-env.sh sync <name>          create or refresh the environment from its lock
#   tk-env.sh lock <name>          re-solve the environment and rewrite its lock
#   tk-env.sh run <name> <cmd>...  run a command inside the environment
#
# An environment directory holds either a uv project (pyproject.toml, uv.lock) or a
# micromamba spec (environment.yml, environment.lock.txt, and optionally pip.in with
# pip.lock.txt for pip-only packages). The prefix is always <tree>/.store/envs/<name>,
# never a global environments directory. The caller's environment is neutralised: the
# repository .envrc puts the root .venv first on PATH and exports UV_PROJECT_ENVIRONMENT,
# and a user site directory (~/.local) would shadow the environment's own packages.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tree="$(dirname "$here")"
root="$(dirname "$tree")"
store="$tree/.store"

command="${1:?usage: tk-env.sh sync|lock|run <name> [command...]}"
name="${2:?usage: tk-env.sh $command <name>}"
shift 2
spec="$here/$name"
prefix="$store/envs/$name"
[[ -d $spec ]] || { echo "tk-env: no environment '$name' under $here" >&2; exit 2; }

unset VIRTUAL_ENV PYTHONPATH PYTHONHOME UV_PYTHON
while IFS= read -r variable; do unset "$variable"; done < <(compgen -e | grep -E '^(CONDA|MAMBA)' || true)
export PYTHONNOUSERSITE=1
export MAMBA_ROOT_PREFIX="$store/mamba-root"
export UV_PROJECT_ENVIRONMENT="$prefix"

# A heavy solve or install runs in its own memory-capped scope when the repository provides one.
cap=()
[[ -f $root/scripts/memory-cap.sh ]] && cap=(bash "$root/scripts/memory-cap.sh")

solve=(--no-rc --override-channels --strict-channel-priority -r "$store/mamba-root")
mkdir -p "$store/envs"

# Fingerprint of everything a prefix is created from.
stamp() {
  local file
  for file in "$spec/environment.lock.txt" "$spec/pip.lock.txt"; do
    if [[ -f $file ]]; then cat "$file"; fi
  done | sha256sum | cut -d' ' -f1
}

pip_install() {  # install the hashed pip lock into the conda prefix, without dependencies
  [[ -f $spec/pip.lock.txt ]] || return 0
  uv pip install --python "$prefix/bin/python" --require-hashes --no-deps -r "$spec/pip.lock.txt"
}

case "$command" in
  sync)
    if [[ -f $spec/pyproject.toml ]]; then
      "${cap[@]}" uv sync --project "$spec" --locked
    else
      [[ -f $spec/environment.lock.txt ]] || { echo "tk-env: $name has no lock; run tk-env-lock $name" >&2; exit 2; }
      if [[ -f $prefix/.tk-lock-stamp && "$(cat "$prefix/.tk-lock-stamp")" == "$(stamp)" ]]; then
        echo "tk-env: $name is up to date with its lock"
      else
        rm -rf "${prefix:?}"
        "${cap[@]}" micromamba create "${solve[@]}" -y -p "$prefix" -f "$spec/environment.lock.txt"
        pip_install
        stamp > "$prefix/.tk-lock-stamp"
      fi
    fi
    ;;
  lock)
    if [[ -f $spec/pyproject.toml ]]; then
      "${cap[@]}" uv lock --project "$spec"
    else
      # A fresh solve from the spec: the prefix is rebuilt, then the lock is exported from it.
      rm -rf "${prefix:?}"
      "${cap[@]}" micromamba create "${solve[@]}" -y -p "$prefix" -f "$spec/environment.yml"
      if [[ -f $spec/pip.in ]]; then
        # numpy is owned by the conda solve; everything else pip needs is locked with hashes.
        uv pip compile "$spec/pip.in" --python "$prefix/bin/python" --generate-hashes \
          --no-emit-package numpy --no-header -q -o "$spec/pip.lock.txt"
        pip_install
      fi
      micromamba env export --no-rc -r "$store/mamba-root" -p "$prefix" --explicit > "$spec/environment.lock.txt"
      stamp > "$prefix/.tk-lock-stamp"
    fi
    ;;
  run)
    [[ $# -gt 0 ]] || { echo "tk-env: run needs a command" >&2; exit 2; }
    if [[ -f $spec/pyproject.toml ]]; then
      exec uv run --project "$spec" --locked -- "$@"
    else
      [[ -x $prefix/bin/python ]] || { echo "tk-env: $name is not created; run tk-env-sync $name" >&2; exit 2; }
      exec micromamba run -r "$store/mamba-root" -p "$prefix" "$@"
    fi
    ;;
  *)
    echo "usage: tk-env.sh sync|lock|run <name> [command...]" >&2
    exit 2
    ;;
esac
