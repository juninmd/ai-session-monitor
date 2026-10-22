# Manual Windows checklist

What the automated tests cannot see: how the tray icon looks, where the flyout lands, who gets
focus, and what the app does to the OS (registry, Credential Manager, toasts). Run it before a
release and after any change to `src-tauri/`. Record the Windows build, display setup and result
in the PR.

Where this says **tab**, read the provider's card on the **Triagem** surface (or its row on
**Horizonte**) and the detail that opens beside it in the lateral drawer; where it says **tab
strip**, read the surface switch (Triagem, Horizonte, Canais de cota) above the body. The wording
of the per-source steps did not change with the interface.

## Setup

- One Windows 10 22H2 and one Windows 11 machine if possible; a second monitor with a different
  scale factor for the multi-monitor section.
- Release check: `pnpm install`, `pnpm tauri build` (it first runs
  `node src-tauri/scripts/prepare-sidecar.mjs --release` to build and stage the Claude shim),
  then install the NSIS package (per-user install).
- Dev check: `pnpm tauri dev`. The sidecar is a placeholder there, so installing the Claude shim
  must fail cleanly (see Claude shim). Run `node src-tauri/scripts/prepare-sidecar.mjs` once to
  stage a real one. With a custom `build-dir` in your cargo config, the debug exe does not end up
  next to the staged shim: copy `src-tauri/binaries/statusline-shim-<triple>.exe` beside the debug
  exe as `statusline-shim.exe` to test the install in dev.
- Throwaway profile (debug builds only; compiled out of release). With these set, nothing below
  touches your real settings, `~\.claude`, `~\.codex`, the Credential Manager or the Run key:

  ```powershell
  $env:AISM_DATA_DIR = "$env:TEMP\aism-profile"        # settings, webview data, test home
  $env:AISM_HOME_DIR = "$env:TEMP\aism-profile\home"   # fabricated harness state goes here
  $env:AISM_MEMORY_SECRETS = "1"                       # secrets stay in memory
  $env:AISM_SHOW_ON_START = "1"                        # open the flyout right after start
  ```

  Both paths must be absolute; a relative one stops the app at startup with a message on stderr.
- Use fabricated keys and fixtures only. Never paste a real API key into a test run.

## Tray icon

- [ ] The icon appears within two seconds of launch. Tooltip is `AI Session Monitor: sem dados`
      until the first snapshot, then reads like
      `Claude Code · 5 horas: 72% disponível · reseta em 1h 53m`.
- [ ] Ring is crisp (no blur, nothing cut off) at 100, 125, 150, 175 and 200 % scaling, on a
      light and on a dark taskbar. Sizes in use: 16, 24 and 32 px.
- [ ] Colours read on both taskbars: blue (ok), amber (at or below the warning threshold), red
      (critical), full red ring (out), dashed grey (no reading).
- [ ] The arc shrinks clockwise from 12 o'clock as availability drops.
- [ ] Changing the display scale while the app runs re-renders the icon within about 30 s.
- [ ] Windows 11: icon pinned in the corner and icon left in the overflow flyout both work (see
      Placement for the overflow case).
- [ ] Restart Explorer (`taskkill /f /im explorer.exe`, then `explorer.exe`): the icon comes back.
- [ ] Sleep and resume: the tooltip and ring are current within one minute of resume.

## Left click, blur, Escape

- [ ] Left click opens the flyout next to the icon; a second left click closes it and does **not**
      reopen it. Repeat 10 times quickly.
- [ ] Clicking another window, the desktop or the taskbar hides the flyout. Alt+Tab away hides it.
- [ ] Escape inside the flyout closes the palette, then the settings section, then the settings,
      then the provider detail, and hides the flyout only when nothing else is open (the webview
      owns focus on open).
- [ ] Right button and middle button on the icon never toggle the flyout.
- [ ] Known limitation, record the behaviour: focus lost within about 250 ms of opening is
      ignored on purpose (the tray click itself shuffles focus), so the flyout stays until the
      next blur.

## Placement

- [ ] Bottom taskbar: the flyout sits above it, centred on the icon, 8 px from the work area
      edge, never under the taskbar.
- [ ] Top, left and right taskbars (Windows 10): the flyout sits beside the bar, never under it.
- [ ] Auto-hide taskbar: the flyout opens by the icon, with the bar shown or hidden.
- [ ] Icon in the overflow popup: the flyout anchors to the taskbar edge, not to the popup.
- [ ] Icon near the screen edge: the flyout is pushed inside the work area, margin kept.
- [ ] Two monitors, primary on the left, right, above and below: the flyout opens on the monitor
      that holds the icon (the primary one) with correct coordinates.
- [ ] Monitors at 100 % and 150 %: the flyout is 432 logical px wide and crisp on both, and does
      not jump after opening.
- [ ] Small work area (for example 1366x768 at 150 %): the body scrolls, header and surface switch
      stay, height never drops below 200 logical px, 8 px margin all round.
- [ ] Opening another provider's detail changes the content height without flicker; with a bottom
      taskbar the bottom edge stays put.

## Right-click menu

- [ ] Right click shows `Abrir`, `Atualizar agora`, `Sair`, in that order. Left click shows no
      menu.
- [ ] `Abrir` opens the flyout at the icon.
- [ ] `Atualizar agora` refreshes; clicking it again within 10 s does nothing and shows no error.
- [ ] `Sair` ends the process (Task Manager shows no `ai-session-monitor.exe`, no leftover
      `statusline-shim.exe` or WebView2 children) and the icon leaves the notification area at
      once, with no ghost after hovering it. Explorer on Windows 11 (build 26200) dropped even a
      killed process's icon within about 100 ms, so only Windows 10 really tests this.
- [ ] Closing the flyout window by any other route (Alt+F4 while focused) only hides it.

## First run and single instance

- [ ] Fresh profile: the flyout opens by itself once, after the page is painted (no empty panel
      flash). `settings.json` then has `firstRunDone: true` and the next launch does not open it.
- [ ] That first opening lands exactly where a click-opened one does (same position and height,
      bottom edge 8 px above the taskbar). The very first resize of a never-shown window used to
      come out 30 px too tall and spill over the taskbar.
- [ ] Start the app a second time (Start menu or exe): no second tray icon, the running instance
      opens its flyout, the new process exits within about a second.

## Startup failure

A release build has no console, so a startup failure has to reach the user as a toast and a
non-zero exit code. Debug build, throwaway profile (see Setup), fabricated content only. To make
the service unable to start, hold a corrupt `settings.json` open exclusively, so the app can
neither read nor move it:

```powershell
$exe = ".\target\debug\ai-session-monitor.exe"   # wherever your debug exe is
Set-Content "$env:AISM_DATA_DIR\settings.json" '{ not json' -NoNewline
$hold = [System.IO.File]::Open("$env:AISM_DATA_DIR\settings.json", 'Open', 'ReadWrite', 'None')
$app = Start-Process $exe -PassThru
$app.WaitForExit(); $app.ExitCode      # expect 1
$hold.Close()
```

- [ ] A toast titled `AI Session Monitor não iniciou` says the settings file is unreadable and
      could not be moved, then `(código: settings_unreadable). O aplicativo vai fechar.` It shows
      no path and no OS error text.
- [ ] The process exits with code 1 a few seconds later, no tray icon is left behind, and
      `settings.json` is byte for byte what you wrote. Started from a terminal, stderr also
      carries `ai-session-monitor: cannot start: settings_unreadable: ...`.
- [ ] After `$hold.Close()`, start again: the app runs on defaults (the Notifications section
      covers the recovery toast).
- [ ] Known limitation, record the behaviour: a failure before the setup hook runs (no WebView2
      runtime, for example) only writes the stderr line and shows no toast.

## Start with Windows

Release build only; the test overrides skip this on purpose.

- [ ] Turning the setting on creates `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value
      `AI Session Monitor` holding the **quoted** full path of the exe. Nothing is written under
      `HKLM`, also when the app runs elevated.
- [ ] Turning it off removes the value. The entry shows in Task Manager > Startup apps while on.
- [ ] Sign out and in: the app starts on its own, with no console window, and the icon appears.
- [ ] If you disabled the entry in Task Manager, record whether turning the setting on again
      re-enables it (Windows keeps that user decision in `StartupApproved`).
- [ ] Disable the entry in Task Manager > Startup apps, then restart the app: it stays disabled and
      is not silently re-enabled at startup.
- [ ] Move the installed exe to another folder: the Run value keeps the old path until the setting
      is toggled off and on, which registers the new one.
- [ ] A debug build registers its own exe path, so turn the setting off again after testing.

## Notifications

- [ ] With "avisar quando esgotar" on, a window running out shows a toast with the pt-BR title
      and body. Use a fabricated harness fixture under `AISM_HOME_DIR` to force it.
- [ ] Installed build: note which name the toast shows (the app's is expected). The notification
      plugin only sets the app identity for installed builds, so a debug build is expected to
      show Windows PowerShell as the source.
- [ ] Do Not Disturb / Focus Assist holds the toast back as usual.
- [ ] Put garbage in `settings.json` while the app is closed, then start it: one toast titled
      `Configurações restauradas` appears right after the start (the service announces it, once
      per start) and names `settings.corrupt-<epoch>.json` with no folder path. The app runs on
      defaults and that copy holds the original bytes. Starting again shows no toast.

## Polling

The polling loop runs under a supervisor: if it ever panics, the app logs it and starts the loop
again after 2 s (then 4, 8, 16 and 30 s while it keeps failing). Unit tests cover that schedule
with virtual time; this is the check in a real session.

- [ ] Debug build with a fabricated fixture under `AISM_HOME_DIR`: leave it running across a
      sleep and resume, and across a network drop (disable the adapter for a minute, then enable
      it). After each, change the fixture: the tooltip follows within one polling interval and
      nothing needs a restart.
- [ ] Quit from the tray menu while a Codex poll is in flight: no orphan `codex` or
      `codex app-server` process remains in Task Manager.
- [ ] Run a debug build from a terminal for an hour: stderr never shows
      `ai-session-monitor: polling stopped: ...`. If it does, the app recovered by itself but
      something panicked: keep the line (it names the cause and carries no data) and file a bug.

## Quota history

- [ ] Debug build with `AISM_DATA_DIR` on a throwaway folder: after a few polls the provider's
      detail shows "Histórico" with a chart, a "Pico de N% ..." sentence and "Ver em tabela"
      (collapsed). `history.json` appears within about a minute and holds only ids, times and
      numbers.
- [ ] Keyboard only: the window chips under "Histórico" are reachable and switch the chart, and
      "Ver em tabela" opens the readings table. Narrator reads the chart's title and description
      and the sentence.
- [ ] Light and dark themes and high contrast: the current line stays heavier than the earlier
      ones and keeps its "atual N%" label; with "reduzir animações" on, the chevron does not turn
      smoothly.
- [ ] Put garbage in `history.json` while the app is closed, then start it: no toast, the history
      is empty, `history.corrupt.json` holds the bytes.
- [ ] Settings, "Histórico de uso": switch it off and the section disappears from the provider
      details; "Limpar histórico" asks first, then deletes `history.json`; Cancelar keeps it.

## Secrets

Use a fake key. Run this section in the **release** build: a debug build with any `AISM_*`
switch keeps secrets in memory (`AISM_MEMORY_SECRETS`) and never reaches the Credential Manager.

- [ ] Saving a LiteLLM key adds `ai-session-monitor/litellm` under Control Panel > Credential
      Manager > Windows Credentials > Generic, persistence "Local computer". Clearing removes it.
      The OpenCode key uses `ai-session-monitor/opencode`, Kimi Code `ai-session-monitor/kimi`
      z.ai `ai-session-monitor/zai`, OpenRouter `ai-session-monitor/openrouter`, DeepSeek
      `ai-session-monitor/deepseek`, DeepInfra `ai-session-monitor/deepinfra`, Moonshot
      `ai-session-monitor/moonshot`, Novita `ai-session-monitor/novita`, Ollama Cloud
      `ai-session-monitor/ollama` and MiniMax `ai-session-monitor/minimax`.
- [ ] With fourteen providers the triage scrolls in the flyout: the arrow keys move the highlight
      and wrap, the focused row scrolls into view and shows a visible ring, and only the selected
      row is a tab stop.
- [ ] Kimi Code and z.ai tabs: with no key they show the setup card and the "não oficial" badge;
      with a real key the windows match the vendor's own page; a wrong key reads "não foi possível
      verificar a chave", not "chave errada". The console links open the vendor page.
- [ ] OpenRouter and DeepSeek tabs (never exercised against a live account yet): with no key they
      show the setup card and no "não oficial" badge (both are documented APIs). OpenRouter with a
      limited key shows one "Limite de gasto da chave" window that matches the dashboard; an
      unlimited key shows usage facts and no percentage; the balance appears only with a
      management key and matches the credits page. DeepSeek shows one set of facts per currency,
      no percentage, and the tab chip reads "Valores". A wrong key reads "chave ou login
      recusado". The console links open the vendor key pages.
- [ ] DeepInfra tab (never exercised against a live account yet): with no key the setup card and no
      "não oficial" badge. With a spending limit there is one "Limite de gasto da conta" window; the
      balance, "Gasto desde a última fatura" and "Gasto no mês" match the dashboard to the cent. If
      they are 100x off, DeepInfra answers in cents: fix the unit (see data-sources.md).
- [ ] Moonshot tab (never exercised against a live account yet): no "não oficial" badge, no
      percentage, chip reads "Valores". The three balances match the platform's billing page; a key
      of the kimi.com platform reads "chave ou login recusado", not a wrong balance.
- [ ] Novita tab (never exercised against a live account yet): no "não oficial" badge, no percentage,
      chip reads "Valores". The balance, cash balance and credit limit match the billing page (the
      API answers in 1/10000 dollar; a figure 10000x off means the unit changed).
- [ ] Ollama Cloud tab (never exercised against a live account yet): no "não oficial" badge. A
      current plan shows "Créditos incluídos do mês" matching ollama.com/settings, plus the
      purchased balance; a legacy plan shows "Sessão" and "Semana". A key minted at
      ollama.com/settings/keys works; nothing under `~/.ollama` is read.
- [ ] MiniMax tab (UNOFFICIAL route, never exercised against a live account): with no key the setup
      card and the "não oficial" badge. With a Token Plan key, each model shows an interval and a
      weekly row; compare **both percentages with the console** the first time. If they are
      inverted (used shown as left), the `*_usage_count` meaning is wrong: fix `parse.rs`. A pay-as-you-go key reads "chave ou login recusado".
- [ ] Cursor tab (UNOFFICIAL, never exercised against a live account): without consent the setup
      card and the "não oficial" badge, and nothing under `%APPDATA%\Cursor` is opened. After consent
      with Cursor signed in, the included usage matches cursor.com/dashboard (check that the figure
      is a 0..100 percentage, not a fraction) and the on-demand spend matches (cents). Signed out:
      "needs setup", not an error. The token appears nowhere (settings, snapshot, devtools).
- [ ] GitHub Copilot tab (UNOFFICIAL endpoint, never exercised against a live account): a token
      pasted in the key field wins over the local file; with only the local-file consent the token
      is read from `%LOCALAPPDATA%\github-copilot\apps.json`. "Premium requests" matches
      github.com/settings/copilot; the reset reads "estimated" when the response carries no date.
- [ ] Settings → "Hide from screenshots and screen sharing": the flyout is blank or absent in a
      Snipping Tool capture and in a Teams/Meet window share; turning it off restores it at once.
- [ ] Settings → "Local API": off, `curl http://127.0.0.1:6737/v1/limits` is refused; on, it
      returns the readings with no secret in it; with a foreign `Host:` or any `Origin:` header it
      answers 403; a second app holding port 6737 shows no crash and the toggle retries.
- [ ] The key appears nowhere else: not in `settings.json`, not in the webview devtools console,
      not in stderr.
- [ ] A value the Credential Manager cannot hold (about 1,280 characters: it stores UTF-16 in a
      2,560-byte blob) is rejected with a field error on the value, not a crash.
- [ ] `cargo test -p ai-session-monitor --no-default-features -- --ignored` passes. It writes
      and deletes `ai-session-monitor-manual-test/litellm` in the real Credential Manager, which
      is why it is not part of the normal run.

## Claude shim (bundled sidecar)

This edits the tester's own `~\.claude\settings.json`; back it up first, or use the test
overrides in a debug build.

- [ ] Installed build: `statusline-shim.exe` sits next to the main exe.
- [ ] Install (after consenting) sets the statusLine command; uninstall restores the previous
      one exactly.
- [ ] `pnpm tauri dev` without a staged sidecar: install fails with an error in the UI and the app
      keeps running.
- [ ] `CLAUDE_CONFIG_DIR` set to a fabricated folder in the shell that launches the exe (release
      build, or a debug build with no `AISM_*` set): the Claude detail and the shim install follow
      that folder instead of `~\.claude`, unless a folder is typed in the settings, which wins. A
      blank value changes nothing. A debug build with any `AISM_*` override ignores the variable
      on purpose.
- [ ] Settings > Claude Code folder field: type an absolute fabricated folder; the shim status
      follows that folder. Clear the field: it returns to `CLAUDE_CONFIG_DIR` or `~\.claude`.
- [ ] App update with the command installed: after starting the new build, the copy under the data
      dir `bin` folder matches the new bundled `statusline-shim.exe` (compare hashes) and
      `settings.json` is byte for byte unchanged. With Claude Code running (copy busy), the app
      still starts, the detail stays *installed*, and the next start replaces the copy.

## Webview permissions

The flyout may listen to backend events and call the app's own commands, nothing more
(`src-tauri/capabilities/default.json`; a unit test pins the list). This confirms a live webview
enforces it; repeat it after a Tauri upgrade.

- [ ] Debug build, throwaway profile. Open the flyout's DevTools (F12 while it has focus, or
      attach over WebView2 remote debugging with
      `$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9223"`) and run in
      the Console:

      ```js
      await window.__TAURI_INTERNALS__.invoke('plugin:event|emit', { event: 'probe', payload: null })
      ```

      It is rejected with `event.emit not allowed`. The app's own commands still work, for
      example `await window.__TAURI_INTERNALS__.invoke('get_settings')` returns the settings.

## Look and feel

- [ ] A failed settings save is announced as an alert by a screen reader (Narrator).
- [ ] A stale tab stays legible (numbers and labels) in both the light and dark themes.

- [ ] Windows 11: Mica backdrop, rounded corners, shadow. Windows 10: no Mica; the panel is still
      legible with nothing of the desktop showing through the text.
- [ ] Dark and light system themes both render correctly.
- [ ] No title bar, no taskbar button. Note whether the flyout appears in Alt+Tab.
- [ ] Row order and focus rings work with the keyboard; Escape closes the palette, then the open
      settings section, then the settings, then the provider detail, then the flyout.
- [ ] A settings section opens in the same lateral drawer as a provider detail, wide enough that the
      rows keep their rhythm and leaving a 20 px reveal of the dimmed menu; the scrim, the arrow and
      Escape all leave the section, and the keyboard goes back to the row that opened it.
- [ ] A provider selected from elsewhere (a palette jump, a settings link, consent, the stored
      selection on start) is highlighted and scrolled into view, never left offscreen.
