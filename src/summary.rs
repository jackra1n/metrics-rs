use crate::model::Profile;
use std::collections::BTreeMap;

/// Human-readable byte size from raw bytes (B, KB, MB, GB).
pub fn fmt_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        let gb = bytes as f64 / 1_000_000_000.0;
        let s = format!("{gb:.1}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        format!("{s}GB")
    } else if bytes >= 1_000_000 {
        let mb = bytes as f64 / 1_000_000.0;
        let s = format!("{mb:.1}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        format!("{s}MB")
    } else if bytes >= 1_000 {
        let kb = bytes as f64 / 1_000.0;
        let s = format!("{kb:.1}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        format!("{s}KB")
    } else {
        format!("{bytes}B")
    }
}

/// Print comprehensive stdout breakdown of all statistics and repo impacts.
pub fn print_summary(p: &Profile) {
    println!("\n=== metrics-rs summary for {} ===", p.user.login);

    // 1. General Profile & Fact Stats
    println!("\n[Profile & Stats]");
    let display_name = p.user.name.as_deref().unwrap_or(&p.user.login);
    println!("  User: {} ({})", display_name, p.user.login);
    println!("  Analyzed repos: {} owned", p.user.repos.len());
    println!(
        "  Total disk storage: {}",
        crate::render::fmt(p.storage_kb / 1000) + "MB" // or fmt_bytes(p.storage_kb * 1024)
    );
    println!(
        "  Code lines: +{} / −{} ({} commits)",
        crate::render::fmt(p.lines.added),
        crate::render::fmt(p.lines.deleted),
        crate::render::fmt(p.lines.commits),
    );
    println!(
        "  Community: {} stars · {} watchers · {} forks · {} releases · {} sponsors",
        p.stars, p.watchers, p.forks, p.releases, p.user.sponsors
    );

    // License distribution
    let mut license_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut unlicensed = 0usize;
    for r in &p.user.repos {
        match &r.license {
            Some(lic) => *license_counts.entry(lic.clone()).or_insert(0) += 1,
            None => unlicensed += 1,
        }
    }
    let mut ranked_licenses: Vec<(String, usize)> = license_counts.into_iter().collect();
    ranked_licenses.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let lic_str = ranked_licenses
        .iter()
        .map(|(lic, count)| format!("{lic} ({count})"))
        .collect::<Vec<_>>()
        .join(", ");
    let pref = p.preferred_license.as_deref().unwrap_or("none");
    if unlicensed > 0 {
        println!("  Licenses: preferred: {pref} | breakdown: {lic_str}, None ({unlicensed})");
    } else {
        println!("  Licenses: preferred: {pref} | breakdown: {lic_str}");
    }

    // 2. Recent Activity & "Contributed to X repositories"
    println!("\n[Recent Activity (last 14 days)]");
    let total_14d_commits: u64 = p.activity.days.iter().map(|(_, c)| *c).sum();
    println!(
        "  Total 14-day commits: {} across {} repositories:",
        total_14d_commits, p.activity.repos
    );
    if p.activity.contrib_repos.is_empty() {
        println!("    (no repository commit contributions in this window)");
    } else {
        for cr in &p.activity.contrib_repos {
            let is_external = !cr.name_with_owner.to_lowercase().starts_with(&format!("{}/", p.user.login.to_lowercase()));
            let ext_tag = if is_external { " [external]" } else { "" };
            println!(
                "    • {}: {} commit{}{}",
                cr.name_with_owner,
                cr.commits,
                if cr.commits == 1 { "" } else { "s" },
                ext_tag
            );
        }
    }

    // 3. Languages Breakdown & Repo Impact
    println!("\n[Languages Breakdown & Repo Impact]");
    let mut total_code_bytes = 0u64;
    // Map: language -> (total_bytes, Vec<(repo_name, bytes)>)
    let mut lang_map: BTreeMap<String, (u64, Vec<(&str, u64)>)> = BTreeMap::new();
    // Map: repo_name -> (total_repo_bytes, Vec<(lang_name, bytes)>)
    let mut repo_code_map: BTreeMap<&str, (u64, Vec<(&str, u64)>)> = BTreeMap::new();

    for repo in &p.user.repos {
        let mut repo_total = 0u64;
        for (lang, bytes) in &repo.langs {
            total_code_bytes += *bytes;
            repo_total += *bytes;
            let entry = lang_map.entry(lang.clone()).or_insert((0, Vec::new()));
            entry.0 += *bytes;
            entry.1.push((&repo.name, *bytes));
        }
        if repo_total > 0 {
            let mut rlangs: Vec<(&str, u64)> = repo.langs.iter().map(|(l, b)| (l.as_str(), *b)).collect();
            rlangs.sort_by(|a, b| b.1.cmp(&a.1));
            repo_code_map.insert(&repo.name, (repo_total, rlangs));
        }
    }

    println!(
        "  Total measured code: {} across {} repos",
        fmt_bytes(total_code_bytes),
        p.user.repos.len()
    );

    if total_code_bytes > 0 {
        // Rank languages by total bytes
        let mut ranked_langs: Vec<(String, u64, Vec<(&str, u64)>)> = lang_map
            .into_iter()
            .map(|(lang, (bytes, mut repos))| {
                repos.sort_by(|a, b| b.1.cmp(&a.1));
                (lang, bytes, repos)
            })
            .collect();
        ranked_langs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        println!("\n  Language share in the languages bar & top contributing repos:");
        for (lang, bytes, repos) in &ranked_langs {
            let overall_pct = *bytes as f64 / total_code_bytes as f64 * 100.0;
            println!(
                "  • {}: {:.1}% ({})",
                lang,
                overall_pct,
                fmt_bytes(*bytes)
            );
            // Show top contributing repos for this language
            for (rname, rbytes) in repos.iter().take(3) {
                let lang_share = *rbytes as f64 / *bytes as f64 * 100.0;
                let total_share = *rbytes as f64 / total_code_bytes as f64 * 100.0;
                println!(
                    "      - {}: {} ({:.1}% of {}, {:.1}% of total code)",
                    rname,
                    fmt_bytes(*rbytes),
                    lang_share,
                    lang,
                    total_share
                );
            }
            if repos.len() > 3 {
                let other_bytes: u64 = repos.iter().skip(3).map(|(_, b)| *b).sum();
                println!(
                    "      - ... and {} other repo{} ({})",
                    repos.len() - 3,
                    if repos.len() - 3 == 1 { "" } else { "s" },
                    fmt_bytes(other_bytes)
                );
            }
        }

        // Top repos by total code volume
        let mut ranked_repos: Vec<(&str, u64, Vec<(&str, u64)>)> = repo_code_map
            .into_iter()
            .map(|(name, (total, langs))| (name, total, langs))
            .collect();
        ranked_repos.sort_by(|a, b| b.1.cmp(&a.1));

        println!("\n  Top repositories affecting total language bar volume:");
        for (i, (rname, rtotal, rlangs)) in ranked_repos.iter().take(10).enumerate() {
            let repo_pct = *rtotal as f64 / total_code_bytes as f64 * 100.0;
            let top_langs_str = rlangs
                .iter()
                .take(3)
                .map(|(l, b)| format!("{l} {}", fmt_bytes(*b)))
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "    {:2}. {:<24} {:>7} ({:4.1}% of all code)  [{}]",
                i + 1,
                rname,
                fmt_bytes(*rtotal),
                repo_pct,
                top_langs_str
            );
        }
        if ranked_repos.len() > 10 {
            println!("    ... ({} more repositories)", ranked_repos.len() - 10);
        }
    }
    println!();
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_bytes_boundaries() {
        assert_eq!(fmt_bytes(0), "0B");
        assert_eq!(fmt_bytes(999), "999B");
        assert_eq!(fmt_bytes(1_000), "1KB");
        assert_eq!(fmt_bytes(1_500), "1.5KB");
        assert_eq!(fmt_bytes(1_000_000), "1MB");
        assert_eq!(fmt_bytes(2_500_000), "2.5MB");
        assert_eq!(fmt_bytes(1_000_000_000), "1GB");
        assert_eq!(fmt_bytes(3_200_000_000), "3.2GB");
    }
}
