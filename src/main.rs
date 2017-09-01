extern crate rustyline;
extern crate combine;
use combine::combinator as cc;
use combine::Parser;
use std::iter::Iterator;
use std::process;
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
enum Command {
    Command(Vec<String>),
}
impl Command {
    fn execute(&self) -> Result<std::process::Child, String> {
        match *self {
            Command::Command(ref args) => {
                if let Some((cmd, args)) = args.split_first() {
                    process::Command::new(cmd)
                                     .args(args)
                                     .spawn()
                                     .map_err(|e| format!("{}", e))
                } else {
                    Err(format!("Invalid command {:?}", args))
                }
            }
        }
    }
}

fn parse_cmd(line: &str) -> Command {
    let any_nonquoated = || cc::satisfy(|c: char| !c.is_whitespace() && c != '\'');
    let unquoated_arg     = || cc::many1::<String, _>(any_nonquoated());
    let quoated_arg       = || cc::between(combine::char::char('\''), combine::char::char('\''), cc::many::<String, _>(cc::satisfy(|c: char| c != '\'')));
    let arg               = || combine::try(unquoated_arg()).or(quoated_arg()).skip(combine::char::spaces());
    let cmd               = || cc::many1::<Vec<_>, _>(arg()).map(|args| Command::Command(args));
    (cmd(), cc::eof()).parse(combine::State::new(line)).map(|((x, _), _)| x).unwrap()
}

fn main() {
    let cmds = CmdReader::new("> ");
    for line in cmds {
        let command = parse_cmd(&line);
        command.execute().unwrap().wait().unwrap();
    }
}
