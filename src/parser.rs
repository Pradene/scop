use crate::object::{
    Object,
    Group
};
use crate::lexer::{
    Lexer,
    Token
};

use lineal::Vector;

pub fn parse(path: &str) -> Object {
    
    let mut lexer = Lexer::new(path).expect("Error");
    
    let mut object = Object::new();
    let mut group = Group::new();

    while let Ok(token) = lexer.next_token() {
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
                let mut coordinates = Vec::new();
                for _ in 0..3 {
                    match lexer.next_token() {
                        Ok(Token::Number(num)) => {
                            coordinates.push(num);
                        }

                        _ => {
                            eprintln!("Expected a number for vertex coordinate, but got something else.");
                            break;
                        }
                    }
                }

                if coordinates.len() == 3 {
                    coordinates.push(0.);
                    group.vertices.push(Vector::try_from(coordinates).unwrap());
                } else {
                    eprintln!("Error: Invalid number of vertex coordinates. Expected 3 but got {}", coordinates.len());
                    break;
                }
            }

            Token::Normal => {
                let mut coordinates = Vec::new();
                for _ in 0..3 {
                    match lexer.next_token() {
                        Ok(Token::Number(num)) => {
                            coordinates.push(num);
                        }

                        _ => {
                            eprintln!("Expected a number for vertex coordinate, but got something else.");
                            break;
                        }
                    }
                }

                if coordinates.len() == 3 {
                    coordinates.push(1.);
                    group.vertices.push(Vector::try_from(coordinates).unwrap());
                } else {
                    eprintln!("Error: Invalid number of vertex coordinates. Expected 3 but got {}", coordinates.len());
                    break;
                }
            }

            Token::Face => {
                let mut face = Vec::new();
                
                loop {
                    let next_token = lexer.peek_token();
                    match next_token {
                        Ok(Token::Number(index)) => {
                            let vertex_index = (index as usize).saturating_sub(1);
                            let _ = lexer.next_token();
                        
                            let mut texture_index = None;
                            let mut normal_index = None;
                        
                            if let Ok(Token::Slash) = lexer.peek_token() {
                                let _ = lexer.next_token();
                            
                                if let Ok(Token::Number(index)) = lexer.peek_token() {
                                    texture_index = Some((index as usize).saturating_sub(1));
                                    let _ = lexer.next_token();
                                }
                            
                                if let Ok(Token::Slash) = lexer.peek_token() {
                                    let _ = lexer.next_token();
                                
                                    if let Ok(Token::Number(index)) = lexer.peek_token() {
                                        normal_index = Some((index as usize).saturating_sub(1));
                                        let _ = lexer.next_token();
                                    }
                                }
                            }
                        
                            face.push((vertex_index, texture_index, normal_index));
                        }
                    
                        Ok(_) => break,
                        Err(_) => break,
                    }        
                }

                group.faces.push(face);
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

    println!("{:#?}", group);

    return object;
}