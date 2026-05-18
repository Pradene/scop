use std::sync::Arc;
use ash::vk;

use super::{Uniforms, VkBuffer, VkDevice};

pub struct VkDescriptorPool {
    device: Arc<VkDevice>,
    pub handle: vk::DescriptorPool,
}

impl VkDescriptorPool {
    pub fn new(device: Arc<VkDevice>, max_sets: u32) -> Result<Self, String> {
        let pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: max_sets,
        };

        let create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            pool_size_count: 1,
            p_pool_sizes: &pool_size,
            max_sets,
            ..Default::default()
        };

        let handle = unsafe {
            device
                .handle
                .create_descriptor_pool(&create_info, None)
                .map_err(|e| format!("Failed to create descriptor pool: {}", e))?
        };

        return Ok(VkDescriptorPool { device, handle });
    }

    pub fn create_set(
        &self,
        layout: &VkDescriptorSetLayout,
        uniform_buffer: &VkBuffer<Uniforms>,
    ) -> Result<vk::DescriptorSet, String> {
        let allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: self.handle,
            descriptor_set_count: 1,
            p_set_layouts: &layout.handle,
            ..Default::default()
        };

        let set = unsafe {
            self.device
                .handle
                .allocate_descriptor_sets(&allocate_info)
                .map_err(|e| format!("Failed to allocate descriptor set: {}", e))?
                .remove(0)
        };

        let buffer_info = vk::DescriptorBufferInfo {
            buffer: uniform_buffer.handle,
            offset: 0,
            range: std::mem::size_of::<Uniforms>() as u64,
        };

        let write = vk::WriteDescriptorSet {
            s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: set,
            dst_binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            p_buffer_info: &buffer_info,
            ..Default::default()
        };

        unsafe { self.device.handle.update_descriptor_sets(&[write], &[]) };

        Ok(set)
    }
}

impl Drop for VkDescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle
                .destroy_descriptor_pool(self.handle, None);
        }
    }
}

pub struct VkDescriptorSetLayout {
    device: Arc<VkDevice>,
    pub handle: vk::DescriptorSetLayout,
}

impl VkDescriptorSetLayout {
    pub fn new(device: Arc<VkDevice>) -> Result<VkDescriptorSetLayout, String> {
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
            ..Default::default()
        };

        let create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            binding_count: 1,
            p_bindings: &ubo_layout_binding,
            ..Default::default()
        };

        let handle = unsafe {
            device
                .handle
                .create_descriptor_set_layout(&create_info, None)
                .map_err(|e| format!("Failed to create descriptor set layout: {}", e))?
        };

        return Ok(VkDescriptorSetLayout { device, handle });
    }
}

impl Drop for VkDescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle
                .destroy_descriptor_set_layout(self.handle, None);
        }
    }
}
