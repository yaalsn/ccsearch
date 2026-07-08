# 掘金 (Juejin)

**发文规范：** 技术长文在掘金和搜索引擎里都吃香（利于 GEO/SEO）。配一张封面图，选 2–3 个标签。正文要有代码块、有「怎么实现的」干货，不要纯安利。

**建议标签：** `Rust`、`命令行工具`、`AI`

---

## 标题

```
我把整个 Claude Code 历史做成了全文搜索：BM25 排序 + 一键恢复会话
```

## 正文（Markdown）

```markdown
`claude --resume` 只会列出一堆 **AI 生成的标题**。标题没认出来，就只能干翻。
可你明明记得上周调过那个偶发失败的测试、设计过那张表、画过那块板子——就是想不起
是哪个会话。

所以我写了 **ccsearch**（命令 `ccs`）：对整个 Claude Code 历史做全文、BM25 排序的
搜索，选中后一键恢复会话。

## 用起来是这样的

```console
$ ccs logo pcb silkscreen
[1] Design a logo-shaped PCB business card   (1.03)
    cc-logo-pcb · 4d ago · 96 msgs · main
    …make the silkscreen match the logo outline and route the LED traces…
    resume: cd ~/code/cc-logo-pcb && claude --resume 7f3a1c8e-…

Resume which? [1-6, Enter=cancel] 1
# → 直接回到 ~/code/cc-logo-pcb，会话完整恢复
```

## 原理

每个 Claude Code 会话都是一个 JSONL 文件，位于
`~/.claude/projects/<编码后的cwd>/<session-id>.jsonl`。ccsearch：

1. **索引**：把每个会话的文本（含 `subagents/*.jsonl`）建进 SQLite **FTS5**。
   每次运行增量刷新——只重新解析改动过的文件——所以结果始终最新。
2. **排序**：BM25，按列加权 标题/提问/正文 = 100/10/1，让标题命中压过正文命中，
   再按查询归一化并加一点点时间新鲜度做 tiebreak。
3. **恢复**：重新编码出正确的启动目录，交给 shell 执行 `cd` + `claude --resume`。

## 最有意思的部分：中文搜索

FTS5 自带分词器搞不定中文子串检索——`trigram` 要 ≥3 个字，`unicode61` 会把一整串
汉字当成一个 token。于是 ccsearch 在**索引和查询时**都跑自己的分析器：中文串切成
重叠的**二元组**（`全文检索` → `全文 文检 检索`），两个字的查询（比如 `检索`）也能命中；
英文保持整词，并按前缀查询。

## 部署很省心

单个 Rust 静态二进制——`rusqlite` 的 `bundled` 特性把 SQLite + FTS5 直接编进去，
任何平台都零运行时依赖。热搜索 <50ms。100% 本地；唯一可能离开本机的，是你主动开启
语义召回（`-e`）时的那句查询词。

## 安装

```bash
cargo install ccsearch          # 二进制名是 ccs
eval "$(ccs init zsh)"          # shell 集成（或 bash/fish/powershell）
```

仓库 + 文档：https://github.com/yaalsn/ccsearch（MIT）。如果你也天天用 Claude Code，
很想听听排序有没有帮你把对的会话顶上来。
```
