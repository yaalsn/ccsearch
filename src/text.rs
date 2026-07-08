//! CJK-aware analyzer, FTS5 MATCH builders, ANSI colouring, snippets, dates.
//!
//! FTS5's own tokenizers can't do Chinese substring search (trigram needs >=3
//! chars, unicode61 makes a whole CJK run one token). So we tokenize ourselves:
//! CJK runs become overlapping bigrams ("全文检索" -> 全文 文检 检索) so a 2-char
//! query matches; ASCII stays whole words (queried as prefixes).

use std::collections::HashSet;
use std::sync::OnceLock;

// ---- colour -----------------------------------------------------------------
static COLOR: OnceLock<bool> = OnceLock::new();

pub fn set_color(on: bool) {
    let _ = COLOR.set(on);
}
fn color() -> bool {
    *COLOR.get().unwrap_or(&false)
}
fn wrap(code: &str, s: &str) -> String {
    if color() {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}
pub fn bold(s: &str) -> String {
    wrap("1", s)
}
pub fn dim(s: &str) -> String {
    wrap("2", s)
}
pub fn cyan(s: &str) -> String {
    wrap("36", s)
}
pub fn green(s: &str) -> String {
    wrap("32", s)
}
pub fn yellow(s: &str) -> String {
    wrap("33", s)
}
fn hl(s: &str) -> String {
    wrap("1;43;30", s)
}

// ---- CJK tokenizer ----------------------------------------------------------
fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x3400..=0x4DBF   // CJK Ext A
        | 0x4E00..=0x9FFF // CJK Unified
        | 0xF900..=0xFAFF // CJK Compat Ideographs
        | 0x3040..=0x30FF // Hiragana + Katakana
        | 0xAC00..=0xD7AF // Hangul syllables
    )
}

/// Tokenize into FTS terms: CJK -> overlapping bigrams, ASCII -> whole words.
pub fn analyze(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.to_lowercase().chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if is_cjk(c) {
            let start = i;
            while i < chars.len() && is_cjk(chars[i]) {
                i += 1;
            }
            let run = &chars[start..i];
            if run.len() == 1 {
                toks.push(run[0].to_string());
            } else {
                for w in run.windows(2) {
                    toks.push(w.iter().collect());
                }
            }
        } else if c.is_ascii_alphanumeric() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_alphanumeric() {
                i += 1;
            }
            toks.push(chars[start..i].iter().collect());
        } else {
            i += 1;
        }
    }
    toks
}

/// Join analyzed tokens with spaces for storage in the FTS index.
pub fn tokenized(text: &str) -> String {
    analyze(text).join(" ")
}

fn term_expr(term: &str) -> Option<String> {
    let t = analyze(term);
    if t.is_empty() {
        return None;
    }
    if t.len() == 1 {
        let tok = &t[0];
        let ascii = tok.chars().next().is_some_and(|c| c.is_ascii());
        Some(if ascii {
            format!("\"{tok}\"*") // prefix-match ASCII
        } else {
            format!("\"{tok}\"")
        })
    } else {
        Some(format!("\"{}\"", t.join(" "))) // CJK bigram phrase (adjacent)
    }
}

/// Strict query: AND across the user's whitespace-separated terms.
pub fn to_match(query: &str) -> String {
    query
        .split_whitespace()
        .filter_map(term_expr)
        .collect::<Vec<_>>()
        .join(" AND ")
}

/// Expanded query: OR across every word/CJK-run in a list of related terms.
pub fn to_match_or(terms: &[String]) -> String {
    let joined = terms.join(" ");
    let mut seen = HashSet::new();
    let mut exprs = Vec::new();
    for w in joined.split_whitespace() {
        if let Some(e) = term_expr(w) {
            if seen.insert(e.clone()) {
                exprs.push(e);
            }
        }
    }
    exprs.join(" OR ")
}

// ---- snippet ----------------------------------------------------------------
fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A ~width-char window around the earliest matching term, with matches highlit.
pub fn snippet(content: &str, terms: &[String], width: usize) -> String {
    let orig: Vec<char> = content.chars().collect();
    let lower = content.to_lowercase();
    // earliest term start, as a char index into `content` (lower is ~1:1 for CJK/ASCII)
    let mut pos: Option<usize> = None;
    for t in terms {
        if t.is_empty() {
            continue;
        }
        if let Some(byte_p) = lower.find(&t.to_lowercase()) {
            let ci = lower[..byte_p].chars().count();
            pos = Some(pos.map_or(ci, |x| x.min(ci)));
        }
    }
    let start = pos.map_or(0, |p| p.saturating_sub(40));
    let end = (start + width).min(orig.len());
    let mut seg: String = orig[start..end].iter().collect();
    seg = collapse_ws(&seg);
    if start > 0 {
        seg = format!("…{seg}");
    }
    highlight(&seg, terms)
}

/// Single pass over the ORIGINAL text (longest terms first) so we never re-scan
/// ANSI codes we just inserted — a short/numeric term like "0" must not match
/// digits inside an escape sequence.
fn highlight(seg: &str, terms: &[String]) -> String {
    if !color() {
        return seg.to_string();
    }
    let mut toks: Vec<Vec<char>> = terms
        .iter()
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase().chars().collect())
        .collect();
    toks.sort_by_key(|t| std::cmp::Reverse(t.len()));
    toks.dedup();
    if toks.is_empty() {
        return seg.to_string();
    }
    let chars: Vec<char> = seg.chars().collect();
    let lower: Vec<char> = seg.to_lowercase().chars().collect();
    // guard the 1:1 assumption; if lowercase changed the char count, skip highlighting
    if lower.len() != chars.len() {
        return seg.to_string();
    }
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let mut m = 0;
        for tc in &toks {
            if !tc.is_empty() && i + tc.len() <= lower.len() && lower[i..i + tc.len()] == tc[..] {
                m = tc.len();
                break;
            }
        }
        if m > 0 {
            let sub: String = chars[i..i + m].iter().collect();
            out.push_str(&hl(&sub));
            i += m;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

// ---- dates ------------------------------------------------------------------
pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Parse "2026-06-14T09:36:41.802Z" (always UTC) to epoch seconds.
fn parse_ts(ts: &str) -> Option<i64> {
    if ts.len() < 19 {
        return None;
    }
    let year: i64 = ts.get(0..4)?.parse().ok()?;
    let month: i64 = ts.get(5..7)?.parse().ok()?;
    let day: i64 = ts.get(8..10)?.parse().ok()?;
    let hour: i64 = ts.get(11..13)?.parse().ok()?;
    let min: i64 = ts.get(14..16)?.parse().ok()?;
    let sec: i64 = ts.get(17..19)?.parse().ok()?;
    // days_from_civil (Howard Hinnant)
    let y = if month <= 2 { year - 1 } else { year };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

pub fn age_days(ts: &str, now: i64) -> Option<f64> {
    parse_ts(ts).map(|t| (now - t) as f64 / 86400.0)
}

pub fn rel_date(ts: &str) -> String {
    let d = match age_days(ts, now_secs()) {
        Some(d) => d,
        None => return "?".to_string(),
    };
    if d < 1.0 / 24.0 {
        "just now".to_string()
    } else if d < 1.0 {
        format!("{}h ago", (d * 24.0) as i64)
    } else if d < 30.0 {
        format!("{}d ago", d as i64)
    } else if d < 365.0 {
        format!("{}mo ago", (d / 30.0) as i64)
    } else {
        format!("{}y ago", (d / 365.0) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_becomes_overlapping_bigrams() {
        assert_eq!(analyze("全文检索"), vec!["全文", "文检", "检索"]);
        assert_eq!(analyze("检"), vec!["检"]); // lone CJK char kept as unigram
    }

    #[test]
    fn ascii_is_lowercased_whole_words() {
        assert_eq!(analyze("Docker Compose"), vec!["docker", "compose"]);
        assert_eq!(analyze("usb-c"), vec!["usb", "c"]); // '-' splits
    }

    #[test]
    fn match_builders() {
        assert_eq!(to_match("检索"), "\"检索\""); // 2-char CJK -> single bigram phrase
        assert_eq!(to_match("airwall"), "\"airwall\"*"); // ASCII -> prefix
        assert_eq!(to_match("全文 检索"), "\"全文\" AND \"检索\"");
        let m = to_match_or(&["cut delay".into(), "延迟".into()]);
        assert!(m.contains(" OR ") && m.contains("\"cut\"*") && m.contains("\"延迟\""));
    }

    #[test]
    fn highlight_does_not_corrupt_ansi_with_numeric_terms() {
        set_color(true);
        let out = snippet(
            "board 检索 0 and 43",
            &["检索".into(), "0".into(), "43".into()],
            140,
        );
        // stripping valid color codes must leave no stray ESC bytes (no nested escapes)
        let stripped: String = {
            let mut s = out.clone();
            while let Some(i) = s.find("\x1b[") {
                if let Some(j) = s[i..].find('m') {
                    s.replace_range(i..i + j + 1, "");
                } else {
                    break;
                }
            }
            s
        };
        assert!(!stripped.contains('\x1b'), "corrupted ANSI: {out:?}");
    }

    #[test]
    fn parses_utc_timestamps() {
        // 2026-06-14T00:00:00Z is 12 days before 2026-06-26T00:00:00Z
        let t0 = parse_ts("2026-06-14T00:00:00.000Z").unwrap();
        let t1 = parse_ts("2026-06-26T00:00:00.000Z").unwrap();
        assert_eq!(t1 - t0, 12 * 86400);
        assert!(parse_ts("garbage").is_none());
    }
}
