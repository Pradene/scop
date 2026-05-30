use ash::vk;
use std::sync::Arc;

use crate::renderer::VkTexture;

use super::{Uniforms, VkBuffer, VkDevice};

pub struct VkDescriptorPool {
    device: Arc<VkDevice>,
    pub handle: vk::DescriptorPool,
}

impl VkDescriptorPool {
    pub fn new(device: Arc<VkDevice>, max_sets: u32) -> Result<Self, String> {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: max_sets,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                // 3 texture slots (diffuse, specular, ambient) per set
                descriptor_count: max_sets * 3,
            },
        ];

        let create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
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

    pub fn update_textures(
        &self,
        set: vk::DescriptorSet,
        diffuse: &VkTexture,
        specular: &VkTexture,
        ambient: &VkTexture,
    ) {
        let image_infos = [
            vk::DescriptorImageInfo {
                sampler: diffuse.sampler,
                image_view: diffuse.view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            vk::DescriptorImageInfo {
                sampler: specular.sampler,
                image_view: specular.view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            vk::DescriptorImageInfo {
                sampler: ambient.sampler,
                image_view: ambient.view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
        ];

        let writes: Vec<vk::WriteDescriptorSet> = image_infos
            .iter()
            .enumerate()
            .map(|(i, info)| vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                dst_set: set,
                dst_binding: (i + 1) as u32, // bindings 1, 2, 3
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                p_image_info: info,
                ..Default::default()
            })
            .collect();

        unsafe {
            self.device.handle.update_descriptor_sets(&writes, &[]);
        }
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
        let bindings = [
            // binding 0: UBO (view/proj matrices)
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                p_immutable_samplers: std::ptr::null(),
                ..Default::default()
            },
            // binding 1: diffuse texture
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: std::ptr::null(),
                ..Default::default()
            },
            // binding 2: specular texture
            vk::DescriptorSetLayoutBinding {
                binding: 2,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: std::ptr::null(),
                ..Default::default()
            },
            // binding 3: ambient texture
            vk::DescriptorSetLayoutBinding {
                binding: 3,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: std::ptr::null(),
                ..Default::default()
            },
        ];

        let create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
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
