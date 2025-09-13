pub struct LastHash {
    hash: String,
}

impl LastHash {
    pub fn new() -> LastHash {
        LastHash {
            hash: "RGVmYXVsdEhhc2g".to_string(),
        }
    }

    pub fn set(&mut self, hash: String) {
        self.hash = hash;
    }

    pub fn get(&self) -> String {
        self.hash.clone()
    }
}
