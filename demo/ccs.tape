# ccsearch demo — reproduce with (from repo root):
#   cargo build --release && bash demo/seed.sh && vhs demo/ccs.tape
# Records against a synthetic ~/.claude (see seed.sh) so no real data is shown.

Output assets/demo.gif

Set Shell "bash"
Set FontSize 17
Set Width 1160
Set Height 720
Set Padding 26
Set Theme "Catppuccin Mocha"
Set TypingSpeed 55ms
Set Framerate 24
Set WindowBar Colorful

# --- hidden setup ---
Hide
Type "source demo/setup-env.sh" Enter
Sleep 900ms
Show

# --- the demo ---
Sleep 800ms
Type "ccs logo pcb"
Sleep 500ms
Enter
Sleep 2400ms
Type "1"
Sleep 400ms
Enter
Sleep 5500ms      # hold the "resumed, now in ~/code/cc-logo-pcb" payoff so the loop doesn't read as "exit"
