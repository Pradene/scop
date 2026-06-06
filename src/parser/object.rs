use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::math::{Vec2, Vec3};
use crate::renderer::Vertex;

use super::{Material, MtlFileParser};

pub struct Primitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Option<String>,
}

pub struct Mesh {
    pub submeshes: Vec<Primitive>,
    pub materials: HashMap<String, Material>,
}

pub struct ObjFileParser;

impl ObjFileParser {
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Mesh, String> {
        let path_ref = path.as_ref();
        let base_dir = path_ref.parent().unwrap_or(Path::new(""));

        let file = File::open(path_ref).map_err(|e| format!("Failed to open OBJ: {}", e))?;
        let reader = BufReader::new(file);

        let mut positions: Vec<Vec3> = Vec::new();
        let mut normals: Vec<Vec3> = Vec::new();
        let mut texcoords: Vec<Vec2> = Vec::new();
        let mut materials: HashMap<String, Material> = HashMap::new();

        let mut submeshes: Vec<Primitive> = Vec::new();
        let mut cur_verts: Vec<Vertex> = Vec::new();
        let mut cur_indices: Vec<u32> = Vec::new();
        let mut cur_index_map: HashMap<(usize, Option<usize>, Option<usize>), u32> = HashMap::new();
        let mut cur_material: Option<String> = None;

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
                    positions.push(Self::to_vec3(remainder).ok_or("Invalid vertex coordinates")?)
                }
                "vn" => normals.push(Self::to_vec3(remainder).ok_or("Invalid normal coordinates")?),
                "vt" => {
                    texcoords.push(Self::to_vec2(remainder).ok_or("Invalid texture coordinates")?)
                }
                "usemtl" => {
                    if !cur_indices.is_empty() {
                        submeshes.push(Primitive {
                            vertices: std::mem::take(&mut cur_verts),
                            indices: std::mem::take(&mut cur_indices),
                            material: cur_material.clone(),
                        });
                        cur_index_map.clear();
                    }
                    cur_material = if remainder.is_empty() {
                        None
                    } else {
                        Some(remainder.join(" "))
                    };
                }
                "f" => {
                    if remainder.len() < 3 {
                        return Err("Face needs at least 3 vertices".to_string());
                    }

                    let parse_fv =
                        |s: &str| -> Result<(usize, Option<usize>, Option<usize>), String> {
                            let idx: Vec<&str> = s.split('/').collect();

                            let v = idx
                                .get(0)
                                .and_then(|s| Self::to_usize(s))
                                .ok_or("Missing or invalid vertex index")?;
                            if v >= positions.len() {
                                return Err(format!("Vertex index {} out of bounds", v + 1));
                            }

                            let parse_sub = |i: usize| -> Option<usize> {
                                idx.get(i)
                                    .filter(|s| !s.is_empty())
                                    .and_then(|s| Self::to_usize(s))
                            };

                            let t = parse_sub(1);
                            if let Some(t) = t {
                                if t >= texcoords.len() {
                                    return Err(format!("Texture index {} out of bounds", t + 1));
                                }
                            }

                            let n = parse_sub(2);
                            if let Some(n) = n {
                                if n >= normals.len() {
                                    return Err(format!("Normal index {} out of bounds", n + 1));
                                }
                            }

                            Ok((v, t, n))
                        };

                    let first = parse_fv(remainder[0])?;
                    let mut prev = parse_fv(remainder[1])?;

                    for token in &remainder[2..] {
                        let current = parse_fv(token)?;

                        let normal = compute_normal(positions[first.0], positions[prev.0], positions[current.0]);

                        for (vi, ti, ni) in [first, prev, current] {
                            let idx = *cur_index_map.entry((vi, ti, ni)).or_insert_with(|| {
                                let i = cur_verts.len() as u32;
                                cur_verts.push(Vertex {
                                    position: positions[vi],
                                    normal: ni
                                        .and_then(|n| normals.get(n))
                                        .copied()
                                        .unwrap_or(normal),
                                    uv: ti
                                        .and_then(|t| texcoords.get(t))
                                        .map(|v| Vec2::new(v.x, v.y))
                                        .unwrap_or_default(),
                                });
                                i
                            });
                            cur_indices.push(idx);
                        }

                        prev = current;
                    }
                }
                "mtllib" => {
                    if !remainder.is_empty() {
                        let parsed = MtlFileParser::parse(base_dir.join(remainder.join(" ")))?;
                        materials.extend(parsed);
                    }
                }
                _ => {}
            }
        }

        if !cur_indices.is_empty() {
            submeshes.push(Primitive {
                vertices: cur_verts,
                indices: cur_indices,
                material: cur_material,
            });
        }

        let total: usize = submeshes.iter().map(|s| s.vertices.len()).sum();
        if total > 0 {
            let center = submeshes
                .iter()
                .flat_map(|s| s.vertices.iter().map(|v| v.position))
                .fold(Vec3::ZERO, |acc, p| acc + p)
                / total as f32;

            for sm in &mut submeshes {
                for v in &mut sm.vertices {
                    v.position -= center;
                }
            }
        }

        Ok(Mesh {
            submeshes,
            materials,
        })
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
        Some(Vec3::new(
            Self::to_f32(tokens[0])?,
            Self::to_f32(tokens[1])?,
            Self::to_f32(tokens[2])?,
        ))
    }

    fn to_vec2(tokens: &[&str]) -> Option<Vec2> {
        if tokens.len() != 2 {
            return None;
        }
        Some(Vec2::new(
            Self::to_f32(tokens[0])?,
            Self::to_f32(tokens[1])?,
        ))
    }
}

fn compute_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let edge1 = b - a;
    let edge2 = c - a;
    edge1.cross(edge2)
}