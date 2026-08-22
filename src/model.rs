pub struct GhRepo {
    pub name: String,
    pub stars: u64,
    pub forks: u64,
    pub watchers: u64,
    pub releases: u64,
    /// repo size in KB as reported by GitHub (`diskUsage`)
    pub disk_usage_kb: u64,
    /// SPDX id, e.g. "MIT"; None when unlicensed
    pub license: Option<String>,
    /// (language name, byte size)
    pub langs: Vec<(String, u64)>,
}

pub struct GhUser {
    pub login: String,
    pub name: Option<String>,
    /// ISO 8601 profile creation timestamp
    pub created_at: String,
    pub avatar_url: String,
    /// sponsor count (sponsors of this user)
    pub sponsors: u64,
    pub repos_total: u64,
    pub repos: Vec<GhRepo>,
}

/// added/deleted/commit counts per week from contributors-stats
#[derive(Clone, serde::Deserialize)]
pub struct ContribWeek {
    pub a: u64,
    pub d: u64,
    pub c: u64,
}

#[derive(Debug, Default)]
pub struct LineTotals {
    pub added: u64,
    pub deleted: u64,
    /// commits authored by the user across analyzed repos
    pub commits: u64,
}

/// Last-14-day commit activity for the mini contribution graph.
#[derive(Default)]
pub struct Activity {
    /// (YYYY-MM-DD, commit count) for the last 14 days, oldest first
    pub days: Vec<(String, u64)>,
    /// distinct repos with commit contributions in the same window
    pub repos: u64,
}

pub struct LangStat {
    pub name: String,
    pub pct: f64,
    pub color: &'static str,
}

pub struct Profile {
    pub user: GhUser,
    pub stars: u64,
    pub forks: u64,
    pub watchers: u64,
    pub releases: u64,
    /// summed repo sizes in KB
    pub storage_kb: u64,
    /// most common license across repos by count
    pub preferred_license: Option<String>,
    pub languages: Vec<LangStat>,
    pub activity: Activity,
    pub lines: LineTotals,
}
