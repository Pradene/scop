use std::collections::HashMap;

use crate::math::{Vec2, Vec3};
use crate::renderer::Vertex;
use crate::scene::RawMaterial;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FaceVertex {
    pub vertex: usize,
    pub normal: Option<usize>,
    pub texture: Option<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct Group {
    pub faces: Vec<[FaceVertex; 3]>,
    pub material: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Object {
    pub groups: Vec<Group>,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub textures: Vec<Vec2>,
    pub center: Vec3,
    pub materials: HashMap<String, RawMaterial>,
}

impl Object {
    pub fn get_group_vertices_and_indices(&self, group: &Group) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_map: HashMap<FaceVertex, u32> = HashMap::new();

        for face in &group.faces {
            for fv in face {
                if let Some(&idx) = index_map.get(fv) {
                    indices.push(idx);
                } else {
                    let idx = vertices.len() as u32;

                    let normal = fv
                        .normal
                        .and_then(|n| self.normals.get(n))
                        .copied()
                        .unwrap_or(Vec3::new(0.0, 1.0, 0.0));

                    let uv = fv
                        .texture
                        .and_then(|t| self.textures.get(t))
                        .map(|v| Vec2::new(v.x, v.y))
                        .unwrap_or(Vec2::new(0.0, 0.0));

                    vertices.push(Vertex {
                        position: self.vertices[fv.vertex],
                        normal,
                        uv,
                    });

                    index_map.insert(fv.clone(), idx);
                    indices.push(idx);
                }
            }
        }

        (vertices, indices)
    }
}
