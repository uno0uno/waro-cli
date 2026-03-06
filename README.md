# waro-cli

Developer CLI for the [WaRo Colombia](https://warocol.com) public API — built in Rust for fast, reliable, scriptable access.

## Install

### From source (requires Rust)
```bash
git clone https://github.com/uno0uno/waro-cli
cd waro-cli
cargo build --release
# Binary at: ./target/release/waro
```

### Prebuilt binary (macOS + Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/uno0uno/waro-cli/main/install.sh | sh
```

Installs to `/usr/local/bin/waro` (or `~/.local/bin/waro` if no write access).
Supported: macOS ARM64, macOS Intel, Linux x86_64, Linux ARM64.

## Setup

```bash
cp .env.example .env
# Edit .env — add your WARO_API_KEY
```

Or export directly:
```bash
export WARO_API_URL=https://api.warocol.com
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

## Global flags

| Flag | Description |
|---|---|
| `--output json\|table` | Output format (default: json) |
| `--fields id,name,...` | Return only these fields (reduces response size) |
| `--no-color` | Disable colored output |

## Authentication

Every command requires a `WARO_API_KEY` starting with `waro_sk_`. Generate one in the WaRo dashboard under **Settings → API Tokens**.

## For AI Agents

See [SKILL.md](SKILL.md) for invariants, canonical examples, and scope requirements.

## Roadmap

See [GitHub Issues](https://github.com/uno0uno/waro-cli/issues) for planned features.
