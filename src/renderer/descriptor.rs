use std::sync::Arc;

use ash::vk;

use super::MAX_FRAMES_IN_FLIGHT;
use super::{Uniforms, VkBuffer, VkDevice};

pub struct VkDescriptorPool {
    device: Arc<VkDevice>,
    pub handle: vk::DescriptorPool,
}

impl VkDescriptorPool {
    pub fn new(device: Arc<VkDevice>) -> Result<Self, String> {
        let pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: MAX_FRAMES_IN_FLIGHT,
        };

        let create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            pool_size_count: 1,
            p_pool_sizes: &pool_size,
            max_sets: MAX_FRAMES_IN_FLIGHT,
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

    pub fn create_sets(
        &self,
        set_layout: &VkDescriptorSetLayout,
        uniform_buffers: &Vec<VkBuffer<Uniforms>>,
    ) -> Result<Vec<vk::DescriptorSet>, String> {
        let layouts = vec![set_layout.handle; MAX_FRAMES_IN_FLIGHT as usize];

        let allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: self.handle,
            descriptor_set_count: MAX_FRAMES_IN_FLIGHT,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };

        let descriptor_sets = unsafe {
            self.device
                .handle
                .allocate_descriptor_sets(&allocate_info)
                .map_err(|e| format!("Failed to allocate descriptor sets: {}", e))?
        };

        for index in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info = vk::DescriptorBufferInfo {
                buffer: uniform_buffers[index as usize].handle,
                offset: 0,
                range: std::mem::size_of::<Uniforms>() as u64,
            };

            let descriptor_write = vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                dst_set: descriptor_sets[index as usize],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &buffer_info,
                ..Default::default()
            };

            unsafe {
                self.device
                    .handle
                    .update_descriptor_sets(&[descriptor_write], &[])
            };
        }

        return Ok(descriptor_sets);
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
