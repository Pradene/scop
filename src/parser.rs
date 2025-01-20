use crate::object::Object;
use crate::lexer::{
    Lexer,
    Token
};

pub fn parse(path: &str) -> Vec<Object> {
    
    let mut objects: Vec<Object> = Vec::new();
    let mut lexer = Lexer::new(path).expect("Error");

    while let Ok(token) = lexer.next_token() {
        println!("{:?}", token);

        match token {
            Token::Group => {
                match lexer.next_token() {
                    Ok(Token::Identifier(name)) => {
                        objects.push(Object::new(&name));
                    }
                    
                    _ => {
                        eprintln!("Expected an identifier for group name, but got a different token.");
                        break;
                    }
                }
            }

            Token::Vertice => {
                // Handle the 'v' token (vertex), expecting 3 tokens for the coordinates
                let mut coordinates = Vec::new();
                for _ in 0..3 {
                    match lexer.next_token() {
                        Ok(Token::Number(num)) => {
                            coordinates.push(num); // Handle integer as float
                        }

                        _ => {
                            eprintln!("Expected a number for vertex coordinate, but got something else.");
                            break;
                        }
                    }
                }

                if coordinates.len() == 3 {
                    // Create a vertex with 3 coordinates
                    coordinates.push(0.);
                    println!("{:?}", coordinates);
                } else {
                    eprintln!("Error: Invalid number of vertex coordinates. Expected 3 but got {}", coordinates.len());
                }
            }

            Token::Normal => {
                // Handle the 'v' token (vertex), expecting 3 tokens for the coordinates
                let mut coordinates = Vec::new();
                for _ in 0..3 {
                    match lexer.next_token() {
                        Ok(Token::Number(num)) => {
                            coordinates.push(num); // Handle integer as float
                        }

                        _ => {
                            eprintln!("Expected a number for vertex coordinate, but got something else.");
                            break;
                        }
                    }
                }

                if coordinates.len() == 3 {
                    // Create a vertex with 3 coordinates
                    coordinates.push(1.);
                    println!("{:?}", coordinates);
                } else {
                    eprintln!("Error: Invalid number of vertex coordinates. Expected 3 but got {}", coordinates.len());
                }
            }

            Token::Face => {
                break;
            }

            Token::EOF => {
                break;
            }
            _ => {
                eprintln!("{:?} not implemented", token);
            }
        }
    }

    return objects;
}