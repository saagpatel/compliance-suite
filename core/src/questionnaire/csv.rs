use super::ColumnProfile;
use super::ParsedQuestionnaireRow;
use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use std::collections::BTreeMap;
use std::path::Path;

const SAMPLE_LIMIT: usize = 5;
const ROW_LIMIT: usize = 50;

pub(crate) fn profile_columns(path: &Path) -> CoreResult<Vec<ColumnProfile>> {
    let s = crate::util::fs::read_to_string(path)?;
    let mut lines = s.lines();
    let header_line = lines.next().ok_or_else(|| {
        CoreError::new(
            CoreErrorCode::ImportFailed,
            "CSV file is empty (missing header)",
        )
    })?;

    let headers = parse_csv_line(header_line.trim_end_matches('\r'))?;
    if headers.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::ImportFailed,
            "CSV header row is empty",
        ));
    }

    // Enforce unique column refs so mapping is unambiguous.
    for (i, h) in headers.iter().enumerate() {
        if h.trim().is_empty() {
            return Err(CoreError::new(
                CoreErrorCode::ImportFailed,
                format!("CSV header contains empty column name at index {}", i),
            ));
        }
        if headers.iter().filter(|x| *x == h).count() > 1 {
            return Err(CoreError::new(
                CoreErrorCode::ImportFailed,
                format!("CSV header contains duplicate column name: {}", h),
            ));
        }
    }

    let mut cols: Vec<ColumnProfile> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| ColumnProfile {
            col_ref: h.clone(),
            ordinal: i as i64,
            label: h.clone(),
            non_empty_count: 0,
            sample: Vec::new(),
        })
        .collect();

    for line in lines.take(ROW_LIMIT) {
        let fields = parse_csv_line(line.trim_end_matches('\r'))?;
        for (col_i, c) in cols.iter_mut().enumerate() {
            let v = fields.get(col_i).map(|s| s.as_str()).unwrap_or("");
            let v = v.trim();
            if v.is_empty() {
                continue;
            }
            c.non_empty_count += 1;
            if c.sample.len() < SAMPLE_LIMIT {
                c.sample.push(v.to_string());
            }
        }
    }

    Ok(cols)
}

pub(crate) fn extract_rows(path: &Path) -> CoreResult<Vec<ParsedQuestionnaireRow>> {
    let s = crate::util::fs::read_to_string(path)?;
    let mut lines = s.lines();
    let header_line = lines.next().ok_or_else(|| {
        CoreError::new(
            CoreErrorCode::ImportFailed,
            "CSV file is empty (missing header)",
        )
    })?;

    let headers = parse_csv_line(header_line.trim_end_matches('\r'))?;
    if headers.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::ImportFailed,
            "CSV header row is empty",
        ));
    }

    let mut out = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let fields = parse_csv_line(line.trim_end_matches('\r'))?;
        let mut cells = BTreeMap::new();
        for (col_index, header) in headers.iter().enumerate() {
            let value = fields.get(col_index).cloned().unwrap_or_default();
            if !value.trim().is_empty() {
                cells.insert(header.clone(), value);
            }
        }
        if cells.is_empty() {
            continue;
        }
        out.push(ParsedQuestionnaireRow {
            row_ordinal: (line_index as i64) + 2,
            cells,
        });
    }

    Ok(out)
}

fn parse_csv_line(line: &str) -> CoreResult<Vec<String>> {
    // Minimal RFC4180-ish parser good enough for fixtures and offline-first imports.
    // Handles:
    // - commas as separators
    // - double-quoted fields
    // - "" as escaped quote inside quoted fields
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            match ch {
                '"' => {
                    if matches!(chars.peek(), Some('"')) {
                        // Escaped quote.
                        let _ = chars.next();
                        cur.push('"');
                    } else {
                        in_quotes = false;
                    }
                }
                _ => cur.push(ch),
            }
        } else {
            match ch {
                ',' => {
                    out.push(cur);
                    cur = String::new();
                }
                '"' => in_quotes = true,
                _ => cur.push(ch),
            }
        }
    }

    if in_quotes {
        return Err(CoreError::new(
            CoreErrorCode::ImportFailed,
            "CSV parse error: unterminated quote",
        ));
    }

    out.push(cur);
    Ok(out)
}
