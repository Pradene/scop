use crate::objects::Object;

pub struct Scene {
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::with_capacity(4),
        }
    }

    pub fn add(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn remove(&mut self, index: usize) {
        self.objects.remove(index);
    }
}
