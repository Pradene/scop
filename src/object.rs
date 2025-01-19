use lineal::Vector;

#[derive(Debug)]
struct Object {
    vertices: Vec<Vector<f32, 4>>,
    normals: Vec<Vector<f32, 4>>,
    faces: Vec<Vec<usize>>
}