use std::ffi::{CStr, CString};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk;
use ash::vk::Handle;
use ash::{Device, Entry, Instance};
use glam::{Mat4, Vec3};
use image::GenericImageView;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const MAX_FRAMES_IN_FLIGHT: usize = 2;

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
#[derive(Clone, Copy)]
struct UniformBufferObject {
    mvp: [[f32; 4]; 4],
    position: [[f32; 4]; 36],
    attr: [[f32; 4]; 36],
}

struct VulkanApp {
    instance: Instance,
    surface_loader: Surface,
    surface: vk::SurfaceKHR,

    device: Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,

    swapchain_loader: Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_imageviews: Vec<vk::ImageView>,
    swapchain_extent: vk::Extent2D,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    framebuffers: Vec<vk::Framebuffer>,

    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,

    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,

    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    _descriptor_sets: Vec<vk::DescriptorSet>,

    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
    current_frame: usize,
}

impl VulkanApp {
    fn new(window: &sdl2::video::Window) -> Result<Self> {
        let entry = unsafe { Entry::load() }.context("load Vulkan entry")?;

        let mut extension_names = window
            .vulkan_instance_extensions()
            .map_err(|e| anyhow!(e))?
            .into_iter()
            .map(CString::new)
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let available_exts = entry
            .enumerate_instance_extension_properties(None)
            .context("enumerate instance extensions")?;
        let has_portability = available_exts.iter().any(|e| unsafe {
            CStr::from_ptr(e.extension_name.as_ptr()) == CStr::from_bytes_with_nul(b"VK_KHR_portability_enumeration\0").unwrap()
        });
        let mut create_flags = vk::InstanceCreateFlags::empty();
        if has_portability {
            extension_names.push(CString::new("VK_KHR_portability_enumeration")?);
            create_flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        }

        let extension_ptrs: Vec<*const i8> = extension_names
            .iter()
            .map(|e| e.as_ptr())
            .collect();

        let app_name = CString::new("VulkanCube")?;
        let engine_name = CString::new("No Engine")?;
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&engine_name)
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let instance_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_ptrs)
            .flags(create_flags);

        let instance = unsafe { entry.create_instance(&instance_info, None) }
            .context("create instance")?;

        let surface_loader = Surface::new(&entry, &instance);
        let raw = window
            .vulkan_create_surface(instance.handle().as_raw() as usize)
            .map_err(|e| anyhow!(e))?;
        let surface = vk::SurfaceKHR::from_raw(raw as u64);

        let (physical_device, graphics_queue_index, present_queue_index) =
            pick_physical_device(&instance, &surface_loader, surface)?;

        let queue_priorities = [1.0_f32];
        let mut queue_infos = Vec::new();
        let mut unique_queues = vec![graphics_queue_index];
        if present_queue_index != graphics_queue_index {
            unique_queues.push(present_queue_index);
        }
        for &queue_family in &unique_queues {
            queue_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue_family)
                    .queue_priorities(&queue_priorities)
                    .build(),
            );
        }

        let device_exts = unsafe { instance.enumerate_device_extension_properties(physical_device) }?;
        let portability_subset = CStr::from_bytes_with_nul(b"VK_KHR_portability_subset\0").unwrap();
        let has_portability_subset = device_exts.iter().any(|e| unsafe {
            CStr::from_ptr(e.extension_name.as_ptr()) == portability_subset
        });

        let mut device_extension_names = vec![Swapchain::name().as_ptr()];
        if has_portability_subset {
            device_extension_names.push(portability_subset.as_ptr());
        }

        let device_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&device_extension_names);

        let device = unsafe { instance.create_device(physical_device, &device_info, None) }
            .context("create device")?;

        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_queue_index, 0) };

        let swapchain_loader = Swapchain::new(&instance, &device);
        let (swapchain, swapchain_images, swapchain_format, swapchain_extent) =
            create_swapchain(
                window,
                &surface_loader,
                &swapchain_loader,
                physical_device,
                surface,
                graphics_queue_index,
                present_queue_index,
            )?;

        let swapchain_imageviews = swapchain_images
            .iter()
            .map(|&image| create_image_view(&device, image, swapchain_format, vk::ImageAspectFlags::COLOR))
            .collect::<Result<Vec<_>>>()?;

        let render_pass = create_render_pass(&device, swapchain_format)?;
        let descriptor_set_layout = create_descriptor_set_layout(&device)?;
        let (pipeline_layout, pipeline) =
            create_graphics_pipeline(&device, render_pass, descriptor_set_layout)?;

        let command_pool = create_command_pool(&device, graphics_queue_index)?;

        let (depth_image, depth_image_memory, depth_image_view) = create_depth_resources(
            &instance,
            &device,
            physical_device,
            swapchain_extent,
            command_pool,
            graphics_queue,
        )?;

        let (texture_image, texture_image_memory) = create_texture_image(
            &instance,
            &device,
            physical_device,
            command_pool,
            graphics_queue,
        )?;
        let texture_image_view = create_image_view(&device, texture_image, vk::Format::R8G8B8A8_UNORM, vk::ImageAspectFlags::COLOR)?;
        let texture_sampler = create_texture_sampler(&device)?;

        let (uniform_buffers, uniform_buffers_memory) = create_uniform_buffers(
            &instance,
            &device,
            physical_device,
            swapchain_images.len(),
        )?;

        let descriptor_pool = create_descriptor_pool(&device, swapchain_images.len())?;
        let descriptor_sets = create_descriptor_sets(
            &device,
            descriptor_pool,
            descriptor_set_layout,
            &uniform_buffers,
            texture_image_view,
            texture_sampler,
        )?;

        let framebuffers = create_framebuffers(
            &device,
            render_pass,
            &swapchain_imageviews,
            depth_image_view,
            swapchain_extent,
        )?;

        let command_buffers = create_command_buffers(
            &device,
            command_pool,
            render_pass,
            &framebuffers,
            pipeline,
            pipeline_layout,
            &descriptor_sets,
            swapchain_extent,
        )?;

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
            create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT)?;
        let images_in_flight = vec![vk::Fence::null(); swapchain_images.len()];

        Ok(Self {
            instance,
            surface_loader,
            surface,
            device,
            graphics_queue,
            present_queue,
            swapchain_loader,
            swapchain,
            swapchain_imageviews,
            swapchain_extent,
            render_pass,
            pipeline_layout,
            pipeline,
            command_pool,
            command_buffers,
            framebuffers,
            depth_image,
            depth_image_memory,
            depth_image_view,
            texture_image,
            texture_image_memory,
            texture_image_view,
            texture_sampler,
            descriptor_set_layout,
            descriptor_pool,
            _descriptor_sets: descriptor_sets,
            uniform_buffers,
            uniform_buffers_memory,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
        })
    }

    fn update_uniform_buffer(&self, image_index: usize, model: Mat4, view: Mat4, proj: Mat4) -> Result<()> {
        let mvp = proj * view * model;
        let mut ubo: UniformBufferObject = unsafe { std::mem::zeroed() };
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

        unsafe {
            let data_ptr = self.device.map_memory(
                self.uniform_buffers_memory[image_index],
                0,
                std::mem::size_of::<UniformBufferObject>() as u64,
                vk::MemoryMapFlags::empty(),
            )? as *mut u8;
            let src = (&ubo as *const UniformBufferObject) as *const u8;
            std::ptr::copy_nonoverlapping(
                src,
                data_ptr,
                std::mem::size_of::<UniformBufferObject>(),
            );
            self.device.unmap_memory(self.uniform_buffers_memory[image_index]);
        }
        Ok(())
    }

    fn draw_frame(&mut self, model: Mat4, view: Mat4, proj: Mat4) -> Result<()> {
        let fence = self.in_flight_fences[self.current_frame];
        unsafe {
            self.device.wait_for_fences(&[fence], true, u64::MAX)?;
            self.device.reset_fences(&[fence])?;
        }

        let (image_index, _is_suboptimal) = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )?
        };
        let image_index = image_index as usize;

        if self.images_in_flight[image_index] != vk::Fence::null() {
            unsafe {
                self.device.wait_for_fences(&[self.images_in_flight[image_index]], true, u64::MAX)?;
            }
        }
        self.images_in_flight[image_index] = fence;

        self.update_uniform_buffer(image_index, model, view, proj)?;

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.command_buffers[image_index]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.device.queue_submit(self.graphics_queue, &[*submit_info], fence)?;
        }

        let swapchains = [self.swapchain];
        let image_indices = [image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain_loader.queue_present(self.present_queue, &present_info)?;
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().ok();

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device.destroy_semaphore(self.image_available_semaphores[i], None);
                self.device.destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            for i in 0..self.uniform_buffers.len() {
                self.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device.free_memory(self.uniform_buffers_memory[i], None);
            }

            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            self.device.destroy_sampler(self.texture_sampler, None);
            self.device.destroy_image_view(self.texture_image_view, None);
            self.device.destroy_image(self.texture_image, None);
            self.device.free_memory(self.texture_image_memory, None);

            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            self.device.free_memory(self.depth_image_memory, None);

            for framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }

            self.device.free_command_buffers(self.command_pool, &self.command_buffers);
            self.device.destroy_command_pool(self.command_pool, None);

            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for view in &self.swapchain_imageviews {
                self.device.destroy_image_view(*view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);

            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
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

    let mut app = VulkanApp::new(&window)?;

    let mut event_pump = sdl.event_pump().map_err(|e| anyhow!(e)).context("event pump")?;
    let mut spin_angle = 1.0_f32;

    let eye = Vec3::new(0.0, 3.0, 5.0);
    let center = Vec3::ZERO;
    let up = Vec3::Y;
    let mut model = Mat4::IDENTITY;

    let mut last_frame = Instant::now();
    let frame_time = Duration::from_micros(1_000_000 / 60);

    'main: loop {
        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: Some(key), .. } => {
                    match key {
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
                    }
                }
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

        if last_frame.elapsed() >= frame_time {
            let aspect = app.swapchain_extent.width as f32 / app.swapchain_extent.height as f32;
            let mut proj = Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect, 0.1, 100.0);
            proj.y_axis.y *= -1.0;
            let view = Mat4::look_at_rh(eye, center, up);

            model = model * Mat4::from_rotation_y(spin_angle.to_radians());
            app.draw_frame(model, view, proj)?;
            last_frame = Instant::now();
        }
    }

    Ok(())
}

fn pick_physical_device(
    instance: &Instance,
    surface_loader: &Surface,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32, u32)> {
    let devices = unsafe { instance.enumerate_physical_devices() }?;
    for device in devices {
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };
        let mut graphics_index = None;
        let mut present_index = None;
        for (index, qf) in queue_families.iter().enumerate() {
            if qf.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_index = Some(index as u32);
            }
            let supports_present = unsafe {
                surface_loader.get_physical_device_surface_support(device, index as u32, surface)
            }?;
            if supports_present {
                present_index = Some(index as u32);
            }
            if graphics_index.is_some() && present_index.is_some() {
                break;
            }
        }
        if let (Some(g), Some(p)) = (graphics_index, present_index) {
            return Ok((device, g, p));
        }
    }
    Err(anyhow!("no suitable physical device"))
}

fn create_swapchain(
    window: &sdl2::video::Window,
    surface_loader: &Surface,
    swapchain_loader: &Swapchain,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    graphics_queue_index: u32,
    present_queue_index: u32,
) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D)> {
    let surface_caps = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
    };
    let surface_formats = unsafe {
        surface_loader.get_physical_device_surface_formats(physical_device, surface)?
    };
    let present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?
    };

    let preferred_format = surface_formats
        .iter()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_UNORM
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .cloned()
        .unwrap_or(surface_formats[0]);

    let present_mode = present_modes
        .iter()
        .cloned()
        .find(|&m| m == vk::PresentModeKHR::FIFO)
        .unwrap_or(vk::PresentModeKHR::FIFO);

    let (width, height) = window.drawable_size();
    let extent = if surface_caps.current_extent.width != u32::MAX {
        surface_caps.current_extent
    } else {
        vk::Extent2D {
            width: width.max(1),
            height: height.max(1),
        }
    };

    let mut image_count = surface_caps.min_image_count + 1;
    if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
        image_count = surface_caps.max_image_count;
    }

    let queue_family_indices = [graphics_queue_index, present_queue_index];
    let image_sharing_mode = if graphics_queue_index != present_queue_index {
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };

    let swapchain_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(preferred_format.format)
        .image_color_space(preferred_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(surface_caps.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_info, None) }?;
    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }?;

    Ok((swapchain, images, preferred_format.format, extent))
}

fn create_image_view(
    device: &Device,
    image: vk::Image,
    _format: vk::Format,
    aspect: vk::ImageAspectFlags,
) -> Result<vk::ImageView> {
    let view_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(_format)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect)
                .level_count(1)
                .layer_count(1)
                .build(),
        );
    let view = unsafe { device.create_image_view(&view_info, None) }?;
    Ok(view)
}

fn create_render_pass(device: &Device, color_format: vk::Format) -> Result<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(color_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build();

    let depth_attachment = vk::AttachmentDescription::builder()
        .format(vk::Format::D16_UNORM)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();

    let color_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();
    let depth_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&[color_ref])
        .depth_stencil_attachment(&depth_ref)
        .build();

    let attachments = [color_attachment, depth_attachment];
    let subpasses = [subpass];
    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses);

    let render_pass = unsafe { device.create_render_pass(&render_pass_info, None) }?;
    Ok(render_pass)
}

fn create_descriptor_set_layout(device: &Device) -> Result<vk::DescriptorSetLayout> {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .build();

    let sampler_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .build();

    let bindings = [ubo_layout_binding, sampler_layout_binding];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

    let layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }?;
    Ok(layout)
}

fn create_graphics_pipeline(
    device: &Device,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> Result<(vk::PipelineLayout, vk::Pipeline)> {
    let vert_shader_code = include_bytes!("../shaders/cube.vert.spv");
    let frag_shader_code = include_bytes!("../shaders/cube.frag.spv");

    let vert_module = create_shader_module(device, vert_shader_code)?;
    let frag_module = create_shader_module(device, frag_shader_code)?;

    let main = CString::new("main")?;
    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(&main)
        .build();
    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(&main)
        .build();

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::builder();
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .scissor_count(1);

    let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

    let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false)
        .build();

    let blend_attachments = [color_blend_attachment];
    let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
        .attachments(&blend_attachments);

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&dynamic_states);

    let set_layouts = [descriptor_set_layout];
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&set_layouts);

    let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;

    let shader_stages = [vert_stage, frag_stage];
    let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .color_blend_state(&color_blending)
        .depth_stencil_state(&depth_stencil)
        .dynamic_state(&dynamic_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);

    let pipelines = unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[*pipeline_info], None)
    }
    .map_err(|e| e.1)?;

    unsafe {
        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);
    }

    Ok((pipeline_layout, pipelines[0]))
}

fn create_shader_module(device: &Device, code: &[u8]) -> Result<vk::ShaderModule> {
    let code_aligned = ash::util::read_spv(&mut std::io::Cursor::new(code))?;
    let info = vk::ShaderModuleCreateInfo::builder().code(&code_aligned);
    let module = unsafe { device.create_shader_module(&info, None) }?;
    Ok(module)
}

fn create_command_pool(device: &Device, queue_family_index: u32) -> Result<vk::CommandPool> {
    let info = vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family_index);
    let pool = unsafe { device.create_command_pool(&info, None) }?;
    Ok(pool)
}

fn create_depth_resources(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    extent: vk::Extent2D,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
    let (image, memory) = create_image(
        instance,
        device,
        physical_device,
        extent.width,
        extent.height,
        vk::Format::D16_UNORM,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    let view = create_image_view(device, image, vk::Format::D16_UNORM, vk::ImageAspectFlags::DEPTH)?;

    transition_image_layout(
        device,
        command_pool,
        graphics_queue,
        image,
        vk::Format::D16_UNORM,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    )?;

    Ok((image, memory, view))
}

fn create_texture_image(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    let png_bytes = include_bytes!("../textures/gopher.png");
    let img = image::load_from_memory(png_bytes).context("load texture")?;
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    let image_size = (width * height * 4) as vk::DeviceSize;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        physical_device,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    unsafe {
        let data_ptr = device.map_memory(
            staging_buffer_memory,
            0,
            image_size,
            vk::MemoryMapFlags::empty(),
        )? as *mut u8;
        data_ptr.copy_from_nonoverlapping(rgba.as_ptr(), rgba.len());
        device.unmap_memory(staging_buffer_memory);
    }

    let (texture_image, texture_image_memory) = create_image(
        instance,
        device,
        physical_device,
        width,
        height,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    transition_image_layout(
        device,
        command_pool,
        graphics_queue,
        texture_image,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    )?;

    copy_buffer_to_image(
        device,
        command_pool,
        graphics_queue,
        staging_buffer,
        texture_image,
        width,
        height,
    )?;

    transition_image_layout(
        device,
        command_pool,
        graphics_queue,
        texture_image,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    )?;

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    Ok((texture_image, texture_image_memory))
}

fn create_texture_sampler(device: &Device) -> Result<vk::Sampler> {
    let info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .anisotropy_enable(false)
        .max_anisotropy(1.0)
        .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::NEVER)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST);

    let sampler = unsafe { device.create_sampler(&info, None) }?;
    Ok(sampler)
}

fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    count: usize,
) -> Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
    let buffer_size = std::mem::size_of::<UniformBufferObject>() as u64;
    let mut buffers = Vec::with_capacity(count);
    let mut memories = Vec::with_capacity(count);
    for _ in 0..count {
        let (buffer, memory) = create_buffer(
            instance,
            device,
            physical_device,
            buffer_size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        buffers.push(buffer);
        memories.push(memory);
    }
    Ok((buffers, memories))
}

fn create_descriptor_pool(device: &Device, count: usize) -> Result<vk::DescriptorPool> {
    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: count as u32,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: count as u32,
        },
    ];

    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(count as u32);

    let pool = unsafe { device.create_descriptor_pool(&info, None) }?;
    Ok(pool)
}

fn create_descriptor_sets(
    device: &Device,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    uniform_buffers: &[vk::Buffer],
    texture_view: vk::ImageView,
    sampler: vk::Sampler,
) -> Result<Vec<vk::DescriptorSet>> {
    let layouts = vec![layout; uniform_buffers.len()];
    let alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts);

    let sets = unsafe { device.allocate_descriptor_sets(&alloc_info) }?;

    for (i, set) in sets.iter().enumerate() {
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffers[i])
            .offset(0)
            .range(std::mem::size_of::<UniformBufferObject>() as u64)
            .build();

        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture_view)
            .sampler(sampler)
            .build();

        let writes = [
            vk::WriteDescriptorSet::builder()
                .dst_set(*set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&[buffer_info])
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(*set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&[image_info])
                .build(),
        ];

        unsafe { device.update_descriptor_sets(&writes, &[]) };
    }

    Ok(sets)
}

fn create_framebuffers(
    device: &Device,
    render_pass: vk::RenderPass,
    image_views: &[vk::ImageView],
    depth_view: vk::ImageView,
    extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>> {
    let mut framebuffers = Vec::with_capacity(image_views.len());
    for &view in image_views {
        let attachments = [view, depth_view];
        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);
        let fb = unsafe { device.create_framebuffer(&info, None) }?;
        framebuffers.push(fb);
    }
    Ok(framebuffers)
}

fn create_command_buffers(
    device: &Device,
    pool: vk::CommandPool,
    render_pass: vk::RenderPass,
    framebuffers: &[vk::Framebuffer],
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_sets: &[vk::DescriptorSet],
    extent: vk::Extent2D,
) -> Result<Vec<vk::CommandBuffer>> {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(framebuffers.len() as u32);

    let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info) }?;

    for (i, &cmd) in command_buffers.iter().enumerate() {
        let begin_info = vk::CommandBufferBeginInfo::builder();
        unsafe { device.begin_command_buffer(cmd, &begin_info) }?;

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue { float32: [0.2, 0.2, 0.2, 0.2] },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
            },
        ];

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffers[i])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            })
            .clear_values(&clear_values);

        unsafe {
            device.cmd_begin_render_pass(cmd, &render_pass_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline);
            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &[descriptor_sets[i]],
                &[],
            );

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };
            device.cmd_set_viewport(cmd, 0, &[viewport]);
            device.cmd_set_scissor(cmd, 0, &[scissor]);
            device.cmd_draw(cmd, 36, 1, 0, 0);
            device.cmd_end_render_pass(cmd);
            device.end_command_buffer(cmd)?;
        }
    }

    Ok(command_buffers)
}

fn create_sync_objects(
    device: &Device,
    count: usize,
) -> Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>)> {
    let mut image_available_semaphores = Vec::with_capacity(count);
    let mut render_finished_semaphores = Vec::with_capacity(count);
    let mut in_flight_fences = Vec::with_capacity(count);

    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

    for _ in 0..count {
        let image_available = unsafe { device.create_semaphore(&semaphore_info, None) }?;
        let render_finished = unsafe { device.create_semaphore(&semaphore_info, None) }?;
        let fence = unsafe { device.create_fence(&fence_info, None) }?;

        image_available_semaphores.push(image_available);
        render_finished_semaphores.push(render_finished);
        in_flight_fences.push(fence);
    }

    Ok((image_available_semaphores, render_finished_semaphores, in_flight_fences))
}

fn create_buffer(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buffer = unsafe { device.create_buffer(&info, None) }?;

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let mem_type = find_memory_type(instance, physical_device, mem_requirements.memory_type_bits, properties)?;

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(mem_type);
    let memory = unsafe { device.allocate_memory(&alloc_info, None) }?;

    unsafe { device.bind_buffer_memory(buffer, memory, 0) }?;

    Ok((buffer, memory))
}

fn create_image(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(tiling)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let image = unsafe { device.create_image(&image_info, None) }?;
    let mem_requirements = unsafe { device.get_image_memory_requirements(image) };
    let mem_type = find_memory_type(instance, physical_device, mem_requirements.memory_type_bits, properties)?;

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(mem_type);

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }?;
    unsafe { device.bind_image_memory(image, memory, 0) }?;

    Ok((image, memory))
}

fn find_memory_type(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> Result<u32> {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };
    for i in 0..mem_properties.memory_type_count {
        if (type_filter & (1 << i)) != 0
            && mem_properties.memory_types[i as usize]
                .property_flags
                .contains(properties)
        {
            return Ok(i);
        }
    }
    Err(anyhow!("failed to find suitable memory type"))
}

fn transition_image_layout(
    device: &Device,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    image: vk::Image,
    _format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let (src_access_mask, dst_access_mask, src_stage, dst_stage) = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        ),
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        ),
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        ),
        _ => {
            return Err(anyhow!("unsupported layout transition"));
        }
    };

    let aspect = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        vk::ImageAspectFlags::DEPTH
    } else {
        vk::ImageAspectFlags::COLOR
    };

    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        )
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask)
        .build();

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }

    end_single_time_commands(device, command_pool, graphics_queue, command_buffer)?;
    Ok(())
}

fn copy_buffer_to_image(
    device: &Device,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, command_pool)?;

    let region = vk::BufferImageCopy::builder()
        .image_subresource(
            vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        )
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .build();

    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }

    end_single_time_commands(device, command_pool, graphics_queue, command_buffer)?;
    Ok(())
}

fn begin_single_time_commands(device: &Device, command_pool: vk::CommandPool) -> Result<vk::CommandBuffer> {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let command_buffer = unsafe { device.allocate_command_buffers(&alloc_info) }?[0];
    let begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }?;
    Ok(command_buffer)
}

fn end_single_time_commands(
    device: &Device,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    unsafe {
        device.end_command_buffer(command_buffer)?;
        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder().command_buffers(&command_buffers);
        device.queue_submit(graphics_queue, &[*submit_info], vk::Fence::null())?;
        device.queue_wait_idle(graphics_queue)?;
        device.free_command_buffers(command_pool, &[command_buffer]);
    }
    Ok(())
}
