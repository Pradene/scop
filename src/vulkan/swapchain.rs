use ash::khr::swapchain;
use ash::{khr, vk};
use std::sync::Arc;

use super::{create_image, create_image_view, find_depth_format};
use super::{VkDevice, VkInstance, VkPhysicalDevice, VkQueue, VkRenderPass, VkSurface};

pub struct VkSwapchain {
    device: Arc<VkDevice>,
    pub loader: khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
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
        instance: &VkInstance,
        surface: &VkSurface,
        physical_device: &VkPhysicalDevice,
        device: Arc<VkDevice>,
        render_pass: &VkRenderPass,
        capabilities: vk::SurfaceCapabilitiesKHR,
        surface_format: vk::SurfaceFormatKHR,
        present_mode: vk::PresentModeKHR,
        extent: vk::Extent2D,
    ) -> Result<VkSwapchain, String> {
        let image_count = std::cmp::min(
            capabilities.max_image_count,
            capabilities.min_image_count + 1,
        )
        .max(capabilities.min_image_count + 1);

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
            pre_transform: capabilities.current_transform,
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
        let swapchain = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(|e| format!("Failed to create swapchain: {}", e))?
        };

        let images = unsafe {
            loader
                .get_swapchain_images(swapchain)
                .map_err(|e| format!("Failed to get swapchain images: {}", e))?
        };

        let image_views = VkSwapchain::create_image_views(&device, &images, &image_format)?;

        let format = find_depth_format(instance, physical_device)?;

        let (depth_image, depth_image_memory) = create_image(
            instance,
            physical_device,
            &device,
            extent.width,
            extent.height,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let depth_image_view =
            create_image_view(&device, &depth_image, format, vk::ImageAspectFlags::DEPTH)?;

        let mut framebuffers = Vec::new();
        for image_view in &image_views {
            let attachments = [*image_view, depth_image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                render_pass: render_pass.render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: extent.width,
                height: extent.height,
                layers: 1,
                ..Default::default()
            };

            let framebuffer = unsafe {
                device
                    .device
                    .create_framebuffer(&framebuffer_create_info, None)
                    .map_err(|e| format!("Failed to create framebuffer: {}", e))?
            };

            framebuffers.push(framebuffer);
        }

        return Ok(VkSwapchain {
            device,
            loader,
            swapchain,
            images,
            image_format,
            extent,
            image_views,

            framebuffers,

            depth_image,
            depth_image_memory,
            depth_image_view,
        });
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

    pub fn present_queue(
        &self,
        queue: &VkQueue,
        signal_semaphores: &[vk::Semaphore],
        image_index: u32,
    ) {
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
            self.loader
                .queue_present(queue.queue, &present_info)
                .unwrap()
        };
    }

    pub fn resize(
        &mut self,
        instance: &VkInstance,
        surface: &VkSurface,
        physical_device: &VkPhysicalDevice,
        device: Arc<VkDevice>,
        render_pass: &VkRenderPass,
        capabilities: vk::SurfaceCapabilitiesKHR,
        surface_format: vk::SurfaceFormatKHR,
        present_mode: vk::PresentModeKHR,
        extent: vk::Extent2D,
    ) {
        let _ = unsafe { self.device.device.device_wait_idle() };

        let swapchain = VkSwapchain::new(
            instance,
            surface,
            physical_device,
            device,
            render_pass,
            capabilities,
            surface_format,
            present_mode,
            extent,
        ).unwrap();

        *self = swapchain;
    }

    pub fn destroy(&mut self) {
        unsafe {
            for index in 0..self.framebuffers.len() {
                self.device
                    .device
                    .destroy_framebuffer(self.framebuffers[index], None);
            }

            for index in 0..self.image_views.len() {
                self.device
                    .device
                    .destroy_image_view(self.image_views[index], None);
            }

            self.device
                .device
                .destroy_image_view(self.depth_image_view, None);
            self.device.device.destroy_image(self.depth_image, None);
            self.device
                .device
                .free_memory(self.depth_image_memory, None);

            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}

impl Drop for VkSwapchain {
    fn drop(&mut self) {
        self.destroy();
    }
}
