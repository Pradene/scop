use ash::vk;
use std::sync::Arc;

use super::MAX_FRAMES_IN_FLIGHT;
use super::{VkDevice, VkPhysicalDevice};

pub struct VkCommandPool {
    device: Arc<VkDevice>,
    pub inner: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
}

impl VkCommandPool {
    pub fn new(
        physical_device: &VkPhysicalDevice,
        device: Arc<VkDevice>,
    ) -> Result<VkCommandPool, String> {
        let inner = VkCommandPool::create_pool(&device, &physical_device)?;
        let buffers = VkCommandPool::create_command_buffers(&device, &inner)?;

        return Ok(VkCommandPool {
            device,
            inner,
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
                .inner
                .create_command_pool(&create_info, None)
                .map_err(|e| format!("Failed to create command pool: {}", e))?
        };

        return Ok(command_pool);
    }

    fn create_command_buffers(
        device: &Arc<VkDevice>,
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
                .inner
                .allocate_command_buffers(&allocate_info)
                .map_err(|e| format!("Failed to allocate command buffers: {}", e))?
        };

        return Ok(command_buffer);
    }
}

impl Drop for VkCommandPool {
    fn drop(&mut self) {
        unsafe {
            for buffer in &self.buffers {
                self.device
                    .inner
                    .free_command_buffers(self.inner, &[*buffer]);
            }

            self.device.inner.destroy_command_pool(self.inner, None);
        }
    }
}
