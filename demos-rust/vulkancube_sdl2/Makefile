.PHONY: clean build build-trimui

clean:
	cargo clean

build:
	cargo build --release

build-trimui:
	PKG_CONFIG_ALLOW_CROSS=1 \
	SDL2_INCLUDE_PATH="$${PREFIX}/include/SDL2" \
	SDL2_LIB_PATH="$${PREFIX}/lib" \
	RUSTFLAGS="-C link-args=-Wl,-rpath,$${PREFIX}/lib $${RUSTFLAGS}" \
	cargo build --release --target aarch64-unknown-linux-gnu
