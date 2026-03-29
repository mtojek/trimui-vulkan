#!/usr/bin/env bash
set -euo pipefail

# TrimUI toolchain env provides SYSROOT/PREFIX/CROSS_*; use it if present.
if [[ -n "${PREFIX:-}" ]]; then
  export PKG_CONFIG_ALLOW_CROSS=1
  export SDL2_INCLUDE_PATH="${PREFIX}/include/SDL2"
  export SDL2_LIB_PATH="${PREFIX}/lib"

  # sdl2-sys links with -lSDL2, so ensure the linker can find libSDL2.so
  if [[ ! -e "${SDL2_LIB_PATH}/libSDL2.so" && -e "${SDL2_LIB_PATH}/libSDL2-2.0.so.0" ]]; then
    ln -sf "${SDL2_LIB_PATH}/libSDL2-2.0.so.0" "${SDL2_LIB_PATH}/libSDL2.so"
  fi
fi

# Avoid using the cross compiler for host build scripts/proc-macros.
if [[ -n "${CROSS_COMPILE:-}" && -n "${CROSS_ROOT:-}" ]]; then
  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="${CROSS_ROOT}/bin/${CROSS_COMPILE}gcc"
  export CC_aarch64_unknown_linux_gnu="${CROSS_ROOT}/bin/${CROSS_COMPILE}gcc"
  export CXX_aarch64_unknown_linux_gnu="${CROSS_ROOT}/bin/${CROSS_COMPILE}g++"
  export AR_aarch64_unknown_linux_gnu="${CROSS_ROOT}/bin/${CROSS_COMPILE}ar"

  if command -v cc >/dev/null 2>&1; then
    export CC="$(command -v cc)"
  fi
  if command -v c++ >/dev/null 2>&1; then
    export CXX="$(command -v c++)"
  fi
fi

TARGET="aarch64-unknown-linux-gnu"
if [[ -n "${CROSS_COMPILE:-}" ]]; then
  cargo build --release --target "${TARGET}"
else
  cargo build
fi
