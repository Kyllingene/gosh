use std::fmt::Display;
use std::io::{stdin, stdout, Write};
use std::process::{Child, Command, Stdio};

use std::env;

use std::path::Path;

use liner::Context;

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

// struct History {
//     lines: Vec<String>,
//     index: usize,
// }

// impl History {
//     pub fn new() -> Self {
//         History {
//             lines: Vec::new(),
//             index: 0,
//         }
//     }

//     pub fn add(&mut self, line: String) {
//         self.lines.insert(self.lines.len(), line);
//     }

//     pub fn get(&self) -> String {
//         if self.index == self.lines.len() {
//             String::new()
//         } else {
//             self.lines[self.index].clone()
//         }
//     }

//     pub fn up(&mut self) {
//         if self.index > 0 {
//             self.index -= 1;
//         }
//     }

//     pub fn down(&mut self) {
//         if self.index < self.lines.len() {
//             self.index += 1;
//         }
//     }
// }

fn error(e: &dyn Display) {
    cod::color_fg(1);
    print!("error: ");
    cod::decolor();
    println!("{}", e);
}

fn main() {
    let mut aliases = Aliases::new();
    let mut con = Context::new();

    loop {
        // print!(" ➜ ");
        // stdout().flush().unwrap();

        // let mut input = String::new();
        // stdin().read_line(&mut input).unwrap();

        let input = con.read_line(" ➜ ", &mut |_| {}).unwrap();

        if input.is_empty() {
            continue
        }

        con.history.push(input.clone().into()).unwrap();

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
                return;
            }

            match command.unwrap() {
                "cd" => {
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        error(&e);
                    }
                }

                "exit" => return,

                "alias" => {
                    let alias = match args.next() {
                        Some(alias) => alias,
                        None => {
                            error(&"missing alias");
                            return;
                        }
                    };

                    if args.clone().peekable().peek().is_none() {
                        error(&"missing target");
                        return;
                    }

                    let target: String = args.collect::<Vec<_>>().join(" ");

                    aliases.set(String::from(alias), target);
                }

                "aliases" => {
                    for (k, v) in aliases.pairs() {
                        println!("{}: {}", k, v);
                    }
                }

                mut command => {
                    let cmd = aliases
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
                            error(&e);
                        }
                    }
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            final_command.wait().unwrap();
        }
    }
}
