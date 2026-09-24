#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# Source before the selected native solver feature build. Build upstream vendored
# KLU explicitly: suitesparse_sys 0.1.4 exports DEP paths before its vendor build
# and otherwise enables CHOLMOD/CUDA. No upstream source is modified.
set -euo pipefail
pse_math_root="$(git rev-parse --show-toplevel)"
pse_klu_prefix="$pse_math_root/target/native-math/klu-profile-v1"
mkdir -p "$pse_math_root/target/native-math"
(
flock 9
if [[ ! -f "$pse_klu_prefix/.complete" ]]; then
  pse_suite_source="$(cargo metadata --offline --locked --features pse-backend-native/native-solvers --format-version 1 | "$pse_math_root/.venv/bin/python" -c 'import json,sys,pathlib; p=[p for p in json.load(sys.stdin)["packages"] if p["name"]=="suitesparse_sys" and p["version"]=="0.1.4"]; assert len(p)==1; print(pathlib.Path(p[0]["manifest_path"]).parent / "vendor")')"
  cmake -S "$pse_suite_source" -B "$pse_klu_prefix/build" \
    -DCMAKE_INSTALL_PREFIX="$pse_klu_prefix" -DCMAKE_INSTALL_LIBDIR=lib \
    -DCMAKE_BUILD_TYPE=Release -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
    -DBUILD_SHARED_LIBS=OFF -DBUILD_STATIC_LIBS=ON \
    '-DSUITESPARSE_ENABLE_PROJECTS=suitesparse_config;amd;btf;colamd;klu' \
    -DKLU_USE_CHOLMOD=OFF -DSUITESPARSE_USE_CUDA=OFF \
    -DSUITESPARSE_CONFIG_USE_OPENMP=OFF -DSUITESPARSE_DEMOS=OFF
  cmake --build "$pse_klu_prefix/build" --parallel "${CARGO_BUILD_JOBS:-4}"
  cmake --install "$pse_klu_prefix/build"
  test -f "$pse_klu_prefix/include/suitesparse/klu.h"
  test -f "$pse_klu_prefix/lib/libklu.a"
  touch "$pse_klu_prefix/.complete"
fi
) 9>"$pse_math_root/target/native-math/klu-profile-v1.lock"
export SUITESPARSE_INCLUDE_DIR="$pse_klu_prefix/include/suitesparse"
export SUITESPARSE_LIBRARY_DIR="$pse_klu_prefix/lib"
