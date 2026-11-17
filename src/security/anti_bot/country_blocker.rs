use std::collections::HashSet;

pub(super) struct CountryBlocker {
    blocked: HashSet<String>,
}

impl CountryBlocker {
    pub fn new(blocked: HashSet<String>) -> Self {
        Self { blocked }
    }
    pub fn len(&self) -> usize {
        self.blocked.len()
    }
    pub fn blocks(&self, country_code: &str) -> bool {
        self.blocked.contains(country_code)
    }
}
