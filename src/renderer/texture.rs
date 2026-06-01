use ash::vk;
use std::sync::Arc;

use crate::renderer::find_memory_type;

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

        let staging = VkBuffer::<u8>::device_local(
            context,
            queue,
            command_pool,
            pixels,
            vk::BufferUsageFlags::TRANSFER_SRC,
        )?;

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

        transition_image_layout(
            context,
            queue,
            command_pool,
            handle,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;

        copy_buffer_to_image(
            context,
            queue,
            command_pool,
            staging.handle,
            handle,
            width,
            height,
        )?;

        transition_image_layout(
            context,
            queue,
            command_pool,
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

fn transition_image_layout(
    context: &VkContext,
    queue: &VkQueue,
    command_pool: &VkCommandPool,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> Result<(), String> {
    let cmd = command_pool.begin_single_cmd(context.device())?;

    let (src_access, dst_access, src_stage, dst_stage) = match (old_layout, new_layout) {
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
        _ => return Err("Unsupported layout transition".to_string()),
    };

    let barrier = vk::ImageMemoryBarrier {
        s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
        old_layout,
        new_layout,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        src_access_mask: src_access,
        dst_access_mask: dst_access,
        ..Default::default()
    };

    unsafe {
        context.device().handle.cmd_pipeline_barrier(
            cmd,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }

    command_pool.end_single_cmd(context.device(), queue, cmd)
}

fn copy_buffer_to_image(
    context: &VkContext,
    queue: &VkQueue,
    command_pool: &VkCommandPool,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let cmd = command_pool.begin_single_cmd(context.device())?;

    unsafe {
        context.device().handle.cmd_copy_buffer_to_image(
            cmd,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[vk::BufferImageCopy {
                buffer_offset: 0,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                image_extent: vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                },
            }],
        );
    }

    command_pool.end_single_cmd(context.device(), queue, cmd)
}
