extern crate unspacklib;

use conch_parser::ast;
use conch_parser::lexer::Lexer;
use conch_parser::parse::DefaultParser;
use log::*;
use sha2::{Digest, Sha256};
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::process::exit;
use std::{env, fs};
use unspacklib::serializable::Serializable;
use unspacklib::{
    command_word,
    transform::{ExtractCommand, FindCommandWord},
};

static COMPILE_FUNC_NAME: &str = "__unspacked_rs_compile";

fn filter_parser(
    contents: String,
    spack_calls: &mut Vec<(String, ast::DefaultSimpleCommand)>,
    spack_source: &mut ast::DefaultSimpleCommand,
) -> Vec<ast::TopLevelCommand<String>> {
    // Initialize our token lexer and shell parser with the first argument
    let lex = Lexer::new(contents.chars());
    let parser = DefaultParser::new(lex);

    parser
        .into_iter()
        .filter_map(|line| {
            if let Ok(mut ast) = line {
                let mut export: bool = true;

                if let Some(cmd) = ast.extract(".") {
                    // Here we look for a spack setup source script
                    if cmd.position(".*setup-env.sh").is_some() {
                        *spack_source = cmd.clone();
                        export = false;
                    }
                } else if let Some(cmd) = ast.extract("spack") {
                    export = false;
                    // Next we match lines beginning with spack load and no list
                    if let Some(_) = cmd.position("load") {
                        if let None = cmd.position("--list") {
                            let mut spack_call = cmd.clone();

                            let mut hasher = Sha256::new();
                            hasher.update(spack_call.into_string());
                            let result: String = format!("load_{:x}", hasher.finalize());

                            spack_call.redirects_or_env_vars = vec![];
                            spack_calls.push((String::from(&result), spack_call.clone()));

                            cmd.redirects_or_cmd_words = vec![command_word!(result)];
                            export = true;
                        }
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
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::init(LevelFilter::Info, Config::default()).unwrap();

    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        eprintln!("Usage: {} <spacked script>", &args[0]);
        exit(1);
    }

    let filename = &args[1];

    debug!("Reading file: {}", filename);
    let contents = fs::read_to_string(filename).expect("Error reading file");

    // This will contain the spack setup source call
    let mut spack_source: ast::DefaultSimpleCommand = ast::DefaultSimpleCommand {
        redirects_or_env_vars: vec![],
        redirects_or_cmd_words: vec![],
    };
    // This will hold information about the spack calls found in the script
    let mut spack_calls = Vec::new();

    // Parse our input!
    let transformed = filter_parser(contents, &mut spack_calls, &mut spack_source);

    let compile_directives = spack_calls
        .iter_mut()
        .filter_map(|tuple| {
            if let Some(index) = tuple.1.position("load") {
                tuple
                    .1
                    .redirects_or_cmd_words
                    .insert(index + 1, command_word!("--sh"));
                Some(format!(
                    "HASH={} {} {}",
                    tuple.0,
                    COMPILE_FUNC_NAME,
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
        COMPILE_FUNC_NAME,
        transformed
            .iter()
            .map(|ast| ast.into_string())
            .collect::<Vec<String>>()
            .join("\n"),
        compile_directives
    );

    Ok(())
}
