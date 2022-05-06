use conch_parser::ast;

pub trait Serializable<S> {
    fn into_string(&self) -> S;
}

impl Serializable<String> for String {
    fn into_string(&self) -> String {
        String::from(self)
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::TopLevelCommand<S> {
    fn into_string(&self) -> String {
        self.0.into_string()
    }
}

impl<T: Serializable<String>> Serializable<String> for ast::Command<T> {
    fn into_string(&self) -> String {
        match self {
            ast::Command::Job(list) => format!("{} &", list.into_string()),
            ast::Command::List(list) => list.into_string(),
        }
    }
}

impl<L: Serializable<String>, P: Serializable<String>, S: Serializable<String>> Serializable<String>
    for ast::SimpleWord<L, P, S>
{
    fn into_string(&self) -> String {
        match self {
            ast::SimpleWord::Literal(l) => l.into_string(),
            ast::SimpleWord::Escaped(e) => format!("\\{}", e.into_string()),
            ast::SimpleWord::Star => String::from("*"),
            ast::SimpleWord::Question => String::from("?"),
            ast::SimpleWord::SquareOpen => String::from("["),
            ast::SimpleWord::SquareClose => String::from("]"),
            ast::SimpleWord::Tilde => String::from("~"),
            ast::SimpleWord::Colon => String::from(":"),
            ast::SimpleWord::Param(value) => value.into_string(),
            ast::SimpleWord::Subst(value) => value.into_string(),
        }
    }
}

impl<
        P: Serializable<String>,
        W: Serializable<String>,
        C: Serializable<String>,
        A: Serializable<String>,
    > Serializable<String> for ast::ParameterSubstitution<P, W, C, A>
{
    fn into_string(&self) -> String {
        match self {
            ast::ParameterSubstitution::Command(vec) => {
                let cmd = vec
                    .iter()
                    .map(|c| c.into_string())
                    .collect::<Vec<String>>()
                    .join(" ");

                format!("$({})", cmd)
            }
            ast::ParameterSubstitution::Len(value) => format!("${{#{}}}", value.into_string()),
            ast::ParameterSubstitution::Arith(option) => match option {
                Some(value) => format!("$(({}))", value.into_string()),
                None => String::from("$(())"),
            },
            _ => String::from("UNSUPPORTED"),
        }
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::Arithmetic<S> {
    fn into_string(&self) -> String {
        String::from("UNSUPPORTED")
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::AndOr<S> {
    fn into_string(&self) -> String {
        String::from("UNSUPPORTED")
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::Parameter<S> {
    fn into_string(&self) -> String {
        match self {
            ast::Parameter::At => String::from("$@"),
            ast::Parameter::Star => String::from("$*"),
            ast::Parameter::Question => String::from("$?"),
            ast::Parameter::Pound => String::from("$#"),
            ast::Parameter::Dash => String::from("$-"),
            ast::Parameter::Dollar => String::from("$$"),
            ast::Parameter::Bang => String::from("$!"),
            ast::Parameter::Positional(value) => format!("${}", value),
            ast::Parameter::Var(value) => format!("${}", value.into_string()),
        }
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::ComplexWord<S> {
    fn into_string(&self) -> String {
        match self {
            ast::ComplexWord::Single(w) => w.into_string(),
            ast::ComplexWord::Concat(vec) => vec
                .iter()
                .map(|w| w.into_string())
                .collect::<Vec<String>>()
                .join(""),
        }
    }
}

impl<L: Serializable<String>, W: Serializable<String>> Serializable<String> for ast::Word<L, W> {
    fn into_string(&self) -> String {
        match self {
            ast::Word::Simple(w) => w.into_string(),
            ast::Word::DoubleQuoted(w) => {
                let contents = w
                    .iter()
                    .map(|x| x.into_string())
                    .collect::<Vec<String>>()
                    .join("");
                format!("\"{}\"", contents)
            }
            _ => String::from("UNSUPPORTED"),
        }
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::TopLevelWord<S> {
    fn into_string(&self) -> String {
        self.0.into_string()
    }
}

impl<V: Serializable<String>, W: Serializable<String>, C: Serializable<String>> Serializable<String>
    for ast::CompoundCommandKind<V, W, C>
{
    fn into_string(&self) -> String {
        match self {
            ast::CompoundCommandKind::Brace(vec) => {
                let commands = vec
                    .iter()
                    .map(|x| x.into_string())
                    .collect::<Vec<String>>()
                    .join("; ");

                format!("{{ {}; }}", commands)
            }
            _ => String::from("UNSUPPORTED"),
        }
    }
}

impl<T: Serializable<String>, R: Serializable<String>> Serializable<String>
    for ast::CompoundCommand<T, R>
{
    fn into_string(&self) -> String {
        let compound = self.kind.into_string();

        let io = self
            .io
            .iter()
            .map(|x| x.into_string())
            .collect::<Vec<String>>()
            .join("  ");

        vec![compound, io]
            .into_iter()
            .filter(|x| x.len() != 0)
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl<L: Serializable<String>, W: Serializable<String>, R: Serializable<String>> Serializable<String>
    for ast::SimpleCommand<L, W, R>
{
    fn into_string(&self) -> String {
        let roev = self
            .redirects_or_env_vars
            .iter()
            .map(|roev| roev.into_string())
            .collect::<Vec<String>>()
            .join(" ");

        let rocw = self
            .redirects_or_cmd_words
            .iter()
            .map(|rocw| rocw.into_string())
            .collect::<Vec<String>>()
            .join(" ");

        // The following adds a space only when necessary
        vec![roev, rocw]
            .into_iter()
            .filter(|x| x.len() != 0)
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl<T: Serializable<String>> Serializable<String> for ast::ListableCommand<T> {
    fn into_string(&self) -> String {
        match self {
            ast::ListableCommand::Single(cmd) => cmd.into_string(),
            ast::ListableCommand::Pipe(_, cmds) => cmds
                .into_iter()
                .map(|cmd| cmd.into_string())
                .collect::<Vec<String>>()
                .join(" "),
        }
    }
}

impl<N, S: Serializable<String>, C: Serializable<String>, F> Serializable<String>
    for ast::PipeableCommand<N, S, C, F>
{
    fn into_string(&self) -> String {
        match self {
            ast::PipeableCommand::Simple(cmd) => cmd.into_string(),
            ast::PipeableCommand::Compound(cmd) => cmd.into_string(),
            ast::PipeableCommand::FunctionDef(_, _) => String::from("UNSUPPORTED"),
        }
    }
}

impl<T: Serializable<String>> Serializable<String> for ast::AndOrList<T> {
    fn into_string(&self) -> String {
        let first = self.first.into_string();

        let rest = self
            .rest
            .iter()
            .map(|cmd| match cmd {
                ast::AndOr::And(dlc) => format!("&& {}", dlc.into_string()),
                ast::AndOr::Or(dlc) => format!("|| {}", dlc.into_string()),
            })
            .collect::<Vec<String>>()
            .join(" ");

        // The following adds a space only when necessary
        vec![first, rest]
            .into_iter()
            .filter(|x| x.len() != 0)
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl<T: Serializable<String>> Serializable<String> for ast::Redirect<T> {
    fn into_string(&self) -> String {
        match self {
            ast::Redirect::Read(fd_option, rest) => match fd_option {
                Some(value) => format!("{}< {}", value, rest.into_string()),
                None => format!("< {}", rest.into_string()),
            },
            ast::Redirect::Write(fd_option, rest) => match fd_option {
                Some(value) => format!("{}> {}", value, rest.into_string()),
                None => format!("> {}", rest.into_string()),
            },
            ast::Redirect::ReadWrite(fd_option, rest) => match fd_option {
                Some(value) => format!("{}<> {}", value, rest.into_string()),
                None => format!("<> {}", rest.into_string()),
            },
            ast::Redirect::Append(fd_option, rest) => match fd_option {
                Some(value) => format!("{}>> {}", value, rest.into_string()),
                None => format!(">> {}", rest.into_string()),
            },
            ast::Redirect::Clobber(fd_option, rest) => match fd_option {
                Some(value) => format!("{}>| {}", value, rest.into_string()),
                None => format!(">| {}", rest.into_string()),
            },
            _ => String::from("UNSUPPORTED"),
        }
    }
}

impl<R: Serializable<String>, W: Serializable<String>> Serializable<String>
    for ast::RedirectOrCmdWord<R, W>
{
    fn into_string(&self) -> String {
        match self {
            ast::RedirectOrCmdWord::CmdWord(w) => w.into_string(),
            ast::RedirectOrCmdWord::Redirect(r) => r.into_string(),
        }
    }
}

impl<R: Serializable<String>, V: Serializable<String>, W: Serializable<String>> Serializable<String>
    for ast::RedirectOrEnvVar<R, V, W>
{
    fn into_string(&self) -> String {
        match self {
            ast::RedirectOrEnvVar::EnvVar(k, Some(v)) => {
                format!("{}={}", k.into_string(), v.into_string())
            }
            ast::RedirectOrEnvVar::EnvVar(k, None) => format!("{}=", k.into_string()),
            ast::RedirectOrEnvVar::Redirect(r) => r.into_string(),
        }
    }
}

impl<B: Serializable<String>> Serializable<String> for Box<B> {
    fn into_string(&self) -> String {
        (**self).into_string()
    }
}
