use lineal::Vector;

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub vertices: Vec<Vector<f32, 4>>,
    pub normals: Vec<Vector<f32, 4>>,
    pub faces: Vec<Vec<(usize, Option<usize>, Option<usize>)>>,
}

#[derive(Debug)]
pub struct Object {
    pub groups: Vec<Group>,
}

impl Object {
    pub fn new() -> Self {
        return Object { groups: Vec::new() };
    }
}

impl Group {
    pub fn new() -> Self {
        return Group {
            name: String::new(),
            vertices: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
        };
    }
}
