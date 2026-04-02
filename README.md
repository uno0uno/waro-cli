# waro-cli

Developer CLI for the [WaRo Colombia](https://warocol.com) public API — built in Rust for fast, reliable, scriptable access.

## Install

### Prebuilt binary — recommended (macOS + Linux)

Download and run the installer:

```bash
curl -fsSL https://raw.githubusercontent.com/uno0uno/waro-cli/main/install.sh | sh
```

Or inspect the script before running it:

```bash
curl -fsSL https://raw.githubusercontent.com/uno0uno/waro-cli/main/install.sh > install.sh
cat install.sh   # review it
sh install.sh
```

Installs to `/usr/local/bin/waro` (or `~/.local/bin/waro` if no write access).
Supported platforms: macOS ARM64, macOS Intel, Linux x86_64, Linux ARM64.

### From source (requires Rust)

```bash
git clone https://github.com/uno0uno/waro-cli
cd waro-cli
cargo build --release
cp target/release/waro ~/.local/bin/waro   # or any directory in your PATH
```

### Update

Re-run the installer at any time — it always fetches the latest release:

```bash
curl -fsSL https://raw.githubusercontent.com/uno0uno/waro-cli/main/install.sh | sh
```

## Setup

```bash
cp .env.example .env
# Edit .env — add your WARO_API_KEY
```

Or export directly:
```bash
export WARO_API_URL=https://api.warolabs.com
export WARO_API_KEY=waro_sk_your_key_here
```

## Usage

```bash
waro --help

# Sales
waro sales list --limit 20 --fields id,status,total
waro sales list --date-from 2026-03-01 --date-to 2026-03-06 --status completed
waro sales metrics --group-by date --date-from 2026-03-01
waro sales detail --order-id <uuid>

# Menu
waro menu products --fields id,name,price
waro menu recipes
waro menu modifiers

# Table output
waro --output table sales list --fields id,status,total --limit 10

# Inspect endpoint schema (useful for AI agents — no API key needed)
waro schema
waro schema sales list
waro schema sales detail | jq '.params[] | select(.required == true)'

# Auto-paginate (NDJSON output, one object per line)
waro sales list --all --fields id,status,total
waro menu products --all | wc -l

# Dry run (validate without API call)
waro sales list --dry-run

# Check config
waro config
```

## Shell Completion

Generate and install a completion script for your shell:

```bash
# zsh (recommended)
waro completions zsh > ~/.zsh/completions/_waro
# Reload: exec zsh

# bash
waro completions bash | sudo tee /etc/bash_completion.d/waro

# fish
waro completions fish > ~/.config/fish/completions/waro.fish
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`

## Profiles

Work with multiple environments (staging, production, local) using named profiles:

```toml
# ~/.waro/config.toml
[profiles.staging]
api_url = "https://staging-api.warolabs.com"
api_key  = "waro_sk_staging_xxx"

[profiles.prod]
api_url = "https://api.warolabs.com"
api_key  = "waro_sk_prod_xxx"
```

```bash
waro --profile staging sales list
waro --profile prod sales metrics --group-by date

# Or use an env var
export WARO_PROFILE=staging
waro sales list
```

If no profile is set, falls back to `WARO_API_KEY` / `WARO_API_URL` env vars (existing behaviour).

## Global flags

| Flag | Description |
|---|---|
| `--output json\|table` | Output format (default: json) |
| `--fields id,name,...` | Return only these fields (reduces response size) |
| `--no-color` | Disable colored output |
| `--profile <name>` | Use a named profile from `~/.waro/config.toml` |

## Authentication

Every command requires a `WARO_API_KEY` starting with `waro_sk_`. Generate one in the WaRo dashboard under **Settings → API Tokens**.

## For AI Agents

See [SKILL.md](SKILL.md) for invariants, canonical examples, and scope requirements.

## Roadmap

See [GitHub Issues](https://github.com/uno0uno/waro-cli/issues) for planned features.
