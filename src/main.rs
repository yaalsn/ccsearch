//! ccsearch (`ccs`) — fuzzy full-text search & instant resume for Claude Code sessions.

mod db;
mod resume;
mod search;
mod shell;
mod text;

use clap::Parser;
use search::Doc;
use std::io::{IsTerminal, Write};
use text::{bold, cyan, dim, green, rel_date, snippet};

#[derive(Parser)]
#[command(
    name = "ccs",
    bin_name = "ccs",
    about = "Fuzzy full-text search & instant resume for your Claude Code sessions.",
    version
)]
struct Cli {
    /// Search terms (Chinese/CJK and English both work)
    query: Vec<String>,
    /// Query (alternate form)
    #[arg(short = 'q', long = "query")]
    q2: Option<String>,
    /// Max results
    #[arg(short = 'n', long = "limit", default_value_t = 15)]
    limit: usize,
    /// Only sessions started in the current directory
    #[arg(long)]
    here: bool,
    /// Print results, don't prompt to resume
    #[arg(long = "print")]
    print_only: bool,
    /// Machine-readable output
    #[arg(long)]
    json: bool,
    /// Force a full rebuild of the index
    #[arg(long)]
    reindex: bool,
    /// Resume WITHOUT --dangerously-skip-permissions
    #[arg(long)]
    safe: bool,
    /// Wrapper mode: write the chosen resume target to this file instead of launching
    #[arg(long = "emit-file")]
    emit_file: Option<String>,
    /// Semantic recall: expand the query via Haiku (cached)
    #[arg(short = 'e', long)]
    expand: bool,
    /// Do not auto-expand when a search finds nothing
    #[arg(long = "no-auto")]
    no_auto: bool,
}

fn ignore_dirs() -> Vec<String> {
    std::env::var("CCS_IGNORE")
        .unwrap_or_else(|_| "/private/tmp/,/tmp/".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn main() {
    // `ccs init <shell>` is handled before clap so a positional query never collides.
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if raw.first().map(String::as_str) == Some("init") {
        print!(
            "{}",
            shell::init_script(raw.get(1).map(String::as_str).unwrap_or(""))
        );
        return;
    }

    let cli = Cli::parse();
    text::set_color(std::io::stdout().is_terminal());

    let conn = db::connect();
    db::reindex(&conn, cli.reindex, !cli.json);

    let query = if !cli.query.is_empty() {
        cli.query.join(" ")
    } else {
        cli.q2.clone().unwrap_or_default()
    };
    if query.trim().is_empty() {
        if cli.reindex {
            println!(
                "Indexed {} sessions at {}",
                db::count_sessions(&conn),
                db::db_path().display()
            );
        } else {
            println!("usage: ccs <terms>   (try: ccs --help, or `ccs init <shell>`)");
        }
        return;
    }

    let here: Option<String> = if cli.here {
        std::env::current_dir()
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        None
    };
    let ignore = ignore_dirs();
    let terms: Vec<String> = query.split_whitespace().map(String::from).collect();
    let match_expr = text::to_match(&query);

    let strict = search::run_match(&conn, &match_expr, here.as_deref(), &ignore, cli.limit);
    let mut results: Vec<Doc> = strict.iter().take(cli.limit).cloned().collect();
    let mut hl_terms = terms.clone();

    // Expand via LLM when forced (-e) or when an exact search of a real query found nothing.
    if cli.expand || (results.is_empty() && !cli.no_auto && !match_expr.is_empty()) {
        if !search::is_expansion_cached(&conn, &query) {
            let note = if results.is_empty() {
                "no exact matches — "
            } else {
                ""
            };
            eprintln!(
                "{}",
                dim(&format!(
                    "{note}expanding query via Haiku (~5s, cached after)…"
                ))
            );
        }
        let related = search::expand_query(&conn, &query);
        if !related.is_empty() {
            let shown: Vec<&str> = related.iter().take(12).map(String::as_str).collect();
            let ell = if related.len() > 12 { "…" } else { "" };
            eprintln!("{}", dim(&format!("↳ {}{}", shown.join(", "), ell)));
            let exp = search::run_match(
                &conn,
                &text::to_match_or(&related),
                here.as_deref(),
                &ignore,
                cli.limit,
            );
            let fused = search::rrf_fuse(&[(&strict, 1.6), (&exp, 1.0)], 60.0);
            results = fused.into_iter().take(cli.limit).collect();
            hl_terms = terms
                .iter()
                .cloned()
                .chain(related.iter().take(15).cloned())
                .collect();
        } else {
            eprintln!(
                "{}",
                dim("(expansion unavailable — showing exact matches only)")
            );
        }
    }

    if cli.json {
        print_json(&results);
        return;
    }
    if results.is_empty() {
        println!("No sessions match: {query}");
        if here.is_some() {
            println!(
                "{}",
                dim("(searched current project only; drop --here to search all projects)")
            );
        }
        return;
    }

    render(&results, &hl_terms);

    if cli.print_only || !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return;
    }
    eprint!(
        "{}",
        bold(&format!(
            "Resume which? [1-{}, Enter=cancel] ",
            results.len()
        ))
    );
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_err() {
        return;
    }
    let idx: usize = match line.trim().parse() {
        Ok(n) => n,
        Err(_) => return,
    };
    if idx >= 1 && idx <= results.len() {
        let chosen = &results[idx - 1];
        let danger = !cli.safe;
        match &cli.emit_file {
            Some(f) => resume::emit_resume(chosen, danger, f),
            None => resume::do_resume(chosen, danger),
        }
    }
}

fn render(results: &[Doc], hl_terms: &[String]) {
    for (i, d) in results.iter().enumerate() {
        let head = if !d.title.is_empty() {
            d.title.clone()
        } else if !d.last_prompt.is_empty() {
            format!("{}…", d.last_prompt.chars().take(80).collect::<String>())
        } else {
            "(untitled)".to_string()
        };
        let proj = std::path::Path::new(&d.cwd)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| d.project_dir.clone());
        let mut meta = format!("{} · {} · {} msgs", proj, rel_date(&d.last_ts), d.msg_count);
        if !d.branch.is_empty() {
            meta.push_str(&format!(" · {}", d.branch));
        }
        println!(
            "{} {}  {}",
            bold(&cyan(&format!("[{}]", i + 1))),
            bold(&head),
            dim(&format!("({})", d.score))
        );
        println!("    {}", dim(&meta));
        let snip = snippet(&d.content, hl_terms, 140);
        if !snip.is_empty() {
            println!("    {snip}");
        }
        println!(
            "    {} {}",
            dim("resume:"),
            green(&format!("cd {} && claude --resume {}", d.cwd, d.session_id))
        );
        println!();
    }
}

fn print_json(results: &[Doc]) {
    let arr: Vec<serde_json::Value> = results
        .iter()
        .map(|d| {
            serde_json::json!({
                "session_id": d.session_id,
                "cwd": d.cwd,
                "branch": d.branch,
                "title": d.title,
                "last_prompt": d.last_prompt,
                "last_ts": d.last_ts,
                "msg_count": d.msg_count,
                "project_dir": d.project_dir,
                "score": d.score,
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".into())
    );
}
