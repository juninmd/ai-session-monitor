# Changelog

All notable changes are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Cursor and GitHub Copilot (both unofficial, opt-in): Cursor reads the sign-in token from its local
  database and shows the included usage and on-demand spend; Copilot shows the monthly premium
  requests with a pasted token or the one the extension keeps on disk.
- Optional local API (`127.0.0.1:6737`, read-only JSON, off by default) and a setting that hides the
  flyout from screenshots and screen sharing.
- Ten interface languages, English by default: English, 简体中文, हिन्दी, Español, Français, العربية,
  বাংলা, Português (Brasil), Русский and اردو (right-to-left for Arabic and Urdu). Pick one under
  Settings → General → Language; the desktop notifications, tray tooltip and tray menu follow it
  (`general.language`).
- Themes from the Canto widget (Default, Hueco Mundo, Dracula, Glass, Light, or follow the system),
  Settings → General → Theme. Glass surfaces on the flyout, the harness plan next to each name, a
  discreet rail of icons for the surfaces and an info tooltip on the Horizon surface. The
  "runs out at…" line in the triage is quieter.
- System-tray flyout (Tauri 2, Windows 11 first) with one tab per harness: Claude Code, Codex,
  Antigravity, OpenCode, Kimi Code, z.ai and LiteLLM. Each tab shows the available share of every usage window, the
  time until it resets, a pace estimate, the data age and an official / unofficial source badge.
- Claude Code: `rate_limits` from the `statusLine` JSON through an opt-in, reversible shim that
  chains any existing status line; optional per-model weeks (Opus, Sonnet, Fable; off by default,
  unofficial) read from `claude -p "/usage"` (cost 0, the app never touches the token), with the
  account usage API as the fallback when the CLI is absent or fails.
- Codex: `codex app-server` rate limits with a rollout-file fallback.
- Antigravity: local language-server quota pools (unofficial).
- OpenCode: Go usage windows (unofficial) plus a read-only local cost and token estimate.
- Kimi Code and z.ai (both unofficial, key pasted by you, kept in the Credential Manager): 5 h,
  weekly and monthly windows from each vendor's usage endpoint. The tab strip scrolls horizontally
  now that there are seven tabs.
- OpenRouter and DeepSeek (both official, documented APIs, key pasted by you, kept in the Credential
  Manager): OpenRouter shows the key's spending limit as a percent window only when the key has a real
  limit, plus usage facts and, with a management key, the credit balance; DeepSeek shows the account
  balance per currency (CNY and USD, never summed) as facts, with no percentage.
- DeepInfra, Moonshot, Novita and Ollama Cloud (all official, documented APIs, key pasted by you, kept in
  the Credential Manager): DeepInfra shows the balance, month spend and, with a spending limit, one
  percent window (the response states no unit, read as dollars); Moonshot and Novita show balances as
  facts in dollars with no percentage; Ollama Cloud shows the monthly included credits (or session and
  weekly percentages on legacy plans) and the purchased balance.
- MiniMax (unofficial: the Token Plan quota route has no published schema, key pasted by you): interval
  and weekly quota per model. Compare the numbers with the console the first time.
- LiteLLM: parametrizable base URL, scope (key, user, team), auth header, poll interval, timeout and
  TLS mode (system, custom CA, insecure for one confirmed host); API key kept in the Windows
  Credential Manager.
- Tray icon ring with the lowest available share, tooltip summary, desktop notifications when a
  window runs out or resets, optional autostart.
- Contract-first architecture: shared Rust and TypeScript models pinned by golden files.
- The logo of each tool in the tab strip, the first-run cards and the settings, vendored as inline
  SVG (no request at run time); origin, licences and hashes in `docs/third-party-assets.md`.
- Portuguese wording for every notice, fact, window and error a collector can send, with tests
  that read the collectors' sources so a new code cannot ship without a message.
- A window that ran out gets its "limite disponível" notification at the reset time even when its
  source stays silent (the Claude status line only moves while Claude Code runs).
- Rate-limit cool-down: a source that answers "too many requests" is left alone (its
  `Retry-After`, 5 min by default, at most 30 min), keeps showing its last good numbers marked
  stale, and the panel says when the next attempt is; a manual refresh during it is dropped and
  the UI explains why.
- Usage-threshold alerts: one desktop notification per window each time it crosses a configurable
  used percentage (default 80 % and 95 %, up to four values, or off), re-armed at every reset.
- Adaptive polling (setting "Ritmo adaptativo", on by default): a source far from its limit and
  from its reset is read up to every 30 minutes; it goes back to the normal interval at 70 % used
  or 10 minutes before a reset.
- Quota burndown history ("Histórico" in each tab, collapsed by default): a local, bounded store
  of used percentages per window (35 days, 20 000 points, 1 MiB; no account data), shown as an
  inline SVG of the current window against the earlier ones with a text summary and a table.
  New commands `get_history` and `clear_history`, the setting "Guardar o histórico de uso" and a
  confirmed "Limpar histórico" button; a damaged file is set aside and the history restarts empty.
- Codex: the count of earned rate-limit reset credits (`rateLimitResetCredits.availableCount` of
  `account/rateLimits/read`) shows as a "Recargas de reset" fact. Only the count is read (strict:
  absent, null, negative, fractional or absurd counts stay unknown); the detail rows and the
  consume RPC are never touched.
- Setting `general.showUsed` ("Mostrar cota como: disponível / usado"): the headline numbers,
  gauges, tab chips, spoken labels, live announcements, the tray tooltip and the "limite
  disponível" notification switch wording. Status colours, pace and stored numbers are unchanged.
  The tray icon arc stays on the available share.
- A "~" prefix, read aloud as "estimativa", on values that are estimates: the pace projection
  ("No ritmo atual..."), the OpenCode local cost and token facts, and an Antigravity window whose
  5 h / weekly name was guessed from its reset time.
- Polling is supervised: if the scheduler dies, the shell restarts it (2 s doubling to 30 s).
- A start that fails shows a notification with the step and the error code and exits with code 1,
  instead of vanishing from the tray.
- `CLAUDE_CONFIG_DIR` is honoured for the Claude Code folder (a folder typed in the settings wins).
- Debug builds accept `AISM_DATA_DIR`, `AISM_HOME_DIR`, `AISM_MEMORY_SECRETS` and
  `AISM_SHOW_ON_START` so smoke runs never touch a real profile; release builds do not contain them.

### Changed

- The flyout has one home screen instead of a tab per harness: **Triagem** (every reachable provider
  in one urgency-ordered grid, each card carrying its own quota windows and renewals), **Horizonte**
  (one runway bar per provider, from now to that window's renewal) and **Canais de cota** (the
  evidence list of channels the app reads, could read or must not read). The detail of a provider
  opens beside the list in a lateral drawer, and **Ctrl+K** opens a command palette that jumps to a
  provider, refreshes one source or the fleet, copies the fleet state as text and opens a source's
  settings. The previous grid iteration, its `?prototype=` routes and the review-only surfaces
  (inline command list, side-by-side comparison) were removed; the browser dev loop keeps `?mock=`.
- Node engines floor is `^22.22.2 || ^24.15.0 || >=26`.
- The Cargo bin of the shim is `aism-statusline-shim`; the sidecar stem stays `statusline-shim` and
  a test keeps the two apart.
- The quit path waits up to 2.5 s for the scheduler to unwind before exiting.
- A tie for the lowest available share in the tray summary goes to the earlier tab.
- Autostart starts passively (no write unless the state differs) and a failed registration is
  retried and reported with a toast.
- The reset toast no longer covers LiteLLM.
- An empty tray icon rectangle no longer positions the flyout.
- `scripts/soak.mjs` prints its usage and exits 2 on a missing or invalid argument.

### Fixed

- Claude shim install keeps the previous `statusLine` (saved and chained) instead of replacing it.
- Shim state is per Claude profile: files in the data dir carry an FNV-1a suffix of the config dir,
  and the saved `statusLine` record is also kept beside `settings.json`
  (`settings.json.aism-state`), so a wiped data dir cannot lose it.
- The shim's chained command finishes on process exit; it no longer leaves a child running.
- The installed shim copy is refreshed from the bundled build once at each app start (copy only;
  `settings.json` is never touched). A busy copy is retried at the next start.
- An empty OpenCode `-wal` file next to `opencode.db` is ignored.
- A LiteLLM key is bound to the base URL it was saved for and is not sent to another destination.
- A debug run with any single `AISM_*` switch keeps secrets in memory, so a smoke run never reaches
  the real Credential Manager.
- Settings: a failed save is announced with the `alert` role; a Credential Manager failure no longer
  blanks Settings (a notice is shown); a Claude folder field was added; focus handling fixed.
- Stale tabs keep legible contrast in both themes; an expired window is worded once per card; the
  setup card shows the shim-outdated notice; a snapshot event racing the first `get_snapshot` no
  longer overwrites newer data.
- Antigravity tab was empty for the `agy` command-line client (its language server is embedded and
  answers 401 without an in-memory token). The collector now runs `agy -p "/usage" --output-format
  json` from the fixed install path (no PATH search, no tokens spent) and prefers it over the IDE
  server; the IDE path is unchanged when the CLI is absent or fails.

### Known limitations / follow-ups

- `Retry-After` is honored only for rate-limit answers (the cool-down, capped at 30 min); other
  retryable errors use the plain cadence backoff and do not read it.
- Paths in a loaded `settings.json` are validated on save, not on load.
- Large integers in settings are rewritten on save (JSON number round trip).
- The Claude shim status is read before consent is given.
- LiteLLM timestamps (`expires`, `budget_reset_at`) are not bounded.
- Some transient I/O errors are classified as non-transient.
- No Windows Job Object kill-on-close for the Codex child: a crash or End Task can leave an orphan
  `codex app-server`.
- CI has no Node 22.22.2 leg in its matrix (the engines floor is not exercised).
- `litellmDraft.ts` and the Rust validation treat `app.localhost` differently.
- Antigravity: the IDE plan label is not shown when the `agy` CLI is the source.
- `crates/usage-support/tests/it/redact.rs` is at 298 of the 300-line limit.

### Security

- The Claude and Antigravity CLI probes start in an empty directory under the app's data dir, not in
  the shared temp dir, so a stray `.claude` folder there is never read as project settings.
- All network access happens in Rust through one hardened client (https only, no redirects, size and
  time caps); the webview has no network access and never receives a secret.
- The flyout capability grants only `core:event:allow-listen` and `allow-unlisten`; the webview
  cannot emit events.
- A settings file that cannot be read is set aside as `settings.corrupt-<epoch>.json` and you are
  told once; the `authHeader` setting accepts only header-name characters and refuses headers the
  HTTP client sets itself, and custom paths refuse UNC shares and device paths.
- Supply chain: direct crates and npm packages must be at least 7 days old, pnpm install scripts
  are denied, and CI runs RustSec audit and `cargo deny` (licences, bans, sources).
