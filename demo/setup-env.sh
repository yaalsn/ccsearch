# Sourced by demo/ccs.tape inside the VHS recording (run from the repo root).
# Points ccs at the throwaway synthetic ~/.claude from seed.sh so nothing real
# is ever shown, and loads the shell integration + a clean prompt.
CCSBIN="$(pwd)/target/release/ccs"
export HOME=/tmp/home
export PATH="/tmp/ccs-demo-bin:$PATH"
export CCS_IGNORE=""   # the synthetic home lives under /tmp, which is ignored by default
export PS1='\[\e[1;36m\]\w\[\e[0m\] $ '
eval "$("$CCSBIN" init bash)"
"$CCSBIN" --reindex >/dev/null 2>&1
cd "$HOME/code/notes"
clear
