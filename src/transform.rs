#![allow(unused_imports)]
use conch_parser::ast;

macro_rules! command_word {
    ($string:ident) => {
        ast::RedirectOrCmdWord::CmdWord(ast::TopLevelWord(ast::ComplexWord::Single(
            ast::Word::Simple(ast::SimpleWord::Literal($string)),
        )))
    };

    ($string:expr) => {
        ast::RedirectOrCmdWord::CmdWord(ast::TopLevelWord(ast::ComplexWord::Single(
            ast::Word::Simple(ast::SimpleWord::Literal(String::from($string))),
        )))
    };
}

pub(crate) use command_word;

pub trait ExtractCommand {
    // Return a SimpleCommand if it starts with the given string
    fn extract(&mut self, value: &str) -> Option<&mut ast::DefaultSimpleCommand>;
}

impl ExtractCommand for ast::DefaultSimpleCommand {
    fn extract(&mut self, value: &str) -> Option<&mut ast::DefaultSimpleCommand> {
        match self.redirects_or_cmd_words.first() {
            Some(command_word!(word)) if word == value => Some(self),
            _ => None,
        }
    }
}

impl ExtractCommand for ast::DefaultPipeableCommand {
    fn extract(&mut self, value: &str) -> Option<&mut ast::DefaultSimpleCommand> {
        match self {
            ast::PipeableCommand::Simple(l) => l.extract(value),
            _ => None,
        }
    }
}

impl ExtractCommand for ast::DefaultAndOrList {
    fn extract(&mut self, value: &str) -> Option<&mut ast::DefaultSimpleCommand> {
        match &mut self.first {
            ast::ListableCommand::Single(l) => l.extract(value),
            ast::ListableCommand::Pipe(_, v_l) => match v_l.first_mut() {
                Some(l) => l.extract(value),
                _ => None,
            },
        }
    }
}

impl ExtractCommand for ast::TopLevelCommand<String> {
    fn extract(&mut self, value: &str) -> Option<&mut ast::DefaultSimpleCommand> {
        match &mut self.0 {
            ast::Command::Job(l) => l.extract(value),
            ast::Command::List(l) => l.extract(value),
        }
    }
}
