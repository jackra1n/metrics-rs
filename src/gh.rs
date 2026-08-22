use crate::model::{ContribWeek, GhRepo, GhUser, LineTotals};
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Duration;

const GRAPHQL_URL: &str = "https://api.github.com/graphql";

/// Live-validated query: owner repos only, no forks, newest first, 100/page.
pub const PROFILE_QUERY: &str = r#"query($login:String!,$after:String){
  user(login:$login){
    login name avatarUrl createdAt
    followers{totalCount} sponsors{totalCount}
    repositories(first:100,after:$after,ownerAffiliations:[OWNER],isFork:false,
                 orderBy:{field:UPDATED_AT,direction:DESC}){
      totalCount
      pageInfo{endCursor hasNextPage}
      nodes{ name
             stargazers{totalCount}
             watchers{totalCount}
             forkCount
             releases{totalCount}
             diskUsage
             languages(first:8){edges{size node{name}}} }
    }
  }
}"#;

/// Shared HTTP agent: raw statuses visible (we must see 202/404), 30s cap.
pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .http_status_as_error(false)
        .user_agent("metrics-rs")
        .build()
        .into()
}

/// POST one GraphQL document. Non-200 => parsed `message` as error.
/// 200 with non-empty `errors` => `errors[0].message` as error.
pub fn graphql(
    agent: &ureq::Agent,
    token: &str,
    query: &str,
    vars: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut res = agent
        .post(GRAPHQL_URL)
        .header("Authorization", &format!("Bearer {token}"))
        .send_json(json!({ "query": query, "variables": vars }))?;
    let status = res.status().as_u16();
    let body = res.body_mut().read_to_string()?;
    if status != 200 {
        let msg = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|v| v["message"].as_str().map(str::to_string))
            .unwrap_or_else(|| format!("HTTP {status}: {body}"));
        return Err(msg.into());
    }
    let v: Value = serde_json::from_str(&body)?;
    if let Some(errs) = v.get("errors").and_then(Value::as_array)
        && let Some(first) = errs.first()
    {
        return Err(first["message"]
            .as_str()
            .unwrap_or("unknown graphql error")
            .to_string()
            .into());
    }
    Ok(v)
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TotalCount {
    #[serde(default)]
    total_count: u64,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    #[serde(default)]
    end_cursor: Option<String>,
    #[serde(default)]
    has_next_page: bool,
}

#[derive(Deserialize)]
struct LangName {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct LangEdge {
    #[serde(default)]
    size: u64,
    node: LangName,
}

#[derive(Deserialize, Default)]
struct LangConn {
    #[serde(default)]
    edges: Vec<LangEdge>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RepoPayload {
    #[serde(default)]
    name: String,
    #[serde(default)]
    stargazers: TotalCount,
    #[serde(default)]
    watchers: TotalCount,
    #[serde(default)]
    fork_count: u64,
    #[serde(default)]
    releases: TotalCount,
    #[serde(default)]
    disk_usage: u64,
    #[serde(default)]
    languages: LangConn,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RepoConn {
    #[serde(default)]
    total_count: u64,
    #[serde(default)]
    page_info: PageInfo,
    #[serde(default)]
    nodes: Vec<RepoPayload>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct UserPayload {
    #[serde(default)]
    login: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    avatar_url: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    sponsors: TotalCount,
    #[serde(default)]
    repositories: RepoConn,
}

/// Paginated profile + owned-repo fetch; hard cap 10 pages (1000 repos).
pub fn fetch_profile(
    agent: &ureq::Agent,
    token: &str,
    login: &str,
) -> Result<GhUser, Box<dyn std::error::Error>> {
    let mut user: Option<GhUser> = None;
    let mut after: Option<String> = None;
    for _page in 0..10usize {
        let v = graphql(
            agent,
            token,
            PROFILE_QUERY,
            json!({ "login": login, "after": after }),
        )?;
        let user_val = &v["data"]["user"];
        if user_val.is_null() {
            return Err(format!("user not found: {login}").into());
        }
        let payload: UserPayload = serde_json::from_value(user_val.clone())?;
        let u = user.get_or_insert_with(|| GhUser {
            login: payload.login.clone(),
            name: payload.name.clone(),
            avatar_url: payload.avatar_url.clone(),
            created_at: payload.created_at.clone(),
            sponsors: payload.sponsors.total_count,
            repos_total: 0,
            repos: Vec::new(),
        });
        u.repos_total = payload.repositories.total_count;
        for n in payload.repositories.nodes {
            u.repos.push(GhRepo {
                langs: n
                    .languages
                    .edges
                    .into_iter()
                    .map(|e| (e.node.name, e.size))
                    .collect(),
                stars: n.stargazers.total_count,
                watchers: n.watchers.total_count,
                forks: n.fork_count,
                releases: n.releases.total_count,
                disk_usage_kb: n.disk_usage,
                license: None, // filled per-repo from REST (proper SPDX case)
                name: n.name,
            });
        }
        match (
            payload.repositories.page_info.has_next_page,
            payload.repositories.page_info.end_cursor,
        ) {
            (true, Some(cursor)) => after = Some(cursor),
            _ => break,
        }
    }
    let mut user = user.ok_or_else(|| format!("user not found: {login}"))?;
    // REST license metadata: proper SPDX casing ("AGPL-3.0"); serial calls,
    // rate-limit friendly; failures leave license None
    for repo in &mut user.repos {
        let url = format!("https://api.github.com/repos/{}/{}", user.login, repo.name);
        if let Ok(mut res) = agent
            .get(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .call()
            && res.status().as_u16() == 200
            && let Ok(v) = res.body_mut().read_json::<Value>()
        {
            repo.license = v["license"]["spdx_id"]
                .as_str()
                .filter(|s| !s.is_empty() && *s != "NOASSERTION" && *s != "OTHER")
                .map(str::to_string);
        }
    }
    Ok(user)
}

/// GET /repos/{owner}/{repo}/stats/contributors.
/// 202 => GitHub still computing (caller retries); 200 array => Some;
/// anything else (404 stats-disabled, 403) => None: skip, not fatal.
/// Weeks are pre-filtered to `login` (case-insensitive) at the source.
pub fn contributors_stats(
    agent: &ureq::Agent,
    token: &str,
    owner: &str,
    repo: &str,
    login: &str,
) -> Result<Option<Vec<ContribWeek>>, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/stats/contributors");
    let mut res = agent
        .get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .call()?;
    let status = res.status().as_u16();
    if status != 200 {
        // 202 = still computing; 404/403 = stats unavailable.
        return Ok(None);
    }
    let entries: Vec<serde_json::Value> = res.body_mut().read_json()?;
    let mut out = Vec::new();
    for entry in entries {
        let author = entry["author"]["login"].as_str().unwrap_or_default();
        if !author.eq_ignore_ascii_case(login) {
            continue;
        }
        if let Some(weeks) = entry["weeks"].as_array() {
            for w in weeks {
                out.push(ContribWeek {
                    a: w["a"].as_u64().unwrap_or(0),
                    d: w["d"].as_u64().unwrap_or(0),
                    c: w["c"].as_u64().unwrap_or(0),
                });
            }
        }
    }
    Ok(Some(out))
}

/// UTC calendar date (YYYY-MM-DD) from unix seconds; pure civil-from-days math.
pub fn iso_date(unix_secs: u64) -> String {
    let days = (unix_secs / 86_400) as i64;
    // Howard Hinnant's civil_from_days; 1970-01-01 = day 0.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// Last-14-day commit contributions for the mini contribution graph:
/// per-day counts plus distinct repos committed to in the window.
pub fn fetch_activity(
    agent: &ureq::Agent,
    token: &str,
    login: &str,
    now_secs: u64,
) -> Result<crate::model::Activity, Box<dyn std::error::Error>> {
    const ACTIVITY_QUERY: &str = r#"query($login:String!,$from:DateTime!,$to:DateTime!){
user(login:$login){
contributionsCollection(from:$from,to:$to){
contributionCalendar{weeks{contributionDays{date contributionCount}}}
commitContributionsByRepository{repository{name}}
}
}
}"#;
    // window: start of the day 13 days ago .. now (14 days inclusive)
    let from = format!("{}T00:00:00Z", iso_date(now_secs.saturating_sub(13 * 86_400)));
    let to = format!("{}T23:59:59Z", iso_date(now_secs));
    let v = graphql(
        agent,
        token,
        ACTIVITY_QUERY,
        json!({ "login": login, "from": from, "to": to }),
    )?;
    let cc = &v["data"]["user"]["contributionsCollection"];
    if cc.is_null() {
        return Err(format!("activity not found: {login}").into());
    }
    let mut by_date = std::collections::BTreeMap::<String, u64>::new();
    if let Some(weeks) = cc["contributionCalendar"]["weeks"].as_array() {
        for w in weeks {
            if let Some(days) = w["contributionDays"].as_array() {
                for d in days {
                    if let (Some(date), Some(n)) =
                        (d["date"].as_str(), d["contributionCount"].as_u64())
                    {
                        *by_date.entry(date.to_string()).or_insert(0) += n;
                    }
                }
            }
        }
    }
    let days = (0..14usize)
        .rev()
        .map(|i| {
            let date = iso_date(now_secs.saturating_sub(i as u64 * 86_400));
            let count = by_date.get(&date).copied().unwrap_or(0);
            (date, count)
        })
        .collect();
    Ok(crate::model::Activity {
        days,
        repos: cc["commitContributionsByRepository"]
            .as_array()
            .map(|a| a.len() as u64)
            .unwrap_or(0),
    })
}

const STATS_ATTEMPTS: usize = 12;

/// Per-repo contributors stats with 202 backoff (12 x 2s), summed to total
/// added/deleted lines for the profile user only. Unavailable stats warn.
pub fn collect_lines(agent: &ureq::Agent, token: &str, user: &GhUser) -> LineTotals {
    let mut totals = LineTotals::default();
    for repo in &user.repos {
        let mut got: Option<Vec<ContribWeek>> = None;
        for _attempt in 0..STATS_ATTEMPTS {
            match contributors_stats(agent, token, &user.login, &repo.name, &user.login) {
                Ok(Some(weeks)) => {
                    got = Some(weeks);
                    break;
                }
                Ok(None) => std::thread::sleep(Duration::from_secs(2)),
                Err(_) => break,
            }
        }
        match got {
            Some(weeks) => {
                for w in weeks {
                    totals.added += w.a;
                    totals.deleted += w.d;
                    totals.commits += w.c;
                }
            }
            None => eprintln!(
                "lines: skipping {}/{} (stats unavailable)",
                user.login, repo.name
            ),
        }
    }
    totals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_date_known_vector() {
        assert_eq!(iso_date(86_400), "1970-01-02");
        assert_eq!(iso_date(1_759_622_400), "2025-10-05");
        assert_eq!(iso_date(0), "1970-01-01");
        assert_eq!(iso_date(1_759_622_400 + 86_400 * 365), "2026-10-05");
    }
}
