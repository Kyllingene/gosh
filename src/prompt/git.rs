use std::process::Command;

use regex::Regex;

pub struct Git {}

impl Git {
    pub fn branch() -> Option<String> {
        let result =
            String::from_utf8(Command::new("git").arg("status").output().unwrap().stdout).unwrap();

        if !result.starts_with("On branch ") {
            return None;
        }

        let re = Regex::new("On branch (.*)\n").unwrap();
        for branch in re.captures_iter(&result) {
            return Some(String::from(&branch[1]));
        }

        None
    }

    pub fn dirty() -> Option<bool> {
        let result = String::from_utf8(
            Command::new("git")
                .arg("diff")
                .arg("--cached")
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();

        if result.starts_with("warning: Not a git repository") {
            return None;
        } else if result.starts_with("diff") {
            return Some(true);
        } else {
            return Some(false);
        }
    }
}
