#!/usr/bin/env bash
# Build a throwaway, synthetic ~/.claude so the demo recording never touches real
# sessions. Creates fake cc-logo-pcb sessions and a stub `claude` that just prints
# a "resumed" banner (so the demo shows the jump+resume without launching Claude).
set -euo pipefail

H=/tmp/home
BINDIR=/tmp/ccs-demo-bin
rm -rf "$H" "$BINDIR"
mkdir -p "$H/code/cc-logo-pcb" "$H/code/notes" "$BINDIR"
CWD="$H/code/cc-logo-pcb"
ENC=$(printf '%s' "$CWD" | sed 's![/.]!-!g')   # cwd -> Claude Code project-dir name
PROJ="$H/.claude/projects/$ENC"
mkdir -p "$PROJ"

# stub claude
cat > "$BINDIR/claude" <<'SH'
#!/bin/sh
printf '\n  \033[38;5;114m✔ Resumed session · cc-logo-pcb · full conversation restored\033[0m\n'
printf '  \033[2m  96 messages · last topic: routing the LED traces around the logo silkscreen\033[0m\n'
printf '  \033[2m  quit Claude → you land back in the project directory, ready to keep going\033[0m\n\n'
SH
chmod +x "$BINDIR/claude"

python3 - "$PROJ" "$CWD" <<'PY'
import json, sys, datetime
proj, cwd = sys.argv[1], sys.argv[2]
now = datetime.datetime.utcnow()
def ts(days): return (now - datetime.timedelta(days=days)).strftime("%Y-%m-%dT%H:%M:%S.000Z")
# (session_id, days_ago, msg_count, title, user_prompt, assistant_reply)
sessions = [
    ("7f3a1c8e-0b12-4c3a-9e11-a1b2c3d4e5f6", 4, 96,
     "Design a logo-shaped PCB business card",
     "make the silkscreen match the logo outline and route the LED traces around it",
     "I'll shape the silkscreen layer to follow the logo outline in KiCad, then route the WS2812 LED traces around the perimeter and keep the ground pour clear of the badge."),
    ("2b9d44a1-77c8-4690-bf0c-13b6d7635737", 6, 142,
     "KiCad footprint for the USB-C connector",
     "the castellated pads aren't aligning with the USB-C footprint on the logo PCB",
     "Let's check the connector footprint against the datasheet — the pad pitch looks off. I'll regenerate it and re-run DRC on the logo PCB."),
    ("a54e458b-b20a-4c85-0f31-c1c2bf99f0a1", 9, 58,
     "Panelize the logo boards for JLCPCB",
     "panelize the logo PCB boards with mouse-bites for the JLCPCB order",
     "I'll lay out a 3x3 panel with mouse-bite tabs, add fiducials, and fill in the JLCPCB fab fields."),
]
for sid, days, msgs, title, prompt, reply in sessions:
    out = [
        {"type": "ai-title", "aiTitle": title, "sessionId": sid},
        {"type": "user", "sessionId": sid, "cwd": cwd, "gitBranch": "main", "timestamp": ts(days),
         "message": {"role": "user", "content": prompt}},
        {"type": "assistant", "sessionId": sid, "cwd": cwd, "gitBranch": "main", "timestamp": ts(days),
         "message": {"role": "assistant", "content": [{"type": "text", "text": reply}]}},
    ]
    for i in range((msgs - 2) // 2):  # pad to a realistic message count
        out.append({"type": "user", "sessionId": sid, "cwd": cwd, "timestamp": ts(days),
                    "message": {"role": "user", "content": f"iterate on the routing, pass {i}"}})
        out.append({"type": "assistant", "sessionId": sid, "cwd": cwd, "timestamp": ts(days),
                    "message": {"role": "assistant", "content": [{"type": "text", "text": "done — re-running DRC"}]}})
    with open(f"{proj}/{sid}.jsonl", "w") as f:
        for o in out:
            f.write(json.dumps(o, ensure_ascii=False) + "\n")
print(f"seeded {len(sessions)} synthetic sessions in {proj}")
PY
