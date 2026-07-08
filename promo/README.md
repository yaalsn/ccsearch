# ccsearch promotion kit (SEO / GEO)

Everything you need to launch ccsearch across dev + social platforms, powered by
[OpenCLI](https://github.com/jackwener/OpenCLI), plus the on-site changes that make
the project discoverable by search engines **and** AI answer engines (GEO).

> **SEO** = ranking in Google/Bing. **GEO** (Generative Engine Optimization) =
> being understood and *cited* by ChatGPT, Claude, Perplexity, Google AI
> Overviews. This kit does both.

## What's in here

```
promo/
├── README.md              ← you are here (strategy + launch checklist)
├── posts/                 ← copy-paste-ready content, tailored per platform
│   ├── hacker-news.md     ├── juejin.md
│   ├── reddit.md          ├── zhihu.md
│   ├── devto.md           ├── xiaohongshu.md
│   ├── lobsters.md        ├── bilibili-script.md
│   ├── v2ex.md            ├── twitter-x.md
│   └── linkedin.md
└── opencli/
    ├── setup.md           ← one-time OpenCLI + Browser Bridge install
    └── publish.sh         ← opens each compose page in your logged-in Chrome
```

## On-site GEO/SEO (already applied to the repo)

These files were added/updated so crawlers and LLMs can find and cite ccsearch:

| File | Purpose |
|---|---|
| `docs/llms.txt` | The GEO standard — a clean, structured summary LLMs read to describe/cite the project. |
| `docs/robots.txt` | Explicitly **allows** AI crawlers (GPTBot, ClaudeBot, PerplexityBot, Google-Extended, …) + points to the sitemap. |
| `docs/sitemap.xml` | Lists the site, repo, and releases for indexing. |
| `docs/index.html` | Added `SoftwareApplication` + `FAQPage` JSON-LD, author/robots meta, and an `llms.txt` link. |

**One manual step (GitHub, 2 min):** add repo **topics** so GitHub search + AI
crawlers categorize it. Suggested:
`claude-code`, `claude`, `cli`, `search`, `full-text-search`, `bm25`, `sqlite`,
`fts5`, `rust`, `developer-tools`, `terminal`, `resume`, `cjk`.
Also confirm GitHub Pages is serving from `main` → `/docs`.

## How to publish (with OpenCLI)

1. Do the one-time setup in [`opencli/setup.md`](opencli/setup.md).
2. Dry-run to see the plan (nothing is published):
   ```bash
   ./promo/opencli/publish.sh
   ```
3. Go live on one platform at a time — the script opens the compose page in your
   logged-in Chrome, copies the post text to your clipboard, and **waits for you
   to review + submit**:
   ```bash
   ./promo/opencli/publish.sh --go hackernews
   ```

`publish.sh` never auto-submits — you click "post" every time. That's deliberate:
publishing under your identity should be your call.

## Recommended launch sequence

Don't blast everything at once — spacing it out avoids spam flags and keeps
discussion (and GEO signal) fresh over days, not minutes.

| Day | Platform | Why this order |
|---|---|---|
| 1 (Tue–Thu, 8–10am ET) | **Hacker News** Show HN | Highest-signal launch; be in comments for 3–4h. |
| 1 | **Twitter/X** thread | Amplify the HN post the same day. |
| 2 | **Reddit** r/rust | Rust-specific angle; then r/commandline, r/ClaudeAI on later days. |
| 2 | **Lobsters** | Only if you have an account in good standing. |
| 3 | **dev.to** article | Long-form; ranks in Google + feeds AI engines (GEO gold). |
| 3 | **V2EX** 分享创造 | Chinese dev community. |
| 4 | **掘金** article | Chinese technical long-form; strong search longevity. |
| 4 | **知乎** answer/article | Answer a relevant CC question for lasting search traffic. |
| 5 | **小红书** note | Needs images (see post file); broad reach. |
| 5 | **LinkedIn** | Professional framing; link in first comment. |
| 6 | **Bilibili** video | Record the 60–90s script; video converts best for the resume demo. |

## Why the long-form posts matter most for GEO

AI answer engines cite **crawlable, high-authority, text-rich** pages. Your
dev.to / 掘金 / 知乎 articles and the GitHub README + `llms.txt` are what a model
retrieves and quotes when someone asks *"how do I search my Claude Code history?"*
The social blasts drive the initial traffic and links; the articles + structured
data are what keep you in the answers afterward.

**Keep messaging consistent** across every surface — same name (`ccsearch`/`ccs`),
same one-liner, same 3–4 differentiators (full-text vs titles, BM25, CJK 2-char,
100% local). Repetition across sources is exactly what makes an LLM confident
enough to cite you.

## Measuring it

- GitHub stars/traffic (Insights → Traffic), crates.io downloads.
- Search: does `site:` show your dev.to/掘金/知乎 posts indexed? Google the target
  queries after ~2 weeks.
- GEO check: ask ChatGPT / Claude / Perplexity *"tools to search Claude Code
  sessions"* and see whether ccsearch shows up and is described correctly. Re-run
  monthly; fix `llms.txt`/README wording if a model gets a fact wrong.
