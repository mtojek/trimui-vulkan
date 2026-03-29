#!/usr/bin/env bash
set -euo pipefail

echo "Marcin 1"
export PKG_CONFIG_ALLOW_CROSS=1
export SDL2_INCLUDE_PATH="${PREFIX}/include/SDL2"
export SDL2_LIB_PATH="${PREFIX}/lib"

if [[ ! -e "${SDL2_LIB_PATH}/libSDL2.so" && -e "${SDL2_LIB_PATH}/libSDL2-2.0.so.0" ]]; then
  echo "Marcin 2"
  ln -sf "${SDL2_LIB_PATH}/libSDL2-2.0.so.0" "${SDL2_LIB_PATH}/libSDL2.so"
fi

export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="${CROSS_ROOT}/bin/${CROSS_COMPILE}gcc"
export CC_aarch64_unknown_linux_gnu="${CROSS_ROOT}/bin/${CROSS_COMPILE}gcc"
export CXX_aarch64_unknown_linux_gnu="${CROSS_ROOT}/bin/${CROSS_COMPILE}g++"
export AR_aarch64_unknown_linux_gnu="${CROSS_ROOT}/bin/${CROSS_COMPILE}ar"
export CC="$(command -v cc)"

cargo build --target aarch64-unknown-linux-gnu
