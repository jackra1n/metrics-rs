# metrics-rs

A fast, lightweight, dependency-minimal GitHub profile metrics card generator written in Rust.

Designed as an efficient native alternative to heavy JavaScript/Chromium-based profile metric tools.

![GitHub metrics preview](./docs/images/metrics.svg)

---

## Features

- **Blazing Fast**: Single native compiled binary. Generates deterministic SVG cards in milliseconds (~30 seconds for in-depth commit analysis across 60+ repositories in parallel).
- **Pure Vector SVG**: Emits clean, dark-theme SVG without spinning up Puppeteer, Headless Chrome, Docker, or Node.js.
- **Two Analysis Modes**:
  - **Fast API Mode** *(default)*: Instant calculation from GitHub GraphQL and REST endpoints.
  - **In-Depth Mode** (`--indepth`): Concurrent bare-clone analysis that inspects authored commit diffs to attribute exact lines added per language.
- **14-Day Contribution Heatmap**: Mini GitHub-style activity grid in the header, paired with lifetime contributed repository counts.
- **GitHub Linguist Palette**: Bundled language colors and file classification matching GitHub Linguist, normalized across the top 8 languages.
- **Balanced Stat Grid**: Key stats (license, stars, watchers, forks, sponsors, releases, storage, and colored `+`/`−` lines) formatted for dark backgrounds.
---

## Installation

### Prerequisites
- [Rust](https://www.rust-lang.org/) (2024 edition / 1.85+)
- Git CLI (for `--indepth` mode)

### Build & Install
```bash
git clone https://github.com/jackra1n/metrics-rs.git
cd metrics-rs
cargo install --path .
```

---

## Usage

```bash
# Basic usage (reads token from GITHUB_TOKEN or GH_TOKEN)
export GITHUB_TOKEN="ghp_your_token_here"
metrics-rs <USERNAME> <OUTPUT_SVG_PATH>

# In-depth authored commit analysis
metrics-rs <USERNAME> <OUTPUT_SVG_PATH> --indepth

# In-depth mode with excluded languages
metrics-rs <USERNAME> <OUTPUT_SVG_PATH> --indepth --ignore-languages swift,gdscript

# Passing token explicitly via flag
metrics-rs <USERNAME> <OUTPUT_SVG_PATH> --token "ghp_..."
```
### CLI Options

| Argument / Flag | Description |
| :--- | :--- |
| `<USERNAME>` | Target GitHub username |
| `<OUTPUT>` | Destination path for the output `.svg` file |
| `--indepth` | Enable bare-clone Git log analysis for author-specific commit diffs |
| `--ignore-languages <csv>` | Comma-separated list of languages to exclude from ranking |
| `--token <TOKEN>` | GitHub Personal Access Token (defaults to `$GITHUB_TOKEN` or `$GH_TOKEN`) |

---

## GitHub Actions Usage

You can use `metrics-rs` directly in your GitHub profile workflow with `uses: jackra1n/metrics-rs@master`:

```yaml
name: Generate Metrics Card

on:
  schedule:
    - cron: "0 0 * * *" # Daily at midnight
  workflow_dispatch:
  push:
    branches: [main]

jobs:
  metrics:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Generate metrics SVG
        uses: jackra1n/metrics-rs@master
        with:
          token: ${{ secrets.METRICS_TOKEN || secrets.GITHUB_TOKEN }}
          filename: github-metrics.svg
          indepth: yes
          ignore_languages: swift,gdscript

      - name: Commit and push changes
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add github-metrics.svg
          git diff --quiet && git diff --staged --quiet || (git commit -m "chore: update metrics card" && git push)
```

### Action Inputs

| Input | Description | Default |
| :--- | :--- | :--- |
| `token` | GitHub Personal Access Token or `GITHUB_TOKEN` | `${{ github.token }}` |
| `user` | Target username | `${{ github.repository_owner }}` |
| `filename` | Destination path for the `.svg` | `github-metrics.svg` |
| `indepth` | Run in-depth authored commit analysis (`yes` / `no`) | `no` |
| `ignore_languages` | Comma-separated languages to exclude | `""` |

