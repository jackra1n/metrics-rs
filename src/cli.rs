pub struct Args {
    pub username: String,
    pub output: String,
    pub token: String,
    /// --indepth: analyze authored commits via bare git clones instead of
    /// GitHub's precomputed repo language totals
    pub indepth: bool,
    /// --ignore-languages: languages excluded from in-depth analysis
    /// (lowercased at parse time)
    pub ignored_languages: Vec<String>,
}

/// Hand-rolled parsing: `metrics-rs <USERNAME> <OUTPUT> [--token <TOKEN>]
/// [--indepth] [--ignore-languages <csv>]`.
/// Token precedence: --token > GITHUB_TOKEN > GH_TOKEN > usage error (exit 2).
pub fn parse(argv: Vec<String>) -> Result<Args, String> {
    let mut username = None;
    let mut output = None;
    let mut token = None;
    let mut indepth = false;
    let mut ignored_languages = Vec::new();
    let mut it = argv.into_iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--token" => {
                token = Some(
                    it.next()
                        .ok_or_else(|| "missing value for --token".to_string())?,
                );
            }
            "--indepth" => indepth = true,
            "--ignore-languages" | "--ignored-languages" => {
                let csv = it
                    .next()
                    .ok_or_else(|| "missing value for --ignore-languages".to_string())?;
                ignored_languages.extend(
                    csv.split(',')
                        .map(|s| s.trim().to_ascii_lowercase())
                        .filter(|s| !s.is_empty()),
                );
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown flag: {other}"));
            }
            other if username.is_none() => username = Some(other.to_string()),
            other if output.is_none() => output = Some(other.to_string()),
            other => return Err(format!("unexpected argument: {other}")),
        }
    }
    let username = username.ok_or_else(|| "usage: metrics-rs <USERNAME> <OUTPUT>".to_string())?;
    let output = output.ok_or_else(|| "usage: metrics-rs <USERNAME> <OUTPUT>".to_string())?;
    let token = token
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .or_else(|| std::env::var("GH_TOKEN").ok())
        .ok_or_else(|| "missing token: pass --token or set GITHUB_TOKEN".to_string())?;
    Ok(Args {
        username,
        output,
        token,
        indepth,
        ignored_languages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(argv: &[&str]) -> Result<Args, String> {
        parse(argv.iter().map(|s| s.to_string()).collect())
    }

    #[test]
    fn indepth_flag_sets_mode() {
        let a = args(&[
            "metrics-rs",
            "jackra1n",
            "out.svg",
            "--indepth",
            "--token",
            "t",
        ])
        .unwrap();
        assert!(a.indepth);
        assert!(a.ignored_languages.is_empty());
    }

    #[test]
    fn indepth_defaults_off() {
        let a = args(&["metrics-rs", "jackra1n", "out.svg", "--token", "t"]).unwrap();
        assert!(!a.indepth);
    }

    #[test]
    fn ignore_languages_splits_trims_lowercases() {
        let a = args(&[
            "metrics-rs",
            "jackra1n",
            "out.svg",
            "--ignore-languages",
            "swift, gdscript",
            "--token",
            "t",
        ])
        .unwrap();
        assert_eq!(a.ignored_languages, vec!["swift", "gdscript"]);
    }

    #[test]
    fn ignored_languages_alias_works() {
        let a = args(&[
            "metrics-rs",
            "jackra1n",
            "out.svg",
            "--ignored-languages",
            "C++,Rust",
            "--token",
            "t",
        ])
        .unwrap();
        assert_eq!(a.ignored_languages, vec!["c++", "rust"]);
    }

    #[test]
    fn ignore_languages_empty_csv_is_fine() {
        let a = args(&[
            "metrics-rs",
            "jackra1n",
            "out.svg",
            "--ignore-languages",
            " , ",
            "--token",
            "t",
        ])
        .unwrap();
        assert!(a.ignored_languages.is_empty());
    }

    #[test]
    fn indepth_still_requires_positionals() {
        assert!(args(&["metrics-rs", "--indepth", "--token", "t"]).is_err());
        assert!(args(&["metrics-rs", "jackra1n", "--indepth", "--token", "t"]).is_err());
    }

    #[test]
    fn missing_ignore_languages_value_errors() {
        assert!(args(&["metrics-rs", "jackra1n", "out.svg", "--ignore-languages"]).is_err());
    }
}
