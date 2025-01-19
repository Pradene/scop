use std::fs::File;
use std::io::prelude::*;
use std::io::{
    self,
    BufReader
};

pub fn parser(pathname: &str) -> io::Result<()> {
    let file = File::open(pathname)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let tokens: Vec<&str> = line.split_whitespace().collect();

        match tokens.first() {
            Some(&"g") => handle_group(tokens[1]),
            Some(&"f") => handle_face(&tokens[1..]),
            Some(&"v") => handle_vertice(&tokens[1..]),
            Some(&"vn") => handle_normal(&tokens[1..]),
            Some(&"#") => println!("Comment"),
            Some(&token) => eprintln!("Error: Unexpected token: {}", token),
            None => eprintln!("Error: Empty line"),
        }
    }

    Ok(())
}

fn handle_group(name: &str) {
    println!("{}", name);
}

fn handle_vertice(position: &[&str]) {
    if position.len() != 3 {
        return
    }

    println!("{} {} {}", position[0], position[1], position[2]);
}

fn handle_face(vertices: &[&str]) {
    if vertices.len() != 3 {
        return
    }

    println!("{} {} {}", vertices[0], vertices[1], vertices[2]);
}

fn handle_normal(normal: &[&str]) {
    if normal.len() != 3 {
        return
    }

    println!("{} {} {}", normal[0], normal[1], normal[2]);
}
