#![allow(dead_code)]

mod serializable;
mod transform;

use crate::serializable::Serializable;
use crate::transform::{InsertVersion, StartsWith};
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use log::*;
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let args = env::args().collect::<Vec<String>>();
    let filename = &args[1];

    debug!("Reading file: {}", filename);
    let contents = fs::read_to_string(filename).expect("Error reading file");

    // Initialize our token lexer and shell parser with the first argument
    let lex = Lexer::new(contents.chars());
    let parser = DefaultParser::new(lex);

    // Parse our input!
    for line in parser {
        if let Ok(mut ast) = line {
            if ast.starts_with("spack") {
                ast.insert_v();
            }
            println!("{:?}", ast);
            println!("{}", ast.into_string());
        }
    }

    Ok(())
}
