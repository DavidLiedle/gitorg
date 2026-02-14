pub mod auth;
pub mod issues;
pub mod orgs;
pub mod overview;
pub mod repos;
pub mod stale;
pub mod stats;

use crate::config::Config;
use crate::github::GithubClient;

pub async fn resolve_orgs(
    org_flag: &Option<String>,
    config: &Config,
    client: &GithubClient,
) -> crate::error::Result<Vec<String>> {
    if let Some(org) = org_flag {
        return Ok(vec![org.clone()]);
    }

    if let Some(ref orgs) = config.defaults.orgs {
        if !orgs.is_empty() {
            return Ok(orgs.clone());
        }
    }

    let orgs = client.list_user_orgs().await?;
    let names: Vec<String> = orgs.into_iter().map(|o| o.login).collect();
    Ok(names)
}
