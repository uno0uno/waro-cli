# WARO CLI — Agent Skill Guide

## Setup
```bash
cp .env.example .env
# Edit .env and add your WARO_API_KEY
cargo build --release
./target/release/waro --help
```

## Invariants for AI Agents

- ALWAYS use `--dry-run` before any mutation command to validate the request
- ALWAYS use `--fields` on list commands to reduce context window usage
- NEVER expose user emails, names, or phone numbers in outputs — work with IDs only
- Pagination: use `--limit 50 --offset N` to page through results
- Default timezone is `America/Bogota` — override with `--timezone America/Mexico_City` etc.

## Canonical Examples

```bash
# List recent sales (JSON, minimal fields)
waro sales list --limit 20 --fields id,status,total,order_date

# Sales for a date range
waro sales list --date-from 2026-03-01 --date-to 2026-03-06 --status completed

# Sales metrics grouped by day
waro sales metrics --group-by date --date-from 2026-03-01 --date-to 2026-03-06

# Get specific sale
waro sales detail --order-id <uuid>

# Dry run before fetching
waro sales list --dry-run

# Menu products (minimal fields)
waro menu products --fields id,name,price,is_available

# Table output
waro --output table sales list --fields id,status,total --limit 10
```

## Scopes Required

| Command | Scope needed |
|---|---|
| `waro sales *` | `orders:read` |
| `waro menu *` | `menu:read` |

## Roadmap (not yet implemented)

See GitHub issues: https://github.com/uno0uno/waro-cli
