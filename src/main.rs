use std::path::Path;
// use std::time::Instant;

use scop::app::App;
use scop::parser::ObjFileParser;

fn main() -> Result<(), String> {
    let mut app: App = App::new()?;

    let path = Path::new("assets/teapot.obj");
    let mesh = ObjFileParser::parse(path)?;

    app.add_object(mesh)?;

    // let start = Instant::now();

    loop {
        if !app.handle_events()? {
            break;
        }

        // let now = Instant::now();
        // let elapsed = now.duration_since(start).as_millis() as f32 / 1000.;
        // let speed = 2.;
        // let angle = speed * elapsed;
        // let transform = Mat4::identity().rotate(angle, Vec3::Y);

        // app.get_object(handle).update_transform(transform);

        app.update();
        app.draw();
    }

    Ok(())
}
