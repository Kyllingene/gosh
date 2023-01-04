use std::env;

mod git;
use git::Git;

pub struct Prompt {
    pub template: String,
    pub formatter: fn(String) -> String,
}

impl Prompt {
    pub fn new(template: String, formatter: fn(String) -> String) -> Self {
        Prompt {
            template,
            formatter,
        }
    }

    pub fn display(&self) -> String {
        (self.formatter)(self.template.clone())
    }

    pub fn basic_prompt(template: String) -> String {
        template
    }

    pub fn reactive_prompt(template: String) -> String {
        let mut out = template.replace("{pwd}", env::current_dir().unwrap().to_str().unwrap());

        out = out.replace(
            "{pwd-end}",
            env::current_dir()
                .unwrap()
                .iter()
                .last()
                .unwrap()
                .to_str()
                .unwrap(),
        );

        let branch = Git::branch();
        if branch.is_some() {
            out = out.replace("{branch}", &branch.unwrap());
            if Git::dirty().unwrap() {
                out = out.replace("{dirty}", "!");
            } else {
                out = out.replace("{dirty}", "");
            }
        }

        out
    }
}
