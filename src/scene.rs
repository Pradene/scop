use crate::camera::Camera;
use crate::math::Mat4;
use crate::objects::Object;

pub type ObjectId = usize;

pub struct SceneObject {
    pub id: ObjectId,
    pub object: Object,
    pub transform: Mat4,
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
            transform: Mat4::identity()
        });
    }

    pub fn remove(&mut self, index: usize) {
        self.objects.remove(index);
    }
}