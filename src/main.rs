use scop::app::App;
use scop::camera::Camera;
use scop::math::Vec3;
use scop::scene::{Object, Scene};

fn main() -> Result<(), String> {
    let width = 800u32;
    let height = 600u32;

    let camera = Camera::new(
        Vec3::Z * 200.,
        Vec3::Z * -1.,
        45f32.to_radians(),
        width as f32 / height as f32,
        0.1,
        500.,
    );

    let scene = Scene::new();
    let object =
        Object::parse("assets/teapot.obj").map_err(|e| format!("Failed to parse object: {}", e))?;

    let mut app = App::new(scene, camera, width, height)?;

    app.renderer
        .upload_mesh(&object)
        .map_err(|e| format!("Failed to upload mesh: {}", e))?;
    app.scene.add(object);

    loop {
        if !app.handle_events()? {
            break;
        }
        app.update();
        app.draw();
    }

    Ok(())
}
