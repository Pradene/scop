use ash::{khr, vk};
use std::sync::Arc;

use super::{create_image, create_image_view, find_depth_format};
use super::{VkDevice, VkQueue, VkRenderPass, VkContext};

pub struct VkSwapchain {
    device: Arc<VkDevice>,
    pub loader: khr::swapchain::Device,
    pub inner: vk::SwapchainKHR,
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
        context: &VkContext,
        render_pass: &VkRenderPass,
        capabilities: vk::SurfaceCapabilitiesKHR,
        surface_format: vk::SurfaceFormatKHR,
        present_mode: vk::PresentModeKHR,
        extent: vk::Extent2D,
    ) -> Result<VkSwapchain, String> {
        Self::create_swapchain_internal(
            context,
            render_pass,
            capabilities,
            surface_format,
            present_mode,
            extent,
            vk::SwapchainKHR::null(),
        )
    }

    pub fn resize(
        &mut self,
        context: &VkContext,
        render_pass: &VkRenderPass,
        capabilities: vk::SurfaceCapabilitiesKHR,
        surface_format: vk::SurfaceFormatKHR,
        present_mode: vk::PresentModeKHR,
        extent: vk::Extent2D,
    ) -> Result<(), String> {
        self.device.wait_idle();

        let old_handle = self.inner;

        let new_swapchain = Self::create_swapchain_internal(
            context,
            render_pass,
            capabilities,
            surface_format,
            present_mode,
            extent,
            old_handle,
        )?;

        let mut old_swapchain = std::mem::replace(self, new_swapchain);
        old_swapchain.inner = vk::SwapchainKHR::null();
        old_swapchain.destroy();

        Ok(())
    }

    fn create_swapchain_internal(
        context: &VkContext,
        render_pass: &VkRenderPass,
        capabilities: vk::SurfaceCapabilitiesKHR,
        surface_format: vk::SurfaceFormatKHR,
        present_mode: vk::PresentModeKHR,
        extent: vk::Extent2D,
        old_swapchain: vk::SwapchainKHR,
    ) -> Result<VkSwapchain, String> {
        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        let image_format = surface_format.format;
        let mut create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: context.surface.inner,
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
            old_swapchain,
            ..Default::default()
        };

        let graphics_family = context.graphics_family();
        let present_family = context.present_family();
        let queue_family_indices = [graphics_family, present_family];

        if graphics_family != present_family {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.queue_family_index_count = 2;
            create_info.p_queue_family_indices = queue_family_indices.as_ptr();
        } else {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        }

        let loader = khr::swapchain::Device::new(&context.instance.inner, &context.device().inner);
        let inner = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .map_err(|e| format!("Failed to create swapchain: {}", e))?
        };

        let images = unsafe {
            loader
                .get_swapchain_images(inner)
                .map_err(|e| format!("Failed to get swapchain images: {}", e))?
        };

        let image_views = VkSwapchain::create_image_views(&context.device(), &images, &image_format)?;
        let format = find_depth_format(&context.instance, &context.physical_device)?;

        let (depth_image, depth_image_memory) = create_image(
            context,
            extent.width,
            extent.height,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let depth_image_view = create_image_view(&context.device(), &depth_image, format, vk::ImageAspectFlags::DEPTH)?;

        let mut framebuffers = Vec::new();
        for image_view in &image_views {
            let attachments = [*image_view, depth_image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                render_pass: render_pass.inner,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: extent.width,
                height: extent.height,
                layers: 1,
                ..Default::default()
            };

            let framebuffer = unsafe {
                context.device()
                    .inner
                    .create_framebuffer(&framebuffer_create_info, None)
                    .map_err(|e| format!("Failed to create framebuffer: {}", e))?
            };

            framebuffers.push(framebuffer);
        }

        Ok(VkSwapchain {
            device: context.device(),
            loader,
            inner,
            images,
            image_format,
            extent,
            image_views,
            framebuffers,
            depth_image,
            depth_image_memory,
            depth_image_view,
        })
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
                    .inner
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
        let swapchains = [self.inner];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: signal_semaphores.len() as u32,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: std::ptr::null_mut(),
            ..Default::default()
        };

        let _ = unsafe {
            self.loader
                .queue_present(queue.inner, &present_info)
                .unwrap()
        };
    }

    pub fn destroy(&mut self) {
        unsafe {
            for framebuffer in self.framebuffers.drain(..) {
                self.device.inner.destroy_framebuffer(framebuffer, None);
            }

            for image_view in self.image_views.drain(..) {
                self.device.inner.destroy_image_view(image_view, None);
            }

            if self.depth_image_view != vk::ImageView::null() {
                self.device.inner.destroy_image_view(self.depth_image_view, None);
                self.depth_image_view = vk::ImageView::null();
            }

            if self.depth_image != vk::Image::null() {
                self.device.inner.destroy_image(self.depth_image, None);
                self.depth_image = vk::Image::null();
            }

            if self.depth_image_memory != vk::DeviceMemory::null() {
                self.device.inner.free_memory(self.depth_image_memory, None);
                self.depth_image_memory = vk::DeviceMemory::null();
            }

            if self.inner != vk::SwapchainKHR::null() {
                self.loader.destroy_swapchain(self.inner, None);
                self.inner = vk::SwapchainKHR::null();
            }
        }
    }
}

impl Drop for VkSwapchain {
    fn drop(&mut self) {
        self.destroy();
    }
}
