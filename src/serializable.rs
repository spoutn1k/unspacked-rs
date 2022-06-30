use conch_parser::ast;
use std::rc::Rc;

pub trait Serializable<S> {
    fn into_string(&self) -> S;
}

impl Serializable<String> for String {
    fn into_string(&self) -> String {
        String::from(self)
    }
}

impl<B: Serializable<String>> Serializable<String> for Box<B> {
    fn into_string(&self) -> String {
        (**self).into_string()
    }
}

impl<B: Serializable<String>> Serializable<String> for Rc<B> {
    fn into_string(&self) -> String {
        (**self).into_string()
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::TopLevelCommand<S> {
    fn into_string(&self) -> String {
        self.0.into_string()
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::TopLevelWord<S> {
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

macro_rules! substitution {
    // I'm sorry
    ($colon:expr, $prefix:expr, $suffix:expr, $symbol:expr) => {
        match ($colon, $prefix, $suffix) {
            (true, ast::Parameter::Var(v), Some(value)) => format!(
                "${{{}:{}{}}}",
                v.into_string(),
                $symbol,
                value.into_string()
            ),
            (true, ast::Parameter::Var(v), None) => {
                format!("${{{}:{}}}", v.into_string(), $symbol)
            }
            (true, v, Some(value)) => format!(
                "${{{}:{}{}}}",
                v.into_string(),
                $symbol,
                value.into_string()
            ),
            (true, v, None) => {
                format!("${{{}:{}}}", v.into_string(), $symbol)
            }
            (false, ast::Parameter::Var(v), Some(value)) => {
                format!("${{{}{}{}}}", v.into_string(), $symbol, value.into_string())
            }
            (false, ast::Parameter::Var(v), None) => {
                format!("${{{}{}}}", v.into_string(), $symbol)
            }
            (false, v, Some(value)) => {
                format!("${{{}{}{}}}", v.into_string(), $symbol, value.into_string())
            }
            (false, v, None) => {
                format!("${{{}{}}}", v.into_string(), $symbol)
            }
        }
    };
}

macro_rules! join {
    // Iter over vec, serialize contents and join using the provided separator
    ($vec:expr, $sep:expr) => {
        $vec.iter()
            .map(|x| x.into_string())
            .collect::<Vec<String>>()
            .join($sep)
    };
}

impl<
        P: Serializable<String>,
        W: Serializable<String>,
        C: Serializable<String>,
        A: Serializable<String>,
    > Serializable<String> for ast::ParameterSubstitution<ast::Parameter<P>, W, C, A>
{
    fn into_string(&self) -> String {
        match self {
            ast::ParameterSubstitution::Command(vec) => {
                format!("$({})", join!(vec, " "))
            }
            ast::ParameterSubstitution::Len(ast::Parameter::Var(value)) => {
                format!("${{#{}}}", value.into_string())
            }
            ast::ParameterSubstitution::Len(value) => {
                format!("${{#{}}}", value.into_string())
            }
            ast::ParameterSubstitution::Arith(option) => match option {
                Some(value) => format!("$(({}))", value.into_string()),
                None => String::from("$(())"),
            },
            ast::ParameterSubstitution::Default(colon, prefix, suffix) => {
                substitution!(colon, prefix, suffix, "-")
            }
            ast::ParameterSubstitution::Assign(colon, prefix, suffix) => {
                substitution!(colon, prefix, suffix, "=")
            }
            ast::ParameterSubstitution::Error(colon, prefix, suffix) => {
                substitution!(colon, prefix, suffix, "?")
            }
            ast::ParameterSubstitution::Alternative(colon, prefix, suffix) => {
                substitution!(colon, prefix, suffix, "+")
            }
            ast::ParameterSubstitution::RemoveSmallestPrefix(prefix, suffix) => {
                substitution!(false, prefix, suffix, "%")
            }
            ast::ParameterSubstitution::RemoveLargestPrefix(prefix, suffix) => {
                substitution!(false, prefix, suffix, "%%")
            }
            ast::ParameterSubstitution::RemoveSmallestSuffix(prefix, suffix) => {
                substitution!(false, prefix, suffix, "#")
            }
            ast::ParameterSubstitution::RemoveLargestSuffix(prefix, suffix) => {
                substitution!(false, prefix, suffix, "##")
            }
        }
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::Arithmetic<S> {
    fn into_string(&self) -> String {
        match self {
            ast::Arithmetic::Var(value) => value.into_string(),
            ast::Arithmetic::Literal(value) => format!("{}", value),
            ast::Arithmetic::Pow(lhs, rhs) => {
                format!("{} ** {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::PostIncr(value) => format!("{}++", value.into_string()),
            ast::Arithmetic::PostDecr(value) => format!("{}--", value.into_string()),
            ast::Arithmetic::PreIncr(value) => format!("++{}", value.into_string()),
            ast::Arithmetic::PreDecr(value) => format!("--{}", value.into_string()),
            ast::Arithmetic::UnaryPlus(value) => format!("+({})", value.into_string()),
            ast::Arithmetic::UnaryMinus(value) => format!("-({})", value.into_string()),
            ast::Arithmetic::LogicalNot(value) => format!("!{}", value.into_string()),
            ast::Arithmetic::BitwiseNot(value) => format!("~{}", value.into_string()),
            ast::Arithmetic::Mult(lhs, rhs) => {
                format!("{} * {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::Div(lhs, rhs) => {
                format!("{} / {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::Modulo(lhs, rhs) => {
                format!("{} % {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::Add(lhs, rhs) => {
                format!("{} + {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::Sub(lhs, rhs) => {
                format!("{} - {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::ShiftLeft(lhs, rhs) => {
                format!("{} << {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::ShiftRight(lhs, rhs) => {
                format!("{} >> {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::Less(lhs, rhs) => {
                format!("{} < {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::LessEq(lhs, rhs) => {
                format!("{} <= {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::Great(lhs, rhs) => {
                format!("{} > {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::GreatEq(lhs, rhs) => {
                format!("{} >= {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::Eq(lhs, rhs) => {
                format!("{} == {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::NotEq(lhs, rhs) => {
                format!("{} != {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::BitwiseAnd(lhs, rhs) => {
                format!("{} & {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::BitwiseXor(lhs, rhs) => {
                format!("{} ^ {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::BitwiseOr(lhs, rhs) => {
                format!("{} | {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::LogicalAnd(lhs, rhs) => {
                format!("{} && {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::LogicalOr(lhs, rhs) => {
                format!("{} || {}", lhs.into_string(), rhs.into_string())
            }
            ast::Arithmetic::Ternary(cond, lhs, rhs) => {
                format!(
                    "{} ? {} : {}",
                    cond.into_string(),
                    lhs.into_string(),
                    rhs.into_string()
                )
            }
            ast::Arithmetic::Assign(ident, value) => {
                format!("{} = {}", ident.into_string(), value.into_string(),)
            }
            ast::Arithmetic::Sequence(values) => {
                join!(values, ", ")
            }
        }
    }
}

impl<S: Serializable<String>> Serializable<String> for ast::AndOr<S> {
    fn into_string(&self) -> String {
        match self {
            ast::AndOr::And(dlc) => format!("&& {}", dlc.into_string()),
            ast::AndOr::Or(dlc) => format!("|| {}", dlc.into_string()),
        }
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
            ast::ComplexWord::Concat(vec) => join!(vec, ""),
        }
    }
}

impl<L: Serializable<String>, W: Serializable<String>> Serializable<String> for ast::Word<L, W> {
    fn into_string(&self) -> String {
        match self {
            ast::Word::Simple(w) => w.into_string(),
            ast::Word::DoubleQuoted(w) => format!("\"{}\"", join!(w, "")),
            ast::Word::SingleQuoted(w) => w.into_string(),
        }
    }
}

impl<V: Serializable<String>, W: Serializable<String>, C: Serializable<String>> Serializable<String>
    for ast::CompoundCommandKind<V, W, C>
{
    fn into_string(&self) -> String {
        match self {
            ast::CompoundCommandKind::Brace(vec) => format!("{{ {}; }}", join!(vec, "; ")),
            ast::CompoundCommandKind::Subshell(vec) => format!("( {} )", join!(vec, "; ")),
            ast::CompoundCommandKind::While(gbp) => {
                let guard = join!(gbp.guard, "; ");
                let body = join!(gbp.body, "\n");
                format!("while {}; do\n{}\ndone", guard, body)
            }
            ast::CompoundCommandKind::Until(gbp) => {
                let guard = join!(gbp.guard, "; ");
                let body = join!(gbp.body, "\n");
                format!("until {}; do\n{}\ndone", guard, body)
            }
            ast::CompoundCommandKind::If {
                conditionals,
                else_branch,
            } => {
                // conditionals is a list of all conditions,actions of a if/elif list - we proceed
                // by formatting them all the same way - as a regular if - and then join them using
                // the "el" delimiter, transforming all but the first member into elifs
                let cases = conditionals
                    .iter()
                    .map(|gbp| {
                        let guard = join!(gbp.guard, "; ");
                        let body = join!(gbp.body, "\n");
                        format!("if {}; then\n{}", guard, body)
                    })
                    .collect::<Vec<String>>();

                if let Some(statements) = else_branch {
                    format!(
                        "{}\nelse\n{}\nfi",
                        join!(cases, "\nel"),
                        join!(statements, "\n")
                    )
                } else {
                    format!("{}\nfi", join!(cases, "\nel"))
                }
            }
            ast::CompoundCommandKind::For { var, words, body } => {
                if let Some(word_v) = words {
                    format!(
                        "for {} in {}; do\n{}\ndone",
                        var.into_string(),
                        join!(word_v, " "),
                        join!(body, "\n")
                    )
                } else {
                    format!("for {}; do\n{}\ndone", var.into_string(), join!(body, "\n"))
                }
            }
            ast::CompoundCommandKind::Case { word, arms } => {
                format!("case {} in {}\nesac", word.into_string(), join!(arms, "\n"))
            }
        }
    }
}

impl<W: Serializable<String>, C: Serializable<String>> Serializable<String>
    for ast::PatternBodyPair<W, C>
{
    fn into_string(&self) -> String {
        format!(
            "{})\n{}\n;;",
            join!(self.patterns, "|"),
            join!(self.body, "\n")
        )
    }
}

impl<T: Serializable<String>, R: Serializable<String>> Serializable<String>
    for ast::CompoundCommand<T, R>
{
    fn into_string(&self) -> String {
        let compound = self.kind.into_string();

        let io = join!(self.io, " ");

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
        let roev = join!(self.redirects_or_env_vars, " ");
        let rocw = join!(self.redirects_or_cmd_words, " ");

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
            ast::ListableCommand::Pipe(_, cmds) => join!(cmds, " "),
        }
    }
}

impl<
        N: Serializable<String>,
        S: Serializable<String>,
        C: Serializable<String>,
        F: Serializable<String>,
    > Serializable<String> for ast::PipeableCommand<N, S, C, F>
{
    fn into_string(&self) -> String {
        match self {
            ast::PipeableCommand::Simple(cmd) => cmd.into_string(),
            ast::PipeableCommand::Compound(cmd) => cmd.into_string(),
            ast::PipeableCommand::FunctionDef(name, body) => {
                format!(
                    "function {}() {{\n{}\n}}",
                    name.into_string(),
                    body.into_string()
                )
            }
        }
    }
}

impl<T: Serializable<String>> Serializable<String> for ast::AndOrList<T> {
    fn into_string(&self) -> String {
        let first = self.first.into_string();
        let rest = join!(self.rest, " ");

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
            ast::Redirect::DupRead(fd_option, rest) => match fd_option {
                Some(value) => format!("{}<& {}", value, rest.into_string()),
                None => format!("<& {}", rest.into_string()),
            },
            ast::Redirect::DupWrite(fd_option, rest) => match fd_option {
                Some(value) => format!("{}>& {}", value, rest.into_string()),
                None => format!(">& {}", rest.into_string()),
            },
            ast::Redirect::Heredoc(fd_option, rest) => match fd_option {
                Some(value) => format!("{} in {}", rest.into_string(), value),
                None => format!("<<EOF\n{}\nEOF", rest.into_string()),
            },
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serialize_string() {
        assert_eq!(String::from("test").into_string(), String::from("test"));
    }

    #[test]
    fn test_serialize_parameter() {
        let positional = 1;

        assert_eq!(
            ast::Parameter::<String>::At.into_string(),
            String::from("$@")
        );
        assert_eq!(
            ast::Parameter::<String>::Star.into_string(),
            String::from("$*")
        );
        assert_eq!(
            ast::Parameter::<String>::Question.into_string(),
            String::from("$?")
        );
        assert_eq!(
            ast::Parameter::<String>::Pound.into_string(),
            String::from("$#")
        );
        assert_eq!(
            ast::Parameter::<String>::Dash.into_string(),
            String::from("$-")
        );
        assert_eq!(
            ast::Parameter::<String>::Dollar.into_string(),
            String::from("$$")
        );
        assert_eq!(
            ast::Parameter::<String>::Bang.into_string(),
            String::from("$!")
        );
        assert_eq!(
            ast::Parameter::<String>::Positional(positional).into_string(),
            format!("${}", positional)
        );
        assert_eq!(
            ast::Parameter::<String>::Var(String::from("test")).into_string(),
            String::from("$test")
        );
    }

    #[test]
    fn test_serialize_arithmetic() {
        assert_eq!(
            ast::Arithmetic::<String>::Var(String::from("test")).into_string(),
            String::from("test")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Literal(1).into_string(),
            format!("{}", 1)
        );

        assert_eq!(
            ast::Arithmetic::<String>::Pow(
                Box::new(ast::Arithmetic::Literal(2)),
                Box::new(ast::Arithmetic::Literal(16))
            )
            .into_string(),
            format!("{} ** {}", 2, 16)
        );

        assert_eq!(
            ast::Arithmetic::PostIncr(String::from("1")).into_string(),
            format!("{}++", 1)
        );

        assert_eq!(
            ast::Arithmetic::PostDecr(String::from("1")).into_string(),
            format!("{}--", 1)
        );

        assert_eq!(
            ast::Arithmetic::PreIncr(String::from("1")).into_string(),
            format!("++{}", 1)
        );

        assert_eq!(
            ast::Arithmetic::PreDecr(String::from("1")).into_string(),
            format!("--{}", 1)
        );

        assert_eq!(
            ast::Arithmetic::<String>::UnaryPlus(Box::new(ast::Arithmetic::Var(String::from(
                "test"
            ))))
            .into_string(),
            format!("+({})", "test")
        );

        assert_eq!(
            ast::Arithmetic::<String>::UnaryMinus(Box::new(ast::Arithmetic::Var(String::from(
                "test"
            ))))
            .into_string(),
            format!("-({})", "test")
        );

        assert_eq!(
            ast::Arithmetic::<String>::LogicalNot(Box::new(ast::Arithmetic::Var(String::from(
                "test"
            ))))
            .into_string(),
            format!("!{}", "test")
        );

        assert_eq!(
            ast::Arithmetic::<String>::BitwiseNot(Box::new(ast::Arithmetic::Var(String::from(
                "test"
            ))))
            .into_string(),
            format!("~{}", "test")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Mult(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} * {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Div(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} / {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Modulo(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} % {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Add(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} + {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Sub(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} - {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::ShiftLeft(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} << {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::ShiftRight(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} >> {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Less(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} < {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Great(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} > {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::LessEq(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} <= {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::GreatEq(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} >= {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Eq(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} == {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::NotEq(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} != {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::BitwiseAnd(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} & {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::BitwiseXor(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} ^ {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::BitwiseOr(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} | {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::LogicalAnd(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} && {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::LogicalOr(
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} || {}", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Ternary(
                Box::new(ast::Arithmetic::Var(String::from("cond"))),
                Box::new(ast::Arithmetic::Var(String::from("lhs"))),
                Box::new(ast::Arithmetic::Var(String::from("rhs")))
            )
            .into_string(),
            format!("{} ? {} : {}", "cond", "lhs", "rhs")
        );

        assert_eq!(
            ast::Arithmetic::<String>::Assign(
                String::from("test"),
                Box::new(ast::Arithmetic::Literal(1))
            )
            .into_string(),
            format!("{} = {}", "test", 1)
        );

        assert_eq!(
            ast::Arithmetic::<String>::Sequence(vec![
                ast::Arithmetic::Literal(1),
                ast::Arithmetic::Literal(2),
                ast::Arithmetic::Literal(3)
            ])
            .into_string(),
            String::from("1, 2, 3")
        );
    }
}
