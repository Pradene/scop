use ash::vk;
use crate::math::{Vec3, Vec2};

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
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
        let position_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: std::mem::offset_of!(Vertex, position) as u32,
        };

        let normal_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: std::mem::offset_of!(Vertex, normal) as u32,
        };

        let uv_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::R32G32_SFLOAT, // Note: R32G32 (2 floats)
            offset: std::mem::offset_of!(Vertex, uv) as u32,
        };

        return [position_attribute, normal_attribute, uv_attribute];
    }
}
