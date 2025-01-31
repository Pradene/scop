#[derive(Clone)]
pub struct QueueFamiliesIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}
