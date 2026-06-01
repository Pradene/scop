use std::path::Path;

use crate::camera::Camera;
use crate::renderer::{Mesh, MeshHandle, Renderer};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use sdl3::video::Window;
use sdl3::Sdl;

pub struct App {
    pub sdl_context: Sdl,
    pub renderer: Renderer,
    pub window: Window,
    pub camera: Camera,
    event_pump: sdl3::EventPump,

    // Mouse state
    mouse_pressed: bool,
    last_mouse: Option<(f32, f32)>,

    // Keys currently held
    key_forward: bool,
    key_backward: bool,
    key_left: bool,
    key_right: bool,
    key_up: bool,
    key_down: bool,

    last_update: std::time::Instant,
}

impl App {
    pub fn new(camera: Camera, width: u32, height: u32) -> Result<App, String> {
        let sdl_context = sdl3::init().map_err(|e| format!("Failed to init SDL3: {}", e))?;

        let video_subsystem = sdl_context
            .video()
            .map_err(|e| format!("Failed to get video subsystem: {}", e))?;

        let window = video_subsystem
            .window("Scop", width, height)
            .position_centered()
            .vulkan()
            .resizable()
            .build()
            .map_err(|e| format!("Failed to create window: {}", e))?;

        let renderer =
            Renderer::new(&window).map_err(|e| format!("Failed to create renderer: {}", e))?;

        let event_pump = sdl_context
            .event_pump()
            .map_err(|e| format!("Failed to get event pump: {}", e))?;

        Ok(App {
            sdl_context,
            window,
            renderer,
            camera,
            event_pump,
            mouse_pressed: false,
            last_mouse: None,
            key_forward: false,
            key_backward: false,
            key_left: false,
            key_right: false,
            key_up: false,
            key_down: false,
            last_update: std::time::Instant::now(),
        })
    }

    pub fn handle_events(&mut self) -> Result<bool, String> {
        let events: Vec<Event> = self.event_pump.poll_iter().collect();

        for event in events {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return Ok(false);
                }

                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    if w > 0 && h > 0 {
                        self.camera.resize(w as u32, h as u32);
                        if let Err(e) = self.renderer.resize(w as u32, h as u32) {
                            eprintln!("Failed to resize swapchain: {:?}", e);
                        }
                    }
                }

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Right,
                    ..
                } => {
                    self.mouse_pressed = true;
                }
                Event::MouseButtonUp {
                    mouse_btn: MouseButton::Right,
                    ..
                } => {
                    self.mouse_pressed = false;
                    self.last_mouse = None;
                }

                Event::MouseMotion { x, y, .. } => {
                    let current = (x as f32, y as f32);
                    if self.mouse_pressed {
                        if let Some(last) = self.last_mouse {
                            let (w, h) = self.window.size();
                            let dx = (current.0 - last.0) / w as f32;
                            let dy = (current.1 - last.1) / h as f32;
                            self.camera.look(dx, -dy);
                        }
                    }
                    self.last_mouse = Some(current);
                }

                Event::MouseWheel { y, .. } => {
                    let amount = y * 10.0;
                    self.camera
                        .move_forward(amount * self.camera.move_speed * 0.05);
                }

                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    self.set_key(key, true);
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    self.set_key(key, false);
                }

                _ => {}
            }
        }

        Ok(true)
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        let speed = self.camera.move_speed * dt;

        if self.key_forward {
            self.camera.move_forward(speed);
        }
        if self.key_backward {
            self.camera.move_forward(-speed);
        }
        if self.key_right {
            self.camera.move_right(speed);
        }
        if self.key_left {
            self.camera.move_right(-speed);
        }
        if self.key_up {
            self.camera.move_up(speed);
        }
        if self.key_down {
            self.camera.move_up(-speed);
        }
    }

    pub fn draw(&mut self) {
        if let Err(e) = self.renderer.draw(&self.window, &self.camera) {
            eprintln!("Failed to draw: {:?}", e);
        }
    }

    fn set_key(&mut self, key: Keycode, pressed: bool) {
        match key {
            Keycode::W | Keycode::Up => self.key_forward = pressed,
            Keycode::S | Keycode::Down => self.key_backward = pressed,
            Keycode::A | Keycode::Left => self.key_left = pressed,
            Keycode::D | Keycode::Right => self.key_right = pressed,
            Keycode::E | Keycode::Space => self.key_up = pressed,
            Keycode::Q | Keycode::LShift => self.key_down = pressed,
            _ => {}
        }
    }

    pub fn load_object(&mut self, path: &Path) -> Result<MeshHandle, String> {
        self.renderer.load_object(path)
    }

    pub fn get_object(&mut self, handle: MeshHandle) -> &mut Mesh {
        self.renderer.get_mesh(handle)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.renderer.wait_idle();
    }
}
