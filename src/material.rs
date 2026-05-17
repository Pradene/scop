use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use crate::math::Vec3;

#[derive(Debug, Default, Clone)]
pub struct Material {
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

#[repr(C)]
pub struct MaterialPushConstants {
    pub ambient: Vec3,
    pub dissolve: f32,
    pub diffuse: Vec3,
    pub shininess: f32,
    pub specular: Vec3,
    pub optical_density: f32,
    pub illum: i32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

impl MaterialPushConstants {
    pub fn from_material(mat: &Material) -> Self {
        Self {
            ambient: mat.ka.unwrap_or(Vec3::new(0.1, 0.1, 0.1)),
            dissolve: mat.dissolve.unwrap_or(1.0),
            diffuse: mat.kd.unwrap_or(Vec3::new(0.7, 0.7, 0.7)),
            shininess: mat.ns.unwrap_or(32.0),
            specular: mat.ks.unwrap_or(Vec3::new(1.0, 1.0, 1.0)),
            optical_density: mat.ni.unwrap_or(1.0),
            illum: mat.illum.unwrap_or(2),
            _pad1: 0.0,
            _pad2: 0.0,
            _pad3: 0.0,
        }
    }
}
