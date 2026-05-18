use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::math::{Vec2, Vec3};
use crate::scene::{FaceVertex, Group, Object};

use super::MaterialParser;

pub struct ObjectParser;

impl ObjectParser {
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Object, String> {
        let path_ref = path.as_ref();
        let base_dir = path_ref.parent().unwrap_or(Path::new(""));

        let file = File::open(path_ref).map_err(|e| format!("Failed to open OBJ: {}", e))?;
        let reader = BufReader::new(file);

        let mut object = Object::new();
        let mut current_group = Group::new("default".to_string());
        let mut current_material: Option<String> = None;

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| e.to_string())?;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let remainder = &parts[1..];

            match parts[0] {
                "v" => {
                    let v = Self::to_vec3(remainder).ok_or("Invalid vertex coordinates")?;
                    object.vertices.push(v);
                }
                "vn" => {
                    let v = Self::to_vec3(remainder).ok_or("Invalid normal coordinates")?;
                    object.normals.push(v);
                }
                "vt" => {
                    let v = Self::to_vec2(remainder).ok_or("Invalid texture coordinates")?;
                    object.textures.push(v);
                }
                "g" | "o" => {
                    if !current_group.is_empty() {
                        object.groups.push(current_group);
                    }
                    let name = if remainder.is_empty() {
                        "unnamed".to_string()
                    } else {
                        remainder.join(" ")
                    };
                    current_group = Group::new(name);
                    current_group.material = current_material.clone();
                }
                "f" => Self::parse_face(remainder, &mut current_group, &object)?,
                "mtllib" => {
                    if !remainder.is_empty() {
                        let mtl_filename = remainder.join(" ");
                        let materials = MaterialParser::parse(base_dir.join(mtl_filename))?;
                        object.materials.extend(materials);
                    }
                }
                "usemtl" => {
                    let new_material = if remainder.is_empty() {
                        None
                    } else {
                        Some(remainder.join(" "))
                    };
                    if !current_group.is_empty() {
                        object.groups.push(current_group);
                        let group_name = format!(
                            "{}_{}",
                            remainder.first().unwrap_or(&"mat"),
                            object.groups.len()
                        );
                        current_group = Group::new(group_name);
                    }
                    current_material = new_material;
                    current_group.material = current_material.clone();
                }
                _ => continue,
            }
        }

        if !current_group.is_empty() {
            object.groups.push(current_group);
        }

        object.center = object.compute_center();
        Ok(object)
    }

    fn parse_face(face_tokens: &[&str], group: &mut Group, object: &Object) -> Result<(), String> {
        if face_tokens.len() < 3 {
            return Err("Face needs at least 3 vertices".to_string());
        }
        let mut face = Vec::with_capacity(face_tokens.len());

        for vertex_str in face_tokens {
            let indices: Vec<&str> = vertex_str.split('/').collect();

            let vertex_index = indices
                .get(0)
                .and_then(|s| Self::to_usize(s))
                .ok_or("Missing or invalid vertex index")?;

            if vertex_index >= object.vertices.len() {
                return Err(format!("Vertex index {} out of bounds", vertex_index + 1));
            }

            let parse_sub_idx = |i: usize| -> Option<usize> {
                indices
                    .get(i)
                    .filter(|s| !s.is_empty())
                    .and_then(|s| Self::to_usize(s))
            };

            let texture_index = parse_sub_idx(1);
            if let Some(idx) = texture_index {
                if idx >= object.textures.len() {
                    return Err(format!("Texture index {} out of bounds", idx + 1));
                }
            }

            let normal_index = parse_sub_idx(2);
            if let Some(idx) = normal_index {
                if idx >= object.normals.len() {
                    return Err(format!("Normal index {} out of bounds", idx + 1));
                }
            }

            face.push(FaceVertex {
                vertex: vertex_index,
                texture: texture_index,
                normal: normal_index,
            });
        }

        for triangle in Object::triangulate_face(&face) {
            group.faces.push(triangle);
        }
        Ok(())
    }

    fn to_usize(s: &str) -> Option<usize> {
        s.parse::<usize>().ok()?.checked_sub(1)
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

    fn to_vec2(tokens: &[&str]) -> Option<Vec2> {
        if tokens.len() != 2 {
            return None;
        }
        let x = Self::to_f32(tokens[0])?;
        let y = Self::to_f32(tokens[1])?;
        Some(Vec2::new(x, y))
    }
}
