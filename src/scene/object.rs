use std::collections::HashMap;

use crate::math::{Vec2, Vec3};
use crate::renderer::Vertex;
use crate::scene::RawMaterial;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FaceVertex {
    pub vertex: usize,
    pub texture: Option<usize>,
    pub normal: Option<usize>,
}

pub type Face = Vec<FaceVertex>;

#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub faces: Vec<Face>,
    pub material: Option<String>,
}

impl Group {
    pub fn new(name: String) -> Self {
        Group {
            name,
            faces: Vec::new(),
            material: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }
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
    pub fn triangulate_face(face: &[FaceVertex]) -> Vec<Face> {
        let mut triangles: Vec<Face> = Vec::new();

        if face.len() == 3 {
            triangles.push(face.to_vec());
            return triangles;
        }

        for i in 1..face.len() - 1 {
            triangles.push(vec![face[0].clone(), face[i].clone(), face[i + 1].clone()]);
        }

        triangles
    }

    pub fn compute_center(&self) -> Vec3 {
        if self.vertices.is_empty() {
            return Vec3::new(0.0, 0.0, 0.0);
        }

        let mut sum = Vec3::new(0.0, 0.0, 0.0);
        for vertex in &self.vertices {
            sum += *vertex;
        }

        sum / (self.vertices.len() as f32)
    }

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
