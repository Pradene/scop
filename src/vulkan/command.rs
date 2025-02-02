use crate::vulkan::MAX_FRAMES_IN_FLIGHT;
use crate::vulkan::{VkDevice, VkPhysicalDevice};

use ash::vk;
use std::sync::Arc;

pub struct VkCommandPool {
    device: Arc<VkDevice>,
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
}

impl VkCommandPool {
    pub fn new(
        physical_device: &VkPhysicalDevice,
        device: Arc<VkDevice>,
    ) -> Result<VkCommandPool, String> {
        let pool = VkCommandPool::create_pool(&device, &physical_device)?;
        let buffers = VkCommandPool::create_buffers(&device, &pool)?;

        return Ok(VkCommandPool {
            device,
            pool,
            buffers,
        });
    }

    fn create_pool(
        device: &VkDevice,
        physical_device: &VkPhysicalDevice,
    ) -> Result<vk::CommandPool, String> {
        let create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: physical_device.queue_families.graphics_family.unwrap(),
            ..Default::default()
        };

        let command_pool = unsafe {
            device
                .device
                .create_command_pool(&create_info, None)
                .map_err(|e| format!("Failed to create command pool: {}", e))?
        };

        return Ok(command_pool);
    }

    fn create_buffers(
        device: &VkDevice,
        command_pool: &vk::CommandPool,
    ) -> Result<Vec<vk::CommandBuffer>, String> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: *command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MAX_FRAMES_IN_FLIGHT,
            ..Default::default()
        };

        let command_buffer = unsafe {
            device
                .device
                .allocate_command_buffers(&allocate_info)
                .map_err(|e| format!("Failed to allocate command buffers: {}", e))?
        };

        return Ok(command_buffer);
    }
}

impl Drop for VkCommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .free_command_buffers(self.pool, &self.buffers);
            self.device.device.destroy_command_pool(self.pool, None);
        }
    }
}
