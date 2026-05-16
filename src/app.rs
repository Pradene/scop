use crate::{WINDOW_HEIGHT, WINDOW_WIDTH, renderer::{Renderer, VkContext}, scene::Scene};

use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}
};

pub struct App {
    window: Option<Window>,
    renderer: Option<Renderer>,
    scene: Scene,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Scop")
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");

            self.window = Some(window);
            let window = self.window.as_ref().unwrap();

            let context = VkContext::new(window).unwrap();
            match Renderer::new(window, context) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                    println!("Vulkan renderer initialized successfully.");
                }
                Err(e) => {
                    println!("Failed to create Vulkan renderer: {:?}", e);
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    let _ = renderer.draw(self.window.as_ref().unwrap(), &self.scene);
                }

                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    let (width, height): (u32, u32) = window.inner_size().into();
                    self.scene.resize(width, height);
                    if let Some(renderer) = &mut self.renderer {
                        renderer.resize(width, height).unwrap();
                    }
                }
            }

            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => match event.physical_key {
                PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                _ => {}
            },

            _ => (),
        }
    }
}

impl App {
    pub fn new(scene: Scene) -> App {
        return App {
            window: None,
            renderer: None,
            scene,
        };
    }
}
