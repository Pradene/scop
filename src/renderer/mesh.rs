use crate::{
    math::{Mat4, Vec3},
    renderer::{MaterialHandle, TextureHandle, Vertex, VkBuffer},
};

#[derive(Debug, Clone)]
pub struct GpuMaterial {
    pub ka: Option<Vec3>,
    pub kd: Option<Vec3>,
    pub ks: Option<Vec3>,
    pub ns: Option<f32>,
    pub ni: Option<f32>,
    pub dissolve: Option<f32>,
    pub illum: Option<i32>,
    pub map_ka: Option<TextureHandle>,
    pub map_kd: Option<TextureHandle>,
    pub map_ks: Option<TextureHandle>,
}

impl Default for GpuMaterial {
    fn default() -> Self {
        Self {
            ka: Some(Vec3::new(0.7, 0.8, 0.6)),
            kd: Some(Vec3::new(0.7, 0.8, 0.6)),
            ks: Some(Vec3::new(0.7, 0.8, 0.6)),
            ns: Some(0.5),
            ni: Some(0.5),
            dissolve: Some(0.5),
            illum: Some(1),
            map_ka: Some(0),
            map_kd: Some(0),
            map_ks: Some(0),
        }
    }
}

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
