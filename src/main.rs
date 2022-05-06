#![allow(dead_code)]

mod serializable;

use crate::serializable::Serializable;
use conch_parser::ast;
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
        if let Ok(cmd) = line {
            if is_spack_top_level(&cmd) {
                //println!("{:?}", ast);
                println!("{}", cmd.into_string());
            }
        }
    }

    Ok(())
}

fn is_spack_top_level(cmd: &ast::TopLevelCommand<String>) -> bool {
    match &cmd.0 {
        ast::Command::Job(list) | ast::Command::List(list) => is_spack_listable(&list.first),
    }
}

fn is_spack_listable(cmd: &ast::DefaultListableCommand) -> bool {
    match cmd {
        ast::ListableCommand::Single(cmd) => is_spack_pipeable(cmd),
        ast::ListableCommand::Pipe(_, cmds) => cmds
            .into_iter()
            .map(is_spack_pipeable)
            .fold(true, |acc, spk| acc || spk),
    }
}

fn is_spack_pipeable(cmd: &ast::DefaultPipeableCommand) -> bool {
    match cmd {
        ast::PipeableCommand::Simple(cmd) => is_spack_simple(cmd),
        ast::PipeableCommand::Compound(_) => false,
        ast::PipeableCommand::FunctionDef(_, _) => false,
    }
}

fn is_spack_simple(cmd: &ast::DefaultSimpleCommand) -> bool {
    let spack_refs = cmd
        .redirects_or_cmd_words
        .iter()
        .filter_map(|redirect_or_word| match redirect_or_word {
            ast::RedirectOrCmdWord::CmdWord(w) => Some(&w.0),
            ast::RedirectOrCmdWord::Redirect(_) => None,
        })
        .filter_map(|word| match word {
            ast::ComplexWord::Single(w) => Some(w),
            // We're going to ignore concatenated words for simplicity here
            ast::ComplexWord::Concat(_) => None,
        })
        .filter_map(|word| match word {
            ast::Word::SingleQuoted(w) => Some(w),
            ast::Word::Simple(w) => get_simple_word_as_string(w),
            ast::Word::DoubleQuoted(words) if words.len() == 1 => {
                get_simple_word_as_string(&words[0])
            }
            ast::Word::DoubleQuoted(_) => None, // Ignore all multi-word double quoted strings
        })
        .filter(|w| *w == "spack")
        .count();

    return spack_refs == 1;
}

fn get_simple_word_as_string(word: &ast::DefaultSimpleWord) -> Option<&String> {
    match word {
        ast::SimpleWord::Literal(w) => Some(w),
        _ => None, // Ignoring substitutions and others for simplicity here
    }
}
