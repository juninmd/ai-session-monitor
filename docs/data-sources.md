# Data sources

What the app reads for each tab, how much that source can be trusted, what it sends, and how to
turn it off. Facts were verified against vendor documentation or source on 2026-10-10/11; items
marked *unverified* have never been exercised against a live account.

## Principles

- **Official first.** Each tab uses a documented interface when one exists. Anything else is
  badged **não oficial** in the UI and stays behind an explicit consent switch.
- **Consent before reading.** A tab whose source touches another tool's files or processes stays in
  *needs consent* and does no I/O until you agree. Disabling a tab stops all reads for it.
- **Unknown is not zero.** A missing value is shown as unknown. A window whose reset time has
  passed is shown as reset, without a status.
- **Read-only toward providers.** The app never writes to a provider, never refreshes or stores a
  provider's OAuth tokens, never uses browser cookies and never scrapes dashboards.
- **Secrets.** The only secrets the app stores are the LiteLLM and OpenCode Go API keys you paste.
  They live in the Windows Credential Manager (`ai-session-monitor/<kind>`), never in
  `settings.json`, snapshots, logs or events. The webview can write, clear and ask *whether* a
  secret exists, never read it back.
- **Network only from Rust.** The webview CSP allows no network. Requests go through one hardened
  client: https only (plain http only to loopback), no redirects, response-size and time caps.

## Claude Code — official (statusLine)

| | |
|---|---|
| Source | `rate_limits.five_hour` and `rate_limits.seven_day` (`used_percentage`, `resets_at`) in the JSON Claude Code pipes to its `statusLine` command ([docs](https://code.claude.com/docs/en/statusline)). |
| Mechanism | You consent to install a tiny `statusline-shim`. It records **only** the `rate_limits` object (never cwd, transcript path, model, cost or session id) to `%LOCALAPPDATA%\dev.aisessionmonitor.tray\claude\rate_limits.json`, and runs your previous status line command unchanged (chained, first in Git Bash, else PowerShell, else cmd, with a 5 s limit). The command written to `settings.json` is a bare path when it can be, `~/` under your profile, or a quoted path as a last resort. A bare path runs in Git Bash, PowerShell 7, Windows PowerShell 5.1 and cmd; the `~/` form does not run in cmd, and Git Bash resolves it from `HOME`, which must equal the Windows profile (the default); the quoted form does not run in PowerShell. `spend_limit` (a money cap) is shown when the status line reports one. |
| Edits | `settings.json` in the Claude config dir (a folder typed in the settings wins, else `CLAUDE_CONFIG_DIR` as set at launch, else `~/.claude`; the variable is ignored when any debug `AISM_*` override is set). A one-time `settings.json.aism-backup` is written first; the previous `statusLine` is saved and restored exactly on uninstall. That record is kept twice: in the app data dir and beside `settings.json` as `settings.json.aism-state`, so a wiped data dir cannot lose it (the backup is never a source). A `settings.json` that is not valid JSON is never modified. Rewriting it normalizes duplicate keys (the last value wins) and number formatting, which the backup keeps byte for byte; a symlinked `settings.json` would be replaced by a regular file (*unverified*). |
| Profiles | The files an install owns in the app data dir (`rate_limits`, `shim-state`, `chain`, the OAuth cache, and the shim copy under `bin\`) are per config dir: a 16-hex-digit FNV-1a 64 suffix of the normalized folder, none for the default `~/.claude`. Installing into or removing from one Claude profile never reaches another. |
| Refresh | When you consented and `settings.json` holds our healthy command, the installed shim copy is replaced from the bundled build once at each app start, so an app update reaches it. It replaces only the exe copy, never `settings.json`, the saved previous `statusLine` or the chain file. A busy copy (Windows refuses one Claude Code is running) is logged and retried at the next start; the status stays *installed*. No notice, no IPC command. |
| Names | The Cargo bin is `aism-statusline-shim`; the sidecar (`externalBin`) stem is `statusline-shim`. They must differ, or `tauri-build` overwrites the workspace binary (guard: `src-tauri/tests/sidecar_names.rs`). |
| Limits | The data only updates while Claude Code runs and after its first API response, so a quiet period shows *stale* data with its age. Sessions share one file and the last writer wins, so a long-idle session can briefly bring older weekly numbers back. Some plans report no `rate_limits` (shown as no data). Opus/Sonnet weekly caps are not in this channel: verified absent as of 2026-10-12. The official [statusline docs](https://code.claude.com/docs/en/statusline) list only `rate_limits.five_hour`, `seven_day` and `spend_limit`; community reports on v2.1.199 to v2.1.260 (anthropics/claude-code issues [#69791](https://claudeissues.com/issue/69791-statusline-stdin-missing-seven-day-sonnet-and-seven-day-opus-rate-limit-fields), [#73770](https://claudeissues.com/issue/73770-statusline-expose-per-model-weekly-rate-limits-opus-sonnet-fable-to-the-status-l), still open, no maintainer reply) show no `seven_day_opus`, `seven_day_sonnet` or `model_scoped` keys. Recheck when the docs change. |
| Optional | *OAuth usage* (off by default, unofficial): `GET https://api.anthropic.com/api/oauth/usage` with the access token Claude Code already holds. Undocumented; it may violate Anthropic's terms and may be rate limited. The token is read from Claude Code's credentials file, kept in memory, never refreshed or written. It is asked only while the status-line data is fresh and at most once per 300 s, so an idle machine never polls it. Failures never hide the statusLine data (one `claude_oauth_partial` notice). Never exercised against a live account (*unverified*). |
| Off switch | Uninstall the shim in the tab (restores `settings.json`); or disable the tab. |

## Codex — official (app-server)

| | |
|---|---|
| Source | `codex app-server` JSON-RPC over stdio: `initialize`, `initialized`, `account/rateLimits/read` ([docs](https://developers.openai.com/codex/app-server)). Windows are named by their reported `windowDurationMins` (within 5 %: 300 = 5 h, 10080 = weekly), so a weekly-only or swapped answer lands in the right slot; a window without a length falls back to primary = 5 h, secondary = weekly. Plan and credits when present. The protocol also has `rateLimitResetCredits` (earned resets) and `account/rateLimitResetCredit/consume`; the app does not read or call them. |
| Mechanism | On each poll the app spawns your installed `codex` (resolved through PATH/PATHEXT, `codex.cmd` included) with no console window, a 15–20 s budget, and kills the whole process tree afterwards. |
| Limits | The command is documented as experimental, so fields are treated as optional. Pro plans currently report no 5 h window; API-key sign-in reports no plan windows. Not signed in is its own state. |
| Fallback | If the app-server cannot run, the newest `token_count.rate_limits` event in `~/.codex/sessions/**/rollout-*.jsonl` is read (tail only, only that object, never message content) and labelled *last seen*. |
| Never | `auth.json` and the ChatGPT backend usage endpoint are not used. |
| Off switch | Withdraw consent or disable the tab. |

## Antigravity — não oficial (local language server)

| | |
|---|---|
| Source | The local language server of the running Antigravity IDE (Connect over HTTPS on `127.0.0.1`), using the CSRF token from the process command line. `GetUserStatus` gives the plan (and, on older IDEs, per-model quota); `RetrieveUserQuotaSummary` gives the 5 h and weekly buckets (404 on older IDEs). Community-reverse-engineered; **never exercised against a live IDE in this project** (*unverified*). |
| Pools | Gemini Pro + Flash share one pool; Claude + GPT-OSS keep a separate pool. Each may have a 5 h and a weekly window, or only weekly on the free tier. A missing remaining-fraction is *unknown*. Windows from an older IDE's per-model quota have a guessed cadence (5 h or weekly, from how far away the reset is) and no `window_minutes`, so they never raise a pace alarm. |
| Mechanism | Finds same-user `language_server*` processes (`sysinfo`; the command line is read only for name matches), takes the token, ports and `--app_data_dir` from the arguments, lists listening ports with `%SystemRoot%\System32\netstat.exe -ano -p tcp`, and calls the loopback endpoint (self-signed certificate accepted **only** for `127.0.0.1`). Plain HTTP is tried only on the extension-server port, on loopback. The token is a local credential: kept in memory, never logged or shown. Budget: 4 s per request, at most 4 servers and 6 HTTPS ports each, 24 s per poll. |
| States | IDE closed → *not running* (a normal state). Shape changed → *endpoint changed* instead of a made-up number. A pool no response mentions gets no window; a pool with an unreadable share gets a window with unknown use and one `antigravity_pool_unknown` notice. |
| Never | Google's cloud endpoints and the Antigravity OAuth token are not used. |
| Off switch | Withdraw consent or disable the tab. |

## OpenCode — não oficial (Go usage endpoint + local estimate)

| | |
|---|---|
| Remote | `GET https://opencode.ai/zen/go/v1/usage` with your OpenCode Go API key as Bearer: rolling (5 h), weekly and monthly *percent used* and reset times. Built by the OpenCode maintainers but undocumented. The host is a constant; the key is never sent anywhere else. |
| Errors | 401/403 are reported as *could not verify the key or the Go subscription* (valid keys have been rejected by the service), with a link to the console, never as a certain "bad key". |
| Local estimate | With consent, the app opens OpenCode's `opencode.db` **read-only** and sums numeric cost and token fields of assistant messages for 5 h / 7 d / 30 d. This is a list-price estimate for this PC, **not quota**: it is never turned into a percentage or a reset time. Credential, account and prompt tables are never queried. |
| Pay-as-you-go | Zen balance has no key-readable API; the tab offers a link to the billing page instead of a made-up balance. |
| Off switch | Clear the key, withdraw the local-estimate consent, or disable the tab. |

## LiteLLM — official (proxy API, parametrizable)

| | |
|---|---|
| Source | `GET {baseUrl}/key/info` (the key identifies itself); or `/user/info?user_id=` / `/team/info?team_id=&key_limit=1` for the user or team scope (`key_limit=1` keeps the team response under the body cap). Fields: `spend`, `max_budget`, `budget_duration`, `budget_reset_at`, per-model budgets, concurrent budget windows, static limits (`rpm`, `tpm`, parallel requests, `expires`). |
| Follow-up | A key with no budget data of its own asks its user, then its team (at most two extra requests, same host and credential, never an id that contains the key). The first level with budget data supplies the windows (`meta.level` = key / user / team); the static limits always come from the record you configured. |
| Shape | The headline `budget` window carries a USD amount, `used_pct` (may exceed 100), `resets_at` and `window_minutes` (nominal length of `budget_duration`; months stay unknown). Per-model caps and concurrent budget windows become extra windows, at most 16, most consumed first. |
| Budget | A request is capped at 1 MiB and 1–30 s; the whole poll is bounded by min(3 × timeout, 25 s). |
| Settings | Base URL, scope (key / user / team) and entity id, auth header (default `Authorization`), poll interval (60–3600 s, default 300), timeout, TLS mode. |
| Rules | https only (plain http only for loopback); no credentials, query or fragment in the URL; a path prefix is kept; no redirects. TLS modes: system store, custom CA (PEM file) or *insecure for one confirmed host* (shown with a warning). |
| Semantics | A key without `max_budget` is shown as *no cap configured* — absent does not mean unlimited; `max_budget = 0` counts as no cap, and a cap that lives only in a linked budget table is not read. Budget resets come from a proxy sweep and can lag a few minutes. TPM/RPM are static caps, not live usage. |
| Confidence | Response shapes come from the proxy source (v1.104.2) and its docs, never from a live proxy: a different proxy surfaces as *endpoint changed*, not as invented numbers. |
| Off switch | Clear the key or disable the tab. |

## Local files

| Path | Content |
|---|---|
| `%LOCALAPPDATA%\dev.aisessionmonitor.tray\settings.json` | Non-secret settings. A corrupt file is kept as `settings.corrupt-<epoch>.json` and you are notified. |
| `...\history.json` | Quota history for the burndown (setting "Guardar o histórico de uso", on by default): used percentages and times per window id, 35 days, at most 1 MiB. No account, plan, label or path. Cleared by "Limpar histórico"; a damaged file is kept as `history.corrupt.json`. |
| `...\claude\rate_limits.json` | The Claude `rate_limits` object plus `capturedAt`. |
| `...\claude\shim-state.json`, `chain.json` | The previous `statusLine`, kept for exact restore and chaining. Non-default Claude profiles get `shim-state-<key>.json` / `chain-<key>.json` (and `bin\<key>\`). A second copy of the record sits at `<config dir>\settings.json.aism-state`. |
| `...\bin\statusline-shim.exe` | Stable copy of the shim referenced from Claude Code's settings; refreshed from the bundled build at each start. |
| Windows Credential Manager | `ai-session-monitor/litellm`, `ai-session-monitor/opencode`. |
