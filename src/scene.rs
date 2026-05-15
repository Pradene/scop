use crate::camera::Camera;
use lineal::Matrix;
use crate::objects::Object;

pub type ObjectId = usize;

pub struct SceneObject {
    pub id: ObjectId,
    pub object: Object,
    pub transform: Matrix<f32, 4, 4>,
}

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<SceneObject>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            objects: Vec::with_capacity(4),
        }
    }

    pub fn add(&mut self, object: Object) {
        self.objects.push(SceneObject {
            id: self.objects.len(),
            object,
            transform: Matrix::identity()
        });
    }

    pub fn remove(&mut self, index: usize) {
        self.objects.remove(index);
    }
}