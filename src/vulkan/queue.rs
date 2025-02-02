use ash::vk;
use std::sync::Arc;

use super::VkDevice;

#[derive(Clone)]
pub struct QueueFamiliesIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

pub struct VkQueue {
    device: Arc<VkDevice>,
    pub queue: vk::Queue,
}

impl VkQueue {
    pub fn new(device: Arc<VkDevice>, queue_family_index: u32) -> VkQueue {
        let queue = unsafe { device.device.get_device_queue(queue_family_index, 0) };

        return VkQueue { device, queue };
    }
}

impl Drop for VkQueue {
    fn drop(&mut self) {
        unsafe { self.device.device.queue_wait_idle(self.queue).unwrap() };
    }
}
