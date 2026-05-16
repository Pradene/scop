use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign};
use crate::math::{Vec3, Vec4};

// ─────────────────────────────────────────────
//  Mat4  (column-major, 4 × Vec4 columns)
// ─────────────────────────────────────────────
//
//  Columns:  x_axis  y_axis  z_axis  w_axis
//  Layout:   col 0   col 1   col 2   col 3
//
//  Indexing: mat[col][row]

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Mat4 {
    pub x_axis: Vec4,   // column 0
    pub y_axis: Vec4,   // column 1
    pub z_axis: Vec4,   // column 2
    pub w_axis: Vec4,   // column 3
}

// ── Constructors ─────────────────────────────

impl Mat4 {
    /// Build from four *column* vectors.
    #[inline]
    pub fn from_cols(x: Vec4, y: Vec4, z: Vec4, w: Vec4) -> Self {
        Self { x_axis: x, y_axis: y, z_axis: z, w_axis: w }
    }

    /// Build from a column-major array of arrays: `cols[col][row]`.
    #[inline]
    pub fn from_cols_array(cols: [[f32; 4]; 4]) -> Self {
        Self::from_cols(
            Vec4::from(cols[0]),
            Vec4::from(cols[1]),
            Vec4::from(cols[2]),
            Vec4::from(cols[3]),
        )
    }

    pub fn identity() -> Self {
        Self::from_cols(
            Vec4::new(1., 0., 0., 0.),
            Vec4::new(0., 1., 0., 0.),
            Vec4::new(0., 0., 1., 0.),
            Vec4::new(0., 0., 0., 1.),
        )
    }

    pub fn zero() -> Self {
        Self::from_cols(Vec4::ZERO, Vec4::ZERO, Vec4::ZERO, Vec4::ZERO)
    }

    /// Access a column by index (0–3).
    #[inline]
    pub fn col(&self, i: usize) -> Vec4 {
        match i {
            0 => self.x_axis,
            1 => self.y_axis,
            2 => self.z_axis,
            3 => self.w_axis,
            _ => panic!("Mat4::col index {i} out of bounds"),
        }
    }

    /// Mutably access a column by index (0–3).
    #[inline]
    pub fn col_mut(&mut self, i: usize) -> &mut Vec4 {
        match i {
            0 => &mut self.x_axis,
            1 => &mut self.y_axis,
            2 => &mut self.z_axis,
            3 => &mut self.w_axis,
            _ => panic!("Mat4::col_mut index {i} out of bounds"),
        }
    }

    /// Get a single element at (col, row).
    #[inline]
    pub fn get(&self, col: usize, row: usize) -> f32 {
        self.col(col)[row]
    }

    /// Set a single element at (col, row).
    #[inline]
    pub fn set(&mut self, col: usize, row: usize, v: f32) {
        self.col_mut(col)[row] = v;
    }
}

// ── Linear algebra ───────────────────────────

impl Mat4 {
    pub fn transpose(&self) -> Self {
        Self::from_cols_array([
            [self.get(0,0), self.get(1,0), self.get(2,0), self.get(3,0)],
            [self.get(0,1), self.get(1,1), self.get(2,1), self.get(3,1)],
            [self.get(0,2), self.get(1,2), self.get(2,2), self.get(3,2)],
            [self.get(0,3), self.get(1,3), self.get(2,3), self.get(3,3)],
        ])
    }

    /// Transform a Vec4 (column vector on the right: M * v).
    pub fn mul_vec4(&self, v: Vec4) -> Vec4 {
        self.x_axis * v.x + self.y_axis * v.y + self.z_axis * v.z + self.w_axis * v.w
    }
}

// ── Camera / projection ──────────────────────

impl Mat4 {
    /// Right-handed look-at view matrix (camera looks down −Z, OpenGL convention).
    pub fn look_at(position: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - position).normalize();
        let r = f.cross(up).normalize();
        let u = r.cross(f);

        Self::from_cols(
            Vec4::new( r.x,  u.x, -f.x, 0.),
            Vec4::new( r.y,  u.y, -f.y, 0.),
            Vec4::new( r.z,  u.z, -f.z, 0.),
            Vec4::new(-r.dot(position), -u.dot(position), f.dot(position), 1.),
        )
    }

    /// Perspective projection (right-handed, depth −1..1 / OpenGL convention).
    pub fn projection(fov: f32, ratio: f32, near: f32, far: f32) -> Self {
        let scale        = 1. / (fov * 0.5).tan();
        let range        = near - far;
        let two_near_far = 2. * near * far;

        Self::from_cols(
            Vec4::new(scale / ratio, 0.,     0.,                    0.),
            Vec4::new(0.,            -scale, 0.,                    0.),
            Vec4::new(0.,            0.,     (far + near) / range, -1.),
            Vec4::new(0.,            0.,     two_near_far / range,   0.),
        )
    }
}

// ── Transforms ───────────────────────────────

impl Mat4 {
    /// Returns `translation * self`.
    pub fn translate(&self, position: Vec3) -> Self {
        let t = Self::from_cols(
            Vec4::new(1., 0., 0., 0.),
            Vec4::new(0., 1., 0., 0.),
            Vec4::new(0., 0., 1., 0.),
            Vec4::new(position.x, position.y, position.z, 1.),
        );
        t * *self
    }

    /// Returns `rotation(angle, axis) * self`.
    pub fn rotate(&self, angle: f32, axis: Vec3) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let n = axis.normalize();
        let (x, y, z) = (n.x, n.y, n.z);
        let mc = 1. - c;

        let rotation = Self::from_cols(
            Vec4::new(x*x*mc + c,   y*x*mc + z*s, z*x*mc - y*s, 0.),
            Vec4::new(x*y*mc - z*s, y*y*mc + c,   z*y*mc + x*s, 0.),
            Vec4::new(x*z*mc + y*s, y*z*mc - x*s, z*z*mc + c,   0.),
            Vec4::new(0.,           0.,            0.,          1.),
        );
        rotation * *self
    }

    /// Returns `scale(s) * self`.
    pub fn scale(&self, s: Vec3) -> Self {
        let sc = Self::from_cols(
            Vec4::new(s.x, 0.,  0.,  0.),
            Vec4::new(0.,  s.y, 0.,  0.),
            Vec4::new(0.,  0.,  s.z, 0.),
            Vec4::new(0.,  0.,  0.,  1.),
        );
        sc * *self
    }
}

// ── Arithmetic ops ───────────────────────────

impl Add for Mat4 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::from_cols(
            self.x_axis + rhs.x_axis,
            self.y_axis + rhs.y_axis,
            self.z_axis + rhs.z_axis,
            self.w_axis + rhs.w_axis,
        )
    }
}
impl AddAssign for Mat4 {
    fn add_assign(&mut self, rhs: Self) { *self = *self + rhs; }
}

impl Sub for Mat4 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::from_cols(
            self.x_axis - rhs.x_axis,
            self.y_axis - rhs.y_axis,
            self.z_axis - rhs.z_axis,
            self.w_axis - rhs.w_axis,
        )
    }
}
impl SubAssign for Mat4 {
    fn sub_assign(&mut self, rhs: Self) { *self = *self - rhs; }
}

impl Mul for Mat4 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::from_cols(
            self.mul_vec4(rhs.x_axis),
            self.mul_vec4(rhs.y_axis),
            self.mul_vec4(rhs.z_axis),
            self.mul_vec4(rhs.w_axis),
        )
    }
}
impl MulAssign for Mat4 {
    fn mul_assign(&mut self, rhs: Self) { *self = *self * rhs; }
}

impl Mul<f32> for Mat4 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::from_cols(
            self.x_axis * rhs,
            self.y_axis * rhs,
            self.z_axis * rhs,
            self.w_axis * rhs,
        )
    }
}
impl MulAssign<f32> for Mat4 {
    fn mul_assign(&mut self, rhs: f32) { *self = *self * rhs; }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 { self.mul_vec4(rhs) }
}

impl PartialEq for Mat4 {
    fn eq(&self, other: &Self) -> bool {
        [
            self.x_axis - other.x_axis,
            self.y_axis - other.y_axis,
            self.z_axis - other.z_axis,
            self.w_axis - other.w_axis,
        ]
        .iter()
        .all(|diff| {
            diff.x.abs() <= f32::EPSILON
                && diff.y.abs() <= f32::EPSILON
                && diff.z.abs() <= f32::EPSILON
                && diff.w.abs() <= f32::EPSILON
        })
    }
}

// ── Display ───────────────────────────────────

impl fmt::Display for Mat4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print row by row for readability
        for row in 0..4 {
            write!(f, "[ {:.4}  {:.4}  {:.4}  {:.4} ]",
                self.get(0, row), self.get(1, row),
                self.get(2, row), self.get(3, row))?;
            if row < 3 { writeln!(f)?; }
        }
        Ok(())
    }
}
