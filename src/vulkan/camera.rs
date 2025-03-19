use lineal::{Matrix, Vector};

pub struct Camera {
    pub position: Vector<f32, 3>,
    pub direction: Vector<f32, 3>,

    ratio: f32,
    far: f32,
    near: f32,
    fov: f32,
}

impl Camera {
    pub fn new(
        position: Vector<f32, 3>,
        direction: Vector<f32, 3>,
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

    pub fn projection_matrix(&self) -> Matrix<f32, 4, 4> {
        let projection = Matrix::projection(self.fov, self.ratio, self.near, self.far);

        return projection;
    }

    pub fn view_matrix(&self) -> Matrix<f32, 4, 4> {
        let view = Matrix::look_at(self.position, self.direction, Vector::new([0., 1., 0.]));

        return view;
    }
}
