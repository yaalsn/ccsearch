//! BM25 search, reciprocal-rank fusion, and LLM query expansion.

use crate::text::{age_days, now_secs};
use rusqlite::Connection;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Clone)]
pub struct Doc {
    pub session_id: String,
    pub cwd: String,
    pub branch: String,
    pub title: String,
    pub last_prompt: String,
    pub content: String,
    pub last_ts: String,
    pub msg_count: i64,
    pub project_dir: String,
    pub score: f64,
}

struct Raw {
    session_id: String,
    cwd: String,
    branch: String,
    title: String,
    last_prompt: String,
    content: String,
    last_ts: String,
    msg_count: i64,
    project_dir: String,
    bm: f64,
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// Run one FTS5 MATCH; rank by BM25 (title-heavy), min-max normalized + recency nudge.
pub fn run_match(
    conn: &Connection,
    match_expr: &str,
    here: Option<&str>,
    ignore: &[String],
    limit: usize,
) -> Vec<Doc> {
    if match_expr.is_empty() {
        return Vec::new();
    }
    let mut sql = String::from(
        "SELECT s.session_id,s.cwd,s.git_branch,s.title,s.last_prompt,s.content,\
         s.last_ts,s.msg_count,s.project_dir, bm25(fts,100.0,10.0,1.0) AS bm \
         FROM fts JOIN sessions s ON s.id=fts.rowid WHERE fts MATCH ?1",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(match_expr.to_string())];
    if let Some(h) = here {
        params.push(Box::new(h.to_string()));
        sql.push_str(&format!(" AND s.cwd = ?{}", params.len()));
    } else {
        for sub in ignore {
            params.push(Box::new(format!("%{sub}%")));
            sql.push_str(&format!(" AND s.cwd NOT LIKE ?{}", params.len()));
        }
    }
    let win = (limit * 4).max(40) as i64;
    params.push(Box::new(win));
    sql.push_str(&format!(" ORDER BY bm LIMIT ?{}", params.len()));

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mapped = stmt.query_map(refs.as_slice(), |row| {
        Ok(Raw {
            session_id: row.get(0)?,
            cwd: row.get(1)?,
            branch: row.get(2)?,
            title: row.get(3)?,
            last_prompt: row.get(4)?,
            content: row.get(5)?,
            last_ts: row.get(6)?,
            msg_count: row.get(7)?,
            project_dir: row.get(8)?,
            bm: row.get(9)?,
        })
    });
    let raws: Vec<Raw> = match mapped {
        Ok(it) => it.filter_map(Result::ok).collect(),
        Err(_) => return Vec::new(),
    };
    if raws.is_empty() {
        return Vec::new();
    }

    let rels: Vec<f64> = raws.iter().map(|r| -r.bm).collect(); // bm25: more negative == better
    let lo = rels.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = rels.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let now = now_secs();
    let mut docs: Vec<Doc> = raws
        .into_iter()
        .zip(rels)
        .map(|(r, raw)| {
            let rel = if hi > lo { (raw - lo) / (hi - lo) } else { 1.0 };
            let recency = age_days(&r.last_ts, now).map_or(0.0, |a| 1.0 / (1.0 + a / 30.0));
            Doc {
                session_id: r.session_id,
                cwd: r.cwd,
                branch: r.branch,
                title: r.title,
                last_prompt: r.last_prompt,
                content: r.content,
                last_ts: r.last_ts,
                msg_count: r.msg_count,
                project_dir: r.project_dir,
                score: round3(rel + 0.05 * recency),
            }
        })
        .collect();
    docs.sort_by(|a, b| b.score.total_cmp(&a.score));
    docs
}

/// Reciprocal-rank fusion: a doc high in either list ranks high; in both, higher.
pub fn rrf_fuse(lists: &[(&[Doc], f64)], k: f64) -> Vec<Doc> {
    let mut agg: HashMap<String, (Doc, f64)> = HashMap::new();
    for (lst, w) in lists {
        for (rank, d) in lst.iter().enumerate() {
            let e = agg
                .entry(d.session_id.clone())
                .or_insert_with(|| (d.clone(), 0.0));
            e.1 += w / (k + (rank as f64 + 1.0));
        }
    }
    let top = agg
        .values()
        .map(|(_, s)| *s)
        .fold(0.0_f64, f64::max)
        .max(f64::MIN_POSITIVE);
    let mut out: Vec<Doc> = agg
        .into_values()
        .map(|(mut d, s)| {
            d.score = round3(s / top);
            d
        })
        .collect();
    out.sort_by(|a, b| b.score.total_cmp(&a.score));
    out
}

// ---- LLM query expansion (only the query is sent, never session content) ----
const EXPAND_PROMPT: &str = "为全文检索做查询扩展。给定查询，输出与它语义相关的检索关键词：\
    同义词、近义词、相关术语、中英文对照都要。只输出关键词本身，用英文逗号分隔，\
    不要解释、不要编号、不要重复原词太多。查询：";

fn llm_expand(query: &str) -> Vec<String> {
    let prompt = format!("{EXPAND_PROMPT}{query}");
    let mut cmd = match std::env::var("CCS_EXPAND_CMD") {
        Ok(c) if !c.is_empty() => {
            // custom backend: prompt on stdin, comma/newline-separated terms on stdout
            let mut b = if cfg!(windows) {
                let mut b = Command::new("cmd");
                b.arg("/C").arg(&c);
                b
            } else {
                let mut b = Command::new("sh");
                b.arg("-c").arg(&c);
                b
            };
            b.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            b
        }
        _ => {
            // --no-session-persistence: don't let the expansion call create a resumable
            // session that would pollute the index with the query terms.
            let mut b = Command::new("claude");
            b.args([
                "-p",
                "--model",
                "claude-haiku-4-5-20251001",
                "--strict-mcp-config",
                "--no-session-persistence",
            ]);
            b.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            b
        }
    };

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(prompt.as_bytes());
    }
    let out = match child.wait_with_output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&out.stdout);
    text.split([',', '，', '、', '\n'])
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .take(40)
        .collect()
}

/// Cached LLM expansion: query -> related terms (paid once per unique query).
pub fn expand_query(conn: &Connection, query: &str) -> Vec<String> {
    if let Ok(json) = conn.query_row(
        "SELECT terms FROM expansions WHERE query=?1",
        [query],
        |r| r.get::<_, String>(0),
    ) {
        if let Ok(cached) = serde_json::from_str::<Vec<String>>(&json) {
            if !cached.is_empty() {
                return cached; // trust only non-empty cached expansions
            }
        }
    }
    let terms = llm_expand(query);
    if !terms.is_empty() {
        // never cache a failure (timeout / not logged in), so it retries next time
        let json = serde_json::to_string(&terms).unwrap_or_else(|_| "[]".into());
        let _ = conn.execute(
            "INSERT OR REPLACE INTO expansions(query,terms,ts) VALUES(?1,?2,?3)",
            rusqlite::params![query, json, now_secs() as f64],
        );
    }
    terms
}

pub fn is_expansion_cached(conn: &Connection, query: &str) -> bool {
    conn.query_row("SELECT 1 FROM expansions WHERE query=?1", [query], |_| {
        Ok(())
    })
    .is_ok()
}
