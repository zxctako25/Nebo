use std::collections::HashSet;

pub(super) struct AsnBlocker {
    blocked: HashSet<u32>,
}

impl AsnBlocker {
    pub fn new(blocked: HashSet<u32>) -> Self {
        Self { blocked }
    }
    pub fn len(&self) -> usize {
        self.blocked.len()
    }
    pub fn blocks(&self, asn: u32) -> bool {
        self.blocked.contains(&asn)
    }
}
