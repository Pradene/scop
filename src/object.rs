use lineal::Vector;

#[derive(Debug)]
pub struct Object {
    name: String,
    vertices: Vec<Vector<f32, 4>>,
    normals: Vec<Vector<f32, 4>>,
    faces: Vec<Vec<usize>>
}

impl Object {
    pub fn new(name: &str) -> Self {
        return Object {
            name: name.to_string(),
            vertices: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
        };
    }
}