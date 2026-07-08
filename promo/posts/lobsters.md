# Lobste.rs

**Norms:** Invite-only, low tolerance for self-promo. Only post if you have an account in good standing. Tag accurately. The `authored_by_submitter` checkbox applies since you wrote it. Keep the "more text" short and technical. Suggested tags: `rust`, `release`.

---

**URL:**
```
https://github.com/yaalsn/ccsearch
```

**Title:**
```
ccsearch: full-text BM25 search and resume for Claude Code sessions (Rust, SQLite FTS5)
```

**Tags:** `rust`

**More text (optional):**
```
Single-binary Rust CLI. Indexes Claude Code session JSONL into SQLite FTS5 and
ranks with column-weighted BM25. The tokenizer is custom because FTS5 can't do
CJK substring search — CJK runs become overlapping bigrams at index and query
time so 2-char queries work, ASCII stays whole-word + prefix. Author here, happy
to discuss the tokenizer or ranking.
```
