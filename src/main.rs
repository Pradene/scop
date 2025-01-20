use scop::lexer::Lexer;
use scop::lexer::Token;

use std::io::prelude::*;
use std::io::{
    self,
    BufReader
};

fn main() -> io::Result<()> {
    let pathname = "./assets/cube.obj";
    let mut lexer = Lexer::new(pathname).expect("Error");

    while let Ok(token) = lexer.next_token() {
        if token == Token::EOF {
            break;
        }

        println!("{:?}", token);
    }

    return Ok(());
}
