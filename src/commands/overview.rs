use crate::commands::resolve_orgs;
use crate::config::load_config;
use crate::display;
use crate::error::Result;
use crate::github::GithubClient;
use chrono::Utc;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct OverviewData {
    pub total_repos: usize,
    pub total_stars: u32,
    pub total_forks: u32,
    pub total_open_issues: u32,
    pub top_languages: Vec<LangEntry>,
    pub recently_active: Vec<RepoEntry>,
    pub stale_repos: Vec<RepoEntry>,
    pub recent_issues: Vec<IssueEntry>,
}

#[derive(Debug, Serialize)]
pub struct LangEntry {
    pub language: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct RepoEntry {
    pub org: String,
    pub name: String,
    pub stars: u32,
    pub last_push: String,
    pub days_since_push: i64,
}

#[derive(Debug, Serialize)]
pub struct IssueEntry {
    pub org: String,
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub updated: String,
}

pub async fn run(org: &Option<String>, days: u64, json: bool, verbose: bool) -> Result<()> {
    let config = load_config()?;
    let token = config.token()?;
    let client = GithubClient::new(token, verbose)?;

    client.warn_if_rate_limited().await.ok();

    let orgs = resolve_orgs(org, &config, &client).await?;
    let now = Utc::now();

    let mut total_repos = 0usize;
    let mut total_stars = 0u32;
    let mut total_forks = 0u32;
    let mut total_open_issues = 0u32;
    let mut lang_map: HashMap<String, usize> = HashMap::new();
    let mut all_repo_entries = Vec::new();
    let mut recent_issues = Vec::new();

    for org_name in &orgs {
        let repos = match client.list_org_repos(org_name).await {
            Ok(r) => r,
            Err(e) => {
                display::warn(&format!("Failed to fetch repos for {org_name}: {e}"));
                continue;
            }
        };

        for repo in &repos {
            total_repos += 1;
            let stars = repo.stargazers_count.unwrap_or(0);
            total_stars += stars;
            total_forks += repo.forks_count.unwrap_or(0);
            total_open_issues += repo.open_issues_count.unwrap_or(0);

            let language = repo
                .language
                .as_ref()
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();
            *lang_map.entry(language).or_insert(0) += 1;

            let days_since = repo
                .pushed_at
                .map(|dt| (now - dt).num_days())
                .unwrap_or(99999);

            all_repo_entries.push(RepoEntry {
                org: org_name.clone(),
                name: repo.name.clone(),
                stars,
                last_push: repo
                    .pushed_at
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "never".to_string()),
                days_since_push: days_since,
            });

            // Fetch issues for repos that have them and aren't archived
            if !repo.archived.unwrap_or(false) && repo.open_issues_count.unwrap_or(0) > 0 {
                if let Ok(issues) = client.list_repo_issues(org_name, &repo.name).await {
                    for issue in issues.into_iter().take(3) {
                        if issue.pull_request.is_some() {
                            continue;
                        }
                        recent_issues.push(IssueEntry {
                            org: org_name.clone(),
                            repo: repo.name.clone(),
                            number: issue.number,
                            title: issue.title,
                            updated: issue.updated_at.format("%Y-%m-%d").to_string(),
                        });
                    }
                }
            }
        }
    }

    // Sort and limit
    all_repo_entries.sort_by(|a, b| a.days_since_push.cmp(&b.days_since_push));
    let recently_active: Vec<RepoEntry> = all_repo_entries
        .iter()
        .filter(|r| r.days_since_push < days as i64)
        .take(10)
        .map(|r| RepoEntry {
            org: r.org.clone(),
            name: r.name.clone(),
            stars: r.stars,
            last_push: r.last_push.clone(),
            days_since_push: r.days_since_push,
        })
        .collect();

    let stale_repos: Vec<RepoEntry> = all_repo_entries
        .iter()
        .rev()
        .filter(|r| r.days_since_push >= days as i64)
        .take(10)
        .map(|r| RepoEntry {
            org: r.org.clone(),
            name: r.name.clone(),
            stars: r.stars,
            last_push: r.last_push.clone(),
            days_since_push: r.days_since_push,
        })
        .collect();

    recent_issues.sort_by(|a, b| b.updated.cmp(&a.updated));
    recent_issues.truncate(10);

    let mut top_languages: Vec<LangEntry> = lang_map
        .into_iter()
        .map(|(language, count)| LangEntry { language, count })
        .collect();
    top_languages.sort_by(|a, b| b.count.cmp(&a.count));
    top_languages.truncate(5);

    let overview = OverviewData {
        total_repos,
        total_stars,
        total_forks,
        total_open_issues,
        top_languages,
        recently_active,
        stale_repos,
        recent_issues,
    };

    display::output(json, &overview, |data| {
        render_overview(data);
    });

    client.check_rate_limit_if_verbose().await;

    Ok(())
}

fn render_overview(data: &OverviewData) {
    // Summary
    display::section_header("Summary");
    println!(
        "  {} {}   {} {}   {} {}   {} {}",
        "Repos:".bold(),
        data.total_repos,
        "Stars:".bold(),
        data.total_stars,
        "Forks:".bold(),
        data.total_forks,
        "Issues:".bold(),
        data.total_open_issues,
    );

    // Top Languages
    if !data.top_languages.is_empty() {
        display::section_header("Top Languages");
        for lang in &data.top_languages {
            println!("  {} ({})", lang.language, lang.count);
        }
    }

    // Recently Active Repos
    if !data.recently_active.is_empty() {
        display::section_header("Recently Active Repos");
        let mut table = display::new_table(&["Org", "Name", "Stars", "Last Push"]);
        for r in &data.recently_active {
            table.add_row(vec![&r.org, &r.name, &r.stars.to_string(), &r.last_push]);
        }
        println!("{table}");
    }

    // Stale Repos
    if !data.stale_repos.is_empty() {
        display::section_header("Stale Repos");
        let mut table = display::new_table(&["Org", "Name", "Stars", "Last Push", "Days Stale"]);
        for r in &data.stale_repos {
            table.add_row(vec![
                &r.org,
                &r.name,
                &r.stars.to_string(),
                &r.last_push,
                &r.days_since_push.to_string(),
            ]);
        }
        println!("{table}");
    }

    // Recent Issues
    if !data.recent_issues.is_empty() {
        display::section_header("Recent Issues");
        let mut table = display::new_table(&["Org", "Repo", "#", "Title", "Updated"]);
        for i in &data.recent_issues {
            table.add_row(vec![
                &i.org,
                &i.repo,
                &i.number.to_string(),
                &i.title,
                &i.updated,
            ]);
        }
        println!("{table}");
    }
}
