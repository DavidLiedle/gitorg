use crate::config::{load_config, save_config};
use crate::display;
use crate::error::Result;
use crate::github::GithubClient;

pub async fn run(token: &Option<String>) -> Result<()> {
    let token = match token {
        Some(t) => t.clone(),
        None => {
            let url = "https://github.com/settings/tokens/new?description=gitorg&scopes=read:org,repo";
            eprintln!("Opening GitHub token creation page in your browser...");
            if open::that(url).is_err() {
                eprintln!("Could not open browser. Visit: {url}");
            }
            eprintln!("\nPaste your token below:");
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
