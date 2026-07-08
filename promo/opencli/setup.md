# OpenCLI setup (one-time)

[OpenCLI](https://github.com/jackwener/OpenCLI) drives your **logged-in Chrome**, so
publishing runs under your own accounts on your own machine. Do this setup once,
then use `publish.sh`.

## 1. Install the CLI

```bash
npm install -g @jackwener/opencli
# or download the desktop app: https://opencli.info/download
opencli --version
```

## 2. Install the Browser Bridge extension

- Chrome Web Store: search **"OpenCLI"** and add it
  (https://chromewebstore.google.com/detail/opencli/ildkmabpimmkaediidaifkhjpohdnifk)
- or load the unpacked `opencli-extension-*.zip` from OpenCLI's GitHub Releases
  via `chrome://extensions` → Developer mode → Load unpacked.

## 3. Verify the connection

```bash
opencli doctor
```

You should see the bridge connected. If not, make sure Chrome is open and the
extension is enabled.

## 4. (Recommended) A dedicated profile

Keep posting isolated from your everyday browsing:

```bash
opencli profile list
opencli profile rename <contextId> promo
opencli profile use promo
```

## 5. Log in to each platform in that Chrome profile

Open and sign in to the ones you'll post to: X/Twitter, Reddit, dev.to, Lobsters,
V2EX, 掘金, 知乎, 小红书, Bilibili, LinkedIn. OpenCLI reuses these sessions — no API
keys needed.

## 6. (Optional) Install the OpenCLI skill into Claude Code

So you can drive it conversationally later:

```bash
npx skills add jackwener/opencli --skill opencli-browser
```

---

## Command cheat-sheet used by publish.sh

Generic browser automation (works on any site):

```bash
opencli browser <session> open <url>
opencli browser <session> fill <selector> "<text>"
opencli browser <session> type "<text>"
opencli browser <session> click [selector]
opencli browser <session> wait [selector]
opencli browser <session> screenshot
```

Native adapter publish verbs (flags vary by version — always check `--help`):

```bash
opencli twitter post        # also: thread, reply
opencli zhihu answer        # also: comment, like, favorite
opencli xiaohongshu publish
opencli reddit comment      # note: submitting a new post uses browser automation
```

> `opencli linkedin`, `opencli bilibili`, HN, dev.to, Lobsters, V2EX and 掘金 have
> **no publish verb** — `publish.sh` uses generic browser automation (open the
> compose page in your logged-in Chrome) for those.
