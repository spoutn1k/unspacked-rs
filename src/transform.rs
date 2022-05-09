#![allow(unused_imports)]
use conch_parser::ast;

/*
pub trait Transform {
    type NodeType;

    fn replace_match(&mut self, search: Self::NodeType, value: Self::NodeType) -> bool;
}

impl<L, P, S> Transform for ast::SimpleWord<L, P, S> {
    type NodeType = ast::SimpleWord<L, P, S>;

    fn replace_match(&mut self, _search: Self::NodeType, value: Self::NodeType) -> bool {
        if matches!(&self, _search) {
            *self = value;
            true
        } else {
            false
        }
    }
}*/

pub trait StartsWith {
    fn starts_with(&self, value: &str) -> bool;
}

impl<P, S> StartsWith for ast::SimpleWord<String, P, S> {
    fn starts_with(&self, value: &str) -> bool {
        match self {
            ast::SimpleWord::Literal(l) => *l == value,
            _ => false,
        }
    }
}

impl<L, W: StartsWith> StartsWith for ast::Word<L, W> {
    fn starts_with(&self, value: &str) -> bool {
        match self {
            ast::Word::Simple(w) => w.starts_with(value),
            _ => false,
        }
    }
}

impl StartsWith for ast::TopLevelWord<String> {
    fn starts_with(&self, value: &str) -> bool {
        match &self.0 {
            ast::ComplexWord::Single(w) => w.starts_with(value),
            ast::ComplexWord::Concat(v_w) => match v_w.first() {
                Some(word) => word.starts_with(value),
                _ => false,
            },
        }
    }
}

impl StartsWith for ast::DefaultSimpleCommand {
    fn starts_with(&self, value: &str) -> bool {
        match self.redirects_or_cmd_words.first() {
            Some(ast::RedirectOrCmdWord::CmdWord(word)) => word.starts_with(value),
            _ => false,
        }
    }
}

impl StartsWith for ast::DefaultPipeableCommand {
    fn starts_with(&self, value: &str) -> bool {
        match self {
            ast::PipeableCommand::Simple(l) => l.starts_with(value),
            _ => false,
        }
    }
}

impl StartsWith for ast::DefaultAndOrList {
    fn starts_with(&self, value: &str) -> bool {
        match &self.first {
            ast::ListableCommand::Single(l) => l.starts_with(value),
            ast::ListableCommand::Pipe(_, v_l) => match v_l.first() {
                Some(l) => l.starts_with(value),
                _ => false,
            },
        }
    }
}

impl StartsWith for ast::TopLevelCommand<String> {
    fn starts_with(&self, value: &str) -> bool {
        match &self.0 {
            ast::Command::Job(l) => l.starts_with(value),
            ast::Command::List(l) => l.starts_with(value),
        }
    }
}

pub trait InsertVersion {
    fn insert_v(&mut self) -> ();
}

impl InsertVersion for ast::DefaultSimpleCommand {
    fn insert_v(&mut self) -> () {
        self.redirects_or_cmd_words.insert(
            1,
            ast::RedirectOrCmdWord::CmdWord(ast::TopLevelWord(ast::ComplexWord::Single(
                ast::Word::Simple(ast::SimpleWord::Literal(String::from("-v"))),
            ))),
        );
    }
}

impl InsertVersion for ast::DefaultPipeableCommand {
    fn insert_v(&mut self) -> () {
        match self {
            ast::PipeableCommand::Simple(l) => l.insert_v(),
            _ => (),
        }
    }
}

impl InsertVersion for ast::DefaultAndOrList {
    fn insert_v(&mut self) -> () {
        match &mut self.first {
            ast::ListableCommand::Single(l) => l.insert_v(),
            ast::ListableCommand::Pipe(_, v_l) => match v_l.first_mut() {
                Some(l) => l.insert_v(),
                _ => (),
            },
        }
    }
}

impl InsertVersion for ast::TopLevelCommand<String> {
    fn insert_v(&mut self) -> () {
        match &mut self.0 {
            ast::Command::Job(l) => l.insert_v(),
            ast::Command::List(l) => l.insert_v(),
        }
    }
}
