//! Resuming a chosen session — either by emitting data for the shell wrapper to
//! act on (so the `cd` persists in the user's shell), or, without a wrapper,
//! chdir + launch `claude` in-process (works, but the dir won't persist).

use crate::search::Doc;
use crate::text::{dim, yellow};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Wrapper mode: write three lines — cwd, session id, danger flag ("1"/"0").
/// Each shell's `ccs` function reads these and does a native cd + `claude --resume`.
pub fn emit_resume(doc: &Doc, danger: bool, emit_file: &str) {
    if !doc.cwd.is_empty() && !Path::new(&doc.cwd).is_dir() {
        eprintln!(
            "{}",
            yellow(&format!(
                "warning: original dir {} no longer exists; resume may fail",
                doc.cwd
            ))
        );
    }
    let payload = format!(
        "{}\n{}\n{}\n",
        doc.cwd,
        doc.session_id,
        if danger { "1" } else { "0" }
    );
    let _ = fs::write(emit_file, payload);
    eprintln!(
        "{}",
        dim(&format!(
            "→ cd {} && claude --resume {}",
            doc.cwd, doc.session_id
        ))
    );
}

/// Standalone mode (no shell wrapper): chdir + launch claude as a child, inherit
/// stdio. Resume works, but the parent shell can't be moved — warn about it.
pub fn do_resume(doc: &Doc, danger: bool) -> ! {
    eprintln!(
        "{}",
        yellow(
            "note: your shell will return to its current dir after you quit Claude. For it to \
             stay in the session's dir, load the ccs function (see `ccs init <shell>`)."
        )
    );
    if !doc.cwd.is_empty() {
        if Path::new(&doc.cwd).is_dir() {
            let _ = std::env::set_current_dir(&doc.cwd);
        } else {
            eprintln!(
                "{}",
                yellow(&format!(
                    "warning: original dir {} no longer exists; resume may fail",
                    doc.cwd
                ))
            );
        }
    }
    let mut cmd = Command::new("claude");
    cmd.arg("--resume").arg(&doc.session_id);
    if danger {
        cmd.arg("--dangerously-skip-permissions");
    }
    eprintln!(
        "{}",
        dim(&format!(
            "→ (cd {}) claude --resume {}",
            doc.cwd, doc.session_id
        ))
    );
    match cmd.status() {
        Ok(st) => std::process::exit(st.code().unwrap_or(0)),
        Err(_) => {
            eprintln!("claude not found on PATH");
            std::process::exit(1);
        }
    }
}
