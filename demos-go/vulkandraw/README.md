## VulkanDraw

<img src="screens/triangle.png" alt="vulkan triangle golang" width="200" height="200">

## Supported platforms

- Linux graphics (GLFW, desktop only)
- OS X / macOS (GLFW + MoltenVK)

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
