use crate::math::Vec3;

#[derive(Debug, Clone)]
pub struct Material {
    pub ka: Vec3,
    pub kd: Vec3,
    pub ks: Vec3,
    pub ns: f32,
    pub ni: f32,
    pub dissolve: f32,
    pub illum: i32,
    pub map_ka: String,
    pub map_kd: String,
    pub map_ks: String,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ka: Vec3::new(1., 0., 0.),
            kd: Vec3::new(1., 0., 0.),
            ks: Vec3::new(1., 0., 0.),
            ns: 1.,
            ni: 1.,
            dissolve: 1.,
            illum: 1,
            map_ka: String::new(),
            map_kd: String::new(),
            map_ks: String::new(),
        }
    }
}
