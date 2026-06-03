use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::math::Vec3;
use crate::renderer::TextureHandle;

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

#[derive(Debug, Default, Clone)]
pub struct Material {
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

pub struct MtlFileParser;

impl MtlFileParser {
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<HashMap<String, Material>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open MTL: {}", e))?;
        let reader = BufReader::new(file);

        let mut materials = HashMap::new();
        let mut current = Material::default();
        let mut name = String::new();

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Error reading file: {}", e))?;
            Self::parse_line(&line, &mut name, &mut current, &mut materials)?;
        }

        if !name.is_empty() {
            materials.insert(name.clone(), current);
        }

        Ok(materials)
    }

    fn parse_line(
        line: &str,
        name: &mut String,
        current: &mut Material,
        materials: &mut HashMap<String, Material>,
    ) -> Result<(), String> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        let tag = *tokens.get(0).ok_or("Empty line token")?;
        let rem = tokens.get(1..).unwrap_or(&[]);

        match tag {
            "newmtl" => {
                if !name.is_empty() {
                    materials.insert(name.clone(), current.clone());
                }
                *current = Material::default();
                *name = rem.join(" ");
            }
            "Ka" => current.ka = Some(Self::to_vec3(rem).ok_or("Invalid Ambient Color (Ka)")?),
            "Kd" => current.kd = Some(Self::to_vec3(rem).ok_or("Invalid Diffuse Color (Kd)")?),
            "Ks" => current.ks = Some(Self::to_vec3(rem).ok_or("Invalid Specular Color (Ks)")?),

            "Ns" => {
                current.ns = Some(
                    rem.get(0)
                        .and_then(|s| Self::to_f32(s))
                        .ok_or("Invalid Specular Exponent (Ns)")?,
                )
            }
            "Ni" => {
                current.ni = Some(
                    rem.get(0)
                        .and_then(|s| Self::to_f32(s))
                        .ok_or("Invalid Optical Density (Ni)")?,
                )
            }
            "d" => {
                current.dissolve = Some(
                    rem.get(0)
                        .and_then(|s| Self::to_f32(s))
                        .ok_or("Invalid Dissolve (d)")?,
                )
            }

            "illum" => {
                current.illum = Some(
                    rem.get(0)
                        .and_then(|s| s.parse::<i32>().ok())
                        .ok_or("Invalid Illumination Model")?,
                )
            }

            "map_Ka" => current.map_ka = Some(rem.join(" ")),
            "map_Kd" => current.map_kd = Some(rem.join(" ")),
            "map_Ks" => current.map_ks = Some(rem.join(" ")),
            _ => {}
        }
        Ok(())
    }

    fn to_f32(s: &str) -> Option<f32> {
        s.parse::<f32>().ok()
    }

    fn to_vec3(tokens: &[&str]) -> Option<Vec3> {
        if tokens.len() < 3 {
            return None;
        }
        let x = Self::to_f32(tokens[0])?;
        let y = Self::to_f32(tokens[1])?;
        let z = Self::to_f32(tokens[2])?;
        Some(Vec3::new(x, y, z))
    }
}
