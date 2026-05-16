use ash::vk;
use crate::math::{Mat4, Vec3, Vec4};

pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec4,
}

impl Vertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        return vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        };
    }

    pub fn get_attribute_description() -> [vk::VertexInputAttributeDescription; 3] {
        let base = std::ptr::null::<Vertex>();
        let position_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: unsafe { &(*base).position as *const _ as u32 },
        };

        let normal_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: unsafe { &(*base).normal as *const _ as u32 },
        };

        let color_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: unsafe { &(*base).color as *const _ as u32 },
        };

        return [position_attribute, normal_attribute, color_attribute];
    }
}
