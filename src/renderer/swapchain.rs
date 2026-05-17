use ash::{khr, vk};
use std::sync::Arc;

use super::find_depth_format;
use super::{VkContext, VkDevice, VkRenderPass, VkImage, VkImageView};

pub struct VkSwapchain {
    device: Arc<VkDevice>,
    pub loader: khr::swapchain::Device,
    pub inner: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub depth_image: VkImage,
    pub depth_view: VkImageView,
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
        Self::swapchain_create(
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

        let new_swapchain = Self::swapchain_create(
            context,
            render_pass,
            capabilities,
            surface_format,
            present_mode,
            extent,
            old_handle,
        )?;

        let mut old = std::mem::replace(self, new_swapchain);
        old.inner = vk::SwapchainKHR::null();

        Ok(())
    }

    fn swapchain_create(
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

        let image_views = Self::create_image_views(&context.device(), &images, image_format)?;

        let depth_format = find_depth_format(&context.instance, &context.physical_device)?;
        let depth_image = VkImage::new(
            context,
            extent.width,
            extent.height,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::DEPTH,
        )?;

        let depth_view = VkImageView::new(
            context.device(),
            depth_image.inner,
            depth_format,
            vk::ImageAspectFlags::DEPTH,
        )?;

        let framebuffers = image_views
            .iter()
            .map(|&view| {
                let attachments = [view, depth_view.inner];
                let create_info = vk::FramebufferCreateInfo {
                    s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                    render_pass: render_pass.inner,
                    attachment_count: attachments.len() as u32,
                    p_attachments: attachments.as_ptr(),
                    width: extent.width,
                    height: extent.height,
                    layers: 1,
                    ..Default::default()
                };
                unsafe {
                    context
                        .device()
                        .inner
                        .create_framebuffer(&create_info, None)
                        .map_err(|e| format!("Failed to create framebuffer: {}", e))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

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
            depth_view,
        })
    }

    fn create_image_views(
        device: &Arc<VkDevice>,
        images: &[vk::Image],
        format: vk::Format,
    ) -> Result<Vec<vk::ImageView>, String> {
        images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo {
                    s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                    image,
                    view_type: vk::ImageViewType::TYPE_2D,
                    format,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };
                unsafe {
                    device
                        .inner
                        .create_image_view(&create_info, None)
                        .map_err(|e| format!("Failed to create image view: {}", e))
                }
            })
            .collect()
    }

    pub fn queue_present(
        &self,
        queue: &vk::Queue,
        signal_semaphores: &[vk::Semaphore],
        image_index: u32,
    ) -> Result<bool, String> {
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

        match unsafe { self.loader.queue_present(*queue, &present_info) } {
            Ok(suboptimal) => Ok(suboptimal),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Ok(true),
            Err(e) => Err(format!("Failed to present queue: {}", e)),
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            for framebuffer in self.framebuffers.drain(..) {
                self.device.inner.destroy_framebuffer(framebuffer, None);
            }
            for view in self.image_views.drain(..) {
                self.device.inner.destroy_image_view(view, None);
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
