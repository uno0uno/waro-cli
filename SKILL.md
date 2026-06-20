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
- Use `--output agent-json` for machine workflows; it returns a stable envelope and structured errors
- ALWAYS use `--fields` on list commands to reduce context window usage
- Treat `waro schema <group> <subcommand> .response.fields` as the source of truth for valid fields
- NEVER expose user emails, names, or phone numbers in outputs — work with IDs only
- Pagination: use `--limit 50 --offset N` to page through results
- Default timezone is `America/Bogota` — override with `--timezone America/Mexico_City` etc.

## Schema Introspection (start here)

```bash
# Discover all endpoints and their params (no API key needed)
waro schema

# Inspect a specific endpoint before calling it
waro schema sales list
waro schema sales detail
waro schema customers list | jq '.response'

# Find required params before calling
waro schema sales detail | jq '.params[] | select(.required == true)'
```

## Canonical Examples

```bash
# List recent sales (agent JSON, minimal fields)
waro --output agent-json sales list --limit 20 --fields id,status,totalAmount,orderDate

# List customers with the stable agent envelope
waro --output agent-json customers list --limit 20 --fields customer_id,total_spent,order_count

# Sales for a date range
waro sales list --date-from 2026-03-01 --date-to 2026-03-06 --status completed

# Sales metrics grouped by day
waro sales metrics --group-by date --date-from 2026-03-01 --date-to 2026-03-06

# Get specific sale
waro sales detail --order-id <uuid>

# Dry run before fetching
waro sales list --dry-run

# Menu products (minimal fields)
waro --output agent-json menu products --fields id,name,price,isAvailable

# Table output
waro --output table sales list --fields id,status,totalAmount --limit 10
```

## Scopes Required

| Command | Scope needed |
|---|---|
| `waro sales *` | `orders:read` |
| `waro customers *` | `customers:read` |
| `waro menu *` | `menu:read` |
| `waro analytics *` | `analytics:read` |
| `waro financial *` | `financial:read` |
| `waro waros *` | `waros:read` |

## Roadmap (not yet implemented)

See GitHub issues: https://github.com/uno0uno/waro-cli
