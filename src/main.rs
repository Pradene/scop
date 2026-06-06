use std::time::Instant;

use scop::{app::App, math::Vec3, scene::Object};

fn main() -> Result<(), String> {
    let mut app: App = App::new()?;

    let mesh_id = app.load_mesh("assets/low_poly_fox.obj")?;

    let obj1 = Object::new(mesh_id);
    // let obj2 = Object::new(mesh_id);
    let obj1_id = app.add_object(obj1);
    // let obj2_id = app.add_object(obj2);

    // app.get_object(obj1_id)
    //     .set_scale(Vec3::new(2., 2., 2.))
    //     .set_rotation(0., 180f32.to_radians(), 0.)
    //     .translate(Vec3::new(100., 0., 0.));
    // app.get_object(obj2_id).translate(Vec3::new(-100., 0., 0.));

    let start = Instant::now();

    loop {
        if !app.handle_events()? {
            break;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(start).as_millis() as f32 / 1000.;
        let speed = 2.;
        let angle = speed * elapsed;

        app.get_object(obj1_id).set_rotation(0., angle, 0.);

        app.update();
        app.draw();
    }

    Ok(())
}
