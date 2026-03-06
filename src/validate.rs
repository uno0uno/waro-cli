use anyhow::{bail, Result};

/// Reject path traversal, null bytes, CRLF injection, and embedded query params.
fn validate_safe_string(name: &str, val: &str) -> Result<()> {
    if val.contains("../") || val.contains("..\\") {
        bail!("invalid value for --{}: path traversal not allowed", name);
    }
    if val.contains('\0') {
        bail!("invalid value for --{}: null bytes not allowed", name);
    }
    if val.contains('\r') || val.contains('\n') {
        bail!(
            "invalid value for --{}: newline characters not allowed",
            name
        );
    }
    if val.contains('?') {
        bail!(
            "invalid value for --{}: embedded query parameters not allowed",
            name
        );
    }
    Ok(())
}

/// Validate a UUID string (8-4-4-4-12 lowercase hex).
pub fn validate_uuid(name: &str, val: &str) -> Result<()> {
    validate_safe_string(name, val)?;
    let parts: Vec<&str> = val.split('-').collect();
    let valid = parts.len() == 5
        && parts[0].len() == 8
        && parts[1].len() == 4
        && parts[2].len() == 4
        && parts[3].len() == 4
        && parts[4].len() == 12
        && parts
            .iter()
            .all(|p| p.chars().all(|c| c.is_ascii_hexdigit()));
    if !valid {
        bail!(
            "invalid value for --{}: expected UUID format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            name
        );
    }
    Ok(())
}

/// Validate a date string (YYYY-MM-DD, numeric ranges only).
pub fn validate_date(name: &str, val: &str) -> Result<()> {
    validate_safe_string(name, val)?;
    let parts: Vec<&str> = val.split('-').collect();
    let valid = parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
        && {
            let month: u32 = parts[1].parse().unwrap_or(0);
            let day: u32 = parts[2].parse().unwrap_or(0);
            (1..=12).contains(&month) && (1..=31).contains(&day)
        };
    if !valid {
        bail!(
            "invalid value for --{}: expected date format YYYY-MM-DD (e.g. 2026-03-01)",
            name
        );
    }
    Ok(())
}

/// Validate that a string matches one of the allowed enum values.
pub fn validate_enum(name: &str, val: &str, options: &[&str]) -> Result<()> {
    if !options.contains(&val) {
        bail!(
            "invalid value for --{}: '{}' is not allowed. Valid values: {}",
            name,
            val,
            options.join(", ")
        );
    }
    Ok(())
}
