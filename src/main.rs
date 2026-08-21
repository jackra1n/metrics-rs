mod cli;
mod gh;
mod langs;
mod model;
mod render;

use crate::cli::Args;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse(std::env::args().collect())?;
    let profile = run(&args)?;
    std::fs::write(&args.output, render::render(&profile))?;
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
    eprintln!(
        "lines: aggregating weekly stats for {} repos",
        user.repos.len()
    );
    let lines = gh::collect_lines(&agent, &args.token, &user);
    let languages = langs::top_languages(&user.repos);
    let sum = |f: fn(&model::GhRepo) -> u64| user.repos.iter().map(f).sum();
    Ok(model::Profile {
        stars: sum(|r| r.stars),
        forks: sum(|r| r.forks),
        watchers: sum(|r| r.watchers),
        releases: sum(|r| r.releases),
        storage_kb: sum(|r| r.disk_usage_kb),
        preferred_license: preferred_license(&user.repos),
        languages,
        lines,
        user,
    })
}
