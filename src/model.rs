pub struct GhRepo {
    pub name: String,
    pub stars: u64,
    pub forks: u64,
    pub open_issues: u64,
    /// (language name, byte size)
    pub langs: Vec<(String, u64)>,
}

pub struct GhUser {
    pub login: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: String,
    pub created_at: String,
    pub followers: u64,
    pub following: u64,
    pub repos_total: u64,
    pub repos: Vec<GhRepo>,
}

/// added/deleted/commits per week from the contributors-stats REST endpoint
#[derive(Clone, serde::Deserialize)]
pub struct ContribWeek {
    pub w: u64,
    pub a: u64,
    pub d: u64,
}

#[derive(Debug)]
pub struct LineWeek {
    pub date: String,
    pub added: u64,
    pub deleted: u64,
}

pub struct LangStat {
    pub name: String,
    pub size: u64,
    pub pct: f64,
    pub color: &'static str,
}

pub struct Profile {
    pub user: GhUser,
    pub stars: u64,
    pub forks: u64,
    pub open_issues: u64,
    pub languages: Vec<LangStat>,
    pub weeks: Vec<LineWeek>,
}
