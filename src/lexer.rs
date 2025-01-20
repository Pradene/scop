use std::fs::File;
use std::io::prelude::*;
use std::io::{
    self,
    BufReader
};

const BUFFER_SIZE: usize = 16;

#[derive(Debug, PartialEq)]
pub enum Token {
    Number(f32),
    Identifier(String),
    Group,
    Face,
    Vertice,
    Normal,
    Slash,
    EOF,
}

pub struct Lexer {
    reader: BufReader<File>,

    row: usize,
    col: usize,

    buffer: Vec<char>,
    buffer_position: usize,
}

impl Lexer {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(BUFFER_SIZE, file);

        let mut lexer = Lexer {
            reader: reader,
            col: 1,
            row: 1,
            buffer: Vec::new(),
            buffer_position: 0,
        };

        lexer.read_file()?;

        return Ok(lexer);
    }

    fn read_file(&mut self) -> io::Result<bool> {
        let mut bytes = [0; BUFFER_SIZE];
        
        match self.reader.read(&mut bytes)? {
            0 => {
                return Ok(false);
            }

            n => {
                self.buffer.clear();
                self.buffer.extend(String::from_utf8_lossy(&bytes[..n]).chars());
                self.buffer_position = 0;
                return Ok(true);
            }
        }
    }

    fn peek(&self) -> Option<char> {
        return self.buffer.get(self.buffer_position).copied();
    }

    fn advance(&mut self) -> io::Result<()> {
        if let Some(c) = self.peek() {
            if c == '\n' {
                self.row += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            
            self.buffer_position += 1;
            if self.buffer_position >= self.buffer.len() {
                self.read_file()?;
            }
        }

        return Ok(());
    }

    fn skip_whitespace(&mut self) -> io::Result<()> {
        while let Some(c) = self.peek() {
            match c {
                c if c.is_whitespace() => self.advance()?,
                _ => break,
            }
        }

        return Ok(());
    }

    pub fn next_token(&mut self) -> io::Result<Token> {
        self.skip_whitespace()?;

        if self.peek().is_none() {
            return Ok(Token::EOF);
        }
        
        while let Some(c) = self.peek() {
            match c {
                c if c.is_alphabetic() || c == '_' => {
                    return self.consume_identifier();
                }

                c if c.is_numeric() || c == '.'  || c == '-' => {
                    return self.consumer_number();
                }

                '/' => {
                    self.advance()?;
                    return Ok(Token::Slash);
                }

                _ => {
                    return Ok(Token::EOF);
                }
            }
        }

        return Ok(Token::EOF);
    }

    fn consume_identifier(&mut self) -> io::Result<Token> {
        let mut identifier = String::new();

        while let Some(c) = self.peek() {
            match c {
                c if c.is_alphabetic() || c == '_' => {
                    identifier.push(c);
                    self.advance()?;
                }

                _ => {
                    break;
                }
            }
        }
    
        // Return special tokens for specific identifiers
        match identifier.as_str() {
            "g" => Ok(Token::Group),
            "f" => Ok(Token::Face),
            "v" => Ok(Token::Vertice),
            "vn" => Ok(Token::Normal),
            _ => Ok(Token::Identifier(identifier)),
        }
    }

    fn consumer_number(&mut self) -> io::Result<Token> {
        let mut number = String::new();

        while let Some(c) = self.peek() {
            match c {
                c if c.is_numeric() || c == '.' || c == '-' => {
                    number.push(c);
                    self.advance()?;
                }

                _ => {
                    break;
                }
            }
        }

        if let Ok(n) = number.parse::<f32>() {
            Ok(Token::Number(n))
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid number"))
        }
    
    }
}