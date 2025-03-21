use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use lineal::Vector;

use crate::materials::{Material, MaterialParser};
use crate::vulkan::Vertex;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Object {
    pub groups: Vec<Group>,
    pub vertices: Vec<Vector<f32, 3>>,
    pub normals: Vec<Vector<f32, 3>>,
    pub center: Vector<f32, 3>,
    pub materials: HashMap<String, Material>,
}

impl Object {
    pub fn new() -> Self {
        Object {
            groups: Vec::new(),
            vertices: Vec::new(),
            normals: Vec::new(),
            center: Vector::new([0., 0., 0.]),
            materials: HashMap::new(),
        }
    }

    pub fn parse(path: &str) -> Result<Object, String> {
        let parser = ObjectParser::new(path)?;
        parser.parse()
    }

    pub fn triangulate_face(face: &[FaceVertex]) -> Vec<Face> {
        let mut triangles: Vec<Face> = Vec::new();

        if face.len() == 3 {
            triangles.push(face.to_vec());
            return triangles;
        }

        // Fan triangulation for convex polygons
        for i in 1..face.len() - 1 {
            triangles.push(vec![face[0].clone(), face[i].clone(), face[i + 1].clone()]);
        }

        triangles
    }

    pub fn compute_center(&self) -> Vector<f32, 3> {
        if self.vertices.is_empty() {
            return Vector::from([0.0, 0.0, 0.0]);
        }

        let mut sum = Vector::from([0.0, 0.0, 0.0]);
        for vertex in &self.vertices {
            sum += *vertex;
        }

        sum / (self.vertices.len() as f32)
    }

    pub fn get_vertices_and_indices(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // First, create the vertices
        for (i, v) in self.vertices.iter().enumerate() {
            let normal = if i < self.normals.len() {
                self.normals[i]
            } else {
                Vector::new([1.0, 0.0, 0.0]) // Default normal
            };

            let color = Vector::new([0.7, 0.7, 0.7]); // Default color

            vertices.push(Vertex {
                position: v.clone(),
                normal,
                color,
            });
        }

        for group in &self.groups {
            let base_color = if let Some(material_name) = &group.material {
                if let Some(material) = self.materials.get(material_name) {
                    // Use diffuse color if available, otherwise use default
                    material.kd.unwrap_or(Vector::new([0.7, 0.7, 0.7]))
                } else {
                    Vector::new([0.7, 0.7, 0.7])
                }
            } else {
                Vector::new([0.7, 0.7, 0.7])
            };

            // Apply the material color to vertices referenced by this group
            for face in &group.faces {
                for face_vertex in face {
                    // Update the vertex color
                    if face_vertex.vertex < vertices.len() {
                        vertices[face_vertex.vertex].color = base_color;
                    }

                    indices.push(face_vertex.vertex as u32);
                }
            }
        }

        (vertices, indices)
    }
}

#[derive(Debug)]
pub enum ObjError {
    IoError(std::io::Error),
    ParseError(String),
    InvalidIndex,
}

impl From<std::io::Error> for ObjError {
    fn from(e: std::io::Error) -> Self {
        ObjError::IoError(e)
    }
}

impl From<std::num::ParseFloatError> for ObjError {
    fn from(e: std::num::ParseFloatError) -> Self {
        ObjError::ParseError(e.to_string())
    }
}

pub struct ObjectParser {
    path: String,
    base_dir: String,
}

impl ObjectParser {
    pub fn new(path: &str) -> Result<Self, String> {
        let path_obj = Path::new(path);
        let base_dir = if let Some(parent) = path_obj.parent() {
            parent.to_string_lossy().to_string()
        } else {
            String::new()
        };

        Ok(ObjectParser {
            path: path.to_string(),
            base_dir,
        })
    }

    pub fn parse(&self) -> Result<Object, String> {
        let file = File::open(&self.path).map_err(|e| e.to_string())?;
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

            match parts[0] {
                "v" => self.parse_vertex(&parts, &mut object)?,
                "vn" => self.parse_normal(&parts, &mut object)?,
                "g" => {
                    // Save current group if it has faces
                    if !current_group.is_empty() {
                        object.groups.push(current_group);
                    }

                    current_group = self.parse_group(&parts, current_material.clone())?;
                }
                "f" => self.parse_face(&parts, &mut current_group, &object)?,
                "mtllib" => self.parse_material_lib(&parts, &mut object)?,
                "usemtl" => {
                    current_material = self.parse_use_material(&parts)?;
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

    fn parse_vertex(&self, parts: &[&str], object: &mut Object) -> Result<(), String> {
        if parts.len() < 4 {
            return Err("Not enough coordinates for vertex".to_string());
        }

        let x = parts[1].parse::<f32>().map_err(|e| e.to_string())?;
        let y = parts[2].parse::<f32>().map_err(|e| e.to_string())?;
        let z = parts[3].parse::<f32>().map_err(|e| e.to_string())?;

        object.vertices.push(Vector::new([x, y, z]));
        Ok(())
    }

    fn parse_normal(&self, parts: &[&str], object: &mut Object) -> Result<(), String> {
        if parts.len() < 4 {
            return Err("Not enough coordinates for normal".to_string());
        }

        let x = parts[1].parse::<f32>().map_err(|e| e.to_string())?;
        let y = parts[2].parse::<f32>().map_err(|e| e.to_string())?;
        let z = parts[3].parse::<f32>().map_err(|e| e.to_string())?;

        object.normals.push(Vector::new([x, y, z]));
        Ok(())
    }

    fn parse_group(&self, parts: &[&str], material: Option<String>) -> Result<Group, String> {
        let name = if parts.len() > 1 {
            parts[1..].join(" ")
        } else {
            "unnamed".to_string()
        };

        let mut group = Group::new(name);
        group.material = material;

        Ok(group)
    }

    fn parse_face(&self, parts: &[&str], group: &mut Group, object: &Object) -> Result<(), String> {
        if parts.len() < 4 {
            return Err("Face needs at least 3 vertices".to_string());
        }

        let mut face = Vec::new();

        for vertex_str in &parts[1..] {
            let indices: Vec<&str> = vertex_str.split('/').collect();

            let vertex_index = indices
                .get(0)
                .ok_or_else(|| "Missing vertex index".to_string())?
                .parse::<usize>()
                .map_err(|_| "Invalid vertex index".to_string())?
                .saturating_sub(1); // OBJ indices are 1-based

            if vertex_index >= object.vertices.len() {
                return Err(format!("Vertex index {} out of bounds", vertex_index + 1));
            }

            let texture_index = indices
                .get(1)
                .and_then(|idx| if idx.is_empty() { None } else { Some(idx) })
                .and_then(|idx| idx.parse::<usize>().ok())
                .map(|idx| idx.saturating_sub(1));

            let normal_index = indices
                .get(2)
                .and_then(|idx| if idx.is_empty() { None } else { Some(idx) })
                .and_then(|idx| idx.parse::<usize>().ok())
                .map(|idx| idx.saturating_sub(1));

            // Validate normal index if present
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

        let triangles = Object::triangulate_face(&face);
        for triangle in triangles {
            group.faces.push(triangle);
        }

        Ok(())
    }

    fn parse_material_lib(&self, parts: &[&str], object: &mut Object) -> Result<(), String> {
        if parts.len() < 2 {
            return Ok(()); // Skip if no material library specified
        }

        let mtl_path = parts[1..].join(" ");
        let full_path = if Path::new(&mtl_path).is_absolute() {
            mtl_path
        } else {
            format!("{}/{}", self.base_dir, mtl_path)
        };

        let mut parser = MaterialParser::new(full_path)
            .map_err(|e| format!("Failed to open material file: {}", e))?;

        let materials = parser
            .parse()
            .map_err(|e| format!("Failed to parse material file: {}", e))?;

        object.materials.extend(materials);
        Ok(())
    }

    fn parse_use_material(&self, parts: &[&str]) -> Result<Option<String>, String> {
        if parts.len() < 2 {
            Ok(None)
        } else {
            Ok(Some(parts[1..].join(" ")))
        }
    }
}
