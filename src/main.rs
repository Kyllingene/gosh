use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::process::{exit, Child, Command, Stdio};

use std::{env, path};

use std::path::Path;

use liner::Context;

use dirs::home_dir;

struct Shell {
    aliases: Aliases,
    context: Context,
    home: String,
    prompt: String,
}

impl Shell {
    pub fn new(home: String) -> Self {
        let mut context = Context::new();
        context
            .history
            .set_file_name(Some(home.clone() + "/.goshist"));
        context.history.set_max_size(1000);
        context.history.load_history().unwrap_or_else(|_| {
            Shell::warn(&"failed to load history");
        });

        Shell {
            aliases: Aliases::new(),
            context,
            home,
            prompt: String::from(" ➜ "),
        }
    }

    pub fn main(&mut self) {
        let goshrc = self.home.clone() + "/.goshrc";

        if path::Path::new(&goshrc).exists() {
            let file = File::open(goshrc);

            if let Ok(mut file) = file {
                let mut rc = String::new();
                if let Ok(_) = file.read_to_string(&mut rc) {
                    self.eval(rc);
                }
            } else {
                Shell::warn(&"couldn't open ~/.goshrc");
            }
        } else {
            Shell::warn(&"couldn't find ~/.goshrc");
        }

        loop {
            let input = match self.context.read_line(self.prompt.clone(), &mut |_| {}) {
                Ok(input) => input,
                Err(_) => break,
            };

            if input.is_empty() {
                continue;
            }

            self.line(input);
        }

        self.context.history.commit_history();
    }

    fn exit(&mut self) {
        self.context.history.commit_history();
        exit(0);
    }

    fn eval(&mut self, lines: String) {
        for line in lines.split_terminator("\n") {
            self.line(String::from(line));
        }
    }

    fn line(&mut self, line: String) {
        let home = home_dir()
            .unwrap_or("./".into())
            .to_str()
            .unwrap()
            .chars()
            .collect();
        let input = Shell::substitute(&line, home);

        self.context.history.push(input.clone().into()).unwrap();

        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            let mut parts = command
                .trim()
                .split_whitespace()
                .collect::<Vec<_>>()
                .into_iter();
            let command = parts.next();
            let mut args = parts;

            if command.is_none() {
                self.exit();
            }

            match command.unwrap() {
                "cd" => {
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        Shell::error(&e);
                    }
                }

                "exit" => {
                    self.exit();
                }

                "alias" => {
                    let alias = match args.next() {
                        Some(alias) => alias,
                        None => {
                            Shell::error(&"missing alias");
                            break;
                        }
                    };

                    if args.clone().peekable().peek().is_none() {
                        Shell::error(&"missing alias target");
                        break;
                    }

                    let target: String = args.collect::<Vec<_>>().join(" ");

                    self.aliases.set(String::from(alias), target);
                }

                "aliases" => {
                    for (k, v) in self.aliases.pairs() {
                        println!("{}: {}", k, v);
                    }
                }

                mut command => {
                    let cmd = self
                        .aliases
                        .get(String::from(command))
                        .unwrap_or(String::from(command));

                    if cmd.as_str().contains(' ') {
                        let mut alias_args = cmd.as_str().split_whitespace();
                        command = alias_args.next().unwrap();

                        let mut new_args: Vec<&str> = args.collect();
                        for arg in alias_args {
                            new_args.insert(0, arg);
                        }

                        args = new_args.into_iter().into();
                    } else {
                        command = &cmd;
                    }

                    let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                        Stdio::from(output.stdout.unwrap())
                    });

                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => {
                            previous_command = Some(output);
                        }
                        Err(e) => {
                            previous_command = None;
                            Shell::error(&e);
                        }
                    }
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            final_command.wait().unwrap();
        }
    }

    fn substitute(input: &String, mut home: Vec<char>) -> String {
        let chars = input.chars().collect::<Vec<char>>();
        let mut new: Vec<char> = Vec::new();

        let mut escaped = false;

        for ch in chars {
            if ch == '\\' && !escaped {
                escaped = true;
            } else if ch == '~' && !escaped {
                new.append(&mut home);
            } else {
                escaped = false;
                new.insert(new.len(), ch);
            }
        }

        new.iter().collect()
    }

    fn error(e: &dyn Display) {
        cod::color_fg(1);
        print!("error: ");
        cod::decolor();
        println!("{}", e);
    }

    fn warn(w: &dyn Display) {
        cod::color_fg(2);
        print!("warn: ");
        cod::decolor();
        println!("{}", w);
    }
}

// TODO: figure out how to do this with a HashMap without the borrowing issues
#[derive(Debug, Clone)]
struct Aliases {
    keys: Vec<String>,
    vals: Vec<String>,
}

impl Aliases {
    pub fn new() -> Self {
        Aliases {
            keys: Vec::new(),
            vals: Vec::new(),
        }
    }

    pub fn set(&mut self, key: String, val: String) {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                self.vals[i] = val;
                return;
            }
        }

        self.keys.insert(0, key);
        self.vals.insert(0, val);
    }

    pub fn get(&self, key: String) -> Option<String> {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                return Some(self.vals[i].clone());
            }
        }

        None
    }

    pub fn pairs(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for i in 0..self.keys.len() {
            out.insert(out.len(), (self.keys[i].clone(), self.vals[i].clone()));
        }

        out
    }
}

fn main() {
    let mut shell = Shell::new(String::from("/home/kyllingene"));

    shell.main();
}
