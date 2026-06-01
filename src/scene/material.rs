use crate::{math::Vec3, renderer::TextureHandle};

#[derive(Debug, Clone)]
pub struct Material {
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

impl Default for Material {
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

#[derive(Debug, Default, Clone)]
pub struct RawMaterial {
    pub name: String,
    pub ka: Option<Vec3>,
    pub kd: Option<Vec3>,
    pub ks: Option<Vec3>,
    pub ns: Option<f32>,
    pub ni: Option<f32>,
    pub dissolve: Option<f32>,
    pub illum: Option<i32>,
    pub map_ka: Option<String>,
    pub map_kd: Option<String>,
    pub map_ks: Option<String>,
}
