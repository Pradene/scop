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
    pub inner: vk::Queue,
}

impl VkQueue {
    pub fn new(device: Arc<VkDevice>, queue_family_index: u32) -> VkQueue {
        let inner = unsafe { device.inner.get_device_queue(queue_family_index, 0) };

        return VkQueue { device, inner };
    }

    pub fn submit(
        &self,
        command_buffer: &vk::CommandBuffer,
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
        wait_stages: &[vk::PipelineStageFlags],
        fence: &vk::Fence,
    ) {
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: command_buffer,
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        unsafe {
            self.device
                .inner
                .queue_submit(self.inner, &[submit_info], *fence)
                .unwrap()
        };
    }
}

impl Drop for VkQueue {
    fn drop(&mut self) {
        unsafe { self.device.inner.queue_wait_idle(self.inner).unwrap() };
    }
}
