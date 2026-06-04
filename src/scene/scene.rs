use super::{Object, ObjectHandle};

pub struct Scene {
    pub objects: Vec<Object>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Object) -> ObjectHandle {
        self.objects.push(object);

        (self.objects.len() - 1) as ObjectHandle
    }

    pub fn get_object(&mut self, object_id: ObjectHandle) -> &mut Object {
        &mut self.objects[object_id]
    }
}
