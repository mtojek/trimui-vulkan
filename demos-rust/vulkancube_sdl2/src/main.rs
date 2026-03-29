use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use image::GenericImageView;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassContents, SubpassEndInfo,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::layout::{
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType,
};
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
use vulkano::pipeline::graphics::depth_stencil::{DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{CullMode, FrontFace, RasterizationState};
use vulkano::pipeline::graphics::viewport::{Scissor, Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, PipelineBindPoint, PipelineLayout,
    PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, Subpass};
use vulkano::shader::spirv::bytes_to_words;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo, ShaderStages};
use vulkano::swapchain::{
    self, PresentMode, Surface, SurfaceApi, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::GpuFuture;
use vulkano::{sync, Handle, Validated, VulkanError, VulkanLibrary, VulkanObject};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn open_input_devices(
    gc: &sdl2::GameControllerSubsystem,
    js: &sdl2::JoystickSubsystem,
    controllers: &mut Vec<sdl2::controller::GameController>,
    joysticks: &mut Vec<sdl2::joystick::Joystick>,
) {
    controllers.clear();
    joysticks.clear();

    let num = js.num_joysticks().unwrap_or(0);
    for i in 0..num {
        if gc.is_game_controller(i) {
            if let Ok(c) = gc.open(i) {
                let name = gc
                    .name_for_index(i)
                    .unwrap_or_else(|_| "Unknown".to_string());
                println!("SDL controller opened: idx={} name={}", i, name);
                controllers.push(c);
            }
            continue;
        }
        if let Ok(j) = js.open(i) {
            let name = js
                .name_for_index(i)
                .unwrap_or_else(|_| "Unknown".to_string());
            println!("SDL joystick opened: idx={} name={}", i, name);
            joysticks.push(j);
        }
    }
}

#[rustfmt::skip]
const VERTEX_DATA: [f32; 36 * 3] = [
    -1.0, -1.0, -1.0, -1.0, -1.0,  1.0, -1.0,  1.0,  1.0, -1.0,  1.0,  1.0,
    -1.0,  1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0,  1.0,  1.0, -1.0,
     1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0,  1.0, -1.0,  1.0,  1.0, -1.0,
    -1.0, -1.0, -1.0,  1.0, -1.0, -1.0,  1.0, -1.0,  1.0, -1.0, -1.0, -1.0,
     1.0, -1.0,  1.0, -1.0, -1.0,  1.0, -1.0,  1.0, -1.0, -1.0,  1.0,  1.0,
     1.0,  1.0,  1.0, -1.0,  1.0, -1.0,  1.0,  1.0,  1.0,  1.0,  1.0, -1.0,
     1.0,  1.0, -1.0,  1.0,  1.0,  1.0,  1.0, -1.0,  1.0,  1.0, -1.0,  1.0,
     1.0, -1.0, -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0, -1.0,  1.0,
     1.0,  1.0,  1.0, -1.0, -1.0,  1.0,  1.0, -1.0,  1.0,  1.0,  1.0,  1.0,
];

#[rustfmt::skip]
const UV_DATA: [f32; 36 * 2] = [
    0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0,
    1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0,
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0,
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0,
];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UniformBufferObject {
    mvp: [[f32; 4]; 4],
    position: [[f32; 4]; 36],
    attr: [[f32; 4]; 36],
}

fn build_ubo(model: Mat4, view: Mat4, proj: Mat4) -> UniformBufferObject {
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
    ubo
}

fn load_shader(device: Arc<Device>, spv_bytes: &[u8]) -> Result<Arc<ShaderModule>> {
    let words = bytes_to_words(spv_bytes).map_err(|e| anyhow!("bad SPIR-V: {}", e))?;
    let module = unsafe {
        ShaderModule::new(device, ShaderModuleCreateInfo::new(&words))
    }
    .map_err(|e| anyhow!("shader module: {}", e))?;
    Ok(module)
}

fn main() -> Result<()> {
    // --- SDL2 init ---
    let sdl = sdl2::init().map_err(|e| anyhow!(e))?;
    let game_controller = sdl.game_controller().map_err(|e| anyhow!(e))?;
    let joystick = sdl.joystick().map_err(|e| anyhow!(e))?;
    let video = sdl.video().map_err(|e| anyhow!(e))?;
    let window = video
        .window("VulkanCube (SDL2 + Vulkano)", WIDTH, HEIGHT)
        .vulkan()
        .build()
        .map_err(|e| anyhow!(e))?;

    // --- Vulkan library + instance ---
    let library = VulkanLibrary::new().context("load Vulkan library")?;

    let sdl_ext_names = window
        .vulkan_instance_extensions()
        .map_err(|e| anyhow!(e))?;

    let mut instance_extensions = InstanceExtensions::empty();
    for name in &sdl_ext_names {
        match *name {
            "VK_KHR_surface" => instance_extensions.khr_surface = true,
            "VK_KHR_xlib_surface" => instance_extensions.khr_xlib_surface = true,
            "VK_KHR_xcb_surface" => instance_extensions.khr_xcb_surface = true,
            "VK_KHR_wayland_surface" => instance_extensions.khr_wayland_surface = true,
            "VK_KHR_android_surface" => instance_extensions.khr_android_surface = true,
            "VK_KHR_win32_surface" => instance_extensions.khr_win32_surface = true,
            "VK_EXT_metal_surface" => instance_extensions.ext_metal_surface = true,
            _ => {}
        }
    }

    // Some SDL2 Vulkan backends (e.g., PVR/Display) require VK_KHR_display
    // even if it isn't reported in SDL's extension list.
    if library.supported_extensions().khr_display {
        instance_extensions.khr_display = true;
    }

    // MoltenVK / portability enumeration support.
    let mut create_flags = InstanceCreateFlags::empty();
    if library
        .supported_extensions()
        .khr_portability_enumeration
    {
        instance_extensions.khr_portability_enumeration = true;
        create_flags |= InstanceCreateFlags::ENUMERATE_PORTABILITY;
    }

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: instance_extensions,
            flags: create_flags,
            ..Default::default()
        },
    )
    .context("create instance")?;

    // --- Surface via SDL2 ---
    // SDL2 creates the VkSurfaceKHR for us; we wrap it in vulkano's
    // Surface type using the raw handle.
    let surface = {
        let raw_instance = instance.handle().as_raw() as usize;
        let raw_surface = window
            .vulkan_create_surface(raw_instance)
            .map_err(|e| anyhow!(e))?;
        let vk_surface = ash::vk::SurfaceKHR::from_raw(raw_surface as u64);
        Arc::new(unsafe {
            Surface::from_handle(
                Arc::clone(&instance),
                vk_surface,
                SurfaceApi::Wayland, // Informational — actual API chosen
                // by SDL2 at runtime.
                None,
            )
        })
    };

    // --- Physical device ---
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
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .ok_or_else(|| anyhow!("no suitable physical device"))?;

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    // --- Logical device + queue ---
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions,
            ..Default::default()
        },
    )
    .context("create device")?;
    let queue = queues.next().ok_or_else(|| anyhow!("no queue"))?;

    // --- Allocators ---
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let cb_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));
    let ds_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        Default::default(),
    ));

    // --- Swapchain ---
    let (swapchain, swapchain_images) = {
        let surface_caps = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .context("surface capabilities")?;

        let image_format = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .context("surface formats")?
            .into_iter()
            .find(|(f, _)| *f == Format::B8G8R8A8_UNORM)
            .map(|(f, _)| f)
            .unwrap_or(Format::B8G8R8A8_SRGB);

        let (w, h) = window.drawable_size();
        let image_extent = surface_caps
            .current_extent
            .unwrap_or([w.max(1), h.max(1)]);

        Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_caps.min_image_count.max(2),
                image_format,
                image_extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            },
        )
        .context("create swapchain")?
    };

    let swapchain_extent = swapchain.image_extent();

    // --- Render pass ---
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: Format::D16_UNORM,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {depth},
        },
    )
    .context("create render pass")?;

    // --- Shaders ---
    let vs_module = load_shader(device.clone(), include_bytes!("../shaders/cube.vert.spv"))?;
    let fs_module = load_shader(device.clone(), include_bytes!("../shaders/cube.frag.spv"))?;
    let vs_entry = vs_module
        .entry_point("main")
        .ok_or_else(|| anyhow!("no 'main' entry in vertex shader"))?;
    let fs_entry = fs_module
        .entry_point("main")
        .ok_or_else(|| anyhow!("no 'main' entry in fragment shader"))?;

    // --- Pipeline ---
    let stages = [
        PipelineShaderStageCreateInfo::new(vs_entry),
        PipelineShaderStageCreateInfo::new(fs_entry),
    ];

    let mut set_bindings = BTreeMap::new();
    set_bindings.insert(
        0,
        DescriptorSetLayoutBinding {
            stages: ShaderStages::VERTEX,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
        },
    );
    set_bindings.insert(
        1,
        DescriptorSetLayoutBinding {
            stages: ShaderStages::FRAGMENT,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::CombinedImageSampler)
        },
    );

    let ds_layout = DescriptorSetLayout::new(
        device.clone(),
        DescriptorSetLayoutCreateInfo {
            bindings: set_bindings,
            ..Default::default()
        },
    )
    .context("descriptor set layout")?;

    // Work around overly-strict update-after-bind limit validation on PVR.
    let layout = unsafe {
        PipelineLayout::new_unchecked(
            device.clone(),
            PipelineLayoutCreateInfo {
                set_layouts: vec![ds_layout.clone()],
                ..Default::default()
            },
        )
    }
    .context("pipeline layout")?;

    let subpass = Subpass::from(render_pass.clone(), 0)
        .ok_or_else(|| anyhow!("no subpass 0"))?;

    let pipeline = GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(Default::default()),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::Back,
                front_face: FrontFace::CounterClockwise,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
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
            ..GraphicsPipelineCreateInfo::layout(layout.clone())
        },
    )
    .context("create graphics pipeline")?;

    // --- Depth image ---
    let depth_image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::D16_UNORM,
            extent: [swapchain_extent[0], swapchain_extent[1], 1],
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .context("depth image")?;
    let depth_view = ImageView::new_default(depth_image).context("depth image view")?;

    // --- Texture ---
    let (texture_view, texture_future) = {
        let png_bytes = include_bytes!("../textures/gopher.png");
        let img = image::load_from_memory(png_bytes).context("load texture")?;
        let rgba = img.to_rgba8();
        let (width, height) = img.dimensions();
        let pixels: Vec<u8> = rgba.into_raw();

        let staging_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                    | MemoryTypeFilter::PREFER_HOST,
                ..Default::default()
            },
            pixels,
        )
        .context("texture staging buffer")?;

        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [width, height, 1],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .context("texture image")?;

        let mut builder = AutoCommandBufferBuilder::primary(
            cb_allocator.clone(),
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )
        .context("texture upload cmd")?;

        builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                staging_buffer,
                image.clone(),
            ))
            .context("copy buffer to image")?;

        let cb = builder.build().context("build texture upload cmd")?;
        let future = sync::now(device.clone())
            .then_execute(queue.clone(), cb)
            .context("execute texture upload")?
            .then_signal_fence_and_flush()
            .context("flush texture upload")?;

        let view = ImageView::new_default(image).context("texture image view")?;
        (view, future)
    };

    let sampler = Sampler::new(
        device.clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            address_mode: [SamplerAddressMode::ClampToEdge; 3],
            ..Default::default()
        },
    )
    .context("sampler")?;

    // --- Uniform buffers (one per swapchain image) ---
    let uniform_buffers: Vec<Subbuffer<UniformBufferObject>> = swapchain_images
        .iter()
        .map(|_| {
            Buffer::new_sized(
                memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                        | MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
            )
            .expect("uniform buffer")
        })
        .collect();

    // --- Descriptor sets (one per swapchain image) ---
    let descriptor_sets: Vec<Arc<DescriptorSet>> = uniform_buffers
        .iter()
        .map(|ubo| {
            DescriptorSet::new(
                ds_allocator.clone(),
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
            .expect("descriptor set")
        })
        .collect();

    // --- Framebuffers ---
    let framebuffers: Vec<Arc<Framebuffer>> = swapchain_images
        .iter()
        .map(|image| {
            let color_view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![color_view, depth_view.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect();

    // --- Event loop state ---
    let mut event_pump = sdl.event_pump().map_err(|e| anyhow!(e))?;
    let mut controllers: Vec<sdl2::controller::GameController> = Vec::new();
    let mut joysticks: Vec<sdl2::joystick::Joystick> = Vec::new();
    open_input_devices(&game_controller, &joystick, &mut controllers, &mut joysticks);
    let mut spin_angle = 1.0_f32;
    let eye = Vec3::new(0.0, 3.0, 5.0);
    let center = Vec3::ZERO;
    let up = Vec3::Y;
    let mut model = Mat4::IDENTITY;
    let frame_time = Duration::from_micros(1_000_000 / 60);
    let mut next_frame = Instant::now();

    // Wait for texture upload to finish before entering the render loop.
    texture_future.wait(None).context("wait texture upload")?;
    let mut previous_frame_end: Option<Box<dyn GpuFuture>> =
        Some(Box::new(sync::now(device.clone())));

    'main: loop {
        // --- Poll SDL2 events ---
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
                        println!("spin angle: {:.2}", spin_angle);
                    }
                    if button == sdl2::controller::Button::B {
                        spin_angle = (spin_angle + 0.5).min(20.0);
                        println!("spin angle: {:.2}", spin_angle);
                    }
                }
                Event::JoyButtonDown {
                    which, button_idx, ..
                } => {
                    println!("SDL joy button: which={} button={}", which, button_idx);
                }
                Event::JoyDeviceAdded { which, .. } => {
                    println!("SDL joy device added: which={}", which);
                    open_input_devices(&game_controller, &joystick, &mut controllers, &mut joysticks);
                }
                Event::ControllerDeviceAdded { which, .. } => {
                    println!("SDL controller device added: which={}", which);
                    open_input_devices(&game_controller, &joystick, &mut controllers, &mut joysticks);
                }
                _ => {}
            }
        }

        // --- Rate limiting ---
        let now = Instant::now();
        if now < next_frame {
            std::thread::sleep(next_frame - now);
        }
        next_frame += frame_time;

        // --- Update matrices ---
        let aspect = swapchain_extent[0] as f32 / swapchain_extent[1] as f32;
        let mut proj = Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        proj.y_axis.y *= -1.0;
        let view = Mat4::look_at_rh(eye, center, up);
        model = model * Mat4::from_rotation_y(spin_angle.to_radians());

        // --- Cleanup finished futures ---
        if let Some(ref mut fut) = previous_frame_end {
            fut.cleanup_finished();
        }

        // --- Acquire swapchain image ---
        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => continue,
                Err(e) => return Err(anyhow!("acquire: {:?}", e)),
            };
        if suboptimal {
            // Could recreate swapchain here; for demo simplicity we
            // just continue.
        }
        let image_index = image_index as usize;

        // --- Update uniform buffer ---
        {
            let ubo = build_ubo(model, view, proj);
            let mut content = uniform_buffers[image_index]
                .write()
                .context("map uniform buffer")?;
            *content = ubo;
        }

        // --- Record command buffer ---
        let mut builder = AutoCommandBufferBuilder::primary(
            cb_allocator.clone(),
            queue_family_index,
            CommandBufferUsage::OneTimeSubmit,
        )?;

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
            )?
            .bind_pipeline_graphics(pipeline.clone())?
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                layout.clone(),
                0,
                descriptor_sets[image_index].clone(),
            )?
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: [
                        swapchain_extent[0] as f32,
                        swapchain_extent[1] as f32,
                    ],
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )?
            .set_scissor(
                0,
                [Scissor {
                    offset: [0, 0],
                    extent: swapchain_extent,
                }]
                .into_iter()
                .collect(),
            )?;
        unsafe { builder.draw(36, 1, 0, 0)? };
        builder.end_render_pass(SubpassEndInfo::default())?;

        let command_buffer = builder.build().context("build command buffer")?;

        // --- Submit + present ---
        let future = previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .context("execute")?
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    swapchain.clone(),
                    image_index as u32,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                previous_frame_end = Some(Box::new(future));
            }
            Err(VulkanError::OutOfDate) => {
                previous_frame_end = Some(Box::new(sync::now(device.clone())));
            }
            Err(e) => {
                return Err(anyhow!("flush: {:?}", e));
            }
        }

        // next_frame already advanced
    }

    // Wait for GPU to finish before dropping.
    if let Some(mut fut) = previous_frame_end.take() {
        fut.cleanup_finished();
    }

    Ok(())
}
