#![allow(dead_code)]

mod serializable;
mod transform;

use crate::serializable::Serializable;
use crate::transform::{command_word, ExtractCommand};
use conch_parser::ast;
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use log::*;
use sha2::{Digest, Sha256};
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::collections::HashMap;
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
    let mut spack_calls = HashMap::new();

    // Parse our input!
    let transformed = parser
        .into_iter()
        .filter_map(|line| {
            if let Ok(mut ast) = line {
                if let Some(cmd) = ast.extract("spack") {
                    if let Some(_) = cmd.redirects_or_cmd_words.iter().find(|x| match x {
                        command_word!(w) if w == "--list" => true,
                        _ => false,
                    }) {
                    } else {
                        let spack_call = cmd.clone();

                        let mut hasher = Sha256::new();
                        hasher.update(spack_call.into_string());
                        let result: String = format!("load_{:x}", hasher.finalize());

                        spack_calls.insert(String::from(&result), spack_call.clone());

                        cmd.redirects_or_cmd_words = vec![command_word!(result)];
                    }
                }
                Some(ast)
            } else {
                None
            }
        })
        .collect::<Vec<ast::TopLevelCommand<String>>>();

    for (hash, mut call) in spack_calls {
        if let Some(index) = call.redirects_or_cmd_words.iter().position(|x| match x {
            command_word!(w) if w == "load" => true,
            _ => false,
        }) {
            call.redirects_or_cmd_words
                .insert(index + 1, command_word!("--sh"));
            println!("HASH={} compile {}", hash, call.into_string());
        }
    }

    println!(
        "SCRIPT=\"\"\"{}\"\"\"",
        transformed
            .iter()
            .map(|ast| ast.into_string())
            .collect::<Vec<String>>()
            .join("\\n")
    );

    Ok(())
}
