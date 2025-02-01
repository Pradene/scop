use ash::vk::{self, ImageSubresourceRange};

use crate::vulkan::VkDevice;

use super::{VkBuffer, VkInstance, VkPhysicalDevice};

pub fn create_image(
    instance: &VkInstance,
    physical_device: &VkPhysicalDevice,
    device: &VkDevice,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory), String> {
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

    let image = unsafe {
        device
            .device
            .create_image(&create_info, None)
            .map_err(|e| format!("Failed to create image: {}", e))?
    };

    let memory_requirements = unsafe { device.device.get_image_memory_requirements(image) };
    let memory_type = VkBuffer::find_memory_type(
        instance,
        physical_device,
        memory_requirements.memory_type_bits,
        properties,
    )?;

    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        allocation_size: memory_requirements.size,
        memory_type_index: memory_type,
        ..Default::default()
    };

    let memory = unsafe {
        device
            .device
            .allocate_memory(&allocate_info, None)
            .map_err(|e| format!("Failed to allocate image memory: {}", e))?
    };

    let _ = unsafe {
        device
            .device
            .bind_image_memory(image, memory, 0)
            .map_err(|e| format!("Failed to bind memory to image: {}", e))
    };

    return Ok((image, memory));
}

pub fn create_image_view(
    device: &VkDevice,
    image: &vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
) -> Result<vk::ImageView, String> {
    let create_info = vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        image: *image,
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        subresource_range: ImageSubresourceRange {
            aspect_mask: aspect_flags,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };

    let image_view = unsafe {
        device
            .device
            .create_image_view(&create_info, None)
            .map_err(|e| format!("Failed to create image view: {}", e))?
    };

    return Ok(image_view);
}
