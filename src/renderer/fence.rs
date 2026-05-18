use ash::vk;
use std::sync::Arc;

use super::VkDevice;

pub struct VkFence {
    device: Arc<VkDevice>,
    pub handle: vk::Fence,
}

impl VkFence {
    pub fn new(device: Arc<VkDevice>) -> Result<VkFence, String> {
        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let handle = unsafe {
            device
                .handle
                .create_fence(&fence_info, None)
                .map_err(|e| format!("Failed to create fence: {}", e))?
        };

        return Ok(VkFence { device, handle });
    }
}

impl Drop for VkFence {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_fence(self.handle, None);
        }
    }
}
