use ash::vk;
use std::sync::Arc;

use super::VkDevice;

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
}

impl Drop for VkCommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_command_pool(self.handle, None);
        }
    }
}
