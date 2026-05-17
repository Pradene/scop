use crate::{WINDOW_HEIGHT, WINDOW_WIDTH, renderer::{Renderer, VkContext}, scene::Scene};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

pub struct App {
    window: Option<Window>,
    renderer: Option<Renderer>,
    scene: Scene,

    // Mouse state
    mouse_pressed: bool,
    last_mouse: Option<(f32, f32)>,

    // Keys currently held
    key_forward:  bool,
    key_backward: bool,
    key_left:     bool,
    key_right:    bool,
    key_up:       bool,
    key_down:     bool,

    last_update: std::time::Instant,
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
                    eprintln!("Failed to create window: {:?}", e);
                    return;
                }
            };

            let context = match VkContext::new(&window) {
                Ok(ctx) => ctx,
                Err(e) => {
                    eprintln!("Failed to create Vulkan context: {:?}", e);
                    return;
                }
            };

            match Renderer::new(&window, context) {
                Ok(renderer) => {
                    self.renderer = Some(renderer);
                    self.window = Some(window);
                }
                Err(e) => {
                    eprintln!("Failed to create Vulkan renderer: {:?}", e);
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

                self.tick_movement();

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
                            eprintln!("Failed to handle swapchain resize: {:?}", e);
                        }
                    }
                }
            }

            WindowEvent::MouseInput { state, button: MouseButton::Right, .. } => {
                self.mouse_pressed = state == ElementState::Pressed;
                if !self.mouse_pressed {
                    self.last_mouse = None;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let current = (position.x as f32, position.y as f32);
                if self.mouse_pressed {
                    if let Some(last) = self.last_mouse {
                        if let Some(window) = &self.window {
                            let size = window.inner_size();
                            let dx = (current.0 - last.0) / size.width as f32;
                            let dy = (current.1 - last.1) / size.height as f32;
                            self.scene.camera.look(dx, dy);
                        }
                    }
                }
                self.last_mouse = Some(current);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 10.0,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.5,
                };
                self.scene.camera.move_forward(amount * self.scene.camera.move_speed * 0.05);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => {
                        if pressed { event_loop.exit(); }
                    }
                    PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp)    => self.key_forward  = pressed,
                    PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown)  => self.key_backward = pressed,
                    PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft)  => self.key_left     = pressed,
                    PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => self.key_right    = pressed,
                    PhysicalKey::Code(KeyCode::KeyE) | PhysicalKey::Code(KeyCode::Space)      => self.key_up       = pressed,
                    PhysicalKey::Code(KeyCode::KeyQ) | PhysicalKey::Code(KeyCode::ShiftLeft)  => self.key_down     = pressed,
                    _ => {}
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
            mouse_pressed: false,
            last_mouse: None,
            key_forward:  false,
            key_backward: false,
            key_left:     false,
            key_right:    false,
            key_up:       false,
            key_down:     false,
            last_update: std::time::Instant::now(),
        }
    }

    fn tick_movement(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        let speed = self.scene.camera.move_speed * dt;

        if self.key_forward  { self.scene.camera.move_forward( speed); }
        if self.key_backward { self.scene.camera.move_forward(-speed); }
        if self.key_right    { self.scene.camera.move_right(   speed); }
        if self.key_left     { self.scene.camera.move_right(  -speed); }
        if self.key_up       { self.scene.camera.move_up(      speed); }
        if self.key_down     { self.scene.camera.move_up(     -speed); }
    }
}
