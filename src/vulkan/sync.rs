use crate::vulkan::VkDevice;

use ash::vk;
use std::sync::Arc;

pub struct VkFence {
    device: Arc<VkDevice>,
    pub fence: vk::Fence,
}

impl VkFence {
    pub fn new(device: Arc<VkDevice>) -> Result<VkFence, String> {
        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let fence = unsafe {
            device
                .device
                .create_fence(&fence_info, None)
                .map_err(|e| format!("Failed to create fence: {}", e))?
        };

        return Ok(VkFence { device, fence });
    }
}

impl Drop for VkFence {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_fence(self.fence, None);
        }
    }
}

pub struct VkSemaphore {
    device: Arc<VkDevice>,
    pub semaphore: vk::Semaphore,
}

impl VkSemaphore {
    pub fn new(device: Arc<VkDevice>) -> Result<VkSemaphore, String> {
        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let semaphore = unsafe {
            device
                .device
                .create_semaphore(&semaphore_info, None)
                .map_err(|e| format!("Failed to create semaphore: {}", e))?
        };

        return Ok(VkSemaphore { device, semaphore });
    }
}

impl Drop for VkSemaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_semaphore(self.semaphore, None);
        }
    }
}
