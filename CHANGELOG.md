# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `auth` command for authenticating with a GitHub personal access token
- `orgs` command to list user organizations
- `repos` command with sorting by activity, stars, staleness, or name
- `stale` command to find repositories with no recent pushes
- `issues` command to list open issues across organizations (excludes PRs)
- `stats` command for aggregate statistics (repos, stars, forks, languages)
- `overview` command for a full multi-section dashboard
- Global `--json` flag for machine-readable output
- Global `--verbose` flag for rate limit info and debug output
- Config file at `~/.config/gitorg/config.toml` with XDG support
- Secure token storage with 0600 file permissions on Unix
- CI workflow with check, test, format, and clippy jobs
