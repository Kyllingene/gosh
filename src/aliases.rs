// TODO: figure out how to do this with a HashMap without the borrowing issues
#[derive(Debug, Clone)]
pub struct Aliases {
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
