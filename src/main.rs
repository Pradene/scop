use std::time::Duration;
use scop::camera::Camera;
use scop::renderer::Renderer;
use scop::scene::Object;
use scop::scene::Scene;
use scop::math::Vec3;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;

fn main() -> Result<(), String> {
    let sdl_context = sdl3::init()
        .map_err(|e| format!("Failed to create sdl context: {}", e))?;
    let video_subsystem = sdl_context
        .video()
        .map_err(|e| format!("Failed to create video subsystem: {}", e))?;

    let width = 600u32;
    let height = 400u32;

    let window = video_subsystem
        .window("scop", width, height)
        .position_centered()
        .vulkan()
        .resizable()
        .build()
        .map_err(|e| format!("Failed to create window: {}", e))?;

    let mut camera = Camera::new(
        Vec3::Z * 200.,
        Vec3::Z * -1.,
        45f32.to_radians(),
        width as f32 / height as f32,
        0.1,
        500.,
    );

    let mut renderer = Renderer::new(&window)
        .map_err(|e| format!("Failed to create renderer: {}", e))?;

    let mut scene = Scene::new();
    let object = Object::parse("assets/teapot.obj")
        .map_err(|e| format!("Failed to parse object: {}", e))?;
    let _ = renderer.upload_mesh(&object);
    scene.add(object);

    let mut event_pump = sdl_context.event_pump()
        .map_err(|e| format!("Failed to create event pump: {}", e))?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                Event::Window {
                    win_event: WindowEvent::Resized(w, h), ..
                } => {
                    if w > 0 && h > 0 {
                        camera.resize(w as u32, h as u32);
                        if let Err(e) = renderer.resize(w as u32, h as u32) {
                            eprintln!("Failed to resize swapchain: {:?}", e);
                        }
                    }
                }
                _ => {}
            }
        }

        if let Err(e) = renderer.draw(&window, &scene, &camera) {
            eprintln!("Failed to draw: {:?}", e);
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
