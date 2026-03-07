# trimui-vulkan

Vulkan Go demos adapted for macOS and TrimUI Smart Pro workflows.

## What’s here

- `demos-go/vulkancube` – textured spinning cube (GLFW and SDL2 variants)
- `demos-go/vulkandraw` – simple triangle (GLFW)
- `demos-go/vulkaninfo` – device/info dump (GLFW)

Each demo has its own README with platform-specific steps.

## macOS quick start

See per-demo README files:

- [vulkancube README](demos-go/vulkancube/README.md)
- [vulkandraw README](demos-go/vulkandraw/README.md)
- [vulkaninfo README](demos-go/vulkaninfo/README.md)

## TrimUI Smart Pro (container)

Builds are done inside the toolchain container. See per-demo README files for runtime details.

Container setup in the repo root:

```sh
make build
make shell
```

Note: For TrimUI we currently build only the SDL2 variant (`vulkancube_sdl2`).

## TrimUI Smart Pro (SD layout)

Prepared SD card layout is in:

- `tg5040/Emus/tg5040/VUL.pak`
- `tg5040/Roms/Vulkan Demos (VUL)`

This layout assumes built binaries are placed in `/mnt/SDCARD/Roms/Vulkan Demos (VUL)/.demos-go` on the device.
