use crate::{scene::Scene, renderer::Renderer, WINDOW_HEIGHT, WINDOW_WIDTH};

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

            match Renderer::new(&window) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                    println!("Vulkan renderer initialized successfully.");
                }
                Err(e) => {
                    println!("Failed to create Vulkan renderer: {:?}", e);
                    return;
                }
            }

            self.window = Some(window);
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
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(&self.window.as_ref().unwrap()).unwrap();
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
