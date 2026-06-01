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
}

impl Drop for VkCommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_command_pool(self.handle, None);
        }
    }
}
