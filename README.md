# gitorg

A CLI tool for managing and monitoring multiple GitHub organizations.

## Install

```bash
cargo install --path .
```

Or build from source:

```bash
git clone https://github.com/davidliedle/gitorg.git
cd gitorg
cargo build --release
```

## Quick Start

```bash
# Authenticate with your GitHub personal access token
gitorg auth --token ghp_yourtoken

# Or authenticate interactively (prompts for token)
gitorg auth

# List your organizations
gitorg orgs

# List all repos sorted by stars
gitorg repos --sort stars

# Find stale repos (no push in 60+ days)
gitorg stale --days 60

# View open issues across all orgs
gitorg issues

# See aggregate stats
gitorg stats

# Full dashboard overview
gitorg overview
```

## Commands

| Command | Description |
|---------|-------------|
| `auth` | Authenticate with a GitHub personal access token |
| `orgs` | List your GitHub organizations |
| `repos` | List repositories across organizations |
| `stale` | Find stale repositories with no recent pushes |
| `issues` | List open issues across organizations |
| `stats` | Show aggregate statistics |
| `overview` | Show a full dashboard overview |

### Global Flags

- `--json` — Output results as JSON (for scripting/piping)
- `--verbose` — Show rate limit info and debug output

### Command Options

```bash
gitorg repos --org myorg --sort stars    # Filter org, sort by stars
gitorg repos --sort activity             # Sort by most recent push
gitorg repos --sort name                 # Sort alphabetically
gitorg repos --sort staleness            # Sort by least recent push

gitorg stale --days 30                   # Repos with no push in 30+ days
gitorg stale --org myorg --days 90       # Filter to specific org

gitorg issues --org myorg                # Issues for specific org

gitorg stats --org myorg                 # Stats for specific org

gitorg overview --org myorg --days 60    # Dashboard for specific org
```

## Configuration

Config is stored at `~/.config/gitorg/config.toml` (or `$XDG_CONFIG_HOME/gitorg/config.toml`).

```toml
[auth]
token = "ghp_yourtoken"

[defaults]
orgs = ["myorg", "otherorg"]
```

Setting `defaults.orgs` limits commands to those organizations by default. Without it, all organizations your token has access to are used.

## Token Permissions

Create a [personal access token](https://github.com/settings/tokens) with these scopes:

- `read:org` — List organizations
- `repo` — Access repositories and issues

## License

MIT
