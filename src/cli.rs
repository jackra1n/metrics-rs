pub struct Args {
    pub username: String,
    pub output: String,
    pub token: String,
}

/// Hand-rolled parsing: `metrics-rs <USERNAME> <OUTPUT> [--token <TOKEN>]`.
/// Token precedence: --token > GITHUB_TOKEN > GH_TOKEN > usage error (exit 2).
pub fn parse(argv: Vec<String>) -> Result<Args, String> {
    let mut username = None;
    let mut output = None;
    let mut token = None;
    let mut it = argv.into_iter().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--token" => {
                token = Some(
                    it.next()
                        .ok_or_else(|| "missing value for --token".to_string())?,
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
    })
}
