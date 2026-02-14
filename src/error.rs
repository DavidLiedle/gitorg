use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum GitorgError {
    #[error("Not authenticated. Run `gitorg auth` first.")]
    NotAuthenticated,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("GitHub API error: {0}")]
    GitHub(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Rate limited. Resets at {0}. Please wait and retry.")]
    RateLimited(String),

    #[error("Organization not found: {0}")]
    OrgNotFound(String),
}

impl From<octocrab::Error> for GitorgError {
    fn from(err: octocrab::Error) -> Self {
        GitorgError::GitHub(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GitorgError>;
