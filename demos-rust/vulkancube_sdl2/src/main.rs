use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use image::GenericImageView;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents, SubpassEndInfo,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags};
use vulkano::format::Format;
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::{CompareOp, DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{CullMode, FrontFace, RasterizationState};
use vulkano::pipeline::graphics::vertex_input::VertexInputState;
use vulkano::pipeline::graphics::viewport::{Scissor, Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{
    self, Surface, SurfaceApi, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::GpuFuture;
use vulkano::{sync, VulkanLibrary, VulkanObject};
use ash::vk::{Handle, SurfaceKHR};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const VERTEX_DATA: [f32; 36 * 3] = [
    -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
    -1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, 1.0, -1.0,
    1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0,
    -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0, -1.0,
    1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0,
    1.0, 1.0, 1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0,
    1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
    1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0,
    1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0,
];

const UV_DATA: [f32; 36 * 2] = [
    0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0,
    1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0,
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0,
];

/// UBO layout matching the GLSL shader (std140).
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct UniformBufferObject {
    mvp: [[f32; 4]; 4],
    position: [[f32; 4]; 36],
    attr: [[f32; 4]; 36],
}

fn main() -> Result<()> {
    // --- SDL2 init ---
    let sdl = sdl2::init().map_err(|e| anyhow!(e)).context("sdl init")?;
    let video = sdl.video().map_err(|e| anyhow!(e)).context("sdl video")?;

    let window = video
        .window("VulkanCube (Vulkano + SDL2)", WIDTH, HEIGHT)
        .vulkan()
        .build()
        .map_err(|e| anyhow!(e))?;

    // --- Vulkan Instance ---
    let library = VulkanLibrary::new().context("load vulkan library")?;

    let sdl_extensions = window
        .vulkan_instance_extensions()
        .map_err(|e| anyhow!(e))?;
    let mut instance_extensions = InstanceExtensions::from_iter(sdl_extensions.iter().copied());

    let supported_extensions = library.supported_extensions();
    let mut flags = InstanceCreateFlags::empty();
    if supported_extensions.khr_portability_enumeration {
        instance_extensions.khr_portability_enumeration = true;
        flags |= InstanceCreateFlags::ENUMERATE_PORTABILITY;
    }

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags,
            enabled_extensions: instance_extensions,
            ..Default::default()
        },
    )
    .context("create vulkan instance")?;

    // --- Surface from SDL2 ---
    // SDL2 0.36 uses raw-window-handle 0.5 which is incompatible with vulkano 0.35's
    // raw-window-handle 0.6 requirement for from_window_ref(). We create the surface
    // manually via SDL2's Vulkan integration.
    let raw_surface = window
        .vulkan_create_surface(instance.handle().as_raw() as usize)
        .map_err(|e| anyhow!(e))?;
    let surface_api = surface_api_from_extensions(&sdl_extensions);
    let surface = Arc::new(unsafe {
        Surface::from_handle(
            instance.clone(),
            SurfaceKHR::from_raw(raw_surface as u64),
            surface_api,
            None,
        )
    });

    // --- Physical device selection ---
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .context("enumerate physical devices")?
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.intersects(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .ok_or_else(|| anyhow!("no suitable physical device"))?;

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    // --- Logical device + queues ---
    let (device, mut queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions,
            ..Default::default()
        },
    )
    .context("create logical device")?;

    let queue = queues.next().ok_or_else(|| anyhow!("no queue"))?;

    // --- Allocators ---
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let command_buffer_allocator: Arc<StandardCommandBufferAllocator> =
        Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
    let descriptor_set_allocator: Arc<StandardDescriptorSetAllocator> =
        Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

    // --- Swapchain ---
    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .context("surface capabilities")?;

    let image_format = physical_device
        .surface_formats(&surface, Default::default())
        .context("surface formats")?[0]
        .0;

    let composite_alpha = caps
        .supported_composite_alpha
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no composite alpha"))?;

    let (w, h) = window.drawable_size();
    let image_extent = [w.max(1), h.max(1)];

    let (mut swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count.max(2),
            image_format,
            image_extent,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha,
            ..Default::default()
        },
    )
    .context("create swapchain")?;

    let swapchain_image_views: Vec<Arc<ImageView>> = images
        .iter()
        .map(|image| ImageView::new_default(image.clone()).map_err(|e| anyhow!(e)))
        .collect::<Result<Vec<_>>>()?;

    // --- Render pass ---
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: image_format,
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth_stencil: {
                format: Format::D16_UNORM,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {depth_stencil},
        },
    )
    .context("create render pass")?;

    // --- Shaders (pre-compiled SPIR-V from the ash version) ---
    #[allow(deprecated)]
    let vs_module = unsafe {
        ShaderModule::from_bytes(device.clone(), include_bytes!("../shaders/cube.vert.spv"))
    }
    .context("load vertex shader")?;

    #[allow(deprecated)]
    let fs_module = unsafe {
        ShaderModule::from_bytes(device.clone(), include_bytes!("../shaders/cube.frag.spv"))
    }
    .context("load fragment shader")?;

    let vs_entry = vs_module
        .entry_point("main")
        .ok_or_else(|| anyhow!("no main in vs"))?;
    let fs_entry = fs_module
        .entry_point("main")
        .ok_or_else(|| anyhow!("no main in fs"))?;

    let stages = [
        PipelineShaderStageCreateInfo::new(vs_entry),
        PipelineShaderStageCreateInfo::new(fs_entry),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .context("pipeline layout create info")?,
    )
    .context("create pipeline layout")?;

    let subpass =
        Subpass::from(render_pass.clone(), 0).ok_or_else(|| anyhow!("no subpass 0"))?;

    let pipeline = GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(VertexInputState::default()),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::Back,
                front_face: FrontFace::CounterClockwise,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::LessOrEqual,
                }),
                ..Default::default()
            }),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                1,
                ColorBlendAttachmentState::default(),
            )),
            dynamic_state: [DynamicState::Viewport, DynamicState::Scissor]
                .into_iter()
                .collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .context("create graphics pipeline")?;

    // --- Depth buffer ---
    let depth_image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::D16_UNORM,
            extent: [image_extent[0], image_extent[1], 1],
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .context("create depth image")?;

    let depth_view = ImageView::new_default(depth_image).context("create depth view")?;

    // --- Texture ---
    let png_bytes = include_bytes!("../textures/gopher.png");
    let img = image::load_from_memory(png_bytes).context("load texture")?;
    let rgba = img.to_rgba8();
    let (tex_width, tex_height) = img.dimensions();
    let pixels: Vec<u8> = rgba.into_raw();

    let texture_upload_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        pixels,
    )
    .context("create texture staging buffer")?;

    let texture_image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [tex_width, tex_height, 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .context("create texture image")?;

    let texture_view =
        ImageView::new_default(texture_image.clone()).context("create texture view")?;

    // Upload texture
    {
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .context("begin upload cmd")?;

        builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                texture_upload_buffer,
                texture_image,
            ))
            .context("copy buffer to image")?;

        let cmd = builder.build().context("build upload cmd")?;
        let future = sync::now(device.clone())
            .then_execute(queue.clone(), cmd)
            .context("execute upload")?
            .then_signal_fence_and_flush()
            .context("flush upload")?;
        future.wait(None).context("wait upload")?;
    }

    let sampler = Sampler::new(
        device.clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            address_mode: [SamplerAddressMode::ClampToEdge; 3],
            ..Default::default()
        },
    )
    .context("create sampler")?;

    // --- Uniform buffers (one per swapchain image) ---
    let uniform_buffers: Vec<_> = (0..images.len())
        .map(|_| {
            Buffer::new_sized::<UniformBufferObject>(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
            )
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("create uniform buffers")?;

    // --- Descriptor sets (one per swapchain image) ---
    let ds_layout = pipeline
        .layout()
        .set_layouts()
        .first()
        .ok_or_else(|| anyhow!("no descriptor set layout"))?;

    let descriptor_sets: Vec<Arc<DescriptorSet>> = uniform_buffers
        .iter()
        .map(|ubo| {
            DescriptorSet::new(
                descriptor_set_allocator.clone(),
                ds_layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, ubo.clone()),
                    WriteDescriptorSet::image_view_sampler(
                        1,
                        texture_view.clone(),
                        sampler.clone(),
                    ),
                ],
                [],
            )
            .map_err(|e| anyhow!(e))
        })
        .collect::<Result<Vec<_>>>()?;

    // --- Framebuffers ---
    let framebuffers: Vec<Arc<Framebuffer>> = swapchain_image_views
        .iter()
        .map(|view| {
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view.clone(), depth_view.clone()],
                    ..Default::default()
                },
            )
            .map_err(|e| anyhow!(e))
        })
        .collect::<Result<Vec<_>>>()?;

    // --- Main loop ---
    let mut event_pump = sdl
        .event_pump()
        .map_err(|e| anyhow!(e))
        .context("event pump")?;
    let mut spin_angle = 1.0_f32;

    let eye = Vec3::new(0.0, 3.0, 5.0);
    let center = Vec3::ZERO;
    let up = Vec3::Y;
    let mut model = Mat4::IDENTITY;

    let mut last_frame = Instant::now();
    let frame_time = Duration::from_micros(1_000_000 / 60);

    let mut previous_frame_end: Option<Box<dyn GpuFuture>> =
        Some(Box::new(sync::now(device.clone())));

    let mut recreate_swapchain = false;

    'main: loop {
        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Escape => break 'main,
                    Keycode::A => {
                        spin_angle = (spin_angle - 0.5).max(0.0);
                        println!("spin angle: {:.2}", spin_angle);
                    }
                    Keycode::B => {
                        spin_angle = (spin_angle + 0.5).min(20.0);
                        println!("spin angle: {:.2}", spin_angle);
                    }
                    _ => {}
                },
                Event::ControllerButtonDown {
                    which, button, ..
                } => {
                    println!(
                        "SDL controller button: which={} button={:?}",
                        which, button
                    );
                    if which == 0 && button as i32 == 5 {
                        break 'main;
                    }
                    if button == sdl2::controller::Button::A {
                        spin_angle = (spin_angle - 0.5).max(0.0);
                    }
                    if button == sdl2::controller::Button::B {
                        spin_angle = (spin_angle + 0.5).min(20.0);
                    }
                }
                Event::JoyButtonDown {
                    which, button_idx, ..
                } => {
                    println!("SDL joy button: which={} button={}", which, button_idx);
                }
                Event::JoyDeviceAdded { which, .. } => {
                    println!("SDL joy device added: which={}", which);
                }
                _ => {}
            }
        }

        if last_frame.elapsed() < frame_time {
            continue;
        }

        if let Some(ref mut fut) = previous_frame_end {
            fut.cleanup_finished();
        }

        if recreate_swapchain {
            let (w, h) = window.drawable_size();
            let new_extent = [w.max(1), h.max(1)];
            match swapchain.recreate(SwapchainCreateInfo {
                image_extent: new_extent,
                ..swapchain.create_info()
            }) {
                Ok((new_swapchain, _new_images)) => {
                    swapchain = new_swapchain;
                    recreate_swapchain = false;
                }
                Err(e) => {
                    eprintln!("Failed to recreate swapchain: {:?}", e);
                    continue;
                }
            }
        }

        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(vulkano::Validated::Error(vulkano::VulkanError::OutOfDate)) => {
                    recreate_swapchain = true;
                    continue;
                }
                Err(e) => {
                    eprintln!("acquire_next_image error: {:?}", e);
                    continue;
                }
            };

        if suboptimal {
            recreate_swapchain = true;
        }

        let image_index = image_index as usize;

        // Update uniform buffer
        let aspect =
            swapchain.image_extent()[0] as f32 / swapchain.image_extent()[1] as f32;
        let mut proj = Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        proj.y_axis.y *= -1.0;
        let view = Mat4::look_at_rh(eye, center, up);
        model = model * Mat4::from_rotation_y(spin_angle.to_radians());
        let mvp = proj * view * model;

        let mut ubo = UniformBufferObject::zeroed();
        ubo.mvp = mvp.to_cols_array_2d();
        for i in 0..36 {
            ubo.position[i] = [
                VERTEX_DATA[i * 3],
                VERTEX_DATA[i * 3 + 1],
                VERTEX_DATA[i * 3 + 2],
                1.0,
            ];
            ubo.attr[i] = [UV_DATA[i * 2], UV_DATA[i * 2 + 1], 0.0, 0.0];
        }

        {
            let mut write_lock = uniform_buffers[image_index]
                .write()
                .context("map uniform buffer")?;
            *write_lock = ubo;
        }

        // Build command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .context("begin draw cmd")?;

        let extent = swapchain.image_extent();
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent[0] as f32, extent[1] as f32],
            depth_range: 0.0..=1.0,
        };

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.2, 0.2, 0.2, 0.2].into()),
                        Some(1.0f32.into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(framebuffers[image_index].clone())
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .context("begin render pass")?
            .set_viewport(0, [viewport].into_iter().collect())
            .context("set viewport")?
            .set_scissor(
                0,
                [Scissor {
                    offset: [0, 0],
                    extent: [extent[0], extent[1]],
                }]
                .into_iter()
                .collect(),
            )
            .context("set scissor")?
            .bind_pipeline_graphics(pipeline.clone())
            .context("bind pipeline")?
            .bind_descriptor_sets(
                vulkano::pipeline::PipelineBindPoint::Graphics,
                pipeline.layout().clone(),
                0,
                descriptor_sets[image_index].clone(),
            )
            .context("bind descriptor sets")?;
        unsafe { builder.draw(36, 1, 0, 0) }.context("draw")?;
        builder
            .end_render_pass(SubpassEndInfo::default())
            .context("end render pass")?;

        let command_buffer = builder.build().context("build draw cmd")?;

        let future = previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .context("execute draw")?
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    swapchain.clone(),
                    image_index as u32,
                ),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame_end = Some(Box::new(future));
            }
            Err(vulkano::Validated::Error(vulkano::VulkanError::OutOfDate)) => {
                recreate_swapchain = true;
                previous_frame_end = Some(Box::new(sync::now(device.clone())));
            }
            Err(e) => {
                eprintln!("Failed to flush future: {:?}", e);
                previous_frame_end = Some(Box::new(sync::now(device.clone())));
            }
        }

        last_frame = Instant::now();
    }

    if let Some(ref mut fut) = previous_frame_end {
        fut.cleanup_finished();
    }
    unsafe { device.wait_idle() }.ok();

    Ok(())
}

fn surface_api_from_extensions(extensions: &[&str]) -> SurfaceApi {
    if extensions.iter().any(|e| *e == "VK_EXT_metal_surface") {
        return SurfaceApi::Metal;
    }
    if extensions.iter().any(|e| *e == "VK_MVK_macos_surface") {
        return SurfaceApi::MacOs;
    }
    if extensions.iter().any(|e| *e == "VK_KHR_wayland_surface") {
        return SurfaceApi::Wayland;
    }
    if extensions.iter().any(|e| *e == "VK_KHR_xcb_surface") {
        return SurfaceApi::Xcb;
    }
    if extensions.iter().any(|e| *e == "VK_KHR_xlib_surface") {
        return SurfaceApi::Xlib;
    }
    if extensions.iter().any(|e| *e == "VK_KHR_win32_surface") {
        return SurfaceApi::Win32;
    }
    if extensions.iter().any(|e| *e == "VK_KHR_android_surface") {
        return SurfaceApi::Android;
    }
    if extensions.iter().any(|e| *e == "VK_KHR_display") {
        return SurfaceApi::DisplayPlane;
    }
    SurfaceApi::Headless
}
