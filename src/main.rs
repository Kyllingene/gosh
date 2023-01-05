use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{stderr, Read, Write};
use std::path::{self, Path};
use std::process::{exit, Child, Command, Stdio};

use dirs::home_dir;
use liner::{Context, KeyBindings};

mod prompt;
use prompt::Prompt;

mod aliases;
use aliases::Aliases;

trait SplitWithQuotes {
    type Output;

    fn split_whitespace_quotes(&self) -> Vec<Self::Output>;
}

impl SplitWithQuotes for &str {
    type Output = String;

    fn split_whitespace_quotes(&self) -> Vec<Self::Output> {
        let mut split = Vec::new();
        let mut current = String::new();

        let mut quoted = false;

        for ch in self.chars() {
            match ch {
                '"' => {
                    if !current.is_empty() {
                        split.push(current.clone());
                    }
                    current.clear();
                    quoted = !quoted;
                }
                ' ' => {
                    if quoted {
                        current.push(' ');
                    } else {
                        if !current.is_empty() {
                            split.push(current.clone());
                        }
                        current.clear();
                    }
                }
                ch => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            split.push(current.clone());
        }

        split
    }
}

struct Shell {
    aliases: Aliases,
    context: Context,
    home: String,
    prompt: Prompt,
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

        let prompt = Prompt::new(String::from(" ➜ "), Prompt::basic_prompt);

        Shell {
            aliases: Aliases::new(),
            context,
            home,
            prompt,
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
            let input = match self.context.read_line(self.prompt.display(), &mut |_| {}) {
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

    pub fn eval(&mut self, lines: String) {
        for line in lines.split_terminator("\n") {
            self.line(String::from(line));
        }
    }

    fn line(&mut self, line: String) {
        if line.starts_with("#") {
            return;
        }

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
            let parts_vec = command.trim().split_whitespace_quotes();
            let mut parts = parts_vec.iter();

            let command = parts.next();
            let mut args = parts;

            if command.is_none() {
                return;
            }

            match command.unwrap().as_str() {
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

                "exec" => {
                    // TODO: is there a better way?
                    let command = args.map(|s| s.clone()).collect::<Vec<String>>().join(" ");
                    self.line(command);
                    self.exit();
                }

                "set-mode" => {
                    let mode = match args.next() {
                        Some(mode) => mode,
                        None => {
                            Shell::error(&"missing mode");
                            break;
                        }
                    };

                    match mode.as_str() {
                        "vi" => {
                            self.context.key_bindings = KeyBindings::Vi;
                        }
                        "emacs" => {
                            self.context.key_bindings = KeyBindings::Emacs;
                        }
                        _ => {
                            Shell::error(&"invalid mode");
                        }
                    }
                }

                "set-prompt" => {
                    let template = args.map(|s| s.clone()).collect::<Vec<String>>().join(" ");

                    self.prompt.template = template;
                }

                "set-prompt-mode" => {
                    let mode = match args.next() {
                        Some(mode) => mode,
                        None => {
                            Shell::error(&"missing mode");
                            break;
                        }
                    };

                    match mode.as_str() {
                        "basic" => {
                            self.prompt.formatter = Prompt::basic_prompt;
                        }
                        "reactive" => {
                            self.prompt.formatter = Prompt::reactive_prompt;
                        }
                        _ => {
                            Shell::error(&"invalid mode");
                        }
                    }
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

                    let target: String = args.map(|s| s.clone()).collect::<Vec<_>>().join(" ");

                    self.aliases.set(String::from(alias), target);
                }

                "aliases" => {
                    for (k, v) in self.aliases.pairs() {
                        println!("{}: {}", k, v);
                    }
                }

                "echo" => {
                    println!(
                        "{}",
                        args.map(|s| s.clone()).collect::<Vec<String>>().join(" ")
                    );
                }

                mut command => {
                    let cmd = self
                        .aliases
                        .get(String::from(command))
                        .unwrap_or(String::from(command));

                    let mut new_args: Vec<String>;
                    if cmd.as_str().contains(' ') {
                        let mut alias_args = cmd.as_str().split_whitespace();
                        command = alias_args.next().unwrap();

                        new_args = args.map(|s| s.clone()).collect();
                        for arg in alias_args {
                            new_args.insert(0, String::from(arg));
                        }

                        args = new_args.iter();
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
        let mut stderr = stderr();

        cod::color_fg(1);
        write!(stderr, "error: ").unwrap_or_else(|_| {
            print!("error: ");
        });
        cod::decolor();
        writeln!(stderr, "{}", e).unwrap_or_else(|_| {
            println!("{}", e);
        });
    }

    fn warn(w: &dyn Display) {
        cod::color_fg(2);
        print!("warn: ");
        cod::decolor();
        println!("{}", w);
    }
}

fn main() {
    let mut shell = Shell::new(String::from("/home/kyllingene"));

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        if &args[1] == "--help" || &args[1] == "-h" {
            println!("usage: {} [--help | -h] [script]", args[0]);
            exit(0);
        } else {
            let mut file = match File::open(&args[1]) {
                Ok(file) => file,

                Err(e) => {
                    Shell::error(&e);
                    exit(e.raw_os_error().unwrap_or(1));
                }
            };

            let mut data = String::new();
            if let Err(e) = file.read_to_string(&mut data) {
                Shell::error(&e);
                exit(e.raw_os_error().unwrap_or(1));
            }

            shell.eval(data);
        }
    } else {
        shell.main();
    }
}
