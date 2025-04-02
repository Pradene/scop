use crate::{camera::Camera, objects::Object, vulkan::VkContext, WINDOW_HEIGHT, WINDOW_WIDTH};

use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, window::{Window, WindowId}
};

pub struct App {
    window: Option<Window>,
    context: Option<VkContext>,
    camera: Camera,
    object: Object,
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

            match VkContext::new(&window, &self.camera, &self.object) {
                Ok(context) => {
                    self.context = Some(context);
                    println!("Vulkan context initialized successfully.");
                }
                Err(e) => {
                    println!("Failed to create Vulkan context: {:?}", e);
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
                if let Some(context) = &mut self.context {
                    context.draw_frame(self.window.as_ref().unwrap());
                }

                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::Resized(_) => {
                if let Some(context) = &mut self.context {
                    context.resize(&self.window.as_ref().unwrap()).unwrap();
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
    pub fn new(camera: Camera, object: Object) -> App {
        return App {
            window: None,
            context: None,
            camera,
            object,
        };
    }
}
