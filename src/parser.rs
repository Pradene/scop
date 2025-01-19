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
        println!("{}", line?);
    }

    Ok(())
}