use super::ip_list::IpBlacklist;
use std::net::IpAddr;

pub(super) struct IpBlocker {
    list: IpBlacklist,
}

impl IpBlocker {
    pub fn new(list: IpBlacklist) -> Self {
        Self { list }
    }
    pub fn len(&self) -> usize {
        self.list.len()
    }
    pub fn is_blocked(&self, ip: IpAddr) -> bool {
        self.list.contains(&ip)
    }
}
