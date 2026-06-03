use sdl3::video::Window;
use std::sync::Arc;

use super::{Renderer, ResourcesManager, VkContext};
use crate::camera::Camera;
use crate::parser::Mesh;

pub struct Engine {
    context: Arc<VkContext>,

    manager: ResourcesManager,
    renderer: Renderer,
}

impl Engine {
    pub fn new(window: &Window) -> Result<Engine, String> {
        let context = Arc::new(VkContext::new(window)?);

        let renderer = Renderer::new(window, context.clone())?;
        let manager = ResourcesManager::new(context.clone())?;

        Ok(Self {
            context,
            renderer,
            manager,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.renderer.resize(width, height)
    }

    pub fn draw(&mut self, window: &Window, camera: &Camera) -> Result<(), String> {
        self.renderer.draw(window, camera, &self.manager)
    }

    pub fn add_object(&mut self, mesh: Mesh) {
        self.manager.upload_object(&*self.context, &mesh);
    }

    pub fn wait_idle(&self) {
        self.context.device.wait_idle();
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.wait_idle();
    }
}
