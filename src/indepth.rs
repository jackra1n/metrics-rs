use crate::langs::classify_path;
use crate::model::IndepthStats;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

const CONCURRENCY: usize = 8;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// One user's login, optional display name, and GitHub noreply email.
pub fn author_identities(login: &str, display_name: Option<&str>) -> Vec<String> {
    let mut identities = vec![login.to_string()];
    if let Some(name) = display_name
        && !name.trim().is_empty()
        && !name.eq_ignore_ascii_case(login)
    {
        identities.push(name.to_string());
    }
    identities.push(format!("{login}@users.noreply.github.com"));
    identities
}

/// Parsed authored-commit totals for one repository.
pub struct RepoAnalysis {
    pub repo_name: String,
    pub lines_by_lang: BTreeMap<String, u64>,
    pub added: u64,
    pub deleted: u64,
    pub files: u64,
    pub commits: u64,
}

struct TempClone(PathBuf);

impl Drop for TempClone {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn clone_bare(url: &str, destination: &Path, single_branch: bool) -> Result<bool, String> {
    let mut command = Command::new("git");
    command.args(["clone", "--bare", "--quiet"]);
    if single_branch {
        command.arg("--single-branch");
    }
    let status = command
        .arg(url)
        .arg(destination)
        .status()
        .map_err(|e| format!("failed to start git clone: {e}"))?;
    Ok(status.success())
}

fn escape_regex(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(
            ch,
            '\\' | '^' | '$' | '.' | '[' | ']' | '*' | '+' | '?' | '{' | '}' | '(' | ')' | '|'
        ) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

/// Case-insensitive language exclusion check used by the numstat parser.
pub fn is_ignored(language: &str, ignored: &[String]) -> bool {
    ignored
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(language))
}

/// Clone one repository and stream its authored `git log --numstat` output.
/// The clone is removed automatically before this function returns.
pub fn analyze_repo(
    token: &str,
    owner_repo: &str,
    author_identities: &[&str],
    ignored: &[String],
) -> Result<RepoAnalysis, String> {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let destination = std::env::temp_dir().join(format!(
        "metrics-rs-indepth-{}-{nanos}-{sequence}",
        std::process::id()
    ));
    let _cleanup = TempClone(destination.clone());
    let clone_url = format!("https://{token}@github.com/{owner_repo}");

    let cloned = clone_bare(&clone_url, &destination, true)?;
    if !cloned {
        eprintln!("indepth: {owner_repo}: single-branch clone failed; retrying full clone");
        let _ = std::fs::remove_dir_all(&destination);
        if !clone_bare(&clone_url, &destination, false)? {
            return Err(format!("clone failed for {owner_repo}"));
        }
    }

    let mut command = Command::new("git");
    command
        .current_dir(&destination)
        .args(["log", "--no-merges", "--numstat", "--format=COMMIT %H"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for identity in author_identities {
        command.arg(format!("--author={}", escape_regex(identity)));
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("failed to start git log for {owner_repo}: {e}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| format!("git log for {owner_repo} did not provide a stdout stream"))?;
    let mut analysis = RepoAnalysis {
        repo_name: owner_repo.to_string(),
        lines_by_lang: BTreeMap::new(),
        added: 0,
        deleted: 0,
        files: 0,
        commits: 0,
    };

    for line in BufReader::new(stdout).lines() {
        let line = line.map_err(|e| format!("failed reading git log for {owner_repo}: {e}"))?;
        if line.starts_with("COMMIT ") {
            analysis.commits += 1;
            continue;
        }
        let mut fields = line.splitn(3, '\t');
        let (Some(added), Some(deleted), Some(path)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Ok(added) = added.parse::<u64>() else {
            // Binary file numstat uses `-` for both line counts.
            continue;
        };
        let deleted = deleted.parse::<u64>().unwrap_or(0);
        analysis.added += added;
        analysis.deleted += deleted;
        analysis.files += 1;
        if let Some((language, _)) = classify_path(path)
            && !is_ignored(language, ignored)
        {
            *analysis
                .lines_by_lang
                .entry(language.to_string())
                .or_insert(0) += added;
        }
    }

    let status = child
        .wait()
        .map_err(|e| format!("failed waiting for git log on {owner_repo}: {e}"))?;
    if !status.success() {
        return Err(format!("git log failed for {owner_repo}"));
    }
    Ok(analysis)
}

/// Analyze repositories with a bounded worker set. Individual clone/log
/// failures are non-fatal and reported to stderr; successful results remain.
pub fn analyze_all(
    token: &str,
    repositories: &[String],
    author_identities: &[String],
    ignored: &[String],
) -> Vec<RepoAnalysis> {
    if repositories.is_empty() {
        return Vec::new();
    }
    let ids: Vec<&str> = author_identities.iter().map(String::as_str).collect();
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::with_capacity(repositories.len()));
    let workers = repositories.len().min(CONCURRENCY);

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(repository) = repositories.get(index) else {
                        break;
                    };
                    match analyze_repo(token, repository, &ids, ignored) {
                        Ok(result) => results.lock().expect("results mutex poisoned").push(result),
                        Err(error) => eprintln!("indepth: {error}"),
                    }
                }
            });
        }
    });

    results.into_inner().expect("results mutex poisoned")
}

/// Merge per-repository results into the profile's in-depth summary data.
pub fn merge(
    results: &[RepoAnalysis],
    ignored: &[String],
    repositories_total: usize,
) -> IndepthStats {
    let mut stats = IndepthStats {
        repos_total: repositories_total,
        repos_analyzed: results.len(),
        ignored: ignored.to_vec(),
        ..IndepthStats::default()
    };
    for result in results {
        stats.added += result.added;
        stats.deleted += result.deleted;
        stats.files += result.files;
        stats.commits += result.commits;
        for (language, lines) in &result.lines_by_lang {
            *stats.lines_by_lang.entry(language.clone()).or_insert(0) += lines;
            stats
                .repos_by_lang
                .entry(language.clone())
                .or_default()
                .push(result.repo_name.clone());
        }
    }
    for repositories in stats.repos_by_lang.values_mut() {
        repositories.sort();
        repositories.dedup();
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_include_login_name_and_noreply_email() {
        assert_eq!(
            author_identities("jackra1n", Some("Jack")),
            vec!["jackra1n", "Jack", "jackra1n@users.noreply.github.com"]
        );
        assert_eq!(
            author_identities("jackra1n", Some("jackra1n")),
            vec!["jackra1n", "jackra1n@users.noreply.github.com"]
        );
    }

    #[test]
    fn ignored_languages_match_case_insensitively() {
        let ignored = vec!["swift".to_string(), "gdscript".to_string()];
        assert!(is_ignored("Swift", &ignored));
        assert!(is_ignored("GDScript", &ignored));
        assert!(!is_ignored("Rust", &ignored));
    }

    #[test]
    fn merge_sums_lines_and_repo_attribution() {
        let first = RepoAnalysis {
            repo_name: "u/a".into(),
            lines_by_lang: [("Rust".to_string(), 10), ("Python".to_string(), 5)]
                .into_iter()
                .collect(),
            added: 15,
            deleted: 2,
            files: 2,
            commits: 3,
        };
        let second = RepoAnalysis {
            repo_name: "u/b".into(),
            lines_by_lang: [("Rust".to_string(), 7)].into_iter().collect(),
            added: 7,
            deleted: 1,
            files: 1,
            commits: 1,
        };
        let stats = merge(&[first, second], &[], 2);
        assert_eq!(stats.lines_by_lang["Rust"], 17);
        assert_eq!(stats.lines_by_lang["Python"], 5);
        assert_eq!(stats.repos_by_lang["Rust"], vec!["u/a", "u/b"]);
        assert_eq!(stats.added, 22);
        assert_eq!(stats.deleted, 3);
        assert_eq!(stats.files, 3);
        assert_eq!(stats.commits, 4);
        assert_eq!(stats.repos_analyzed, 2);
    }
}
