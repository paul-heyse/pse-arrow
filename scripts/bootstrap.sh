#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Create a working development environment for this repository, idempotently.
# Run it again any time; every step is safe to repeat. `just bootstrap` is the entry point.
#
# Stages are separable because some are slow or network-bound:
#   --venv-only      independent reference and repository-tool dependencies (uv sync)
#   --quality-only   the quality tools from [dependency-groups].quality (as uv.lock resolved them)
#   --rust-only      cargo development tools (protocol-sensitive versions pinned)
#   --linters-only   repository linters that are plain binaries (actionlint, ast-grep, ...)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
# This entry point prepares ordinary development tools, never a caller's oracle prefix.
unset VIRTUAL_ENV UV_PYTHON UV_PROJECT UV_WORKING_DIR UV_NO_PROJECT UV_NO_SYNC UV_FROZEN
export UV_PROJECT_ENVIRONMENT="$ROOT/.venv"

# Names live here; protocol-sensitive versions come from diagnostic-tools.toml.
CARGO_TOOLS=(
  "cargo-nextest" "cargo-deny"   "cargo-audit"
  "cargo-shear"   "cargo-llvm-cov"
  "cargo-insta"   "cargo-hack"   "cargo-msrv"
  "cargo-mutants" "cargo-fuzz"
  "cargo-semver-checks" "git-cliff"
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
  say "Syncing ${UV_PROJECT_ENVIRONMENT} from uv.lock (platform set + dev groups)"
  uv sync --locked --no-install-project
}

stage_quality() {
  need uv "https://docs.astral.sh/uv/"
  say "Installing quality tools from [dependency-groups].quality"
  uv sync --locked --no-install-project --group quality
}

stage_docs() {
  need uv "https://docs.astral.sh/uv/"
  # Selection and alias refusal belong to the shared boundary, before exact sync.
  "$ROOT/scripts/pse-env" --docs -- cargo run --locked -p xtask --no-default-features -- docs install
}

stage_rust() {
  need cargo "https://rustup.rs"
  if ! command -v cargo-binstall >/dev/null 2>&1; then
    say "Installing cargo-binstall (prebuilt binaries beat compiling 15 tools)"
    cargo install cargo-binstall --locked
  fi
  say "Installing cargo development tools"
  tool_specs_text=$("$ROOT/scripts/pse-host" cargo-tools "${CARGO_TOOLS[@]}")
  mapfile -t tool_specs <<< "$tool_specs_text"
  cargo binstall --no-confirm "${tool_specs[@]}"
  say "Installing pinned compiler cache"
  "$ROOT/scripts/pse-host" diagnostic-tools install sccache
  "$ROOT/scripts/pse-env" -- cargo run --locked -p xtask --no-default-features -- docs install
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

main() {
  "$ROOT/scripts/bootstrap-host"
  case "${1:-all}" in
    --venv-only)    stage_venv ;;
    --quality-only) stage_quality ;;
    --docs-only)    stage_docs; return ;;
    --rust-only)    stage_rust ;;
    --linters-only) stage_repo_linters ;;
    all)
      stage_venv
      stage_quality
      stage_rust
      stage_repo_linters
      ;;
    *) echo "usage: $0 [--venv-only|--quality-only|--docs-only|--rust-only|--linters-only]" >&2
       exit 2 ;;
  esac

  if command -v direnv >/dev/null 2>&1 && [[ -f .envrc ]]; then
    direnv status 2>/dev/null | grep -q "Found RC allowed 1\|Found RC allowed true" \
      || echo -e "\nnote: run 'direnv allow' to activate the environment on cd"
  fi

  # Skill aliases (a no-op where the committed symlinks work) and the selected library skills.
  "$ROOT/scripts/pse-env" -- cargo run --locked -p xtask --no-default-features -- agent-config
  "$ROOT/scripts/pse-env" -- cargo run --locked -p xtask --no-default-features -- skills

  say "Status"
  "$ROOT/scripts/pse-host" doctor
}

main "$@"
