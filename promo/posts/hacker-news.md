# Hacker News — Show HN

**Norms:** No marketing voice. First-person, honest, technical. The best Show HN posts explain what you built, why, and one interesting technical detail, then get out of the way. Be present in the comments for the first few hours. Post Tue–Thu, ~8–10am ET for best visibility.

---

## Title (≤80 chars)

```
Show HN: ccsearch – full-text search and resume for your Claude Code sessions
```

## URL

```
https://github.com/yaalsn/ccsearch
```

## Text (first comment / "text" field)

```
I use Claude Code all day and kept losing conversations. `claude --resume` only
shows AI-generated titles, so if I didn't recognize the title I was stuck
scrolling — even though I clearly remembered debugging that flaky test or routing
that PCB last week.

ccsearch indexes the full text of every session (prompts, Claude's replies, the
commands it ran, the files it touched, even subagent transcripts) into SQLite
FTS5 and ranks matches with BM25. You run `ccs <what you remember>`, pick a
result, and your shell cd's into the original project directory and runs
`claude --resume` on that session.

A couple of things that were more work than expected:

- CJK search. FTS5 can't do Chinese substring search out of the box — trigram
  needs ≥3 chars and unicode61 treats a whole run of Han characters as one token.
  So ccsearch runs its own analyzer at index and query time: CJK runs become
  overlapping bigrams (全文检索 → 全文 文检 检索) so a 2-char query like 检索 matches,
  while ASCII stays whole words and is queried as a prefix.

- Ranking. Column-weighted BM25 (title/prompt/body 100/10/1) so a title hit beats
  a body hit even for common terms, normalized per query with a small recency
  tiebreak.

It's a single static Rust binary (SQLite + FTS5 compiled in), incremental
reindex on every run (~30ms warm), and 100% local — nothing leaves your machine
except the short query text if you opt into the semantic-recall flag.

Repo: https://github.com/yaalsn/ccsearch
Would love feedback, especially on the ranking and the CJK analyzer.
```

**Comment-ready follow-ups (keep in your back pocket):**
- If asked "why not ripgrep over the JSONL?" → rg is great but has no ranking, no CJK bigram handling, no resume, and JSONL has a lot of noise (tool output, base64) that tanks precision; ccsearch parses structure and weights columns.
- If asked about privacy → indexing/search fully local; `-e` sends only the query string, and only when used; you can point it at a local model via `CCS_EXPAND_CMD`.
