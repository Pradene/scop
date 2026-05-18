use crate::renderer::VkDevice;
use ash::vk;
use std::sync::Arc;

pub struct VkSemaphore {
    device: Arc<VkDevice>,
    pub handle: vk::Semaphore,
}

impl VkSemaphore {
    pub fn new(device: Arc<VkDevice>) -> Result<VkSemaphore, String> {
        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let handle = unsafe {
            device
                .handle
                .create_semaphore(&semaphore_info, None)
                .map_err(|e| format!("Failed to create semaphore: {}", e))?
        };

        return Ok(VkSemaphore { device, handle });
    }
}

impl Drop for VkSemaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.handle.destroy_semaphore(self.handle, None);
        }
    }
}
