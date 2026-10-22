# IPC and view semantics

Contract between the webview (`src/`), the shell (`src-tauri/`), the window-free core
(`crates/app-core`) and the collectors (`crates/collector-*`). Types live in
`crates/usage-model` (Rust) and `src/lib/contract` (Zod); `contract/*.json` pins both.

## Commands (webview -> shell)

Invoked through `Backend` (`src/lib/backend.ts`). Argument names are camelCase on the wire. A
failed command rejects with a serialized `HarnessError` (`{ code, retryable, params }`); the
webview never shows raw error text.

| Command | Args | Returns | Notes |
|---|---|---|---|
| `get_snapshot` | - | `Snapshot` | Current views in tab order; never blocks on a collector. |
| `refresh` | `harness?: HarnessId` | - | Non-blocking. Asks the scheduler to collect now; the result arrives as a `snapshot` event. Rate-limited per harness (>= 10 s); a refused call is not an error. |
| `get_settings` | - | `Settings` | |
| `save_settings` | `settings: Settings` | `Settings` | Normalizes (clamps ranges), persists atomically, applies (interval, enabled harnesses, autostart, tray source) and emits `settings-changed`. Returns the normalized value. |
| `secret_status` | - | `{ litellm, opencode, kimi, zai, openrouter, deepseek, deepinfra, moonshot, novita, ollama, minimax }` | Booleans: a secret exists in the Credential Manager. Rejects with `internal` (`where = secret_store`) when the Credential Manager cannot be read; the UI keeps Settings usable and shows a notice. |
| `set_secret` | `kind: 'litellm' | 'opencode' | 'kimi' | 'zai' | 'openrouter' | 'deepseek' | 'deepinfra' | 'moonshot' | 'novita' | 'ollama' | 'minimax'`, `value: string` | `SecretStatus` | Trims the value; an empty one rejects with `invalid_input` (`field = value`). The value is never logged or echoed. Triggers a refresh of that harness. |
| `clear_secret` | `kind` | `SecretStatus` | Idempotent. |
| `claude_shim_status` | - | `ShimStatus` | Read-only look at Claude Code's settings.json. |
| `install_claude_shim` | - | `ShimStatus` | Requires `claude.shimConsent`. Copies the shim to a stable path under the data dir and wraps any existing statusLine. |
| `uninstall_claude_shim` | - | `ShimStatus` | Restores the previous statusLine exactly. |
| `get_history` | `harness: HarnessId`, `limit?: 1..=8` | `HarnessHistory` | Past windows for the burndown: `{ harness, epochs: [{ windowId, resetsAt, points: [{ t, usedPct }] }] }`, the newest `limit` (default and maximum 8) epochs of each window, oldest first, at most 120 points each (thinned keeping the first, the last and the peak of each slice). Empty while `general.historyEnabled` is false. A `limit` outside 1..=8 rejects with `invalid_input` (`field = limit`). |
| `clear_history` | - | - | Forgets every stored reading, in memory and on disk (the `history.corrupt.json` copy too). Idempotent. |
| `open_link` | `link: ExternalLink` | - | Opens a fixed destination in the default browser. The webview cannot pass a URL. |
| `hide_flyout` | - | - | |
| `set_flyout_height` | `height: number` | - | Content height in CSS px; the shell clamps to the monitor work area and resizes keeping the anchor edge. |
| `quit_app` | - | - | |
| `get_update_status` | - | `UpdateStatus` | Current updater state; no network. |
| `check_update` | - | `UpdateStatus` | Checks the fixed public GitHub channel through SafeClient. |
| `download_update` | - | `UpdateStatus` | Downloads the selected installer, verifies both Minisign signatures and its authenticated version. |
| `install_update` | - | `UpdateStatus` | Requires verified bytes and a production Windows x64 build. Drains polling before opening NSIS and exiting. |

## Events (shell -> webview)

| Event | Payload | When |
|---|---|---|
| `snapshot` | `Snapshot` | Whenever any view changed. |
| `settings-changed` | `Settings` | After a successful `save_settings` (also from another surface). |
| `flyout-shown` | none | The flyout became visible. The webview refocuses what the user was on (the settings heading, the open detail or the selected row) and calls `refresh()`; the scheduler skips collectors that are fresh. |
| `update-status` | `UpdateStatus` | Updater transitions and download progress (throttled to 100 ms). |

## GitHub update protocol

`UpdateStatus` exposes `currentVersion`, `phase`, `availableVersion`, `notes`,
`downloadedBytes`, `totalBytes`, `checkedAt`, `failure` and `installable`. Unknown version,
notes, total and check time are `null`. Installer URLs, signatures and raw errors stay in Rust.
Phases: `idle`, `checking`, `current`, `available`, `downloading`, `ready`, `installing`,
`error`, `unavailable`. Failure kinds: `check`, `download`, `signature`, `install`.

`general.autoUpdateCheck` defaults to true, including settings saved by an older version.
The shell checks at startup and then every six hours, without automatic downloads or
installation. Turning it off leaves manual checks available. Debug runs with any test
override disable the updater entirely; other debug builds can check but cannot install.

Operations are serialized; concurrent requests return the current state. The channel
accepts stable versions newer than the running one and only this repository's release
assets. SafeClient bounds redirects to GitHub release hosts, time and body size (1 MiB
manifest, 128 MiB installer), uses verified TLS and sends no credentials. Installation
requires an authenticated NSIS executable and signed version. Failed downloads never
invoke an installer. A ready download remains in memory until explicit installation or
app exit. Installation confirmation drains queued settings saves, shuts down collectors
and flushes history. If staging or process launch fails, monitoring remains or resumes.
Unsaved form drafts block installation until saved or explicitly discarded. The UI freezes
settings inputs while draining saves and invoking installation. Closing an edited source,
leaving settings, changing the selected source or quitting through the UI requires explicit
discard; Escape in that dialog keeps the edits. In-flight form saves cannot be discarded.
The registry holds only field IDs and dirty/pending booleans, never draft text or keys.
An external settings event updates clean fields but preserves local edits.

## Window protocol

- The shell hides the flyout on blur (with a ~250 ms re-open guard so the tray click that
  re-opens it does not immediately close it) and on `Esc` pressed in the webview (`hide_flyout`).
- The webview owns its height: after each render it measures its content and calls
  `set_flyout_height`. Width is fixed at 432 px (27 rem). When the content is taller than the work
  area the panel body scrolls; the header and the surface switch stay.
- First launch only (`general.firstRunDone = false`): the shell opens the flyout once and then
  sets the flag through `save_settings`.

## View semantics

- Order of the views (and of the rows): `claude`, `codex`, `antigravity`, `opencode`, `kimi`, `zai`, `openrouter`, `deepseek`, `deepinfra`, `moonshot`, `novita`, `ollama`, `minimax`, `litellm` (`HarnessId::ALL`).
- `state`: `disabled` (user switched it off) | `needs_consent` (nothing is read until the user
  agrees) | `needs_setup` (configuration missing: LiteLLM URL/key, OpenCode key and no local
  consent, Kimi Code, z.ai, OpenRouter, DeepSeek, DeepInfra, Moonshot, Novita, Ollama Cloud or MiniMax key) | `loading` (first read in flight) | `ready` | `no_data` (the source works but has
  nothing yet) | `error`.
- A `ready` view with `stale = true` keeps its windows and is shown desaturated (gauges faded, text keeps its contrast) and excluded from
  the tray summary. `error` is set in the `error` state and also on a stale `ready` view whose last
  refresh failed (the windows are the previous good ones).
- `usedPct = null` is unknown and is never rendered as 0 %. Over-budget values above 100 are
  allowed and clamp to 0 % available. A window whose `resetsAt` passed is expired: shown as
  "reiniciou", no status, no pace.
- `nextAttemptAt` (epoch seconds or `null`) is set only while a rate-limit cool-down holds the
  source back (see Polling). The view keeps its last good windows, `stale = true` and the
  `rate_limited` error; the UI says "nova tentativa em N min". Any later result clears it.
- `readAt` is when the data was produced (file mtime, response time), not when we polled; the UI
  shows "lido há N min".
- Status and pace come only from the shared rules (`usage-model::status`, `src/lib/usage.ts`).
- `meta` is display-only and never holds secrets (keys below).
- `facts` are small labelled numbers (`id` is a message key; `unit` is `usd`, `tokens`,
  `requests`, `credits`, `epoch_s` (unix seconds, rendered as a date) or a free label).

### Wire vocabulary

What the collectors really send, so the UI can word it. `src/test/vocabulary.ts` mirrors these
lists and `src/i18n/emitted.test.ts` reads the collectors' Rust sources, so a new value that the
UI cannot word turns `pnpm test` red.

Window ids and kinds, per source:

| Source | Window `id` | `kind` |
|---|---|---|
| `claude` | `five_hour`, `weekly`; `spend_limit` (a money cap with `amount`, when the status line reports one) | `five_hour`, `weekly`; `budget` |
| `claude` (`claude /usage` or OAuth, opt-in) | `<family>:weekly` (`group` = the model family, e.g. `opus`, `fable`; `label` the display name for families the UI does not name) | `weekly` |
| `codex` | by cadence: `five_hour`, `daily`, `weekly`, `monthly`; a cadence it cannot name stays `primary` / `secondary` with `windowMinutes` set | same, else `other` |
| `antigravity` | `<pool>:five_hour`, `<pool>:weekly`, `<pool>:<slug>` (`label` = the bucket); pool is `gemini`, `claude_gpt`, a slug from the service, or `group_<n>` (0-based index) | `five_hour`, `weekly`, `other` |
| `opencode` | `rolling`, `weekly`, `monthly` | `five_hour`, `weekly`, `monthly` |
| `kimi` | `five_hour`, `weekly`, `monthly` (the body states the 5 h length; weekly and monthly have none, so no pace is computed for them); a stated 10080-minute `limits[]` entry is `limit_weekly` and is folded with the top-level `weekly` block into one window (the one that states a reset); other stated durations become `daily` or `window_<n>m` | `five_hour`, `daily`, `weekly`, `monthly`, `other` |
| `zai` | `five_hour`, `daily`, `weekly`, `window_<n>m` (length = number x unit); `mcp` (monthly tool allowance, no stated length) | `five_hour`, `daily`, `weekly`, `monthly`, `other` |
| `openrouter` | `key_limit` (a money cap with `amount` in `usd`; only when the key reports a positive `limit`; no `resetsAt`, the reset moment is not reported; the share can exceed 100 when the key overspent). A key without a limit has no window. | `budget` |
| `deepseek` | none: a balance has no percentage and no reset, so the view carries facts only | - |
| `deepinfra` | `spend_limit` (a money cap with `amount` in `usd`; only when the account reports a positive `limit`; no `resetsAt`). No limit, no window. | `budget` |
| `moonshot` | none: a balance has no percentage and no reset, so the view carries facts only | - |
| `novita` | none: a balance has no percentage and no reset, so the view carries facts only | - |
| `ollama` | current plan: `monthly` (money window with `amount` in `usd`, `resetsAt` = end of the period); legacy plan: `session` (kind `other`, no stated length) and `weekly`, percent only | `monthly`, `other`, `weekly` |
| `minimax` | per model bucket (`group` = the model name, at most 16): `<model_slug>:interval` (kind from the stated length: 300 min `five_hour`, 1440 min `daily`, else `other`; reset from the time-left field) and `<model_slug>:weekly` (no stated length or reset). A zero total means no such quota: no window, not 0 %. | `five_hour`, `daily`, `weekly`, `other` |
| `litellm` | `budget` (`label` = key alias); `limit:<duration>` (`label` = the raw duration); `model:<name>` (`group` and `label` = the model) | `budget` |

Fact ids and units:

| Source | Fact `id` (`unit`) |
|---|---|
| `codex` | `credits_balance` (`credits`); `reset_credits` (`resets`, the count of earned resets still available, `app_server` only; absent when the service gives no count) |
| `opencode` | `local_cost_5h`, `local_cost_7d`, `local_cost_30d` (`usd`); `local_tokens_5h`, `local_tokens_7d` (`tokens`). Local list-price estimate, not quota. |
| `openrouter` | `balance` (credits purchased minus used, only when `/credits` answered, which needs a management key); `usage_daily`, `usage_weekly`, `usage_monthly` (UTC day, Monday week, UTC month), `usage_total` (all `usd`) |
| `deepseek` | `balance_<cur>`, `granted_<cur>`, `topped_up_<cur>` with `<cur>` = `cny` or `usd` (the unit is the same word); one set per currency, never summed |
| `deepinfra` | `balance` (or `owed` when the account owes), `usage_recent`, `usage_month` (all `usd`; the unit of the first two is read from an undocumented unit, see data-sources) |
| `moonshot` | `balance` (available), `voucher`, `cash` (all `usd`; `cash` may be negative) |
| `novita` | `balance`, `cash` (may be negative), `credit_limit`, `pending`, `owed` (all `usd`, converted from 1/10000 dollar) |
| `ollama` | `included_balance` (current plans only), `purchased_balance` (both `usd`) |
| `minimax` | none: every figure is a quota share |
| `litellm` | `rpm_limit`, `max_parallel_requests` (`requests`); `tpm_limit` (`tokens`); `expires` (`epoch_s`) |

`meta` keys (display only): `alias`, `host`, `scope` (`key`, `user`, `team`) and `level` (`key`,
`user`, `team`: where the budget was found) for LiteLLM; `local_estimate = true` for OpenCode when
facts are a local estimate; `extra_source = cli_usage` (the `claude -p "/usage"` command) or `oauth_usage` (the fallback API) for
Claude when some windows come from the opt-in per-model sources.

Error params are non-secret and drive the wording: `reason` (per code: `install_failed`,
`tls_error`, `endpoint_changed`, `not_running = no_port`, `unauthorized = no_token`), `field`
(see below), `retryAfterSecs`, `httpStatus` and `redirect`. `host`, `phase`, `rpcCode`, `code` and
`where` are diagnostics for bug reports and are never shown to the person. On a notice,
`reason` is an error code except for `opencode_db_unreadable`, which has its own list.

### Notice codes (initial catalog)

`Notice.code` is free-form; the UI maps known codes to localized text and renders any unknown code
with a generic message that includes the code, so a missing translation is visible, not silent.

| Code | Severity | Meaning |
|---|---|---|
| `claude_waiting_first_session` | info | Shim installed, no Claude Code session has reported yet. |
| `claude_no_rate_limits` | warn | Claude Code reports no `rate_limits` (API-key billing or a plan without limits). |
| `claude_shim_outdated` | warn | The configured shim executable is missing; reinstall. |
| `claude_oauth_partial` | info | Per-model usage is on but both the CLI and the OAuth API failed; showing statusLine data only (`reason` param, the API's). |
| `codex_rollout_fallback` | info | `codex app-server` unavailable; reading the newest rollout (may be old). `reason` param: the snake_case error code that stopped the app-server (`binary_not_found`, `timeout`, ...). |
| `codex_no_limits` | warn | The signed-in Codex account reports no limits (API-key auth). |
| `antigravity_pool_unknown` | warn | A quota pool returned no data (`pool` param); unknown, not zero. |
| `litellm_no_budget` | info | The key/user/team has no budget cap. |
| `litellm_level_unreadable` | warn | The user or team follow-up failed, so the budget may live at a level that could not be read (`level` = `user` or `team`, `reason` = error code). `litellm_no_budget` is withheld then. |
| `litellm_windows_truncated` | info | More than 16 per-model or concurrent budget windows; the least consumed were dropped (`dropped` param = how many). |
| `opencode_window_idle` | info | 0 % with a full-length reset: the window starts at the next request (`window` param). |
| `opencode_rate_limited` | warn | A window reports `rate-limited` (`window` param). |
| `opencode_no_key` | info | Only the local estimate is shown; paste the Go API key for the real windows. |
| `opencode_local_estimate` | info | Facts are a local list-price estimate for this PC, not quota. |
| `opencode_db_unreadable` | warn | The local database could not be read or its schema changed (`reason` param: `invalid_path`, `not_found`, `permission`, `locked`, `not_a_database`, `schema`, `timeout`, `other`). |

New codes are added by the collector that emits them and reported at integration so the UI
catalog is updated in the same change.

## Error codes

`ErrorCode` (23 values, `contract/error-codes.json`). The UI maps each to a message and one
action (retry, open settings, open the console, reinstall). `retryable = true` means the next tick
may succeed by itself. Params are non-secret (`httpStatus`, `host`, ...).

### Field names in rejected commands

`save_settings`, `set_secret`, `install_claude_shim` and `get_history` reject with `invalid_url` or `invalid_input`
and `params.field` naming the offending input (never its value). The UI shows field-level wording for
the names marked UI; any other name gets the generic message. `install_claude_shim` also rejects a
relative or NUL-bearing data or home directory as `invalid_input` with `field = configDir`; the
contract has no better name for it.

| Command | Code | `field` | UI |
|---|---|---|---|
| `save_settings` | `invalid_url` | `baseUrl` (not https, userinfo, query or fragment; plain http only to loopback) | yes |
| `save_settings` | `invalid_input` | `entityId` (user/team scope without an id) | yes |
| `save_settings` | `invalid_input` | `authHeader` (letters, digits and hyphen only, up to 128; not a header the HTTP client sets itself) | yes |
| `save_settings` | `invalid_input` | `pemPath` (custom CA path not absolute, NUL, too long, or a UNC share, device-namespace or reserved device path) | yes |
| `save_settings` | `invalid_input` | `host` (insecure TLS host empty or different from the base URL host) | yes |
| `save_settings` | `invalid_input` | `configDir`, `binaryPath`, `dataDir` (custom path not absolute, NUL, too long, or a UNC share, device-namespace or reserved device path; the extended-length form `\\?\C:\` stays valid) | yes |
| `set_secret` | `invalid_input` | `value` (empty after trimming, or longer than 4096 characters) | yes |
| `install_claude_shim` | `invalid_input` | `shimConsent` (consent not given) | yes |
| `get_history` | `invalid_input` | `limit` (0 or above 8) | yes |

OpenCode: 401/403 from the usage endpoint are reported as `unauthorized` with `ambiguous = true`
because valid keys have been rejected by the service; the UI says "could not verify the key or the
Go subscription" and offers the console link, never a certain "bad key".
Kimi Code and z.ai follow the same rule (a z.ai failure can also arrive as `success: false` inside an HTTP 200 and is mapped by its `code`; the vendor `msg` is never shown). Their sources are `coding_usages` and `quota_limit`, both unofficial.
OpenRouter (`key_credits`) and DeepSeek (`user_balance`) are official and documented, so a 401/403 is a plain `unauthorized` (no `ambiguous`). A 402 has no code of its own and arrives as `bad_response` with `httpStatus = 402`; 429 is `rate_limited` with `retryAfterSecs` when given. DeepInfra (`payment_checklist`), Moonshot (`users_balance`), Novita (`balance_detail`) and Ollama Cloud (`balance`) are official too and follow the same rule (Ollama Cloud documents 403 as *account suspended*, which the app cannot tell from a bad key). MiniMax (`token_plan_remains`) is unofficial: a 200 with a non-zero `base_resp.status_code` is `bad_response`, a body without `model_remains` is `endpoint_changed`. For DeepInfra the checklist decides the poll and the usage call only adds `usage_month`. For OpenRouter only the key document decides the poll: a refused or failed `/credits` call (an ordinary key gets a 403) only leaves the balance fact out.

## Polling

- `general.intervalSecs` (30..3600, default 60) is the base cadence; it drives local sources (Claude shim file, Codex,
  Antigravity, OpenCode local database) and also Claude when per-model usage is on: the collector itself
  runs `claude -p "/usage"` no more than once per 600 s (failures 1800 s) and asks the OAuth endpoint, its
  fallback, no more than once per 300 s. LiteLLM uses `litellm.pollIntervalSecs`
  (>= 60, default 300). Scheduled OpenCode polls, which reach its Go endpoint, never run faster
  than 300 s; a manual refresh can (see the 10 s gap above).
- Failures back off exponentially per harness up to 15 minutes; a successful read resets it. A Codex
  rollout fallback counts as a success (no error on the view), so it does not back off.
- Rate-limit cool-down: an `error` view with code `rate_limited` leaves that harness alone for
  `max(retryAfterSecs or 300 s, ordinary backoff, normal cadence)`, held to 30 minutes
  (`timing::cooldown`). The hub keeps the last good view stale and sets `nextAttemptAt`. A manual
  `refresh` is dropped during it like `REFRESH_MIN_GAP`, and the UI says why; a forced read (the
  harness' own settings or secret changed) ends the cool-down.
- Adaptive cadence (`general.adaptive`, default on): after a healthy read the next wait is
  `base * (2 + unchanged reads in a row)`, held to 30 minutes and to 10 minutes before the
  nearest reset, never below `base` (`timing::adaptive_wait`). `base` applies as is when any live
  window has `usedPct` unknown, at or above 70 % (lower when the first alert threshold minus 10
  is lower), or resets within 10 minutes, and when no live window exists. A failure backs off
  from `base`, not from the stretched wait; a manual refresh and the OpenCode 300 s floor are
  untouched. Turning the switch off brings a pending read back to `base`.
- `timing::cadence` gives each harness its pace: `general.intervalSecs` for Claude, Codex and
  Antigravity, `max(intervalSecs, 300 s)` for OpenCode, Kimi Code, z.ai, OpenRouter, DeepSeek, DeepInfra, Moonshot, Novita, Ollama Cloud and MiniMax, `litellm.pollIntervalSecs` for LiteLLM.
- A manual `refresh` is dropped when that harness started a collection less than 10 s ago
  (`REFRESH_MIN_GAP`).
- Snapshot events: the webview registers the `snapshot` listener first, then calls
  `get_snapshot`; whichever finishes last, the UI keeps the larger `generatedAt`.
- At each start the shell refreshes the installed Claude shim copy from the bundled build. It is a
  silent step: no status, no notice and no IPC command.
- Disabled and consent-gated harnesses are never polled.
- The first snapshot after start has every enabled harness in `loading`.

## Notifications

`general.notifyOnOut` and `general.notifyOnReset` (both off by default) switch the desktop toasts;
`general.notifyOnRisk` (off by default) adds one toast per window epoch when the pace (`Pace::Risk`) projects the window
running out before it renews; a window first seen already at risk only sets the baseline.
`general.alertThresholds` (default `[80, 95]`, empty = off) switches the usage-threshold toasts.
`general.showUsed` (default `false`) is presentation only: the UI, the tray tooltip and the "limite disponível"
toast print the used share instead of the available one; every payload keeps `usedPct`, and the tray icon arc
stays on the available share.
`general.pinned` (default `[]`) lists providers the triage list keeps first, in pin order; presentation only.
`general.compact` (default `false`) is presentation only: the flyout lists use tighter spacing.

`general.language` (default `"en"`) is the language of the interface and of the desktop text the
Rust side produces (notifications, tray tooltip and menu). One of `en`, `zh-CN`, `hi`, `es`, `fr`,
`ar`, `bn`, `pt-BR`, `ru`, `ur`; an unknown value is rejected like any other enum. Presentation
only: it never changes what is measured.
`app-core` plans them from snapshot changes and the shell shows them; title and body are
ready-made localized text and carry no path, key or raw error.

- Window ran out: a fresh, non-expired window of a `ready` view goes from some availability to
  none. The first snapshot after start never notifies.
- Window available again: the window that was out is seen with availability, or, when its source
  stays silent (the Claude status line only moves while Claude Code runs), the clock announces it
  once at `resetsAt` ("limite disponível", without a percentage). A reading that later confirms
  that reset does not notify a second time.
- Usage threshold: a fresh, non-expired window with a known `usedPct` reaches a value in
  `general.alertThresholds` (1..99, at most 4, sorted and unique after `normalize`). One toast per
  crossing per window and reset epoch (state keyed by harness, window id and `resetsAt`, so a
  reset re-arms it); a jump over several thresholds tells only the highest; unknown, stale and
  expired readings never alert. The state lives in memory like the other notification state, so
  the first reading of a window after a start (or a switch-on) only sets the baseline and a
  restart never repeats a toast. A window with no `resetsAt` alerts once per level per run.
  Text carries the harness, the window name and numbers only. `NotificationKind::ThresholdCrossed`.
- Settings recovered: `settings.json` was unreadable, was kept as `settings.corrupt-<epoch>.json`
  and defaults are in effect. Shown once per start, before the first snapshot, whatever the two
  toggles say.

## Data directory

`%LOCALAPPDATA%\dev.aisessionmonitor.tray\` holds `settings.json` (non-secret), `claude\rate_limits.json`
(written by the shim, `capturedAt` epoch seconds plus the raw `rate_limits` object), and `bin\`
(the stable copy of the shim referenced from Claude Code's settings). Secrets are never written here.

`history.json` is the burndown store: per harness and window id and reset time, a list of
`[epoch seconds, used percentage]`. It holds no account, plan, label, path or error text. Rules, all
in `crates/app-core/src/history`: only fresh, non-stale readings with a known percentage and a
future reset are sampled (unknown is skipped, never stored as 0); a sample is kept when the value
changed or 10 minutes passed; a reset restated up to 2 minutes apart stays the same epoch;
readings older than 35 days go first, then the oldest beyond 20 000 points or 1 MiB. The file is
written atomically at most once a minute and again at shutdown. A file that fails to parse, is
over 2 MiB or has another `version` is renamed `history.corrupt.json` (one copy, replaced next
time) and the history starts empty: no notice, no error text. `general.historyEnabled = false`
stops recording and hides the history; it does not delete, `clear_history` does.
