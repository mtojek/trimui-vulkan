use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use glam::{Mat4, Vec3};
use image::GenericImageView;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use foldhash::{HashSet, HashSetExt};
use smallvec::smallvec;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
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
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo};
use vulkano::image::sampler::{Filter, Sampler, SamplerCreateInfo, SamplerMipmapMode};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::subpass::PipelineSubpassType;
use vulkano::pipeline::graphics::viewport::{Scissor, Viewport, ViewportState};
use vulkano::pipeline::graphics::{color_blend::ColorBlendState, depth_stencil::DepthStencilState};
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::shader::{spirv::bytes_to_words, ShaderModule, ShaderModuleCreateInfo, ShaderStages};
use vulkano::swapchain::{
    acquire_next_image, PresentMode, Surface, SurfaceApi, Swapchain, SwapchainCreateInfo,
    SwapchainPresentInfo,
};
use vulkano::sync::{self, GpuFuture};
use vulkano::{Validated, VulkanError, VulkanLibrary, VulkanObject};
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

#[repr(C, align(16))]
#[derive(BufferContents, Clone, Copy)]
struct UniformBufferObject {
    mvp: [[f32; 4]; 4],
    position: [[f32; 4]; 36],
    attr: [[f32; 4]; 36],
}

impl Default for UniformBufferObject {
    fn default() -> Self {
        Self {
            mvp: [[0.0; 4]; 4],
            position: [[0.0; 4]; 36],
            attr: [[0.0; 4]; 36],
        }
    }
}

fn main() -> Result<()> {
    let sdl = sdl2::init().map_err(|e| anyhow!(e)).context("sdl init")?;
    let video = sdl.video().map_err(|e| anyhow!(e)).context("sdl video")?;

    let window = video
        .window("VulkanCube (SDL2)", WIDTH, HEIGHT)
        .vulkan()
        .build()
        .map_err(|e| anyhow!(e))?;

    let library = VulkanLibrary::new().context("load Vulkan library")?;

    let sdl_extensions = window
        .vulkan_instance_extensions()
        .map_err(|e| anyhow!(e))
        .context("surface extensions")?;
    let mut extensions = InstanceExtensions::from_iter(sdl_extensions.iter().copied());

    let supported_extensions = library.supported_extensions();
    if supported_extensions.khr_portability_enumeration {
        extensions.khr_portability_enumeration = true;
    }
    let mut instance_flags = InstanceCreateFlags::empty();
    if extensions.khr_portability_enumeration {
        instance_flags |= InstanceCreateFlags::ENUMERATE_PORTABILITY;
    }

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: extensions,
            flags: instance_flags,
            ..Default::default()
        },
    )
    .context("create instance")?;

    let raw_surface = window
        .vulkan_create_surface(instance.handle().as_raw() as usize)
        .map_err(|e| anyhow!(e))
        .context("create surface")?;
    let surface_api = surface_api_from_extensions(&sdl_extensions);
    let surface = Arc::new(unsafe {
        Surface::from_handle(
            instance.clone(),
            SurfaceKHR::from_raw(raw_surface as u64),
            surface_api,
            None,
        )
    });

    let required_device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .context("enumerate physical devices")?
        .filter(|p| p.supported_extensions().contains(&required_device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(_i, q)| q.queue_flags.contains(vulkano::device::QueueFlags::GRAPHICS))
                .map(|i| (p, i as u32))
        })
        .find(|(p, i)| p.surface_support(*i, &surface).unwrap_or(false))
        .ok_or_else(|| anyhow!("no suitable device"))?;

    let mut device_extensions = required_device_extensions;
    if physical_device.supported_extensions().khr_portability_subset {
        device_extensions.khr_portability_subset = true;
    }

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .context("create device")?;

    let queue = queues.next().ok_or_else(|| anyhow!("missing queue"))?;

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        Default::default(),
    ));
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));

    let (mut swapchain, mut images) = create_swapchain(&window, device.clone(), surface.clone())?;
    let render_pass = create_render_pass(device.clone(), swapchain.image_format())?;
    let set_layout = create_descriptor_set_layout(device.clone())?;
    let pipeline_layout = PipelineLayout::new(
        device.clone(),
        PipelineLayoutCreateInfo {
            set_layouts: vec![set_layout.clone()],
            ..Default::default()
        },
    )
    .map_err(Validated::unwrap)
    .context("pipeline layout")?;
    let pipeline = create_pipeline(
        device.clone(),
        render_pass.clone(),
        pipeline_layout.clone(),
    )?;

    let mut depth_images = create_depth_images(&memory_allocator, &images);
    let mut framebuffers = create_framebuffers(&render_pass, &images, &depth_images)?;

    let (texture_view, sampler) =
        create_texture(&memory_allocator, queue.clone(), command_buffer_allocator.clone())?;

    let mut event_pump = sdl.event_pump().map_err(|e| anyhow!(e)).context("event pump")?;
    let mut spin_angle = 1.0_f32;
    let eye = Vec3::new(0.0, 3.0, 5.0);
    let center = Vec3::ZERO;
    let up = Vec3::Y;
    let mut model = Mat4::IDENTITY;

    let mut previous_frame_end: Option<Box<dyn GpuFuture>> = None;
    let frame_time = Duration::from_micros(1_000_000 / 60);
    let mut last_frame = Instant::now();

    'main: loop {
        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: Some(key), .. } => match key {
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
                Event::ControllerButtonDown { which, button, .. } => {
                    println!("SDL controller button: which={} button={:?}", which, button);
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
                Event::JoyButtonDown { which, button_idx, .. } => {
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
        last_frame = Instant::now();

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap) {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => continue,
                Err(err) => return Err(anyhow!(err)),
            };

        if suboptimal {
            recreate_swapchain(
                &window,
                &memory_allocator,
                &mut swapchain,
                &mut images,
                &mut depth_images,
                &mut framebuffers,
                &render_pass,
            )?;
            continue;
        }

        let aspect = swapchain.image_extent()[0] as f32 / swapchain.image_extent()[1] as f32;
        let mut proj = Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        proj.y_axis.y *= -1.0;
        let view = Mat4::look_at_rh(eye, center, up);
        model = model * Mat4::from_rotation_y(spin_angle.to_radians());

        let mut ubo = UniformBufferObject::default();
        ubo.mvp = (proj * view * model).to_cols_array_2d();
        for i in 0..36 {
            ubo.position[i] = [
                VERTEX_DATA[i * 3],
                VERTEX_DATA[i * 3 + 1],
                VERTEX_DATA[i * 3 + 2],
                1.0,
            ];
            ubo.attr[i] = [UV_DATA[i * 2], UV_DATA[i * 2 + 1], 0.0, 0.0];
        }

        let uniform_buffer = create_uniform_buffer(&memory_allocator, ubo)?;
        let descriptor_set = DescriptorSet::new(
            descriptor_set_allocator.clone(),
            set_layout.clone(),
            [
                WriteDescriptorSet::buffer(0, uniform_buffer),
                WriteDescriptorSet::image_view_sampler(1, texture_view.clone(), sampler.clone()),
            ],
            [],
        )
        .map_err(Validated::unwrap)
        .context("create descriptor set")?;

        let mut builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .context("command buffer")?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.2, 0.2, 0.2, 0.2].into()),
                        Some(1.0f32.into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(framebuffers[image_index as usize].clone())
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .context("begin render pass")?;

        builder.bind_pipeline_graphics(pipeline.clone())?;
        builder.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            pipeline_layout.clone(),
            0,
            descriptor_set,
        )?;
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [
                swapchain.image_extent()[0] as f32,
                swapchain.image_extent()[1] as f32,
            ],
            depth_range: 0.0..=1.0,
        };
        let scissor = Scissor {
            offset: [0, 0],
            extent: [swapchain.image_extent()[0], swapchain.image_extent()[1]],
        };
        builder.set_viewport(0, smallvec![viewport])?;
        builder.set_scissor(0, smallvec![scissor])?;
        unsafe {
            builder.draw(36, 1, 0, 0).context("draw")?;
        }
        builder
            .end_render_pass(SubpassEndInfo::default())
            .context("end render pass")?;

        let command_buffer = builder.build().context("build command buffer")?;

        let future = match previous_frame_end.take() {
            Some(f) => f.boxed(),
            None => sync::now(device.clone()).boxed(),
        };

        let future = future
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .context("execute")?
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                future.wait(None).context("wait frame")?;
                previous_frame_end = Some(sync::now(device.clone()).boxed());
            }
            Err(err) => match err.unwrap() {
                VulkanError::OutOfDate => {
                    previous_frame_end = Some(sync::now(device.clone()).boxed());
                    recreate_swapchain(
                        &window,
                        &memory_allocator,
                        &mut swapchain,
                        &mut images,
                        &mut depth_images,
                        &mut framebuffers,
                        &render_pass,
                    )?;
                }
                err => return Err(anyhow!(err)),
            },
        }
    }

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

fn create_swapchain(
    window: &sdl2::video::Window,
    device: Arc<Device>,
    surface: Arc<Surface>,
) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>)> {
    let caps = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .context("surface capabilities")?;
    let formats = device
        .physical_device()
        .surface_formats(&surface, Default::default())
        .context("surface formats")?;
    let (format, _color_space) = formats
        .iter()
        .cloned()
        .find(|(f, _)| *f == vulkano::format::Format::B8G8R8A8_UNORM)
        .unwrap_or(formats[0]);

    let (width, height) = window.drawable_size();
    let extent = [width.max(1), height.max(1)];

    let mut image_count = caps.min_image_count + 1;
    if let Some(max) = caps.max_image_count {
        if image_count > max {
            image_count = max;
        }
    }

    let (swapchain, images) = Swapchain::new(
        device,
        surface,
        SwapchainCreateInfo {
            min_image_count: image_count,
            image_format: format,
            image_extent: extent,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: caps.supported_composite_alpha.into_iter().next().unwrap(),
            present_mode: PresentMode::Fifo,
            ..Default::default()
        },
    )
    .context("create swapchain")?;

    Ok((swapchain, images))
}

fn recreate_swapchain(
    window: &sdl2::video::Window,
    memory_allocator: &Arc<StandardMemoryAllocator>,
    swapchain: &mut Arc<Swapchain>,
    images: &mut Vec<Arc<Image>>,
    depth_images: &mut Vec<Arc<ImageView>>,
    framebuffers: &mut Vec<Arc<Framebuffer>>,
    render_pass: &Arc<RenderPass>,
) -> Result<()> {
    let (width, height) = window.drawable_size();
    if width == 0 || height == 0 {
        return Ok(());
    }

    let (new_swapchain, new_images) = match swapchain
        .recreate(SwapchainCreateInfo {
            image_extent: [width, height],
            ..swapchain.create_info()
        })
        .map_err(Validated::unwrap)
    {
        Ok(r) => r,
        Err(VulkanError::OutOfDate) => return Ok(()),
        Err(err) => return Err(anyhow!(err)),
    };

    *swapchain = new_swapchain;
    *images = new_images;

    *depth_images = create_depth_images(memory_allocator, images);
    *framebuffers = create_framebuffers(render_pass, images, depth_images)?;

    Ok(())
}

fn create_render_pass(device: Arc<Device>, format: vulkano::format::Format) -> Result<Arc<RenderPass>> {
    let render_pass = vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                format: format,
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: vulkano::format::Format::D16_UNORM,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {depth}
        }
    )
    .context("render pass")?;

    Ok(render_pass)
}

fn create_descriptor_set_layout(device: Arc<Device>) -> Result<Arc<DescriptorSetLayout>> {
    let mut bindings = BTreeMap::new();
    bindings.insert(
        0,
        DescriptorSetLayoutBinding {
            stages: ShaderStages::VERTEX,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
        },
    );
    bindings.insert(
        1,
        DescriptorSetLayoutBinding {
            stages: ShaderStages::FRAGMENT,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::CombinedImageSampler)
        },
    );

    let layout = DescriptorSetLayout::new(
        device,
        DescriptorSetLayoutCreateInfo {
            bindings,
            ..Default::default()
        },
    )
    .map_err(Validated::unwrap)
    .context("descriptor set layout")?;

    Ok(layout)
}

fn create_pipeline(
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
    pipeline_layout: Arc<PipelineLayout>,
) -> Result<Arc<GraphicsPipeline>> {
    let vs_words = bytes_to_words(include_bytes!("../shaders/cube.vert.spv"))
        .map_err(|e| anyhow!(e))?;
    let fs_words = bytes_to_words(include_bytes!("../shaders/cube.frag.spv"))
        .map_err(|e| anyhow!(e))?;
    let vs = unsafe {
        ShaderModule::new(device.clone(), ShaderModuleCreateInfo::new(&vs_words))
            .context("vertex shader")?
    };
    let fs = unsafe {
        ShaderModule::new(device.clone(), ShaderModuleCreateInfo::new(&fs_words))
            .context("fragment shader")?
    };

    let stages = [
        PipelineShaderStageCreateInfo::new(vs.entry_point("main").context("vs entry")?),
        PipelineShaderStageCreateInfo::new(fs.entry_point("main").context("fs entry")?),
    ];

    let mut dynamic_state = HashSet::new();
    dynamic_state.insert(DynamicState::Viewport);
    dynamic_state.insert(DynamicState::Scissor);

    let viewport_state = ViewportState::viewport_fixed_scissor_irrelevant([Viewport {
        offset: [0.0, 0.0],
        extent: [1.0, 1.0],
        depth_range: 0.0..=1.0,
    }]);

    let mut create_info = vulkano::pipeline::graphics::GraphicsPipelineCreateInfo::layout(
        pipeline_layout,
    );
    create_info.stages = smallvec![stages[0].clone(), stages[1].clone()];
    create_info.vertex_input_state =
        Some(vulkano::pipeline::graphics::vertex_input::VertexInputState::new());
    create_info.input_assembly_state = Some(InputAssemblyState::default());
    create_info.viewport_state = Some(viewport_state);
    create_info.rasterization_state = Some(RasterizationState::default());
    create_info.multisample_state = Some(MultisampleState::default());
    create_info.color_blend_state = Some(ColorBlendState::new(1));
    create_info.depth_stencil_state = Some(DepthStencilState::simple_depth_test());
    let subpass = vulkano::render_pass::Subpass::from(render_pass.clone(), 0)
        .ok_or_else(|| anyhow!("missing subpass 0"))?;
    create_info.subpass = Some(PipelineSubpassType::BeginRenderPass(subpass));
    create_info.dynamic_state = dynamic_state;

    let pipeline = GraphicsPipeline::new(device, None, create_info)
        .map_err(Validated::unwrap)
        .context("build pipeline")?;

    Ok(pipeline)
}

fn create_depth_images(
    memory_allocator: &Arc<StandardMemoryAllocator>,
    images: &[Arc<Image>],
) -> Vec<Arc<ImageView>> {
    images
        .iter()
        .map(|image| {
            let extent = image.extent();
            let depth_image = Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: vulkano::format::Format::D16_UNORM,
                    extent,
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            )
            .expect("depth image");
            ImageView::new_default(depth_image).expect("depth view")
        })
        .collect()
}

fn create_framebuffers(
    render_pass: &Arc<RenderPass>,
    images: &[Arc<Image>],
    depth_images: &[Arc<ImageView>],
) -> Result<Vec<Arc<Framebuffer>>> {
    let framebuffers = images
        .iter()
        .zip(depth_images)
        .map(|(image, depth)| {
            let color_view = ImageView::new_default(image.clone()).context("color view")?;
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![color_view, depth.clone()],
                    ..Default::default()
                },
            )
            .context("framebuffer")
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(framebuffers)
}

fn create_uniform_buffer(
    memory_allocator: &Arc<StandardMemoryAllocator>,
    ubo: UniformBufferObject,
) -> Result<Subbuffer<UniformBufferObject>> {
    let buffer = Buffer::from_data(
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
        ubo,
    )
    .context("uniform buffer")?;

    Ok(buffer)
}

fn create_texture(
    memory_allocator: &Arc<StandardMemoryAllocator>,
    queue: Arc<vulkano::device::Queue>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
) -> Result<(Arc<ImageView>, Arc<Sampler>)> {
    let png_bytes = include_bytes!("../textures/gopher.png");
    let img = image::load_from_memory(png_bytes).context("load texture")?;
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    let staging = Buffer::from_iter(
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
        rgba.into_raw(),
    )
    .context("staging buffer")?;

    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: vulkano::format::Format::R8G8B8A8_UNORM,
            extent: [width, height, 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )
    .context("texture image")?;

    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .context("texture command buffer")?;

    builder
        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(staging, image.clone()))
        .context("copy buffer to image")?;

    let command_buffer = builder.build().context("build texture command buffer")?;

    let future = sync::now(queue.device().clone())
        .then_execute(queue.clone(), command_buffer)
        .context("submit texture")?
        .then_signal_fence_and_flush();

    match future {
        Ok(future) => {
            future.wait(None).context("wait texture")?;
        }
        Err(err) => return Err(anyhow!(err.unwrap())),
    }

    let view = ImageView::new_default(image).context("texture view")?;
    let sampler = Sampler::new(
        queue.device().clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            mipmap_mode: SamplerMipmapMode::Nearest,
            ..Default::default()
        },
    )
    .context("sampler")?;

    Ok((view, sampler))
}
