use scop::app::App;
use scop::objects::Object;

use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), String> {
    let object = Object::parse("assets/teapot.obj").unwrap();

    let event_loop = EventLoop::new().map_err(|e| format!("Failed to create event loop: {}", e))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(object);

    let _ = event_loop.run_app(&mut app);

    return Ok(());
}
