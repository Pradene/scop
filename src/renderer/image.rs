use ash::vk;
use std::sync::Arc;

use super::find_memory_type;
use super::{VkContext, VkDevice};

pub struct VkImage {
    device: Arc<VkDevice>,
    pub inner: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub format: vk::Format,
}

impl VkImage {
    pub fn new(
        context: &VkContext,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
        aspect_flags: vk::ImageAspectFlags,
    ) -> Result<Self, String> {
        let device = context.device();

        let create_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            image_type: vk::ImageType::TYPE_2D,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            format,
            tiling,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage,
            samples: vk::SampleCountFlags::TYPE_1,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let inner = unsafe {
            device
                .inner
                .create_image(&create_info, None)
                .map_err(|e| format!("Failed to create image: {}", e))?
        };

        let memory_requirements = unsafe { device.inner.get_image_memory_requirements(inner) };
        let memory_type =
            find_memory_type(context, memory_requirements.memory_type_bits, properties)?;

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index: memory_type,
            ..Default::default()
        };

        let memory = unsafe {
            device
                .inner
                .allocate_memory(&allocate_info, None)
                .map_err(|e| format!("Failed to allocate image memory: {}", e))?
        };

        unsafe {
            device
                .inner
                .bind_image_memory(inner, memory, 0)
                .map_err(|e| format!("Failed to bind memory to image: {}", e))?
        };

        let view_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            image: inner,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: aspect_flags,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };

        let view = unsafe {
            device
                .inner
                .create_image_view(&view_create_info, None)
                .map_err(|e| format!("Failed to create image view: {}", e))?
        };

        Ok(Self {
            device,
            inner,
            memory,
            view,
            format,
        })
    }
}

impl Drop for VkImage {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_image_view(self.view, None);
            self.device.inner.free_memory(self.memory, None);
            self.device.inner.destroy_image(self.inner, None);
        }
    }
}
