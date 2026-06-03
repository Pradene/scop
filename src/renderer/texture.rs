use ash::vk;
use std::sync::Arc;

use super::{VkBuffer, VkCommandPool, VkContext, VkDevice, VkQueue};

pub struct VkTexture {
    device: Arc<VkDevice>,
    pub handle: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub format: vk::Format,
    pub sampler: vk::Sampler,
}

impl VkTexture {
    pub fn from_path(
        context: &VkContext,
        queue: &VkQueue,
        command_pool: &VkCommandPool,
        path: &str,
    ) -> Result<Self, String> {
        let img = image::open(path)
            .map_err(|e| format!("Failed to open texture '{}': {}", path, e))?
            .to_rgba8();
        let (width, height) = img.dimensions();
        Self::from_rgba8(context, queue, command_pool, img.as_raw(), width, height)
    }

    pub fn white(
        context: &VkContext,
        queue: &VkQueue,
        command_pool: &VkCommandPool,
    ) -> Result<Self, String> {
        let pixels: [u8; 4] = [255, 255, 255, 255];
        Self::from_rgba8(context, queue, command_pool, &pixels, 1, 1)
    }

    fn from_rgba8(
        context: &VkContext,
        queue: &VkQueue,
        command_pool: &VkCommandPool,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        let device = context.device();

        let staging = VkBuffer::<u8>::host_visible(
            context,
            pixels.len(),
            vk::BufferUsageFlags::TRANSFER_SRC,
        )?;

        staging.write(pixels);

        let format = vk::Format::R8G8B8A8_SRGB;
        let tiling = vk::ImageTiling::OPTIMAL;
        let usage = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED;
        let properties = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let aspect_mask = vk::ImageAspectFlags::COLOR;

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

        let handle = unsafe {
            device
                .handle
                .create_image(&create_info, None)
                .map_err(|e| format!("Failed to create image: {}", e))?
        };

        let memory_requirements = unsafe { device.handle.get_image_memory_requirements(handle) };
        let memory_type = context
            .physical_device
            .find_memory_type(memory_requirements.memory_type_bits, properties)?;

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index: memory_type,
            ..Default::default()
        };

        let memory = unsafe {
            device
                .handle
                .allocate_memory(&allocate_info, None)
                .map_err(|e| format!("Failed to allocate image memory: {}", e))?
        };

        unsafe {
            device
                .handle
                .bind_image_memory(handle, memory, 0)
                .map_err(|e| format!("Failed to bind memory to image: {}", e))?
        };

        let view_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            image: handle,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };

        let view = unsafe {
            device
                .handle
                .create_image_view(&view_create_info, None)
                .map_err(|e| format!("Failed to create image view: {}", e))?
        };

        command_pool.transition_image_layout(
            queue,
            handle,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;

        command_pool.copy_buffer_to_image(queue, staging.handle, handle, width, height)?;

        command_pool.transition_image_layout(
            queue,
            handle,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        let sampler_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: vk::FALSE,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
            compare_enable: vk::FALSE,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            ..Default::default()
        };

        let sampler = unsafe {
            device
                .handle
                .create_sampler(&sampler_info, None)
                .map_err(|e| format!("Failed to create sampler: {}", e))?
        };

        Ok(Self {
            device,
            handle,
            memory,
            view,
            format,
            sampler,
        })
    }
}

impl Drop for VkTexture {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_sampler(self.sampler, None);
            self.device.handle.destroy_image_view(self.view, None);
            self.device.handle.free_memory(self.memory, None);
            self.device.handle.destroy_image(self.handle, None);
        }
    }
}
