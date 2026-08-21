use crate::model::{ContribWeek, GhRepo, GhUser, LineWeek};
use std::collections::BTreeMap;

pub struct Args {
    pub username: String,
    pub output: String,
    pub token: String,
}

pub fn agent() -> ureq::Agent {
    unimplemented!()
}

pub fn fetch_profile(
    _agent: &ureq::Agent,
    _token: &str,
    _login: &str,
) -> Result<GhUser, Box<dyn std::error::Error>> {
    unimplemented!()
}

pub fn contributors_stats(
    _agent: &ureq::Agent,
    _token: &str,
    _owner: &str,
    _repo: &str,
) -> Result<Option<Vec<ContribWeek>>, Box<dyn std::error::Error>> {
    unimplemented!()
}

pub fn iso_date(_unix_secs: u64) -> String {
    unimplemented!()
}

pub fn collect_weeks(
    _agent: &ureq::Agent,
    _token: &str,
    _user: &GhUser,
) -> Vec<LineWeek> {
    let _ = BTreeMap::<String, (u64, u64)>::new();
    Vec::new()
}
