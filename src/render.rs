use crate::model::Profile;

// GitHub-dark palette.
const BG: &str = "#0d1117";
const STROKE: &str = "#30363d";
const DIVIDER: &str = "#21262d";
const TEXT: &str = "#e6edf3";
const MUTED: &str = "#8b949e";
const GREEN: &str = "#3fb950";
const RED: &str = "#f85149";

const X0: f64 = 16.0;
const X1: f64 = 464.0;
const CHART_W: f64 = X1 - X0;

/// XML-escape user-controlled text (bio, name, login, language names).
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
        let s = format!("{:.1}", v).trim_end_matches('0').trim_end_matches('.').to_string();
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

fn text(x: f64, y: f64, size: u32, fill: &str, content: &str) -> String {
    format!(
        r#"  <text x="{x}" y="{y}" font-size="{size}" fill="{fill}">{content}</text>"#,
        x = fmt_coord(x),
        y = fmt_coord(y),
        size = size,
        fill = fill,
        content = content
    )
}

fn bold(x: f64, y: f64, size: u32, fill: &str, content: &str) -> String {
    format!(
        r#"  <text x="{x}" y="{y}" font-size="{size}" font-weight="bold" fill="{fill}">{content}</text>"#,
        x = fmt_coord(x),
        y = fmt_coord(y),
        size = size,
        fill = fill,
        content = content
    )
}

fn fmt_coord(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v:.2}")
    }
}

/// Normalize whitespace and word-wrap at `width` chars, max `max_lines`
/// lines; overflow on the final line gets `…`.
fn wrap(s: &str, width: usize, max_lines: usize) -> Vec<String> {
    let flat: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut lines: Vec<String> = Vec::new();
    let mut rest = flat.as_str();
    while lines.len() < max_lines && !rest.is_empty() {
        let cut = rest
            .char_indices()
            .nth(width)
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        if cut >= rest.len() {
            lines.push(rest.to_string());
            return lines;
        }
        let head = &rest[..cut];
        let take = match head.rfind(' ') {
            Some(sp) if sp > width / 4 => sp,
            _ => cut,
        };
        lines.push(rest[..take].trim_end().to_string());
        rest = rest[take..].trim_start();
    }
    if !rest.is_empty() {
        if let Some(last) = lines.last_mut() {
            last.push('…');
        }
    }
    lines
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

/// Render the full profile card. Body first with a row cursor, then wrap in
/// the root `<svg>` element so the height is known before emission.
pub fn render(p: &Profile) -> String {
    let mut b = Builder { y: 16.0, body: String::new() };

    // header: avatar + identity (y=16..88)
    b.push(r#"  <clipPath id="av"><circle cx="44" cy="44" r="28"/></clipPath>"#.to_string());
    b.push(format!(
        r#"  <image href="{}" x="16" y="16" width="56" height="56" clip-path="url(#av)"/>"#,
        esc(&p.user.avatar_url)
    ));
    let display_name = p.user.name.as_deref().unwrap_or(&p.user.login);
    b.push(bold(84.0, 36.0, 18, TEXT, &esc(display_name)));
    if let Some(bio) = &p.user.bio {
        for (i, line) in wrap(bio, 52, 2).into_iter().enumerate() {
            b.push(text(84.0, 54.0 + i as f64 * 14.0, 11, MUTED, &esc(&line)));
        }
    }
    b.push(text(
        84.0,
        86.0,
        12,
        MUTED,
        &format!(
            "{} followers • {} following",
            fmt(p.user.followers),
            fmt(p.user.following)
        ),
    ));
    b.y += 88.0;

    // divider
    b.push(format!(
        r#"  <line x1="16" y1="{}" x2="464" y2="{}" stroke="{}"/>"#,
        b.y, b.y, DIVIDER
    ));
    b.y += 16.0;

    // stats row
    const XS: [f64; 4] = [16.0, 136.0, 256.0, 376.0];
    for (i, (label, value)) in [
        ("REPOS", p.user.repos_total),
        ("STARS", p.stars),
        ("FORKS", p.forks),
        ("OPEN ISSUES", p.open_issues),
    ]
    .into_iter()
    .enumerate()
    {
        b.push(text(XS[i], b.y, 10, MUTED, label));
        b.push(bold(XS[i], b.y + 20.0, 16, TEXT, &fmt(value)));
    }
    b.y += 48.0;

    // lines-changed chart
    b.push(text(16.0, b.y, 10, MUTED, "LINES CHANGED"));
    b.y += 14.0;

    let total_changed: u64 = p.weeks.iter().map(|w| w.added + w.deleted).sum();
    if p.weeks.len() < 2 || total_changed == 0 {
        b.push(text((X0 + X1) / 2.0, b.y + 50.0, 11, MUTED, "no activity data"));
    } else {
        let ymid = b.y + 50.0;
        let peak = p
            .weeks
            .iter()
            .map(|w| w.added.max(w.deleted))
            .max()
            .unwrap_or(1)
            .max(1);
        let k = (45.0 / peak as f64).max(1.0);
        let n = p.weeks.len();
        let xi = |i: usize| X0 + i as f64 * CHART_W / (n - 1) as f64;

        let green_pts: String = p
            .weeks
            .iter()
            .enumerate()
            .map(|(i, w)| format!("{:.2},{:.2}", xi(i), ymid - w.added as f64 * k))
            .collect::<Vec<_>>()
            .join(" ");
        let red_pts: String = p
            .weeks
            .iter()
            .enumerate()
            .map(|(i, w)| format!("{:.2},{:.2}", xi(i), ymid + w.deleted as f64 * k))
            .collect::<Vec<_>>()
            .join(" ");

        b.push(format!(
            r#"  <polygon points="{x:.2},{ymid:.2} {pts} {xn:.2},{ymid:.2}" fill="{fill}" opacity=".85"/>"#,
            x = xi(0),
            ymid = ymid,
            pts = green_pts,
            xn = xi(n - 1),
            fill = GREEN,
        ));
        b.push(format!(
            r#"  <polygon points="{x:.2},{ymid:.2} {pts} {xn:.2},{ymid:.2}" fill="{fill}" opacity=".85"/>"#,
            x = xi(0),
            ymid = ymid,
            pts = red_pts,
            xn = xi(n - 1),
            fill = RED,
        ));

        // corner date labels
        b.push(text(xi(0), ymid - 52.0, 10, MUTED, &p.weeks[0].date));
        b.push(text(
            (xi(n - 1) - 62.0).max(X0),
            ymid + 62.0,
            10,
            MUTED,
            &p.weeks[n - 1].date,
        ));
        b.push(text(
            16.0,
            b.y + 112.0,
            11,
            MUTED,
            &format!(
                "+{} −{} across {} weeks",
                fmt(p.weeks.iter().map(|w| w.added).sum()),
                fmt(p.weeks.iter().map(|w| w.deleted).sum()),
                n
            ),
        ));
    }
    b.y += 118.0;

    // languages
    b.push(text(16.0, b.y, 10, MUTED, "LANGUAGES"));
    b.y += 14.0;

    if p.languages.is_empty() {
        b.push(text((X0 + X1) / 2.0, b.y + 14.0, 11, MUTED, "no language data"));
        b.y += 18.0;
    } else {
        // stacked bar; last segment absorbs rounding remainder
        let total_pct: f64 = p.languages.iter().map(|l| l.pct).sum::<f64>().max(f64::MIN_POSITIVE);
        let mut bx = X0;
        for (i, lang) in p.languages.iter().enumerate() {
            let seg_w = if i == p.languages.len() - 1 {
                X1 - bx
            } else {
                ((CHART_W * lang.pct / total_pct).round() * 100.0) / 100.0
            };
            b.push(format!(
                r#"  <rect x="{bx:.2}" y="{y:.0}" width="{w:.2}" height="8" rx="4" fill="{fill}"/>"#,
                bx = bx,
                y = b.y,
                w = seg_w,
                fill = lang.color,
            ));
            bx += seg_w;
        }
        b.y += 20.0;

        // legend grid: 2 cols x up to 5 rows
        for (i, lang) in p.languages.iter().enumerate() {
            let col = i % 2;
            let row = i / 2;
            let lx = X0 + col as f64 * 224.0;
            let ly = b.y + 8.0 + row as f64 * 18.0;
            b.push(format!(
                r#"  <circle cx="{cx:.0}" cy="{cy:.2}" r="4" fill="{fill}"/>"#,
                cx = lx + 4.0,
                cy = ly - 4.0,
                fill = lang.color,
            ));
            b.push(text(lx + 14.0, ly, 11, TEXT, &esc(&lang.name)));
            b.push(format!(
                r#"  <text x="{x:.2}" y="{y:.2}" text-anchor="end" font-size="11" fill="{fill}">{pct:.1}%</text>"#,
                x = lx + 208.0,
                y = ly,
                fill = MUTED,
                pct = lang.pct,
            ));
        }
        let rows = (p.languages.len() + 1) / 2;
        b.y += rows as f64 * 18.0 + 4.0;
    }

    let h = b.y + 16.0;
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="480" height="{h}" viewBox="0 0 480 {h}">
<rect width="480" height="{h}" rx="6" fill="{bg}" stroke="{stroke}"/>
<g font-family="-apple-system,Segoe UI,Ubuntu,Sans-serif">
"#,
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
    use crate::model::{GhRepo, GhUser, LangStat, LineWeek, Profile};

    fn profile() -> Profile {
        Profile {
            user: GhUser {
                login: "jackra1n".into(),
                name: Some("Jack & <Friends>".into()),
                bio: Some("line one
and more".into()),
                avatar_url: "https://avatars.githubusercontent.com/u/1?v=4".into(),
                created_at: "2020-01-01T00:00:00Z".into(),
                followers: 1234,
                following: 42,
                repos_total: 2,
                repos: vec![GhRepo {
                    name: "r".into(),
                    stars: 7,
                    forks: 3,
                    open_issues: 1,
                    langs: vec![],
                }],
            },
            stars: 7,
            forks: 3,
            open_issues: 1,
            languages: vec![LangStat { name: "Rust".into(), size: 10, pct: 100.0, color: "#dea584" }],
            weeks: vec![
                LineWeek { date: "2025-09-28".into(), added: 10, deleted: 2 },
                LineWeek { date: "2025-10-05".into(), added: 20, deleted: 4 },
            ],
        }
    }

    #[test]
    fn fmt_boundaries() {
        assert_eq!(fmt(999), "999");
        assert_eq!(fmt(1234), "1.2k");
        assert_eq!(fmt(1500000), "1.5m");
        assert_eq!(fmt(1_000_000), "1m");
        assert_eq!(fmt(1000), "1k");
    }

    #[test]
    fn esc_escapes_all_metacharacters() {
        assert_eq!(esc("&<a>"), "&amp;&lt;a&gt;");
        assert_eq!(esc("\"'"), "&quot;&#39;");
    }

    #[test]
    fn wrap_normalizes_and_truncates() {
        assert_eq!(wrap("a\r\nb  c", 10, 2).join("|"), "a b c");
        let long = "word ".repeat(40);
        let lines = wrap(&long, 52, 2);
        assert_eq!(lines.len(), 2);
        assert!(lines[1].ends_with('…'));
    }

    #[test]
    fn render_shape_and_escaping() {
        let svg = render(&profile());
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("#0d1117"));
        assert!(svg.contains("Jack &amp; &lt;Friends&gt;"));
        // bio newline normalized away by wrap()
        assert!(svg.lines().filter(|l| l.contains("line one")).count() == 1);
        assert!(svg.ends_with("</svg>\n"));
    }

    #[test]
    fn render_deterministic() {
        assert_eq!(render(&profile()), render(&profile()));
    }
}
