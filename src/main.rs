use scop::WINDOW_HEIGHT;
use scop::{app::App, WINDOW_WIDTH};
use scop::camera::Camera;
use scop::objects::Object;

use lineal::Vector;

use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), String> {
    let object = Object::parse("assets/teapot.obj").unwrap();

    let event_loop = EventLoop::new().map_err(|e| format!("Failed to create event loop: {}", e))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let camera = Camera::new(
        Vector::new([0., 0., -200.]),
        Vector::new([0., 0., 1.]),
        45.0f32.to_radians(),
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        0.1,
        500.,
    );

    let mut app = App::new(camera, object);

    let _ = event_loop.run_app(&mut app);

    return Ok(());
}
