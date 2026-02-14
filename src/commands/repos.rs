use crate::commands::resolve_orgs;
use crate::config::load_config;
use crate::display;
use crate::error::Result;
use crate::github::GithubClient;
use chrono::Utc;
use octocrab::models::Repository;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RepoSummary {
    pub org: String,
    pub name: String,
    pub language: String,
    pub stars: u32,
    pub forks: u32,
    pub open_issues: u32,
    pub last_push: String,
    pub status: String,
}

impl RepoSummary {
    pub fn from_repo(org: &str, repo: &Repository) -> Self {
        let language = repo
            .language
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("-")
            .to_string();

        let pushed_at = repo.pushed_at;
        let last_push = pushed_at
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "never".to_string());

        let status = if repo.archived.unwrap_or(false) {
            "archived".to_string()
        } else {
            let days = pushed_at
                .map(|dt| (Utc::now() - dt).num_days())
                .unwrap_or(999);
            if days > 365 {
                "stale".to_string()
            } else {
                "active".to_string()
            }
        };

        Self {
            org: org.to_string(),
            name: repo.name.clone(),
            language,
            stars: repo.stargazers_count.unwrap_or(0),
            forks: repo.forks_count.unwrap_or(0),
            open_issues: repo.open_issues_count.unwrap_or(0),
            last_push,
            status,
        }
    }
}

pub async fn run(org: &Option<String>, sort: &str, json: bool, verbose: bool) -> Result<()> {
    let config = load_config()?;
    let token = config.token()?;
    let client = GithubClient::new(token, verbose)?;

    let orgs = resolve_orgs(org, &config, &client).await?;

    let mut summaries = Vec::new();
    for org_name in &orgs {
        match client.list_org_repos(org_name).await {
            Ok(repos) => {
                for repo in &repos {
                    summaries.push(RepoSummary::from_repo(org_name, repo));
                }
            }
            Err(e) => {
                display::warn(&format!("Failed to fetch repos for {org_name}: {e}"));
            }
        }
    }

    sort_repos(&mut summaries, sort);

    display::output(json, &summaries, |data| {
        render_repos_table(data);
    });

    client.check_rate_limit_if_verbose().await;

    Ok(())
}

fn sort_repos(repos: &mut [RepoSummary], sort: &str) {
    match sort {
        "stars" => repos.sort_by(|a, b| b.stars.cmp(&a.stars)),
        "name" => repos.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        "staleness" => repos.sort_by(|a, b| a.last_push.cmp(&b.last_push)),
        _ => repos.sort_by(|a, b| b.last_push.cmp(&a.last_push)), // activity (most recent first)
    }
}

fn render_repos_table(repos: &[RepoSummary]) {
    if repos.is_empty() {
        display::warn("No repositories found.");
        return;
    }

    display::section_header("Repositories");

    let mut table = display::new_table(&[
        "Org",
        "Name",
        "Language",
        "Stars",
        "Forks",
        "Issues",
        "Last Push",
        "Status",
    ]);

    for r in repos {
        table.add_row(vec![
            &r.org,
            &r.name,
            &r.language,
            &r.stars.to_string(),
            &r.forks.to_string(),
            &r.open_issues.to_string(),
            &r.last_push,
            &r.status,
        ]);
    }

    println!("{table}");
    println!("\n{} repository(ies) found.", repos.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_repo(name: &str, stars: u32, last_push: &str) -> RepoSummary {
        RepoSummary {
            org: "test-org".to_string(),
            name: name.to_string(),
            language: "Rust".to_string(),
            stars,
            forks: 0,
            open_issues: 0,
            last_push: last_push.to_string(),
            status: "active".to_string(),
        }
    }

    #[test]
    fn sort_by_stars_descending() {
        let mut repos = vec![
            make_repo("low", 5, "2024-01-01"),
            make_repo("high", 100, "2024-01-01"),
            make_repo("mid", 50, "2024-01-01"),
        ];
        sort_repos(&mut repos, "stars");
        assert_eq!(repos[0].name, "high");
        assert_eq!(repos[1].name, "mid");
        assert_eq!(repos[2].name, "low");
    }

    #[test]
    fn sort_by_name_case_insensitive() {
        let mut repos = vec![
            make_repo("Zebra", 0, "2024-01-01"),
            make_repo("alpha", 0, "2024-01-01"),
            make_repo("Beta", 0, "2024-01-01"),
        ];
        sort_repos(&mut repos, "name");
        assert_eq!(repos[0].name, "alpha");
        assert_eq!(repos[1].name, "Beta");
        assert_eq!(repos[2].name, "Zebra");
    }

    #[test]
    fn sort_by_activity_most_recent_first() {
        let mut repos = vec![
            make_repo("old", 0, "2023-01-01"),
            make_repo("new", 0, "2024-06-01"),
            make_repo("mid", 0, "2024-01-01"),
        ];
        sort_repos(&mut repos, "activity");
        assert_eq!(repos[0].name, "new");
        assert_eq!(repos[1].name, "mid");
        assert_eq!(repos[2].name, "old");
    }

    #[test]
    fn sort_by_staleness_oldest_first() {
        let mut repos = vec![
            make_repo("new", 0, "2024-06-01"),
            make_repo("old", 0, "2023-01-01"),
            make_repo("mid", 0, "2024-01-01"),
        ];
        sort_repos(&mut repos, "staleness");
        assert_eq!(repos[0].name, "old");
        assert_eq!(repos[1].name, "mid");
        assert_eq!(repos[2].name, "new");
    }
}
