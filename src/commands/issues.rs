use crate::commands::resolve_orgs;
use crate::config::load_config;
use crate::display;
use crate::error::Result;
use crate::github::GithubClient;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct IssueSummary {
    pub org: String,
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub author: String,
    pub labels: String,
    pub updated: String,
}

pub async fn run(org: &Option<String>, json: bool, verbose: bool) -> Result<()> {
    let config = load_config()?;
    let token = config.token()?;
    let client = GithubClient::new(token, verbose)?;

    client.warn_if_rate_limited().await.ok();

    let orgs = resolve_orgs(org, &config, &client).await?;

    let mut all_issues = Vec::new();

    for org_name in &orgs {
        let repos = match client.list_org_repos(org_name).await {
            Ok(r) => r,
            Err(e) => {
                display::warn(&format!("Failed to fetch repos for {org_name}: {e}"));
                continue;
            }
        };

        for repo in &repos {
            if repo.archived.unwrap_or(false) {
                continue;
            }
            if repo.open_issues_count.unwrap_or(0) == 0 {
                continue;
            }

            let issues = match client.list_repo_issues(org_name, &repo.name).await {
                Ok(i) => i,
                Err(e) => {
                    display::warn(&format!(
                        "Failed to fetch issues for {}/{}: {e}",
                        org_name, repo.name
                    ));
                    continue;
                }
            };

            for issue in &issues {
                // Filter out pull requests
                if issue.pull_request.is_some() {
                    continue;
                }

                let labels: Vec<String> = issue.labels.iter().map(|l| l.name.clone()).collect();

                all_issues.push(IssueSummary {
                    org: org_name.clone(),
                    repo: repo.name.clone(),
                    number: issue.number,
                    title: issue.title.clone(),
                    author: issue.user.login.clone(),
                    labels: if labels.is_empty() {
                        "-".to_string()
                    } else {
                        labels.join(", ")
                    },
                    updated: issue.updated_at.format("%Y-%m-%d").to_string(),
                });
            }
        }
    }

    display::output(json, &all_issues, |data| {
        render_issues_table(data);
    });

    client.check_rate_limit_if_verbose().await;

    Ok(())
}

fn render_issues_table(issues: &[IssueSummary]) {
    if issues.is_empty() {
        display::success("No open issues found.");
        return;
    }

    display::section_header("Open Issues");

    let mut table =
        display::new_table(&["Org", "Repo", "#", "Title", "Author", "Labels", "Updated"]);

    for i in issues {
        table.add_row(vec![
            &i.org,
            &i.repo,
            &i.number.to_string(),
            &i.title,
            &i.author,
            &i.labels,
            &i.updated,
        ]);
    }

    println!("{table}");
    println!("\n{} open issue(s) found.", issues.len());
}
