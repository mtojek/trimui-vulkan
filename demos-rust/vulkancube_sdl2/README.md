# VulkanCube (SDL2, Rust)

## Supported platforms

- macOS (SDL2 + MoltenVK)
- TrimUI Smart Pro (Linux, SDL2 only)

## macOS

### Setup

Homebrew packages:

- `molten-vk`
- `vulkan-headers`
- `vulkan-loader`
- `vulkan-tools`
- `shaderc`
- `sdl2`

Optional:

- `HOMEBREW_NO_AUTO_UPDATE=1`

### Build

From `/Users/$USER/code/trimui-vulkan/demos-rust/vulkancube_sdl2`:

```sh
cargo build
```

### Run

```sh
export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$PKG_CONFIG_PATH"
cargo run
```

## TrimUI Smart Pro

### Container

Build inside the container (from the `vulkancube_sdl2` folder):

```sh
cargo build --release
```

### Runtime

Uses system SDL2:

```sh
export LD_LIBRARY_PATH=/usr/trimui/lib:$LD_LIBRARY_PATH
./target/release/vulkancube_sdl2
```

### Controls (SDL2)

- `A` key / controller button A: slow down rotation
- `B` key / controller button B: speed up rotation
- controller button `5` (which=0): exit (same as `Esc`)
