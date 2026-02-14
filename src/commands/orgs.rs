use crate::config::load_config;
use crate::display;
use crate::error::Result;
use crate::github::GithubClient;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct OrgSummary {
    pub name: String,
    pub description: String,
    pub url: String,
}

pub async fn run(json: bool, verbose: bool) -> Result<()> {
    let config = load_config()?;
    let token = config.token()?;
    let client = GithubClient::new(token, verbose)?;

    let orgs = client.list_user_orgs().await?;

    let summaries: Vec<OrgSummary> = orgs
        .into_iter()
        .map(|o| OrgSummary {
            name: o.login.clone(),
            description: o.description.unwrap_or_default(),
            url: format!("https://github.com/{}", o.login),
        })
        .collect();

    display::output(json, &summaries, |data| {
        render_orgs_table(data);
    });

    client.check_rate_limit_if_verbose().await;

    Ok(())
}

fn render_orgs_table(orgs: &[OrgSummary]) {
    if orgs.is_empty() {
        display::warn("No organizations found.");
        return;
    }

    display::section_header("Organizations");

    let mut table = display::new_table(&["Name", "Description", "URL"]);

    for org in orgs {
        table.add_row(vec![&org.name, &org.description, &org.url]);
    }

    println!("{table}");
    println!("\n{} organization(s) found.", orgs.len());
}
