use lineal::Vector;

#[derive(Debug, Clone)]
pub struct Material {
    pub ka: Option<Vector<f32, 3>>,
    pub kd: Option<Vector<f32, 3>>,
    pub ks: Option<Vector<f32, 3>>,
    pub ns: Option<f32>,
    pub ni: Option<f32>,
    pub dissolve: Option<f32>,
    pub illum: Option<i32>,
    pub map_ka: Option<String>,
    pub map_kd: Option<String>,
    pub map_ks: Option<String>,
}
