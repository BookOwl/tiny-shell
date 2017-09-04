extern crate rustyline;
extern crate combine;
use combine::combinator as cc;
use combine::char::spaces;
use combine::Parser;
use std::iter::Iterator;
use std::process;
use std::process::Stdio;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
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
struct Command {
    args: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Pipeline {
    pipeline: Vec<Command>
}

impl Command {
    fn execute(&self, stdin: Stdio, stdout: Stdio) -> Result<std::process::Child, String> {
        if let Some((cmd, args)) = self.args.split_first() {
            Ok(process::Command::new(cmd)
                                .args(args)
                                .stdin(stdin)
                                .stdout(stdout)
                                .spawn()
                                .map_err(|e| format!("{}", e))?)
        } else {
            Err(format!("Invalid command {:?}", self.args))
        }
    }
}

impl Pipeline {
    fn execute(&self, stdin: Stdio, stdout: Stdio) -> Result<Vec<std::process::Child>, String> {
        let mut children = Vec::with_capacity(self.pipeline.len());
        let mut stdin = stdin;
        let (last, pipeline) = self.pipeline.split_last().unwrap();
        for c in pipeline {
            let child = c.execute(stdin, Stdio::piped())?;
            let handle = child.stdout.as_ref().ok_or_else(|| format!("Could not get child stdout"))?.as_raw_fd();
            stdin = unsafe { Stdio::from_raw_fd(handle) };
            children.push(child);
        }
        children.push(last.execute(stdin, stdout)?);
        Ok(children)
    }
}

fn parse_cmd(line: &str) -> Pipeline {
    let any_nonquoated = || cc::satisfy(|c: char| !c.is_whitespace() && c != '\'' && c != '|');
    let unquoated_arg  = || cc::many1::<String, _>(any_nonquoated());
    let quoated_arg    = || cc::between(combine::char::char('\''), combine::char::char('\''), cc::many::<String, _>(cc::satisfy(|c: char| c != '\'')));
    let arg            = || combine::try(unquoated_arg()).or(quoated_arg()).skip(spaces());
    let cmd            = || cc::many1::<Vec<_>, _>(arg()).map(|args| Command{args});
    let pipe           = || (spaces(), combine::char::char('|'), spaces());
    let pipeline       = || combine::sep_by1(cmd(), pipe()).map(|pipeline| Pipeline{pipeline});
    let cmdline        = || pipeline().skip(cc::eof());
    cmdline().parse(combine::State::new(line)).unwrap().0
}

fn main() {
    let cmds = CmdReader::new("> ");
    for line in cmds {
        let command = parse_cmd(&line);
        let stdin  = Stdio::inherit();
        let stdout = Stdio::inherit();
        for mut p in command.execute(stdin, stdout).unwrap() {
            p.wait().unwrap();
        }
    }
}
