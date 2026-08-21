mod cli;
mod gh;
mod langs;
mod model;
mod render;
use crate::cli::Args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse(std::env::args().collect())?;
    let profile = run(&args)?;
    std::fs::write(&args.output, render::render(&profile))?;
    Ok(())
}

fn run(args: &Args) -> Result<model::Profile, Box<dyn std::error::Error>> {
    let agent = gh::agent();
    let user = gh::fetch_profile(&agent, &args.token, &args.username)?;
    eprintln!("lines: aggregating weekly stats for {} repos", user.repos.len());
    let weeks = gh::collect_weeks(&agent, &args.token, &user);
    let languages = langs::top_languages(&user.repos);
    let stars = user.repos.iter().map(|r| r.stars).sum();
    let forks = user.repos.iter().map(|r| r.forks).sum();
    let open_issues = user.repos.iter().map(|r| r.open_issues).sum();
    Ok(model::Profile { user, stars, forks, open_issues, languages, weeks })
}
