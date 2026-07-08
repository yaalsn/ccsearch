//! SQLite FTS5 index: schema, session JSONL parsing, incremental reindex.

use crate::text::tokenized;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA: &str = "
  CREATE TABLE IF NOT EXISTS sessions(
    id INTEGER PRIMARY KEY, session_id TEXT, file_path TEXT UNIQUE, project_dir TEXT,
    cwd TEXT, git_branch TEXT, title TEXT, last_prompt TEXT, content TEXT,
    first_ts TEXT, last_ts TEXT, mtime REAL, msg_count INTEGER, user_count INTEGER);
  CREATE VIRTUAL TABLE IF NOT EXISTS fts USING fts5(title, last_prompt, content, tokenize='unicode61');
  CREATE TABLE IF NOT EXISTS expansions(query TEXT PRIMARY KEY, terms TEXT, ts REAL);
";

pub fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}
pub fn projects_dir() -> PathBuf {
    home().join(".claude").join("projects")
}
pub fn db_path() -> PathBuf {
    home().join(".claude").join("ccsearch-index.db")
}

/// cwd -> Claude Code project-dir name: '/' and '.' both become '-'.
pub fn encode_project(path: &str) -> String {
    path.chars()
        .map(|c| if c == '/' || c == '.' { '-' } else { c })
        .collect()
}
fn decode_project(name: &str) -> String {
    if let Some(rest) = name.strip_prefix('-') {
        format!("/{}", rest.replace('-', "/"))
    } else {
        name.to_string()
    }
}

pub struct Record {
    pub session_id: String,
    pub project_dir: String,
    pub cwd: String,
    pub git_branch: String,
    pub title: String,
    pub last_prompt: String,
    pub content: String,
    pub first_ts: String,
    pub last_ts: String,
    pub msg_count: i64,
    pub user_count: i64,
}

// ---- text extraction --------------------------------------------------------
fn blocks_text(content: &Value) -> String {
    if let Some(s) = content.as_str() {
        return s.to_string();
    }
    let arr = match content.as_array() {
        Some(a) => a,
        None => return String::new(),
    };
    let mut out = Vec::new();
    for b in arr {
        let t = b.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match t {
            "text" => {
                if let Some(x) = b.get("text").and_then(|v| v.as_str()) {
                    out.push(x.to_string());
                }
            }
            "thinking" => {
                if let Some(x) = b.get("thinking").and_then(|v| v.as_str()) {
                    out.push(x.to_string());
                }
            }
            "tool_use" => {
                if let Some(inp) = b.get("input").and_then(|v| v.as_object()) {
                    for k in [
                        "command",
                        "file_path",
                        "path",
                        "pattern",
                        "query",
                        "url",
                        "description",
                        "prompt",
                    ] {
                        if let Some(v) = inp.get(k).and_then(|v| v.as_str()) {
                            out.push(v.to_string());
                        }
                    }
                }
            }
            "tool_result" => {
                if let Some(s) = b.get("content").and_then(|v| v.as_str()) {
                    out.push(truncate(s, 500));
                } else if let Some(ra) = b.get("content").and_then(|v| v.as_array()) {
                    for rb in ra {
                        if rb.get("type").and_then(|v| v.as_str()) == Some("text") {
                            if let Some(x) = rb.get("text").and_then(|v| v.as_str()) {
                                out.push(truncate(x, 500));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out.join("\n")
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}

pub fn parse_session(fp: &Path) -> Option<Record> {
    let raw = fs::read(fp).ok()?;
    let raw = String::from_utf8_lossy(&raw);
    let mut title: Option<String> = None;
    let mut last_prompt: Option<String> = None;
    let mut branch: Option<String> = None;
    let mut cwds: Vec<String> = Vec::new(); // distinct cwds in order (session may cd around)
    let (mut first_ts, mut last_ts) = (String::new(), String::new());
    let (mut msg_count, mut user_count) = (0i64, 0i64);
    let mut sid = fp.file_stem()?.to_string_lossy().to_string();
    let mut parts: Vec<String> = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let o: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(s) = o.get("sessionId").and_then(|v| v.as_str()) {
            sid = s.to_string();
        }
        match o.get("type").and_then(|v| v.as_str()).unwrap_or("") {
            "ai-title" => {
                if let Some(x) = o.get("aiTitle").and_then(|v| v.as_str()) {
                    title = Some(x.to_string());
                }
            }
            "last-prompt" => {
                if let Some(x) = o.get("lastPrompt").and_then(|v| v.as_str()) {
                    last_prompt = Some(x.to_string());
                }
            }
            "summary" => {
                if let Some(x) = o.get("summary").and_then(|v| v.as_str()) {
                    parts.push(x.to_string());
                }
            }
            "user" | "assistant" => {
                msg_count += 1;
                let is_user = o.get("type").and_then(|v| v.as_str()) == Some("user");
                if let Some(c) = o.get("cwd").and_then(|v| v.as_str()) {
                    if cwds.last().map(String::as_str) != Some(c) {
                        cwds.push(c.to_string());
                    }
                }
                if let Some(b) = o.get("gitBranch").and_then(|v| v.as_str()) {
                    branch = Some(b.to_string());
                }
                if let Some(ts) = o.get("timestamp").and_then(|v| v.as_str()) {
                    if first_ts.is_empty() {
                        first_ts = ts.to_string();
                    }
                    last_ts = ts.to_string();
                }
                let txt = o
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .map(blocks_text)
                    .unwrap_or_default();
                if is_user {
                    user_count += 1;
                    if last_prompt.is_none()
                        && !txt.is_empty()
                        && !txt.trim_start().starts_with('<')
                    {
                        last_prompt = Some(truncate(&txt, 2000));
                    }
                }
                if !txt.is_empty() {
                    parts.push(txt);
                }
            }
            _ => {}
        }
    }

    fold_subagents(fp, &mut parts);

    let mut content = parts.join("\n");
    if content.len() > 800_000 {
        // truncate on a char boundary
        let mut end = 800_000;
        while end > 0 && !content.is_char_boundary(end) {
            end -= 1;
        }
        content.truncate(end);
    }

    let proj = fp.parent()?.file_name()?.to_string_lossy().to_string();
    // resume from the cwd whose encoding matches where the file lives (the launch
    // dir) — NOT a cwd the session cd'd into later (would map to a missing project dir).
    let resume_cwd = cwds
        .iter()
        .find(|c| encode_project(c) == proj)
        .cloned()
        .or_else(|| cwds.first().cloned())
        .unwrap_or_else(|| decode_project(&proj));

    Some(Record {
        session_id: sid,
        project_dir: proj,
        cwd: resume_cwd,
        git_branch: branch.unwrap_or_default(),
        title: title.unwrap_or_default(),
        last_prompt: last_prompt.unwrap_or_default(),
        content,
        first_ts,
        last_ts,
        msg_count,
        user_count,
    })
}

fn fold_subagents(fp: &Path, parts: &mut Vec<String>) {
    let stem = match fp.file_stem() {
        Some(s) => s,
        None => return,
    };
    let sub_dir = fp.with_file_name(stem).join("subagents");
    let entries = match fs::read_dir(&sub_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("jsonl") {
            continue;
        }
        let raw = match fs::read(&p) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for line in String::from_utf8_lossy(&raw).lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(o) = serde_json::from_str::<Value>(line) {
                let t = o.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if t == "user" || t == "assistant" {
                    let st = o
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .map(blocks_text)
                        .unwrap_or_default();
                    if !st.is_empty() {
                        parts.push(st);
                    }
                }
            }
        }
    }
}

// ---- connection & schema ----------------------------------------------------
pub fn connect() -> Connection {
    let conn = Connection::open(db_path()).expect("open index db");
    // migrate away from any pre-FTS 'sessions' table (no `id` column)
    let (exists, has_id) = {
        let mut exists = false;
        let mut has_id = false;
        if let Ok(mut st) = conn.prepare("PRAGMA table_info(sessions)") {
            if let Ok(rows) = st.query_map([], |r| r.get::<_, String>(1)) {
                for name in rows.flatten() {
                    exists = true;
                    if name == "id" {
                        has_id = true;
                    }
                }
            }
        }
        (exists, has_id)
    };
    if exists && !has_id {
        let _ = conn.execute_batch("DROP TABLE IF EXISTS sessions; DROP TABLE IF EXISTS fts;");
    }
    conn.execute_batch(SCHEMA).expect("create schema");
    conn
}

fn fts_write(conn: &Connection, rid: i64, rec: &Record) {
    let _ = conn.execute("DELETE FROM fts WHERE rowid=?1", params![rid]);
    let _ = conn.execute(
        "INSERT INTO fts(rowid,title,last_prompt,content) VALUES(?1,?2,?3,?4)",
        params![
            rid,
            tokenized(&rec.title),
            tokenized(&rec.last_prompt),
            tokenized(&rec.content)
        ],
    );
}

/// Incrementally reindex all sessions; returns (new, updated, removed).
pub fn reindex(conn: &Connection, force: bool, verbose: bool) {
    let mut have: HashMap<String, (i64, f64)> = HashMap::new();
    if let Ok(mut st) = conn.prepare("SELECT file_path, id, mtime FROM sessions") {
        if let Ok(rows) = st.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, f64>(2)?,
            ))
        }) {
            for row in rows.flatten() {
                have.insert(row.0, (row.1, row.2));
            }
        }
    }

    let mut seen = std::collections::HashSet::new();
    let (mut n_new, mut n_upd) = (0i64, 0i64);
    let _ = conn.execute_batch("BEGIN");
    for fp in session_files() {
        let fps = fp.to_string_lossy().to_string();
        seen.insert(fps.clone());
        let mt = match fs::metadata(&fp).and_then(|m| m.modified()) {
            Ok(t) => t
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            Err(_) => continue,
        };
        if !force {
            if let Some((_, old)) = have.get(&fps) {
                if (old - mt).abs() < 1e-6 {
                    continue;
                }
            }
        }
        let rec = match parse_session(&fp) {
            Some(r) => r,
            None => continue,
        };
        if rec.msg_count == 0 && rec.last_prompt.is_empty() {
            continue;
        }
        if let Some((id, _)) = have.get(&fps).copied() {
            let _ = conn.execute(
                "UPDATE sessions SET session_id=?1,project_dir=?2,cwd=?3,git_branch=?4,title=?5,\
                 last_prompt=?6,content=?7,first_ts=?8,last_ts=?9,mtime=?10,msg_count=?11,user_count=?12 WHERE id=?13",
                params![rec.session_id, rec.project_dir, rec.cwd, rec.git_branch, rec.title,
                        rec.last_prompt, rec.content, rec.first_ts, rec.last_ts, mt, rec.msg_count, rec.user_count, id],
            );
            fts_write(conn, id, &rec);
            n_upd += 1;
        } else {
            let _ = conn.execute(
                "INSERT INTO sessions(file_path,session_id,project_dir,cwd,git_branch,title,\
                 last_prompt,content,first_ts,last_ts,mtime,msg_count,user_count) \
                 VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
                params![
                    fps,
                    rec.session_id,
                    rec.project_dir,
                    rec.cwd,
                    rec.git_branch,
                    rec.title,
                    rec.last_prompt,
                    rec.content,
                    rec.first_ts,
                    rec.last_ts,
                    mt,
                    rec.msg_count,
                    rec.user_count
                ],
            );
            fts_write(conn, conn.last_insert_rowid(), &rec);
            n_new += 1;
        }
    }
    // prune deleted files
    let gone: Vec<(String, i64)> = have
        .iter()
        .filter(|(k, _)| !seen.contains(*k))
        .map(|(k, (id, _))| (k.clone(), *id))
        .collect();
    for (_, id) in &gone {
        let _ = conn.execute("DELETE FROM sessions WHERE id=?1", params![id]);
        let _ = conn.execute("DELETE FROM fts WHERE rowid=?1", params![id]);
    }
    let _ = conn.execute_batch("COMMIT");
    if verbose && (n_new > 0 || n_upd > 0 || !gone.is_empty()) {
        eprintln!(
            "{}",
            crate::text::dim(&format!(
                "index: +{} new, {} updated, -{} removed",
                n_new,
                n_upd,
                gone.len()
            ))
        );
    }
}

/// All top-level session files: ~/.claude/projects/<dir>/<id>.jsonl (one level deep).
fn session_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let projects = projects_dir();
    let dirs = match fs::read_dir(&projects) {
        Ok(d) => d,
        Err(_) => return out,
    };
    for d in dirs.flatten() {
        let p = d.path();
        if !p.is_dir() {
            continue;
        }
        if let Ok(files) = fs::read_dir(&p) {
            for f in files.flatten() {
                let fp = f.path();
                if fp.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                    out.push(fp);
                }
            }
        }
    }
    out
}

pub fn count_sessions(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_dir_encoding_roundtrips() {
        // `claude --resume` re-encodes the cwd, so this must match Claude Code's scheme.
        assert_eq!(
            encode_project("/Users/x/code/esim-pcb"),
            "-Users-x-code-esim-pcb"
        );
        assert_eq!(
            encode_project("/Users/x/code/cc-logo-pcb/.claude/worktrees/foo"),
            "-Users-x-code-cc-logo-pcb--claude-worktrees-foo"
        );
        // decode is intentionally lossy (a '-' in a real dir name is unrecoverable);
        // it's only a fallback when a session recorded no cwd.
        assert_eq!(decode_project("-Users-x-code-foo"), "/Users/x/code/foo");
    }

    #[test]
    fn fts_search_end_to_end() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(SCHEMA).unwrap();
        conn.execute(
            "INSERT INTO sessions(file_path,session_id,cwd,project_dir,git_branch,title,last_prompt,\
             content,first_ts,last_ts,mtime,msg_count,user_count) \
             VALUES('f','sid','/tmp','-tmp','','全文检索会话恢复','','x','','2026-06-14T00:00:00.000Z',0,1,1)",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        fts_write(
            &conn,
            id,
            &Record {
                session_id: "sid".into(),
                project_dir: "-tmp".into(),
                cwd: "/tmp".into(),
                git_branch: String::new(),
                title: "全文检索会话恢复".into(),
                last_prompt: String::new(),
                content: "x".into(),
                first_ts: String::new(),
                last_ts: String::new(),
                msg_count: 1,
                user_count: 1,
            },
        );
        // 2-char CJK query must hit via the bigram analyzer
        let hits = crate::search::run_match(&conn, &crate::text::to_match("检索"), None, &[], 5);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].session_id, "sid");
    }
}
