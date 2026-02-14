use crate::error::{GitorgError, Result};
use octocrab::models::issues::Issue;
use octocrab::models::Repository;
use octocrab::Octocrab;
use serde::Deserialize;

pub struct GithubClient {
    octocrab: Octocrab,
    verbose: bool,
}

#[derive(Debug, Deserialize)]
pub struct OrgInfo {
    pub login: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticatedUser {
    pub login: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RateLimit {
    pub resources: RateLimitResources,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitResources {
    pub core: RateLimitResource,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitResource {
    pub limit: u64,
    pub remaining: u64,
    pub reset: i64,
}

impl GithubClient {
    pub fn new(token: &str, verbose: bool) -> Result<Self> {
        let octocrab = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| GitorgError::GitHub(e.to_string()))?;
        Ok(Self { octocrab, verbose })
    }

    pub async fn validate_token(&self) -> Result<AuthenticatedUser> {
        let user: AuthenticatedUser = self
            .octocrab
            .get("/user", None::<&()>)
            .await
            .map_err(|e| GitorgError::GitHub(format!("Token validation failed: {e}")))?;
        Ok(user)
    }

    pub async fn get_rate_limit(&self) -> Result<RateLimit> {
        let rate_limit: RateLimit = self.octocrab.get("/rate_limit", None::<&()>).await?;
        Ok(rate_limit)
    }

    pub async fn check_rate_limit_if_verbose(&self) {
        if !self.verbose {
            return;
        }
        match self.get_rate_limit().await {
            Ok(rl) => {
                let core = &rl.resources.core;
                eprintln!(
                    "Rate limit: {}/{} remaining (resets at {})",
                    core.remaining,
                    core.limit,
                    chrono::DateTime::from_timestamp(core.reset, 0)
                        .map(|dt| dt.format("%H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| core.reset.to_string())
                );
            }
            Err(e) => eprintln!("Could not check rate limit: {e}"),
        }
    }

    pub async fn warn_if_rate_limited(&self) -> Result<()> {
        let rl = self.get_rate_limit().await?;
        if rl.resources.core.remaining < 100 {
            crate::display::warn(&format!(
                "Only {} API calls remaining (resets at {})",
                rl.resources.core.remaining,
                chrono::DateTime::from_timestamp(rl.resources.core.reset, 0)
                    .map(|dt| dt.format("%H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| rl.resources.core.reset.to_string())
            ));
        }
        Ok(())
    }

    pub async fn list_org_repos(&self, org: &str) -> Result<Vec<Repository>> {
        let mut all_repos = Vec::new();
        let mut page = 1u32;
        loop {
            let page_result = self
                .octocrab
                .orgs(org)
                .list_repos()
                .repo_type(octocrab::params::repos::Type::All)
                .per_page(100)
                .page(page)
                .send()
                .await?;

            let items = page_result.items;
            if items.is_empty() {
                break;
            }
            all_repos.extend(items);
            if page_result.next.is_none() {
                break;
            }
            page += 1;
        }
        Ok(all_repos)
    }

    pub async fn list_repo_issues(&self, owner: &str, repo: &str) -> Result<Vec<Issue>> {
        let mut all_issues = Vec::new();
        let mut page = 1u32;
        loop {
            let page_result = self
                .octocrab
                .issues(owner, repo)
                .list()
                .state(octocrab::params::State::Open)
                .per_page(100)
                .page(page)
                .send()
                .await?;

            let items = page_result.items;
            if items.is_empty() {
                break;
            }
            all_issues.extend(items);
            if page_result.next.is_none() {
                break;
            }
            page += 1;
        }
        Ok(all_issues)
    }

    pub async fn list_user_orgs(&self) -> Result<Vec<OrgInfo>> {
        let mut all_orgs = Vec::new();
        let mut page = 1u32;
        loop {
            let orgs: Vec<OrgInfo> = self
                .octocrab
                .get(
                    "/user/orgs",
                    Some(&[("per_page", "100"), ("page", &page.to_string())]),
                )
                .await?;
            if orgs.is_empty() {
                break;
            }
            all_orgs.extend(orgs);
            page += 1;
        }
        Ok(all_orgs)
    }
}
