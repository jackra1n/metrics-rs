use crate::model::{ContribWeek, GhRepo, GhUser, LineWeek};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;

const GRAPHQL_URL: &str = "https://api.github.com/graphql";

/// Live-validated query: owner repos only, no forks, newest first, 100/page.
pub const PROFILE_QUERY: &str = r#"query($login:String!,$after:String){
  user(login:$login){
    login name bio avatarUrl createdAt
    followers{totalCount} following{totalCount}
    repositories(first:100,after:$after,ownerAffiliations:[OWNER],isFork:false,
                 orderBy:{field:UPDATED_AT,direction:DESC}){
      totalCount
      pageInfo{endCursor hasNextPage}
      nodes{ name
             stargazers{totalCount}
             forkCount
             issues(states:OPEN){totalCount}
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
    if let Some(errs) = v.get("errors").and_then(Value::as_array) {
        if let Some(first) = errs.first() {
            return Err(first["message"]
                .as_str()
                .unwrap_or("unknown graphql error")
                .to_string()
                .into());
        }
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoPayload {
    #[serde(default)]
    name: String,
    #[serde(default)]
    stargazers: TotalCount,
    #[serde(default)]
    fork_count: u64,
    #[serde(default)]
    issues: TotalCount,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserPayload {
    #[serde(default)]
    login: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    bio: Option<String>,
    #[serde(default)]
    avatar_url: String,
    #[serde(default)]
    created_at: String,
    #[serde(default)]
    followers: TotalCount,
    #[serde(default)]
    following: TotalCount,
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
            bio: payload.bio.clone(),
            avatar_url: payload.avatar_url.clone(),
            created_at: payload.created_at.clone(),
            followers: payload.followers.total_count,
            following: payload.following.total_count,
            repos_total: 0,
            repos: Vec::new(),
        });
        u.repos_total = payload.repositories.total_count;
        for n in payload.repositories.nodes {
            u.repos.push(GhRepo {
                langs: n.languages.edges.into_iter().map(|e| (e.node.name, e.size)).collect(),
                stars: n.stargazers.total_count,
                forks: n.fork_count,
                open_issues: n.issues.total_count,
                name: n.name,
            });
        }
        match (payload.repositories.page_info.has_next_page, payload.repositories.page_info.end_cursor) {
            (true, Some(cursor)) => after = Some(cursor),
            _ => break,
        }
    }
    Ok(user.ok_or_else(|| format!("user not found: {login}"))?)
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
                    w: w["w"].as_u64().unwrap_or(0),
                    a: w["a"].as_u64().unwrap_or(0),
                    d: w["d"].as_u64().unwrap_or(0),
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

const STATS_ATTEMPTS: usize = 12;

/// Per-repo contributors stats with 202 backoff (12 x 2s), aggregated to
/// weekly added/deleted for the profile user only. Unavailable stats warn.
pub fn collect_weeks(agent: &ureq::Agent, token: &str, user: &GhUser) -> Vec<LineWeek> {
    let mut agg: BTreeMap<String, (u64, u64)> = BTreeMap::new();
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
                    let date = iso_date(w.w);
                    let e = agg.entry(date).or_insert((0, 0));
                    e.0 += w.a;
                    e.1 += w.d;
                }
            }
            None => eprintln!("lines: skipping {}/{} (stats unavailable)", user.login, repo.name),
        }
    }
    agg.into_iter()
        .map(|(date, (added, deleted))| LineWeek { date, added, deleted })
        .collect()
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
