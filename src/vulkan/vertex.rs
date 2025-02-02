use ash::vk;
use lineal::{Matrix, Vector};

pub struct UniformBufferObject {
    pub model: Matrix<f32, 4, 4>,
    pub view: Matrix<f32, 4, 4>,
    pub proj: Matrix<f32, 4, 4>,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vector<f32, 3>,
    pub normal: Vector<f32, 3>,
    pub color: Vector<f32, 3>,
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
