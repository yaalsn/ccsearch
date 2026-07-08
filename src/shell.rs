//! `ccs init <shell>` — prints a shell function that runs the binary, then does a
//! NATIVE `cd` + `claude --resume` in the CURRENT shell (so the directory change
//! persists). A subprocess can never move its parent shell, so this wrapper is
//! how the "jump into the session's dir" trick works on every platform.
//!
//! The binary only ever emits three lines (cwd, session id, danger flag); each
//! shell reconstructs the resume command in its own syntax. Supported:
//! zsh, bash, fish, powershell (Windows/pwsh). cmd.exe is best-effort.

fn exe() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "ccs".to_string())
}

pub fn init_script(shell: &str) -> String {
    let bin = exe();
    let body = match shell {
        "zsh" | "bash" | "sh" => POSIX.replace("@@BIN@@", &bin.replace('\'', "'\\''")),
        "fish" => FISH.replace("@@BIN@@", &bin.replace('\'', "\\'")),
        "powershell" | "pwsh" => POWERSHELL.replace("@@BIN@@", &bin.replace('\'', "''")),
        "cmd" | "bat" => CMD.replace("@@BIN@@", &bin),
        "" => return "error: usage: ccs init <zsh|bash|fish|powershell|cmd>\n".to_string(),
        other => {
            return format!(
                "error: unknown shell '{other}'. Supported: zsh, bash, fish, powershell, cmd\n"
            )
        }
    };
    body
}

const POSIX: &str = r#"# ccsearch shell integration — add to ~/.zshrc or ~/.bashrc:
#   eval "$(ccs init zsh)"
ccs() {
  local __tmp __cwd __sid __danger
  __tmp="$(mktemp -t ccs.XXXXXX)" || return 1
  '@@BIN@@' --emit-file "$__tmp" "$@"
  if [ -s "$__tmp" ]; then
    { IFS= read -r __cwd; IFS= read -r __sid; IFS= read -r __danger; } < "$__tmp"
    rm -f "$__tmp"
    if [ -n "$__sid" ]; then
      cd "$__cwd" || return
      if [ "$__danger" = "1" ]; then
        command claude --resume "$__sid" --dangerously-skip-permissions
      else
        command claude --resume "$__sid"
      fi
    fi
  else
    rm -f "$__tmp"
  fi
}
"#;

const FISH: &str = r#"# ccsearch shell integration — add to ~/.config/fish/config.fish:
#   ccs init fish | source
function ccs
    set -l __tmp (mktemp -t ccs.XXXXXX)
    '@@BIN@@' --emit-file $__tmp $argv
    if test -s $__tmp
        set -l __lines (cat $__tmp)
        rm -f $__tmp
        if test (count $__lines) -ge 2; and test -n "$__lines[2]"
            cd $__lines[1]
            if test (count $__lines) -ge 3; and test "$__lines[3]" = 1
                command claude --resume $__lines[2] --dangerously-skip-permissions
            else
                command claude --resume $__lines[2]
            end
        end
    else
        rm -f $__tmp
    end
end
"#;

const POWERSHELL: &str = r#"# ccsearch shell integration — add to your $PROFILE:
#   Invoke-Expression (& ccs init powershell | Out-String)
function ccs {
    $__tmp = [System.IO.Path]::GetTempFileName()
    & '@@BIN@@' --emit-file $__tmp @args
    $__lines = @(Get-Content -LiteralPath $__tmp -ErrorAction SilentlyContinue)
    Remove-Item -LiteralPath $__tmp -ErrorAction SilentlyContinue
    if ($__lines.Count -ge 2 -and $__lines[1]) {
        Set-Location -LiteralPath $__lines[0]
        if ($__lines.Count -ge 3 -and $__lines[2] -eq '1') {
            claude --resume $__lines[1] --dangerously-skip-permissions
        } else {
            claude --resume $__lines[1]
        }
    }
}
"#;

const CMD: &str = r#":: ccsearch — save as ccs.cmd on your PATH (before ccs.exe). Best-effort; PowerShell recommended.
@echo off
setlocal enabledelayedexpansion
set "__tmp=%TEMP%\ccs_%RANDOM%.txt"
"@@BIN@@" --emit-file "%__tmp%" %*
if not exist "%__tmp%" goto :done
set "__i=0"
for /f "usebackq delims=" %%L in ("%__tmp%") do (
  set /a __i+=1
  if !__i!==1 set "__cwd=%%L"
  if !__i!==2 set "__sid=%%L"
  if !__i!==3 set "__danger=%%L"
)
del "%__tmp%" 2>nul
if "%__sid%"=="" goto :done
endlocal & cd /d "%__cwd%" & (
  if "%__danger%"=="1" ( claude --resume "%__sid%" --dangerously-skip-permissions ) else ( claude --resume "%__sid%" )
)
:done
"#;
