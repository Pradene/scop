use std::sync::Arc;

use super::{MeshHandle, Renderer, ResourcesManager, VkContext};
use crate::camera::Camera;
use crate::scene::Scene;

use sdl3::video::Window;

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

    pub fn load_mesh(&mut self, path: &str) -> Result<MeshHandle, String> {
        self.manager.load_mesh(&*self.context, path)
    }

    pub fn draw(&mut self, window: &Window, camera: &Camera, scene: &Scene) -> Result<(), String> {
        self.renderer.draw(window, camera, scene, &self.manager)
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
