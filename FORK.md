# VoiceKey — personal fork (branch `history`)

Fork of [J3n5en/voicekey](https://github.com/J3n5en/voicekey) at tag `v1.6.1`, kept for one user on Windows.
Remote `fork` = https://github.com/0hickenduck/voicekey. Upstream is untouched; nothing is sent upstream.

## What this branch adds

| Commit | Change |
| --- | --- |
| `e7de878` | Transcription history: every text that reaches the input field is appended to `%APPDATA%\VoiceKey\history.jsonl` (`ts` ms, `text`, `channel`, `lang`, `alts` = the other engines' finals in multi-engine mode). Settings → 历史 page: search, click to copy, open folder, on/off (`history` setting, default on). Code: `app/src-tauri/src/history.rs`, three call sites in `controller.rs` |
| `7d74258` | `.github/workflows/build-windows.yml`: unsigned Windows build on every push to `history` (the upstream Release workflow needs the author's updater signing key) |
| `bd20cfe` | Compact overlay: pick panel 400 px, no header/footer/logos/state dots, five-bar level meter (`app/src/lib/Bars.svelte`) in the selected row; HUD capsule 34 px. History records the language mix (`mix`, e.g. `zh+en`, languages with >= 15% share) |
| `8b5695f` | Pick panel pinned to the bottom center of the screen the user is typing on (no longer follows the caret) |

## Build and install

- Push to `history` → GitHub Actions builds in ~11 min (runs `cargo test history` first). Download:
  `gh run download <run-id> --repo 0hickenduck/voicekey -n VoiceKey-history-windows -D <dir>`
  (local copies in `D:\VibeCoding\voicekey-builds\`).
- Local builds need Visual Studio Build Tools (C++), not installed on this PC.
- Install: quit VoiceKey from the tray, run the NSIS `*-setup.exe` (unsigned: SmartScreen → More info → Run anyway).
  It installs over the upstream build; settings and history stay in `%APPDATA%\VoiceKey`.
- **Decline in-app updates**: the updater still points at upstream releases and would replace this build.

## Who uses the history

The language pipeline (`C:\Users\libowen\Tools\language-pipeline`, see its README "Next steps") is meant to analyse
`history.jsonl` for the user's English/Japanese speaking patterns. Not built yet.
