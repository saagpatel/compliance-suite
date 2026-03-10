use super::ColumnProfile;
use super::ParsedQuestionnaireRow;
use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use crate::domain::ids::Ulid;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const SAMPLE_LIMIT: usize = 5;
const ROW_LIMIT: usize = 50;

pub(crate) fn profile_columns(path: &Path) -> CoreResult<Vec<ColumnProfile>> {
    // XLSX is a zip file containing XML parts. For Phase 2 we keep a small,
    // dependency-free parser that is good enough for our sanitized fixtures.
    let tmp = std::env::temp_dir().join(format!("cs_xlsx_{}_{}", std::process::id(), Ulid::new()?));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;

    crate::util::zip::unzip_to_dir(path, &tmp)?;

    let shared = read_shared_strings(&tmp)?;
    let sheet_path = pick_sheet_xml(&tmp)?;
    let sheet_xml = crate::util::fs::read_to_string(&sheet_path)?;

    let mut header_by_col: BTreeMap<String, String> = BTreeMap::new();
    let mut non_empty_by_col: BTreeMap<String, i64> = BTreeMap::new();
    let mut samples_by_col: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut max_col_idx: i64 = -1;

    // Scan rows (1..ROW_LIMIT) and populate maps.
    let mut row_count = 0usize;
    let mut pos = 0usize;
    while let Some(row_start) = find_from(&sheet_xml, "<row", pos) {
        let row_tag_end = find_from(&sheet_xml, ">", row_start)
            .ok_or_else(|| CoreError::new(CoreErrorCode::ImportFailed, "invalid xlsx row tag"))?;
        let row_tag = &sheet_xml[row_start..=row_tag_end];
        let row_r = attr_value(row_tag, "r").unwrap_or_else(|| "0".to_string());
        let row_num: i64 = row_r.parse().unwrap_or(0);

        let row_end = find_from(&sheet_xml, "</row>", row_tag_end).ok_or_else(|| {
            CoreError::new(CoreErrorCode::ImportFailed, "invalid xlsx row end tag")
        })?;
        let row_body = &sheet_xml[row_tag_end + 1..row_end];

        parse_cells_in_row(
            row_num,
            row_body,
            &shared,
            &mut header_by_col,
            &mut non_empty_by_col,
            &mut samples_by_col,
            &mut max_col_idx,
        )?;

        pos = row_end + "</row>".len();
        row_count += 1;
        if row_count >= ROW_LIMIT {
            break;
        }
    }

    if max_col_idx < 0 {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(CoreError::new(
            CoreErrorCode::ImportFailed,
            "xlsx contained no readable cells",
        ));
    }

    let mut cols = Vec::new();
    for idx in 0..=max_col_idx {
        let col_ref = index_to_col_letters(idx as usize);
        let label = header_by_col
            .get(&col_ref)
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| col_ref.clone());
        let non_empty_count = *non_empty_by_col.get(&col_ref).unwrap_or(&0);
        let sample = samples_by_col.remove(&col_ref).unwrap_or_default();

        cols.push(ColumnProfile {
            col_ref,
            ordinal: idx,
            label,
            non_empty_count,
            sample,
        });
    }

    let _ = std::fs::remove_dir_all(&tmp);
    Ok(cols)
}

pub(crate) fn extract_rows(path: &Path) -> CoreResult<Vec<ParsedQuestionnaireRow>> {
    let tmp = std::env::temp_dir().join(format!("cs_xlsx_{}_{}", std::process::id(), Ulid::new()?));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;

    crate::util::zip::unzip_to_dir(path, &tmp)?;

    let shared = read_shared_strings(&tmp)?;
    let sheet_path = pick_sheet_xml(&tmp)?;
    let sheet_xml = crate::util::fs::read_to_string(&sheet_path)?;

    let mut rows = Vec::new();
    let mut pos = 0usize;
    while let Some(row_start) = find_from(&sheet_xml, "<row", pos) {
        let row_tag_end = find_from(&sheet_xml, ">", row_start)
            .ok_or_else(|| CoreError::new(CoreErrorCode::ImportFailed, "invalid xlsx row tag"))?;
        let row_tag = &sheet_xml[row_start..=row_tag_end];
        let row_r = attr_value(row_tag, "r").unwrap_or_else(|| "0".to_string());
        let row_num: i64 = row_r.parse().unwrap_or(0);

        let row_end = find_from(&sheet_xml, "</row>", row_tag_end).ok_or_else(|| {
            CoreError::new(CoreErrorCode::ImportFailed, "invalid xlsx row end tag")
        })?;
        let row_body = &sheet_xml[row_tag_end + 1..row_end];

        let mut cells = BTreeMap::new();
        collect_cells_in_row(row_body, &shared, &mut cells)?;

        if row_num > 1 && !cells.is_empty() {
            rows.push(ParsedQuestionnaireRow {
                row_ordinal: row_num,
                cells,
            });
        }

        pos = row_end + "</row>".len();
    }

    let _ = std::fs::remove_dir_all(&tmp);
    Ok(rows)
}

fn pick_sheet_xml(unzipped_root: &Path) -> CoreResult<PathBuf> {
    let sheet1 = unzipped_root
        .join("xl")
        .join("worksheets")
        .join("sheet1.xml");
    if sheet1.exists() {
        return Ok(sheet1);
    }

    let dir = unzipped_root.join("xl").join("worksheets");
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| CoreError::new(CoreErrorCode::ImportFailed, e.to_string()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "xml").unwrap_or(false))
        .collect();
    candidates.sort();

    candidates.into_iter().next().ok_or_else(|| {
        CoreError::new(
            CoreErrorCode::ImportFailed,
            "xlsx missing worksheets/sheet*.xml",
        )
    })
}

fn read_shared_strings(unzipped_root: &Path) -> CoreResult<Vec<String>> {
    let p = unzipped_root.join("xl").join("sharedStrings.xml");
    if !p.exists() {
        return Ok(Vec::new());
    }
    let xml = crate::util::fs::read_to_string(&p)?;
    Ok(parse_shared_strings(&xml))
}

fn parse_shared_strings(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(si_start) = find_from(xml, "<si", pos) {
        let si_open_end = match find_from(xml, ">", si_start) {
            Some(v) => v,
            None => break,
        };
        let si_end = match find_from(xml, "</si>", si_open_end) {
            Some(v) => v,
            None => break,
        };
        let body = &xml[si_open_end + 1..si_end];

        let mut text = String::new();
        let mut tpos = 0usize;
        while let Some(t_start) = find_from(body, "<t", tpos) {
            let t_open_end = match find_from(body, ">", t_start) {
                Some(v) => v,
                None => break,
            };
            let t_end = match find_from(body, "</t>", t_open_end) {
                Some(v) => v,
                None => break,
            };
            let raw = &body[t_open_end + 1..t_end];
            text.push_str(&decode_xml_entities(raw));
            tpos = t_end + "</t>".len();
        }
        out.push(text);
        pos = si_end + "</si>".len();
    }
    out
}

fn parse_cells_in_row(
    row_num: i64,
    row_body: &str,
    shared: &[String],
    header_by_col: &mut BTreeMap<String, String>,
    non_empty_by_col: &mut BTreeMap<String, i64>,
    samples_by_col: &mut BTreeMap<String, Vec<String>>,
    max_col_idx: &mut i64,
) -> CoreResult<()> {
    let mut pos = 0usize;
    while let Some(c_start) = find_from(row_body, "<c", pos) {
        let c_tag_end = find_from(row_body, ">", c_start)
            .ok_or_else(|| CoreError::new(CoreErrorCode::ImportFailed, "invalid xlsx cell tag"))?;
        let c_tag = &row_body[c_start..=c_tag_end];

        let cell_ref = match attr_value(c_tag, "r") {
            Some(v) => v,
            None => {
                pos = c_tag_end + 1;
                continue;
            }
        };
        let col_letters = cell_ref
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_ascii_uppercase();
        if col_letters.is_empty() {
            pos = c_tag_end + 1;
            continue;
        }

        let col_idx = col_letters_to_index(&col_letters);
        if col_idx > *max_col_idx {
            *max_col_idx = col_idx;
        }

        let t = attr_value(c_tag, "t").unwrap_or_default();

        let c_end = find_from(row_body, "</c>", c_tag_end).unwrap_or(c_tag_end);
        let c_body = if c_end > c_tag_end {
            &row_body[c_tag_end + 1..c_end]
        } else {
            ""
        };

        let value = read_cell_value(c_body, &t, shared);
        let value = value.trim();

        if row_num == 1 {
            if !value.is_empty() {
                header_by_col.insert(col_letters.clone(), value.to_string());
            }
        } else if !value.is_empty() {
            *non_empty_by_col.entry(col_letters.clone()).or_insert(0) += 1;
            let entry = samples_by_col.entry(col_letters.clone()).or_default();
            if entry.len() < SAMPLE_LIMIT {
                entry.push(value.to_string());
            }
        }

        pos = if c_end > c_tag_end {
            c_end + "</c>".len()
        } else {
            c_tag_end + 1
        };
    }
    Ok(())
}

fn collect_cells_in_row(
    row_body: &str,
    shared: &[String],
    cells: &mut BTreeMap<String, String>,
) -> CoreResult<()> {
    let mut pos = 0usize;
    while let Some(c_start) = find_from(row_body, "<c", pos) {
        let c_tag_end = find_from(row_body, ">", c_start)
            .ok_or_else(|| CoreError::new(CoreErrorCode::ImportFailed, "invalid xlsx cell tag"))?;
        let c_tag = &row_body[c_start..=c_tag_end];

        let cell_ref = match attr_value(c_tag, "r") {
            Some(v) => v,
            None => {
                pos = c_tag_end + 1;
                continue;
            }
        };
        let col_letters = cell_ref
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_ascii_uppercase();
        if col_letters.is_empty() {
            pos = c_tag_end + 1;
            continue;
        }

        let t = attr_value(c_tag, "t").unwrap_or_default();

        let c_end = find_from(row_body, "</c>", c_tag_end).unwrap_or(c_tag_end);
        let c_body = if c_end > c_tag_end {
            &row_body[c_tag_end + 1..c_end]
        } else {
            ""
        };

        let value = read_cell_value(c_body, &t, shared);
        if !value.trim().is_empty() {
            cells.insert(col_letters, value);
        }

        pos = if c_end > c_tag_end {
            c_end + "</c>".len()
        } else {
            c_tag_end + 1
        };
    }
    Ok(())
}

fn read_cell_value(c_body: &str, cell_type: &str, shared: &[String]) -> String {
    if cell_type == "s" {
        if let Some(v) = extract_simple_tag_text(c_body, "v") {
            if let Ok(idx) = v.parse::<usize>() {
                return shared.get(idx).cloned().unwrap_or_default();
            }
        }
        return String::new();
    }

    if cell_type == "inlineStr" {
        // <is><t>...</t></is>
        if let Some(t) = extract_simple_tag_text(c_body, "t") {
            return decode_xml_entities(&t);
        }
        return String::new();
    }

    extract_simple_tag_text(c_body, "v")
        .map(|s| decode_xml_entities(&s))
        .unwrap_or_default()
}

fn extract_simple_tag_text(s: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = s.find(&open)?;
    let end = s[start + open.len()..].find(&close)?;
    let inner = &s[start + open.len()..start + open.len() + end];
    Some(inner.to_string())
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    // Very small attribute scanner: name="...".
    let pat = format!("{}=\"", name);
    let i = tag.find(&pat)?;
    let rest = &tag[i + pat.len()..];
    let j = rest.find('"')?;
    Some(rest[..j].to_string())
}

fn find_from(hay: &str, needle: &str, from: usize) -> Option<usize> {
    hay.get(from..)?.find(needle).map(|i| from + i)
}

fn decode_xml_entities(s: &str) -> String {
    // Minimal entity decoder for our fixture XML.
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn col_letters_to_index(s: &str) -> i64 {
    // A -> 0, B -> 1, Z -> 25, AA -> 26...
    let mut n: i64 = 0;
    for c in s.chars() {
        if !c.is_ascii_alphabetic() {
            continue;
        }
        let v = (c.to_ascii_uppercase() as u8 - b'A' + 1) as i64;
        n = n * 26 + v;
    }
    n - 1
}

fn index_to_col_letters(mut idx: usize) -> String {
    // 0 -> A, 1 -> B, 25 -> Z, 26 -> AA...
    let mut out = String::new();
    idx += 1;
    while idx > 0 {
        let rem = (idx - 1) % 26;
        out.push((b'A' + rem as u8) as char);
        idx = (idx - 1) / 26;
    }
    out.chars().rev().collect()
}
