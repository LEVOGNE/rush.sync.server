/// Shared parsing utilities for bulk server operations (start, stop, etc.)

#[derive(Debug)]
pub enum BulkMode {
    Single(String),
    Range(u32, u32),
    All,
    Invalid(String),
}

/// Parse arguments for bulk server operations.
/// Supports: "all", "1-3" (range), or single identifier.
pub fn parse_bulk_args(args: &[&str]) -> BulkMode {
    if args.len() != 1 {
        return BulkMode::Invalid("Too many arguments".to_string());
    }

    let arg = args[0];

    if arg.eq_ignore_ascii_case("all") {
        return BulkMode::All;
    }

    // Range: "1-3" or "001-005"
    if let Some((start_str, end_str)) = arg.split_once('-') {
        match (start_str.parse::<u32>(), end_str.parse::<u32>()) {
            (Ok(start), Ok(end)) => {
                if start == 0 || end == 0 {
                    return BulkMode::Invalid("Range indices must be > 0".to_string());
                }
                if start > end {
                    return BulkMode::Invalid("Start must be <= end in range".to_string());
                }
                if end - start > 500 {
                    return BulkMode::Invalid("Maximum 500 servers in range operation".to_string());
                }
                BulkMode::Range(start, end)
            }
            _ => BulkMode::Single(arg.to_string()),
        }
    } else {
        BulkMode::Single(arg.to_string())
    }
}
