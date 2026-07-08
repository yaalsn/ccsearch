# dev.to

**Norms:** Long-form article ranks well in Google and gets picked up by AI engines (great for GEO). Use a canonical_url pointing at your own site if you cross-post. Front-matter tags max 4. Add a cover image.

**Suggested tags:** `rust`, `cli`, `productivity`, `ai`

---

## Front matter

```yaml
---
title: "I lost too many Claude Code sessions, so I built full-text search for them"
published: false
description: "ccsearch indexes your entire Claude Code history and ranks it with BM25, so you can find and resume any conversation by what actually happened in it."
tags: rust, cli, productivity, ai
canonical_url: https://yaalsn.github.io/ccsearch/
cover_image: https://raw.githubusercontent.com/yaalsn/ccsearch/main/assets/demo.svg
---
```

## Body (Markdown)

```markdown
`claude --resume` only shows you a list of **AI-generated titles**. If you don't
recognize the title, you're stuck scrolling. But you *know* you debugged that
flaky test / designed that schema / routed that PCB last week — you just can't
remember which session it was.

So I built **ccsearch** (`ccs`): full-text, BM25-ranked search over your entire
Claude Code history, with one-keystroke resume.

## What it feels like

```console
$ ccs logo pcb silkscreen
[1] Design a logo-shaped PCB business card   (1.03)
    cc-logo-pcb · 4d ago · 96 msgs · main
    …make the silkscreen match the logo outline and route the LED traces…
    resume: cd ~/code/cc-logo-pcb && claude --resume 7f3a1c8e-…

Resume which? [1-6, Enter=cancel] 1
# → you're now in ~/code/cc-logo-pcb with the full conversation restored
```

## How it works

Every Claude Code session is a JSONL file under
`~/.claude/projects/<encoded-cwd>/<session-id>.jsonl`. ccsearch:

1. **Indexes** each session's text (folding in `subagents/*.jsonl`) into a SQLite
   **FTS5** database. It refreshes incrementally on every run — only changed
   files are re-parsed — so results are always current.
2. **Ranks** with BM25, column-weighted title/prompt/body 100/10/1 so a title
   hit beats a body hit, normalized per query with a small recency tiebreak.
3. **Resumes** by re-encoding the launch directory and handing your shell a
   `cd` + `claude --resume`.

## The interesting bit: Chinese/CJK search

FTS5's built-in tokenizers can't do Chinese substring search — `trigram` needs
≥3 characters and `unicode61` treats a whole run of Han characters as one token.
So ccsearch runs its own analyzer at index *and* query time: CJK runs become
overlapping **bigrams** (`全文检索` → `全文 文检 检索`), so a 2-character query like
`检索` matches; ASCII stays whole words and is queried as a prefix.

## It's boringly deployable

Single static Rust binary — `rusqlite` with the `bundled` feature compiles
SQLite + FTS5 right in, so there's no runtime dependency anywhere. Warm searches
are sub-50ms. It's 100% local; the only thing that ever leaves your machine is
the short query text if you opt into semantic recall (`-e`).

## Install

```bash
cargo install ccsearch          # binary is `ccs`
eval "$(ccs init zsh)"          # shell integration (or bash/fish/powershell)
```

Repo + docs: https://github.com/yaalsn/ccsearch (MIT). If you use Claude Code, I'd
love to know whether the ranking surfaces the right session for you.
```
