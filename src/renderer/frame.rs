use super::{
    Uniforms, VkBuffer, VkCommandPool, VkContext, VkDescriptorPool, VkDescriptorSetLayout, VkFence,
    VkSemaphore,
};
use crate::camera::Camera;
use ash::vk;

pub struct FrameData {
    pub image_available: VkSemaphore,
    pub render_finished: VkSemaphore,
    pub in_flight: VkFence,
    pub command_buffer: vk::CommandBuffer,
    pub uniform_buffer: VkBuffer<Uniforms>,
    pub descriptor_set: vk::DescriptorSet,
}

impl FrameData {
    pub fn new(
        context: &VkContext,
        command_pool: &VkCommandPool,
        descriptor_pool: &VkDescriptorPool,
        descriptor_set_layout: &VkDescriptorSetLayout,
    ) -> Result<Self, String> {
        let uniform_buffer =
            VkBuffer::host_visible(context, 1, vk::BufferUsageFlags::UNIFORM_BUFFER)?;

        let descriptor_set =
            descriptor_pool.create_set(descriptor_set_layout, &uniform_buffer)?;

        let command_buffer = command_pool
            .allocate_buffers(vk::CommandBufferLevel::PRIMARY, 1)?
            .remove(0);

        let image_available = VkSemaphore::new(context.device())?;
        let render_finished = VkSemaphore::new(context.device())?;
        let in_flight = VkFence::new(context.device())?;

        Ok(Self {
            image_available,
            render_finished,
            in_flight,
            command_buffer,
            uniform_buffer,
            descriptor_set,
        })
    }

    pub fn update_uniforms(&self, camera: &Camera) {
        self.uniform_buffer.write(&[Uniforms {
            view: camera.get_view_matrix(),
            proj: camera.get_projection_matrix(),
        }]);
    }
}
