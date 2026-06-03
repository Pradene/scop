use ash::vk;
use std::sync::Arc;

use super::{VkDevice, VkQueue};

pub struct VkCommandPool {
    device: Arc<VkDevice>,
    pub handle: vk::CommandPool,
}

impl VkCommandPool {
    pub fn new(
        device: Arc<VkDevice>,
        queue_family_index: u32,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<Self, String> {
        let create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            flags,
            queue_family_index,
            ..Default::default()
        };

        let handle = unsafe {
            device
                .handle
                .create_command_pool(&create_info, None)
                .map_err(|e| format!("Failed to create command pool: {}", e))?
        };

        Ok(VkCommandPool { device, handle })
    }

    pub fn allocate_buffers(
        &self,
        level: vk::CommandBufferLevel,
        count: u32,
    ) -> Result<Vec<vk::CommandBuffer>, String> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: self.handle,
            level,
            command_buffer_count: count,
            ..Default::default()
        };

        unsafe {
            self.device
                .handle
                .allocate_command_buffers(&allocate_info)
                .map_err(|e| format!("Failed to allocate command buffers: {}", e))
        }
    }

    pub unsafe fn free_buffers(&self, buffers: &[vk::CommandBuffer]) {
        self.device
            .handle
            .free_command_buffers(self.handle, buffers);
    }

    pub fn begin_single_cmd(&self) -> Result<vk::CommandBuffer, String> {
        let command_buffer = self
            .allocate_buffers(vk::CommandBufferLevel::PRIMARY, 1)?
            .remove(0);

        unsafe {
            self.device
                .handle
                .begin_command_buffer(
                    command_buffer,
                    &vk::CommandBufferBeginInfo {
                        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                        ..Default::default()
                    },
                )
                .map_err(|e| format!("Failed to begin command buffer: {}", e))?;
        }

        Ok(command_buffer)
    }

    pub fn end_single_cmd(
        &self,
        queue: &VkQueue,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), String> {
        unsafe {
            self.device
                .handle
                .end_command_buffer(command_buffer)
                .map_err(|e| format!("Failed to end command buffer: {}", e))?;

            self.device
                .handle
                .queue_submit(
                    queue.handle,
                    &[vk::SubmitInfo {
                        s_type: vk::StructureType::SUBMIT_INFO,
                        command_buffer_count: 1,
                        p_command_buffers: &command_buffer,
                        ..Default::default()
                    }],
                    vk::Fence::null(),
                )
                .map_err(|e| format!("Failed to submit: {}", e))?;

            self.device
                .handle
                .queue_wait_idle(queue.handle)
                .map_err(|e| format!("Failed to wait idle: {}", e))?;

            self.free_buffers(&[command_buffer]);
        }

        Ok(())
    }

    pub fn copy_buffer_to_image(
        &self,
        queue: &VkQueue,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let cmd = self.begin_single_cmd()?;
        unsafe {
            self.device.handle.cmd_copy_buffer_to_image(
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
        self.end_single_cmd(queue, cmd)
    }

    pub fn transition_image_layout(
        &self,
        queue: &VkQueue,
        image: vk::Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> Result<(), String> {
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

        let cmd = self.begin_single_cmd()?;
        unsafe {
            self.device.handle.cmd_pipeline_barrier(
                cmd,
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier {
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
                }],
            );
        }
        self.end_single_cmd(queue, cmd)
    }
}

impl Drop for VkCommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_command_pool(self.handle, None);
        }
    }
}
