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

fn is_real_color(color: &str) -> bool {
    !color.is_empty() && color != "#000000"
}

/// Classify a repo-relative file path via the bundled Linguist table.
/// Returns `(language name, display color)` for programming/markup languages
/// only — Data (`.json`, `.svg`, `.lock`) and Prose (`.md`) are skipped.
///
/// Ambiguous extensions prefer candidates with a real Linguist color over
/// null-color placeholders, so `.rs` resolves to Rust rather than RenderScript.
pub fn classify_path(path: &str) -> Option<(&'static str, &'static str)> {
    let langs = github_languages::get_languages();
    let base = path.rsplit('/').next().unwrap_or(path);
    let candidates = if base.is_empty() {
        return None;
    } else {
        let by_filename = langs.get_by_filename(base);
        if !by_filename.is_empty() {
            by_filename
        } else {
            let (_, ext) = base.rsplit_once('.')?;
            if ext.is_empty() {
                return None;
            }
            let mut dotted = String::with_capacity(ext.len() + 1);
            dotted.push('.');
            dotted.extend(ext.chars().flat_map(char::to_lowercase));
            langs.get_by_extension(&dotted)
        }
    };
    if candidates.iter().any(|language| language.r#type == "prose") {
        return None;
    }
    let mut typed: Vec<_> = candidates
        .into_iter()
        .filter(|language| matches!(language.r#type, "programming" | "markup"))
        .collect();
    if typed.is_empty() {
        return None;
    }
    if typed.len() > 1 {
        let colored: Vec<_> = typed
            .iter()
            .filter(|language| is_real_color(language.color))
            .cloned()
            .collect();
        if !colored.is_empty() {
            typed = colored;
        }
    }
    let language = &typed[0];
    Some((language.name, resolve_color(language.name)))
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
    rank(sizes)
}

/// Same top-8 ranking over authored lines added (--indepth mode).
pub fn top_languages_from_lines(lines: &BTreeMap<String, u64>) -> Vec<LangStat> {
    rank(lines.clone())
}

fn rank(sizes: BTreeMap<String, u64>) -> Vec<LangStat> {
    let mut ranked: Vec<(String, u64)> = sizes.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let top: Vec<(String, u64)> = ranked.into_iter().take(TOP_N).collect();
    let total: u64 = top.iter().map(|(_, size)| *size).sum();
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
        assert_eq!(out[0].name, "Rust");
        assert_eq!(out[1].name, "Shell");
        assert!((out[0].pct - 200.0 / 3.0).abs() < 1e-9);
        assert!((out[1].pct - 100.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn rust_resolves_linguist_color_from_bundled_table() {
        let out = top_languages(&[repo(vec![("Rust", 10)])]);
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
        assert!(!out.iter().any(|language| language.name == "Other"));
        for language in &out {
            assert!((language.pct - 12.5).abs() < 1e-9);
        }
    }

    #[test]
    fn empty_input_yields_empty() {
        assert!(top_languages(&[]).is_empty());
    }

    #[test]
    fn classify_common_paths() {
        let cases: [(&str, &str, &str); 5] = [
            ("src/main.rs", "Rust", "#dea584"),
            ("web/App.svelte", "Svelte", "#ff3e00"),
            ("lib/util.ts", "TypeScript", "#3178c6"),
            ("main.py", "Python", "#3572A5"),
            ("Dockerfile", "Dockerfile", "#384d54"),
        ];
        for (path, name, color) in cases {
            let got = classify_path(path).unwrap_or_else(|| panic!("expected {name} for {path}"));
            assert_eq!(got.0, name, "wrong language for {path}");
            assert_eq!(got.1, color, "wrong color for {path}");
        }
    }

    #[test]
    fn classify_skips_data_and_prose() {
        for path in [
            "data.json",
            "assets/icon.svg",
            "Cargo.lock",
            "README.md",
            "package-lock.json",
            "no_extension",
        ] {
            assert_eq!(classify_path(path), None, "{path} should be skipped");
        }
    }

    #[test]
    fn classify_case_insensitive_extension() {
        assert_eq!(
            classify_path("src/Main.RS").map(|(name, _)| name),
            Some("Rust")
        );
    }

    #[test]
    fn top_languages_from_lines_ranks_and_normalizes() {
        let lines = BTreeMap::from([
            ("Rust".to_string(), 600),
            ("Python".to_string(), 200),
            ("Shell".to_string(), 200),
        ]);
        let out = top_languages_from_lines(&lines);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].name, "Rust");
        assert!((out[0].pct - 60.0).abs() < 1e-9);
        assert_eq!(out[1].name, "Python");
        assert_eq!(out[2].name, "Shell");
        assert!(top_languages_from_lines(&BTreeMap::new()).is_empty());
    }
}
