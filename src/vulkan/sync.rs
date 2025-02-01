use crate::vulkan::VkDevice;
use crate::vulkan::MAX_FRAMES_IN_FLIGHT;

use ash::vk;

pub struct VkSyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
}

impl VkSyncObjects {
    pub fn new(device: &VkDevice) -> Result<VkSyncObjects, String> {
        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let capacity = MAX_FRAMES_IN_FLIGHT as usize;
        let mut image_available_semaphores = Vec::with_capacity(capacity);
        let mut render_finished_semaphores = Vec::with_capacity(capacity);
        let mut in_flight_fences = Vec::with_capacity(capacity);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let image_semaphore = unsafe {
                device
                    .device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| format!("Failed to create semaphore: {}", e))?
            };
            let render_semaphore = unsafe {
                device
                    .device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| format!("Failed to create semaphore: {}", e))?
            };
            let fence = unsafe {
                device
                    .device
                    .create_fence(&fence_info, None)
                    .map_err(|e| format!("Failed to create fence: {}", e))?
            };

            image_available_semaphores.push(image_semaphore);
            render_finished_semaphores.push(render_semaphore);
            in_flight_fences.push(fence);
        }

        return Ok(VkSyncObjects {
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
        });
    }
}
