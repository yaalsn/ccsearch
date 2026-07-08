# Twitter / X

**Norms:** Hook in the first tweet (it's the only one most people see). Thread the details. One clear CTA. Put the link in a later tweet or reply so the algorithm doesn't suppress the first. Add the demo GIF/SVG to tweet 1.

---

## Thread

**1/ (hook + media)**
```
claude --resume only shows AI-generated titles.

You KNOW you debugged that flaky test last week — but which session was it?

I built ccsearch: full-text, BM25-ranked search over your entire Claude Code
history. Type what you remember → jump back in. 🧵
```
*(attach assets/demo.svg or a screen recording)*

**2/**
```
It indexes the full text of every session — your prompts, Claude's replies, the
commands it ran, the files it touched, even subagent transcripts — into SQLite
FTS5 and ranks with BM25.

Not titles. What actually happened.
```

**3/**
```
Pick a result and your shell cd's into the original project directory and runs
`claude --resume` — and stays there after you quit.

One keystroke, back in the conversation.
```

**4/**
```
It's Chinese/CJK-aware too. FTS5 can't do Chinese substring search, so ccsearch
runs its own bigram analyzer — 全文检索 → 全文 文检 检索 — and 2-char queries just work.
```

**5/ (CTA + link)**
```
Single static Rust binary. Sub-50ms warm search. 100% local — your sessions
never leave your machine.

cargo install ccsearch

Repo → https://github.com/yaalsn/ccsearch ⭐
```

---

## Single-tweet version (if not threading)

```
Lost a Claude Code session you KNOW you had? `claude --resume` only shows titles.

ccsearch: full-text BM25 search over your whole history → resume any session in
one keystroke. Rust, single binary, CJK-aware, 100% local.

cargo install ccsearch
https://github.com/yaalsn/ccsearch
```
