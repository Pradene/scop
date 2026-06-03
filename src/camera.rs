use crate::math::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,

    pub move_speed: f32,
    pub look_speed: f32,

    ratio: f32,
    far: f32,
    near: f32,
    fov: f32,
}

impl Camera {
    const MIN_PITCH: f32 = -std::f32::consts::FRAC_PI_2 + 0.01;
    const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

    pub fn new(position: Vec3, target: Vec3, fov: f32, ratio: f32, near: f32, far: f32) -> Self {
        let dir = (target - position).normalize();

        let yaw = dir.x.atan2(dir.z);
        let pitch = dir.y.asin().clamp(Self::MIN_PITCH, Self::MAX_PITCH);

        Self {
            position,
            yaw,
            pitch,
            move_speed: 50.0,
            look_speed: 2.,
            fov,
            near,
            far,
            ratio,
        }
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.pitch.cos() * self.yaw.sin(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.cos(),
        )
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.position + self.forward(), Vec3::Y)
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::projection(self.fov, self.ratio, self.near, self.far)
    }

    pub fn look(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw -= delta_x * self.look_speed;
        self.pitch =
            (self.pitch + delta_y * self.look_speed).clamp(Self::MIN_PITCH, Self::MAX_PITCH);
    }

    pub fn move_forward(&mut self, amount: f32) {
        self.position = self.position + self.forward() * amount;
    }

    pub fn move_right(&mut self, amount: f32) {
        self.position = self.position + self.right() * amount;
    }

    pub fn move_up(&mut self, amount: f32) {
        self.position = self.position + Vec3::Y * amount;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.ratio = width as f32 / height as f32;
    }
}
