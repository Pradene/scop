use rand::Rng;

use crate::vulkan::Vertex;
use lineal::Vector;

use crate::objects::lexer::{Lexer, Token};

#[derive(Debug)]
pub enum ObjError {
    IoError(std::io::Error),
    ParseError(String),
    InvalidIndex,
    UnexpectedToken,
}

impl From<std::num::ParseFloatError> for ObjError {
    fn from(e: std::num::ParseFloatError) -> Self {
        ObjError::ParseError(e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct FaceVertex {
    pub vertex: usize,
    pub texture: Option<usize>,
    pub normal: Option<usize>,
}

pub type Face = Vec<FaceVertex>;

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub faces: Vec<Face>,
}

impl Group {
    pub fn new() -> Self {
        return Group {
            name: String::new(),
            faces: Vec::new(),
        };
    }

    pub fn is_empty(&self) -> bool {
        return self.faces.is_empty();
    }
}

#[derive(Debug)]
pub struct Object {
    pub groups: Vec<Group>,
    pub vertices: Vec<Vector<f32, 3>>,
    pub normals: Vec<Vector<f32, 3>>,
}

impl Object {
    pub fn new() -> Self {
        return Object {
            groups: Vec::new(),
            vertices: Vec::new(),
            normals: Vec::new(),
        };
    }

    pub fn parse(path: &str) -> Result<Object, String> {
        let mut lexer = Lexer::new(path).unwrap();

        let mut object = Object::new();
        let mut group = Group::new();

        while let Ok(token) = lexer.next_token() {
            match token {
                Token::Group => {
                    if !group.is_empty() {
                        object.groups.push(group);
                        group = Group::new();
                    }

                    match lexer.next_token() {
                        Ok(Token::Identifier(name)) => {
                            group.name = name;
                        }

                        _ => {
                            return Err(
                                "Expected an identifier for group name, but got a different token."
                                    .to_string(),
                            );
                        }
                    }
                }

                Token::Vertice => {
                    let mut coordinates = Vec::new();
                    for _ in 0..3 {
                        match lexer.next_token() {
                            Ok(Token::Number(num)) => {
                                coordinates.push(num);
                            }

                            _ => {
                                return Err(
                                    "Expected a number for vertex coordinate, but got something else."
                                        .to_string()
                                );
                            }
                        }
                    }

                    if coordinates.len() == 3 {
                        object.vertices.push(Vector::try_from(coordinates).unwrap());
                    } else {
                        return Err(format!(
                            "Error: Invalid number of vertex coordinates. Expected 3 but got {}",
                            coordinates.len()
                        ));
                    }
                }

                Token::Normal => {
                    let mut coordinates = Vec::new();
                    for _ in 0..3 {
                        match lexer.next_token() {
                            Ok(Token::Number(num)) => {
                                coordinates.push(num);
                            }

                            _ => {
                                return Err(
                                    "Expected a number for vertex coordinate, but got something else."
                                        .to_string()
                                );
                            }
                        }
                    }

                    if coordinates.len() == 3 {
                        object.normals.push(Vector::try_from(coordinates).unwrap());
                    } else {
                        return Err(format!(
                            "Error: Invalid number of vertex coordinates. Expected 3 but got {}",
                            coordinates.len()
                        ));
                    }
                }

                Token::Face => {
                    let mut face: Face = Vec::new();

                    loop {
                        let next_token = lexer.peek_token();
                        match next_token {
                            Ok(Token::Number(index)) => {
                                let vertex_index = (index as usize).saturating_sub(1);
                                let _ = lexer.next_token();

                                let mut texture_index = None;
                                let mut normal_index = None;

                                if let Ok(Token::Slash) = lexer.peek_token() {
                                    let _ = lexer.next_token();

                                    if let Ok(Token::Number(index)) = lexer.peek_token() {
                                        texture_index = Some((index as usize).saturating_sub(1));
                                        let _ = lexer.next_token();
                                    }

                                    if let Ok(Token::Slash) = lexer.peek_token() {
                                        let _ = lexer.next_token();

                                        if let Ok(Token::Number(index)) = lexer.peek_token() {
                                            normal_index = Some((index as usize).saturating_sub(1));
                                            let _ = lexer.next_token();
                                        }
                                    }
                                }

                                face.push(FaceVertex {
                                    vertex: vertex_index,
                                    texture: texture_index,
                                    normal: normal_index,
                                });
                            }

                            Ok(_) => break,
                            Err(_) => {
                                return Err("Parsing error".to_string());
                            }
                        }
                    }

                    let triangles = Object::triangulate_face(&face);
                    for triangle in triangles {
                        group.faces.push(triangle);
                    }
                }

                Token::Comment(_) => {
                    continue;
                }

                Token::EOF => {
                    break;
                }

                _ => {
                    return Err(format!("{:?} not implemented", token));
                }
            }
        }

        if !group.is_empty() {
            object.groups.push(group);
        }

        return Ok(object);
    }

    fn triangulate_face(face: &[FaceVertex]) -> Vec<Face> {
        let mut triangles: Vec<Face> = Vec::new();

        if face.len() <= 3 {
            triangles.push(face.to_vec());
            return triangles;
        }

        for i in 0..face.len() - 1 {
            triangles.push(vec![
                face[0].clone(),
                face[i].clone(),
                face[i + 1].clone()
            ])
        }

        return triangles;
    }

    pub fn get_vertices_and_indices(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut rng = rand::rng();

        let vertices = self
            .vertices
            .iter()
            .enumerate()
            .map(|(i, v)| Vertex {
                position: v.clone(),
                normal: if i < self.normals.len() {
                    self.normals[i].clone()
                } else {
                    Vector::new([1., 0., 0.])
                },
                color: Vector::new([
                    rng.random_range(0.0..1.0),
                    rng.random_range(0.0..1.0),
                    rng.random_range(0.0..1.0),
                ]),
            })
            .collect::<Vec<Vertex>>();

        let mut indices = Vec::new();

        for group in &self.groups {
            for face in &group.faces {
                for face_vertex in face {
                    indices.push(face_vertex.vertex as u32);
                }
            }
        }

        return (vertices, indices);
    }
}
