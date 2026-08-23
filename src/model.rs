use std::collections::BTreeMap;

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
    pub repos_contributed: u64,
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

#[derive(Clone, Debug, Default)]
pub struct ContribRepo {
    pub name: String,
    pub name_with_owner: String,
    pub commits: u64,
}

/// Last-14-day commit activity for the mini contribution graph.
#[derive(Default)]
pub struct Activity {
    /// (YYYY-MM-DD, commit count) for the last 14 days, oldest first
    pub days: Vec<(String, u64)>,
    /// distinct repos with commit contributions in the same window
    pub repos: u64,
    /// repositories contributed to in the same window with commit counts
    pub contrib_repos: Vec<ContribRepo>,
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
    /// populated only in --indepth mode: authored-commit language stats
    pub indepth: Option<IndepthStats>,
}

/// Aggregated in-depth analysis of the user's authored commits, derived from
/// bare git clones and `git log --numstat` rather than repo byte totals.
#[derive(Default)]
pub struct IndepthStats {
    /// language name -> authored lines added (programming/markup only)
    pub lines_by_lang: BTreeMap<String, u64>,
    /// language name -> repo full names (owner/repo) contributing those lines
    pub repos_by_lang: BTreeMap<String, Vec<String>>,
    /// total authored lines added across all analyzed commits
    pub added: u64,
    /// total authored lines deleted across all analyzed commits
    pub deleted: u64,
    /// total edited files encountered across all authored commits
    pub files: u64,
    /// authored commits found across all analyzed repos
    pub commits: u64,
    pub repos_total: usize,
    /// repositories that cloned and parsed successfully
    pub repos_analyzed: usize,
    /// languages excluded from the analysis (--ignore-languages)
    pub ignored: Vec<String>,
}
