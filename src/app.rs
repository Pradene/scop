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

            let window = match event_loop.create_window(window_attributes) {
                Ok(w) => w,
                Err(e) => {
                    println!("Failed to create window: {:?}", e);
                    return;
                }
            };

            let context = match VkContext::new(&window) {
                Ok(ctx) => ctx,
                Err(e) => {
                    println!("Failed to create Vulkan context: {:?}", e);
                    return;
                }
            };

            match Renderer::new(&window, context) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                    println!("Vulkan renderer initialized successfully.");
                    self.window = Some(window);
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
                if event_loop.exiting() {
                    return;
                }

                if let (Some(window), Some(renderer)) = (&self.window, &mut self.renderer) {
                    let _ = renderer.draw(window, &self.scene);
                    
                    if !event_loop.exiting() {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::Resized(_) => {
                if let (Some(window), Some(renderer)) = (&self.window, &mut self.renderer) {
                    let (width, height): (u32, u32) = window.inner_size().into();
                    
                    if width > 0 && height > 0 {
                        self.scene.resize(width, height);
                        if let Err(e) = renderer.resize(width, height) {
                            println!("Failed to handle swapchain resize: {:?}", e);
                        }
                    }
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    if event.state.is_pressed() {
                        event_loop.exit();
                    }
                }
            }

            _ => (),
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(renderer) = &self.renderer {
            renderer.wait_idle();
        }
        self.renderer = None;
        self.window = None;
    }
}

impl App {
    pub fn new(scene: Scene) -> App {
        App {
            window: None,
            renderer: None,
            scene,
        }
    }
}
