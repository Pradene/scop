use scop::camera::Camera;
use scop::object::Object;
use scop::scene::Scene;
use scop::WINDOW_HEIGHT;
use scop::{app::App, WINDOW_WIDTH};

use scop::math::Vec3;

use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), String> {
    let event_loop = EventLoop::new().map_err(|e| format!("Failed to create event loop: {}", e))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let camera = Camera::new(
        Vec3::Z * 200.,
        Vec3::Z * -1.,
        45f32.to_radians(),
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        0.1,
        500.,
    );

    let mut scene = Scene::new();

    let object = Object::parse("assets/teapot.obj").unwrap();
    scene.add(object);

    let mut app = App::new(scene, camera);

    let _ = event_loop.run_app(&mut app);

    return Ok(());
}
