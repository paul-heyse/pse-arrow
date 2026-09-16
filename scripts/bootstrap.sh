#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Create a working development environment for this repository, idempotently.
# Run it again any time; every step is safe to repeat. `just bootstrap` is the entry point.
#
# Stages are separable because some are slow or network-bound:
#   --venv-only      interpreter + Python dependencies + the editable extension (uv sync)
#   --quality-only   the quality tools from [dependency-groups].quality (as uv.lock resolved them)
#   --rust-only      cargo development tools (cargo-binstall, current releases)
#   --linters-only   repository linters that are plain binaries (actionlint, ast-grep, ...)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export UV_PROJECT_ENVIRONMENT="${UV_PROJECT_ENVIRONMENT:-.venv}"
case "$UV_PROJECT_ENVIRONMENT" in
  /*|[A-Za-z]:*) VENV="$UV_PROJECT_ENVIRONMENT" ;;
  *) VENV="${ROOT}/${UV_PROJECT_ENVIRONMENT}" ;;
esac
PY="${VENV}/bin/python"  # used by doctor below on Windows
[[ "${OS:-}" == Windows_NT ]] && PY="${VENV}/Scripts/python.exe"
export PY

# The tool set, not its versions: cargo-binstall fetches the current release of each,
# CI installs the same names through .github/actions/setup-rust, and `just doctor`
# reports which are missing. No release is asserted anywhere.
CARGO_TOOLS=(
  "cargo-nextest" "cargo-deny"   "cargo-audit"
  "cargo-shear"   "cargo-machete" "cargo-llvm-cov"
  "cargo-insta"   "cargo-hack"   "cargo-msrv"
  "cargo-mutants" "cargo-geiger" "cargo-udeps"
  "cargo-semver-checks" "mdbook" "git-cliff"
)

say() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: $1 is required but not installed ($2)" >&2
    exit 1
  }
}

stage_venv() {
  need uv "https://docs.astral.sh/uv/"
  say "Syncing ${UV_PROJECT_ENVIRONMENT} from uv.lock (platform set + pyomo extra + dev groups)"
  # uv creates the venv on the interpreter from .python-version, installs the locked
  # dependency set, and builds the extension via maturin (PEP 517, editable).
  uv sync --locked --extra pyomo
}

stage_quality() {
  need uv "https://docs.astral.sh/uv/"
  say "Installing quality tools from [dependency-groups].quality"
  uv sync --locked --extra pyomo --group quality
}

stage_rust() {
  need cargo "https://rustup.rs"
  if ! command -v cargo-binstall >/dev/null 2>&1; then
    say "Installing cargo-binstall (prebuilt binaries beat compiling 15 tools)"
    cargo install cargo-binstall --locked
  fi
  say "Installing cargo development tools"
  cargo binstall --no-confirm "${CARGO_TOOLS[@]}"
}

stage_repo_linters() {
  # Linters for the repository's own configuration. Plain binaries with no Python or
  # Cargo home, fetched directly; taplo/typos/reuse come from the venv instead.
  local bindir="${HOME}/.local/bin"
  mkdir -p "$bindir"
  if ! command -v actionlint >/dev/null 2>&1; then
    say "Installing actionlint"
    curl -fsSL https://raw.githubusercontent.com/rhysd/actionlint/main/scripts/download-actionlint.bash \
      | bash -s -- latest "$bindir"
  fi
  if ! command -v ast-grep >/dev/null 2>&1; then
    say "Installing ast-grep"
    cargo binstall --no-confirm ast-grep
  fi
  if ! command -v shellcheck >/dev/null 2>&1; then
    echo "note: install shellcheck with your package manager (apt install shellcheck / brew install shellcheck)"
  fi
}

stage_hooks() {
  if [[ -x "$PY" ]]; then
    say "Installing git hooks (pre-commit and pre-push)"
    "$PY" -m pre_commit install --install-hooks
  fi
}

main() {
  case "${1:-all}" in
    --venv-only)    stage_venv ;;
    --quality-only) stage_quality ;;
    --rust-only)    stage_rust ;;
    --linters-only) stage_repo_linters ;;
    all)
      stage_venv
      stage_quality
      stage_rust
      stage_repo_linters
      stage_hooks
      ;;
    *) echo "usage: $0 [--venv-only|--quality-only|--rust-only|--linters-only]" >&2
       exit 2 ;;
  esac

  if command -v direnv >/dev/null 2>&1 && [[ -f .envrc ]]; then
    direnv status 2>/dev/null | grep -q "Found RC allowed 1\|Found RC allowed true" \
      || echo -e "\nnote: run 'direnv allow' to activate the environment on cd"
  fi

  say "Status"
  python3 scripts/doctor.py --format=text
}

main "$@"
