use crate::commands::resolve_orgs;
use crate::config::load_config;
use crate::display;
use crate::error::Result;
use crate::github::GithubClient;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct OrgStats {
    pub total_repos: usize,
    pub total_stars: u32,
    pub total_forks: u32,
    pub total_open_issues: u32,
    pub languages: Vec<LanguageCount>,
    pub most_starred: Option<RepoRef>,
    pub most_forked: Option<RepoRef>,
}

#[derive(Debug, Serialize)]
pub struct LanguageCount {
    pub language: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct RepoRef {
    pub org: String,
    pub name: String,
    pub count: u32,
}

pub async fn run(org: &Option<String>, json: bool, verbose: bool) -> Result<()> {
    let config = load_config()?;
    let token = config.token()?;
    let client = GithubClient::new(token, verbose)?;

    let orgs = resolve_orgs(org, &config, &client).await?;

    let mut total_repos = 0usize;
    let mut total_stars = 0u32;
    let mut total_forks = 0u32;
    let mut total_open_issues = 0u32;
    let mut lang_map: HashMap<String, usize> = HashMap::new();
    let mut most_starred: Option<RepoRef> = None;
    let mut most_forked: Option<RepoRef> = None;

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
            let forks = repo.forks_count.unwrap_or(0);
            total_stars += stars;
            total_forks += forks;
            total_open_issues += repo.open_issues_count.unwrap_or(0);

            let language = repo
                .language
                .as_ref()
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            *lang_map.entry(language).or_insert(0) += 1;

            if most_starred.as_ref().is_none_or(|r| stars > r.count) && stars > 0 {
                most_starred = Some(RepoRef {
                    org: org_name.clone(),
                    name: repo.name.clone(),
                    count: stars,
                });
            }
            if most_forked.as_ref().is_none_or(|r| forks > r.count) && forks > 0 {
                most_forked = Some(RepoRef {
                    org: org_name.clone(),
                    name: repo.name.clone(),
                    count: forks,
                });
            }
        }
    }

    let mut languages: Vec<LanguageCount> = lang_map
        .into_iter()
        .map(|(language, count)| LanguageCount { language, count })
        .collect();
    languages.sort_by(|a, b| b.count.cmp(&a.count));

    let stats = OrgStats {
        total_repos,
        total_stars,
        total_forks,
        total_open_issues,
        languages,
        most_starred,
        most_forked,
    };

    display::output(json, &stats, |data| {
        render_stats(data);
    });

    client.check_rate_limit_if_verbose().await;

    Ok(())
}

fn render_stats(stats: &OrgStats) {
    display::section_header("Organization Statistics");

    println!("  {} {}", "Repositories:".bold(), stats.total_repos);
    println!("  {} {}", "Total Stars:".bold(), stats.total_stars);
    println!("  {} {}", "Total Forks:".bold(), stats.total_forks);
    println!("  {} {}", "Open Issues:".bold(), stats.total_open_issues);

    if let Some(ref r) = stats.most_starred {
        println!(
            "  {} {}/{} ({})",
            "Most Starred:".bold(),
            r.org,
            r.name,
            r.count
        );
    }

    if let Some(ref r) = stats.most_forked {
        println!(
            "  {} {}/{} ({})",
            "Most Forked:".bold(),
            r.org,
            r.name,
            r.count
        );
    }

    if !stats.languages.is_empty() {
        println!("\n  {}", "Top Languages:".bold());
        for (i, lang) in stats.languages.iter().take(10).enumerate() {
            println!("    {}. {} ({})", i + 1, lang.language, lang.count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_sorting_by_count_descending() {
        let mut langs = vec![
            LanguageCount {
                language: "Go".into(),
                count: 3,
            },
            LanguageCount {
                language: "Rust".into(),
                count: 10,
            },
            LanguageCount {
                language: "Python".into(),
                count: 7,
            },
        ];
        langs.sort_by(|a, b| b.count.cmp(&a.count));
        assert_eq!(langs[0].language, "Rust");
        assert_eq!(langs[1].language, "Python");
        assert_eq!(langs[2].language, "Go");
    }

    #[test]
    fn language_aggregation() {
        let mut lang_map: HashMap<String, usize> = HashMap::new();
        let languages = ["Rust", "Go", "Rust", "Python", "Rust", "Go"];
        for lang in languages {
            *lang_map.entry(lang.to_string()).or_insert(0) += 1;
        }
        assert_eq!(lang_map["Rust"], 3);
        assert_eq!(lang_map["Go"], 2);
        assert_eq!(lang_map["Python"], 1);
    }

    #[test]
    fn stats_serialization() {
        let stats = OrgStats {
            total_repos: 5,
            total_stars: 100,
            total_forks: 20,
            total_open_issues: 10,
            languages: vec![LanguageCount {
                language: "Rust".into(),
                count: 3,
            }],
            most_starred: Some(RepoRef {
                org: "myorg".into(),
                name: "best-repo".into(),
                count: 50,
            }),
            most_forked: None,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_repos\":5"));
        assert!(json.contains("\"best-repo\""));
    }
}
