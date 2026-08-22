mod cli;
mod gh;
mod indepth;
mod langs;
mod model;
mod render;
mod summary;
use crate::cli::Args;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse(std::env::args().collect())?;
    let profile = run(&args)?;
    std::fs::write(&args.output, render::render(&profile))?;
    summary::print_summary(&profile);
    Ok(())
}

/// Most common non-Other license across repos by count; ties break to the
/// lexicographically smaller SPDX id for determinism.
fn preferred_license(repos: &[model::GhRepo]) -> Option<String> {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for repo in repos {
        if let Some(license) = &repo.license {
            *counts.entry(license.clone()).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .filter(|(_, n)| *n > 0)
        .map(|(id, _)| id)
}

fn run(args: &Args) -> Result<model::Profile, Box<dyn std::error::Error>> {
    let agent = gh::agent();
    let user = gh::fetch_profile(&agent, &args.token, &args.username)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // non-fatal: private/no-contribution profiles just get an empty graph
    let activity =
        gh::fetch_activity(&agent, &args.token, &args.username, now).unwrap_or_else(|e| {
            eprintln!("activity: {e}");
            model::Activity::default()
        });

    let (languages, lines, indepth_stats) = if args.indepth {
        let targets = gh::indepth_targets(&user, &activity);
        eprintln!(
            "indepth: analyzing authored commits across {} repositories in parallel...",
            targets.len()
        );
        let identities = indepth::author_identities(&user.login, user.name.as_deref());
        let results =
            indepth::analyze_all(&args.token, &targets, &identities, &args.ignored_languages);
        let stats = indepth::merge(&results, &args.ignored_languages, targets.len());
        let lines = model::LineTotals {
            added: stats.added,
            deleted: stats.deleted,
            commits: stats.commits,
        };
        (
            langs::top_languages_from_lines(&stats.lines_by_lang),
            lines,
            Some(stats),
        )
    } else {
        eprintln!(
            "lines: aggregating weekly stats for {} repos",
            user.repos.len()
        );
        (
            langs::top_languages(&user.repos),
            gh::collect_lines(&agent, &args.token, &user),
            None,
        )
    };

    let sum = |f: fn(&model::GhRepo) -> u64| user.repos.iter().map(f).sum();
    Ok(model::Profile {
        stars: sum(|r| r.stars),
        forks: sum(|r| r.forks),
        watchers: sum(|r| r.watchers),
        releases: sum(|r| r.releases),
        storage_kb: sum(|r| r.disk_usage_kb),
        preferred_license: preferred_license(&user.repos),
        languages,
        activity,
        lines,
        indepth: indepth_stats,
        user,
    })
}
