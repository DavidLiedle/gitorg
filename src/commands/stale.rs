use crate::commands::resolve_orgs;
use crate::config::load_config;
use crate::display;
use crate::error::Result;
use crate::github::GithubClient;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StaleRepo {
    pub org: String,
    pub name: String,
    pub last_push: String,
    pub days_stale: i64,
    pub stars: u32,
    pub language: String,
}

pub async fn run(org: &Option<String>, days: u64, json: bool, verbose: bool) -> Result<()> {
    let config = load_config()?;
    let token = config.token()?;
    let client = GithubClient::new(token, verbose)?;

    let orgs = resolve_orgs(org, &config, &client).await?;
    let now = Utc::now();
    let threshold = days as i64;

    let mut stale_repos = Vec::new();

    for org_name in &orgs {
        match client.list_org_repos(org_name).await {
            Ok(repos) => {
                for repo in &repos {
                    if repo.archived.unwrap_or(false) {
                        continue;
                    }

                    let days_since = repo
                        .pushed_at
                        .map(|dt| (now - dt).num_days())
                        .unwrap_or(99999);

                    if days_since >= threshold {
                        let language = repo
                            .language
                            .as_ref()
                            .and_then(|v| v.as_str())
                            .unwrap_or("-")
                            .to_string();

                        stale_repos.push(StaleRepo {
                            org: org_name.clone(),
                            name: repo.name.clone(),
                            last_push: repo
                                .pushed_at
                                .map(|dt| dt.format("%Y-%m-%d").to_string())
                                .unwrap_or_else(|| "never".to_string()),
                            days_stale: days_since,
                            stars: repo.stargazers_count.unwrap_or(0),
                            language,
                        });
                    }
                }
            }
            Err(e) => {
                display::warn(&format!("Failed to fetch repos for {org_name}: {e}"));
            }
        }
    }

    stale_repos.sort_by(|a, b| b.days_stale.cmp(&a.days_stale));

    display::output(json, &stale_repos, |data| {
        render_stale_repos(data, days);
    });

    client.check_rate_limit_if_verbose().await;

    Ok(())
}

fn render_stale_repos(repos: &[StaleRepo], days: u64) {
    if repos.is_empty() {
        display::success(&format!("No repositories stale for more than {days} days."));
        return;
    }

    display::section_header(&format!("Stale Repositories (>{days} days)"));

    let mut table = display::new_table(&[
        "Org",
        "Name",
        "Last Push",
        "Days Stale",
        "Stars",
        "Language",
    ]);

    for r in repos {
        table.add_row(vec![
            &r.org,
            &r.name,
            &r.last_push,
            &r.days_stale.to_string(),
            &r.stars.to_string(),
            &r.language,
        ]);
    }

    println!("{table}");
    println!("\n{} stale repository(ies) found.", repos.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_filtering_by_threshold() {
        let repos = vec![
            StaleRepo {
                org: "org".into(),
                name: "very-stale".into(),
                last_push: "2020-01-01".into(),
                days_stale: 1500,
                stars: 0,
                language: "Rust".into(),
            },
            StaleRepo {
                org: "org".into(),
                name: "barely-stale".into(),
                last_push: "2024-01-01".into(),
                days_stale: 100,
                stars: 5,
                language: "Go".into(),
            },
        ];

        // Both are stale at threshold 90
        let filtered: Vec<&StaleRepo> = repos.iter().filter(|r| r.days_stale >= 90).collect();
        assert_eq!(filtered.len(), 2);

        // Only one at threshold 200
        let filtered: Vec<&StaleRepo> = repos.iter().filter(|r| r.days_stale >= 200).collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "very-stale");
    }

    #[test]
    fn stale_sorting_most_stale_first() {
        let mut repos = vec![
            StaleRepo {
                org: "org".into(),
                name: "less-stale".into(),
                last_push: "2024-01-01".into(),
                days_stale: 100,
                stars: 0,
                language: "-".into(),
            },
            StaleRepo {
                org: "org".into(),
                name: "more-stale".into(),
                last_push: "2020-01-01".into(),
                days_stale: 1500,
                stars: 0,
                language: "-".into(),
            },
        ];

        repos.sort_by(|a, b| b.days_stale.cmp(&a.days_stale));
        assert_eq!(repos[0].name, "more-stale");
        assert_eq!(repos[1].name, "less-stale");
    }
}
