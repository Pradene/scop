use crate::math::Mat4;
use crate::math::Vec3;
use crate::renderer::MeshHandle;

pub type ObjectHandle = usize;

pub struct Object {
    mesh: MeshHandle,
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

impl Object {
    pub fn new(mesh: MeshHandle) -> Self {
        Self {
            mesh,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }

    pub fn id(&self) -> MeshHandle {
        self.mesh
    }

    pub fn set_position(&mut self, position: Vec3) -> &mut Self {
        self.position = position;
        self
    }

    pub fn translate(&mut self, delta: Vec3) -> &mut Self {
        self.position += delta;
        self
    }

    pub fn set_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) -> &mut Self {
        self.rotation = Vec3::new(pitch, yaw, roll);
        self
    }

    pub fn rotate(&mut self, pitch: f32, yaw: f32, roll: f32) -> &mut Self {
        self.rotation += Vec3::new(pitch, yaw, roll);
        self
    }

    pub fn set_scale(&mut self, scale: Vec3) -> &mut Self {
        self.scale = scale;
        self
    }

    pub fn scale_uniform(&mut self, factor: f32) -> &mut Self {
        self.scale = Vec3::splat(factor);
        self
    }

    pub fn transform(&self) -> Mat4 {
        // TODO: replace Euler angles with quaternions
        Mat4::identity()
            .translate(self.position)
            .rotate(self.rotation.x, Vec3::X)
            .rotate(self.rotation.y, Vec3::Y)
            .rotate(self.rotation.z, Vec3::Z)
            .scale(self.scale)
    }
}
