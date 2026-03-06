## VulkanDraw

<img src="screens/triangle.png" alt="vulkan triangle golang" width="200" height="200">

## Supported platforms

- Linux graphics (GLFW, desktop only)
- OS X / macOS (GLFW + MoltenVK)
- TrimUI Smart Pro (Linux, SDL2 only, TODO)

## macOS setup (GLFW)

Homebrew packages:

- `molten-vk`
- `vulkan-headers`
- `vulkan-loader`
- `vulkan-tools`
- `shaderc`
- `glfw`

Optional:

- `HOMEBREW_NO_AUTO_UPDATE=1`

## Build shaders (optional, if you modify shaders)

From `/Users/$USER/code/trimui-vulkan/demos-go/vulkandraw`:

```sh
make shaders
```

## Run on macOS

```sh
cd /Users/$USER/code/trimui-vulkan/demos-go/vulkandraw/vulkandraw_glfw
export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
CGO_LDFLAGS="-L/opt/homebrew/lib" go run .
```

## TrimUI (container)

Currently we build only the SDL2 variant in the TrimUI container. GLFW requires X11/Wayland and is not supported on TrimUI.

Build inside the container (from the `vulkandraw_sdl2` folder):
```sh
go build .
```

Runtime on TrimUI (uses system SDL2):
```sh
export LD_LIBRARY_PATH=/usr/trimui/lib:$LD_LIBRARY_PATH
./vulkandraw_sdl2
```
