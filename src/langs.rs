use crate::model::{GhRepo, LangStat};
use std::collections::BTreeMap;

const FALLBACK_COLOR: &str = "#ededed";
const TOP_N: usize = 8;

/// Linguist lookup with dark-bg guard: misses and null-color entries
/// (`#000000`) both fall back to metrics' `#ededed`.
fn resolve_color(name: &str) -> &'static str {
    match github_languages::get_languages().get_by_name(name) {
        Some(info) if info.color != "#000000" && !info.color.is_empty() => info.color,
        _ => FALLBACK_COLOR,
    }
}

/// Merge every repo's language bytes, rank by size desc (name asc tie-break),
/// keep top 8 languages normalized to 100%.
pub fn top_languages(repos: &[GhRepo]) -> Vec<LangStat> {
    let mut sizes: BTreeMap<String, u64> = BTreeMap::new();
    for repo in repos {
        for (name, size) in &repo.langs {
            *sizes.entry(name.clone()).or_insert(0) += size;
        }
    }
    let mut ranked: Vec<(String, u64)> = sizes.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let top: Vec<(String, u64)> = ranked.into_iter().take(TOP_N).collect();
    let total: u64 = top.iter().map(|(_, s)| *s).sum();
    if total == 0 {
        return Vec::new();
    }
    top.into_iter()
        .map(|(name, size)| LangStat {
            color: resolve_color(&name),
            pct: size as f64 / total as f64 * 100.0,
            name,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::GhRepo;

    fn repo(langs: Vec<(&str, u64)>) -> GhRepo {
        GhRepo {
            name: "r".into(),
            stars: 0,
            forks: 0,
            watchers: 0,
            releases: 0,
            disk_usage_kb: 0,
            license: None,
            langs: langs.into_iter().map(|(n, s)| (n.to_string(), s)).collect(),
        }
    }

    #[test]
    fn merges_and_tie_breaks_by_name() {
        let repos = vec![
            repo(vec![("Rust", 100), ("Shell", 50)]),
            repo(vec![("Rust", 100), ("Shell", 50)]),
        ];
        let out = top_languages(&repos);
        assert_eq!(out.len(), 2);
        // equal totals -> alphabetical tie-break
        assert_eq!(out[0].name, "Rust");
        assert_eq!(out[1].name, "Shell");
        assert!((out[0].pct - 200.0 / 3.0).abs() < 1e-9);
        assert!((out[1].pct - 100.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn rust_resolves_linguist_color_from_bundled_table() {
        let repos = vec![repo(vec![("Rust", 10)])];
        let out = top_languages(&repos);
        assert_eq!(out[0].color, "#dea584");
    }

    #[test]
    fn unknown_language_falls_back() {
        assert_eq!(resolve_color("Not A Real Language"), FALLBACK_COLOR);
    }

    #[test]
    fn limits_to_top_8_without_other() {
        let repos = vec![
            repo(vec![("A", 10), ("B", 10), ("C", 10), ("D", 10)]),
            repo(vec![("E", 10), ("F", 10), ("G", 10), ("H", 10), ("I", 10)]),
        ];
        let out = top_languages(&repos);
        assert_eq!(out.len(), 8);
        assert!(!out.iter().any(|l| l.name == "Other"));
        // 8 languages with equal weight 10 sum to 80, each is 12.5%
        for lang in &out {
            assert!((lang.pct - 12.5).abs() < 1e-9);
        }
    }

    #[test]
    fn empty_input_yields_empty() {
        assert!(top_languages(&[]).is_empty());
    }
}
