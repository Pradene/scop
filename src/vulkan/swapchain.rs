use ash::{khr, vk};
use winit::window::Window;

use crate::vulkan::{VkContext, VkDevice, VkInstance, VkPhysicalDevice, VkPipeline, VkSurface};

use super::{create_image_view, query_swapchain_support, create_image, find_depth_format};

pub struct VkSwapchain {
    pub loader: khr::swapchain::Device,
    pub instance: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>,

    pub framebuffers: Vec<vk::Framebuffer>,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,
}

impl VkSwapchain {
    pub fn new(
        window: &Window,
        instance: &VkInstance,
        surface: &VkSurface,
        physical_device: &VkPhysicalDevice,
        device: &VkDevice,
    ) -> Result<VkSwapchain, String> {
        let support_details = query_swapchain_support(
            &physical_device.physical_device,
            &surface.loader,
            &surface.surface,
        )?;
        let surface_format = VkSwapchain::choose_surface_format(&support_details.formats);
        let present_mode = VkSwapchain::choose_present_mode(&support_details.present_modes);
        let extent = VkSwapchain::choose_extent(window, &support_details.capabilities);

        let image_count = std::cmp::min(
            support_details.capabilities.max_image_count,
            support_details.capabilities.min_image_count + 1,
        )
        .max(support_details.capabilities.min_image_count + 1);

        let image_format = surface_format.format;
        let mut create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: surface.surface,
            min_image_count: image_count,
            image_format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            pre_transform: support_details.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };

        if physical_device.queue_families.graphics_family
            != physical_device.queue_families.present_family
        {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.queue_family_index_count = 2;
            create_info.p_queue_family_indices = [
                physical_device.queue_families.graphics_family.unwrap(),
                physical_device.queue_families.present_family.unwrap(),
            ]
            .as_ptr()
        } else {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
            create_info.queue_family_index_count = 0;
            create_info.p_queue_family_indices = std::ptr::null();
        }

        let loader = khr::swapchain::Device::new(&instance.instance, &device.device);
        let instance = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(|e| format!("Failed to create swapchain: {}", e))?
        };

        let images = unsafe {
            loader
                .get_swapchain_images(instance)
                .map_err(|e| format!("Failed to get swapchain images: {}", e))?
        };

        let image_views = VkSwapchain::create_image_views(device, &images, &image_format)?;

        return Ok(VkSwapchain {
            loader,
            instance,
            images,
            image_format,
            extent,
            image_views,

            framebuffers: Vec::new(),

            depth_image: vk::Image::null(),
            depth_image_memory: vk::DeviceMemory::null(),
            depth_image_view: vk::ImageView::null(),
        });
    }

    fn choose_surface_format(
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

    fn choose_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for available_present_mode in available_present_modes {
            if *available_present_mode == vk::PresentModeKHR::MAILBOX {
                return *available_present_mode;
            }
        }

        return vk::PresentModeKHR::FIFO;
    }

    fn choose_extent(window: &Window, capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
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
        device: &VkDevice,
        images: &Vec<vk::Image>,
        format: &vk::Format,
    ) -> Result<Vec<vk::ImageView>, String> {
        let mut swapchain_image_views: Vec<vk::ImageView> = Vec::new();

        for image in images {
            let image_view =
                create_image_view(device, image, *format, vk::ImageAspectFlags::COLOR)?;

            swapchain_image_views.push(image_view);
        }

        return Ok(swapchain_image_views);
    }

    pub fn create_depth_ressources(&mut self, instance: &VkInstance, physical_device: &VkPhysicalDevice, device: &VkDevice) -> Result<(), String> {
        let format = find_depth_format(instance, physical_device)?;

        let (image, memory) = create_image(
            instance,
            physical_device,
            device,
            self.extent.width,
            self.extent.height,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL
        )?;

        let depth_image_view = create_image_view(device, &image, format, vk::ImageAspectFlags::DEPTH)?;

        self.depth_image = image;
        self.depth_image_memory = memory;
        self.depth_image_view = depth_image_view;
        
        return Ok(());
    }

    pub fn create_framebuffers(
        &mut self,
        device: &VkDevice,
        render_pass: &vk::RenderPass,
    ) -> Result<(), String> {
        for swapchain_image_view in &self.image_views {
            let attachments = [*swapchain_image_view, self.depth_image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                render_pass: *render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: self.extent.width,
                height: self.extent.height,
                layers: 1,
                ..Default::default()
            };

            let framebuffer = unsafe {
                device
                    .device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .map_err(|e| format!("Failed to create framebuffer: {}", e))?
            };
            self.framebuffers.push(framebuffer);
        }

        return Ok(());
    }

    pub fn cleanup(&mut self, device: &VkDevice) {
        unsafe {
            for index in 0..self.framebuffers.len() {
                device
                    .device
                    .destroy_framebuffer(self.framebuffers[index], None);
            }

            for index in 0..self.image_views.len() {
                device
                    .device
                    .destroy_image_view(self.image_views[index], None);
            }

            self.loader.destroy_swapchain(self.instance, None);
        }
    }

    pub fn recreate(
        &mut self,
        window: &Window,
        instance: &VkInstance,
        surface: &VkSurface,
        physical_device: &VkPhysicalDevice,
        device: &VkDevice,
        pipeline: &VkPipeline,
    ) {
        let _ = unsafe { device.device.device_wait_idle() };

        self.cleanup(device);
        let mut swapchain =
            VkSwapchain::new(window, instance, surface, physical_device, device).unwrap();
        
        swapchain
            .create_depth_ressources(instance, physical_device, device).unwrap();
        
        swapchain
            .create_framebuffers(device, &pipeline.render_pass)
            .unwrap();

        *self = swapchain;
    }
}

impl VkContext {
    pub fn recreate_swapchain(&mut self, window: &Window) {
        self.swapchain.recreate(
            window,
            &self.instance,
            &self.surface,
            &self.physical_device,
            &self.device,
            &self.pipeline,
        );
    }
}
