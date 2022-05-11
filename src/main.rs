#![allow(dead_code)]

mod serializable;
mod transform;

use crate::serializable::Serializable;
use crate::transform::{command_word, ExtractCommand, FindCommandWord};
use conch_parser::ast;
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use log::*;
use sha2::{Digest, Sha256};
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
    let mut spack_source: ast::DefaultSimpleCommand = ast::DefaultSimpleCommand {
        redirects_or_env_vars: vec![],
        redirects_or_cmd_words: vec![],
    };
    let mut spack_calls = Vec::new();

    // Parse our input!
    let transformed = parser
        .into_iter()
        .filter_map(|line| {
            if let Ok(mut ast) = line {
                let mut export: bool = true;

                if let Some(cmd) = ast.extract(".") {
                    if cmd.position(".*setup-env.sh").is_some() {
                        spack_source = cmd.clone();
                        export = false;
                    }
                } else if let Some(cmd) = ast.extract("spack") {
                    if let Some(_) = cmd.position("--list") {
                        export = false;
                    } else {
                        let mut spack_call = cmd.clone();

                        let mut hasher = Sha256::new();
                        hasher.update(spack_call.into_string());
                        let result: String = format!("load_{:x}", hasher.finalize());

                        spack_call.redirects_or_env_vars = vec![];
                        spack_calls.push((String::from(&result), spack_call.clone()));

                        cmd.redirects_or_cmd_words = vec![command_word!(result)];
                    }
                }

                if export {
                    Some(ast)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<ast::TopLevelCommand<String>>>();

    let compile_directives = spack_calls
        .iter_mut()
        .filter_map(|tuple| {
            if let Some(index) = tuple.1.position("load") {
                tuple
                    .1
                    .redirects_or_cmd_words
                    .insert(index + 1, command_word!("--sh"));
                Some(format!(
                    "HASH={} compile {}",
                    tuple.0,
                    tuple.1.into_string()
                ))
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    println!(
        include_str!("template.sh.fmt"),
        spack_source.into_string(),
        transformed
            .iter()
            .map(|ast| ast.into_string())
            .collect::<Vec<String>>()
            .join("\n"),
        compile_directives
    );

    Ok(())
}
