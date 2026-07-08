# Reddit

**Norms:** Each subreddit hates cross-posted copy-paste. Tailor the angle per sub, read the rules (some require flair or ban "self-promo" outside a weekly thread). Lead with value, link at the end, reply to every comment. Space posts out over days, don't blast all three at once.

---

## r/rust

**Title:**
```
ccsearch: a single-binary Rust CLI that full-text searches and resumes your Claude Code sessions (SQLite FTS5 + BM25, CJK-aware)
```

**Body:**
```
I built a small Rust tool to scratch my own itch: searching my Claude Code
history. It's a single static binary — rusqlite with the `bundled` feature bakes
SQLite + FTS5 into the binary so there's no runtime dependency on any platform.

Rust-relevant bits:
- rusqlite (bundled SQLite/FTS5), clap (derive), serde_json, dirs. That's the
  whole dependency tree.
- Custom FTS5 tokenization for CJK: overlapping bigrams at index + query time so
  2-character Chinese queries match, ASCII stays whole-word + prefix.
- Incremental indexing: only changed session files are re-parsed each run; warm
  searches are sub-50ms.
- release profile: strip + lto + codegen-units=1 + panic=abort for a lean binary.

Source (MIT): https://github.com/yaalsn/ccsearch
Happy to talk about the FTS5 tokenizer or the ranking if anyone's interested.
```

---

## r/commandline

**Title:**
```
ccs — fuzzy full-text search over your Claude Code history, then jump back into any session in one keystroke
```

**Body:**
```
`claude --resume` only lists AI-generated titles. ccs indexes the full text of
every session and ranks with BM25, then resumes the one you pick — your shell
cd's into the original project directory and runs `claude --resume`, and stays
there after you quit.

    ccs logo pcb silkscreen      # ranked results, pick a number
    ccs --here <query>           # only sessions started in this dir
    ccs --json <query>           # machine-readable
    ccs 全文 检索                 # CJK works, even 2-char queries

Cross-shell (zsh/bash/fish/PowerShell) via a tiny `ccs init <shell>` function,
single binary, 100% local. https://github.com/yaalsn/ccsearch
```

---

## r/ClaudeAI

**Title:**
```
I made a tool to search + instantly resume any past Claude Code session by what you actually did in it
```

**Body:**
```
If you live in Claude Code, you've hit this: `--resume` shows a list of titles,
and you can't find the session where you fixed that bug / designed that schema
last week.

ccsearch indexes the full text of every session — your prompts, Claude's
replies, the commands it ran, the files it touched, even subagent transcripts —
and ranks with BM25. Type what you remember, pick a result, and you're back in
that project directory with the conversation restored.

100% local (your sessions never leave your machine). There's an optional
semantic-recall mode for when you can't remember the exact words.

https://github.com/yaalsn/ccsearch — feedback very welcome, this community is
exactly who I built it for.
```
