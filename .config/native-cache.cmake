# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# cmake-rs honors CMAKE_TOOLCHAIN_FILE; plain launcher environment variables
# are not forwarded by the pinned HiGHS/SUNDIALS build scripts.
if(DEFINED ENV{PSE_NATIVE_COMPILER_CACHE})
  set(CMAKE_C_COMPILER_LAUNCHER "$ENV{PSE_NATIVE_COMPILER_CACHE}" CACHE STRING "C compiler cache")
  set(CMAKE_CXX_COMPILER_LAUNCHER "$ENV{PSE_NATIVE_COMPILER_CACHE}" CACHE STRING "C++ compiler cache")
endif()
