use crate::vulkan::VkDevice;

use ash::vk;
use std::sync::Arc;

pub struct VkFence {
    device: Arc<VkDevice>,
    pub inner: vk::Fence,
}

impl VkFence {
    pub fn new(device: Arc<VkDevice>) -> Result<VkFence, String> {
        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let inner = unsafe {
            device
                .inner
                .create_fence(&fence_info, None)
                .map_err(|e| format!("Failed to create fence: {}", e))?
        };

        return Ok(VkFence { device, inner });
    }
}

impl Drop for VkFence {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_fence(self.inner, None);
        }
    }
}

pub struct VkSemaphore {
    device: Arc<VkDevice>,
    pub inner: vk::Semaphore,
}

impl VkSemaphore {
    pub fn new(device: Arc<VkDevice>) -> Result<VkSemaphore, String> {
        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let inner = unsafe {
            device
                .inner
                .create_semaphore(&semaphore_info, None)
                .map_err(|e| format!("Failed to create semaphore: {}", e))?
        };

        return Ok(VkSemaphore { device, inner });
    }
}

impl Drop for VkSemaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_semaphore(self.inner, None);
        }
    }
}
