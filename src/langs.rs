use crate::model::{GhRepo, LangStat};
use std::collections::BTreeMap;

const FALLBACK_COLOR: &str = "#ededed";
const OTHER_COLOR: &str = "#30363d";
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
/// keep top 8 + an "Other" bucket when there is a remainder.
pub fn top_languages(repos: &[GhRepo]) -> Vec<LangStat> {
    let mut sizes: BTreeMap<String, u64> = BTreeMap::new();
    let mut total = 0u64;
    for repo in repos {
        for (name, size) in &repo.langs {
            *sizes.entry(name.clone()).or_insert(0) += size;
            total += size;
        }
    }
    if total == 0 {
        return Vec::new();
    }
    let mut ranked: Vec<(String, u64)> = sizes.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut out: Vec<LangStat> = ranked
        .iter()
        .take(TOP_N)
        .map(|(name, size)| LangStat {
            name: name.clone(),
            size: *size,
            pct: *size as f64 / total as f64 * 100.0,
            color: resolve_color(name),
        })
        .collect();
    let rest: u64 = ranked.iter().skip(TOP_N).map(|(_, s)| s).sum();
    if rest > 0 {
        out.push(LangStat {
            name: "Other".to_string(),
            size: rest,
            pct: rest as f64 / total as f64 * 100.0,
            color: OTHER_COLOR,
        });
    }
    out
}
