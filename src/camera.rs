use crate::math::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub direction: Vec3,

    ratio: f32,
    far: f32,
    near: f32,
    fov: f32,
}

impl Camera {
    pub fn new(
        position: Vec3,
        direction: Vec3,
        fov: f32,
        ratio: f32,
        near: f32,
        far: f32,
    ) -> Camera {
        return Camera {
            position,
            direction,

            fov,
            near,
            far,
            ratio,
        };
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        let projection = Mat4::projection(self.fov, self.ratio, self.near, self.far);

        return projection;
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let view = Mat4::look_at(self.position, self.direction, Vec3::Y);

        return view;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ratio = width as f32 / height as f32;
    }
}
