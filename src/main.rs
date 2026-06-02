use std::path::Path;
use std::time::Instant;

use scop::app::App;
use scop::camera::Camera;
use scop::math::{Mat4, Vec3};

fn main() -> Result<(), String> {
    let width = 800u32;
    let height = 600u32;

    let camera = Camera::new(
        Vec3::new(0., 0., -200.),
        Vec3::ZERO,
        45f32.to_radians(),
        width as f32 / height as f32,
        0.1,
        500.,
    );

    let mut app: App = App::new(camera, width, height)?;
    let handle = app.load_object(Path::new("assets/teapot.obj"))?;

    let start = Instant::now();

    loop {
        if !app.handle_events()? {
            break;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(start).as_millis() as f32 / 1000.;
        let speed = 2.;
        let angle = speed * elapsed;
        let transform = Mat4::identity().rotate(angle, Vec3::Y);

        app.get_object(handle).update_transform(transform);

        app.update();
        app.draw();
    }

    Ok(())
}
