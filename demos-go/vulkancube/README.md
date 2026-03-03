## VulkanCube

![vulkan cube golang](http://dl.kc.vc/vulkan/screens/cube.gif)

## Supported platforms

* OS X / macOS (GLFW + MoltenVK)
* TrimUI Smart Pro (Linux, TODO)

## macOS setup (GLFW or SDL2)

Homebrew packages:
- `molten-vk`
- `vulkan-headers`
- `vulkan-loader`
- `vulkan-tools`
- `shaderc`
- `glfw` (for GLFW variant)
- `sdl2` (for SDL2 variant)

Optional:
- `HOMEBREW_NO_AUTO_UPDATE=1`

## Build shaders (optional, if you modify shaders)

From `/Users/$USER/code/trimui-vulkan/demos-go/vulkancube`:

```sh
make shaders
```

## Run on macOS

GLFW:
```sh
cd /Users/$USER/code/trimui-vulkan/demos-go/vulkancube/vulkancube_glfw
export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
CGO_LDFLAGS="-L/opt/homebrew/lib" go run .
```

SDL2:
```sh
cd /Users/$USER/code/trimui-vulkan/demos-go/vulkancube/vulkancube_sdl2
export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
CGO_LDFLAGS="-L/opt/homebrew/lib" go run .
```
