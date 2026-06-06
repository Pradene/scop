use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::math::{Vec2, Vec3};
use crate::renderer::Vertex;
use crate::scene::{Group, Material, Mesh};

use super::MtlFileParser;

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
        let mut materials_map: HashMap<String, usize> = HashMap::new();
        let mut materials: Vec<Material> = Vec::new();

        let mut groups: Vec<Group> = Vec::new();
        let mut cur_verts: Vec<Vertex> = Vec::new();
        let mut cur_indices: Vec<u32> = Vec::new();
        let mut cur_index_map: HashMap<(usize, Option<usize>, Option<usize>), u32> = HashMap::new();
        let mut cur_material: Option<usize> = None;

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
                        groups.push(Group {
                            vertices: std::mem::take(&mut cur_verts),
                            indices: std::mem::take(&mut cur_indices),
                            material: cur_material.clone(),
                        });
                        cur_index_map.clear();
                    }
                    cur_material = if remainder.is_empty() {
                        None
                    } else {
                        let name = remainder.join(" ");
                        materials_map.get(&name).copied()
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

                        for (vi, ti, ni) in [first, prev, current] {
                            let idx = *cur_index_map.entry((vi, ti, ni)).or_insert_with(|| {
                                let i = cur_verts.len() as u32;
                                cur_verts.push(Vertex {
                                    position: positions[vi],
                                    normal: ni
                                        .and_then(|n| normals.get(n))
                                        .copied()
                                        .unwrap_or(Vec3::ZERO),
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
                        for (name, material) in parsed {
                            materials_map.insert(name, materials.len());
                            materials.push(material);
                        }
                    }
                }
                "o" => {
                    continue;
                }
                _ => {
                    println!("{}", parts[0]);
                }
            }
        }

        if normals.is_empty() {
            for triangle in cur_indices.chunks(3) {
                let (a, b, c) = (
                    triangle[0] as usize,
                    triangle[1] as usize,
                    triangle[2] as usize,
                );
                let n = compute_normal(
                    cur_verts[a].position,
                    cur_verts[b].position,
                    cur_verts[c].position,
                );
                cur_verts[a].normal = n;
                cur_verts[b].normal = n;
                cur_verts[c].normal = n;
            }
            for v in &mut cur_verts {
                v.normal = v.normal.normalize();
            }
        }

        if !cur_indices.is_empty() {
            groups.push(Group {
                vertices: cur_verts,
                indices: cur_indices,
                material: cur_material,
            });
        }

        let vertices_count: usize = groups.iter().map(|s| s.vertices.len()).sum();
        if vertices_count > 0 {
            let center = groups
                .iter()
                .flat_map(|s| s.vertices.iter().map(|v| v.position))
                .fold(Vec3::ZERO, |acc, p| acc + p)
                / vertices_count as f32;

            for sm in &mut groups {
                for v in &mut sm.vertices {
                    v.position -= center;
                }
            }
        }

        Ok(Mesh { groups, materials })
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
    edge1.cross(edge2).normalize()
}
