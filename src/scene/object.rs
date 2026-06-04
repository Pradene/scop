use crate::math::Mat4;
use crate::renderer::MeshHandle;

pub type ObjectHandle = usize;

pub struct Object {
    pub mesh: MeshHandle,
    pub transform: Mat4,
}

impl Object {
    pub fn new(mesh: MeshHandle) -> Self {
        Self {
            mesh,
            transform: Mat4::identity(),
        }
    }

    pub fn update_transform(&mut self, transform: Mat4) {
        self.transform = transform;
    }
}
