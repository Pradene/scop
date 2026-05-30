use crate::{
    renderer::{TextureHandle, Vertex, VkBuffer},
    scene::Material,
};

pub struct SubMesh {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: i32,
    pub material: Material,
    pub tex_diffuse: TextureHandle,
    pub tex_specular: TextureHandle,
    pub tex_ambient: TextureHandle,
}

pub struct Mesh {
    pub vertex_buffer: VkBuffer<Vertex>,
    pub index_buffer: VkBuffer<u32>,
    pub primitives: Vec<SubMesh>,
}
