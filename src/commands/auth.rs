use crate::config::{load_config, save_config};
use crate::display;
use crate::error::Result;
use crate::github::GithubClient;

pub async fn run(token: &Option<String>) -> Result<()> {
    let token = match token {
        Some(t) => t.clone(),
        None => {
            eprintln!("Enter your GitHub personal access token:");
            rpassword::read_password()?
        }
    };

    let token = token.trim().to_string();

    let client = GithubClient::new(&token, false)?;
    let user = client.validate_token().await?;

    let mut config = load_config()?;
    config.auth.token = Some(token);
    save_config(&config)?;

    display::success(&format!(
        "Authenticated as {} ({})",
        user.login,
        user.name.as_deref().unwrap_or("no name set")
    ));

    Ok(())
}
