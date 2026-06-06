use crate::{
    math::{Mat4, Vec3},
    renderer::{MaterialHandle, ResourcesManager, TextureHandle, Vertex, VkBuffer},
};

#[derive(Debug, Clone)]
pub struct GpuMaterial {
    pub ka: Vec3,
    pub kd: Vec3,
    pub ks: Vec3,
    pub ns: f32,
    pub ni: f32,
    pub dissolve: f32,
    pub illum: i32,
    pub map_ka: TextureHandle,
    pub map_kd: TextureHandle,
    pub map_ks: TextureHandle,
}

impl Default for GpuMaterial {
    fn default() -> Self {
        let white = ResourcesManager::white_texture();

        Self {
            ka: Vec3::new(0.7, 0.8, 0.6),
            kd: Vec3::new(0.7, 0.8, 0.6),
            ks: Vec3::new(0.7, 0.8, 0.6),
            ns: 0.5,
            ni: 0.5,
            dissolve: 0.5,
            illum: 1,
            map_ka: white,
            map_kd: white,
            map_ks: white,
        }
    }
}

#[repr(C)]
pub struct MaterialPushConstants {
    pub ambient: Vec3,
    pub dissolve: f32,
    pub diffuse: Vec3,
    pub shininess: f32,
    pub specular: Vec3,
    pub optical_density: f32,
    pub illum: i32,
    pub tex_diffuse: u32,
    pub tex_ambient: u32,
    pub tex_specular: u32,
}

impl From<&GpuMaterial> for MaterialPushConstants {
    fn from(mat: &GpuMaterial) -> Self {
        Self {
            ambient: mat.ka,
            diffuse: mat.kd,
            specular: mat.ks,
            shininess: mat.ns,
            optical_density: mat.ni,
            dissolve: mat.dissolve,
            illum: mat.illum,
            tex_diffuse: mat.map_kd as u32,
            tex_specular: mat.map_ks as u32,
            tex_ambient: mat.map_ka as u32,
        }
    }
}

pub struct GpuGroup {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: i32,
    pub material: MaterialHandle,
}

pub struct GpuMesh {
    pub vertex_buffer: VkBuffer<Vertex>,
    pub index_buffer: VkBuffer<u32>,
    pub groups: Vec<GpuGroup>,
}

#[repr(C)]
pub struct MeshPushConstants {
    pub transform: Mat4,
}
