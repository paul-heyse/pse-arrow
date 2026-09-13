#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Create a working development environment for this repository, idempotently.
# Run it again any time; every step is safe to repeat. `just bootstrap` is the entry point.
#
# Stages are separable because some are slow or network-bound:
#   --venv-only      interpreter + Python dependencies + the editable extension (uv sync)
#   --quality-only   the pinned quality tools from [dependency-groups].quality
#   --rust-only      pinned cargo development tools (cargo-binstall)
#   --linters-only   repository linters that are plain binaries (actionlint, ast-grep, ...)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
export UV_PROJECT_ENVIRONMENT="${UV_PROJECT_ENVIRONMENT:-.venv}"
VENV="${ROOT}/${UV_PROJECT_ENVIRONMENT}"
PY="${VENV}/bin/python"  # used by doctor below on Windows
[[ "${OS:-}" == Windows_NT ]] && PY="${VENV}/Scripts/python.exe"
export PY

# Versions pinned here, not on the machine. CI installs the same set (see
# .github/actions/setup-rust); `just doctor` reports what is missing.
CARGO_TOOLS=(
  "cargo-nextest@0.9.143" "cargo-deny@0.20.2"   "cargo-audit@0.22.2"
  "cargo-shear@1.13.4"    "cargo-machete@0.9.2" "cargo-llvm-cov@0.9.0"
  "cargo-insta@1.48.0"    "cargo-hack@0.6.45"   "cargo-msrv@0.19.3"
  "cargo-mutants@27.1.0"  "cargo-geiger@0.13.0" "cargo-udeps@0.1.61"
  "cargo-semver-checks@0.50.0" "mdbook@0.5.4"   "git-cliff@2.14.1"
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
  say "Installing pinned quality tools from [dependency-groups].quality"
  uv sync --locked --extra pyomo --group quality
}

stage_rust() {
  need cargo "https://rustup.rs"
  if ! command -v cargo-binstall >/dev/null 2>&1; then
    say "Installing cargo-binstall (prebuilt binaries beat compiling 15 tools)"
    cargo install cargo-binstall --locked
  fi
  say "Installing pinned cargo development tools"
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
  if [[ -x "${VENV}/bin/pre-commit" ]]; then
    say "Installing git hooks (pre-commit and pre-push)"
    "${VENV}/bin/pre-commit" install --install-hooks
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
  python3 scripts/doctor.py --format=text || true
}

main "$@"
