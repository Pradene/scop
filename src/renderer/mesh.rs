use crate::{
    math::Mat4,
    renderer::{MaterialHandle, Vertex, VkBuffer},
};

pub struct GpuPrimitive {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: i32,
    pub material: MaterialHandle,
}

pub struct GpuMesh {
    pub vertex_buffer: VkBuffer<Vertex>,
    pub index_buffer: VkBuffer<u32>,
    pub primitives: Vec<GpuPrimitive>,
    pub transform: Mat4,
}

impl GpuMesh {
    pub fn update_transform(&mut self, transform: Mat4) {
        self.transform = transform;
    }
}

pub struct MeshPushConstants {
    pub transform: Mat4,
}
