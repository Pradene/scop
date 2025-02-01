use crate::{objects::Object, vulkan::VkContext};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub struct App {
    window: Option<Window>,
    context: Option<VkContext>,
    object: Object,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes().with_title("Scop");
            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");

            match VkContext::new(&window, &self.object) {
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
                // if let Some(context) = &mut self.context {
                //     context.cleanup();
                // }

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
                    context.recreate_swapchain(&self.window.as_ref().unwrap());
                }
            }

            _ => (),
        }
    }
}

impl App {
    pub fn new(object: Object) -> App {
        return App {
            window: None,
            context: None,
            object,
        };
    }
}
