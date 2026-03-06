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

### Prebuilt binary (coming soon)
```bash
curl -fsSL https://get.warocol.com/cli | sh
```

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

## Global flags

| Flag | Description |
|---|---|
| `--output json\|table` | Output format (default: json) |
| `--fields id,name,...` | Return only these fields (reduces response size) |

## Authentication

Every command requires a `WARO_API_KEY` starting with `waro_sk_`. Generate one in the WaRo dashboard under **Settings → API Tokens**.

## For AI Agents

See [SKILL.md](SKILL.md) for invariants, canonical examples, and scope requirements.

## Roadmap

See [GitHub Issues](https://github.com/uno0uno/waro-cli/issues) for planned features.
