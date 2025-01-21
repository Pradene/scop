use crate::object::{
    Object,
    Group
};
use crate::lexer::{
    Lexer,
    Token
};

pub fn parse(path: &str) -> Object {
    
    let mut lexer = Lexer::new(path).expect("Error");
    
    let mut object = Object::new();
    let mut group = Group::new();

    while let Ok(token) = lexer.next_token() {
        println!("{:?}", token);

        match token {
            Token::Group => {
                match lexer.next_token() {
                    Ok(Token::Identifier(name)) => {
                        group.name = name;
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
                    break;
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
                    break;
                }
            }

            Token::Face => {
                let mut face_indices = Vec::new();
                
                loop {
                    let next_token = lexer.peek_token();
                    match next_token {
                        Ok(Token::Number(index)) => {
                            face_indices.push((index as usize).saturating_sub(1));
                            let _ = lexer.next_token(); // Actually consume the token
                        }
                        Ok(Token::Slash) => {
                            let _ = lexer.next_token(); // Consume slash
                            // Skip texture/normal indices for now
                            if let Ok(Token::Number(_)) = lexer.next_token() {
                                // Skip the texture coordinate index
                            }
                        }
                        Ok(_) => {
                            // Print the current face and exit this loop
                            if face_indices.len() >= 3 {
                                println!("Face indices: {:?}", face_indices);
                            }
                            break;
                        }
                        Err(_) => break,
                    }
                }
            }

            Token::Comment(_) => {
                continue;
            }

            Token::EOF => {
                break;
            }
            
            _ => {
                eprintln!("{:?} not implemented", token);
            }
        }
    }

    return object;
}