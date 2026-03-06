# VulkanInfo

## TrimUI Smart Pro

### Example output

#### Udev devices (SDL2 log)

```text
SDL_UDEV_DEVICEADDED 2 /dev/input/event1
SDL_UDEV_DEVICEADDED 8 /dev/audio
SDL_UDEV_DEVICEADDED 8 /dev/dsp
SDL_UDEV_DEVICEADDED 2 /dev/input/event2
SDL_UDEV_DEVICEADDED 8 /dev/mixer
SDL_UDEV_DEVICEADDED 8 /dev/snd/pcmC0D0c
SDL_UDEV_DEVICEADDED 8 /dev/snd/pcmC0D0p
SDL_UDEV_DEVICEADDED 8 /dev/snd/controlC0
SDL_UDEV_DEVICEADDED 2 /dev/input/event0
SDL_UDEV_DEVICEADDED 4 /dev/input/event3
SDL_UDEV_DEVICEADDED 4 /dev/input/js0
SDL_UDEV_DEVICEADDED 8 /dev/snd/seq
SDL_UDEV_DEVICEADDED 8 /dev/sequencer
SDL_UDEV_DEVICEADDED 8 /dev/sequencer2
SDL_UDEV_DEVICEADDED 8 /dev/snd/timer
```

#### Vulkan device table

```text
╭───────────────────────────────────────────────────────────────────────╮
│               VULKAN PROPERTIES AND SURFACE CAPABILITES               │
├────────────────────────┬──────────────────────────────────────────────┤
│ Physical Device Name   │ PowerVR Rogue GE8300                         │
│ Physical Device Vendor │ 1010                                         │
│ Physical Device Type   │ Integrated GPU                               │
│ Physical GPUs          │ 1                                            │
│ API Version            │ 1.3.225                                      │
│ API Version Supported  │ 1.3.225                                      │
│ Driver Version         │ 1.525.317                                    │
│ INSTANCE EXTENSIONS    │                                              │
│ 1                      │ VK_KHR_device_group_creation                 │
│ 2                      │ VK_KHR_display                               │
│ 3                      │ VK_KHR_external_fence_capabilities           │
│ 4                      │ VK_KHR_external_memory_capabilities          │
│ 5                      │ VK_KHR_external_semaphore_capabilities       │
│ 6                      │ VK_KHR_get_display_properties2               │
│ 7                      │ VK_KHR_get_physical_device_properties2       │
│ 8                      │ VK_KHR_get_surface_capabilities2             │
│ 9                      │ VK_KHR_surface                               │
│ 10                     │ VK_EXT_debug_report                          │
│ 11                     │ VK_EXT_debug_utils                           │
│ 12                     │ VK_EXT_headless_surface                      │
├────────────────────────┼──────────────────────────────────────────────┤
│ DEVICE EXTENSIONS      │                                              │
│ 1                      │ VK_KHR_16bit_storage                         │
│ 2                      │ VK_KHR_8bit_storage                          │
│ 3                      │ VK_KHR_bind_memory2                          │
│ 4                      │ VK_KHR_buffer_device_address                 │
│ 5                      │ VK_KHR_copy_commands2                        │
│ 6                      │ VK_KHR_create_renderpass2                    │
│ 7                      │ VK_KHR_dedicated_allocation                  │
│ 8                      │ VK_KHR_depth_stencil_resolve                 │
│ 9                      │ VK_KHR_descriptor_update_template            │
│ 10                     │ VK_KHR_device_group                          │
│ 11                     │ VK_KHR_draw_indirect_count                   │
│ 12                     │ VK_KHR_driver_properties                     │
│ 13                     │ VK_KHR_dynamic_rendering                     │
│ 14                     │ VK_KHR_external_fence                        │
│ 15                     │ VK_KHR_external_fence_fd                     │
│ 16                     │ VK_KHR_external_memory                       │
│ 17                     │ VK_KHR_external_memory_fd                    │
│ 18                     │ VK_KHR_external_semaphore                    │
│ 19                     │ VK_KHR_external_semaphore_fd                 │
│ 20                     │ VK_KHR_format_feature_flags2                 │
│ 21                     │ VK_KHR_get_memory_requirements2              │
│ 22                     │ VK_KHR_global_priority                       │
│ 23                     │ VK_KHR_image_format_list                     │
│ 24                     │ VK_KHR_imageless_framebuffer                 │
│ 25                     │ VK_KHR_maintenance1                          │
│ 26                     │ VK_KHR_maintenance2                          │
│ 27                     │ VK_KHR_maintenance3                          │
│ 28                     │ VK_KHR_maintenance4                          │
│ 29                     │ VK_KHR_multiview                             │
│ 30                     │ VK_KHR_push_descriptor                       │
│ 31                     │ VK_KHR_relaxed_block_layout                  │
│ 32                     │ VK_KHR_sampler_mirror_clamp_to_edge          │
│ 33                     │ VK_KHR_sampler_ycbcr_conversion              │
│ 34                     │ VK_KHR_separate_depth_stencil_layouts        │
│ 35                     │ VK_KHR_shader_draw_parameters                │
│ 36                     │ VK_KHR_shader_float16_int8                   │
│ 37                     │ VK_KHR_shader_float_controls                 │
│ 38                     │ VK_KHR_shader_integer_dot_product            │
│ 39                     │ VK_KHR_shader_non_semantic_info              │
│ 40                     │ VK_KHR_shader_subgroup_extended_types        │
│ 41                     │ VK_KHR_shader_terminate_invocation           │
│ 42                     │ VK_KHR_spirv_1_4                             │
│ 43                     │ VK_KHR_storage_buffer_storage_class          │
│ 44                     │ VK_KHR_swapchain                             │
│ 45                     │ VK_KHR_swapchain_mutable_format              │
│ 46                     │ VK_KHR_synchronization2                      │
│ 47                     │ VK_KHR_timeline_semaphore                    │
│ 48                     │ VK_KHR_uniform_buffer_standard_layout        │
│ 49                     │ VK_KHR_variable_pointers                     │
│ 50                     │ VK_KHR_vulkan_memory_model                   │
│ 51                     │ VK_KHR_zero_initialize_workgroup_memory      │
│ 52                     │ VK_EXT_attachment_feedback_loop_layout       │
│ 53                     │ VK_EXT_blend_operation_advanced              │
│ 54                     │ VK_EXT_buffer_device_address                 │
│ 55                     │ VK_EXT_conditional_rendering                 │
│ 56                     │ VK_EXT_debug_marker                          │
│ 57                     │ VK_EXT_depth_clip_control                    │
│ 58                     │ VK_EXT_device_memory_report                  │
│ 59                     │ VK_EXT_extended_dynamic_state                │
│ 60                     │ VK_EXT_extended_dynamic_state2               │
│ 61                     │ VK_EXT_external_memory_dma_buf               │
│ 62                     │ VK_EXT_global_priority                       │
│ 63                     │ VK_EXT_global_priority_query                 │
│ 64                     │ VK_EXT_host_query_reset                      │
│ 65                     │ VK_EXT_image_compression_control             │
│ 66                     │ VK_EXT_image_drm_format_modifier             │
│ 67                     │ VK_EXT_image_robustness                      │
│ 68                     │ VK_EXT_index_type_uint8                      │
│ 69                     │ VK_EXT_inline_uniform_block                  │
│ 70                     │ VK_EXT_pipeline_creation_cache_control       │
│ 71                     │ VK_EXT_pipeline_creation_feedback            │
│ 72                     │ VK_EXT_pipeline_robustness                   │
│ 73                     │ VK_EXT_primitive_topology_list_restart       │
│ 74                     │ VK_EXT_private_data                          │
│ 75                     │ VK_EXT_provoking_vertex                      │
│ 76                     │ VK_EXT_queue_family_foreign                  │
│ 77                     │ VK_EXT_rasterization_order_attachment_access │
│ 78                     │ VK_EXT_scalar_block_layout                   │
│ 79                     │ VK_EXT_separate_stencil_usage                │
│ 80                     │ VK_EXT_shader_demote_to_helper_invocation    │
│ 81                     │ VK_EXT_subgroup_size_control                 │
│ 82                     │ VK_EXT_subpass_merge_feedback                │
│ 83                     │ VK_EXT_texel_buffer_alignment                │
│ 84                     │ VK_EXT_tooling_info                          │
│ 85                     │ VK_EXT_vertex_attribute_divisor              │
│ 86                     │ VK_IMG_format_pvrtc                          │
│ 87                     │ VK_ARM_rasterization_order_attachment_access │
╰────────────────────────┴──────────────────────────────────────────────╯
```

## macOS

### Example output

```text
╭──────────────────────────────────────────────────────────────────────╮
│              VULKAN PROPERTIES AND SURFACE CAPABILITES               │
├────────────────────────┬─────────────────────────────────────────────┤
│ Physical Device Name   │ Apple M1 Pro                                │
│ Physical Device Vendor │ 106b                                        │
│ Physical Device Type   │ Integrated GPU                              │
│ Physical GPUs          │ 1                                           │
│ API Version            │ 1.0.323                                     │
│ API Version Supported  │ 1.0.323                                     │
│ Driver Version         │ 0.2.2208                                    │
│ INSTANCE EXTENSIONS    │                                             │
│ 1                      │ VK_KHR_device_group_creation                │
│ 2                      │ VK_KHR_external_fence_capabilities          │
│ 3                      │ VK_KHR_external_memory_capabilities         │
│ 4                      │ VK_KHR_external_semaphore_capabilities      │
│ 5                      │ VK_KHR_get_physical_device_properties2      │
│ 6                      │ VK_KHR_get_surface_capabilities2            │
│ 7                      │ VK_KHR_surface                              │
│ 8                      │ VK_KHR_surface_protected_capabilities       │
│ 9                      │ VK_EXT_debug_report                         │
│ 10                     │ VK_EXT_debug_utils                          │
│ 11                     │ VK_EXT_headless_surface                     │
│ 12                     │ VK_EXT_layer_settings                       │
│ 13                     │ VK_EXT_metal_surface                        │
│ 14                     │ VK_EXT_surface_maintenance1                 │
│ 15                     │ VK_EXT_swapchain_colorspace                 │
│ 16                     │ VK_MVK_macos_surface                        │
│ 17                     │ VK_KHR_portability_enumeration              │
│ 18                     │ VK_LUNARG_direct_driver_loading             │
├────────────────────────┼─────────────────────────────────────────────┤
│ DEVICE EXTENSIONS      │                                             │
│ 1                      │ VK_KHR_16bit_storage                        │
│ 2                      │ VK_KHR_8bit_storage                         │
│ 3                      │ VK_KHR_bind_memory2                         │
│ 4                      │ VK_KHR_buffer_device_address                │
│ 5                      │ VK_KHR_calibrated_timestamps                │
│ 6                      │ VK_KHR_copy_commands2                       │
│ 7                      │ VK_KHR_create_renderpass2                   │
│ 8                      │ VK_KHR_dedicated_allocation                 │
│ 9                      │ VK_KHR_deferred_host_operations             │
│ 10                     │ VK_KHR_depth_stencil_resolve                │
│ 11                     │ VK_KHR_descriptor_update_template           │
│ 12                     │ VK_KHR_device_group                         │
│ 13                     │ VK_KHR_driver_properties                    │
│ 14                     │ VK_KHR_dynamic_rendering                    │
│ 15                     │ VK_KHR_dynamic_rendering_local_read         │
│ 16                     │ VK_KHR_external_fence                       │
│ 17                     │ VK_KHR_external_memory                      │
│ 18                     │ VK_KHR_external_semaphore                   │
│ 19                     │ VK_KHR_format_feature_flags2                │
│ 20                     │ VK_KHR_fragment_shader_barycentric          │
│ 21                     │ VK_KHR_get_memory_requirements2             │
│ 22                     │ VK_KHR_global_priority                      │
│ 23                     │ VK_KHR_image_format_list                    │
│ 24                     │ VK_KHR_imageless_framebuffer                │
│ 25                     │ VK_KHR_incremental_present                  │
│ 26                     │ VK_KHR_index_type_uint8                     │
│ 27                     │ VK_KHR_line_rasterization                   │
│ 28                     │ VK_KHR_load_store_op_none                   │
│ 29                     │ VK_KHR_maintenance1                         │
│ 30                     │ VK_KHR_maintenance2                         │
│ 31                     │ VK_KHR_maintenance3                         │
│ 32                     │ VK_KHR_maintenance4                         │
│ 33                     │ VK_KHR_maintenance5                         │
│ 34                     │ VK_KHR_maintenance6                         │
│ 35                     │ VK_KHR_maintenance7                         │
│ 36                     │ VK_KHR_maintenance8                         │
│ 37                     │ VK_KHR_map_memory2                          │
│ 38                     │ VK_KHR_multiview                            │
│ 39                     │ VK_KHR_portability_subset                   │
│ 40                     │ VK_KHR_present_id                           │
│ 41                     │ VK_KHR_present_id2                          │
│ 42                     │ VK_KHR_present_wait                         │
│ 43                     │ VK_KHR_present_wait2                        │
│ 44                     │ VK_KHR_push_descriptor                      │
│ 45                     │ VK_KHR_relaxed_block_layout                 │
│ 46                     │ VK_KHR_robustness2                          │
│ 47                     │ VK_KHR_sampler_mirror_clamp_to_edge         │
│ 48                     │ VK_KHR_sampler_ycbcr_conversion             │
│ 49                     │ VK_KHR_separate_depth_stencil_layouts       │
│ 50                     │ VK_KHR_shader_draw_parameters               │
│ 51                     │ VK_KHR_shader_expect_assume                 │
│ 52                     │ VK_KHR_shader_float_controls                │
│ 53                     │ VK_KHR_shader_float_controls2               │
│ 54                     │ VK_KHR_shader_float16_int8                  │
│ 55                     │ VK_KHR_shader_integer_dot_product           │
│ 56                     │ VK_KHR_shader_maximal_reconvergence         │
│ 57                     │ VK_KHR_shader_non_semantic_info             │
│ 58                     │ VK_KHR_shader_quad_control                  │
│ 59                     │ VK_KHR_shader_relaxed_extended_instruction  │
│ 60                     │ VK_KHR_shader_subgroup_extended_types       │
│ 61                     │ VK_KHR_shader_subgroup_rotate               │
│ 62                     │ VK_KHR_shader_subgroup_uniform_control_flow │
│ 63                     │ VK_KHR_shader_terminate_invocation          │
│ 64                     │ VK_KHR_spirv_1_4                            │
│ 65                     │ VK_KHR_storage_buffer_storage_class         │
│ 66                     │ VK_KHR_swapchain                            │
│ 67                     │ VK_KHR_swapchain_mutable_format             │
│ 68                     │ VK_KHR_synchronization2                     │
│ 69                     │ VK_KHR_timeline_semaphore                   │
│ 70                     │ VK_KHR_uniform_buffer_standard_layout       │
│ 71                     │ VK_KHR_variable_pointers                    │
│ 72                     │ VK_KHR_vertex_attribute_divisor             │
│ 73                     │ VK_KHR_vulkan_memory_model                  │
│ 74                     │ VK_KHR_zero_initialize_workgroup_memory     │
│ 75                     │ VK_EXT_4444_formats                         │
│ 76                     │ VK_EXT_buffer_device_address                │
│ 77                     │ VK_EXT_calibrated_timestamps                │
│ 78                     │ VK_EXT_debug_marker                         │
│ 79                     │ VK_EXT_depth_clip_control                   │
│ 80                     │ VK_EXT_descriptor_indexing                  │
│ 81                     │ VK_EXT_extended_dynamic_state               │
│ 82                     │ VK_EXT_extended_dynamic_state2              │
│ 83                     │ VK_EXT_extended_dynamic_state3              │
│ 84                     │ VK_EXT_external_memory_host                 │
│ 85                     │ VK_EXT_external_memory_metal                │
│ 86                     │ VK_EXT_fragment_shader_interlock            │
│ 87                     │ VK_EXT_global_priority                      │
│ 88                     │ VK_EXT_global_priority_query                │
│ 89                     │ VK_EXT_hdr_metadata                         │
│ 90                     │ VK_EXT_host_image_copy                      │
│ 91                     │ VK_EXT_host_query_reset                     │
│ 92                     │ VK_EXT_image_2d_view_of_3d                  │
│ 93                     │ VK_EXT_image_robustness                     │
│ 94                     │ VK_EXT_index_type_uint8                     │
│ 95                     │ VK_EXT_inline_uniform_block                 │
│ 96                     │ VK_EXT_line_rasterization                   │
│ 97                     │ VK_EXT_load_store_op_none                   │
│ 98                     │ VK_EXT_memory_budget                        │
│ 99                     │ VK_EXT_metal_objects                        │
│ 100                    │ VK_EXT_pipeline_creation_cache_control      │
│ 101                    │ VK_EXT_pipeline_creation_feedback           │
│ 102                    │ VK_EXT_pipeline_robustness                  │
│ 103                    │ VK_EXT_post_depth_coverage                  │
│ 104                    │ VK_EXT_private_data                         │
│ 105                    │ VK_EXT_robustness2                          │
│ 106                    │ VK_EXT_sample_locations                     │
│ 107                    │ VK_EXT_scalar_block_layout                  │
│ 108                    │ VK_EXT_separate_stencil_usage               │
│ 109                    │ VK_EXT_shader_atomic_float                  │
│ 110                    │ VK_EXT_shader_demote_to_helper_invocation   │
│ 111                    │ VK_EXT_shader_stencil_export                │
│ 112                    │ VK_EXT_shader_subgroup_ballot               │
│ 113                    │ VK_EXT_shader_subgroup_vote                 │
│ 114                    │ VK_EXT_shader_viewport_index_layer          │
│ 115                    │ VK_EXT_subgroup_size_control                │
│ 116                    │ VK_EXT_swapchain_maintenance1               │
│ 117                    │ VK_EXT_texel_buffer_alignment               │
│ 118                    │ VK_EXT_texture_compression_astc_hdr         │
│ 119                    │ VK_EXT_tooling_info                         │
│ 120                    │ VK_EXT_vertex_attribute_divisor             │
│ 121                    │ VK_AMD_gpu_shader_half_float                │
│ 122                    │ VK_AMD_negative_viewport_height             │
│ 123                    │ VK_AMD_shader_image_load_store_lod          │
│ 124                    │ VK_AMD_shader_trinary_minmax                │
│ 125                    │ VK_GOOGLE_display_timing                    │
│ 126                    │ VK_IMG_format_pvrtc                         │
│ 127                    │ VK_INTEL_shader_integer_functions2          │
│ 128                    │ VK_NV_fragment_shader_barycentric           │
╰────────────────────────┴─────────────────────────────────────────────╯
```

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
- `glfw`

Optional:

- `HOMEBREW_NO_AUTO_UPDATE=1`

## Run on macOS

```sh
cd /Users/$USER/code/trimui-vulkan/demos-go/vulkaninfo/vulkaninfo_desktop
export DYLD_LIBRARY_PATH="/opt/homebrew/lib:$DYLD_LIBRARY_PATH"
CGO_LDFLAGS="-L/opt/homebrew/lib" go run .
```

## TrimUI (container)

Currently we build only the SDL2 variant in the TrimUI container. GLFW requires X11/Wayland and is not supported on TrimUI.

Build inside the container (from the `vulkaninfo_sdl2` folder):

```sh
go build .
```

Runtime on TrimUI (uses system SDL2):

```sh
export LD_LIBRARY_PATH=/usr/trimui/lib:$LD_LIBRARY_PATH
./vulkaninfo_sdl2
```
