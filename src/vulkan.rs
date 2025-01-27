use ash_window;

use ash::{khr, vk, Device, Entry, Instance};
use lineal::Vector;
use std::collections::{BTreeMap, HashSet};
use std::ffi::{CStr, CString};
use std::fs::File;
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

const VALIDATION_LAYERS_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

const DEVICE_EXTENSIONS: [&CStr; 1] = [vk::KHR_SWAPCHAIN_NAME];

const MAX_FRAMES_IN_FLIGHT: u32 = 2;

struct Vertex {
    position: Vector<f32, 2>,
    color: Vector<f32, 3>,
}

impl Vertex {
    fn get_binding_description() -> vk::VertexInputBindingDescription {
        return vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        };
    }

    fn get_attribute_description() -> [vk::VertexInputAttributeDescription; 2] {
        let base = std::ptr::null::<Vertex>();
        let position_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: unsafe { &(*base).position as *const _ as u32 },
        };

        let color_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: unsafe { &(*base).color as *const _ as u32 },
        };

        return [position_attribute, color_attribute];
    }
}

pub struct VkContext {
    entry: Entry,
    instance: Instance,
    physical_device: vk::PhysicalDevice,
    device: Device,

    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    queue_families: QueueFamiliesIndices,

    surface_loader: khr::surface::Instance,
    surface: vk::SurfaceKHR,

    swapchain_loader: khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_image_views: Vec<vk::ImageView>,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: u32,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
}

#[derive(Clone)]
pub struct QueueFamiliesIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

pub struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl VkContext {
    pub fn new(window: &Window) -> Result<Self, String> {
        let entry = Entry::linked();

        let instance = Self::create_instance(&entry, window)?;

        let surface_loader = khr::surface::Instance::new(&entry, &instance);
        let surface = Self::create_surface(window, &entry, &instance)?;

        let (physical_device, queue_families) =
            Self::choose_physical_device(&instance, &surface_loader, &surface)?;

        let device = Self::create_device(&instance, &physical_device, &queue_families)?;

        let graphics_queue =
            unsafe { device.get_device_queue(queue_families.graphics_family.unwrap(), 0) };

        let present_queue =
            unsafe { device.get_device_queue(queue_families.present_family.unwrap(), 0) };

        let (
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_image_format,
            swapchain_extent,
        ) = Self::create_swapchain(
            &instance,
            window,
            &device,
            &physical_device,
            &queue_families,
            &surface_loader,
            &surface,
        )?;

        let swapchain_image_views =
            Self::create_image_views(&device, &swapchain_images, &swapchain_image_format)?;

        let render_pass = Self::create_render_pass(&device, &swapchain_image_format)?;

        let (pipeline, pipeline_layout) = Self::create_graphics_pipeline(&device, &render_pass)?;

        let swapchain_framebuffers = Self::create_framebuffers(
            &device,
            &render_pass,
            &swapchain_image_views,
            &swapchain_extent,
        )?;

        let command_pool = Self::create_command_pool(&device, &queue_families)?;

        let command_buffers = Self::create_command_buffers(&device, &command_pool)?;

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
            Self::create_sync_objects(&device)?;

        let current_frame = 0;

        let (vertex_buffer, vertex_buffer_memory) = Self::create_vertex_buffer(&instance, &physical_device, &device)?;

        return Ok(Self {
            entry,
            instance,
            physical_device,
            device,
            graphics_queue,
            present_queue,
            surface_loader,
            queue_families,
            surface,
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_image_format,
            swapchain_extent,
            swapchain_image_views,
            render_pass,
            pipeline_layout,
            pipeline,
            swapchain_framebuffers,
            command_pool,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame,
            vertex_buffer,
            vertex_buffer_memory,
        });
    }

    fn check_validation_layer_support(entry: &Entry) -> bool {
        let available_layers: Vec<vk::LayerProperties>;

        unsafe {
            match entry.enumerate_instance_layer_properties() {
                Ok(layers_properties) => available_layers = layers_properties,
                Err(_) => return false,
            }
        }

        for layer_name in VALIDATION_LAYERS {
            let mut found = false;

            for layer_properties in &available_layers {
                let layer_properties: Vec<u8> = layer_properties
                    .layer_name
                    .iter()
                    .map(|&b| b as u8)
                    .collect();

                if layer_name.as_bytes() == layer_properties.as_slice() {
                    found = true;
                    break;
                }
            }

            if found == false {
                return false;
            }
        }

        return true;
    }

    fn create_instance(entry: &Entry, window: &Window) -> Result<Instance, String> {
        if VALIDATION_LAYERS_ENABLED && Self::check_validation_layer_support(entry) {
            return Err("Validation layers not supported".to_string());
        }

        // Set up Vulkan application information
        let application_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };

        let display_handle = window
            .display_handle()
            .map_err(|e| format!("Error with display: {}", e))?;

        let extension_names = ash_window::enumerate_required_extensions(display_handle.as_raw())
            .map_err(|e| format!("Error with extension: {}", e))?;

        let validation_layers: Vec<CString> = VALIDATION_LAYERS
            .iter()
            .map(|&layer| CString::new(layer).unwrap())
            .collect();

        // Get raw pointers to the CStrings
        let validation_layers: Vec<*const i8> = validation_layers
            .iter()
            .map(|layer| layer.as_ptr())
            .collect();

        // Create Vulkan instance
        let mut create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            ..Default::default()
        };

        if VALIDATION_LAYERS_ENABLED {
            create_info.pp_enabled_layer_names = validation_layers.as_ptr();
            create_info.enabled_layer_count = validation_layers.len() as u32;
        }

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|e| format!("Failed to create Vulkan instance: {:?}", e))?
        };

        return Ok(instance);
    }

    fn create_surface(
        window: &Window,
        entry: &Entry,
        instance: &Instance,
    ) -> Result<vk::SurfaceKHR, String> {
        let display_handle = window
            .display_handle()
            .map_err(|e| format!("Error with display: {}", e))?;
        let window_handle = window
            .window_handle()
            .map_err(|e| format!("Error with window: {}", e))?;

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
            .map_err(|e| format!("Failed to create surface: {}", e))?
        };

        return Ok(surface);
    }

    fn create_device(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        queue_family: &QueueFamiliesIndices,
    ) -> Result<Device, String> {
        let graphics_family = queue_family.graphics_family.unwrap();
        let present_family = queue_family.present_family.unwrap();

        let mut unique_queue_families = vec![graphics_family, present_family];
        unique_queue_families.dedup();

        let queue_priority = 1.0;
        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_families
            .iter()
            .map(|&queue_family| vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family,
                queue_count: 1,
                p_queue_priorities: &queue_priority,
                ..Default::default()
            })
            .collect();

        let device_features = vk::PhysicalDeviceFeatures::default();

        let device_extensions: Vec<_> = DEVICE_EXTENSIONS
            .iter()
            .map(|extension| extension.as_ptr())
            .collect();
        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: &device_features,
            enabled_extension_count: device_extensions.len() as u32,
            pp_enabled_extension_names: device_extensions.as_ptr(),
            ..Default::default()
        };

        let device = unsafe {
            instance
                .create_device(*physical_device, &create_info, None)
                .map_err(|e| format!("Failed to create logical device: {}", e))?
        };

        return Ok(device);
    }

    fn choose_physical_device(
        instance: &Instance,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> Result<(vk::PhysicalDevice, QueueFamiliesIndices), String> {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(|e| format!("Failed to enumerate physical devices: {:?}", e))?
        };

        if physical_devices.is_empty() {
            return Err("No Vulkan-compatible physical devices found.".to_string());
        }

        let mut candidates: BTreeMap<i32, (vk::PhysicalDevice, QueueFamiliesIndices)> =
            BTreeMap::new();
        for physical_device in physical_devices {
            let (score, queue_families) =
                Self::rate_device(instance, surface_loader, surface, physical_device)?;
            if score > 0 {
                if Self::is_device_suitable(
                    instance,
                    &physical_device,
                    &queue_families,
                    surface_loader,
                    surface,
                ) {
                    candidates.insert(score, (physical_device, queue_families));
                }
            }
        }

        return candidates.iter().rev().next().map_or_else(
            || Err("Failed to find a suitable GPU.".to_string()),
            |(_, (device, queue_family))| Ok((*device, queue_family.clone())),
        );
    }

    fn rate_device(
        instance: &Instance,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
    ) -> Result<(i32, QueueFamiliesIndices), String> {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.get_physical_device_features(physical_device) };
        let queue_families =
            Self::find_queue_families(instance, &physical_device, surface_loader, surface);

        let mut score = 0;

        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }

        score += properties.limits.max_image_dimension2_d as i32;

        if features.geometry_shader == 0 || queue_families.graphics_family.is_none() {
            return Ok((0, queue_families)); // Skip if no geometry shader or graphics family
        }

        return Ok((score, queue_families));
    }

    fn is_device_suitable(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        queue_families: &QueueFamiliesIndices,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> bool {
        let device_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(*physical_device)
                .map_err(|e| format!("{}", e))
                .unwrap_or_default()
        };

        let swapchain_support;
        match Self::query_swapchain_support(physical_device, surface_loader, surface) {
            Ok(value) => swapchain_support = value,
            Err(_) => return false,
        }

        let mut required_extensions: HashSet<&CStr> = HashSet::from(DEVICE_EXTENSIONS);

        for extension in device_extensions {
            let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };

            if required_extensions.contains(extension_name) {
                required_extensions.remove(extension_name);
            }
        }

        return required_extensions.is_empty()
            && queue_families.graphics_family.is_some()
            && !swapchain_support.formats.is_empty()
            && !swapchain_support.present_modes.is_empty();
    }

    fn find_queue_families(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> QueueFamiliesIndices {
        let mut graphics_family = None;
        let mut present_family = None;

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        for (index, queue_family) in queue_families.iter().enumerate() {
            let index = index as u32;

            let graphics_flags = queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
            if graphics_family.is_none() && graphics_flags {
                graphics_family = Some(index);
            }

            let present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(*physical_device, index, *surface)
                    .unwrap()
            };

            if present_support && present_family.is_none() {
                present_family = Some(index);
            }

            if graphics_family.is_some() && present_family.is_some() {
                break;
            }
        }

        return QueueFamiliesIndices {
            graphics_family,
            present_family,
        };
    }

    fn query_swapchain_support(
        physical_device: &vk::PhysicalDevice,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> Result<SwapChainSupportDetails, String> {
        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(*physical_device, *surface)
                .map_err(|e| format!("Failed to get surface capabilities: {}", e))?
        };

        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(*physical_device, *surface)
                .map_err(|e| format!("Failed to get surface formats: {}", e))?
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(*physical_device, *surface)
                .map_err(|e| format!("Failed to get surface present modes: {}", e))?
        };

        return Ok(SwapChainSupportDetails {
            capabilities,
            formats,
            present_modes,
        });
    }

    fn create_swapchain(
        instance: &Instance,
        window: &Window,
        device: &Device,
        physical_device: &vk::PhysicalDevice,
        queue_families: &QueueFamiliesIndices,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> Result<
        (
            khr::swapchain::Device,
            vk::SwapchainKHR,
            Vec<vk::Image>,
            vk::Format,
            vk::Extent2D,
        ),
        String,
    > {
        let swapchain_support_details =
            Self::query_swapchain_support(physical_device, surface_loader, surface)
                .map_err(|e| format!("Failed to get swapchain support details: {}", e))?;

        let surface_format =
            Self::choose_swapchain_surface_format(&swapchain_support_details.formats);
        let present_mode =
            Self::choose_swapchain_present_mode(&swapchain_support_details.present_modes);
        let extent = Self::choose_swapchain_extent(window, &swapchain_support_details.capabilities);

        let mut image_count = swapchain_support_details.capabilities.min_image_count + 1;
        if swapchain_support_details.capabilities.max_image_count > 0
            && swapchain_support_details.capabilities.max_image_count < image_count
        {
            image_count = swapchain_support_details.capabilities.max_image_count;
        }

        let mut create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: *surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            pre_transform: swapchain_support_details.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };

        if queue_families.graphics_family != queue_families.present_family {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.queue_family_index_count = 2;
            create_info.p_queue_family_indices = [
                queue_families.graphics_family.unwrap(),
                queue_families.present_family.unwrap(),
            ]
            .as_ptr()
        } else {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
            create_info.queue_family_index_count = 0;
            create_info.p_queue_family_indices = std::ptr::null();
        }

        let swapchain_loader = khr::swapchain::Device::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .map_err(|e| format!("Failed to create swapchain: {}", e))?
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .map_err(|e| format!("Failed to get swapchain images: {}", e))?
        };

        return Ok((
            swapchain_loader,
            swapchain,
            swapchain_images,
            surface_format.format,
            extent,
        ));
    }

    fn choose_swapchain_surface_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *available_format;
            }
        }

        return available_formats[0];
    }

    fn choose_swapchain_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for available_present_mode in available_present_modes {
            if *available_present_mode == vk::PresentModeKHR::MAILBOX {
                return *available_present_mode;
            }
        }

        return vk::PresentModeKHR::FIFO;
    }

    fn choose_swapchain_extent(
        window: &Window,
        capabilities: &vk::SurfaceCapabilitiesKHR,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        } else {
            let (width, height): (u32, u32) = window.inner_size().into();

            let extent = vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            };

            return extent;
        }
    }

    fn create_image_views(
        device: &Device,
        swapchain_images: &Vec<vk::Image>,
        swapchain_image_format: &vk::Format,
    ) -> Result<Vec<vk::ImageView>, String> {
        let mut swapchain_image_views: Vec<vk::ImageView> = Vec::new();

        for image in swapchain_images {
            let create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image: *image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: *swapchain_image_format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };

            let image_view = unsafe {
                device
                    .create_image_view(&create_info, None)
                    .map_err(|e| format!("Failed to create image view: {}", e))?
            };
            swapchain_image_views.push(image_view);
        }

        return Ok(swapchain_image_views);
    }

    fn create_render_pass(
        device: &Device,
        swapchain_image_format: &vk::Format,
    ) -> Result<vk::RenderPass, String> {
        let color_attachment = vk::AttachmentDescription {
            format: *swapchain_image_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            ..Default::default()
        };

        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let render_pass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            attachment_count: 1,
            p_attachments: &color_attachment,
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &dependency,
            ..Default::default()
        };

        let render_pass = unsafe {
            device
                .create_render_pass(&render_pass_create_info, None)
                .map_err(|e| format!("Failed to create render pass: {}", e))?
        };

        return Ok(render_pass);
    }

    fn create_graphics_pipeline(
        device: &Device,
        render_pass: &vk::RenderPass,
    ) -> Result<(vk::Pipeline, vk::PipelineLayout), String> {
        let frag = read_spv_file("shaders/shader.frag.spv")?;
        let vert = read_spv_file("shaders/shader.vert.spv")?;

        let frag_shader_module = Self::create_shader_module(device, &frag)?;
        let vert_shader_module = Self::create_shader_module(device, &vert)?;

        let entrypoint = CString::new("main").unwrap();
        let vert_shader_create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::VERTEX,
            module: vert_shader_module,
            p_name: entrypoint.as_ptr(),
            ..Default::default()
        };

        let frag_shader_create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: frag_shader_module,
            p_name: entrypoint.as_ptr(),
            ..Default::default()
        };

        let shader_stages = [vert_shader_create_info, frag_shader_create_info];

        let binding_description = Vertex::get_binding_description();
        let attribute_descriptions = Vertex::get_attribute_description();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &binding_description,
            vertex_attribute_description_count: attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: attribute_descriptions.as_ptr(),
            ..Default::default()
        };

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            viewport_count: 1,
            scissor_count: 1,
            ..Default::default()
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            depth_bias_constant_factor: 0.,
            depth_bias_clamp: 0.,
            depth_bias_slope_factor: 0.,
            ..Default::default()
        };

        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.,
            p_sample_mask: std::ptr::null(),
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable: vk::FALSE,
            ..Default::default()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        };

        let color_blending = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &color_blend_attachment,
            blend_constants: [0.; 4],
            ..Default::default()
        };

        let dynamic_states = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            set_layout_count: 0,
            p_set_layouts: std::ptr::null(),
            push_constant_range_count: 0,
            p_push_constant_ranges: std::ptr::null(),
            ..Default::default()
        };

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .map_err(|e| format!("Failed to create pipeline layout: {}", e))?
        };

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_info,
            p_input_assembly_state: &input_assembly,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterizer,
            p_multisample_state: &multisampling,
            p_color_blend_state: &color_blending,
            p_dynamic_state: &dynamic_state,
            layout: pipeline_layout,
            render_pass: *render_pass,
            subpass: 0,
            ..Default::default()
        };

        let pipeline_create_infos = [pipeline_create_info];
        let pipeline_cache = vk::PipelineCache::null();
        let pipeline = unsafe {
            device
                .create_graphics_pipelines(pipeline_cache, &pipeline_create_infos, None)
                .map_err(|_| format!("Failed to create graphics pipeline"))?
                .remove(0)
        };

        return Ok((pipeline, pipeline_layout));
    }

    fn create_shader_module(device: &Device, code: &Vec<u32>) -> Result<vk::ShaderModule, String> {
        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            code_size: code.len() * std::mem::size_of::<u32>(),
            p_code: code.as_ptr(),
            ..Default::default()
        };

        let shader_module = unsafe {
            device
                .create_shader_module(&create_info, None)
                .map_err(|e| format!("Failed to create shader module: {}", e))?
        };

        return Ok(shader_module);
    }

    fn create_framebuffers(
        device: &Device,
        render_pass: &vk::RenderPass,
        swapchain_image_views: &Vec<vk::ImageView>,
        swapchain_extent: &vk::Extent2D,
    ) -> Result<Vec<vk::Framebuffer>, String> {
        let mut framebuffers = Vec::new();

        for swapchain_image_view in swapchain_image_views {
            let attachment = swapchain_image_view;

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                render_pass: *render_pass,
                attachment_count: 1,
                p_attachments: attachment,
                width: swapchain_extent.width,
                height: swapchain_extent.height,
                layers: 1,
                ..Default::default()
            };

            let framebuffer = unsafe {
                device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .map_err(|e| format!("Failed to create framebuffer: {}", e))?
            };
            framebuffers.push(framebuffer);
        }

        return Ok(framebuffers);
    }

    fn create_command_pool(
        device: &Device,
        queue_families: &QueueFamiliesIndices,
    ) -> Result<vk::CommandPool, String> {
        let create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: queue_families.graphics_family.unwrap(),
            ..Default::default()
        };

        let command_pool = unsafe {
            device
                .create_command_pool(&create_info, None)
                .map_err(|e| format!("Failed to create command pool: {}", e))?
        };

        return Ok(command_pool);
    }

    fn create_command_buffers(
        device: &Device,
        command_pool: &vk::CommandPool,
    ) -> Result<Vec<vk::CommandBuffer>, String> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: *command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MAX_FRAMES_IN_FLIGHT,
            ..Default::default()
        };

        let command_buffer = unsafe {
            device
                .allocate_command_buffers(&allocate_info)
                .map_err(|e| format!("Failed to allocate command buffers: {}", e))?
        };

        return Ok(command_buffer);
    }

    fn create_sync_objects(
        device: &Device,
    ) -> Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>), String> {
        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let capacity = MAX_FRAMES_IN_FLIGHT as usize;
        let mut image_semaphores = Vec::with_capacity(capacity);
        let mut render_semaphores = Vec::with_capacity(capacity);
        let mut fences = Vec::with_capacity(capacity);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let image_semaphore = unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| format!("Failed to create semaphore: {}", e))?
            };
            let render_semaphore = unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| format!("Failed to create semaphore: {}", e))?
            };
            let fence = unsafe {
                device
                    .create_fence(&fence_info, None)
                    .map_err(|e| format!("Failed to create fence: {}", e))?
            };

            image_semaphores.push(image_semaphore);
            render_semaphores.push(render_semaphore);
            fences.push(fence);
        }

        return Ok((image_semaphores, render_semaphores, fences));
    }

    pub fn record_command_buffer(
        &self,
        command_buffer: &vk::CommandBuffer,
        image_index: u32,
    ) -> Result<(), String> {
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags: vk::CommandBufferUsageFlags::empty(),
            p_inheritance_info: std::ptr::null(),
            ..Default::default()
        };

        let _ = unsafe {
            self.device
                .begin_command_buffer(*command_buffer, &begin_info)
                .map_err(|e| format!("Failed to start command buffer: {}", e))?
        };

        let clear_color = vk::ClearColorValue {
            float32: [0., 0., 0., 1.0],
        };

        let clear_value = vk::ClearValue { color: clear_color };

        let render_pass_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass,
            framebuffer: self.swapchain_framebuffers[image_index as usize],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            },
            clear_value_count: 1,
            p_clear_values: &clear_value,
            ..Default::default()
        };

        let _ = unsafe {
            self.device.cmd_begin_render_pass(
                *command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            )
        };

        let _ = unsafe {
            self.device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            )
        };

        let vertex_buffers = [self.vertex_buffer];
        let offsets = [0];

        unsafe { self.device.cmd_bind_vertex_buffers(*command_buffer, 0, &vertex_buffers, &offsets) };

        let viewport = vk::Viewport {
            x: 0.,
            y: 0.,
            width: self.swapchain_extent.width as f32,
            height: self.swapchain_extent.height as f32,
            min_depth: 0.,
            max_depth: 1.,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain_extent,
        };

        unsafe {
            self.device
                .cmd_set_viewport(*command_buffer, 0, &[viewport])
        };
        unsafe { self.device.cmd_set_scissor(*command_buffer, 0, &[scissor]) };

        unsafe { self.device.cmd_draw(*command_buffer, 3, 1, 0, 0) };

        unsafe { self.device.cmd_end_render_pass(*command_buffer) };

        let _ = unsafe {
            self.device
                .end_command_buffer(*command_buffer)
                .map_err(|e| format!("Failed to end command buffer: {}", e))?
        };

        return Ok(());
    }

    pub fn draw_frame(&mut self) {
        let _ = unsafe {
            self.device.wait_for_fences(
                &[self.in_flight_fences[self.current_frame as usize]],
                true,
                u64::MAX,
            )
        };

        let (image_index, _) = unsafe {
            self.swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    self.image_available_semaphores[self.current_frame as usize],
                    vk::Fence::null(),
                )
                .unwrap()
        };

        let _ = unsafe {
            self.device
                .reset_fences(&[self.in_flight_fences[self.current_frame as usize]])
        };

        let _ = unsafe {
            self.device.reset_command_buffer(
                self.command_buffers[self.current_frame as usize],
                vk::CommandBufferResetFlags::empty(),
            )
        };

        let _ = self.record_command_buffer(
            &self.command_buffers[self.current_frame as usize],
            image_index,
        );

        let signal_semaphores = [self.render_finished_semaphores[self.current_frame as usize]];
        let wait_semaphores = [self.image_available_semaphores[self.current_frame as usize]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[self.current_frame as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        let _ = unsafe {
            self.device.queue_submit(
                self.graphics_queue,
                &[submit_info],
                self.in_flight_fences[self.current_frame as usize],
            )
        };

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: [self.swapchain].as_ptr(),
            p_image_indices: &image_index,
            p_results: std::ptr::null_mut(),
            ..Default::default()
        };

        let _ = unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
                .unwrap()
        };

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn cleanup_swapchain(&mut self) {
        unsafe {
            for index in 0..self.swapchain_framebuffers.len() {
                self.device
                    .destroy_framebuffer(self.swapchain_framebuffers[index], None);
            }

            for index in 0..self.swapchain_image_views.len() {
                self.device
                    .destroy_image_view(self.swapchain_image_views[index], None);
            }

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }

    pub fn cleanup(&mut self) {
        self.cleanup_swapchain();

        unsafe {
            self.device.destroy_buffer(self.vertex_buffer, None);
            self.device.free_memory(self.vertex_buffer_memory, None);
            
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for index in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.render_finished_semaphores[index as usize], None);
                self.device
                    .destroy_semaphore(self.image_available_semaphores[index as usize], None);
                self.device
                    .destroy_fence(self.in_flight_fences[index as usize], None);
            }

            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);

            if VALIDATION_LAYERS_ENABLED {}

            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }

    pub fn recreate_swapchain(&mut self, window: &Window) {
        let _ = unsafe { self.device.device_wait_idle() };

        self.cleanup_swapchain();

        (
            self.swapchain_loader,
            self.swapchain,
            self.swapchain_images,
            self.swapchain_image_format,
            self.swapchain_extent,
        ) = Self::create_swapchain(
            &self.instance,
            window,
            &self.device,
            &self.physical_device,
            &self.queue_families,
            &self.surface_loader,
            &self.surface,
        )
        .unwrap();
        self.swapchain_image_views = Self::create_image_views(
            &self.device,
            &self.swapchain_images,
            &self.swapchain_image_format,
        )
        .unwrap();
        self.swapchain_framebuffers = Self::create_framebuffers(
            &self.device,
            &self.render_pass,
            &self.swapchain_image_views,
            &self.swapchain_extent,
        )
        .unwrap();
    }

    fn create_vertex_buffer(instance: &Instance, physical_device: &vk::PhysicalDevice,device: &Device) -> Result<(vk::Buffer, vk::DeviceMemory), String> {
        let mut vertices = [
            Vertex {
                position: Vector::new([0.0, -0.5]),
                color: Vector::new([1.0, 1.0, 1.0]),
            },
            Vertex {
                position: Vector::new([0.5, 0.5]),
                color: Vector::new([0.0, 1.0, 0.0]),
            },
            Vertex {
                position: Vector::new([-0.5, 0.5]),
                color: Vector::new([0.0, 0.0, 1.0]),
            },
        ];

        let buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: (std::mem::size_of::<Vertex>() * vertices.len()) as u64,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let vertex_buffer = unsafe { device.create_buffer(&buffer_info, None).unwrap() };

        let memory_requirements = unsafe {
            device
                .get_buffer_memory_requirements(vertex_buffer)
        };

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index: Self::find_memory_type(
                    instance,
                    physical_device,
                    memory_requirements.memory_type_bits,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
                .unwrap(),
            
            ..Default::default()
        };

        
        let vertex_buffer_memory = unsafe { device.allocate_memory(&allocate_info, None).unwrap()};
        let _ = unsafe { device.bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0).unwrap()};
        let data = unsafe { device.map_memory(vertex_buffer_memory, 0, buffer_info.size, vk::MemoryMapFlags::empty()).unwrap() };
        unsafe { std::ptr::copy_nonoverlapping(vertices.as_ptr(), data as *mut Vertex, vertices.len()) };
        unsafe { device.unmap_memory(vertex_buffer_memory) };

        return Ok((vertex_buffer, vertex_buffer_memory));
    }

    fn find_memory_type(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32, String> {
        let memory_properties = unsafe {
            instance
                .get_physical_device_memory_properties(*physical_device)
        };

        for index in 0..memory_properties.memory_type_count {
            if (type_filter & (1 << index) != 0)
                && ((memory_properties.memory_types[index as usize].property_flags & properties) == properties)
            {
                return Ok(index);
            }
        }

        return Err("Failed to find suitable memory type".to_string());
    }
}

fn read_spv_file(path: &str) -> Result<Vec<u32>, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file {}: {}", path, e))?;

    let content = ash::util::read_spv(&mut file)
        .map_err(|e| format!("Failed to decode SPIR-V file {}: {}", path, e))?;

    return Ok(content);
}
