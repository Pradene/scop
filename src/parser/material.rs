use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::math::Vec3;

#[derive(Debug, Default, Clone)]
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
            "Ka" => {
                current.ka =
                    Self::to_vec3(rem).ok_or_else(|| "Invalid Ambient Color (Ka)".to_string())?;
            }
            "Kd" => {
                current.kd =
                    Self::to_vec3(rem).ok_or_else(|| "Invalid Diffuse Color (Kd)".to_string())?;
            }
            "Ks" => {
                current.ks =
                    Self::to_vec3(rem).ok_or_else(|| "Invalid Specular Color (Ks)".to_string())?;
            }

            "Ns" => {
                current.ns = rem
                    .get(0)
                    .and_then(|s| Self::to_f32(s))
                    .ok_or_else(|| "Invalid Specular Exponent (Ns)".to_string())?;
            }
            "Ni" => {
                current.ni = rem
                    .get(0)
                    .and_then(|s| Self::to_f32(s))
                    .ok_or_else(|| "Invalid Optical Density (Ni)".to_string())?;
            }
            "d" => {
                current.dissolve = rem
                    .get(0)
                    .and_then(|s| Self::to_f32(s))
                    .ok_or_else(|| "Invalid Dissolve (d)".to_string())?;
            }

            "illum" => {
                current.illum = rem
                    .get(0)
                    .and_then(|s| s.parse::<i32>().ok())
                    .ok_or_else(|| "Invalid Illumination Model".to_string())?;
            }

            "map_Ka" => {
                current.map_ka = rem.join(" ");
            }
            "map_Kd" => {
                current.map_kd = rem.join(" ");
            }
            "map_Ks" => {
                current.map_ks = rem.join(" ");
            }
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
