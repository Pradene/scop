use crate::{
    math::Mat4,
    renderer::{MaterialHandle, Vertex, VkBuffer},
};

pub struct SubMesh {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: i32,
    pub material: MaterialHandle,
}

pub struct Mesh {
    pub vertex_buffer: VkBuffer<Vertex>,
    pub index_buffer: VkBuffer<u32>,
    pub primitives: Vec<SubMesh>,
    pub transform: Mat4,
}

impl Mesh {
    pub fn update_transform(&mut self, transform: Mat4) {
        self.transform = transform;
    }
}

pub struct MeshPushConstants {
    pub transform: Mat4,
}
