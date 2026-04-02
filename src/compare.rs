use anyhow::{bail, Result};
use colored::Colorize;

/// Parse --compare-to value.
///
/// Accepted forms:
///   previous-period          → compareTo="previous_period"
///   previous-year            → compareTo="previous_year"
///   YYYY-MM-DD:YYYY-MM-DD   → compareTo="custom", compareFrom/compareDateTo set
///
/// Returns (compare_to, compare_from, compare_date_to)
pub fn parse_compare_to(s: &str) -> Result<(String, Option<String>, Option<String>)> {
    match s {
        "previous-period" => Ok(("previous_period".into(), None, None)),
        "previous-year" => Ok(("previous_year".into(), None, None)),
        _ => {
            let parts: Vec<&str> = s.splitn(2, ':').collect();
            if parts.len() == 2 {
                Ok((
                    "custom".into(),
                    Some(parts[0].to_string()),
                    Some(parts[1].to_string()),
                ))
            } else {
                bail!(
                    "--compare-to must be 'previous-period', 'previous-year', \
                     or 'YYYY-MM-DD:YYYY-MM-DD' (e.g. 2026-01-01:2026-01-31), got: '{}'",
                    s
                )
            }
        }
    }
}

/// Format a percentage change as an arrow + number + suffix, colourised.
///
/// `pct`: change in percent (positive = increase).
/// `lower_is_better`: invert the good/bad colour (e.g. food cost %).
/// `is_pp`: use "pp" (percentage-point) suffix instead of "%".
pub fn format_delta(pct: Option<f64>, lower_is_better: bool, is_pp: bool) -> String {
    let suffix = if is_pp { "pp" } else { "%" };
    let Some(p) = pct else {
        return "-".to_string();
    };
    if p.abs() < 0.05 {
        return format!("→0.0{}", suffix);
    }
    let (arrow, abs) = if p > 0.0 { ("↑", p) } else { ("↓", -p) };
    let plain = format!("{}{:.1}{}", arrow, abs, suffix);
    let good = if lower_is_better { p < 0.0 } else { p > 0.0 };
    if good {
        plain.green().to_string()
    } else {
        plain.red().to_string()
    }
}
