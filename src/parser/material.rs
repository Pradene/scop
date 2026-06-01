use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::math::Vec3;
use crate::scene::RawMaterial;

pub struct MaterialParser;

impl MaterialParser {
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<HashMap<String, RawMaterial>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open MTL: {}", e))?;
        let reader = BufReader::new(file);

        let mut materials = HashMap::new();
        let mut current = RawMaterial::default();

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Error reading file: {}", e))?;
            Self::parse_line(&line, &mut current, &mut materials)?;
        }

        if !current.name.is_empty() {
            materials.insert(current.name.clone(), current);
        }

        Ok(materials)
    }

    fn parse_line(
        line: &str,
        current: &mut RawMaterial,
        materials: &mut HashMap<String, RawMaterial>,
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
                if !current.name.is_empty() {
                    materials.insert(current.name.clone(), current.clone());
                }
                *current = RawMaterial::default();
                current.name = rem.join(" ");
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
