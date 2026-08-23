use crate::model::Profile;

// GitHub-dark palette.
const BG: &str = "#0d1117";
const STROKE: &str = "#30363d";
const DIVIDER: &str = "#21262d";
const TEXT: &str = "#e6edf3";
const MUTED: &str = "#8b949e";

const W: u32 = 480;
const X0: f64 = 16.0;
const X1: f64 = 464.0;
const CHART_W: f64 = X1 - X0;

/// XML-escape user-controlled text (name, login, language names).
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Compact number: >=1e6 -> `1.5m`, >=1e3 -> `1.2k` (trailing `.0` trimmed),
/// else plain integer.
pub fn fmt(n: u64) -> String {
    let compact = |v: f64, suffix: &str| {
        let s = format!("{v:.1}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        format!("{s}{suffix}")
    };
    if n >= 1_000_000 {
        compact(n as f64 / 1_000_000.0, "m")
    } else if n >= 1_000 {
        compact(n as f64 / 1_000.0, "k")
    } else {
        n.to_string()
    }
}

/// Human storage size from KB: GB at >=1000k KB, MB at >=1000 KB, else KB.
fn fmt_storage(kb: u64) -> String {
    if kb >= 1_000_000 {
        format!("{:.1}GB", kb as f64 / 1_000_000.0)
    } else if kb >= 1_000 {
        let mb = kb as f64 / 1_000.0;
        let s = format!("{mb:.1}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        format!("{s}MB")
    } else {
        format!("{kb}KB")
    }
}

/// Coarse age of an ISO timestamp ("2018-11-14T13:40:45Z") relative to the
/// Unix epoch, using only UTC arithmetic: years when >=1y, then months,
/// then days. `now_secs` injected for testability/determinism.
pub fn joined_age(created_iso: &str, now_secs: u64) -> String {
    let parse = |s: &str| -> Option<(i64, i64, i64)> {
        let y: i64 = s.get(0..4)?.parse().ok()?;
        let m: i64 = s.get(5..7)?.parse().ok()?;
        let d: i64 = s.get(8..10)?.parse().ok()?;
        Some((y, m, d))
    };
    // days-from-civil (Howard Hinnant), inverted: civil date -> days
    let days_from_civil = |(mut y, m, d): (i64, i64, i64)| -> i64 {
        y -= if m <= 2 { 1 } else { 0 };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let mp = if m > 2 { m - 3 } else { m + 9 };
        let doy = (153 * mp + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    };
    let now_days = (now_secs / 86_400) as i64;
    let Some(cd) = parse(created_iso).map(days_from_civil) else {
        return "a while".to_string();
    };
    let delta = (now_days - cd).max(0);
    let years = delta / 365;
    if years >= 1 {
        return format!("{years}y ago");
    }
    let months = delta * 12 / 365;
    if months >= 1 {
        return format!("{months}mo ago");
    }
    match delta {
        0 => "today".to_string(),
        1 => "yesterday".to_string(),
        n => format!("{n}d ago"),
    }
}

fn text(x: f64, y: f64, size: u32, fill: &str, content: &str) -> String {
    format!(
        r#"  <text x="{x}" y="{y}" font-size="{size}" fill="{fill}">{content}</text>"#,
        x = coord(x),
        y = coord(y),
        size = size,
        fill = fill,
        content = content
    )
}

fn bold(x: f64, y: f64, size: u32, fill: &str, content: &str) -> String {
    format!(
        r#"  <text x="{x}" y="{y}" font-size="{size}" font-weight="bold" fill="{fill}">{content}</text>"#,
        x = coord(x),
        y = coord(y),
        size = size,
        fill = fill,
        content = content
    )
}

/// Integer-valued coords without decimal noise; others with 2 decimals.
fn coord(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v:.2}")
    }
}

struct Builder {
    y: f64,
    body: String,
}

impl Builder {
    fn push(&mut self, s: String) {
        self.body.push_str(&s);
        self.body.push('\n');
    }
}

/// Minimal 12x12 Feather-style icon paths keyed by fact name, drawn at
/// (x,y) = top-left of the icon box, stroked in `color`.
fn icon(name: &str, x: f64, y: f64, color: &str) -> String {
    let p = |d: &str| {
        format!(
            r#"  <g transform="translate({x},{y}) scale(0.5)" fill="none" stroke="{color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{d}</g>"#,
            x = coord(x),
            y = coord(y),
            color = color,
            d = d
        )
    };
    match name {
        // document / license
        "TOP LICENSE" | "PREFERRED LICENSE" | "LICENSE" => p(
            r#"<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/>"#,
        ),
        // star
        "STARGAZERS" => p(
            r#"<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>"#,
        ),
        // eye
        "WATCHERS" => p(
            r#"<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>"#,
        ),
        // git fork
        "FORKS" => p(
            r#"<circle cx="12" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><circle cx="18" cy="6" r="3"/><path d="M18 9v2a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V9M12 13v2"/>"#,
        ),
        // heart
        "SPONSORS" => p(
            r#"<path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>"#,
        ),
        // package
        "RELEASES" => p(
            r#"<path d="M16.5 9.4 7.55 4.24M21 8v8a2 2 0 0 1-1 1.73l-7 4a2 2 0 0 1-2 0l-7-4A2 2 0 0 1 3 16V8a2 2 0 0 1 1-1.73l7-4a2 2 0 0 1 2 0l7 4A2 2 0 0 1 21 8zM3.3 7l8.7 5 8.7-5M12 22V12"/>"#,
        ),
        // hard drive
        "REPO STORAGE" | "STORAGE" => p(
            r#"<line x1="22" y1="12" x2="2" y2="12"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/><line x1="6" y1="16" x2="6.01" y2="16"/><line x1="10" y1="16" x2="10.01" y2="16"/>"#,
        ),
        // activity / pulse
        "LINES CHANGED" | "LINES" => {
            p(r#"<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>"#)
        }
        _ => String::new(),
    }
}

/// Render the profile card: compact header, 2x4 fact grid, language bar.
/// Body is built first with a row cursor, then wrapped in the root `<svg>`
/// so height is known before emission. Fully deterministic.
pub fn render(p: &Profile) -> String {
    // wall-clock reference for joined-age; only the coarse age string is
    // rendered, so output stays stable within a day
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut b = Builder {
        y: 16.0,
        body: String::new(),
    };

    // header: small avatar + identity (20px avatar in a 48px band)
    b.push(r#"  <clipPath id="av"><circle cx="36" cy="36" r="10"/></clipPath>"#.to_string());
    b.push(format!(
        r#"  <image href="{}" x="26" y="26" width="20" height="20" clip-path="url(#av)"/>"#,
        esc(&p.user.avatar_url)
    ));
    let display_name = p.user.name.as_deref().unwrap_or(&p.user.login);
    b.push(bold(56.0, 34.0, 14, TEXT, &esc(display_name)));
    b.push(text(
        56.0,
        50.0,
        10,
        MUTED,
        &format!("joined {}", joined_age(&p.user.created_at, now)),
    ));

    // mini contribution graph, right-aligned in the header band:
    // 14 day-squares, GitHub-dark heat colors
    if !p.activity.days.is_empty() {
        const LEVELS: [&str; 5] = ["#161b22", "#0e4429", "#006d32", "#26a641", "#39d353"];
        const SQ: f64 = 8.0;
        const GAP: f64 = 3.0;
        let n = p.activity.days.len() as f64;
        let gx = X1 - (n * SQ + (n - 1.0) * GAP);
        let max = p
            .activity
            .days
            .iter()
            .map(|(_, c)| *c)
            .max()
            .unwrap_or(1)
            .max(1);
        for (i, (_, c)) in p.activity.days.iter().enumerate() {
            let level = if *c == 0 {
                0
            } else {
                1 + (*c * 3 / max) as usize
            };
            b.push(format!(
                r#"  <rect x="{x}" y="28" width="8" height="8" rx="2" fill="{fill}"/>"#,
                x = coord(gx + i as f64 * (SQ + GAP)),
                fill = LEVELS[level.min(4)],
            ));
        }
        b.push(format!(
            r#"  <text x="{}" y="50" text-anchor="end" font-size="8" fill="{f}">contributed to {} repositories</text>"#,
            coord(X1),
            p.user.repos_contributed,
            f = MUTED,
        ));
    }
    b.y += 64.0;

    // divider
    b.push(format!(
        r#"  <line x1="16" y1="{}" x2="464" y2="{}" stroke="{}"/>"#,
        b.y, b.y, DIVIDER
    ));
    b.y += 16.0;
    // fact grid: 2 rows x 4 columns, icon left of label
    let license_label = p.preferred_license.as_deref().unwrap_or("None");
    let facts: [(&str, String); 8] = [
        ("TOP LICENSE", esc(license_label)),
        ("STARGAZERS", fmt(p.stars)),
        ("WATCHERS", fmt(p.watchers)),
        ("FORKS", fmt(p.forks)),
        ("SPONSORS", fmt(p.user.sponsors)),
        ("RELEASES", fmt(p.releases)),
        ("REPO STORAGE", fmt_storage(p.storage_kb)),
        ("LINES CHANGED", String::new()),
    ];
    const XS: [f64; 4] = [16.0, 136.0, 256.0, 376.0];
    for (i, (label, value)) in facts.into_iter().enumerate() {
        let col = i % 4;
        let row = i / 4;
        let lx = XS[col];
        let ly = b.y + row as f64 * 44.0;
        b.push(icon(label, lx + 1.0, ly - 9.0, MUTED));
        b.push(text(lx + 15.0, ly, 9, MUTED, label));
        if label == "LINES CHANGED" {
            b.push(format!(
                r#"  <text x="{x}" y="{y}" font-size="11" font-weight="bold" fill="{color}">+{added}</text>"#,
                x = coord(lx + 1.0),
                y = coord(ly + 13.0),
                color = "#3fb950",
                added = fmt(p.lines.added),
            ));
            b.push(format!(
                r#"  <text x="{x}" y="{y}" font-size="11" font-weight="bold" fill="{color}">−{deleted}</text>"#,
                x = coord(lx + 1.0),
                y = coord(ly + 26.0),
                color = "#f85149",
                deleted = fmt(p.lines.deleted),
            ));
        } else {
            b.push(bold(lx + 1.0, ly + 18.0, 15, TEXT, &value));
        }
    }
    b.y += 88.0;

    // divider between facts and languages
    b.push(format!(
        r#"  <line x1="16" y1="{}" x2="464" y2="{}" stroke="{}"/>"#,
        b.y, b.y, DIVIDER
    ));
    b.y += 14.0;

    // languages section
    b.push(text(X0, b.y, 9, MUTED, "LANGUAGES"));
    b.y += 13.0;

    // faint analysis-meta line: what the numbers above were computed over,
    // centered directly above the language bar
    let meta = format!(
        "analyzed {} repos · {} · {} commits",
        p.user.repos_total,
        fmt_storage(p.storage_kb),
        fmt(p.lines.commits),
    );
    b.push(format!(
        r#"  <text x="{}" y="{}" text-anchor="middle" font-size="8" fill="{m}">{m2}</text>"#,
        coord(W as f64 / 2.0),
        coord(b.y + 7.0),
        m = MUTED,
        m2 = esc(&meta),
    ));
    b.y += 15.0;

    if p.languages.is_empty() {
        b.push(text(
            W as f64 / 2.0,
            b.y + 14.0,
            10,
            MUTED,
            "no language data",
        ));
        b.y += 22.0;
    } else {
        // single clipped bar: segments share one rounded clip so the whole
        // strip reads as one pill, not touching pills
        let total_pct: f64 = p
            .languages
            .iter()
            .map(|l| l.pct)
            .sum::<f64>()
            .max(f64::MIN_POSITIVE);
        let bar_y = b.y;
        let bar_h = 8.0;
        b.push(format!(
            r#"  <clipPath id="bar"><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}"/></clipPath>"#,
            x = coord(X0),
            y = coord(bar_y),
            w = coord(CHART_W),
            h = coord(bar_h),
            r = coord(bar_h / 2.0),
        ));
        let mut bx = X0;
        for (i, lang) in p.languages.iter().enumerate() {
            let seg_w = if i == p.languages.len() - 1 {
                X1 - bx
            } else {
                ((CHART_W * lang.pct / total_pct).round() * 100.0) / 100.0
            };
            b.push(format!(
                r#"  <rect x="{bx}" y="{y}" width="{w}" height="{h}" fill="{fill}" clip-path="url(#bar)"/>"#,
                bx = coord(bx),
                y = coord(bar_y),
                w = coord(seg_w),
                h = coord(bar_h),
                fill = lang.color,
            ));
            bx += seg_w;
        }
        b.y += bar_h + 14.0;

        // legend grid: 2 cols x up to 5 rows
        for (i, lang) in p.languages.iter().enumerate() {
            let col = i % 2;
            let row = i / 2;
            let lx = X0 + col as f64 * 224.0;
            let ly = b.y + 8.0 + row as f64 * 17.0;
            b.push(format!(
                r#"  <circle cx="{cx}" cy="{cy}" r="4" fill="{fill}"/>"#,
                cx = coord(lx + 4.0),
                cy = coord(ly - 4.0),
                fill = lang.color,
            ));
            b.push(text(lx + 14.0, ly, 10, TEXT, &esc(&lang.name)));
            b.push(format!(
                r#"  <text x="{x}" y="{y}" text-anchor="end" font-size="10" fill="{fill}">{pct:.1}%</text>"#,
                x = coord(lx + 208.0),
                y = coord(ly),
                fill = MUTED,
                pct = lang.pct,
            ));
        }
        let rows = p.languages.len().div_ceil(2);
        b.y += rows as f64 * 17.0 + 6.0;
    }

    let h = b.y + 16.0;
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
<rect width="{w}" height="{h}" rx="6" fill="{bg}" stroke="{stroke}"/>
<g font-family="-apple-system,Segoe UI,Ubuntu,Sans-serif">
"#,
        w = W,
        h = h,
        bg = BG,
        stroke = STROKE,
    );
    svg.push_str(&b.body);
    svg.push_str("</g>\n</svg>\n");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Activity, GhRepo, GhUser, LangStat, LineTotals, Profile};

    fn profile() -> Profile {
        Profile {
            user: GhUser {
                login: "jackra1n".into(),
                name: Some("Jack & <Friends>".into()),
                created_at: "2018-11-14T13:40:45Z".into(),
                avatar_url: "https://avatars.githubusercontent.com/u/1?v=4".into(),
                sponsors: 3,
                repos_total: 2,
                repos_contributed: 45,
                repos: vec![GhRepo {
                    name: "r".into(),
                    stars: 7,
                    forks: 3,
                    watchers: 1,
                    releases: 2,
                    disk_usage_kb: 2048,
                    license: Some("MIT".into()),
                    langs: vec![],
                }],
            },
            stars: 7,
            forks: 3,
            watchers: 1,
            releases: 2,
            storage_kb: 2048,
            preferred_license: Some("MIT".into()),
            languages: vec![LangStat {
                name: "Rust".into(),
                pct: 100.0,
                color: "#dea584",
            }],
            activity: Activity {
                days: (0..14)
                    .map(|i| (format!("2026-08-{i:02}"), if i % 3 == 0 { 5 } else { 0 }))
                    .collect(),
                repos: 4,
                contrib_repos: Vec::new(),
            },
            lines: LineTotals {
                added: 1234,
                deleted: 56_000,
                commits: 890,
            },
            indepth: None,
        }
    }

    #[test]
    fn fmt_boundaries() {
        assert_eq!(fmt(999), "999");
        assert_eq!(fmt(1234), "1.2k");
        assert_eq!(fmt(1500000), "1.5m");
        assert_eq!(fmt(1_000_000), "1m");
    }

    #[test]
    fn storage_formatting() {
        assert_eq!(fmt_storage(512), "512KB");
        assert_eq!(fmt_storage(2048), "2MB");
        assert_eq!(fmt_storage(1500), "1.5MB");
        assert_eq!(fmt_storage(2500000), "2.5GB");
    }

    #[test]
    fn joined_age_coarse_units() {
        // fixed reference instant, 2026-08-22 UTC
        let now = 1_787_395_200;
        assert_eq!(joined_age("2026-08-20T00:00:00Z", now), "2d ago");
        assert_eq!(joined_age("2026-07-01T00:00:00Z", now), "1mo ago");
        assert_eq!(joined_age("2018-11-14T13:40:45Z", now), "7y ago");
    }

    #[test]
    fn esc_escapes_all_metacharacters() {
        assert_eq!(esc("&<a>"), "&amp;&lt;a&gt;");
        assert_eq!(esc("\"'"), "&quot;&#39;");
    }

    #[test]
    fn render_shape_and_escaping() {
        let svg = render(&profile());
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("#0d1117"));
        assert!(svg.contains("Jack &amp; &lt;Friends&gt;"));
        for label in [
            "TOP LICENSE",
            "STARGAZERS",
            "WATCHERS",
            "FORKS",
            "SPONSORS",
            "RELEASES",
            "REPO STORAGE",
            "LINES CHANGED",
        ] {
            assert!(svg.contains(label), "missing {label}");
        }
        assert!(svg.contains("contributed to 45 repositories"));
        assert!(svg.contains("#39d353"), "heat colors missing");
        assert!(!svg.contains("followers • "));
        // the old chart polygons carried opacity; icons don't
        assert!(!svg.contains("opacity"), "stale chart polygon");
        assert!(svg.ends_with("</svg>\n"));
    }

    #[test]
    fn render_deterministic() {
        assert_eq!(render(&profile()), render(&profile()));
    }
}
