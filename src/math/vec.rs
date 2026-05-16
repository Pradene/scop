use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Index, IndexMut, Neg};

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO:  Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE:   Vec3 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    pub const X:     Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y:     Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z:     Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    #[inline] pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    #[inline] pub fn splat(v: f32) -> Self { Self::new(v, v, v) }

    #[inline] pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline] pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    /// Euclidean (L2) norm
    #[inline] pub fn length(self) -> f32 { self.dot(self).sqrt() }

    /// L1 (Manhattan) norm
    #[inline] pub fn length_l1(self) -> f32 { self.x.abs() + self.y.abs() + self.z.abs() }

    /// L∞ (Chebyshev) norm
    #[inline] pub fn length_inf(self) -> f32 { self.x.abs().max(self.y.abs()).max(self.z.abs()) }

    /// Squared Euclidean length
    #[inline] pub fn length_squared(self) -> f32 { self.dot(self) }

    /// Cosine similarity; returns 0 if either vector is near-zero
    #[inline] pub fn cosine(self, rhs: Self) -> f32 {
        let denom = self.length() * rhs.length();
        if denom <= f32::EPSILON { 0.0 } else { self.dot(rhs) / denom }
    }

    /// Returns a unit vector. Panics if the vector is near-zero.
    #[inline] pub fn normalize(self) -> Self {
        let len = self.length();
        assert!(len > f32::EPSILON, "Vec3::normalize called on a zero (or near-zero) vector");
        self / len
    }

    /// Returns a unit vector, or `None` if the vector is near-zero.
    #[inline] pub fn try_normalize(self) -> Option<Self> {
        let len = self.length();
        if len > f32::EPSILON { Some(self / len) } else { None }
    }
}

// ── From / Into ──────────────────────────────

impl From<[f32; 3]> for Vec3 {
    fn from(a: [f32; 3]) -> Self { Self::new(a[0], a[1], a[2]) }
}

impl From<Vec3> for [f32; 3] {
    fn from(v: Vec3) -> Self { [v.x, v.y, v.z] }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from((x, y, z): (f32, f32, f32)) -> Self { Self::new(x, y, z) }
}

impl From<Vec3> for (f32, f32, f32) {
    fn from(v: Vec3) -> Self { (v.x, v.y, v.z) }
}

// ── Display ──────────────────────────────────

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec3({}, {}, {})", self.x, self.y, self.z)
    }
}

// ── Arithmetic ops ───────────────────────────

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z) }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z) }
}
impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) { self.x += rhs.x; self.y += rhs.y; self.z += rhs.z; }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z) }
}
impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) { self.x -= rhs.x; self.y -= rhs.y; self.z -= rhs.z; }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self { Self::new(self.x * rhs, self.y * rhs, self.z * rhs) }
}
impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) { self.x *= rhs; self.y *= rhs; self.z *= rhs; }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self { Self::new(self.x / rhs, self.y / rhs, self.z / rhs) }
}
impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) { self.x /= rhs; self.y /= rhs; self.z /= rhs; }
}

// ── Indexing ─────────────────────────────────

impl Index<usize> for Vec3 {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        match i { 0 => &self.x, 1 => &self.y, 2 => &self.z, _ => panic!("Vec3 index {i} out of bounds") }
    }
}
impl IndexMut<usize> for Vec3 {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i { 0 => &mut self.x, 1 => &mut self.y, 2 => &mut self.z, _ => panic!("Vec3 index {i} out of bounds") }
    }
}

// ─────────────────────────────────────────────
//  Vec4
// ─────────────────────────────────────────────

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Vec4 = Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const ONE:  Vec4 = Vec4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };

    #[inline] pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self { x, y, z, w } }
    #[inline] pub fn splat(v: f32) -> Self { Self::new(v, v, v, v) }

    /// Construct from a Vec3 and a w component (useful for points/directions)
    #[inline] pub fn from_vec3(v: Vec3, w: f32) -> Self { Self::new(v.x, v.y, v.z, w) }

    /// Discard w, returning the xyz components as a Vec3
    #[inline] pub fn xyz(self) -> Vec3 { Vec3::new(self.x, self.y, self.z) }

    #[inline] pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }

    #[inline] pub fn length(self) -> f32 { self.dot(self).sqrt() }
    #[inline] pub fn length_squared(self) -> f32 { self.dot(self) }

    #[inline] pub fn normalize(self) -> Self {
        let len = self.length();
        assert!(len > f32::EPSILON, "Vec4::normalize called on a zero (or near-zero) vector");
        self / len
    }

    #[inline] pub fn try_normalize(self) -> Option<Self> {
        let len = self.length();
        if len > f32::EPSILON { Some(self / len) } else { None }
    }
}

// ── From / Into ──────────────────────────────

impl From<[f32; 4]> for Vec4 {
    fn from(a: [f32; 4]) -> Self { Self::new(a[0], a[1], a[2], a[3]) }
}

impl From<Vec4> for [f32; 4] {
    fn from(v: Vec4) -> Self { [v.x, v.y, v.z, v.w] }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from((x, y, z, w): (f32, f32, f32, f32)) -> Self { Self::new(x, y, z, w) }
}

impl From<Vec4> for (f32, f32, f32, f32) {
    fn from(v: Vec4) -> Self { (v.x, v.y, v.z, v.w) }
}

// ── Display ──────────────────────────────────

impl fmt::Display for Vec4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec4({}, {}, {}, {})", self.x, self.y, self.z, self.w)
    }
}

// ── Arithmetic ops ───────────────────────────

impl Neg for Vec4 {
    type Output = Self;
    fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z, -self.w) }
}

impl Add for Vec4 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w) }
}
impl AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Self) { self.x += rhs.x; self.y += rhs.y; self.z += rhs.z; self.w += rhs.w; }
}

impl Sub for Vec4 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z, self.w - rhs.w) }
}
impl SubAssign for Vec4 {
    fn sub_assign(&mut self, rhs: Self) { self.x -= rhs.x; self.y -= rhs.y; self.z -= rhs.z; self.w -= rhs.w; }
}

impl Mul<f32> for Vec4 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self { Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs) }
}
impl Mul<Vec4> for f32 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 { rhs * self }
}
impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) { self.x *= rhs; self.y *= rhs; self.z *= rhs; self.w *= rhs; }
}

impl Div<f32> for Vec4 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self { Self::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs) }
}
impl DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, rhs: f32) { self.x /= rhs; self.y /= rhs; self.z /= rhs; self.w /= rhs; }
}

// ── Indexing ─────────────────────────────────

impl Index<usize> for Vec4 {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        match i { 0 => &self.x, 1 => &self.y, 2 => &self.z, 3 => &self.w, _ => panic!("Vec4 index {i} out of bounds") }
    }
}
impl IndexMut<usize> for Vec4 {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i { 0 => &mut self.x, 1 => &mut self.y, 2 => &mut self.z, 3 => &mut self.w, _ => panic!("Vec4 index {i} out of bounds") }
    }
}
