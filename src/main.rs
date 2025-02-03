use scop::app::App;
use scop::objects::Object;

use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), String> {
    let object = Object::parse("assets/teapot2.obj").unwrap();

    // Create the event loop
    let event_loop = EventLoop::new().map_err(|e| format!("Failed to create event loop: {}", e))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create the application instance
    let mut app = App::new(object);

    // Run the application
    let _ = event_loop.run_app(&mut app);

    return Ok(());
}
