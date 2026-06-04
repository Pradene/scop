use std::time::Instant;

use scop::{app::App, math::{Mat4, Vec3}, scene::Object};

fn main() -> Result<(), String> {
    let mut app: App = App::new()?;

    let mesh_id = app.load_mesh("assets/teapot.obj")?;

    let object = Object::new(mesh_id);
    let object_id = app.add_object(object);

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

        app.get_object(object_id).set_rotation(0., angle, 0.);

        app.update();
        app.draw();
    }

    Ok(())
}
