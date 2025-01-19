use scop::parser::parser;

fn main() {
    let pathname = "./assets/pyramid.obj";
    match parser(pathname) {
        Ok(()) => println!("File parsed"),
        Err(e) => eprintln!("Error reading file: {}", e)
    }
}
