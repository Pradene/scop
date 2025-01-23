use scop::parser::parse;
use scop::window::Window;

fn main() -> Result<(), String> {
    let path = "./assets/cube.obj";
    parse(path);

    let mut window = Window::new("Scop", 800, 600)?;
    return window.run();
}
