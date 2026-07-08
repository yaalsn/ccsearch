#!/usr/bin/env bash
#
# publish.sh — drive OpenCLI to open the right compose page (in YOUR logged-in
# Chrome) for each platform, so you can paste the prepared post and hit submit.
#
# SAFETY BY DESIGN:
#   * Dry-run by default. It only PRINTS the opencli commands until you pass --go.
#   * Even with --go it stops BEFORE the final submit on every platform — you
#     review and click "post" yourself. Nothing is published without your click.
#   * It never types your content automatically; it opens the page and copies the
#     post file so you paste deliberately.
#
# Usage:
#   ./publish.sh                      # dry-run, all platforms (prints plan)
#   ./publish.sh --go hackernews      # really open the HN submit page
#   ./publish.sh --go reddit twitter  # multiple platforms
#   ./publish.sh --list               # list platform keys
#
# Requires: opencli (see setup.md) and a connected Browser Bridge (opencli doctor).

set -euo pipefail

SESSION="${OPENCLI_SESSION:-main}"
POSTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../posts" && pwd)"
DRY_RUN=1
PLATFORMS=()

# --- platform table: key | compose URL | post file ---------------------------
url_for()  { case "$1" in
  hackernews) echo "https://news.ycombinator.com/submit" ;;
  reddit)     echo "https://www.reddit.com/submit" ;;
  devto)      echo "https://dev.to/new" ;;
  lobsters)   echo "https://lobste.rs/stories/new" ;;
  v2ex)       echo "https://www.v2ex.com/write?node=create" ;;
  juejin)     echo "https://juejin.cn/editor/drafts/new" ;;
  zhihu)      echo "https://zhuanlan.zhihu.com/write" ;;
  xiaohongshu)echo "https://creator.xiaohongshu.com/publish/publish" ;;
  bilibili)   echo "https://member.bilibili.com/platform/upload/video/frame" ;;
  twitter)    echo "https://x.com/compose/post" ;;
  linkedin)   echo "https://www.linkedin.com/feed/" ;;
  *) return 1 ;;
esac }

file_for() { case "$1" in
  hackernews) echo "hacker-news.md" ;;
  reddit)     echo "reddit.md" ;;
  devto)      echo "devto.md" ;;
  lobsters)   echo "lobsters.md" ;;
  v2ex)       echo "v2ex.md" ;;
  juejin)     echo "juejin.md" ;;
  zhihu)      echo "zhihu.md" ;;
  xiaohongshu)echo "xiaohongshu.md" ;;
  bilibili)   echo "bilibili-script.md" ;;
  twitter)    echo "twitter-x.md" ;;
  linkedin)   echo "linkedin.md" ;;
esac }

# native `opencli <platform> <verb>` shortcut, if one exists (else empty)
adapter_for() { case "$1" in
  twitter)    echo "opencli twitter post        # or: thread  (check: opencli twitter post --help)" ;;
  zhihu)      echo "opencli zhihu answer         # for answering a question (check --help)" ;;
  xiaohongshu)echo "opencli xiaohongshu publish  # image/text note (check --help)" ;;
  reddit)     echo "opencli reddit comment       # comments only; new posts use the page below" ;;
  *) echo "" ;;
esac }

ALL=(hackernews reddit devto lobsters v2ex juejin zhihu xiaohongshu bilibili twitter linkedin)

# --- arg parsing -------------------------------------------------------------
for arg in "$@"; do
  case "$arg" in
    --go)   DRY_RUN=0 ;;
    --list) printf '%s\n' "${ALL[@]}"; exit 0 ;;
    -h|--help) sed -n '2,25p' "$0"; exit 0 ;;
    --*)    echo "unknown flag: $arg" >&2; exit 64 ;;
    *)      PLATFORMS+=("$arg") ;;
  esac
done
[ "${#PLATFORMS[@]}" -eq 0 ] && PLATFORMS=("${ALL[@]}")

# --- helpers -----------------------------------------------------------------
run() {  # echo, then run only if not dry-run
  echo "    \$ $*"
  if [ "$DRY_RUN" -eq 0 ]; then "$@"; fi
}

copy_clip() {  # best-effort copy the post file to clipboard for pasting
  local f="$1"
  if   command -v pbcopy   >/dev/null 2>&1; then pbcopy   < "$f"; echo "    (copied $f → clipboard)"
  elif command -v xclip    >/dev/null 2>&1; then xclip -selection clipboard < "$f"; echo "    (copied → clipboard)"
  elif command -v clip.exe >/dev/null 2>&1; then clip.exe < "$f"; echo "    (copied → clipboard)"
  fi
}

pause() {  # human-in-the-loop gate
  if [ "$DRY_RUN" -eq 0 ]; then
    read -r -p "    ↳ Page open. Paste the post, review, then press Enter here AFTER you submit… " _
  fi
}

banner() {
  echo
  echo "──────────────────────────────────────────────────────────────"
  echo "  $1"
  echo "──────────────────────────────────────────────────────────────"
}

# --- preflight ---------------------------------------------------------------
if [ "$DRY_RUN" -eq 0 ]; then
  command -v opencli >/dev/null 2>&1 || { echo "opencli not found — see setup.md"; exit 69; }
  echo "Checking Browser Bridge…"; opencli doctor || { echo "bridge not ready"; exit 69; }
else
  banner "DRY RUN — nothing will be published. Re-run with --go to execute."
fi

# --- main loop ---------------------------------------------------------------
for p in "${PLATFORMS[@]}"; do
  url="$(url_for "$p")" || { echo "skip unknown platform: $p"; continue; }
  file="$POSTS_DIR/$(file_for "$p")"
  adapter="$(adapter_for "$p")"

  banner "$p"
  echo "  post text : $file"
  [ -n "$adapter" ] && echo "  shortcut  : $adapter"
  echo "  1) open the compose page in your logged-in Chrome:"
  run opencli browser "$SESSION" open "$url"
  echo "  2) copy the prepared text and paste it:"
  [ "$DRY_RUN" -eq 0 ] && copy_clip "$file" || echo "    (would copy $file → clipboard)"
  echo "  3) review carefully, then submit YOURSELF in the browser."
  case "$p" in
    xiaohongshu) echo "  note: 小红书 needs images — see the 配图建议 in the post file." ;;
    bilibili)    echo "  note: Bilibili needs a recorded video — the file is a script, not text." ;;
    twitter)     echo "  note: post as a thread (tweets are separated in the file)." ;;
    linkedin)    echo "  note: click 'Start a post'; put the link in the FIRST COMMENT." ;;
    reddit)      echo "  note: pick the right subreddit tab; r/rust, r/commandline, r/ClaudeAI each have their own text." ;;
  esac
  pause
done

banner "Done. Published: ${PLATFORMS[*]}"
echo "Reminder: stagger posts over a few days, and reply to comments early — it"
echo "drives both ranking and the GEO signal (fresh discussion = more AI citations)."
