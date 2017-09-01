extern crate rustyline;

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


fn main() {
    let cmds = CmdReader::new("> ");
    for line in cmds {
        println!("{}", line)
    }
}
