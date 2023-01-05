# gosh

This is a learning project. I started with [this tutorial](https://www.joshmcguigan.com/blog/build-your-own-shell-rust/), then I moved on to add more features until I was happy with the end product. It's certainly not a full shell, though I may continue to add to it; in any case, I wouldn't recommend this as a daily driver.

## features

- basic builtins - `cd`, `exit`, `echo`, `exec`
- aliases - `alias <alias> <command [args...]>`
- a pretty decent input (provided by [this wonderful crate](https://crates.io/crates/liner))
    - history, with a histfile
    - vi- and emacs-like modes (`set-mode <vi | emacs>`)
    - customizable prompt (`set-prompt <prompt>`)
- a `.goshrc`
- scripts (`gosh <script>`, shebangs)
- basic substitution
    - `~` for home

### todo

- finish reorganizing
- clean up the aliases
- add more customization options
    - more prompt replacements
- wildcard globbing
- `;` / `&&`
- background processes

### license + contributing

This project is under the MIT license (see LICENSE.txt), as are all dependencies. Any and all contributions are welcome, though I request that you do two things first: make sure it compiles and runs, and PLEASE run `cargo fmt`.

### dependencies

- [cod](https://crates.io/crates/cod) - color for warning/error messages
- [dirs](https://crates.io/crates/dirs) - finding the home directory
- [liner](https://crates.io/crates/liner) - input handling + history
- [regex](https://crates.io/crates/regex) - getting git status for the prompt
