use crate::camera::Camera;
use crate::objects::Object;

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            objects: Vec::with_capacity(4),
        }
    }

    pub fn add(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn remove(&mut self, index: usize) {
        self.objects.remove(index);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }
}