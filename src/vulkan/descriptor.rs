use std::sync::Arc;

use ash::vk;

use crate::vulkan::UniformBufferObject;
use crate::vulkan::VkDevice;
use crate::vulkan::MAX_FRAMES_IN_FLIGHT;

pub struct VkDescriptorPool {
    device: Arc<VkDevice>,
    pub inner: vk::DescriptorPool,
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

        let inner = unsafe {
            device
                .device
                .create_descriptor_pool(&create_info, None)
                .map_err(|e| format!("Failed to create descriptor pool: {}", e))?
        };

        return Ok(VkDescriptorPool { device, inner });
    }

    pub fn create_sets(
        &self,
        set_layout: &VkDescriptorSetLayout,
        uniform_buffers: &Vec<vk::Buffer>,
    ) -> Result<Vec<VkDescriptorSet>, String> {
        let layouts = vec![set_layout.inner; MAX_FRAMES_IN_FLIGHT as usize];

        let allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: self.inner,
            descriptor_set_count: MAX_FRAMES_IN_FLIGHT,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };

        let descriptor_sets = unsafe {
            self.device
                .device
                .allocate_descriptor_sets(&allocate_info)
                .map_err(|e| format!("Failed to allocate descriptor sets: {}", e))?
        };

        for index in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info = vk::DescriptorBufferInfo {
                buffer: uniform_buffers[index as usize],
                offset: 0,
                range: std::mem::size_of::<UniformBufferObject>() as u64,
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
                    .device
                    .update_descriptor_sets(&[descriptor_write], &[])
            };
        }

        let sets = descriptor_sets
            .into_iter()
            .map(|inner| VkDescriptorSet {
                device: self.device.clone(),
                inner,
            })
            .collect::<Vec<_>>();

        return Ok(sets);
    }
}

impl Drop for VkDescriptorPool {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_descriptor_pool(self.inner, None);
        }
    }
}

pub struct VkDescriptorSet {
    device: Arc<VkDevice>,
    pub inner: vk::DescriptorSet,
}

pub struct VkDescriptorSetLayout {
    device: Arc<VkDevice>,
    pub inner: vk::DescriptorSetLayout,
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

        let inner = unsafe {
            device
                .device
                .create_descriptor_set_layout(&create_info, None)
                .map_err(|e| format!("Failed to create descriptor set layout: {}", e))?
        };

        return Ok(VkDescriptorSetLayout { device, inner });
    }
}

impl Drop for VkDescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .destroy_descriptor_set_layout(self.inner, None);
        }
    }
}
