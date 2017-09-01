extern crate rustyline;
extern crate combine;
use combine::combinator as cc;
use combine::Parser;

use std::iter::Iterator;
use rustyline::completion::FilenameCompleter;

/// A CmdReader allows iterating over standard input
/// without having to manually manage the rustyline::Editor's history.
struct CmdReader {
    rl: rustyline::Editor<FilenameCompleter>,
    prompt: &'static str,
}
impl CmdReader {
    /// Creates a new CmdReader that uses the supplied prompt.
    fn new(prompt: &'static str) -> CmdReader {
        let mut rl = rustyline::Editor::new();
        rl.set_completer(Some(FilenameCompleter::new()));
        CmdReader {
            rl,
            prompt,
        }
    }
}
impl Iterator for CmdReader {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        match self.rl.readline(self.prompt) {
            Ok(line) => {
                self.rl.add_history_entry(&line);
                Some(line)
            },
            Err(_) => None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Ast {
    Cmd(Vec<String>),
}

fn parse_cmd(line: &str) -> Ast {
    let any_nonwhitespace = || cc::satisfy(|c: char| !c.is_whitespace());
    let arg = || cc::many1::<String, _>(any_nonwhitespace()).skip(combine::char::spaces());
    let cmd = || cc::many1::<Vec<_>, _>(arg()).map(|args| Ast::Cmd(args));
    (cmd(), cc::eof()).parse(combine::State::new(line)).map(|((x, _), _)| x).unwrap()
}

fn main() {
    let cmds = CmdReader::new("> ");
    for line in cmds {
        let ast = parse_cmd(&line);
        println!("{:?}", ast)
    }
}
